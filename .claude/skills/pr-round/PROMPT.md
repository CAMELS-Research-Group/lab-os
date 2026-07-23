# pr-round — executable body

> **Structure.** One round per PR, then stop. There is no loop and no cross-run state.
>
> ```
> Step 0  isolate main loop -> parse args -> resolve skill root + dev root -> establish identity
> Step 1  resolve roster
> Step 2  classify by ownership + filter
> Step 3  resolve consent              (needs Step 2's authors + permissions)
> Step 4  map repo -> local checkout + resolve rubric tiers -> cost guard
> Step 5  FAN OUT #1: review lane | remediate lane   (neither posts)
> Step 6  decide returned design-pins       (main loop)
> Step 7  file follow-up issues             (main loop; own PRs only, consent-gated)
> Step 8  FAN OUT #2: apply decisions       (reuses Step-5 worktrees)
> Step 9  post one handoff comment per remediated PR   (main loop)
> Step 10 post review comment + verdict per reviewed PR  (main loop)
> Step 11 roster summary + worktree cleanup  (MUST run after Steps 8-10)
>
> Every `gh` post is a main-loop step. No subagent posts under the operator's identity.
> ```

## Step 0: Isolate, parse arguments, resolve paths, establish identity

### 0.0 Isolate the main loop before anything writes

Some hosts run a session behind a **write guard**: file edits anywhere in the shared checkout are
rejected until the session isolates itself into a worktree. The guard is a **session-level latch, not
a per-write path check**, and subagents inherit the session's state rather than carrying their own. So
one latch decides the entire run's ability to write:

- Parent isolated → every remediate agent writes, commits, and pushes normally in its own PR
  worktree. That is the design's fast path and nothing about it changes.
- Parent **not** isolated → *every* agent's first `Edit` is rejected, on every PR at once. The
  remediate lane fails wholesale, mid-round, after the roster and consent are already resolved and
  after the review lane has spent its passes.

Because the latch is set once and inherited, closing it costs one call at the top of the run and
removes that failure mode entirely. **Call `EnterWorktree` here, before any dispatch**, unless the
session is already isolated — its working directory already sits under a `.claude/worktrees/` path.

This worktree is the **coordinator's** workspace, not a PR worktree. The main loop remains a
coordinator and never becomes the applier; Step 5.1's per-PR worktrees are separate and unaffected by
it. On a host with no such guard, entering a worktree is harmless — so this is unconditional rather
than something the skill tries to detect. Skip it under `--dry-run`, which writes nothing.

### 0.1 Resolve the skill root and the dev home

Every path handed to a subagent must be **absolute**: subagents run in worktrees of possibly
different repositories, so a relative literal will not resolve. Resolve two roots here, before
anything else, and record both as absolute paths.

**`SKILL_ROOT`:**

1. Try the user-scope skills directory first: `~/.claude/skills/pr-round`.
2. Fall back to `.claude/skills/pr-round` under the current repository, for a session already
   running inside the dev home checkout.
3. Neither readable → abort: `❌ cannot locate pr-round skill root — run link-lab-assets.sh`.

`reference/` files are `SKILL_ROOT/reference/<name>`.

**`DEV_ROOT`** — the dev home this skill is authored in. Tier 2 of the rubric and the Step 4.1 repo
mapping both need it, and **it is not reachable from `~/.claude/`**: `link-lab-assets.sh` links only
`skills/`, `commands/`, and `workflows/`, so `~/.claude/reference/` never exists. Derive it from the
skill root:

1. Resolve `SKILL_ROOT` through any symlink to its real target (`readlink -f`, `realpath`, or the
   platform equivalent). The user-scope entry is a symlink into the dev home; following it lands in
   the authored copy.
2. `DEV_ROOT` is that real path with the trailing `.claude/skills/pr-round` stripped — three levels
   up.
3. Check each tier-2 component **separately**: `DEV_ROOT/reference/code-quality-taxonomy.md` and
   `DEV_ROOT/.claude/rules/`.

A `DEV_ROOT` that cannot be derived makes tier 2 **absent** — not silently empty. Otherwise tier 2
resolves **per component**: each half found in (3) is in scope, each half missing is recorded absent
with the path that was checked — a repo carrying `.claude/rules/` but no taxonomy still gets its
rules applied (tiers accumulate; see `SKILL_ROOT/reference/rubric-layering.md` § Precedence). The
degradation rule there applies per absent component. Never guess a dev-home path; a wrong one
produces a review that claims a standard it did not read. Only an unresolvable `SKILL_ROOT` aborts
the run.

### 0.2 Parse `$ARGUMENTS`

**Positional — zero or more PR refs.** Four accepted forms:

| Form | Example | Resolves to |
|---|---|---|
| `<repo>#<N>` | `lab-os#42` | the repo of that name under the known owner |
| `<owner>/<repo>#<N>` | `CAMELS-Research-Group/lab-os#42` | exactly that |
| pull URL | `https://github.com/CAMELS-Research-Group/lab-os/pull/42` | exactly that |
| bare number | `421` or `#421` | that PR in the **current** repo |

