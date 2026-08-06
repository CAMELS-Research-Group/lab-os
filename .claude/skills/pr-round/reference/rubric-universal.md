<!--
  Structural-class content adapted from Cursor's "thermo-nuclear-code-quality-review" skill
  (github.com/cursor/plugins, MIT licensed, © 2026 Cursor), reached via the finding-class
  catalog it seeded. Restated here in project-neutral form.
-->

# Universal review rubric — tier 1

**The floor, not the ceiling.** This file is what a reviewer applies to *any* pull request, by any
author, in any repository, with no other context available. It ships with the skill and is always
present, so a review is never left with nothing to stand on.

Higher tiers add to it and may override its defaults — see `rubric-layering.md`. Nothing in this
file names a specific organization, repository, language, or house convention; when a check here
feels like it wants a house-specific threshold, that threshold belongs in tier 2 or tier 3.

## Scope rule (read first)

**Only what this PR introduces is in scope.** A finding is in scope when *this diff* creates it,
worsens it, or moves code into the offending shape. A problem the PR merely sits next to is
pre-existing — do not raise it as a Blocker or Important. If it is genuinely worth saying, say it
once as a Suggestion and label it pre-existing.

The test for every finding: *did this change introduce or worsen it?* If the answer is "it was
already like that," it does not gate this PR.

## Severity bars

| Severity | Bar |
|---|---|
| **Blocker** | The PR is not safe or not correct to merge as-is. Data loss, a security hole, a broken contract, a crash path, a structural regression, or a claim in the PR that the diff does not support. |
| **Important** | Should be fixed before merge but does not by itself make merging unsafe. A real gap a maintainer would want closed — missing edge-case handling, an untested branch, a doc that now contradicts the code. |
| **Suggestion** | Quality-of-life. A cleaner shape, a clearer name, a test that would harden an assertion. Never a gate. |

When a finding sits between two bars, state which way you leaned and why. An unexplained severity is
a weaker finding than a well-argued lower one.

## Review dimensions

Work these in order. Each is independent — a clean pass on one says nothing about the others.

### 1. Correctness

Does the code do what the PR says it does? Check the claim against the diff, not against intent.
Look for: off-by-one and boundary handling, null/empty/zero cases, unchecked assumptions about input
shape, concurrency and ordering assumptions, resource lifecycle (opened and not closed, acquired and
not released), and arithmetic that can overflow, truncate, or divide by zero.

### 2. Security

Untrusted input reaching a sensitive sink is the general shape: injection into queries, commands,
templates, or paths; deserialization of attacker-controlled data; path traversal. Also: secrets or
credentials committed or logged, authentication and authorization checks that can be bypassed or are
missing on a new surface, and permission checks performed on the client side only.

### 3. Error handling and silent failure

An error that is caught and discarded is worse than one that propagates — it converts a loud failure
into a wrong answer. Flag: empty catch blocks, catches that log and continue as if nothing happened,
fallback values that mask a real failure, overly broad exception capture that swallows unrelated
errors, and error paths that leave state partially mutated.

A fallback is legitimate only when the degraded behavior is correct *and* the degradation is
observable. If neither holds, it is a silent failure.

### 4. Tests

Does new behavior have a test that would fail without the change? Look for: new branches with no
coverage, tests asserting the implementation rather than the behavior, tests that cannot fail
(tautological assertions, unawaited async, mocked-away subject), and edge cases named in the code
but absent from the tests. A large diff with no test change is itself a finding worth raising.

### 5. Documentation accuracy

Docs that contradict the code are worse than absent docs. Flag: comments and docstrings describing
behavior the diff just changed, README or usage examples now wrong, and public interfaces changed
without their documentation following. Stale documentation is a correctness finding about the
documentation, not a style note.

## Structural classes

Structural findings carry a tag as the **first token** of the finding text. Non-structural findings
(everything in dimensions 1–5 above) are **untagged**.

| Tag | Meaning |
|---|---|
| `[regression]` | A structural regression this PR introduces. |
| `[simplification]` | Avoidable complexity this PR adds that a simpler shape would remove. |

### `[regression]` classes

1. **A file crosses the size budget.** A change pushes a file past the budget, or newly creates one
   above it. Remedy: split along a real seam. A mechanical line-count trim is not the fix. A file
   already over budget that the PR edits without materially lengthening is pre-existing.
2. **A new ad-hoc branch bolted into an unrelated flow.** A special case added to a function whose
   responsibility does not cover it, where the caller — not the callee — should have handled it.
3. **Feature logic leaked into a shared or canonical path.** A code path every consumer traverses
   gains a branch that only one consumer needs. The cost lands on everyone; the benefit on one.

### `[simplification]` classes

4. **Near-duplicate of an existing canonical helper.** New code reimplements something the codebase
   already provides. Remedy: call the existing helper, or extend it if it is genuinely close but
   insufficient.
5. **Missed structural simplification.** The change works but a simpler shape carries the same
   behavior: a repeated block that wants extracting, nested conditionals that want early returns, a
   hand-rolled loop the standard library already implements.
6. **Thin wrapper / identity abstraction.** A function, class, or module that forwards to one other
   thing and earns nothing — no validation, no caching, no error mapping, no interface adaptation.
7. **Cast or optionality churn.** Values repeatedly cast, unwrapped, re-wrapped, or null-checked
   because the type does not say what the code knows. Remedy: fix the type at its source.

The throughline for 4–7 is **less code carrying the same behavior**. State the simpler shape
concretely — "this could be cleaner" is not a finding.

## Overridable defaults

Every value here is a default that a higher tier may replace. When a higher tier sets its own, the
higher tier wins and the review states which value it applied.

| Axis | Tier-1 default |
|---|---|
| File size budget (class 1) | none — a higher tier sets the number. With none set, class 1 fires only on a file this diff makes conspicuously large for its language and role. |
| Structural findings gate? | `[regression]` gates — it belongs in `### Blockers`. A `[simplification]` **never** gates in this skill: place it in `### Important` when the simpler shape is obvious and singular, `### Suggestions` otherwise. This is the owning statement of that placement rule. |
| Test requirement | New behavior needs a test that fails without the change |
| Doc requirement | Public interface changes update their docs |

## What this tier deliberately does not cover

Commit message format, branch naming, changelog and release process, issue-tracker linkage,
documentation tiering, log or audit-trail conventions, review sign-off procedure, and anything
stack-specific. Those are house rules — tier 2 and tier 3 own them. A reviewer applying only this
file must not invent them.
