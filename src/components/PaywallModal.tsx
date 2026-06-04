import { useToast } from '../hooks';

interface PaywallModalProps {
  trigger: string | null;
  onDismiss: () => void;
}

export function PaywallModal({ trigger, onDismiss }: PaywallModalProps) {
  const { toast } = useToast();

  const triggerMessages: Record<string, string> = {
    free_stream_limit: "Free tier is limited to 3 Streams. Upgrade to Pro for unlimited Streams.",
    free_review_limit: "Free tier is limited to 1 Review session per day. Upgrade to Pro for unlimited Reviews.",
    free_device_limit: "Free tier is limited to 1 device. Upgrade to Pro to sync across multiple devices.",
  };

  if (!trigger) return null;

  const message = triggerMessages[trigger] || "You've reached a free tier limit. Upgrade to Pro for full access.";

  return (
    <div className="paywall-overlay" role="dialog" aria-modal="true" aria-label="Upgrade to Pro">
      <div className="paywall-modal">
        <h2>Upgrade to Pro</h2>
        <p>{message}</p>
        <div className="paywall-options">
          <div className="paywall-tier">
            <h3>Pro Monthly</h3>
            <p className="paywall-price">$8 / month</p>
            <ul>
              <li>Unlimited Streams</li>
              <li>Unlimited Reviews</li>
              <li>Multi-device sync</li>
              <li>Priority support</li>
            </ul>
          </div>
          <div className="paywall-tier">
            <h3>Pro Annual</h3>
            <p className="paywall-price">$80 / year</p>
            <ul>
              <li>All Pro features</li>
              <li>~2 months free vs monthly</li>
              <li>Early access to new features</li>
            </ul>
          </div>
        </div>
        <div className="paywall-actions">
          <button
            className="btn btn-primary"
            onClick={() => {
              toast({ message: 'Upgrade via the Settings panel to pay with card.', type: 'info' });
            }}
          >
            Upgrade Now
          </button>
          <button className="btn btn-secondary" onClick={onDismiss}>
            Maybe Later
          </button>
        </div>
      </div>
    </div>
  );
}
