# Handbook Frontend & Information-Design Round — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Each task is a feature-slice with explicit Acceptance and Verification; git is authoritative for progress.

**Goal:** Replace the Docusaurus scaffold chrome (dinosaur logo, Infima green, default type, flat sidebar, plain-markdown landing) with a deliberate indigo + stacked-layers identity, group the navigation, rebuild the landing page, and genericize the downloadable templates — finishing the deferred branding pass before testers are invited.

**Architecture:** Four single-concern PRs against `main`, each independently buildable: (1) branding assets, (2) CSS theme + self-hosted fonts, (3) information architecture + landing page, (4) template white-labeling. The first three touch `site/`; the fourth touches `templates/`. The spec rides in PR 1.

**Tech Stack:** Docusaurus 3.10 (rspack/SWC "faster" toolchain), TypeScript config, Infima CSS variables, Fontsource (self-hosted webfonts), MDX + React components, SVG.

**Spec references** (read before claiming any task):
- [Design doc](../specs/2026-06-12-handbook-frontend-design-round-design.md) — the whole round
- [01-workflow.md](../../../.claude/rules/01-workflow.md) — commit types, PR template, merge bar
- [02-data-protection.md](../../../.claude/rules/02-data-protection.md) — binary-asset rule (favicon note)

## How to consume a task

- **Files** lists exact paths. Stick to them; do not invent files unless the task says so.
- **Depends on** lists task numbers that must merge first.
- **Spec** links the design-doc section governing the task. Read it first.
- **Acceptance** is the bulleted behaviors the implementation must demonstrate — the implementer chooses exact markup, class names, and SVG geometry within them.
- **Verification** is the exact command(s) that must pass before opening the PR. `npm run build` is run **unpiped** (piping swallows the exit code).
- **Commit** is the conventional-commit subject for the PR title.

Per the lab merge bar: ask Watson before creating and before merging each PR. PR screenshots (branding/theme/landing) use Playwright against the local preview, published on a disposable `assets/pr-N` orphan branch and deleted after merge.

## PR / task map

| PR | Concern | Commit subject | Tasks |
|---|---|---|---|
| 1 | Branding (carries spec + this plan) | `feat(site): stacked-layers logo and favicon` | 1–2 |
| 2 | Theme (color + fonts) | `feat(site): indigo palette and self-hosted fonts` | 3–4 |
| 3 | IA + landing | `feat(site): grouped sidebar and card-driven landing` | 5–7 |
| 4 | Templates white-labeling | `docs(templates): white-label genericization` | 8–10 |

