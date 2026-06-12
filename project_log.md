# lab-os — project log

Format: lab standard, `lab-os/.claude/rules/03-logging.md`. Skeleton per
`lab-os/templates/project_log.template.md` (normative — `log-lint` parses this structure).
The `## Standing Decisions` and `## Entries` headings are load-bearing lint anchors: exact
text, one each, never renamed. Entry headers are the only other `##` headings allowed.

## Standing Decisions

- 2026-06-12 12:00 — Plan-execution logs close with their shipping PR · #18
- 2026-06-11 19:45 — Site owns human-facing docs · #15
- 2026-06-10 21:54 — Split the combined rule into 03-logging.md and 04-docs.md · #9
- 2026-06-10 17:45 — Adopt lab-wide logging & documentation standard · #6

## Entries

---

## 2026-06-12 12:00 — Plan-execution logs close with their shipping PR

**Decision:** A plan's `## Execution Log` closes with the PR that ships it. Post-merge
evidence (deploy green, runtime verification, branch cleanup) goes to a comment on that
PR; bigger facts route per the entry triggers. No trailing entries held for a future PR.
Watson granted a narrow standing pre-authorization (recorded in his global CLAUDE.md):
merge-closeout status comments on his own PRs post without per-action confirmation.
**Why:** The execution log lives in the repo, so merge-time facts always arrive after the
last commit that could carry them. Both prior closeouts hitched entries onto whatever PR
came next (#16 carried the tasks 4–13 closeout), coupling unrelated PRs and dangling when
no next PR exists. The routing table already sends bare status to PR comments — this names
when the log closes so closeout facts stop defaulting into it.
**Alternatives:** Dedicated one-line closeout PRs — rejected, noise against the
single-concern merge bar. Predictive pre-merge entries — rejected, evidence written before
it exists.
**Refs:** #18, docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md

---

## 2026-06-11 19:45 — Site owns human-facing docs

**Decision:** The handbook site (`site/`, deployed to watsonwblair.github.io/lab-os) is the
owning home for human-facing docs. Root `BOOTSTRAP.md` and `WORKING-WITH-CLAUDE.md` become
pointer stubs to their site pages; `.claude/rules/` stay AI-tier repo files.
**Why:** Spec D3: one human-facing surface, written for its readers, with build-time link
checking — instead of agent-dense repo markdown doing double duty for stakeholders.
**Alternatives:** Site wraps the repo markdown unchanged — rejected, a brochure over a
codebase; site renders the repo markdown as-is — rejected, agent-dense prose shown to
stakeholders.
**Refs:** #15, docs/superpowers/specs/2026-06-11-lab-os-rename-handbook-site-design.md

---

## 2026-06-11 18:55 — Rename repo lab-rules → lab-os

**Decision:** Repo renamed to `lab-os` on GitHub; local remote re-pointed; dev-root rules
junction recreated; in-repo references swept (docs, templates, test fixtures) including the
reusable workflows' functional self-repo gates (`$GITHUB_REPOSITORY` comparisons) and
checkout refs. Old `lab-rules` URLs redirect; phase-2 caller YAMLs will be authored against
`lab-os`.
**Why:** The name described one slice of what the repo holds — rules, templates, CI
enforcement, onboarding, and the incoming handbook site (spec D1). Renaming before phase-2
callers ship keeps the cost near zero; after rollout every repo would reference the name.
**Refs:** #12, docs/superpowers/specs/2026-06-11-lab-os-rename-handbook-site-design.md

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
