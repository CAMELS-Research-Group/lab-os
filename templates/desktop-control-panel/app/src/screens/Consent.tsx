import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useSession } from "../store/useSession";
import ConfirmDialog from "../components/ConfirmDialog";
import "./Consent.css";

/**
 * First-run privacy acknowledgment (CL-24, offline reviewer build).
 *
 * IMPORTANT — scope: this is the OFFLINE reviewer build. There is no backend.
 * "Consent" here is a LOCAL ACKNOWLEDGMENT ONLY: it records acceptance in the
 * Zustand store (persisted to localStorage) and does NOT call any backend,
 * does NOT call `accept_consent`, and does NOT do any Rust identity work. This
 * is a deliberate, scope-forced divergence from ADD §3.6's backend-registering
 * consent, which remains Phase B. The copy below reflects the real V1 privacy
 * posture: fully on-device, nothing transmitted.
 *
 * Two-step opt-in (#119): "Continue" opens a ConfirmDialog; only clicking
 * "I agree" inside it acknowledges consent and navigates to /setup.
 */
export default function Consent() {
  const nav = useNavigate();
  const acknowledgeConsent = useSession((s) => s.acknowledgeConsent);
  const [dialogOpen, setDialogOpen] = useState(false);

  const onConfirm = () => {
    setDialogOpen(false);
    acknowledgeConsent();
    nav("/setup");
  };

  return (
    <div className="screen consent-screen">
      <h1>How your privacy is protected</h1>
      <p className="lede">
        Before you start, here's exactly what this app does — and does not — do
        with your voice and your information.
      </p>

      <div className="card consent-card">
        <ul className="consent-list">
          <li>
            <span className="consent-list-title">Everything runs on this device.</span>
            All recording and analysis happen locally. There is no account, no
            sign-in, and no server.
          </li>
          <li>
            <span className="consent-list-title">Your voice never leaves.</span>
            No audio, transcripts, or sound embeddings are ever uploaded,
            transmitted, or stored off this device.
          </li>
          <li>
            <span className="consent-list-title">No personal information is collected.</span>
            No name, email, student ID, or other identifying detail is requested
            anywhere in the app.
          </li>
          <li>
            <span className="consent-list-title">You stay in control.</span>
            The only thing kept on this device is your first-language
            preference. You can clear it at any time.
          </li>
        </ul>

        <p className="consent-affirm">
          By continuing, you acknowledge that this practice tool works entirely
          on this device and transmits none of your data.
        </p>

        <div className="consent-actions">
          <button className="primary" onClick={() => setDialogOpen(true)}>
            Continue
          </button>
        </div>
      </div>

      <ConfirmDialog
        open={dialogOpen}
        title="Agree to continue"
        body="By agreeing, you confirm this practice tool works entirely on this device and transmits none of your data."
        confirmLabel="I agree"
        onConfirm={onConfirm}
        onClose={() => setDialogOpen(false)}
      />
    </div>
  );
}
