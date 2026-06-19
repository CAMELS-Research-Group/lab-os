# Getting Started Rework — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Git is authoritative for progress.

**Goal:** Rework the Getting Started page into a folder — a skimmable overview plus three zero-tech sub-pages — with every command marked Terminal vs Claude session, so a zero-technical-capability reader can complete setup without overloading skimmers.

**Architecture:** Convert `getting-started.mdx` to `getting-started/index.mdx` (URL unchanged) and add `terminal-basics.md`, `install-git.md`, `install-github-cli.md`. Apply the backbone conventions (`site/AUTHORING.md`): code-block titles + a one-time legend, link-out-plus-fill-gaps zero-tech, hybrid depth. Sub-pages nest under a collapsed sidebar category. Screenshots ship as marked placeholders, real images dropped in before merge.

**Tech Stack:** Docusaurus 3.10 MDX/Markdown, code-block `title=` support, `<details>`, category-index docs, `sidebars.ts`.

**Spec references** (read before claiming a task):
- [Design doc](../specs/2026-06-13-getting-started-rework-design.md)
- [Backbone conventions](../specs/2026-06-13-handbook-backbone-conventions-design.md) and `site/AUTHORING.md`
- Current page: `site/docs/getting-started.mdx` (the content being restructured — preserve its substance)

## How to consume a task

Per task: **Files** (exact paths), **Depends on**, **Spec** (governing section), **Acceptance**
(behaviors to demonstrate), **Verification** (exact command — `npm run build` run unpiped), **Commit**
(subject). Sub-pages are created before the overview references them so the build stays green; a
"doc not in sidebar" warning is acceptable until Task 5 wires the sidebar. Ask Watson before
creating/merging the PR. The spec + this plan ride in the PR.

---

## Task 1: Terminal basics sub-page

**Context:** The zero-tech floor — a reader who can't open a terminal needs this before anything else.

**Files:**
- Create: `site/docs/getting-started/terminal-basics.md`

**Depends on:** (none)