Unparseable ref → abort naming the ref and listing all four forms. Do not guess.

**Flags:**

| Flag | Default | Effect |
|---|---|---|
| `--limit N` | none | Cap the roster at the N most-recently-updated PRs |
| `--no-skip` | off | Disable the no-new-activity filter (Step 2) |
| `--review-only` | off | Drop the remediate lane |
| `--remediate-only` | off | Drop the review lane |
| `--dry-run` | off | Resolve and report the roster; make **no** write call of any kind |
| `--concurrency N` | 5 | Wave size — how many subagents run at once (Step 5, Step 8) |

Unknown flag → abort: `❌ Unknown flag: <flag>`. `--review-only` with `--remediate-only` → abort;
they are mutually exclusive.

### 0.3 Establish identity

`gh api user --jq .login` → `VIEWER`. A read-only call, so it runs under `--dry-run` too: Step 2.1
classifies every lane on `author == VIEWER`, and the roster is the whole of a dry-run's output.

**Consent is resolved at Step 3, not here.** Its predicate is over the surviving roster, which
Steps 1 and 2 produce. Consent still lands before any dispatch, which is the property that matters.

### 0.4 Emit the banner

```
🔁 pr-round
  Scope:   <N explicit refs | all connected PRs>
  Lanes:   <review + remediate | review only | remediate only>
  Mode:    <live | dry-run>
```

Consent is not known yet — Step 3 emits its own line once it resolves.

## Step 1: Resolve the roster

**With explicit refs:** the roster is exactly those PRs. Run no search — an explicit ref is a
direct instruction and must not be filtered by activity or draft state.

**With no refs:** union of

- `gh search prs --author=@me --state=open --json repository,number,title,updatedAt,isDraft`
- `gh search prs --review-requested=@me --state=open --json repository,number,title,updatedAt,isDraft,author`

Dedupe by `<owner>/<repo>#<number>`. A PR both authored by you and review-requested of you resolves
to the **remediate** lane — you cannot review your own PR, and GitHub will not accept the verdict.

Apply `--limit N` here, keeping the most recently updated. When it truncates, record how many were
dropped; Step 11 reports it. A silent cap reads as full coverage.

## Step 2: Classify by ownership, then filter

For each PR resolve `author`, `isDraft`, `headRefName`, `headRepositoryOwner`, and the comment
timeline:

```
gh pr view <N> --repo <R> --json author,isDraft,headRefName,headRepositoryOwner,comments,reviews,commits
```

`headRefName` is `<head-ref>` — the value Steps 5.1, 5.3, Step 8, and 11.2 all consume. Resolve it here
or those commands have no value to substitute.

### 2.1 Lane

| Condition | Lane |
|---|---|
| `author == VIEWER` | **remediate** — ingest existing feedback and act on it |
| `author != VIEWER` | **review** — produce a review and a verdict |

Apply `--review-only` / `--remediate-only` by dropping the other lane, recording a skip reason.

### 2.2 Drafts

Drop draft PRs with reason `draft`, **unless** named by an explicit ref — an explicit ref overrides.

### 2.3 No-new-activity filter

Skipped when `--no-skip` is set, and never applied to explicit refs.

1. Find the newest comment whose body starts with this skill's marker prefix, read from
   `SKILL_ROOT/reference/review-comment-template.md` § Machine marker — the token is defined there
   and is not repeated here, so a version bump lands in one place. None → never processed;
   **keep it**.
2. Compare that comment's timestamp against the newest of: last commit, last review submission
   **not authored by `VIEWER`**, last non-marker comment.
3. Nothing newer → **skip**, reason `no new activity since <ISO timestamp>`.

**Excluding `VIEWER`'s own reviews is what makes the filter reachable at all.** Step 10 posts the
marker comment and submits a formal `gh pr review` verdict, both under `VIEWER`, so the skill's own
review is always newer than its own marker. Counting it would make the skip branch dead code on the review
lane — every re-run would re-review every untouched third-party PR and post a second comment and a
second verdict under the operator's identity. The comment class already carves the skill's output out
("non-marker comment"); the review class has to as well. `VIEWER` cannot review their own PR, so this
exclusion costs nothing on the remediate lane.

The point is that a re-run costs one `gh` call on an untouched PR instead of a full model pass.

### 2.4 Fork PRs

A remediate-lane PR whose head is a fork the viewer cannot push to → skip, reason
`fork head not pushable`. Detect via `headRepositoryOwner` before dispatch, not by a failed push.

## Step 3: Resolve consent (once, for the whole run)

Runs here — after Step 2, before any dispatch — because the roster it describes to the operator is
not known earlier. Skip entirely under `--dry-run`: it writes nothing, so it needs no permission.

