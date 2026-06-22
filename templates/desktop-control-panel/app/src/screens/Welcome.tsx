import { useNavigate } from "react-router-dom";
import "./Welcome.css";

/**
 * First-run welcome screen (CL-24). The entry point of the first-run flow:
 * Welcome → Consent → L1Setup → ModelDownload → practice. A brief, friendly
 * introduction to the read-aloud pronunciation-practice exercise with a single
 * call to action.
 */
export default function Welcome() {
  const nav = useNavigate();

  return (
    <div className="screen welcome-screen">
      <p className="welcome-kicker">P3 Platform</p>
      <h1>Practice your English pronunciation</h1>
      <p className="lede">
        Read a short passage aloud and get feedback on the sounds you struggle
        with the most.
      </p>

      <div className="card welcome-card">
        <ul className="welcome-points">
          <li>
            <span className="welcome-point-title">Read aloud</span>
            Read aloud a short passage to allow the tool to determine which
            sounds you're having the most difficulty with.
          </li>
          <li>
            <span className="welcome-point-title">Get specific guidance</span>
            Get specific, articulatory guidance on where to place your tongue,
            lips, and breath.
          </li>
          <li>
            <span className="welcome-point-title">Privacy</span>
            PII remains private — no audio, transcript, or personal details
            leave this device.
          </li>
        </ul>

        <div className="welcome-actions">
          <button className="primary" onClick={() => nav("/consent")}>
            Get started
          </button>
        </div>
      </div>
    </div>
  );
}
