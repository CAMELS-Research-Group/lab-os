# Data & Artifact Protection

Applies to every commit, PR, and generated output across all lab repos.

## Gated Datasets

Each repo declares its own gated-dataset table in its local `.claude/rules/10-data-protection.md` (per-repo prefixes, gate type, what's safe to commit; per-repo rules number from `10+` — see `04-docs.md`, Rules numbering). Universal rules across any gated data:

- Never commit raw samples, transcripts, or participant metadata
- Use only dataset prefixes and stem names in reports
- Dev-set fixtures use only synthetic or openly-licensed data
- Derived artifacts (plots, embeddings, coordinate dumps, summary JSONs) require explicit PII review before commit. Author MUST verify and reviewer MUST confirm every item below:
  - [ ] Speaker / participant identifiers replaced with anonymous integer indices (or removed entirely)
  - [ ] No raw transcript text, audio waveforms, or video frames embedded in the artifact
  - [ ] No file paths, stem names, or session IDs that could re-identify a clip
  - [ ] Only aggregated statistics, anonymised coordinates, or corpus-level labels remain
  - [ ] Per-point metadata (emotion, timestamps, etc.) audited — none of it, alone or combined, narrows to a single participant
- When in doubt, exclude the points, aggregate further, or omit the artifact

## Binary and Secret Protection

- File size limit: 5 MB per file (enforce via CI guard; allowlist test fixtures and dev-set directories)
- Never commit checkpoints (`.pt`), consolidated arrays (`.npy`), large model weights, or other binary artifacts
- Run secret-scanning in CI to catch tokens and credentials (gitleaks or equivalent)
- Auto-downloaded model files (`.task`, `.onnx`, `.bin`, `.safetensors`) are gitignored at repo root