**This skill claims no standing permission.** Every run that will post anything asks, once. If the
operator's global `CLAUDE.md` pre-authorizes identity posts for some *other* skill (e.g.
`pr-review-loop`), that grant is scoped to that skill's own actions — such clauses typically close by
narrowing themselves ("any other identity-posting action still needs per-action confirmation") — so a
no-ask path here would be an extension the operator has not made, not a gap in one. Extending it is
an amendment to that document: a separate, operator-made change, not something this skill grants
itself. Until then the dev home's `.claude/CLAUDE.md` § Approval gates governs, and it is
unambiguous: anything under your name is gated.

**Fire the ask iff the surviving roster contains at least one PR this run would post to.** Nothing to
post → no ask. Otherwise exactly one question for the whole run, never one per PR:

- `question`: `This run will post under your GitHub identity: a review comment and a formal approve / request-changes verdict on each PR you do not own, one handoff comment on each of your own PRs it remediates, and a follow-up issue on your own repositories for each item that does not fit its PR (deferred decisions, out-of-band follow-ups). Approve?`
- `header`: `Run consent`
- options: `Accept (Recommended)` → `CONSENT = true`; `Decline (no identity posts)` → `CONSENT = false`.

**A post is authorized iff `CONSENT == true`.** That one rule governs every posting site — Steps 7,
9, and 10 defer to it rather than restating it.

**Declining does not stop the run.** Review and remediation both still execute; remediation still
commits and pushes to your own branches, which running the skill already authorizes. Every withheld
post is listed in the Step 11 summary so nothing is silently dropped.

The review lane never files issues on someone else's repository — findings live in the comment
thread. Filing there would pre-empt the maintainer's own triage. Issue filing exists only for **your
own** PRs, at Step 7, and is governed by this same single consent — no second ask.

Emit: `Consent: <granted | declined | not needed — nothing to post | n/a (dry-run)>`.

## Step 4: Map repositories, resolve rubric tiers, check cost

### 4.1 Local checkout

Map each `<owner>/<repo>` to a local path:

- `DEV_ROOT` itself for the dev home's own repo.
- `DEV_ROOT/projects/<name>` for a nested project repo.

Both need `DEV_ROOT` from Step 0.1. If it could not be derived, no repository maps and every PR
skips with reason `no local checkout mapped` — report that as the derivation failure it is, not as
a per-repo mapping gap.

Verify the path exists and `git -C <path> rev-parse --is-inside-work-tree` is true. Unmappable →
skip with reason `no local checkout mapped`. Never silently.

### 4.2 Rubric tiers

Resolve per repository, per `SKILL_ROOT/reference/rubric-layering.md`, which owns the paths,
precedence, and degradation rules. Record each tier resolved-with-path or absent-with-path-checked;
the resolved set travels in the subagent brief and is reported in the review comment.

A missing tier never fails the run. Collect the cold-start prompt from
`SKILL_ROOT/reference/rubric-layering.md` for each missing tier and hand it to Step 11.

### 4.3 Cost guard

If the surviving roster exceeds **8** PRs, stop and ask before dispatching — each PR costs a full
model pass and the operator is cost-conscious:

- `question`: `Roster is <N> PRs (<R> review, <M> remediate) across <repos, with per-repo counts>. Each costs a full review pass. Proceed?`
- `header`: `Roster size`
- options — **narrow by shape before narrowing by count**, and offer at most 4:
  1. `<repo> only (<n> PRs)` — one option per repository when the roster spans more than one, largest
     first; when the roster is single-repo, `Review lane only (<R> PRs)` takes this slot, which
     unblocks collaborators first and leaves your own PRs for a later run
  2. `Limit to the 5 most recent (Recommended)`
  3. `Proceed with all <N>`
  4. `Abort`

  Drop `Proceed with all` first if repo-scoping would push past four options. A count-only cap slices
  across lanes and repositories arbitrarily, and an operator whose 14-PR roster spans two repos is
  almost always asking about one of them — so stating the breakdown in the question and offering the
  scoped cuts saves the follow-up round-trip that a bare `all / 5 / abort` forces.

Under `--dry-run`, report the roster and **stop here**. Steps 5–11 do not run; there is nothing to
report that the roster does not already contain.

## Step 5: Fan out — one subagent per PR

Dispatch in **waves of `--concurrency`** (default 5). Issue one wave's calls **in a single message**
so that wave runs concurrently, then block on the whole wave before starting the next. Do not run
agents in the background.

Waves are the throttle: issuing every agent in one message would run the entire roster at once,
which is what the flag exists to prevent.

### 5.1 Shared brief (both lanes)

Every brief carries this, with only the lane section differing:

- **Absolute paths only.** `SKILL_ROOT/reference/<file>` for contracts; the mapped checkout path for
  the repository. A relative literal is a defect — the agent's working directory is its own worktree,
  in a possibly different repo.
