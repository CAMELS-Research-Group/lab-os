# Working with Claude rework — SDD-lifecycle reframe — plan

Code-free implementation plan. Spec (source of truth):
[`2026-06-13-working-with-claude-rework-design.md`](../specs/2026-06-13-working-with-claude-rework-design.md).

One PR, single concern: reframe `working-with-claude` into the SDD-lifecycle folder. The folder
conversion and all anchor/referrer updates land together so the build never breaks
(`onBrokenLinks: 'throw'`). Tasks 1–6 create content; Task 7 wires the sidebar; Task 8 updates
referrers; Task 9 is the green-gate close. Tasks 1–6 are independent of each other (separate new
files); 7–9 depend on them.

---

## Task 1 — Overview page (`index.md`)

**Files:** Create `site/docs/working-with-claude/index.md`. Delete `site/docs/working-with-claude.md`
(page→folder move; the old `.md` must be removed or the route collides).
**Depends on:** none.
**Spec:** design §3.1, §6 (command marking), D1–D2, D5–D6.

**Acceptance:**
- Front-matter `title: Working with Claude`, `sidebar_position: 2`, a description matching the
  reframe; H1 is just "Working with Claude" (no "— lab methods" subtitle).
- Lead-in frames the content as the lab's *discovered* SDD lifecycle, owned by the working group and
  still evolving, walked manually once by new members before independent work (D6).
- Retains the three existing pointers: hard rules → `.claude/rules/`; operating philosophy →
  global `CLAUDE.md` template; `superpowers` process skills + "lab convention wins on conflict".
- A **"lifecycle at a glance"** list names all 7 stages in order, one line each, with Verify marked
  *automated checkpoint* and Review *human checkpoint*; the five heavy stages link inward.
- **Brainstorm**, **Specify**, **Close** appear inline with their substance (per §3.1): brainstorm
  + check-what-exists; PRD elements + no-code-until-sign-off; logging/checkpoint/auto-memory.
- A **"Throughout: Communication & Memory"** section presents the overclaim-scrub,
  partners-vs-prospects, and continuous-capture/checkpoint material as cross-cutting (D2).
- No content from §2/§3/§4/§5/§6 is duplicated here beyond a one-line summary + inward link.
- Valid MDX (`<details>` allowed; no bare `<tokens>`, no `<!-- -->` comments).

**Verification:** covered by Task 9 build; locally `cd site && npm run build` resolves the page.
**Commit:** (folded into the single PR commit — see Task 9 note on commit strategy)

---

## Task 2 — Plan stage sub-page (`plan.md`)

**Files:** Create `site/docs/working-with-claude/plan.md`.
**Depends on:** none.
**Spec:** design §3.2 (`plan.md`).

**Acceptance:**
- Carries the current §2 substance: the six plan elements (Files · Depends on · Spec · Acceptance ·
  Verification · Commit), the no-literal-code / no-TDD-walkthrough rule, the rationale (code-heavy
  plans rot), the `superpowers:writing-plans` override, and the `04-docs.md` source-of-truth link.
- Front-matter title "Plan" (or "Plan — code-free implementation plans"); reads as the Plan stage of
  the lifecycle, with a back-link to the overview.

**Verification:** Task 9 build.
**Commit:** folded (Task 9).

---

## Task 3 — Build stage sub-page (`build.md`)

**Files:** Create `site/docs/working-with-claude/build.md`.
**Depends on:** none.
**Spec:** design §3.2 (`build.md`).

**Acceptance:**
- Carries the current §3 substance: plan-is-the-human→agent-handoff-artifact, **plus** the former
  `<details>` content promoted to body prose (subagents discard context / return only their report;
  backlog with stable IDs, agent-suitability, dependency graph, git-authoritative completion).
- Links to `autonomous-loops.md` as the deep dive for unattended runs.
- Front-matter title "Build"; reads as the Build stage; back-link to overview.

**Verification:** Task 9 build.
**Commit:** folded (Task 9).

---

## Task 4 — Autonomous loops sub-page (`autonomous-loops.md`)

**Files:** Create `site/docs/working-with-claude/autonomous-loops.md`.
**Depends on:** none.
**Spec:** design §3.2 (`autonomous-loops.md`).

**Acceptance:**
- Carries the current §6 substance: wake-to-gate-or-clean-halt target, the halt contract
  (digest-written completion signal), **plus** the former `<details>` content promoted to prose
  (budget caps + wall-clock kill, can't-read-own-quota, test-the-halt-path, forbid hard-to-reverse
  ops, human-gated authorization is correct).
- Front-matter title "Autonomous loops" (or "Autonomous / overnight loops"); framed as a Build
  deep dive; back-link to overview / Build.
- **Anchor-critical:** the page URL is `/docs/working-with-claude/autonomous-loops` (consumed by
  Task 8).

**Verification:** Task 9 build.
**Commit:** folded (Task 9).

---

## Task 5 — Verify stage sub-page (`verify.md`)

