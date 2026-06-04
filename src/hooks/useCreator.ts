import { useState, useEffect, useCallback } from 'react';

interface CreatorProfile {
  user_id: string;
  stripe_connect_account_id: string | null;
  is_verified: boolean;
  display_name: string | null;
  bio: string | null;
  platform_fee_percent: number;
  created_at: string;
}

interface MonetizedStream {
  stream_id: string;
  creator_id: string;
  monthly_price_cents: number;
  subscriber_count: number;
  is_active: boolean;
}

interface CreatorAnalytics {
  subscriber_count: number;
  monthly_revenue_cents: number;
  stream_count: number;
}

async function loadCreatorState() {
  try {
    const { invoke } = await import('@tauri-apps/api/core');
    const [profile, streams, analytics] = await Promise.all([
      invoke('get_creator_profile').catch(() => null),
      invoke('list_monetized_streams').catch(() => [] as MonetizedStream[]),
      invoke('get_creator_analytics').catch(() => null),
    ]);
    return { profile: profile as CreatorProfile | null, streams: streams as MonetizedStream[], analytics: analytics as CreatorAnalytics | null };
  } catch {
    return { profile: null, streams: [] as MonetizedStream[], analytics: null };
  }
}

export function useCreator() {
  const [profile, setProfile] = useState<CreatorProfile | null>(null);
  const [streams, setStreams] = useState<MonetizedStream[]>([]);
  const [analytics, setAnalytics] = useState<CreatorAnalytics | null>(null);

  useEffect(() => {
    loadCreatorState().then((s) => {
      setProfile(s.profile);
      setStreams(s.streams);
      setAnalytics(s.analytics);
    });
  }, []);

  const registerAsCreator = useCallback(async () => {
    const { invoke } = await import('@tauri-apps/api/core');
    const resp: { url: string } = await invoke('register_creator');
    window.open(resp.url, '_blank');
    const s = await loadCreatorState();
    setProfile(s.profile);
    setStreams(s.streams);
    setAnalytics(s.analytics);
  }, []);

  const monetizeStream = useCallback(async (streamId: string, priceCents: number) => {
    const { invoke } = await import('@tauri-apps/api/core');
    await invoke('monetize_stream', { streamId, priceCents });
    const s = await loadCreatorState();
    setProfile(s.profile);
    setStreams(s.streams);
    setAnalytics(s.analytics);
  }, []);

  return { profile, streams, analytics, registerAsCreator, monetizeStream };
}
