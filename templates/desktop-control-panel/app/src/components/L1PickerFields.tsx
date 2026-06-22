/**
 * Presentational component — the reusable L1 / regional-variety picker fields.
 *
 * Renders the "What is your first language?" text input, suggestion chips, and
 * the optional "Regional variety or dialect" input. It does NOT include the
 * heading, privacy note, CTA button, or any outer card wrapper — those are the
 * caller's responsibility (L1Setup and Settings both wrap this differently).
 *
 * `idPrefix` keeps input id / htmlFor pairs unique when the same fields appear
 * on two screens simultaneously in the DOM (L1Setup passes "l1setup", Settings
 * passes "settings").
 */

import { L1_SUGGESTIONS } from "../data/l1Suggestions";
import "../screens/L1Setup.css";

type Props = {
  l1: string;
  variety: string;
  onL1Change: (v: string) => void;
  onVarietyChange: (v: string) => void;
  idPrefix?: string;
  /** Focus the L1 input on mount. Right for the first-run screen; off in
   *  Settings so opening the page doesn't grab focus / scroll to the field. */
  autoFocus?: boolean;
};

export default function L1PickerFields({
  l1,
  variety,
  onL1Change,
  onVarietyChange,
  idPrefix = "l1picker",
  autoFocus = false,
}: Props) {
  const l1InputId = `${idPrefix}-l1-input`;
  const varietyInputId = `${idPrefix}-variety-input`;

  return (
    <>
      <label className="field-label" htmlFor={l1InputId}>
        What is your first language?
      </label>
      <input
        id={l1InputId}
        className="text-input"
        type="text"
        value={l1}
        onChange={(e) => onL1Change(e.target.value)}
        placeholder="Type your answer or tap a suggestion below"
        autoFocus={autoFocus}
      />

      <div className="suggestions" aria-label="Common first-language suggestions">
        {L1_SUGGESTIONS.map((s) => (
          <button
            key={s}
            type="button"
            className={`chip ${l1.toLowerCase() === s.toLowerCase() ? "selected" : ""}`}
            onClick={() => onL1Change(s)}
          >
            {s}
          </button>
        ))}
      </div>

      <label className="field-label optional" htmlFor={varietyInputId}>
        Regional variety or dialect (optional)
      </label>
      <input
        id={varietyInputId}
        className="text-input"
        type="text"
        value={variety}
        onChange={(e) => onVarietyChange(e.target.value)}
        placeholder="e.g., Mainland, Cantonese-influenced, Northern…"
      />
    </>
  );
}