Soft ordering 1 → 2 → 3 → 4 (each builds green on its own; PR 3 looks correct only once PR 2's palette is live, so merge 2 before 3). The spec and this plan are committed in PR 1's branch as its first commit.

---

# PR 1 — Branding

## Task 1: Stacked-layers logo

**Context:** The mark is three offset rounded-square planes (the workspace stack) in the indigo primary; it replaces the Docusaurus dinosaur and must survive at favicon size.

**Files:**
- Create: `site/static/img/logo.svg` (overwrite the dinosaur file at this path)

**Depends on:** (none — first task; commit the design doc and this plan in the same branch first)

**Spec:** [§3 Identity / branding](../specs/2026-06-12-handbook-frontend-design-round-design.md#3-identity--branding)

**Acceptance:**
- `logo.svg` renders three offset, overlapping rounded-square planes with graduated opacity (back plane faintest, front plane solid), matching the approved direction C mock.
- The mark uses the indigo primary as its fill (anchor `#5b54e8`); it is a self-contained SVG with no external references.
- The composition reads clearly at 16px (favicon size) — planes remain distinguishable, no detail that collapses when scaled down.
- The SVG `viewBox` is square so the navbar and favicon renders are not distorted.
- No dinosaur artwork remains in the file.

**Verification:**
```bash
cd site && npm run build
```
Expected: build succeeds; load `build/index.html` preview and confirm the navbar shows the layers mark, not the dinosaur.

**Commit:** (part of PR 1 — no standalone commit subject; see Task 2)

---

## Task 2: Favicon and asset retirement

**Context:** Generate a favicon from the same mark and remove the scaffold favicon, so the browser tab matches the navbar identity.

**Files:**
- Create: `site/static/img/favicon.ico` (overwrite the scaffold file at this path)
- Modify: `site/docusaurus.config.ts` (logo `alt` text only, if it still says anything dinosaur-flavored)

**Depends on:** #1

**Spec:** [§3 Identity / branding](../specs/2026-06-12-handbook-frontend-design-round-design.md#3-identity--branding)

**Acceptance:**
- `favicon.ico` is derived from the stacked-layers mark and includes at least the 16px and 32px sizes.
- The browser tab shows the layers mark when the built site is served, not the dinosaur.
- `docusaurus.config.ts` logo `alt` reads `lab-os logo` (or equivalent neutral text); no dinosaur reference remains anywhere in config.
- The favicon is the only binary added; its presence is a deliberate replacement of the pre-existing committed scaffold favicon (noted per `02-data-protection.md` — not a new binary class). No other binary files are introduced.

**Verification:**
```bash
cd site && npm run build
python scripts/docs_budget.py --root .
```
Expected: both pass; served `build/` shows the new favicon in the tab.

**Commit:** `feat(site): stacked-layers logo and favicon`

---

# PR 2 — Theme (color + fonts)

## Task 3: Self-hosted Inter + JetBrains Mono

**Context:** Fonts must load with no third-party CDN request and no committed font binaries; Fontsource ships the `.woff2` files inside `node_modules`.

**Files:**
- Modify: `site/package.json` (add Fontsource dependencies)
- Modify: `site/src/css/custom.css` (font imports + family variables)

**Depends on:** (none — independent of PR 1; do not start before PR 1 merges only to keep PR order)

**Spec:** [§4 Theme (color + typography)](../specs/2026-06-12-handbook-frontend-design-round-design.md#4-theme-color--typography)

**Acceptance:**
- `@fontsource/inter` and `@fontsource/jetbrains-mono` are added to `site/package.json` dependencies and resolved via `package-lock.json`.
- Inter is applied to `--ifm-font-family-base` (headings and body); JetBrains Mono to `--ifm-font-family-monospace` (code).
- Fonts are served from the build output, not fetched from Google Fonts or any external host — the built site makes no request to `fonts.googleapis.com` or `fonts.gstatic.com`.
- No `.woff2`, `.woff`, or `.ttf` files are committed to git (they live in `node_modules`).
- Body text and code blocks visibly render in Inter and JetBrains Mono respectively in the built site.

**Verification:**
```bash
cd site && npm install && npm run build
```
Expected: build succeeds; grep the served HTML/CSS confirms no `googleapis`/`gstatic` references; `git status` shows no font binaries staged.

**Commit:** (part of PR 2 — see Task 4)

---

## Task 4: Indigo palette

**Context:** Replace the Infima default green with the approved indigo palette across light and dark, on every chrome surface.

**Files:**
- Modify: `site/src/css/custom.css` (`:root` and `[data-theme='dark']` color variables)

**Depends on:** #3

**Spec:** [§4 Theme (color + typography)](../specs/2026-06-12-handbook-frontend-design-round-design.md#4-theme-color--typography), [D2](../specs/2026-06-12-handbook-frontend-design-round-design.md#2-decisions)

**Acceptance:**
- The Infima primary green (`#2e8555` light / `#25c2a0` dark) is fully replaced by indigo; no green primary value remains in `custom.css`.
- Light mode uses an indigo primary anchored at `#5b54e8`; dark mode uses a lightened indigo anchored at `#8b85f5` for legible contrast on the dark background.
- The full Infima primary ramp (`--ifm-color-primary` plus `-dark`/`-darker`/`-darkest`/`-light`/`-lighter`/`-lightest`) is set coherently for both themes (monotonic lightness).
- Links, the primary button, the active sidebar item, and the code-block highlight accent all render indigo in both themes.
- Text/background contrast meets WCAG AA for body and link text in both themes.

**Verification:**
```bash
cd site && npm run build
python scripts/docs_budget.py --root .
```
Expected: both pass; local preview in light and dark shows indigo throughout, no residual green.

**Commit:** `feat(site): indigo palette and self-hosted fonts`

---

# PR 3 — Information architecture + landing

## Task 5: Grouped sidebar

**Context:** The flat seven-page list becomes two labeled groups matching the journey/reference split, with current order preserved.

**Files:**
- Modify: `site/sidebars.ts`

**Depends on:** (none structurally; merge after PR 2 per ordering)

**Spec:** [§5 Information architecture](../specs/2026-06-12-handbook-frontend-design-round-design.md#5-information-architecture), [D4](../specs/2026-06-12-handbook-frontend-design-round-design.md#2-decisions)

**Acceptance:**
- `handbookSidebar` is two categories: **Get started** containing `getting-started`, `working-with-claude`, `onboarding-project` (in that order); **Reference** containing `rules-explained`, `repo-setup`, `play-testing`, `tooling-tour` (in that order).
- No doc is dropped or reordered relative to the current flat list; all seven still appear.
- Category labels render in the sidebar; the build raises no broken-link or missing-doc error.
- No doc-page bodies or headings are edited (boundary held).

**Verification:**
```bash
cd site && npm run build
```
Expected: build succeeds (`onBrokenLinks: 'throw'` is satisfied); sidebar shows the two groups.

**Commit:** (part of PR 3 — see Task 7)

---

## Task 6: Landing card components

**Context:** The card-driven landing needs small reusable presentational components; they live in the currently-empty `src/components/`.

**Files:**
- Create: `site/src/components/OutcomeCard/index.tsx`
- Create: `site/src/components/OutcomeCard/styles.module.css`
- Create: `site/src/components/SectionCard/index.tsx`
- Create: `site/src/components/SectionCard/styles.module.css`

**Depends on:** #4

**Spec:** [§5 Information architecture](../specs/2026-06-12-handbook-frontend-design-round-design.md#5-information-architecture), [D5](../specs/2026-06-12-handbook-frontend-design-round-design.md#2-decisions)

**Acceptance:**
- `OutcomeCard` is a non-clickable presentational card taking a title and description, used for the three "What you end up with" items.
- `SectionCard` is a clickable card (wraps a Docusaurus `Link`) taking a title, description, and target href, used for the "Get started" / "Reference" navigation cards; the whole card is the click target and it shows an affordance (e.g., arrow) on hover.
- Both components style via CSS modules using Infima theme variables (so they inherit the indigo palette and adapt to light/dark automatically) — no hard-coded hex values that bypass the theme.
- Cards are responsive: they sit in a row on wide viewports and stack on narrow ones.
- Components are typed (TypeScript props interfaces) and import cleanly with no build warnings.

**Verification:**
```bash
cd site && npm run build
```
Expected: build succeeds with no TypeScript or MDX errors.

**Commit:** (part of PR 3 — see Task 7)

---

## Task 7: Rebuild the landing page

**Context:** Replace the plain-markdown landing with the approved card-driven layout (hero + outcome cards + section cards), reusing the Task 6 components and preserving the existing copy.

**Files:**
- Modify: `site/src/pages/index.mdx`

**Depends on:** #5, #6

**Spec:** [§5 Information architecture](../specs/2026-06-12-handbook-frontend-design-round-design.md#5-information-architecture), [D5](../specs/2026-06-12-handbook-frontend-design-round-design.md#2-decisions)

**Acceptance:**
- The page opens with a hero: the existing H1 ("Learn spec-driven development with an agentic workspace"), the existing intro sentence as a subhead, a primary CTA "Start here" → `/docs/getting-started`, and a secondary CTA "Working with Claude" → `/docs/working-with-claude`.
- "What you end up with" renders as three `OutcomeCard`s carrying the existing three outcome bullets (workspace / workflow / app) — wording preserved, not rewritten.
- A "Where to go" section renders two `SectionCard`s: "Get started" → `/docs/getting-started` and "Reference" → `/docs/rules-explained`, with one-line descriptions.
- All existing landing-page substance is retained or relocated into the new structure; nothing in the front-matter `title`/`description` is lost.
- The page builds as MDX with no bare `<token>` / comment / autolink pitfalls and no broken internal links.

**Verification:**
```bash
cd site && npm run build
python scripts/docs_budget.py --root .
```
Expected: both pass; local preview shows hero + cards rendering with the indigo theme in light and dark.

**Commit:** `feat(site): grouped sidebar and card-driven landing`

---

# PR 4 — Templates white-labeling

> **Audit result (2026-06-12):** Only `dev-root-CLAUDE.template.md` carries CAMELS proper nouns. The rest are already name-clean; their only lab-specificity is *framing*. Tasks below reflect that — Task 8 is a rewrite, Tasks 9–10 are framing edits, not name scrubs.

## Task 8: Rebuild the dev-root template as a structural skeleton

**Context:** A dev-root `CLAUDE.md` is inherently about one workspace's repos, so there is no name-swap version; keep the structure, replace the CAMELS content with placeholders and guidance.

**Files:**
- Modify: `templates/dev-root-CLAUDE.template.md`

**Depends on:** (none structurally; sequence last per PR order)

**Spec:** [§6 Templates white-labeling](../specs/2026-06-12-handbook-frontend-design-round-design.md#6-templates-white-labeling), [D6](../specs/2026-06-12-handbook-frontend-design-round-design.md#2-decisions)

**Acceptance:**
- The section headings are preserved as a reusable skeleton: workspace orientation rationale, a research-lineage / project-history section, an active-repos table, a tooling section, logs and tracking, and approval gates.
- All CAMELS content (the SSUR→Vibe→cs627→LSCA lineage, the repo inventory, LSCA/Global_Pathways/FCM_Analysis/Conversational_Agent, dataset names, Pace) is replaced with `<your-project>`-style placeholders and one-line guidance describing what each section captures.
- Nothing fictional is invented — placeholders and guidance only, not a fake worked example.
- The instructional blockquote at the top is preserved (de-CAMELS-ified) for the bootstrap prompt to strip; references to the global and per-repo template layers remain.
- No CAMELS proper nouns remain (verified by grep).

**Verification:**
```bash
grep -riE "camels|lsca|ssur|vibe_app|cs627|global_pathways|fcm_analysis|conversational_agent|iemocap|candor|mosei|pace" templates/dev-root-CLAUDE.template.md
```
Expected: no matches.

**Commit:** (part of PR 4 — see Task 10)

---

## Task 9: De-lab-frame the global template

**Context:** The global template is already name-clean and placeholder-driven; the only white-label work is softening lab-as-a-specific-org framing while keeping the transferable operating philosophy verbatim.

**Files:**
- Modify: `templates/global-CLAUDE.template.md`

**Depends on:** #8

**Spec:** [§6 Templates white-labeling](../specs/2026-06-12-handbook-frontend-design-round-design.md#6-templates-white-labeling)

**Acceptance:**
- Lab-as-a-specific-org references are softened to neutral team/org language: "the lab"/"lab-wide" → "your team"/"your team's"; "lab repos"/"the lab's public GitHub" → "your repos"/"your GitHub"; "lab spend gates"/"gated-dataset rules" → "your spend gates"/"your data-protection rules".
- The beneficial-AI *mission* line in Ethics becomes a neutral placeholder for the team's own guiding principle (e.g. `<your team's guiding mission>`), while the transferable *mechanism* (tool-design lens, flag-don't-gate) is kept.
- All other operating philosophy — Building anything (PRD-first), Plan writing, Pushback, Reversibility, Reviewing work, Note-taking, Working style, Model defaults, Memory system — is kept verbatim except for the same team/org wording swaps where "lab" appears.
- The About Me block stays the existing `<...>` placeholder scaffold (already neutral — no change needed beyond consistency).
- No CAMELS proper nouns are introduced; the file remains valid instructional Markdown.

**Verification:**
```bash
grep -riE "\bthe lab\b|lab-wide|lab repos|lab spend|gated-dataset|lab's (mission|public)" templates/global-CLAUDE.template.md
```
Expected: no matches (transferable philosophy text that does not reference the lab-as-org is untouched).

**Commit:** (part of PR 4 — see Task 10)

---

## Task 10: Audit and de-frame the remaining templates

**Context:** These are already name-clean; apply the same team/org softening to any residual lab framing and leave clean files untouched, so the whole `templates/` tree is white-label.

**Files:**
- Modify: `templates/repo-CLAUDE.template.md` (if lab framing found)
- Modify: `templates/PRD.template.md` (if lab framing found)
- Modify: `templates/project_log.template.md` (if lab framing found)
- Modify: `templates/work-bundle/` files (if lab framing found)

**Depends on:** #9

**Spec:** [§6 Templates white-labeling](../specs/2026-06-12-handbook-frontend-design-round-design.md#6-templates-white-labeling)

**Acceptance:**
- Each listed file is read; residual lab-as-org framing is softened to "your team / your org" language matching Task 9, preserving structure and instructional intent.
- Files containing no lab framing are left unchanged (the audit is allowed to find nothing in a given file).
- `project_log.template.md`'s normative structure (per `03-logging.md`, parsed by `log-lint`) is unchanged — only example/illustrative content is touched.
- A repo-wide grep across `templates/` returns no CAMELS proper nouns and no lab-as-org framing tokens.

**Verification:**
```bash
grep -riE "camels|lsca|ssur|vibe_app|cs627|global_pathways|iemocap|candor|mosei|watson|pace|\bthe lab\b|lab-wide|lab repos" templates/
python scripts/docs_budget.py --root .
```
Expected: no grep matches; docs_budget passes.

**Commit:** `docs(templates): white-label genericization`

---

## Self-review notes

- **Spec coverage:** §3 → Tasks 1–2; §4 → Tasks 3–4; §5 → Tasks 5–7; §6 → Tasks 8–10; §7 launch gate → PR sequencing in the map; §8 out-of-scope honored (no doc-body edits, Task 5 acceptance states the boundary); §9 verification → per-task Verification lines.
- **Anchors preserved:** no task edits headings inside `working-with-claude.md`, so `PR-LIFECYCLE.md` / `TROUBLESHOOTING.md` citations stay valid.
- **Binary rule:** Task 2 calls out the favicon as a deliberate replacement; Task 3 forbids committed font binaries.
