# Handbook site: frontend & information-design round — design

Status: reviewed — approved by Watson 2026-06-12 (brainstorm session; all four visual choices and scope
decisions made live via the visual companion).

Builds on `2026-06-11-lab-os-rename-handbook-site-design.md` (the site's existence, white-label pivot,
journey/reference page distinction). That spec shipped the content MVP on the scaffold's default chrome;
this round replaces the chrome with a deliberate identity and finishes the deferred branding pass.

## 1. Problem

The handbook site (`site/`, Docusaurus 3.10, live at `watsonwblair.github.io/lab-os`) shipped its
content on scaffold defaults: the Docusaurus dinosaur logo and favicon, the Infima default green
palette, default typography, a flat seven-page sidebar, and a plain-markdown landing page. The
dinosaur reads as "unfinished scaffold" — a poor first impression for the testers Watson is about to
invite. The downloadable templates a tester copies during setup are still CAMELS-shaped, leaking lab
orientation into a white-label exercise. None of this is a content problem; it is identity, theme,
navigation chrome, and template neutrality.

## 2. Decisions

| # | Decision | Alternatives rejected |
|---|---|---|
| D1 | **Stacked-layers logo mark** (three offset rounded planes = the workspace stack), replacing the Docusaurus dinosaur | Terminal-prompt `>_` glyph (assumes a CLI-centric tester); spec-checkmark (ties the mark to one method, less brand-neutral) |
| D2 | **Indigo palette**, light + dark, replacing the scaffold green | Slate + amber (neutral base, warm accent); ink + electric blue (near-monochrome). Indigo harmonizes with the chosen mark and reads as deliberate, not a recolor |
| D3 | **Inter** for headings and body; **JetBrains Mono** for code | Space Grotesk headings (more character, slight mismatch with a tool audience); Fraunces serif headings (editorial, at odds with the terminal-tool reader). Inter disappears so content leads |
| D4 | **Grouped sidebar**: Get started / Reference, current order preserved | Keep flat (no wayfinding between journey and reference pages) |
| D5 | **Card-driven landing page**: hero + outcome cards + section cards | Minimal hero with prose lists (lighter, but the storefront is the first thing a tester sees and earns the richer front door) |
| D6 | **Genericize templates fully** to `<your-lab>`/`<your-project>` placeholders | Lab-truthful reference examples (copies CAMELS orientation into a tester's repo); two variants (double maintenance); defer (leaves the leak through launch) |
| D7 | **Full gate**: all of the above ships before testers are invited | Branding-only gate; no gate. Watson chose to launch onto a fully finished surface |
| D8 | **Four PRs**, one concern each: branding · theme · IA+landing · templates | Three PRs (branding folded into theme); finer splits. Four keeps branding assets and CSS theme as distinct concerns |

## 3. Identity / branding

- A single source `logo.svg` rendering the stacked-layers mark in the indigo primary, legible in the
  navbar at full size and at 16px.
- A regenerated `favicon.ico` derived from the same mark.
- The dinosaur `logo.svg` and scaffold `favicon.ico` in `site/static/img/` are retired.
- Navbar and footer keep the `lab-os` wordmark set in Inter.

**Binary-rule note:** `favicon.ico` is a small binary. The repo already commits one (the scaffold's),
so this replaces an established exception rather than introducing a new binary class. The spec states
this explicitly so a reviewer does not flag it against `02-data-protection.md`'s binary rule. The
logo is SVG (text), not a binary.

## 4. Theme (color + typography)

- The indigo palette replaces the scaffold green across light and dark in `src/css/custom.css`:
  primary, link, button, code-block accent, and the mark color, with dark-mode lightening that keeps
  contrast legible.
- Inter for headings and body; JetBrains Mono for code.
- **Fonts are self-hosted with no third-party CDN request and no committed font binaries.** The
  implementing approach uses the Fontsource npm packages (added to `site/package.json`, imported via
  CSS) so the `.woff2` files live in `node_modules`, never in git — respecting the binary rule. The
  spec states the constraint (self-hosted, no committed binaries); the implementer chooses the exact
  import wiring.

## 5. Information architecture

- **Sidebar** (`sidebars.ts`): the flat seven-page list becomes two groups, preserving current order.
  - **Get started**: Getting started · Working with Claude · The onboarding project
  - **Reference**: The rules, explained · Setting up a new repo · How to play-test · Tooling tour
- **Landing page** (`src/pages/index.mdx`): rebuilt as the card-driven layout — a hero with primary
  and secondary CTA, three outcome cards ("What you end up with"), and two clickable section cards
  ("Get started" / "Reference"). Small reusable card components land in the currently-empty
  `src/components/`.

**Boundary held:** this round is navigation and landing chrome only. No doc-page *bodies* are
restructured — that stays gated on play-test friction data per the prior spec. Heading text inside
doc pages is untouched, so the `PR-LIFECYCLE.md` and `TROUBLESHOOTING.md` anchor citations into
`working-with-claude.md` remain valid.

## 6. Templates white-labeling

A 2026-06-12 audit corrected the handoff premise: **only `dev-root-CLAUDE.template.md` carries CAMELS
proper nouns.** `global-CLAUDE.template.md` and the rest are already name-clean (placeholder-driven, no
CAMELS/persona/dataset names); their only remaining lab-specificity is *framing* ("the lab", "lab-wide",
"lab repos", the beneficial-AI mission). The work splits accordingly:

- **`dev-root-CLAUDE.template.md` — structural skeleton.** This file is inherently about one workspace's
  repos (research lineage, repo inventory, datasets), so there is no name-swap version. Keep the section
  headings (workspace orientation, active repos, tooling, logs, approval gates) and replace all content
  with `<your-project>`-style placeholders plus one-line guidance on what each section captures. No
  CAMELS proper nouns; nothing fictional invented.
- **`global-CLAUDE.template.md` — de-lab-frame, philosophy verbatim.** No proper nouns to strip. Soften
  lab-as-a-specific-org references ("the lab" → "your team / your org"; "lab repos" → "your repos";
  "lab spend gates"/"gated-dataset rules" → "your spend gates"/"your data-protection rules"). Replace
  the beneficial-AI *mission* line with a neutral placeholder for the team's own guiding principle, but
  keep the transferable *mechanism* (tool-design lens, flag-don't-gate) and all other operating
  philosophy (PRD-first, pushback, reversibility, plan format, note-taking) verbatim.
- **`repo-CLAUDE.template.md`, `PRD.template.md`, `project_log.template.md`, `work-bundle/` — audit and
  de-frame in-place.** Already name-clean; apply the same "your team/org" softening to any residual lab
  framing found. Leave files with none unchanged. `project_log.template.md`'s normative structure (per
  `03-logging.md`) is preserved — only example content is touched.

The Getting Started bootstrap prompt already drives tester personalization from these templates, so this
removes the lab framing without changing the prompt's structure.

## 7. Launch gate & sequencing

Full gate: sections 3–6 ship before testers are invited. Sequencing as four single-concern PRs, each
approved by Watson before create and before merge:

1. **Branding** — `logo.svg` + `favicon.ico`, retire the dinosaur assets.
2. **Theme** — indigo palette + Inter/JetBrains Mono via Fontsource.
3. **IA + landing** — sidebar grouping + card-driven landing + card components.
4. **Templates** — genericize all templates in-place.

The play-test-launch log entry (lab altitude) is written when Watson actually invites testers, after
these merge — not in this round.

## 8. Out of scope

Doc-page body rewrites; new pages; restructuring existing page content; changes to the bootstrap
prompt's steps; anything gated on friction data.

## 9. Verification

Per PR and at round close:

- `cd site; npm run build` (unpiped) — must pass; `onBrokenLinks: 'throw'` catches broken nav.
- `python scripts/docs_budget.py --root .` — must pass.
- `log_lint` only if `project_log.md` changes (it does not in this round).
- Visual confirmation via the local preview (`LAB_OS_EDIT_LOCAL=1 npm run start`) and PR screenshots
  (Playwright against the local server) for the branding, theme, and landing PRs.

## 10. Known gaps

- Exact indigo hex values, dark-mode lightening steps, and the final logo SVG geometry are
  implementation details, settled during the branding/theme PRs against the approved directions, not
  pre-specified here.
- The section 6 audit is done: only `dev-root-CLAUDE.template.md` carries CAMELS proper nouns; the rest
  need framing softened, not names scrubbed. PR 4's per-file work follows §6 directly.
