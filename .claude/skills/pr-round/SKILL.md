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
| **yours** | **remediate** | Ingests existing review feedback → auto-fixes mechanical findings → asks you about **every** design-pin it found → applies → posts one handoff comment |

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

## The specialist panel (review lane only)

Beside the composed rubric, the review lane dispatches the **specialist review agents** in
`<DEV_ROOT>/.claude/agents/` (test coverage, silent failures, type design, comment rot) for
dimensions the PR's diff triggers.
**`<DEV_ROOT>/reference/specialist-dispatch.md` is the owning contract, consumed in place**: the
diff-trigger table, per-pass cap, model tier, severity-ceiling knob, and merge rules all live
there.

- **Dispatched from the main loop, in the review-lane PR's wave** — a subagent cannot spawn
  agents, so the specialists run as siblings of that PR's review agent, and their findings return
  to the main loop.
- **Composed before the verdict — and before the operator approves it.** Step 9.0 merges specialist
  findings into the review comment under the three-tier rubric (dedup on `file::symbol` + category;
  cross-category collisions get a recorded same-defect judgment) and re-derives the verdict token
  from the merged findings per the template's trigger table. Step 10 posts that already-merged result
  and never re-derives the token — so the verdict the operator approved is the verdict that lands,
  and it reflects the full panel.
- **Degrades like a missing rubric tier.** Where the dispatch reference does not resolve, the run
  goes specialist-less and the posted comment names the absent layer — one degradation pattern,
  not two. A specialist that errors or returns schema-invalid output is named in the comment as a
  not-run dimension; the review never silently narrows.
- **The remediate lane is untouched** — specialists review, they never remediate.
- **`--no-specialists`** drops the panel for the whole round.

**Availability note:** the agents and the dispatch reference are **not yet carried in lab-os** —
they are maintained in the lab's workspace forks, and a bare clone of this repository resolves
neither. Until they are ported, a run rooted here degrades to the composed rubric alone and says so;
a run from a dev home that carries them gets the full panel, because both paths resolve against
`DEV_ROOT` at runtime rather than against the repository the skill ships from.

## Two fan-outs, and why

Subagents cannot call `AskUserQuestion`. So a design-pin found during remediation can only be
*returned*, decided in the main loop, and applied by a second dispatch:

```
Step 5  fan out #1  → review (returns comment + verdict) | remediate (fixes, returns pins)
Step 6  decide      → batched questions until the pin queue is drained; every one carries Defer
Step 7  compose     → follow-up issues for deferred + out-of-band items (own PRs) — composed, not filed
Step 8  fan out #2  → apply decisions, reusing the Step-5 worktree
Step 9  confirm (9.0) → file the composed issues (9.0a) → ONE handoff comment per remediated PR (9.1)
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
| Design-pin, on your PR | Returned from the subagent, asked at Step 6 with 2–3 options + `Defer`, applied at Step 8. **Step 6 drains the queue** — batches of at most 4 questions, uncapped batch count, until every pin is decided or deferred *by you*. |
| Anything on someone else's PR | Reported in the review comment. Never edited — it is their PR. |
| The reviewed PR's head moved before the verdict posts | The head is re-checked immediately before posting; a sha that no longer matches the reviewed one **forces the verdict to `COMMENT`** and adds a note naming both shas. Deterministic, no interrupt — a formal approve or request-changes is never posted against commits nobody read. The round does not re-review them. |
| Roster over 8 PRs | One cost-guard question before dispatch, offering repo- and lane-scoped cuts before a bare count cap. |
| Pin queue over 8 | One volume-guard question before the batches (same shape as the roster guard): decide the ones on otherwise-merge-ready PRs, decide all, decide Blockers only, or defer all. Everything the answer excludes is recorded as **deferred by you** — the guard is you exercising the deferral, not the skill skipping the ask. |
| A pin the round never asked about | **A skill failure, not an outcome.** Named as its own class in the handoff comment, forces 🔴 `BLOCKED`, and reported separately from deferrals in the roster. |
| Your PR conflicts with its base | Remediate lane skips it, reason `merge conflict — manual`, named in the roster. Detected, never resolved — a semantic merge under your identity is not the skill's to guess. Review-lane PRs are unaffected. (Default; `--merge-base` adds the bounded case below.) |
| Your PR is merely *behind* its base, and `--merge-base` is set | The lane merges the base into the PR's own worktree and proceeds **only** on a zero-conflict merge. One conflict hunk → `merge --abort` and the skip above, reason naming that the merge was attempted. Never resolves; only closes the stale-branch case. The pushed head then carries a merge commit no agent wrote and no reviewer read, so the Step 9.0 pre-post summary flags it per PR and the handoff comment names it. |
| Red gate, or a gate that cannot run at all | No push. The commit survives; worktree preserved, path named in the roster. |
| A resolution the skill cannot perform | Returned as an out-of-band follow-up — on your own PRs, composed as a follow-up issue at Step 7 and filed at 9.0a under the same per-PR consent as the comments; otherwise recorded in the comment and the roster for manual action. |

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
          [--dry-run] [--concurrency N] [--no-specialists] [--comment-only] [--hand-back]
          [--merge-base]
```

