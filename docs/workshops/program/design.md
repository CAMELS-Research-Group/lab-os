# Workshop Program — Planning · Building · Closeout — design

**Status:** draft

**Date:** 2026-06-19 · **Repo:** lab-os · **Slice:** workshop-program
**Spec:** [`_packets/lab-os/workshop-program/prd.md`](../../../../_packets/lab-os/workshop-program/prd.md) (staged; not in published site)

---

## Problem

The handbook teaches spec-driven, multi-agent development as a solo, self-serve exercise on a
throwaway sandbox. That leaves two gaps: the planning runway is under-explained (participants arrive
at execution with plans too thin to run), and the hardest skill — trusting an autonomous agent to
run while knowing how to catch the moments it claims success it did not achieve — cannot be learned
alone. The result is a sandbox exercise and one isolated build day with no coherent arc connecting
them or retrospecting the work afterward.

This design pins the shape of a three-part program that fills that arc before any participant page
is authored.

---

## Decisions

### D1: Three-part arc — Planning → Building → Closeout

**Decision:** The program is structured as three ordered parts: **Planning** (blue-sky idea to
execution-ready plan), **Building** (execution-ready plan to verified, autonomous execution), and
**Closeout** (capture learnings, carry context forward, brief presentations).

**Rationale:** Each part has a distinct skill and a distinct artifact. Planning teaches the spec and
decomposition discipline; Building teaches autonomous execution and verification; Closeout teaches
the retro and carry-forward habits that make the work compoundable. The ordering is load-bearing:
Building consumes the plan Planning teaches a participant to produce; Closeout retrospects what
Building executed.

**Rejected alternatives:**

- **Single "Building with Claude" day (standalone)** — proven format but has no planning runway
  before it and no retrospective after it. Participants arrive with thin plans; the day is consumed
  by setup rather than execution. Superseded.
- **Two-week sandbox as sole path** — teaches by doing but tacitly assumes the participant already
  knows how to plan. The planning mechanics (idea → PRD → decomposed task list) are lab knowledge,
  not surfaced. Absorbed into the program's worked-project through-line; the fixed capability
  checklist model is retired.

---

### D2: Worked-project through-line (participant's own project)

**Decision:** The program is run against a project the participant brings (or a small, self-chosen
low-stakes one). The same project is planned in Part 1, executed in Part 2, and retrospected in
Part 3.

**Rationale:** A through-line project makes the skills concrete on real (if low-stakes) work.
Planning a made-up spec is qualitatively different from planning something the participant actually
wants to build. The connection across parts is also motivating: the artifact from Planning is
directly consumed by Building, making each part's output matter.

**Bring-your-own-project is a hard prerequisite** — no fallback sample repo is provided. The
framing is "bring a low-stakes real project." A project with no stakes and no author investment
undermines the verification discipline Building teaches (it is much harder to care whether a gate
passes on a spec you have no stake in).

**Rejected alternatives:**

- **Fixed sample/starter repository** — the retired sandbox used a fixed capability checklist
  (dashboard-style app with specific surface requirements). It prescribed the *what* so tightly that
  participants practiced workflow mechanics without practicing the design judgment that spec-driven
  development is actually for. Dropped.
- **Optional fallback starter** — deferred. If facilitator experience reveals a meaningful fraction
  of participants arriving without a project, a minimal starter may be added; the program copy does
  not block on it.

---

### D3: "Execution-ready" definition (resolves Open Question Q1)

**Decision:** A plan is **execution-ready** when it satisfies all of the following, stated as
checkable criteria:

1. There is a written PRD with: a concrete **problem** statement (specific, not vibes); **success
   criteria** that are measurable and falsifiable (not "it feels right"); an explicit **scope** with
   both in-scope and out-of-scope named; and **constraints** that bound the work (time, data,
   budget, infra, approvals).
2. The PRD has been **decomposed** into an ordered list of individually-verifiable **tasks**.
3. Each task is small enough to **finish and verify on its own** — a task too large to complete in
   one sitting must be split before the plan is considered execution-ready.
4. Each task has a **verification note**: what will prove it done (a command, an observable
   behavior, a file that exists with specific content).
5. Dependencies between tasks are marked: what blocks what, so parallel waves are visible.

This definition is operationalized as checkboxes in the Planning completion checklist (Task 6). A
plan that passes those boxes can be handed to an autonomous run without back-and-forth about scope,
success, or order.

