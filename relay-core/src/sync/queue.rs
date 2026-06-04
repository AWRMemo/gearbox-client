use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

type SyncFn = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

pub struct OfflineSyncQueue {
    pending: Mutex<Vec<QueuedSync>>,
    shutdown: AtomicBool,
    sync_fn: Mutex<Option<SyncFn>>,
    interval: Duration,
}

struct QueuedSync {
    attempt: u8,
    last_attempt: Instant,
}

impl OfflineSyncQueue {
    pub fn new() -> Arc<Self> {
        Self::new_with_interval(Duration::from_secs(30))
    }

    fn new_with_interval(interval: Duration) -> Arc<Self> {
        let queue = Arc::new(Self {
            pending: Mutex::new(Vec::new()),
            shutdown: AtomicBool::new(false),
            sync_fn: Mutex::new(None),
            interval,
        });

        let q = Arc::clone(&queue);
        thread::spawn(move || {
            while !q.shutdown.load(Ordering::SeqCst) {
                thread::sleep(q.interval);
                if q.shutdown.load(Ordering::SeqCst) {
                    break;
                }

                let ready_indices: Vec<usize> = {
                    let guard = q.pending.lock().unwrap();
                    let now = Instant::now();
                    guard
                        .iter()
                        .enumerate()
                        .filter_map(|(i, item)| {
                            let backoff = backoff_duration(item.attempt);
                            if now.duration_since(item.last_attempt) >= backoff {
                                Some(i)
                            } else {
                                None
                            }
                        })
                        .collect()
                };

                if ready_indices.is_empty() {
                    continue;
                }

                let sync_result = {
                    let guard = q.sync_fn.lock().unwrap();
                    match guard.as_ref() {
                        Some(f) => {
                            let f = Arc::clone(f);
                            drop(guard);
                            f()
                        }
                        None => {
                            // No sync function set yet; skip this cycle.
                            continue;
                        }
                    }
                };

                let mut guard = q.pending.lock().unwrap();
                if sync_result.is_ok() {
                    // Full sync succeeded — all pending items are resolved.
                    guard.clear();
                } else {
                    // Increment attempts for all ready items, remove expired.
                    for i in ready_indices.iter().rev() {
                        if let Some(item) = guard.get_mut(*i) {
                            item.attempt = item.attempt.saturating_add(1);
                            item.last_attempt = Instant::now();
                            if item.attempt >= 10 {
                                eprintln!("Permanent sync failure after 10 attempts");
                                guard.remove(*i);
                            }
                        }
                    }
                }
            }
        });

        queue
    }

    pub fn enqueue(&self) {
        let mut guard = self.pending.lock().unwrap();
        guard.push(QueuedSync {
            attempt: 0,
            last_attempt: Instant::now(),
        });
    }

    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }

    pub fn set_sync_fn<F>(&self, f: F)
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        let mut guard = self.sync_fn.lock().unwrap();
        *guard = Some(Arc::new(f));
    }

    #[cfg(test)]
    fn pending_len(&self) -> usize {
        self.pending.lock().unwrap().len()
    }

    #[cfg(test)]
    fn force_process_cycle(&self) -> Result<(), String> {
        let ready_indices: Vec<usize> = {
            let guard = self.pending.lock().unwrap();
            let now = Instant::now();
            guard
                .iter()
                .enumerate()
                .filter_map(|(i, item)| {
                    let backoff = backoff_duration(item.attempt);
                    if now.duration_since(item.last_attempt) >= backoff {
                        Some(i)
                    } else {
                        None
                    }
                })
                .collect()
        };

        if ready_indices.is_empty() {
            return Ok(());
        }

        let sync_result = {
            let guard = self.sync_fn.lock().unwrap();
            match guard.as_ref() {
                Some(f) => {
                    let f = Arc::clone(f);
                    drop(guard);
                    f()
                }
                None => return Ok(()),
            }
        };

        let mut guard = self.pending.lock().unwrap();
        if sync_result.is_ok() {
            guard.clear();
        } else {
            for i in ready_indices.iter().rev() {
                if let Some(item) = guard.get_mut(*i) {
                    item.attempt = item.attempt.saturating_add(1);
                    item.last_attempt = Instant::now();
                    if item.attempt >= 10 {
                        eprintln!("Permanent sync failure after 10 attempts");
                        guard.remove(*i);
                    }
                }
            }
        }

        sync_result
    }
}