**Spec:** [§3.2](../specs/2026-06-13-getting-started-rework-design.md#32-sub-pages)

**Acceptance:**
- One short page explaining what a terminal is (you type a command and press Enter) and how to open one on Windows (PowerShell from the Start menu), macOS (Terminal via Spotlight), and Linux (the terminal app).
- Any example command shown carries a `title="Terminal"` code-block title (convention per `AUTHORING.md` §2).
- `sidebar_position`/front matter set so it can sit under the Getting Started category (Task 5 wires the sidebar; this task just needs valid front matter and a title).
- Skimmable; no lab/CAMELS specifics; links out rather than re-documenting OS basics where useful.

**Verification:**
```bash
cd site && npm run build
```
Expected: build succeeds (a not-in-sidebar warning for this new doc is acceptable until Task 5).

**Commit:** `docs(handbook): add terminal-basics getting-started sub-page`

---

## Task 2: Install Git sub-page

**Context:** From-scratch Git install for a beginner; linked from the Prerequisites table.

**Files:**
- Create: `site/docs/getting-started/install-git.md`

**Depends on:** (none)

**Spec:** [§3.2](../specs/2026-06-13-getting-started-rework-design.md#32-sub-pages), [§2 D3](../specs/2026-06-13-getting-started-rework-design.md#2-decisions)

**Acceptance:**
- Per-OS install guidance linking the official source (`git-scm.com`): Windows (e.g. winget or the installer), macOS (Xcode Command Line Tools or Homebrew), Linux (apt/dnf). Link-first; show a command only where it's stable.
- Ends with a verification step: a `git --version` block titled `title="Terminal"`, with the expected shape of a successful result.
- Valid front matter + title; no CAMELS specifics.

**Verification:**
```bash
cd site && npm run build
```
Expected: build succeeds (not-in-sidebar warning acceptable until Task 5).

**Commit:** `docs(handbook): add install-git getting-started sub-page`

---

## Task 3: Install GitHub CLI sub-page

**Context:** Install + authenticate `gh`; absorbs the "what gh auth does" explainer currently inline on the main page.

**Files:**
- Create: `site/docs/getting-started/install-github-cli.md`

**Depends on:** (none)

**Spec:** [§3.2](../specs/2026-06-13-getting-started-rework-design.md#32-sub-pages)

**Acceptance:**
- Per-OS install guidance linking the official source (`cli.github.com`): Windows (winget), macOS (Homebrew), Linux (apt/dnf or the official repo). Link-first.
- Authentication section: `gh auth login` and `gh auth setup-git` as `title="Terminal"` blocks, then `gh auth status` to confirm "Logged in to github.com".
- Includes the "what gh auth actually does" explanation migrated verbatim-in-substance from the current `getting-started.mdx` (as prose or a `<details>` collapsible) — wording preserved, not rewritten.
- Valid front matter + title; no CAMELS specifics.

**Verification:**
```bash
cd site && npm run build
```
Expected: build succeeds (not-in-sidebar warning acceptable until Task 5).

**Commit:** `docs(handbook): add install-github-cli getting-started sub-page`

---

## Task 4: Convert the page to a folder overview

**Context:** Turn the single page into the skimmable overview, label every command, link the sub-pages, and place the screenshot placeholders. This is the heart of the round.

**Files:**
- Delete: `site/docs/getting-started.mdx`
- Create: `site/docs/getting-started/index.mdx` (the moved + reworked overview)

**Depends on:** #1, #2, #3

**Spec:** [§3.1](../specs/2026-06-13-getting-started-rework-design.md#31-indexmdx-overview), [§4](../specs/2026-06-13-getting-started-rework-design.md#4-command-marking-applied), [§5](../specs/2026-06-13-getting-started-rework-design.md#5-screenshot-placeholders-d4)

**Acceptance:**
- The page moves to `getting-started/index.mdx`; the old `getting-started.mdx` is removed (no duplicate `/docs/getting-started` route). The URL `/docs/getting-started` still resolves.
- All existing substance is preserved (intro, `<DEV_ROOT>` explainer, Step 1, Step 2 bootstrap prompt + its two collapsibles, Verify, Next steps), restructured per §3.1 — not rewritten away.
- A **command legend** (the `AUTHORING.md` §2 snippet) appears near the top.
- **Every fenced command block carries a title:** the bootstrap prompt, `/plugin`, and the slash-command verify checks → `title="Claude session"`; shell verify checks (`Get-Item`, `ls -l`) → `title="Terminal"`.
- The **Prerequisites table** links each tool to its official install/signup page; Git and GitHub CLI rows additionally link "→ from scratch" to `install-git` / `install-github-cli`. A "New to terminals? → terminal-basics" pointer sits above the table. The inline `gh auth` commands + "what gh auth does" collapsible are removed here (they now live in `install-github-cli`).
- **Two screenshot placeholders** appear in Verify (a `/plugin`-output placeholder and a verify-checks-passing placeholder) as visible, obviously-temporary admonitions — **no broken image links**.
- Builds as valid MDX with no broken internal links.

**Verification:**
```bash
cd site && npm run build
python scripts/docs_budget.py --root .
```
Expected: both pass; `/docs/getting-started` resolves; the three sub-page links resolve.

**Commit:** `docs(handbook): convert getting-started to folder overview with command labels`

---

## Task 5: Nest the sub-pages in the sidebar

**Context:** Make the sub-pages discoverable without cluttering the skimmer's view.

**Files:**
- Modify: `site/sidebars.ts`

**Depends on:** #4

**Spec:** [§6](../specs/2026-06-13-getting-started-rework-design.md#6-sidebar-d5)

**Acceptance:**
- Under the existing "Get started" category, the flat `getting-started` entry becomes a nested category labeled "Getting Started" with `collapsed: true`, whose overview is `getting-started/index` and whose items are `getting-started/terminal-basics`, `getting-started/install-git`, `getting-started/install-github-cli`.
- The other "Get started" items (`working-with-claude`, `onboarding-project`) are unchanged; the "Deep Dives" category is untouched.
- No "document not included in sidebar" warnings remain for the three sub-pages.

**Verification:**
```bash
cd site && npm run build
```
Expected: build succeeds with no not-in-sidebar warnings; sidebar shows a collapsed "Getting Started" with three children.

**Commit:** `docs(handbook): nest getting-started sub-pages in sidebar`

---

## Self-review

- **Spec coverage:** §3.1 overview → Task 4; §3.2 sub-pages → Tasks 1–3; §4 command marking → Tasks 1–4 (titles on every block); §5 screenshot placeholders → Task 4; §6 sidebar → Task 5; §7 scope honored (no other pages, no bootstrap-prompt changes); D4 follow-up images noted as pre-merge, not a task.
- **Placeholder scan:** none (the screenshot "placeholders" are an intended deliverable, not plan gaps).
- **Consistency:** sub-page filenames (`terminal-basics`, `install-git`, `install-github-cli`) and the `getting-started/index` route are identical across Tasks 1–5 and the spec.
- **Ordering:** sub-pages (1–3) precede the overview move (4) so each build stays green; sidebar (5) clears the interim not-in-sidebar warnings last.