`--dry-run` resolves and reports the roster and makes no write call of any kind — the safe first
invocation. `--no-skip` disables the no-new-activity filter, which otherwise skips PRs untouched
since this skill's last comment so a re-run costs one API call instead of a model pass.
`--comment-only` posts every review as a `COMMENT` verdict whatever the Blocker count — for reviewing
from an identity without merge authority (the findings still post in full; only the formal
approve/request-changes claim is withheld). `--hand-back` re-requests review and clears draft after a
successful remediation (off by default — see *When NOT to use*). `--merge-base` lets the remediate
lane merge the PR's base branch into its worktree when the branch is behind or conflicting, and
proceed **only** on a zero-conflict merge — any conflict aborts and the PR skips as it would without
the flag. Also off by default, for the same reason as `--hand-back`: it writes.

## Consent

**Two asks, each authorizing what the other cannot** — each fired once per run, with per-PR
granularity available inside the second. `PROMPT.md` § Step 3 and § Step 9.0 are the owning
definitions; this is a summary.

*Two **consent** asks — not two questions.* The round also asks the cost guard (roster over 8 PRs),
the pin volume guard (queue over 8), and one question per design-pin it is deciding. Those authorize
nothing and are not capped by this section: Step 6 keeps asking until every pin is decided or
deferred, because an un-asked pin is a failure and a `Defer` answer is cheap.

- **Step 3, before any dispatch — authorizes the *run*.** Reviews, and the commits and pushes the
  remediate lane makes to your own branches. It has to come first, because those writes happen inside
  the dispatch it gates. But it necessarily fires before any comment body, verdict, or final sha
  exists, so it can only describe the posts to come — not show them.
- **Step 9.0, before anything posts or is filed — authorizes the *posts*, run-wide or per PR.** Once
  the round knows exactly what it would say and where, it prints a per-PR summary (each PR's verdict
  or readiness, what lands where, how many follow-up issues it would file, and — where `--merge-base`
  merged one — that a base merge landed on that branch) and asks once, with four options: post all,
  **show every full body first** and re-ask, **choose per PR** (batched selection; `POST_OK` is a
  per-PR set), or withhold all. This is the ask that governs the identity comments, the formal
  verdicts, **the follow-up issues**, and — under `--hand-back` — the review re-requests and draft
  clearing, each enumerated in the question rather than named by flag.

  Step 7 *composes* the follow-up issues before Step 8 so the handoff comment can cite their numbers;
  **9.0a**, immediately after this gate, is what actually creates them. Creating an issue under your
  identity is as irreversible as posting a comment, so it sits behind the same body view and the same
  per-PR flag rather than beside them.

