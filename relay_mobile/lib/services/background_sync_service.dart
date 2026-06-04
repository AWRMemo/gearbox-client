import '../src/rust/api/relay_api.dart' as api;
import 'package:sentry_flutter/sentry_flutter.dart';

class BackgroundSyncService {
  static const Duration minInterval = Duration(minutes: 15);

  /// Called by iOS BGAppRefreshTask or Android WorkManager to perform a sync.
  /// Returns true if sync completed, false if skipped (not authenticated).
  static Future<bool> performSync() async {
    try {
      final auth = await api.getAuthStatus();
      if (!auth.loggedIn) return false;

      final start = DateTime.now();
      await api.syncNow();
      final ms = DateTime.now().difference(start).inMilliseconds;

      await Sentry.captureMessage('background_sync_attempt: success, latency=${ms}ms');
      return true;
    } catch (e) {
      await Sentry.captureException(e, stackTrace: StackTrace.current);
      return false;
    }
  }

  /// Called when a silent push notification arrives (content-available: 1).
  static Future<void> onSilentPush() async {
    await performSync();
  }
}