- **Worktree — one per PR, on its own branch, tracking the remote.** Several PRs map to one
  repository and several agents run at once, so each needs an isolated checkout *and* an isolated
  branch: a given branch name can be checked out in only one worktree, so reusing the PR's own
  branch name would make concurrent PRs in one repo collide, and would fail outright whenever that
  branch is already checked out in the main working tree.

  **Same-repo head** (every remediate-lane PR, since 2.4 drops fork heads from that lane):

  ```
  git -C <repo-root> fetch origin <head-ref>
  git -C <repo-root> worktree add -b pr-round-<N> <path> origin/<head-ref>
  ```

  Branching from the freshly fetched remote-tracking ref sets the new branch's upstream to
  `origin/<head-ref>`, so the worktree tracks the remote — `git status` reports ahead/behind against
  the real PR branch rather than sitting detached at a frozen sha. Set the upstream explicitly if the
  local git config does not.

  **Fork head** (review lane only). `origin/<head-ref>` does not exist for a fork, and fetching
  `refs/pull/<N>/head` bare updates only `FETCH_HEAD` — the default refspec does not create a
  remote-tracking ref for it, so `origin/refs/pull/<N>/head` would not resolve. Fetch it into an
  explicit ref and branch from that:

  ```
  git -C <repo-root> fetch origin refs/pull/<N>/head:refs/remotes/origin/pr/<N>
  git -C <repo-root> worktree add -b pr-round-<N> <path> origin/pr/<N>
  ```

  A fork worktree tracks `origin/pr/<N>` — the PR head as GitHub exposes it, not the contributor's
  branch, which this repository has no ref for. It is read-only by nature; nothing pushes here.

  `<path>` is `<WORKTREE_ROOT>/pr-round-<N>` in both cases. `<WORKTREE_ROOT>` defaults to
  `<repo-root>/.claude/worktrees`; when the `PR_ROUND_WORKTREE_ROOT` environment variable is set, it
  is `<PR_ROUND_WORKTREE_ROOT>/<repo-name>` instead — namespaced by repository so two repos' PRs of
  the same number cannot collide in one shared root.

  The override exists because the default sits **inside** the repository, and for a **nested project
  repo** (`DEV_ROOT/projects/<name>`) that also puts every PR worktree inside the dev home's own
  checkout. Two host conditions make that the wrong place: a path-length ceiling that the nested
  `projects/<name>/.claude/worktrees/pr-round-<N>/…` prefix pushes builds past, and any dev-home-wide
  guard or tool that governs everything under the outer checkout and therefore reaches into the
  nested repo's worktrees too. Neither is this skill's to detect, and hardcoding either would bake one
  host's facts into a portable skill — so the escape hatch is one env var the operator sets once per
  host. Report the root actually used in the Step 11 summary whenever it is not the default.

  **A `pr-round-<N>` branch or `<path>` may already exist** — Step 11.2 preserves both when a round
  ends with work in flight, and a crashed run leaves them too. Do not let `worktree add` die on a raw
  `fatal: a branch named … already exists`. Resolve in this order:

  1. `git -C <repo-root> worktree prune` to clear registrations whose directory is gone.
  2. Branch exists and points at this PR's head, with a clean worktree → **reuse it**; skip creation.
  3. Otherwise → **fail this PR** with reason `preserved worktree from a prior run at <path> —
     integrate or delete it`. A preserved worktree holds unintegrated work; silently resetting or
     deleting it would discard exactly what 11.2 preserved it to protect.

  Record `git -C <path> rev-parse HEAD` **after** checkout as the authoritative reviewed sha; it is
  what the comment marker reports, and the fetch may have advanced past the sha Step 2 saw.

  Work only inside `<path>`. Do **not** remove the worktree or its branch — Step 11 owns cleanup, and
  Step 8 may still need both.
- **Rubric tiers.** The resolved tier list from Step 4.2, each as an absolute path, plus which were
  absent. Read tier 1 always; read higher tiers when resolved. Apply precedence per
  `SKILL_ROOT/reference/rubric-layering.md`.
- **No `AskUserQuestion`.** A subagent cannot interrupt. Anything needing a decision is *returned*,
  never guessed at.
- **Values the brief must carry.** Every placeholder the agent's commands substitute, as a literal:
  `<N>` (PR number), `<R>` (`<owner>/<repo>`), `<repo-root>` (the mapped checkout, absolute),
  `<head-ref>` (`headRefName` from Step 2), `<path>` (the worktree, absolute), and the base ref the
  worktree was created from — `origin/<head-ref>` or, for a fork, `origin/pr/<N>`. An agent that has
  to infer any of these is improvising a git command against someone's repository.
- **Structured return.** Report: `pr`, `lane`, `headSha`, findings by severity with each finding's
  location / detail / `mechanical|design-pin` classification, actions taken, the composed review
  comment body and verdict token (review lane) or the returned design-pins (remediate lane),
  readiness, the **gate rung and result** (5.3 step 5), `applyMode` (`applied` or `analysis-only` per
  5.5, with the edit set when analysis-only), any **out-of-band follow-ups** (5.3 step 8), tiers
  applied, and any failure reason. No `comment URL` — a subagent posts
  nothing, so the URL is produced by the main-loop `gh pr comment` in Step 9 or Step 10, not returned here.