A Step-3 decline withholds everything and makes Step 9.0 moot. A Step-9.0 withhold — global or per
PR — keeps the already-pushed commits but sends the affected posts to the withheld list, labelled
run-wide vs per-PR choice so the roster never conflates the two. Either way the round still reviews
and still fixes your branches, and everything withheld is listed at the end for manual action.

**This skill deliberately claims no standing no-ask permission.** If your global `CLAUDE.md`
pre-authorizes identity posts for some *other* skill (e.g. `pr-review-loop`), that grant is scoped to
that skill's own actions, so extending it here would be a self-granted exception rather than an
inherited one. Widening it is an amendment to your own operating instructions — an operator's change
to make deliberately, not a default this skill assumes.

The review lane never files issues on someone else's repository — findings stay in the comment thread
rather than pre-empting the maintainer's triage.

## When NOT to use

- Driving a single PR to a merge bar — that is `pr-review-loop`.
- A PR still moving in draft; scope is not stable enough for a useful review.
- Fork PRs you cannot push to (the remediate lane skips them with a reason).
- A PR in a repository not checked out under your dev home — Step 4.1 maps the dev home itself, its
  `projects/<name>` children, and any repo sitting as a **direct child** of the dev home, taking the
  first that is a git repo root (so a flat layout works, though `projects/` is the blessed one — see
  *Deployment*). A repo that is under none of those still skips with `no local checkout mapped`;
  clone-on-demand is a tracked follow-up, not built.
