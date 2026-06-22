import "./PhonemeBadge.css";

type Props = {
  ipa: string;
  size?: "sm" | "md" | "lg";
  flagged?: boolean;
};

// Static stand-in for the IPA sagittal diagrams from the TRD. We render the
// IPA glyph in a soft circle for now; real diagrams (CC BY-SA 3.0 sourcing)
// are deferred until V1 build-out.
export default function PhonemeBadge({ ipa, size = "md", flagged }: Props) {
  return (
    <span
      className={`phoneme-badge ${size} ${flagged ? "flagged" : ""}`}
      aria-label={`phoneme ${ipa}`}
    >
      <span className="ipa">{ipa}</span>
    </span>
  );
}
