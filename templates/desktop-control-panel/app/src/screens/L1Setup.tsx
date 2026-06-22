import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useSession } from "../store/useSession";
import L1PickerFields from "../components/L1PickerFields";
import "./L1Setup.css";

export default function L1Setup() {
  const setL1 = useSession((s) => s.setL1);
  const existingL1 = useSession((s) => s.l1);
  const existingVariety = useSession((s) => s.regionalVariety);

  const [l1, setL1Local] = useState(existingL1);
  const [variety, setVariety] = useState(existingVariety);
  const nav = useNavigate();

  const canStart = l1.trim().length > 0;

  return (
    <div className="screen l1-setup">
      <h1>Welcome to Reading Practice</h1>
      <p className="lede">
        A short, private practice exercise to help you sharpen your English
        pronunciation. Everything runs on this device.
      </p>

      <div className="card">
        <L1PickerFields
          l1={l1}
          variety={variety}
          onL1Change={setL1Local}
          onVarietyChange={setVariety}
          idPrefix="l1setup"
          autoFocus
        />

        <p className="privacy-note">
          We only ever store this preference. No name, email, or student ID is
          requested anywhere in the app.
        </p>

        <div className="actions">
          <button
            className="primary"
            disabled={!canStart}
            onClick={() => {
              setL1(l1.trim(), variety.trim());
              // Navigate to the root gate (not /passage directly) so the
              // FirstRunGate re-evaluates and routes an uncached model to
              // /model-download before the practice flow.
              nav("/");
            }}
          >
            Start
          </button>
        </div>
      </div>
    </div>
  );
}