**Rationale:** "Execution-ready" without concrete criteria is a judgment call that varies by
facilitator and participant. Making it a checklist makes completion self-attestable and removes the
need for a facilitator to decide whether someone "gets it."

**Rejected alternatives:**

- **"Ready when the facilitator says so"** — not self-attestable; breaks the self-paced path.
- **"Ready when Claude can run it without asking clarifying questions"** — Claude will try to run
  almost anything; this is not a reliable bar.

---

### D4: Dual-delivery contract — live cohort + self-paced

**Decision:** The live-facilitated cohort and the self-paced reader use the **same worksheets**.
The live layer adds facilitation and cohort dynamics (presentations, live debrief); it does not
introduce content a self-paced reader lacks.

**What the live layer adds:**
- A facilitator who coaches the interrogation phase and recovers from stuck participants
- Live debrief of failure modes during the first autonomous execution
- Cohort presentations and collective learnings at Closeout
- Time-boxed structure keeping the session on pace

**What the self-paced layer provides:**
- The same worksheets, with enough framing that a solo reader can work through them unaided
- Completion checklists for self-attestation in the absence of a facilitator
- The presentation worksheet adapted for self-recorded or written share (no live cohort required)

**Rationale:** Content that exists only in the live path is content a self-paced reader cannot
access. The handbook is public and permanent; any material only available in a specific cohort
session is effectively behind a time-gated wall.

**Rejected alternatives:**

- **Live-only content (exercises only in session, not published)** — breaks the self-paced
  guarantee; creates two diverging versions to maintain.
- **Self-paced-only (no facilitator runbooks)** — facilitator runbooks are already proven useful
  (the Exercise-4 runbook from the prior Building day). Dropped.

---

### D5: Multi-agent dynamic-workflow depth (resolves Open Question Q3)

**Decision:** The Building part's third execution mode (multi-agent dynamic workflows) is taught
**conceptually and via documented Claude Code native capabilities** plus the Working-with-Claude
methods. It does not bind to unbuilt lab agent infrastructure. An "advanced / optional" framing is
used for anything beyond the documented Claude Code capabilities.

**Rationale:** Binding the workshop to infrastructure that has not shipped would make Part 2 of the
program incomplete until that infrastructure lands. The conceptual framing is sufficient to teach
the skill; practitioners who want to go further can follow links to Claude Code's own documentation.

**Rejected alternatives:**

- **Defer the mode entirely** — the three-mode progression (sequential → autonomous → dynamic) is
  the pedagogical arc; removing the third mode truncates the arc. Kept as conceptual + optional.
- **Block on lab infra shipping** — blocks the program indefinitely.

---

### D6: Facilitator-runbook location (resolves Open Question Q6)

**Decision:** Per-part facilitator runbooks live under `docs/workshops/` (ENG-tier, not published
to the Docusaurus site). Participant pages and worksheets publish under `site/docs/workshops/`.

**Rationale:** External facilitators read the runbooks; they must carry no internal codenames or
lab context. But they are not participant-facing, so publishing them on the public site is not
necessary. The split keeps the public site clean while making the runbooks available to the
facilitator team.

**Rejected alternatives:**

- **Publish runbooks on the public site** — facilitator notes are not appropriate for public
  participant view; adds noise to the participant experience.

---

## Per-part inventory

The full artifact set per part. Each file named with its target path (repo-relative).

### Part 1 — Planning

| Artifact | Path |
|---|---|
| Participant page | `site/docs/workshops/planning/index.md` |
| Pre-work worksheet | `site/docs/workshops/planning/pre-work.md` |
| PRD-interrogation worksheet | `site/docs/workshops/planning/prd-interrogation-worksheet.md` |
| Plan / decomposition worksheet | `site/docs/workshops/planning/plan-decomposition-worksheet.md` |
| Completion checklist | `site/docs/workshops/planning/completion-checklist.md` |
| Homework prompt + capacity sheet | `site/docs/workshops/planning/homework.md` |
| Facilitator runbook | `docs/workshops/planning/facilitator-runbook.md` |

### Part 2 — Building

