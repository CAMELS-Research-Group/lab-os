# Building Session — Facilitator Runbook

**Status:** draft
**Audience:** Facilitators running the Building part of the workshop program. Read alongside the
participant pages under `site/docs/workshops/building/`. Participant materials contain the full
content; this runbook tells you how to hold the room and what to do when runs go wrong.

---

## Before the session

Confirm participants have:

1. **Completed the Planning part** — they have an execution-ready plan (PRD + ordered,
   individually-verifiable task list) with a verification note on each task. Anyone without a
   plan cannot run Exercise 4.
2. **Completed the pre-flight worksheet** — Claude Code installed and authenticated, Git working,
   project cloned on a branch they can experiment on, git worktrees confirmed working. The
   pre-flight is the session-killer if skipped: setup problems during the session consume everyone
   else's time.
3. **A verification command in mind** — ideally named in the pre-flight. If they do not have one,
   they need to define it as the first task in the session (Exercise 1 / quality-gate worksheet).

Have the following open for reference:

- [Exercises page](../../../site/docs/workshops/building/exercises.md)
- [Quality-gate worksheet](../../../site/docs/workshops/building/quality-gates-worksheet.md)
- [Execution and verification worksheet](../../../site/docs/workshops/building/execution-verification-worksheet.md)
- [Verification-command contract](./verification-command-contract.md) — the behavioral bar for
  deciding whether a participant's command is acceptable (referenced in Segment 1 and the
  failure-mode catalogue)
- This document (specifically the failure-mode catalogue below)

---

## Run-of-show

### Segment 1 — Framing and quality-gate setup (25 min)

**What:** Orient the cohort to the three execution modes and have each participant define their
verification command and quality gate before any autonomous execution begins.

Teach (10 min):
- Introduce the three modes in order: monitored sequential → autonomous → scaling. Explain
  that the exercises move through them in this sequence and that earlier modes teach what
  good output looks like before participants hand off a full task
- State the central discipline: **let go, then verify** — not micromanage, not trust blindly
- Introduce the verification command concept: one command, from the repo root, exit code is
  the source of truth (the full behavioral bar is the
  [verification-command contract](./verification-command-contract.md)). Participants write this
  command down before touching a terminal
- Briefly explain green-is-not-reviewed: a passing gate proves the gate passed, not that the
  work is correct

Quality-gate worksheet (15 min):
- Participants work through the [quality-gate worksheet](../../../site/docs/workshops/building/quality-gates-worksheet.md)
  individually — writing their gate command, noting what it checks and does not check,
  marking the defect classes that apply, and drafting their review standard
- Walk the room. The most common gaps: a participant whose "gate" is two separate commands
  (they need a wrapper), and a participant with no tests at all (see failure-mode catalogue
  below). The [verification-command contract](./verification-command-contract.md) is the bar
  for adjudicating these on the spot — single command, repo-root, exit-code-as-truth
- Close with a quick check: ask each participant to say their verification command aloud.
  This surfaces anyone who is still uncertain before the exercises begin

---

### Segment 2 — Exercises 1 and 2 (30 min)

**What:** Plan to roadmap and sequencing/dependencies — the monitored-sequential exercises that
build the structural foundation the autonomous runs depend on.

These exercises are conversational and can run in the Claude desktop app or terminal. They are
about plan structure, not execution speed. Participants who rush through them and produce a
poorly-sequenced task list will have a harder time in Exercise 4.

Teach briefly (5 min):
- Exercise 1: the goal is a task list where every item is self-contained. If Claude cannot
  understand a task from its text alone, it is not self-contained
- Exercise 2: group tasks into waves. Wave 1 runs first (no dependencies); Wave 2 depends on
  Wave 1; and so on. The wave structure is what they hand off in Exercise 5

Participants work (25 min):
- Circulate and check roadmaps for tasks that are too large ("implement the whole API") or
  too vague ("add error handling"). Ask: "What would you check to know this is done?"
- Look for dependency maps where everything depends on everything — this is a sign the tasks
  are too coarse. Direct those participants to the granularity guidance in the decomposition
  worksheet from Part 1

---

### Segment 3 — Exercise 3: Worktree isolation (10 min)

**What:** Create the throwaway workspace that makes Exercise 4 safe.

This segment is short and technical. Its only purpose is confirming the worktree is created and
the participant knows where it is. Do not let this segment expand — it is setup, not teaching.

Teach (3 min):
- Explain what a worktree is and why it matters: autonomous runs execute in this directory,
  not in the participant's main working tree, so a failed run cannot damage their main branch
- Show the two commands: `git worktree add` and `git worktree list`

Participants work (7 min):
- Participants create their worktrees. Walk the room and confirm: `git worktree list` shows
  two entries for each participant
- Common issue: the worktree directory already exists from a prior attempt. `rm -rf` the
  directory and re-run the add command
- Anyone whose worktree creation fails is not ready for Exercise 4. Diagnose and fix it
  before the segment ends — this is the last checkpoint before the highest-risk segment

---

