# tools

Repo tooling. Each script is self-contained and run from the repo root.

## `build_deck.py` — Building-With-Claude kickoff deck

Builds the kickoff slide deck from its markdown source, so the deck is edited in
the markdown and the `.pptx` falls out — one source of truth, no hand-editing the
binary.

- **Source of truth:** `docs/workshops/building/kickoff-deck.md` (facilitator slide
  copy + speaker notes). Edit the deck there.
- **Output:** `docs/workshops/building/building-with-claude-kickoff.pptx`
  (gitignored — renders are never committed).

```bash
python tools/build_deck.py                  # src + out at their defaults
python tools/build_deck.py --out deck.pptx  # custom output path
python tools/build_deck.py path/to/deck.md  # custom source
```

Requires `python-pptx` (`pip install python-pptx`).

**Export to ODP** (optional, needs LibreOffice):

```bash
soffice --headless --convert-to odp --outdir docs/workshops/building \
  docs/workshops/building/building-with-claude-kickoff.pptx
```

### How it reads the markdown

The parser maps the deck's existing structure — no separate data format:

- `### S<n> · <title>` starts a slide; `S1 · Title` is the title slide and
  `S<n> · Divider — …` is an act divider; everything else is a content slide.
- Under **On-slide**, a leading `*italic*` item is the slide's lede, a
  whole-item `**bold**` is the payoff caption, numbered items render without
  bullets, and `` `code` ``/inline `**bold**` are flattened to plain text.
- **Notes** becomes the speaker-notes pane.

The build is strict: it asserts the source parses to exactly 27 contiguous
slides and prints a per-slide overflow estimate. A `TIGHT` warning means a
slide's body may not fit — tighten its on-slide bullets (push nuance to the
notes) and rebuild.