### 5.2 Review lane

> **This lane posts nothing.** It composes the comment and the verdict and **returns both**; Step 10
> posts them from the main loop. `CONSENT` is main-loop state produced by an `AskUserQuestion` a
> subagent is forbidden to issue (5.1), so a guard evaluated *here* would have no operand — the agent
> would have to pick a default, and both defaults are wrong: posting anyway puts an irreversible
> comment and formal verdict under the operator's identity on a third party's PR after they declined,
> while withholding silently drops every review post on a run they approved. Keeping the decision
> where its input lives is the only shape with no such default.

1. Read `SKILL_ROOT/reference/review-comment-template.md` first and follow it exactly.
2. Review **this PR's diff** against the resolved tiers. The diff is `gh pr diff <N> --repo <R>` —
   GitHub computes it against the PR's real merge base, so the review needs no base ref and no clone
   arithmetic. Do **not** diff against `origin/<head-ref>`: the worktree was branched from exactly
   that ref, so that diff is empty and the review would silently pass over everything. Tag structural
   findings `[regression]` / `[simplification]`; leave correctness, security, error-handling, test,
   and doc findings untagged. On a diff that touches no code, skip structural tagging.
3. Classify each finding mechanical vs design-pin per
   `SKILL_ROOT/reference/classify-blockers.md`, so the author can see which findings are a small edit
   and which need them to choose. **Fix nothing** — this is someone else's PR.
4. Return the composed comment body verbatim.
5. Return the verdict as one of the template's three tokens — `REQUEST CHANGES` / `APPROVE` /
   `COMMENT` — plus a one-line rationale. The verdict is the template's to decide
   (`review-comment-template.md` § Verdict vocabulary owns the trigger table); do not re-derive it
   here, and do not submit it.

### 5.3 Remediate lane

1. **Ingest all existing feedback:** review bodies, inline review comments, issue comments, and
   failing check output (`gh pr view`, `gh api .../comments`, `gh pr checks`). Count the sources —
   Step 9 reports them as the coverage claim.
2. Ignore this skill's own prior marker-carrying comments (same prefix 2.3 matches on) as *input*;
   they are output, and treating them as feedback loops the skill onto itself.
3. Classify each finding per `SKILL_ROOT/reference/classify-blockers.md`.
4. **Auto-fix mechanical findings** with minimal `Edit`s — no adjacent refactoring, no opportunistic
   cleanup. Scope discipline is what makes the diff reviewable.
5. **Run the gate unpiped, and report which rung ran.** Piping swallows the exit code and lets a red
   gate read green. The repo's designated gate (per its own `CLAUDE.md`) is rung 1; heavy or
   environment-fragile gates are common enough that the fallbacks below are **sanctioned rather than
   improvised** — but every rung below 1 carries a mandatory disclosure, which Step 9 prints.

   | Rung | Condition | Reported as | Push? |
   |---|---|---|---|
   | 1 | Repo's designated gate ran, unpiped | `gate: full — green` / `red (<summary>)` | green → yes · red → no |
   | 2 | Rung 1 demonstrably unrunnable; a **scoped** check covering the touched files ran | `gate: scoped — green (<exact commands>; uncovered: <what>)` | green → yes · red → no |
   | 3 | Repo **has** a gate; neither rung 1 nor rung 2 could run | `gate: not run (<reason>)` | **no** |
   | 4 | Repo **defines no gate** (docs-only, per its `CLAUDE.md`) | `gate: none (repo defines none) — unverified` | yes |

   A rung-2 scoped check is the narrowest verification that actually exercises what changed — the
   tests bound to the touched modules, plus the repo's linter and any format or parse check. It is not
   "run something cheap and call it green": name the exact commands, and name what a full gate would
   have covered that this did not. **Descend a rung only on a demonstrated failure of the rung above**
   — record the failure (missing interpreter, unresolvable environment, a breached time bound), never
   a guess that it would have been slow.

   **Rung 3 is the honest dead end, and it does not push.** A repo that declares a gate is a repo
   whose author expects verification, and shipping past one that could not run substitutes silence for
   evidence. Nothing is lost by stopping: step 6 still commits, and Step 11.2 preserves the worktree, so
   a human resolves the environment and pushes the existing commit.
6. **Stage only the files you touched** — never `git add -A`. **Commit regardless of the gate
   result** — the commit is how the work survives — then push **only when the rung reached in step 5
   authorizes it** (rungs 1, 2, 4 on a non-red result; never rung 3), with an
   explicit refspec: `git push origin HEAD:refs/heads/<head-ref>`. The refspec is required — the
   worktree's local branch is `pr-round-<N>`, not the PR branch, so a bare `git push` under the
   default `push.default=simple` refuses a name mismatch. Red gate, or rung 3 → commit, do not push,
   return the failure; Step 11.2 preserves the worktree because the commit is unpushed. A rejected push
   (the remote moved) is a per-PR failure per 5.4 — never force.