- Reviewing **your own** PR as its sole maintainer — the review lane fires only on PRs authored by
  someone else, and the remediate lane ingests *existing* feedback rather than generating a review, so
  a solo-maintained PR with no reviewer gets no review here (the round runs as a remediate no-op). Use
  `pr-review-loop <N>` for a fresh outsider review today (not yet carried in lab-os — see the
  Availability note above). Once a review-bot identity is provisioned
  (per the lab's agent-runtime rule, where adopted), running `/pr-round` from it routes your PRs into
  the review lane — and that identity is what makes the formal verdict postable, since GitHub forbids
  self-approval. Once the rules-parity sync (#58) lands `.claude/rules/05-agent-runtime.md` here, this
  identity posture derives from its § Identity & enablement (bot identity for agent output;
  `acts-as-operator` reserved for the operator's own) rather than being self-owned by this section.
- Replying to a specific review thread, or out-of-band actions beyond issue filing. It posts exactly
  one comment per PR; on **your own** PRs it additionally files follow-up issues for items that don't
  fit the PR (composed at Step 7, filed at 9.0a under the same per-PR consent), and on anyone else's
  repository it files nothing — anything else is returned as a follow-up for you.
- Editing a PR's title or body — its own or anyone's. The round's write surfaces are enumerated:
  commits and pushes to your own branches, one handoff comment, one review comment plus formal
  verdict, follow-up issues. The PR body is the author's narrative surface (the lab's logging rule
  routes session narrative there) and `gh pr edit --body` is not among the surfaces; a body gone
  stale against the remediated diff is returned as an out-of-band follow-up naming what to update,
  never rewritten under your identity.
- Taking a PR out of draft, or re-requesting review, **without `--hand-back`**. The skill never leaves
  draft state on its own; `--hand-back` (Step 9.2, gated by the Step 9.0 posting consent) is the only
  path that clears draft or re-requests review, and only on a PR the round actually advanced.

  **Know what the default costs you.** A PR sitting at `CHANGES_REQUESTED` **stays there after every
  one of its findings is resolved** — GitHub clears that state only when a reviewer re-reviews, and
  the remediate lane will not re-request one unprompted. So the default run ends with a fixed,
  pushed, gate-green PR that still reads as blocked to everyone looking at the queue, and nothing
  moves until you re-request by hand. **Not hypothetical:** the PR that added this skill sat exactly
  there — every reviewer finding resolved, still `CHANGES_REQUESTED`, waiting on a re-request nobody
  had made. Opt into `--hand-back` when you want the round to close that loop; leave it off when you
  want to read the handoff comment before a reviewer is notified. Either is defensible — silently
  getting the first outcome while expecting the second is not.
- **Dismissing a review — its own or anyone else's.** The skill never calls
  `gh api -X PUT .../reviews/<id>/dismissals`, in either lane, on any PR including your own.
  `--hand-back`'s re-request (Step 9.2) is the only sanctioned path out of `CHANGES_REQUESTED`, and
  the evidence bar is the reviewer re-reviewing — not the round judging its own fixes sufficient. This
  bullet sits next to the one above deliberately: the stuck-at-`CHANGES_REQUESTED` cost named there is
  exactly the pressure that makes dismissal look like the tidy fix, and dismissing a standing review to
  clear a state the round did not earn erases a reviewer's open finding from the PR's face.
- Resolving a merge conflict. A conflicting PR is detected and skipped `merge conflict — manual`, not
  merged — resolving two people's intent under your identity is not the skill's call. `--merge-base`
  does not change that: it attempts the base merge in the PR's worktree and proceeds only when the
  merge produces **zero** conflict hunks, aborting to the same skip otherwise. It closes the
  stale-branch case; a real conflict still ends the lane for that PR.
- Merging. `pr-round` never calls `gh pr merge`.

## Claiming compliance

Reporting that you "ran `pr-round`" is truthful only if Steps 0–11 actually executed — real subagent
dispatch, real posted comments, a real roster. If any step was approximated, skipped, or hand-rolled,
enumerate every deviation **up front**, in the same message, before any claim about what the round
accomplished. The rule above is **self-owned here**: lab-os's `.claude/rules/01-workflow.md` carries
no § Claiming compliance clause yet — it arrives with the rules-parity sync (#58). Once it lands, this
section becomes a derivation of it rather than the owning statement.

## Deployment

lab-os-owned, user-scope-deployed. Authored at `<DEV_ROOT>/.claude/skills/pr-round/`; deployed by
`bash .claude/scripts/link-lab-assets.sh`, which symlinks it into `~/.claude/skills/`. Re-run it on
each clone. Deployment matters here: the PRs in scope span repositories, and a nested project repo's
own root blocks the `.claude/` walk-up to the dev home.

**lab-os's own tier-2 gap is transitional, not a designed steady state.** A run rooted in a bare
lab-os clone degrades to tier-2-absent because lab-os does not yet carry
`reference/code-quality-taxonomy.md` (see *The composed rubric* for what tier 2 is and how absence
degrades — not restated here). That is closing at the source: the rules-parity sync (#58) copies the
taxonomy into lab-os at that same `reference/` path, after which a lab-os-rooted run resolves tier 2
like any dev-home run — still consumed in place, since the skill vendors no rubric of its own. Read
the degradation as a gap being closed, not a gap being blessed.

**Blessed dev-home layout: project repos nest under `<DEV_ROOT>/projects/<name>`.** Step 4.1 resolves
a PR's repo to `<DEV_ROOT>/projects/<name>` first, so a dev home that follows this convention is
unambiguous. A repo sitting as a direct child `<DEV_ROOT>/<name>` also resolves — the `projects/`-first
ordering states the preference without forbidding a flat checkout, so an existing flat dev home keeps
working while a new one should prefer `projects/`.

## See also

- [PROMPT.md](PROMPT.md) — the executable body, Steps 0–11
- [reference/rubric-layering.md](reference/rubric-layering.md) — tier discovery, precedence, cold-start
- [reference/rubric-universal.md](reference/rubric-universal.md) — tier 1, the always-present standard
- `<DEV_ROOT>/reference/specialist-dispatch.md` — specialist panel: triggers, cap, merge/verdict contract (consumed in place; not carried in lab-os — absent → degrade like a missing tier)
- [reference/classify-blockers.md](reference/classify-blockers.md) — mechanical vs design-pin
- [reference/review-comment-template.md](reference/review-comment-template.md) — review comment; owns the marker literal
- [reference/remediation-comment-template.md](reference/remediation-comment-template.md) — the handoff comment
