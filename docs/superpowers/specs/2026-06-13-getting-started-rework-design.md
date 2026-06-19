# Getting Started rework — design

Status: reviewed — approved by Watson 2026-06-13 (brainstorm session).

First per-page round of the handbook content rework. Applies the conventions from
`2026-06-13-handbook-backbone-conventions-design.md` (and the working reference `site/AUTHORING.md`):
hybrid depth model, Terminal-vs-Claude command marking, link-out-plus-fill-gaps zero-tech support.

## 1. Problem

The current `site/docs/getting-started.mdx` is a single page that assumes some technical fluency: it
links no install/signup help for Git or the GitHub CLI, never says whether a command goes in the
terminal or an active Claude session (the `/plugin` check in *Verify your setup* reads as a shell
command but isn't), and offers nothing for a reader who can't open a terminal. It must support a
zero-technical-capability reader without burying skimmers in hand-holding.

## 2. Decisions

| # | Decision | Alternatives rejected |
|---|---|---|
| D1 | Convert to a **folder**: `getting-started/index.mdx` overview + sub-pages; URL stays `/docs/getting-started` | Keep one page (can't hold zero-tech depth without overloading skimmers) |
| D2 | **Per-tool zero-tech sub-pages**: `terminal-basics.md`, `install-git.md`, `install-github-cli.md` | Combined "from scratch" page (less linkable); minimal/inline-only (weak for true beginners) |
| D3 | **Mark every command** with code-block titles (`Terminal` / `Claude session`) + a one-time legend | Admonitions / badges (per backbone D2) |
| D4 | **Screenshots are a follow-up**: ship with marked placeholders, real PNGs dropped in before this page's PR merges | Block on screenshots up front; skip them entirely |
| D5 | Sub-pages **nested under a collapsed "Getting Started" sidebar category** | Exclude from sidebar (less discoverable, needs warning suppression) |
| D6 | Claude Code install **stays a Step 1 link-out** (no sub-page) | A Claude Code install sub-page (official setup already suffices — link-out per backbone D3) |

## 3. Structure

```
site/docs/getting-started/
  index.mdx              # skimmable overview — the happy path
  terminal-basics.md     # what a terminal is; how to open one per OS
  install-git.md         # from-scratch Git install per OS + git --version check
  install-github-cli.md  # install gh per OS; gh auth login / setup-git; absorbs the "what gh auth does" explainer
```

### 3.1 `index.mdx` (overview)

Retains the current substance, restructured and labeled:

- **Command legend** near the top (the snippet from `AUTHORING.md` §2).
- The `<DEV_ROOT>` explainer stays.
- A "New to terminals? → [Terminal basics](./terminal-basics.md)" pointer above the Prerequisites table.
- **Prerequisites table** — each tool row links to its official install/signup page, and Git / GitHub CLI
  additionally link "→ from scratch" to their sub-pages. The inline `gh auth` commands and the "what gh
  auth does" collapsible move to `install-github-cli.md`; the table just points there.
- **Step 1 — Install Claude Code:** unchanged (link-out).
- **Step 2 — Paste the bootstrap prompt:** the prompt block gets `title="Claude session"`; the
  junction/symlink and three-CLAUDE-layers `<details>` collapsibles stay inline.
- **Verify your setup:** shell checks (`Get-Item`, `ls -l`) get `title="Terminal"`; slash-command checks
  (`/plugin`, the fresh-session question) get `title="Claude session"`. Two screenshot placeholders land
  here (D4): `/plugin` output and the verify checks passing.
- **Next steps:** unchanged.

### 3.2 Sub-pages

- **`terminal-basics.md`** — one short page: what a terminal is (you type a command, press Enter), and how
  to open one — Windows (PowerShell from Start), macOS (Terminal via Spotlight), Linux (your terminal app).
- **`install-git.md`** — per-OS install (link `git-scm.com`; Windows winget / macOS Xcode CLT or Homebrew /
  Linux apt|dnf), then confirm with a `git --version` block (`title="Terminal"`).
- **`install-github-cli.md`** — per-OS install of `gh` (link `cli.github.com`), then `gh auth login` and
  `gh auth setup-git` (`title="Terminal"`), confirm with `gh auth status`. Absorbs the current "what gh
  auth does" explainer (as inline prose or a `<details>`).

## 4. Command marking (applied)

Every fenced command block in the folder carries a title: `Terminal` for OS-shell commands, `Claude
session` for prompts and slash-commands. The legend appears once in `index.mdx`. This is the convention's
first real application; `AUTHORING.md` is the reference.

## 5. Screenshot placeholders (D4)

Where each screenshot will go, the build ships a visible, obviously-temporary placeholder (a Docusaurus
admonition, e.g. `:::note Screenshot pending: /plugin output :::`) — never a broken image link. The two
images (`/plugin` output in a Claude session; verify checks passing) are captured from a real environment
and dropped in before this page's PR merges. Placeholders must be gone (replaced by images) at merge.

## 6. Sidebar (D5)

In `sidebars.ts`, the flat `getting-started` entry under the "Get started" category becomes a nested
category labeled "Getting Started", `collapsed: true`, whose index is the overview and whose items are the
three sub-pages. The other "Get started" entries (working-with-claude, onboarding-project) are untouched
this round.

## 7. Scope

**In:** the folder conversion; the three sub-pages; command-marking + legend across the folder; screenshot
placeholders; the sidebar nesting for this page.

**Out:** the other six pages (own rounds); the actual screenshot images (follow-up before merge, D4);
any change to the bootstrap prompt's steps or the rules it references.

## 8. Verification

- `cd site && npm run build` (unpiped) passes — `onBrokenLinks: 'throw'` confirms every new sub-page link
  and the `/docs/getting-started` URL resolve; MDX valid.
- `python scripts/docs_budget.py --root .` passes.
- Manual: the overview reads as a skimmable happy path; every command block shows a Terminal/Claude
  session title; the legend is present; sidebar shows a collapsed "Getting Started" with the three
  sub-pages; no broken or placeholder images remain at merge (placeholders allowed pre-merge per D4).

## 9. Known gaps

- The exact per-OS install commands in the sub-pages are settled during implementation against the tools'
  official docs (link-first; commands shown only where stable).
- Screenshot sourcing (who captures, when) is resolved per D4 before merge; if images aren't ready, the
  page ships with placeholders only by explicit Watson call.
- Heading anchors: this page isn't cited by `PR-LIFECYCLE.md` / `TROUBLESHOOTING.md` (those cite
  `working-with-claude`), so the folder move carries no referrer updates — confirmed during implementation.
