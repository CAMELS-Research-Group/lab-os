# Verification-Command Contract

**Status:** draft
**Audience:** Facilitators and technical implementers running the Building part of the workshop
program. Participant-facing application of this contract lives in the
[execution and verification worksheet](../../../site/docs/workshops/building/execution-verification-worksheet.md)
and the [quality-gate and review worksheet](../../../site/docs/workshops/building/quality-gates-worksheet.md).

---

## What this contract specifies

The Building part's worksheets and exercises depend on every participant having a single
verification command for their project. This document defines the **behavioral contract** that
command must satisfy — not its name, not its implementation, but the properties it must exhibit
for the exercises to function correctly.

Command names belong to the implementation (each project's tech stack chooses its own). The
behavior described here is what the worksheets assume to be true of whatever command the
participant names.

---

## The contract

### 1. A single command exists

The project has **one command** that expresses its current passing state. A participant must be
able to name it in one line.

- Acceptable: `npm test`, `pytest`, `cargo test`, `make check`, `./scripts/verify.sh`
- Not acceptable: a sequence of separate commands that must run in a specific order, or "it
  depends on what you're testing"

If a participant's project does not yet have a single verification command, defining one is the
first task before any autonomous run. The command can start minimal — even a single passing test
file is better than no command.

### 2. Runs from the repo root in one invocation

The command runs from the repository root without changing directories first. A facilitator or
participant running the command in a worktree should be able to copy it verbatim and run it.

- Acceptable: `npm test` run from the project root
- Not acceptable: `cd src && python -m pytest tests/` (requires a directory change)

If the project's test runner requires a subdirectory, the verification command is a wrapper (a
script or Makefile target) that handles the `cd` internally.

### 3. Exit code is the source of truth

The command exits with **code 0** when the project's gate passes. It exits with a **non-zero
code** when anything fails. The exit code, not the output, is the pass/fail determination.

This is the property that makes verification deterministic inside an autonomous run. If a
participant needs to read output to decide whether the run passed, the command does not satisfy
this contract.

### 4. Output is human-readable

The command's output is readable text that a person can interpret. The verification step is not
only automated — a facilitator or participant inspects the output during exercise debrief.

- Acceptable: standard test-runner output naming tests and indicating pass/fail
- Not acceptable: raw binary output, a silent command that only emits an exit code

The requirement is minimal: the participant can look at the terminal output and understand what
happened.

### 5. No interactive prompts

The command runs to completion without prompting for input. It does not ask for a confirmation,
a password, or a choice mid-run.

This property is required for the command to be safe inside an autonomous session. A command that
pauses for input will stall a run silently.

If the project's test runner prompts under some conditions (e.g. coverage-report writing), the
verification command is scoped to the non-prompting invocation (e.g. with the prompt-triggering
flag disabled).

---

## The run/start command (if the project is runnable)

For projects that produce a running service or application — not just a library — there is a
parallel contract for the **run/start command**:

- **Single invocation** from the repo root
- **Exit code signals failure** (the process exits non-zero if it cannot start)
- **No interactive prompts** at startup
- **Human-readable startup output** (the participant can tell whether the service started)

The run/start command is used for manual smoke tests on credential and data paths — the paths the
verification command cannot reach. It is not used as the primary gate; the verification command is.

---

## Where the contract surfaces in the exercises

| Exercise / worksheet | How the contract is used |
|---|---|
| [Quality-gate worksheet](../../../site/docs/workshops/building/quality-gates-worksheet.md) | Participant writes and verifies their gate command against the contract |
| [Execution and verification worksheet](../../../site/docs/workshops/building/execution-verification-worksheet.md) | Participant writes the gate command before the first autonomous run; uses exit code as source of truth |
| Exercise 4 | Participant runs the command after the autonomous run; exit code determines pass/fail |
| Exercise 5 | Participant runs the command per task in the multi-task chunk |

---

## Notes for facilitators

**If a participant cannot name a single verification command at the start of the session,** stop
the autonomous run setup and treat command definition as the first task. A project without a
verification command is not ready for autonomous execution — not because of a workshop rule, but
because there is no way to detect a failed run. The participant can define a minimal command
(even a single test file) during the session and run Exercise 4 against that.

**If a participant's project is documentation-only or a static site,** the verification command
is typically the site build (`npm run build` or equivalent). The exit code is the gate: if the
build fails, the run failed.

**If a participant's project has no tests at all,** do not fabricate a gate. Have the participant
identify the simplest behavior that is currently working and define a test for it — even one test
is a gate. An empty test suite that always passes is not a meaningful gate and violates the
contract.

**The contract does not specify a tool.** Whatever the participant's stack provides is acceptable
as long as it satisfies the five behavioral properties above.