### Segment 4 — Exercise 4: First autonomous execution (45 min)

**This is the highest-risk segment.** The meta-skill being built is **let go, then verify**.
Participants often want to micromanage the run or accept the agent's self-report as proof.
Neither produces the skill this exercise is teaching.

See the **Failure-mode catalogue** below — this segment is where every failure mode appears.

Teach (10 min):
- Walk through the hand-off framing: context (over-include), task (paste from plan with
  verification note), stop condition (done when the verification step passes, not when Claude
  thinks it is done)
- Emphasize: do not intervene mid-run unless the run has completely stalled. Watching is fine.
  Correcting mid-run defeats the exercise
- Show what inspecting the diff means: run `git diff HEAD` in the worktree directory and read
  the actual changes, not the summary Claude produces
- State the lesson plainly: "A run where the verification command fails and you catch it is a
  success. You have just proved verify-don't-trust. Record it."

Participants work (25 min):
- Participants set up their hand-offs using the execution and verification worksheet, then
  run Exercise 4
- Your job during this window: **keep your hands off keyboards**. Resist the impulse to fix
  things for participants — let the runs fail if they are going to fail. A failure caught now
  is worth more than a run you rescued
- Watch for participants who intervene mid-run to course-correct. Gently name it: "What are
  you trying to fix?" Often they are reacting to something the agent said, not something the
  run did. Ask them to let the run complete and then verify
- Watch for participants who accept the agent's summary as proof without running the
  verification command. This is the central mistake — catch it before they move on

Check-in / debrief (10 min):
- Ask one or two participants to share what happened in their run — specifically: what did the
  diff show, what did the verification command return, and did those two match?
- Prioritize participants whose runs failed. Walk the failure end to end with the room: what
  happened, which failure mode it matches, and how the verification step caught it
- Name the lesson explicitly: catching a failure is the skill. The discipline worked.

---

### Segment 5 — Exercise 5: Scaling execution (20 min)

**What:** Hand off a multi-task chunk and verify each result separately.

This segment extends the verify-don't-trust discipline from one task to a wave. The framing is
the same as Exercise 4; the new element is verifying each task independently rather than
treating the chunk as a single pass/fail.

Teach (5 min):
- Frame the multi-task hand-off: context, list of tasks (each with its verification note),
  stop condition (complete each task, verify it, report the result before moving to the next)
- Explain per-task verification: do not run the gate once for both tasks and call it done.
  Verify task 1, record the result, then verify task 2

Participants work (15 min):
- Participants run the multi-task hand-off and fill the Part B section of the execution and
  verification worksheet
