---
name: pr-round
description: Use when pushing every open PR connected to you through one round of work — reviewing PRs others opened (approve / request changes with a posted review comment) and remediating review feedback on your own; when the user invokes /pr-round; when a specific PR is named (lab-os#42, a pull URL) and needs a single review or remediation round rather than an iterative loop; when a queue of stalled PRs needs to move toward the merge bar collaboratively.
---

# pr-round

## Overview

One round of the *right kind* of attention on every pull request connected to you, in parallel.
Ownership picks the lane:

| The PR is… | Lane | What happens |
|---|---|---|
| **someone else's**, you're a requested reviewer | **review** | Outsider review against the composed rubric → posts a review comment → posts a formal `approve` / `request-changes` verdict |
| **yours** | **remediate** | Ingests existing review feedback → auto-fixes mechanical findings → asks you about design-pins → applies → posts one handoff comment |

Given a PR reference — `lab-os#42`, `CAMELS-Research-Group/lab-os#42`, a pull URL, or a bare number — it acts on
that PR alone. Given none, it resolves everything you authored plus everything you were asked to
review.

The point is **collaborative** movement toward the merge bar: your collaborators stop waiting on your
review, and your own PRs stop sitting on unactioned feedback.

## Relationship to `pr-review-loop`

They are orthogonal, and reaching for the wrong one wastes a lot of tokens.

|  | `pr-round` | `pr-review-loop` |
|---|---|---|
| Shape | **Breadth** — many PRs, one round each | **Depth** — one PR, many passes |
| Terminates on | Round complete | A merge bar (0 Blockers) |
| Handles others' PRs | Yes — it is half the skill | No |
| Fixes your PR | Mechanical + decided design-pins, once | Iterates until the bar is met |

**Rule:** sweeping a queue, or unblocking a collaborator → `pr-round`. Driving one PR all the way to
merge-ready → `pr-review-loop`. A common sequence is `pr-round` to triage the queue, then
`pr-review-loop` on the one PR you actually want to land today.

`pr-round` has **no runtime dependency on `pr-review-loop`.** Deleting it does not break this skill.

**Availability note:** lab-os does not (yet) carry `pr-review-loop` — it is maintained in the lab's
workspace forks. The redirects above and in *When NOT to use* describe the shape of work to reach
for; until the sibling is ported here, run it from a dev home that carries it.

## The composed rubric

The review standard assembles from **three tiers**, so the skill works in any repository, not just
lab ones. Full contract — discovery paths, precedence, degradation, the cold-start prompt —
is [reference/rubric-layering.md](reference/rubric-layering.md).

| Tier | Home | Scope |
|---|---|---|
| **1 — Universal** | ships with the skill | Any PR anywhere. Correctness, security, error handling, tests, docs, structural classes. Names no house convention. |
| **2 — Lab** | `<DEV_ROOT>/reference/code-quality-taxonomy.md` + `<DEV_ROOT>/.claude/rules/` | What the working group applies to every project. Consumed in place — this skill is one of the taxonomy's derivers, not a copy of it. |
| **3 — Project** | `<repo>/reference/code-quality-rubric.md` | That project's stack and scoped concerns. |

A missing tier **degrades and notifies**: the run proceeds, the comment states which tiers applied,
and the roster prints a ready-to-paste prompt for generating the missing rubric. The skill never
writes a rubric into a repo it was only asked to review.

## Two fan-outs, and why

Subagents cannot call `AskUserQuestion`. So a design-pin found during remediation can only be
*returned*, decided in the main loop, and applied by a second dispatch:

```
Step 5  fan out #1  → review (returns comment + verdict) | remediate (fixes, returns pins)
Step 6  decide      → batched questions, each with a Defer option
Step 7  file issues → follow-up issues for deferred + out-of-band items (own PRs, consent-gated)
Step 8  fan out #2  → apply decisions, reusing the Step-5 worktree
Step 9  comment     → ONE handoff comment per remediated PR
Step 10 comment     → review comment + formal verdict per reviewed PR
Step 11 summary + cleanup (must run last)
```

**One comment per remediated PR, posted when the round is done.** It is the handoff artifact from
remediator back to reviewer — what was ingested, what changed, what was decided and why, what is
still open, and whether the PR is ready for another look. Nothing is posted mid-round, because a
comment written before decisions are applied is stale on arrival.

**Every post happens in the main loop — no subagent posts under your identity.** Consent is
main-loop state and a subagent cannot ask for it, so a subagent asked to decide whether to post has
no way to know; both defaults it could pick are wrong, and one of them is irreversible. The review
lane composes its comment and verdict and returns them; Step 10 posts them.

## Interrupt model

| Finding | Behavior |
|---|---|
| Mechanical, on your PR | Auto-fixed via `Edit`, committed, pushed. No interrupt. |
| Design-pin, on your PR | Returned from the subagent, asked at Step 6 with 2–3 options + `Defer`, applied at Step 8. |
| Anything on someone else's PR | Reported in the review comment. Never edited — it is their PR. |
| Roster over 8 PRs | One cost-guard question before dispatch, offering repo- and lane-scoped cuts before a bare count cap. |
| Red gate, or a gate that cannot run at all | No push. The commit survives; worktree preserved, path named in the roster. |
| A resolution the skill cannot perform | Returned as an out-of-band follow-up — on your own PRs, filed as a follow-up issue at Step 7 (consent-gated); otherwise recorded in the comment and the roster for manual action. |

## Verification: the gate ladder

Nothing is pushed on unverified work, but "the repo's full gate" is not always runnable — a heavy or
environment-fragile gate is normal, and a skill that pretends otherwise gets a quietly-improvised
substitute reported as green. So the fallback is **sanctioned and disclosed** rather than left to
improvisation. `PROMPT.md` § 5.3 step 5 owns the ladder:

| Rung | Ran | Pushes? |
|---|---|---|
| 1 | The repo's designated gate, unpiped | Yes, on green |
| 2 | A **scoped** check covering the touched files | Yes, on green — comment names the exact commands *and* what went uncovered |
| 3 | Nothing; the repo has a gate that could not run | **No.** Commit survives, worktree preserved |
| 4 | Nothing; the repo defines no gate | Yes, disclosed as `unverified` |

Rung 2 is never written as plain `green`. A repo that declares a gate expects verification, so rung 3
stops rather than shipping past it — the missing evidence is the point, and a human resolves it.

## Host prerequisites

- **The main loop isolates itself first (Step 0.0).** Some hosts reject file edits in a shared
  checkout until the session enters a worktree. That guard is a session-level latch that subagents
  *inherit*, so an un-isolated main loop means every remediate agent fails to write at once, mid-round.
  Entering a worktree up front removes it and costs nothing where no guard exists. The main loop stays
  a coordinator — this is its own workspace, not a PR worktree, and agents still apply their own edits.
- **`PR_ROUND_WORKTREE_ROOT`** relocates per-PR worktrees off the default `<repo>/.claude/worktrees`.
  Set it when the default is the wrong place: a host path-length ceiling, or a dev-home-wide guard
  that reaches into a nested project repo's worktrees. Worktrees are namespaced by repo underneath it.

## Usage

```
/pr-round [<PR ref>…] [--limit N] [--no-skip] [--review-only] [--remediate-only]
          [--dry-run] [--concurrency N]
```

`--dry-run` resolves and reports the roster and makes no write call of any kind — the safe first
invocation. `--no-skip` disables the no-new-activity filter, which otherwise skips PRs untouched
since this skill's last comment so a re-run costs one API call instead of a model pass.

## Consent

**One question per run**, at Step 3, before any dispatch — fired whenever the run will post
anything, on your own PRs included. `PROMPT.md` § Step 3 is the owning definition; this is a
summary.

**This skill deliberately claims no standing no-ask permission.** If your global `CLAUDE.md`
pre-authorizes identity posts for some *other* skill (e.g. `pr-review-loop`), that grant is scoped to
that skill's own actions, so extending it here would be a self-granted exception rather than an
inherited one. Widening it is an amendment to your own operating instructions — an operator's change
to make deliberately, not a default this skill assumes.

**The one ask covers every identity post the round makes:** review comments and verdicts, handoff
comments, and the follow-up issues Step 7 files on your own repositories for items that don't fit
their PR (deferred design-pins, out-of-band follow-ups).

**Declining does not stop the run.** Reviews still run and your own branches still get their fixes;
every identity post — comments, verdicts, and issues alike — is withheld and listed at the end for
manual action. The review lane never files issues on someone else's repository — findings stay in
the comment thread rather than pre-empting the maintainer's triage.

## When NOT to use

- Driving a single PR to a merge bar — that is `pr-review-loop`.
- A PR still moving in draft; scope is not stable enough for a useful review.
- Fork PRs you cannot push to (the remediate lane skips them with a reason).
- A PR in a repository not checked out under your dev home — Step 4.1 maps only the dev home and its
  `projects/` children, so anything else skips with `no local checkout mapped`. Clone-on-demand is a
  tracked follow-up, not built.
- Reviewing **your own** PR as its sole maintainer — the review lane fires only on PRs authored by
  someone else, and the remediate lane ingests *existing* feedback rather than generating a review, so
  a solo-maintained PR with no reviewer gets no review here (the round runs as a remediate no-op). Use
  `pr-review-loop <N>` for a fresh outsider review today. Once a review-bot identity is provisioned
  (per the lab's agent-runtime rule, where adopted), running `/pr-round` from it routes your PRs into
  the review lane — and that identity is what makes the formal verdict postable, since GitHub forbids
  self-approval.
- Replying to a specific review thread, or out-of-band actions beyond issue filing. It posts exactly
  one comment per PR; on **your own** PRs it additionally files follow-up issues for items that don't
  fit the PR (Step 7, consent-gated), and on anyone else's repository it files nothing — anything
  else is returned as a follow-up for you.
- Merging. `pr-round` never calls `gh pr merge`.

## Claiming compliance

Reporting that you "ran `pr-round`" is truthful only if Steps 0–11 actually executed — real subagent
dispatch, real posted comments, a real roster. If any step was approximated, skipped, or hand-rolled,
enumerate every deviation **up front**, in the same message, before any claim about what the round
accomplished. This is `.claude/rules/01-workflow.md` § Claiming compliance applied to this skill;
that clause is the owning rule.

## Deployment

lab-os-owned, user-scope-deployed. Authored at `<DEV_ROOT>/.claude/skills/pr-round/`; deployed by
`bash .claude/scripts/link-lab-assets.sh`, which symlinks it into `~/.claude/skills/`. Re-run it on
each clone. Deployment matters here: the PRs in scope span repositories, and a nested project repo's
own root blocks the `.claude/` walk-up to the dev home.

## See also

- [PROMPT.md](PROMPT.md) — the executable body, Steps 0–11
- [reference/rubric-layering.md](reference/rubric-layering.md) — tier discovery, precedence, cold-start
- [reference/rubric-universal.md](reference/rubric-universal.md) — tier 1, the always-present standard
- [reference/classify-blockers.md](reference/classify-blockers.md) — mechanical vs design-pin
- [reference/review-comment-template.md](reference/review-comment-template.md) — review comment; owns the marker literal
- [reference/remediation-comment-template.md](reference/remediation-comment-template.md) — the handoff comment
