# Handbook authoring backbone — conventions for the content + IA rework — design

Status: reviewed — approved by Watson 2026-06-13 (brainstorm session).

First sub-project of a larger handbook content + information-architecture rework (review feedback,
2026-06-13). That rework spans all seven handbook pages plus two conceptual reframes (Working with
Claude → SDD lifecycle; Onboarding Project → Onboarding Workshop). This spec settles the **cross-cutting
authoring conventions** every page round will apply, so per-page rounds don't re-litigate them.

Builds on `2026-06-12-handbook-frontend-design-round-design.md` (chrome/IA round). That round
deferred page-content restructuring to play-test friction data; **this rework supersedes that deferral
by Watson's decision (2026-06-13) and now gates tester launch** — testers see the reworked content, not
the current pages.

## 1. Problem

The handbook content needs a substantial rework before testers arrive, and it spans seven pages. Three
concerns recur on every page: pages overload skimmers instead of letting them chase depth on demand; it
is unclear whether a command goes in the terminal or an active Claude session; and zero-technical-skill
users hit unexplained prerequisites. Deciding these per page would produce inconsistent structure,
command marking, and onboarding depth. They must be fixed once, up front.

## 2. Decisions

| # | Decision | Alternatives rejected |
|---|---|---|
| D1 | **Hybrid depth model**: section overview page + nested sub-pages for substantial depth + inline `<details>` for short asides | Sub-pages for all depth (file sprawl, over-clicking); collapsibles only (long pages, flat sidebar) |
| D2 | **Code-block titles** mark command context: `title="Terminal"` vs `title="Claude session"`, plus a one-time legend | Admonition callouts (verbose, repetitive); custom badge component (build/maintain cost) |
| D3 | **Zero-tech: link out + fill gaps** — link canonical install/signup docs; write a lab sub-page only where the official path is confusing or has lab-specific steps | Own full walkthroughs (screenshot rot, high maintenance); link-only (zero-tech users still get lost) |
| D4 | Backbone ships a **living authoring-conventions reference** in the repo, alongside this spec | Spec-only (contributors lack a working reference outside the design doc) |
| D5 | Per-page sub-page inventories are **deferred to each page's own round** | Specify all sub-pages now (depends on per-page content design not yet done) |

## 3. Depth model (D1)

- **File structure:** each handbook section is a folder — `docs/<section>/index.md` (the skimmable
  overview) plus `docs/<section>/<subtopic>.md` for depth. Docusaurus renders the section as an
  expandable sidebar category whose label links to the overview (category index).
- **Overview carries the skimmable core;** sub-pages are linked inline from it ("→ Full walkthrough",
  "→ Deep dive"). Skimmers stop at the overview; depth-seekers click in.
- **Sub-page vs. collapsible test:** sub-page if it is a self-contained procedure or topic someone
  would land on or link to directly (a from-scratch install, a plugin deep-dive, a lifecycle stage);
  inline `<details>` collapsible if it is a 2–5 sentence "new to this?" aside tied to one spot.
- **Migration note:** existing single-file pages (`getting-started.mdx`, etc.) become
  `getting-started/index.mdx` as their rounds land. Sidebar entries move from flat doc IDs to category
  blocks. Heading anchors inside `working-with-claude` are cited by `PR-LIFECYCLE.md` /
  `TROUBLESHOOTING.md`; any page round that moves or renames those headings updates the referrers in the
  same commit (carried as a constraint into each page round, not handled here).

## 4. Command marking (D2)

- Shell/terminal commands: fenced code block with `title="Terminal"`.
- Input to an active Claude session (prompts, slash-commands such as `/plugin`, `/init`): fenced block
  with `title="Claude session"`.
- A **one-time legend** appears near the top of any page that mixes both (at minimum Getting Started),
  explaining the two titles. Uses Docusaurus's built-in `@theme/CodeBlock` title support — no new
  component.
- Applies retroactively: when a page round touches a code block, it gets the correct title.

## 5. Zero-tech support (D3)

- Link to canonical install/signup docs (Git, GitHub CLI) rather than re-documenting them.
- Write a lab "from scratch" sub-page (per the D1 depth model) only where the official path is confusing
  or has lab-specific steps (e.g. `gh auth login` + `gh auth setup-git`).
- Screenshots are allowed in these sub-pages where they materially help, accepting they need refresh as
  tool UIs change; prefer text + links where a screenshot would just rot.

## 6. Authoring-conventions reference (D4)

A short, living contributor reference at `site/AUTHORING.md` (ENG-tier, source of truth for *how to
author handbook pages*), capturing D1–D3 as actionable rules with copy-paste examples (folder layout,
code-block title syntax, the legend snippet, the sub-page-vs-collapsible test). This spec owns the
*decisions*; `site/AUTHORING.md` is the working reference contributors consult — it links back here as
the rationale source. Page rounds keep it current when a convention evolves.

## 7. How page rounds consume this

Each subsequent page round (Getting Started first, since it exercises all three conventions) cites this
spec and `site/AUTHORING.md`, then designs its own sub-page inventory and content. Sequencing across the
full rework is set when the first page round is scoped; this spec only fixes the shared pattern.

## 8. Scope

**In:** the three conventions (D1–D3); the authoring reference (`site/AUTHORING.md`); this spec.

**Out (own rounds):** any page's content rework or sub-page inventory; the SDD-lifecycle and
Onboarding-Workshop reframes; the sidebar restructure for nested categories (lands per page as each
section becomes a folder); screenshots themselves.

## 9. Verification

- `cd site && npm run build` (unpiped) passes — no broken links, MDX valid. (No page content changes
  this round, so the build is essentially a no-op guard; the real test is that `site/AUTHORING.md`
  exists, is internally consistent, and its examples are valid Docusaurus syntax.)
- `python scripts/docs_budget.py --root .` passes.
- `site/AUTHORING.md` covers D1–D3 with a copy-paste example for each and links to this spec.

## 10. Known gaps

- The per-section folder/sub-page inventories are intentionally unspecified here (D5) — each page round
  defines its own.
- Whether `getting-started.mdx` → `getting-started/index.mdx` migration causes any deploy-path change
  (the URL stays `/docs/getting-started`) is confirmed during the Getting Started round, not here.