**Files:** Create `site/docs/working-with-claude/verify.md`.
**Depends on:** none.
**Spec:** design §3.2 (`verify.md`), D5.

**Acceptance:**
- Carries the current §4 substance: an-agent's-self-report-is-not-evidence (the D5 failure mode that
  *justifies* the stage — keep it), optimistic narrator, green≠reviewed (self-referential coverage),
  run-the-gate-unpiped, credential/data paths never gate-verified,
  `superpowers:verification-before-completion`.
- Framed as the **automated checkpoint** of the lifecycle; back-link to overview.
- **Anchor-critical:** page URL `/docs/working-with-claude/verify` (consumed by Task 8).

**Verification:** Task 9 build.
**Commit:** folded (Task 9).

---

## Task 6 — Review stage sub-page (`review.md`)

**Files:** Create `site/docs/working-with-claude/review.md`.
**Depends on:** none.
**Spec:** design §3.2 (`review.md`).

**Acceptance:**
- Carries the current §5 substance: multi-agent first pass + audit pass (escalate on zero findings),
  review-catches-what-the-gate-cannot, outsider's-eye/declare-if-you-authored,
  review-is-the-deliverable + unsolicited-post approval gate, the `PR-LIFECYCLE.md` /
  `01-workflow.md` links (lifecycle, merge bar, solo-maintainer exception).
- Framed as the **human checkpoint** of the lifecycle; back-link to overview.
- **Anchor-critical:** page URL `/docs/working-with-claude/review` (consumed by Task 8).

**Verification:** Task 9 build.
**Commit:** folded (Task 9).

---

## Task 7 — Sidebar nesting

**Files:** Modify `site/sidebars.ts`.
**Depends on:** 1–6.
**Spec:** design §5.

**Acceptance:**
- The flat `'working-with-claude'` entry under "Get started" becomes a nested `category` labeled
  "Working with Claude", `collapsible: true`, `collapsed: true`, `link` → doc id
  `working-with-claude/index`.
- `items` are the five sub-pages in **lifecycle order**: `working-with-claude/plan`,
  `working-with-claude/build`, `working-with-claude/autonomous-loops`, `working-with-claude/verify`,
  `working-with-claude/review`.
- Position between the "Getting Started" category and `'onboarding-project'` is preserved.

**Verification:** Task 9 build.
**Commit:** folded (Task 9).

---

## Task 8 — Anchor / referrer updates

**Files:** Modify `PR-LIFECYCLE.md`, `site/docs/onboarding-project.md`, `WORKING-WITH-CLAUDE.md`.
**Depends on:** 1, 4, 5, 6 (the target pages must exist for the build to resolve the new links).
**Spec:** design §4.

**Acceptance:**
- `PR-LIFECYCLE.md`: the `#4-verification-discipline` link → `site/docs/working-with-claude/verify.md`;
  both `#5-review-discipline` links → `site/docs/working-with-claude/review.md`; visible "§4–5" /
  "§5" text reworded to name the **Verify** / **Review** stages.
- `site/docs/onboarding-project.md`: `#2-code-free-implementation-plans` →
  `/docs/working-with-claude/plan`; `#6-autonomous--overnight-loops` →
  `/docs/working-with-claude/autonomous-loops`; surrounding link text at both spots lightly reworded
  to match the new stage names (Watson call). No other prose in the file changes.
- `WORKING-WITH-CLAUDE.md`: the source-link `site/docs/working-with-claude.md` →
  `site/docs/working-with-claude/index.md`.
- No remaining reference to a `working-with-claude.md#…` anchor anywhere in the repo (grep clean).

**Verification:** `grep -rn "working-with-claude.md#" .` returns nothing (excluding spec/plan/this
file's historical mentions); full check via Task 9 build.
**Commit:** folded (Task 9).

---

## Task 9 — Gate green + PR

**Files:** none (verification + PR).
**Depends on:** 1–8.
**Spec:** design §8.

**Acceptance:**
- `cd site && npm run build` (unpiped) exits 0 — every new page, the `/docs/working-with-claude`
  folder URL, all five sub-page links, and all updated referrers resolve; MDX valid.
- `python scripts/docs_budget.py --root .` exits 0.
- Old `site/docs/working-with-claude.md` is gone (no duplicate route).
- Manual skim: overview reads as a walkable lifecycle; Verify=automated / Review=human flagged;
  Communication+Memory read as cross-cutting; sidebar shows collapsed "Working with Claude" with
  five sub-pages in order.

**Verification:**
`cd site; npm run build` then `python scripts/docs_budget.py --root .` — both exit 0, run unpiped.
**Commit strategy:** the whole round is one commit (folder + sidebar + referrers must move together
or the build breaks). Subject: `docs(handbook): reframe working-with-claude as the SDD lifecycle`.
PR stacks per the continuous-execution convention; Watson merges.

---

## Execution Log

_(deviations, implementation calls, gate evidence land here as the plan runs)_
