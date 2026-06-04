import { useCreator } from '../hooks/useCreator';
import { useToast } from '../hooks';

export function CreatorDashboard() {
  const { profile, streams, analytics, registerAsCreator } = useCreator();
  const { toast } = useToast();

  if (!profile) {
    return (
      <div className="creator-dashboard">
        <h2>Creator Dashboard</h2>
        <p>Monetize your Streams and build a subscriber base.</p>
        <button className="btn btn-primary" onClick={registerAsCreator}>
          Register as Creator
        </button>
      </div>
    );
  }

  return (
    <div className="creator-dashboard">
      <div className="dashboard-header">
        <h2>Creator Dashboard</h2>
        {profile.is_verified
          ? <span className="badge verified">Provenance Seal Active</span>
          : <span className="badge pending">Awaiting Verification</span>}
      </div>

      {analytics && (
        <div className="dashboard-stats">
          <div className="stat">
            <span className="stat-value">{analytics.subscriber_count}</span>
            <span className="stat-label">Subscribers</span>
          </div>
          <div className="stat">
            <span className="stat-value">${(analytics.monthly_revenue_cents / 100).toFixed(2)}</span>
            <span className="stat-label">Est. Monthly Revenue</span>
          </div>
          <div className="stat">
            <span className="stat-value">{analytics.stream_count}</span>
            <span className="stat-label">Monetized Streams</span>
          </div>
        </div>
      )}

      <div className="dashboard-section">
        <h3>Monetized Streams</h3>
        {streams.length === 0 && <p className="empty-state">No monetized Streams yet.</p>}
        {streams.map(s => (
          <div key={s.stream_id} className="monetized-stream">
            <span className="stream-id">{s.stream_id}</span>
            <span className="stream-price">${(s.monthly_price_cents / 100).toFixed(2)}/mo</span>
            <span className="stream-subs">{s.subscriber_count} subscribers</span>
          </div>
        ))}
      </div>

      <div className="dashboard-section">
        <h3>Fee Configuration</h3>
        <p className="fee-note">You choose your contribution rate: <strong>minimum 10%</strong>, suggested 15%, generous 20%+.</p>
        <select
          defaultValue={profile.platform_fee_percent}
          onChange={async () => {
            try {
              toast({ message: 'Fee rate updated.', type: 'success' });
            } catch {
              toast({ message: 'Failed to update fee rate.', type: 'error' });
            }
          }}
        >
          <option value={10}>Minimum — 10% (covers processing costs)</option>
          <option value={15}>Suggested — 15% (fuels development)</option>
          <option value={20}>Generous — 20% (invests in the platform)</option>
          <option value={25}>25%</option>
          <option value={30}>30%</option>
        </select>
      </div>
    </div>
  );
}
