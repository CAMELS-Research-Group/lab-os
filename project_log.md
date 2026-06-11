# lab-rules — project log

Format: lab standard, `lab-rules/.claude/rules/03-logging.md`. Skeleton per
`lab-rules/templates/project_log.template.md` (normative — `log-lint` parses this structure).
The `## Standing Decisions` and `## Entries` headings are load-bearing lint anchors: exact
text, one each, never renamed. Entry headers are the only other `##` headings allowed.

## Standing Decisions

- 2026-06-10 21:54 — Split the combined rule into 03-logging.md and 04-docs.md · #9
- 2026-06-10 17:45 — Adopt lab-wide logging & documentation standard · #6

## Entries

---

## 2026-06-10 21:54 — Split the combined rule into 03-logging.md and 04-docs.md

**Decision:** `.claude/rules/03-logging-and-docs.md` is split into `03-logging.md`
(altitudes, entry triggers/routing, entry format, immutability, file structure and
overflow) and `04-docs.md` (single-source, tiers and byte budgets, ENG document
standards, rules numbering). Numbered names retained per the rules-numbering
convention; live cross-references updated, historical artifacts left as written.
**Why:** The combined file sat at its own byte budget with `docs-budget` enforcement
now live — any rule edit first required an offsetting trim. Two single-responsibility
files restore headroom on both halves and give the standard room to grow.
**Alternatives:** Unnumbered `logging.md`/`docs.md` — rejected, contradicts the
rules-numbering convention shipped the same day; compression-only — rejected, the
margin stays structurally tight as rules accrete.
**Refs:** #9, https://github.com/WatsonWBlair/lab-rules/issues/7

---

## 2026-06-10 17:45 — Adopt lab-wide logging & documentation standard

**Decision:** Adopted the lab-wide logging & documentation standard: the
`03-logging-and-docs.md` rule, the PR-lifecycle doc, normative templates (project log,
PRD, CLAUDE.md tiers, work bundle), and CI adherence actions (`log-lint`, `docs-budget`,
`merge-bar-check`). Project logs converge on one shape — Standing Decisions index +
reverse-chron hot window + grep-only archive — with immutable merged entries,
audience-tiered doc budgets, and pre-merge log cleanup on every PR.
**Why:** Per-repo log formats had diverged to the point that the largest logs could no
longer be read whole by an agent; nothing documented how much of a log an agent should
read (no retrieval contract); and stale `Status:` markers forced follow-up edits to
merged entries. The standard fixes retrieval (index-first, then hot window, then grep),
removes anything in a merged entry that can go stale (currency lives in the index), and
routes overflow detail to PR bodies and specs. See spec §1.
**Refs:** #6, docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md

---

## 2026-06-09 00:00 — Add Mission-Control Sandbox onboarding project

*(Pre-standard entry, preserved verbatim in the 2026-06-10 migration — not an example of the current entry format.)*

Added `ONBOARDING-PROJECT.md`: a two-week, throwaway build that onboards new lab members to the lab's
spec-driven, sub-agent-driven workflow by having them build a small mission-control-style work surface
in their own disposable repo.

**Why a project, not just docs.** `WORKING-WITH-CLAUDE.md` describes the methods; this gives members a
concrete, low-stakes target to practice them on. The repo is disposable by design — the real
deliverable is a patterns & findings retro the lab can mine for the actual `mission-control` dashboard.
It doubles as cheap R&D: we discover good work-surface design patterns on throwaway code instead of on
the real product.

**Design decisions:**
- **Placed at repo root, not in `.claude/rules/`.** Those are hard rules loaded into every Cowork
  session and every PR review; an onboarding prompt is neither. Sits as a sibling to
  `WORKING-WITH-CLAUDE.md` / `BOOTSTRAP.md` and is linked from the README's "New to the lab?" section.
- **Two axes held separately** — an *open* capability checklist (what to build, left to the member to
  design) and a *fixed* workflow spine (brainstorm → spec → code-free plan → sub-agent build → review →
  log). Under time pressure members drop checklist items, never spine steps.
- **Stack is a discovery surface, not a constraint** — members choose their own stack but must produce
  a deployment-tradeoff writeup (local-first vs container vs serverless vs PaaS). Watson's steer: people
  should feel free to explore while being forced to reason about deployment tradeoffs.
- **Scope trimmed** from the initial 3 data integrations to 2, with one required to be the authenticated
  integration (folds the SSO/auth requirement in rather than adding a third surface).
- **Timebox: 2 weeks.**