fn backoff_duration(attempt: u8) -> Duration {
    let secs = if attempt >= 4 {
        300
    } else {
        30 * (1u64 << attempt)
    };
    Duration::from_secs(secs)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_backoff_formula() {
        assert_eq!(backoff_duration(0), Duration::from_secs(30));
        assert_eq!(backoff_duration(1), Duration::from_secs(60));
        assert_eq!(backoff_duration(2), Duration::from_secs(120));
        assert_eq!(backoff_duration(3), Duration::from_secs(240));
        assert_eq!(backoff_duration(4), Duration::from_secs(300));
        assert_eq!(backoff_duration(5), Duration::from_secs(300));
        assert_eq!(backoff_duration(10), Duration::from_secs(300));
    }

    #[test]
    fn test_enqueue_and_pending() {
        let queue = OfflineSyncQueue::new();
        assert_eq!(queue.pending_len(), 0);
        queue.enqueue();
        assert_eq!(queue.pending_len(), 1);
        queue.enqueue();
        assert_eq!(queue.pending_len(), 2);
    }

    #[test]
    fn test_success_clears_queue() {
        let queue = OfflineSyncQueue::new_with_interval(Duration::from_secs(1));
        let calls = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&calls);
        queue.set_sync_fn(move || {
            c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        // Inject a ready item (backoff elapsed)
        {
            let mut guard = queue.pending.lock().unwrap();
            guard.push(QueuedSync {
                attempt: 0,
                last_attempt: Instant::now() - Duration::from_secs(400),
            });
        }

        queue.force_process_cycle().unwrap();
        assert_eq!(queue.pending_len(), 0);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_failure_increments_attempt() {
        let queue = OfflineSyncQueue::new_with_interval(Duration::from_secs(1));
        queue.set_sync_fn(|| Err("network error".to_string()));

        {
            let mut guard = queue.pending.lock().unwrap();
            guard.push(QueuedSync {
                attempt: 0,
                last_attempt: Instant::now() - Duration::from_secs(400),
            });
        }

        let r = queue.force_process_cycle();
        assert!(r.is_err());
        assert_eq!(queue.pending_len(), 1);
    }

    #[test]
    fn test_max_attempts_removes_item() {
        let queue = OfflineSyncQueue::new_with_interval(Duration::from_secs(1));
        let calls = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&calls);
        queue.set_sync_fn(move || {
            c.fetch_add(1, Ordering::SeqCst);
            Err("network error".to_string())
        });

        // Manually inject an item with attempt=9, last_attempt far in the past
        {
            let mut guard = queue.pending.lock().unwrap();
            guard.push(QueuedSync {
                attempt: 9,
                last_attempt: Instant::now() - Duration::from_secs(400),
            });
        }

        queue.force_process_cycle().unwrap_err();
        assert_eq!(queue.pending_len(), 0);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_not_ready_item_stays_pending() {
        let queue = OfflineSyncQueue::new_with_interval(Duration::from_secs(1));
        let calls = Arc::new(AtomicUsize::new(0));
        let c = Arc::clone(&calls);
        queue.set_sync_fn(move || {
            c.fetch_add(1, Ordering::SeqCst);
            Ok(())
        });

        // Inject item with attempt=1 (backoff=60s) and last_attempt just now
        {
            let mut guard = queue.pending.lock().unwrap();
            guard.push(QueuedSync {
                attempt: 1,
                last_attempt: Instant::now(),
            });
        }

        queue.force_process_cycle().unwrap();
        assert_eq!(queue.pending_len(), 1);
        assert_eq!(calls.load(Ordering::SeqCst), 0);
    }
}