7. **Post no comment.** Return design-pins unapplied, each with 2–3 defensible options and the
   reason it is not mechanical. Step 9 writes the round's single handoff comment.
8. **Every option offered must be an edit this skill can apply.** Step 8's only capability is editing
   files in the worktree, so an option phrased as an action outside that — file an issue, reply to a
   named review thread, ask a collaborator, change a repo setting, re-run someone's CI — cannot be
   executed and must not be offered as one. Return those as **out-of-band follow-ups** instead: the
   action, who would take it, and why this round could not. They surface in the handoff comment's
   `### Still open` and in the Step 11 summary. A pin may legitimately return both — options Step 8 can
   apply, *plus* a follow-up the operator handles — but an option list that is entirely unexecutable
   is a defect: it reads as actionable and silently is not, and the operator's answer then has nowhere
   to land.

   **Subagents post exactly nothing and file nothing.** A subagent does not reply to individual
   review threads and does not open issues, on your repositories or anyone else's — anything
   requiring either is returned as an out-of-band follow-up. The **main loop** may then file a
   follow-up issue for it at Step 7 (your own PRs only, consent-gated); replying to review
   threads remains out of scope everywhere.

### 5.4 Failure handling

A dead agent, empty return, or rejected push is recorded as a **per-PR failure** and does not abort
the run. Siblings continue. The failure appears in the Step 11 roster with its reason.

### 5.5 If a write is rejected anyway

Step 0.0 closes the write-guard latch, which is what makes the remediate lane able to write at all. If
an `Edit` is nonetheless rejected by a host guard, the agent must not retry it, route around it, or
report the finding as fixed. It **switches that PR to analysis-only** and returns:

- `applyMode: analysis-only`, with the rejection text verbatim, and
- the exact edit set it would have made — per edit: absolute file path, the old string, the new
  string, and the finding it closes. Precise enough for the main loop to apply without re-deriving.

The main loop then applies them itself: per affected PR, `EnterWorktree` into that PR's worktree path,
apply the returned edits, run the gate ladder (5.3 step 5), then commit and push under the same rules.
**Group by repository and take each repository's PRs consecutively** — re-entry into a worktree of a
*different* repository is the step most likely to be refused, and a refusal there is a per-PR failure
per 5.4, not an aborted run.

This is a **fallback, not a mode**: it is slower, it spends main-loop context that the fan-out exists
to protect, and it exists so a guarded host degrades to a completed round rather than a failed one. It
is never the default, and 0.0 is what keeps it unreached.

## Step 6: Decide the returned design-pins

Runs in the main loop, where `AskUserQuestion` is available.

1. Collect every design-pin returned by remediate-lane agents. Empty → ask nothing and continue to
   Step 7, which still files any returned out-of-band follow-ups and otherwise no-ops (Step 8 then
   dispatches no agents). Skipping straight to Step 9 would silently drop those follow-ups' issues.
2. Ask in batches of at most 4 questions per call, grouping across PRs when the queue is sparse.
   **Every question names its PR** — a batched call is otherwise ambiguous.
3. Each question carries the 2–3 defensible options the agent supplied, safest marked
   `(Recommended)`, **plus a `Defer` option**. Deferring records the item and routes it to Step 7,
   which files it as a follow-up issue (consent permitting) — say so in the option's description. An
   operator is never forced to decide a pin they want to sit on; one who wants it held *untracked*
   says so in a custom answer.
4. Record each answer against its PR and finding for Step 8 and Step 9.

**No subagent holds an identity-post decision.** Step 6 is the only place `AskUserQuestion` runs, and
`CONSENT` (Step 3) never leaves the main loop — which is why every posting step — 7, 9, and 10 — is
a main-loop step.

## Step 7: File follow-up issues (own PRs only)

Runs in the main loop, after Step 6 and before Step 8, so the handoff comment (Step 9) can cite the filed issue
numbers. It turns the round's "doesn't fit this PR" items into tracked follow-ups instead of lines
that die in a comment:

- every **out-of-band follow-up** returned per 5.3 step 8, and
- every design-pin **deferred** at Step 6 (the `Defer` option notes this — an operator who wants an item
  held privately answers with a custom response instead).

**Own PRs only — hard boundary.** Items from review-lane PRs are never filed (Step 3: filing on
someone else's repository pre-empts the maintainer's triage); they stay in the review comment. Filing
targets the repository the PR belongs to.

**Consent-gated by Step 3's single ask.** `CONSENT == false` → file nothing; hand every would-be
issue (title + body) to Step 11's withheld list so the operator can file manually. Withholding is not
dropping.

Per item, when `CONSENT == true`:

