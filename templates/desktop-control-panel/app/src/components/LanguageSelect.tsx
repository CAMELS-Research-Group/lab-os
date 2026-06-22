/**
 * Presentational, controlled language control for the Settings screen.
 *
 * Purpose-built replacement for the onboarding chip-picker (L1PickerFields):
 *  - a native <select> populated from L1_SUGGESTIONS, plus a trailing "Other…"
 *  - selecting "Other…" reveals a free-text input for a custom language
 *  - a separate optional dialect / regional-variety text input
 *
 * Controlled by `l1`: the dropdown selection and whether the free-text input is
 * shown are DERIVED from the `l1` prop (never from internal selection state), so
 * reopening Settings with a saved value — suggested OR custom — renders right.
 *
 * `idPrefix` keeps label/input id pairs unique (mirrors L1PickerFields).
 *
 * Styling lives in the Settings-context rules of `Settings.css`
 * (`.settings-screen .field-label`, `.language-select`, `.language-text-input`);
 * this component owns no CSS import of its own.
 */

import { useState } from "react";
import { L1_SUGGESTIONS } from "../data/l1Suggestions";

type Props = {
  l1: string;
  variety: string;
  onL1Change: (v: string) => void;
  onVarietyChange: (v: string) => void;
  idPrefix?: string;
};

// Sentinel select value for the "Other…" option (cannot collide with a real
// language name, which is what L1_SUGGESTIONS contains).
const OTHER_VALUE = "__other__";

export default function LanguageSelect({
  l1,
  variety,
  onL1Change,
  onVarietyChange,
  idPrefix = "language",
}: Props) {
  const selectId = `${idPrefix}-language-select`;
  const otherInputId = `${idPrefix}-language-other`;
  const varietyInputId = `${idPrefix}-variety-input`;

  // Case-insensitive match against the suggestions (same comparison the chip
  // `selected` logic used). A non-empty l1 that matches → that suggestion is
  // selected and the free-text input is hidden. A non-empty l1 with no match →
  // "Other…" is selected and the free-text input is revealed, populated with l1.
  const matched = L1_SUGGESTIONS.find(
    (s) => s.toLowerCase() === l1.toLowerCase()
  );

  // "Other…" can be active two ways: (1) derived — a non-empty l1 with no
  // suggestion match (the reopen-with-saved-custom-value case); or (2) the user
  // just picked "Other…" from the dropdown and hasn't typed yet (l1 is "", so
  // there's nothing to derive from — this latch remembers the explicit choice).
  const [otherChosen, setOtherChosen] = useState(false);
  const derivedOther = l1.trim().length > 0 && matched === undefined;
  const isOther = derivedOther || otherChosen;

  // Drive the native select's value from the derived/latched state. Empty l1
  // with no Other choice → the disabled placeholder option.
  const selectValue = matched ?? (isOther ? OTHER_VALUE : "");

  function handleSelectChange(value: string) {
    if (value === OTHER_VALUE) {
      // Latch the explicit "Other…" choice and clear l1 so Save stays disabled
      // until the learner types a custom value into the revealed input.
      setOtherChosen(true);
      onL1Change("");
    } else {
      setOtherChosen(false);
      onL1Change(value);
    }
  }

  return (
    <>
      <label className="field-label" htmlFor={selectId}>
        What is your first language?
      </label>
      <select
        id={selectId}
        className="language-select"
        value={selectValue}
        onChange={(e) => handleSelectChange(e.target.value)}
      >
        <option value="" disabled>
          Select your first language…
        </option>
        {L1_SUGGESTIONS.map((s) => (
          <option key={s} value={s}>
            {s}
          </option>
        ))}
        <option value={OTHER_VALUE}>Other…</option>
      </select>

      {isOther && (
        <>
          <label className="field-label" htmlFor={otherInputId}>
            Your first language
          </label>
          <input
            id={otherInputId}
            className="language-text-input"
            type="text"
            value={l1}
            onChange={(e) => onL1Change(e.target.value)}
            placeholder="Type your first language"
          />
        </>
      )}

      <label className="field-label optional" htmlFor={varietyInputId}>
        Regional variety or dialect (optional)
      </label>
      <input
        id={varietyInputId}
        className="language-text-input"
        type="text"
        value={variety}
        onChange={(e) => onVarietyChange(e.target.value)}
        placeholder="e.g., Mainland, Cantonese-influenced, Northern…"
      />
    </>
  );
}