| Artifact | Path |
|---|---|
| Participant page | `site/docs/workshops/building/index.md` |
| Pre-flight worksheet | `site/docs/workshops/building/pre-flight.md` |
| Five execution exercises | `site/docs/workshops/building/exercises.md` |
| Execution-and-verification worksheet | `site/docs/workshops/building/execution-verification-worksheet.md` |
| Quality-gate / review-standards worksheet | `site/docs/workshops/building/quality-gates-worksheet.md` |
| Completion checklist | `site/docs/workshops/building/completion-checklist.md` |
| Homework prompt + capacity sheet | `site/docs/workshops/building/homework.md` |
| Facilitator runbook | `docs/workshops/building/facilitator-runbook.md` |
| Verification-command contract | `docs/workshops/building/verification-command-contract.md` |

### Part 3 — Closeout

| Artifact | Path |
|---|---|
| Participant page | `site/docs/workshops/closeout/index.md` |
| Pre-work worksheet | `site/docs/workshops/closeout/pre-work.md` |
| Learnings-capture / carry-forward worksheet | `site/docs/workshops/closeout/learnings-carry-forward-worksheet.md` |
| Presentation worksheet | `site/docs/workshops/closeout/presentation-worksheet.md` |
| Completion checklist | `site/docs/workshops/closeout/completion-checklist.md` |
| Facilitator runbook | `docs/workshops/closeout/facilitator-runbook.md` |

### Program-level

| Artifact | Path |
|---|---|
| Program landing page | `site/docs/workshops/index.md` |
| Sidebar wiring | `site/sidebars.ts` |

---

## Supersession map

Every existing file that this program absorbs, retires, or redirects. Dispositions follow the
Q4 decision (redirect vs. keep-as-alternative): the standalone pages no longer present themselves
as the path; each either redirects to or is replaced by the program.

| File | Disposition | Rationale |
|---|---|---|
| `site/docs/onboarding-project.md` | **Redirect / stub** — becomes a short page pointing at the program; labeled as a legacy alternative track if kept, but the program is the presented path. | The two-week sandbox's workflow-spine and retro ideas are absorbed into the worked-project through-line (D2). The fixed capability checklist model (D2, rejected alternative) is retired. |
| `site/docs/workshops/index.md` *(worktree version, one-day framing)* | **Replace** — this program's landing page (`site/docs/workshops/index.md`) supersedes it as the workshops index. | The one-day structure is superseded by the three-part program. |
| `site/docs/workshops/building-with-claude/index.md` | **Absorb + retire** — content absorbed into Building participant page and exercises; the `building-with-claude/` path is retired (not re-published as a parallel path). | Avoids a duplicate Building surface. |
| `site/docs/workshops/building-with-claude/exercises.md` | **Absorb** — the five exercises carry forward into `site/docs/workshops/building/exercises.md`. | Direct content absorption; the old path is not linked going forward. |
| `site/docs/workshops/building-with-claude/preflight.md` | **Absorb** — the environment checklist carries forward into `site/docs/workshops/building/pre-flight.md`. | Direct content absorption. |
| `docs/workshops/building-with-claude/exercise-4-facilitator-runbook.md` | **Absorb** — failure-mode catalogue and recovery moves carry forward into `docs/workshops/building/facilitator-runbook.md`. | Preserves the highest-risk segment's institutional knowledge. |
| `docs/workshops/building-with-claude/verification-command-contract.md` | **Absorb** — the contract carries forward into `docs/workshops/building/verification-command-contract.md`. | Stable contract; re-homed, not altered. |
| `docs/workshops/building-with-claude/HANDOFF.md` | **Archive / retire** — the handoff was specific to the prior worktree transition; no successor needed. | The program's design doc (this file) captures the relevant context. |

**Rejected alternative (applies to all):** Delete rather than redirect/absorb. Rejected because
clean redirects preserve any existing inbound links and make the transition explicit to a reader who
had bookmarked the old path.

---

## Known gaps

- The per-part runbooks do not yet exist (authored in subsequent tasks); this design specifies their
  target paths and content contracts but does not pre-author them.
- The final public-tier overclaim scrub (Task 25) runs after all participant pages are authored; the
  scrub may require edits back to any page, which is by design.
- Q4 (onboarding-project page disposition — redirect vs. keep-as-alternative track) is recorded
  above as "redirect / stub" but the exact page text is authored in Task 24 where the implementation
  evidence is visible. No fresh human decision is needed at Task 24.
- Session cadence (Q5 — per-part time-box, multi-day cohort, self-paced) is deliberately left out
  of participant copy; live cadence is operational, not fixed in the program materials.