1. **Title:** the first sentence of the item's text, truncated to ≤72 chars with `…`.
2. **Body:** a `## Finding` section carrying the item verbatim (for a deferred pin: the pin, its
   options, and that it was deferred), then a `## Backlinks` section with the PR URL and the round's
   head sha.
3. **Label:** `P2-backlog` when the repo has it (`gh label list` once per repo); absent → no label,
   never invent one.
4. **Create:** `gh issue create --repo <owner>/<repo> --title "<title>" --body-file <tmp>
   [--label P2-backlog]`; parse the issue number from the printed URL.
5. **Record** `{PR, item, issue number, URL}` for Step 9 (`### Still open` cites it) and the Step 11
   summary. A `gh issue create` failure is recorded against the item and reported in Step 11 — the
   round continues; the item is then listed for manual filing, never dropped.

## Step 8: Fan out — apply the decisions

Dispatch one subagent per PR with **≥1 non-decided-as-defer** decision, in **waves of
`--concurrency`** exactly as Step 5. A PR whose pins were all deferred, or that had none, gets
**no agent**.

The brief is a **narrowed** 5.3: same worktree, same gate discipline, same commit and push rules —
including the explicit `HEAD:refs/heads/<head-ref>` refspec. State only the differences:

- **Reuse the existing worktree** at the path Step 5.1 recorded for this PR — the default
  `<repo-root>/.claude/worktrees/pr-round-<N>`, or `<PR_ROUND_WORKTREE_ROOT>/<repo-name>/pr-round-<N>`
  when that override is set. It already exists, already on its `pr-round-<N>` branch tracking `origin/<head-ref>`,
  carrying Step 5's commit. Do not create a new worktree and do not create a new branch.
- Apply exactly the chosen options — no scope beyond them.
- **Verify the decided option against the code before applying it.** The operator decided the
  *semantics*; they did not verify the implementation, and they answered from the agent's summary
  rather than from the call sites. An option can be right in intent and wrong as written — the site it
  names may raise a different exception type, the helper may have a second caller, the field may
  already be set upstream. Check the decision at **every** site it touches first. Correct → apply it.
  Verbatim application would be wrong → **apply the smallest variant that delivers the decided
  intent**, and return the divergence explicitly: what was decided, what was applied, and the evidence
  that the literal form was broken, so Step 9 reports it under `### Decisions made`. Never apply an option
  you can demonstrate is broken, and never quietly substitute a *different* intent — the first ships a
  known bug under the operator's approval, the second overrides a decision that was not yours to make.
  Genuinely unsure which it is → apply nothing, and return it as still open with the conflict stated.
- Run the gate **unpiped**, on the same rung ladder as 5.3 step 5, and return the rung reached along
  with the result. **Red gate, or rung 3 → do not push.** Return the failure and leave the worktree
  intact with the work in it; Step 11 preserves it and names its path. A decision that breaks the gate
  is precisely when a human should look.
- **Post no comment.** Return what was applied, the resulting sha, and the gate result.

A Step 8 failure is per-PR and does not abort siblings, matching 5.4.

## Step 9: Post the handoff comment

Runs in the main loop, once per **remediated** PR — including PRs with no design-pins and PRs where
Step 8 failed. A round that touched a PR never leaves it silent.

Review-lane PRs are posted by Step 10 and are skipped here.

1. Read `SKILL_ROOT/reference/remediation-comment-template.md` and follow it.
2. Compose from the accumulated state: Step 5's ingested-source counts and fixes, Step 6's decisions and
   reasoning, Step 8's applied changes and final sha, the last gate result.
   - **Disclose the gate rung.** Report the rung reached (5.3 step 5) in the `Gate` header field, in
     its full reported form. A rung-2 scoped gate names its exact commands and what went uncovered; a
     rung-4 no-gate repo says `unverified`. Never report a scoped or absent gate as plain `green` —
     the disclosure is the whole reason the fallback rungs are sanctioned.
   - **Report any Step 8 divergence.** Where an agent applied a corrected variant rather than the decided
     option verbatim (Step 8), `### Decisions made` states both and why.
3. Set the readiness verdict per the template's trigger table: `READY FOR RE-REVIEW` /
   `PARTIALLY ADDRESSED` / `BLOCKED`.
4. `head` in the marker is the **final** sha — after Step 8 when it ran, else after Step 5. Nothing
   pushed → `unchanged — no commit`.
