import { useNavigate } from "react-router-dom";
import { useSession } from "../store/useSession";
import "./Welcome.css";

/**
 * First-run welcome + gate. The single entry point of the first-run flow:
 * a brief, friendly introduction plus the on-device privacy acknowledgment.
 *
 * RE-HOMED (P4 strip): the former multi-step first-run flow collapsed into
 * this one gate.
 * "Get started" records first-run completion in the session store (the local,
 * persisted flag the FirstRunGate reads) and hands off to the root gate, which
 * routes on to Home.
 */
export default function Welcome() {
  const nav = useNavigate();
  const completeFirstRun = useSession((s) => s.completeFirstRun);

  function getStarted() {
    completeFirstRun();
    // Navigate to the root gate (not /home directly) so the FirstRunGate
    // re-evaluates and routes from a single place.
    nav("/");
  }

  return (
    <div className="screen welcome-screen">
      {/* [DEFAULT]: replace with the app display name after forking. */}
      <p className="welcome-kicker">[DEFAULT]</p>
      <h1>Welcome</h1>
      <p className="lede">
        A private, on-device app. Here's exactly what it does — and does not —
        do with your information.
      </p>

      <div className="card welcome-card">
        <ul className="welcome-points">
          <li>
            <span className="welcome-point-title">Everything runs on this device</span>
            All processing happens locally. There is no account, no sign-in, and
            no server.
          </li>
          <li>
            <span className="welcome-point-title">Nothing leaves</span>
            No personal data is uploaded, transmitted, or stored off this device.
          </li>
          <li>
            <span className="welcome-point-title">Privacy</span>
            PII remains private — no personal details leave this device.
          </li>
        </ul>

        <div className="welcome-actions">
          <button className="primary" onClick={getStarted}>
            Get started
          </button>
        </div>
      </div>
    </div>
  );
}