- Remind participants to use a fresh worktree or a clean state from the Exercise 4 result
  (the prior run's output should be committed or discarded before starting this run)

---

### Segment 6 — Completion checklist and debrief (20 min)

**What:** Self-attest completion and close the session with the debrief structure below.

Checklist (10 min):
- Participants work through the [completion checklist](../../../site/docs/workshops/building/completion-checklist.md)
  independently. Each item is a yes/no check against the artifact — not how the run felt
- Anyone who cannot check an item identifies the remaining work to complete before Closeout

Debrief (10 min):
- Surface **one real failure from the session** — pick the most instructive one, with the
  participant's permission — and walk it end to end with the room:
  1. What was handed off
  2. What the verification step returned
  3. Which failure mode from the catalogue it matches
  4. What the participant would do differently next time
- Close with the homework: scale capacity — how fast can you plan and how fast can you
  execute. The capacity-tracking sheet continues the thread from the Planning homework
- Confirm the hand-off: bring the completion checklist, the carry-forward notes from the
  execution worksheet, and a note about any open tasks the autonomous runs produced.
  Closeout starts with this material

---

## Failure-mode catalogue

These are the failure modes that appear in Exercise 4 (the autonomous execution segment). Each
has a named recovery move for use on-stage.

### Scope creep — agent does too much

**What it looks like:** The participant hands off one task. The agent completes it, then
continues modifying adjacent files, adding features that were not in the task description, or
refactoring things it decided were "while I'm here" improvements. The verification command may
pass because the gate only checks what it was written to check.

**Recovery move:** In debrief, show the diff. Have the participant point to lines that were not
in the task description. This is the central value of reading the diff: scope creep is invisible
until you look at what actually changed. Name the fix: tighten the stop condition. The hand-off
should say "do only this task; stop when the verification step passes and do not touch other
files."

---

### Claims done but isn't — the headline lesson

**What it looks like:** The agent reports the task is complete. The participant runs the
verification command. It returns non-zero. The agent was wrong.

**Recovery move:** This is not a failure to recover from — **celebrate it**. The participant just
proved verify-don't-trust works. Name it explicitly: "You caught this. The verification step
caught a failure the agent missed. This is the skill." Then have the participant paste the
verification output back to the agent as evidence and continue the run from where it failed.

If the participant is embarrassed or frustrated, reframe: the worst outcome in this exercise is
a run that fails *and is not caught*. A caught failure is the lesson working exactly as designed.

---

### Participant won't let go — micromanaging

**What it looks like:** The participant starts the autonomous run, then immediately begins
feeding the agent corrections, clarifying questions, or mid-run direction. The run is supervised,
not autonomous.

**Recovery move:** Gently name what is happening: "You're coaching the run. That's monitored
mode, not autonomous mode. The exercise is to let it run and verify after — not to fix it as it
goes. Can we try again with hands off the keyboard?" If the participant is anxious, acknowledge
it: the anxiety is the point — the exercise is building the confidence to let go. The verification
step is what makes it safe to let go.

If the participant still cannot resist intervening, have them close the terminal during the run
and only reopen it when the agent stops producing output. This is a forcing function.

---

### Environment or tooling stall

**What it looks like:** The run stalls for a reason unrelated to the task — a missing
dependency, a wrong path in the verification command, a worktree that is not in the right state.
The participant cannot tell whether the stall is a task failure or an environment issue.

**Recovery move:** Have the participant run the verification command directly (not via the agent)
in the worktree directory. If the command fails with an environment error rather than a test
failure, it is an environment issue. Fix the environment, confirm the command runs cleanly, then
re-run the exercise from the hand-off framing step.

Common environment stalls: the verification command works from the main working tree but not the
worktree (a missing environment variable or a path that assumes a specific directory); the agent
attempted to install a dependency and failed silently; the worktree is not on the expected branch.

---

### Task too big

**What it looks like:** The participant handed off a task that, once running, turns out to be
multiple tasks. The agent runs for a long time, makes changes across many files, and produces
something that either does not pass the verification command or passes it on some behaviors and
fails on others. The run is hard to debrief because it is unclear what was supposed to change.

**Recovery move:** Stop the run if it has not finished. Remove the worktree (`git worktree
remove`) to reset to a clean state. Return to Exercise 1 (roadmap) and split the task into
smaller pieces — each small enough that you can name its verification step precisely. Then run
Exercise 4 again on the first, smaller piece.

Name the principle: "Done when" is the test for task size. If you cannot state a single,
checkable done condition, the task is too big.

---

## Facilitation do / don't

**Do:**
- State the meta-skill plainly at the start of Exercise 4: let go, then verify. Return to this
  framing every time a participant gets stuck deciding whether to intervene
- Keep your hands off keyboards during autonomous runs. Let safe runs fail; do not rescue every
  task
- Celebrate caught failures loudly. If a participant's verification command returns non-zero,
  that is the best thing that can happen in this exercise
- Use the debrief structure: surface one real failure, walk it end to end, name the failure mode,
  ask what the participant would change. The debrief is the highest-return moment in the session
- Check that every participant reads the actual diff — not the agent's summary of the diff. If
  they have not opened the terminal and looked, they have not completed Exercise 4

**Don't:**
- Rescue a participant's run by helping them fix it mid-execution. If the run is going to fail,
  let it fail and use the failure in debrief
- Accept "the agent said it was done" as evidence of completion without seeing the verification
  command output. Name this explicitly if you see it
- Let the quality-gate setup be skipped or treated as a formality. A participant who cannot name
  their verification command is not ready for Exercise 4 — stop and define it first (the
  [verification-command contract](./verification-command-contract.md) defines what counts as
  acceptable)
- Compress the debrief to make up time. The debrief is where the lesson lands; the exercises
  are only setup for it
- Introduce multi-agent dynamic workflow tooling or external orchestration concepts beyond what
  Claude Code supports natively. The Building part binds to documented Claude Code capabilities;
  anything beyond that is out of scope for this session

---

## Common questions from participants

**"Can I intervene if the agent is heading in the wrong direction?"** Not in Exercise 4. The
exercise is specifically about letting the agent run and then catching what went wrong with the
verification step. An intervention avoids the lesson. If the run looks wrong, let it finish, run
the verification command, and use the failure in debrief.

**"My verification command fails for a reason that has nothing to do with the task."** Diagnose
whether it is an environment issue or a task failure. An environment issue (wrong path, missing
dependency) is fixed first; then the run is re-started. A task failure (the gate finds real
problems) is the lesson.

**"Exercise 5 wants me to use a fresh worktree, but I don't want to lose my Exercise 4 result."**
Commit the Exercise 4 result to the worktree branch first (`git add -A && git commit -m "exercise
4 result"`). Then either continue in the same worktree (the tree is now clean) or create a new
worktree from the committed state.

**"The agent dispatched multiple parallel runs without me asking it to."** This is Claude Code's
native multi-agent coordination in action — it dispatched subagents to handle the tasks in
parallel. The verification discipline is the same: verify each result separately. If the parallel
dispatch produced unexpected interference between tasks, that is a finding: those tasks had a
hidden dependency.

**"I don't trust my verification command — it always passes even when I think something is
wrong."** The gate is not meaningful. Stop and revisit the quality-gate worksheet with the
participant. A gate that always passes is not a gate — it is noise. The fix is to identify a
specific behavior that the project must exhibit and write a test that fails when that behavior
breaks.