5. No section may describe a pin as awaiting a decision; Step 6 resolved them. Deferred items, and every
   **out-of-band follow-up** returned per 5.3 step 8, go under `### Still open` — each citing the
   follow-up issue Step 7 filed for it (`#<N>`), or, where none was filed (consent declined, filing
   failed, or the item belongs to someone else's repo), naming the action and who takes it.
6. Post `gh pr comment <N> --repo <R> --body-file <tmp>` **only when `CONSENT == true`** (Step 3);
   otherwise skip and hand the body to Step 11's withheld list. Being your own PR is not an
   authorization — this skill claims no standing permission (Step 3).

## Step 10: Post the review comments and verdicts

Runs in the main loop, once per **review-lane** PR that produced a review. Step 5.2 returned the
comment body and the verdict token without posting either; this is where they land, because
`CONSENT` lives here and nowhere else.

**`CONSENT == false` → post nothing at all**, and hand every comment body and verdict to Step 11's
withheld list so the operator can post them by hand. Withholding is not dropping.

Otherwise, per PR, in this order:

1. `gh pr comment <N> --repo <R> --body-file <tmp>` — the comment body 5.2 returned, verbatim.
2. `gh pr review <N> --repo <R> <flag> --body "<one-line rationale>"`, mapping the returned token:
   `REQUEST CHANGES` → `--request-changes`, `APPROVE` → `--approve`, `COMMENT` → `--comment`.
   **`--body` is required on all three** — `gh pr review --comment` fails non-interactively without
   one.

A verdict rejected by the API (insufficient permission, or a draft PR named by an explicit ref) is a
per-PR failure per 5.4, recorded as `commented, verdict failed` with the API's reason. The comment is
already posted at that point and is not retracted — the findings are still useful to the author.

## Step 11: Roster summary and cleanup

### 11.1 Summary

One table. **Every PR resolved in Step 1 appears in exactly one row** — processed, skipped, or
failed. Rows reflect the final state after Steps 8-10, not Step 5's intermediate state.

| PR | Lane | Outcome | Detail |
|---|---|---|---|
| `<owner>/<repo>#<N>` | review / remediate | verdict, readiness, `skipped`, or `failed` | comment URL, fix count, or the reason |

Also report, each only when non-empty:

- **Withheld by consent decline** — every unposted comment and verdict, so a declined run still ends
  with an actionable manual list.
- **Deferred decisions** — every pin deferred at Step 6, each with the Step 7 issue that tracks it (or the
  reason none does), so a deferral is visible rather than lost.
- **Issues filed** — every follow-up issue Step 7 created: `#<N> — <title> (<PR it came from>)`.
- **Out-of-band follow-ups not filed** — every action returned per 5.3 step 8 that ends the round
  untracked (consent declined, filing failed, review-lane repo, or an action no issue can carry —
  review threads to reply to, people to ask), with the PR it came from. The round ends with them
  collected in one place for manual action.
- **Gate rungs below 1** — each PR whose gate ran scoped, could not run, or does not exist, with the
  rung and its disclosure string. A run where several PRs pushed on scoped gates should read that way
  at a glance rather than only inside each comment.
- **Analysis-only fallbacks** — any PR whose agent hit a write rejection and was applied by the main
  loop instead (5.5), so a guarded host is visible as a host problem rather than a per-PR oddity.
- **Worktree root** — the path actually used, whenever `PR_ROUND_WORKTREE_ROOT` overrode the default.
- **Truncated by `--limit`** — how many PRs were dropped. A silent cap reads as full coverage.
- **Missing rubric tiers** — per repository, with the cold-start prompt from
  `SKILL_ROOT/reference/rubric-layering.md`.
- **Preserved worktrees** — path, branch name, and why.

### 11.2 Cleanup

**Runs only after Steps 8-10 have completed.** Removing a worktree before Step 8 strands that PR's
decisions, and the failure is silent — the roster still prints, the decisions simply never landed.
This is the one ordering constraint in the skill that gives no error when violated.

For each worktree created in Step 5, decide from the **worktree's actual state**, never from which
step it came out of — a provenance test ("red gate from Step 8") misses the identical condition arriving
from 5.3, and the miss is a silent deletion of committed work:

```
git -C <path> status --porcelain                # uncommitted work?
git -C <path> rev-list --count @{upstream}..HEAD  # unpushed commits?
```

`@{upstream}`, not a reconstructed `origin/<head-ref>`: a fork worktree was created from
`origin/pr/<N>` (5.1) and has no `origin/<head-ref>` to name, so spelling the ref out here would exit
fatal on every review-lane fork PR — and this step has no error path, so the worktree would leak and
poison the next run. Asking the branch what it tracks is correct for both shapes.

- Either non-empty / greater than zero → **preserve it**, and name its path *and its `pr-round-<N>`
  branch* in the summary, with the reason. Discarding a human's next move to keep the tree tidy is
  the wrong trade, and the branch is how they reach the work. Note in the row that the next run on
  this PR will fail with `preserved worktree from a prior run` (Step 5.1) until it is integrated or
  deleted — a preserved worktree is a debt, not just a recovery aid.
- Both clean → `git -C <repo-root> worktree remove <path>` **and**
  `git -C <repo-root> branch -d pr-round-<N>`. The per-PR branch is scaffolding; leaving it behind
  accumulates one dead branch per PR per run. Everything it carried is already on the remote — that
  push is what makes deleting it safe, which is exactly why the check above is what decides. Use
  `-d`, never `-D`: git's own unmerged check is the second line of defense if the state test is ever
  wrong.

Then stop. There is no next round.

