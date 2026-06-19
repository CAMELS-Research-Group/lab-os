---
title: Quality-Gate and Review-Standards Worksheet
description: Define a hard, meaningful quality gate for your project, confront what a passing gate cannot catch, and write your personal review standard for accepting autonomous output.
---

# Quality-Gate and Review-Standards Worksheet

Complete this worksheet **before** Exercise 4 (first autonomous execution). You will need it in
hand when you run the exercise — and every autonomous run after it.

The source of truth for the verification and review discipline behind this worksheet is the
[Working with Claude page](/docs/working-with-claude). That page owns the method; this worksheet
applies it to your project.

---

## Part 1 — Define your quality gate

A quality gate is a single command that proves your project's current passing state. "Hard" means
it has an exit code — it passes (exit 0) or fails (non-zero), with no interpretation required.
"Meaningful" means it checks something that would catch a real defect, not just that the project
runs at all.

**Write your gate command here:**

```
<your command — runs from the repo root, one invocation, no interactive prompts>
```

Before your first autonomous run, verify the command works:

- [ ] I have run the command and it returns exit 0 on the current state
- [ ] I can run it from the repo root in one invocation
- [ ] It produces human-readable output (not just a raw exit code)
- [ ] It does not prompt for input mid-run (safe to use inside an autonomous session)

**What does your gate actually check?**

Write two or three sentences describing what the gate verifies — specifically, what would have to
be true for the gate to return exit 0. Be concrete: "tests pass" is not specific enough.

> _Write here:_

**What does your gate not check?**

Write two or three sentences describing what could be broken that the gate would still pass on.
See Part 2 for a catalogue of common blind spots.

> _Write here:_

---

## Part 2 — Green is not reviewed

A passing gate does not mean the output is correct, safe, or ready to trust. It means the gate
passed. Before you hand off autonomous work, you need to be honest about what a green gate actually
tells you — and what it does not.

### The self-referential coverage problem

When the same agent writes both the code and the tests, the test coverage is self-referential: it
checks that the agent's assumptions are internally consistent. A passing gate in that case does not
mean the implementation is correct — it means the implementation matches what the agent expected
it to do. An agent that misunderstands a requirement will write code that satisfies its own
misunderstanding, then write tests that verify that misunderstanding, and the gate will pass.

**Checkpoint:** Is your gate written by the same agent that wrote the code it tests?

- [ ] Yes — I am accounting for self-referential coverage when I inspect the output
- [ ] No — the gate was written independently of this run

### Defect classes a gate cannot catch

A gate catches what it was written to test. It does not catch what was not tested, what was not
imagined, or what requires a live environment to observe.

At least one of the following defect classes almost certainly applies to your project. Mark the
ones that do:

- [ ] **Failure-recovery bugs.** What happens when a dependency is unavailable, a network call times
  out, or an external service returns an unexpected response? Unit tests rarely simulate these
  conditions in ways that catch real failures.

- [ ] **Credential and data paths.** Tests that run against mocked or simulated services never touch
  live sign-in flows, real API keys, or actual data connectors. A gate can be green while the
  real authentication path is broken.

- [ ] **Integration and wiring gaps.** Two components can each pass their unit tests while failing
  to work together — because neither test exercises the actual connection between them.

- [ ] **Self-referential coverage.** As described above: the agent wrote the tests it expected to
  need, not necessarily the tests that would catch what it got wrong.

**For each class you marked, write one sentence describing the specific risk in your project:**

> _Write here:_

---

## Part 3 — Credential and data paths need a manual smoke test

Mocked tests — tests that substitute a fake service for a real one — never exercise the actual
sign-in flow, real API keys, or live data connectors. This means:

- A gate that is green on every run can still break the moment a real user tries to log in
- A change to an authentication path or external API call cannot be verified by the gate alone
- These paths need a **manual smoke test**: a short, real run-through of the actual flow with real
  credentials and a live connection

**Does your project have credential or data paths?**

- [ ] Yes — I will run a manual smoke test on these paths after every autonomous run that touches them
- [ ] No credential or live-API paths in this project (confirm: the gate does not use mocked services
  to substitute for a real authentication or data connection)

**If yes, write the smoke test steps here** (brief — the point is to run the real path, not to
write a test suite):

> _Write here:_

---

## Part 4 — Write your review standard

A **review standard** is the set of things you inspect before accepting an autonomous run's output
— beyond running the gate.

Your review standard should answer: "What would I need to see to trust this output enough to
merge it, ship it, or hand it to the next task?"

**Draft your review standard here:**

Before I accept an autonomous run's output, I inspect:

1. _The diff_ — I will read the actual changes, not Claude's summary of them. I am looking for:

   > _Write here (example: unexpected files touched, changes outside the task's stated scope):_

2. _The verification output_ — beyond the exit code, I will check:

   > _Write here (example: specific test names, output lines that confirm the expected behavior):_

3. _Defect classes specific to this task_ — based on the task type, I will check whether:

   > _Write here (example: for any task touching a data path — did I run the smoke test?):_

4. _Scope_ — I confirm the run did not touch anything outside the task's stated scope:

   > _Write here:_

---

## Part 5 — Before your first autonomous run

Before starting Exercise 4, confirm:

- [ ] I have written my gate command and verified it runs cleanly
- [ ] I can describe what my gate does and does not check
- [ ] I have marked the defect classes that apply to my project
- [ ] I know whether my project has credential or data paths, and I know what my smoke test is
- [ ] I have written a draft review standard

When these are all checked, you are ready to run Exercise 4. Keep this worksheet open during the
exercise — you will return to it after the run to record whether the gate passed and what you
inspected.

---

## After the run — record what you found

After Exercise 4 (and each subsequent autonomous run), use this section as a short post-run log.

| Run | Gate result | What the diff showed | Smoke test run? | Review finding |
|-----|-------------|----------------------|-----------------|---------------|
| Exercise 4 | | | | |
| Exercise 5 | | | | |

A run where the gate fails and you catch it is a success at verify-don't-trust. Record it as such.

---

When you have completed this worksheet and run Exercise 4, carry your **gate command** and
**review standard** forward — you will use them in Exercise 5 and in every autonomous run you do
from here. The [completion checklist](./completion-checklist.md) will ask you to state them.
