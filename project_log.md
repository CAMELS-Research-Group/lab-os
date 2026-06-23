# lab-os — project log

Format: lab standard, `lab-os/.claude/rules/03-logging.md`. Skeleton per
`lab-os/templates/project_log.template.md` (normative — `log-lint` parses this structure).
The `## Standing Decisions` and `## Entries` headings are load-bearing lint anchors: exact
text, one each, never renamed. Entry headers are the only other `##` headings allowed.

## Standing Decisions

- 2026-06-23 07:51 — Plans track at the fork level; only project code nests · #44
- 2026-06-23 06:30 — Fork-of-lab-os is the default Claude-powered dev home · #43
- 2026-06-23 03:05 — Building sample plan ships as a facilitator-only fallback · #42
- 2026-06-19 05:58 — Workshop Program supersedes onboarding-project and one-day Building · #39
- 2026-06-13 15:00 — Handbook content rework gates tester launch · #25
- 2026-06-12 12:00 — Plan-execution logs close with their shipping PR · #18
- 2026-06-11 19:45 — Site owns human-facing docs · #15
- 2026-06-10 21:54 — Split the combined rule into 03-logging.md and 04-docs.md · #9
- 2026-06-10 17:45 — Adopt lab-wide logging & documentation standard · #6

## Entries

---

## 2026-06-23 07:51 — Plans track at the fork level; only project code nests

**Decision:** Refines the plan/project homing of #43. Methodology artifacts track in the fork itself,
not a nested repo — the plan and backlog at `_plans/`, the dev-home project log at the fork root. Only
the project *code* is re-homed as a separate gitignored nested repo. #43 homed "the plan/project"
together in the nested repo, conflating two artifacts with different needs.
**Why:** The fork is the methodology/coordination home, and plans are methodology — matching how the
lab already works (the dev root tracks the backlog and plan packets, not per-project). #43's
anti-coupling reason — don't couple `git pull upstream` with project history — bites for a full
codebase, not a handful of plan files in a new path upstream never touches. Splitting keeps that
benefit for the code while a dev-home session sees every plan.
**Refs:** #44, site/docs/getting-started/index.mdx, site/docs/workshops/building/pre-flight.md, docs/workshops/program/design.md

---

## 2026-06-23 06:30 — Fork-of-lab-os is the default Claude-powered dev home

**Decision:** Onboarding now forks lab-os (clone fallback) and uses that fork as the participant's
primary dev home, replacing the clone-as-rules-subdir + junction model. Rules live natively in the
fork; `git pull upstream main` keeps them current. A light cleanup makes the fork their own (reset
project log, drop lab-os's own design history + handbook site). Bring-your-own-project is preserved:
the plan/project is re-homed as its own gitignored repo nested in the fork, keeping a clean history
and inheriting the rules. The junction model is retained as the documented multi-repo power-user path.
**Why:** A fork gives a personal, push-able copy (real PR target, upstream-syncable rules) and drops
the most failure-prone bootstrap step. Nesting the project as a separate gitignored repo recovers the
junction's one benefit — lab tooling kept separate from project work — without it.
**Alternatives:** Junction/multi-repo dev-root — retained as the power-user path, not the default.
Commit the project into the fork (monorepo) — rejected, couples upstream pulls with project history.
**Refs:** #43; #42 (sample plan remains the facilitator-only fallback, unchanged by this)

---

## 2026-06-23 03:05 — Building sample plan ships as a facilitator-only fallback

**Decision:** Added `docs/workshops/building/sample-plan.md` — a pre-baked three-task plan (re-home fork identity · brand the handbook · add a backlog/planning surface) that a participant who reaches Building without a plan of their own runs against their own fork. ENG-tier under `docs/`, not published to the site and not in the sidebar; participants point Claude at the file from their fork's CLI. The fork's `cd site && npm run build` (broken-link-throw) is the gate for all three tasks.
**Why:** The mixed-cohort Building kickoff has newcomers who finish setup with no execution-ready plan and so can't practise the three execution modes. A small real plan unblocks them. Kept non-published to hold the program's bring-your-own-project line ("no prescribed sample project") on the public surface — the fallback exists for the blocked without the handbook advertising a sample as the path. Realises the fallback-starter the 2026-06-19 program decision deferred.
**Alternatives:** Publish as a participant page — rejected, contradicts the no-prescribed-sample stance on the public site. Demo-only build by the facilitator, no participant plan — rejected, leaves newcomers watching instead of practising.
**Refs:** #42, docs/workshops/building/sample-plan.md, Development/_packets/lab-os/workshop-program/

---

## 2026-06-19 05:58 — Workshop Program supersedes onboarding-project and one-day Building

**Decision:** The three-part Workshop Program (Planning → Building → Closeout), published under `site/docs/workshops/`, supersedes the two-week onboarding-project sandbox (now a redirect stub into the program) and the standalone one-day Building-with-Claude material (absorbed into the Building part's exercises). Facilitator runbooks are internal under `docs/workshops/`, not published.
**Why:** One coherent bring-your-own-project arc on a single self-paced + live-facilitated handbook surface, instead of a scattered sandbox plus a one-day track that diverge and double the maintenance. Phase-0 design lock approved by Watson at the overnight-run launch.
**Refs:** #39, Development/_packets/lab-os/workshop-program/

---

## 2026-06-13 15:00 — Handbook content rework gates tester launch

**Decision:** The handbook content + IA is being reworked across all seven pages before testers
are invited; tester launch now waits on the rework. This supersedes the prior round's deferral —
page-content restructuring left to play-test friction data, with launch "unblocked" after the
chrome/IA round. The rework is decomposed: a backbone authoring-conventions round first, then
per-page rounds (Getting Started next).
**Why:** The deferral assumed the existing content was good enough for a first cohort and that
friction data should drive structural change. On review of the shipped site, the content needs
reframing (Working-with-Claude → SDD lifecycle; Onboarding Project → Onboarding Workshop), zero-tech
support, and terminal-vs-Claude command clarity before a tester runs the arc — gaps a cohort hits
immediately, not subtle friction worth waiting for.
**Alternatives:** Launch on current content, rework in parallel — rejected, testers would hit content
already judged inadequate. Partial gate (ship some, defer the reframes) — considered, rejected for a
clean gated rework.
**Refs:** #25, docs/superpowers/specs/2026-06-13-handbook-backbone-conventions-design.md, docs/superpowers/specs/2026-06-12-handbook-frontend-design-round-design.md

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
