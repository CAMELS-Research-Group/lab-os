# Review comment template

The shape of the comment the review lane posts on a pull request. This file **owns the finding
structure** for this skill — nothing outside `pr-round/` defines it.

The reader is a maintainer who did not write this review and cannot see the reviewer's reasoning.
Everything they need to act must be in the comment.

## Machine marker (owning source)

Every comment this skill posts **begins** with a marker line. It is defined here and nowhere else;
the staleness filter greps for the `pr-round:v1` prefix to find this skill's own prior comments.

```
<!-- pr-round:v1 lane=review head=<sha7> -->
```

- `lane` is `review` or `remediate` — the only two comments this skill posts. Step 5a asks questions
  and Step 5b posts nothing, so no third value is ever emitted.
- `head` is the short sha actually reviewed — not the branch tip at posting time, which may have
  moved.
- HTML comments do not render, so the marker is invisible to readers and reliable for tooling.

## Structure

In this order. Headings are `###` for findings sections. Do not omit a section — an empty one
carries information.

```markdown
<!-- pr-round:v1 lane=review head=<sha7> -->
## <verdict emoji> <VERDICT> — <one-line rationale>

**Reviewed:** <sha7> on `<branch>` · <N> files, +<A>/-<D> · standard: <tiers applied>

<Scope paragraph: what was read end-to-end, what was verified by running or tracing, what was
spot-checked, and what was NOT examined. A reader must be able to tell how much weight this
review carries.>

### Blockers

<numbered; or `None.`>

### Important

<numbered; or `None.`>

### Suggestions

<numbered; or `None.`>

### Load-bearing strengths

<bulleted — what is working and should survive revision>

### Open questions for the author

<bulleted — what could not be resolved by reading, phrased as questions>
```

## Verdict vocabulary

Exactly three. The trigger is mechanical — the verdict follows the Blocker count, it is not a
judgment call layered on top.

| Verdict | Emoji | Trigger |
|---|---|---|
| `REQUEST CHANGES` | 🔴 | One or more Blockers. |
| `APPROVE` | ✅ | Zero Blockers **and** the reviewer read enough to say so — the scope paragraph must support it. |
| `COMMENT` | 💬 | Zero Blockers but coverage was partial: a generated artifact, a stack the reviewer could not evaluate, or a diff too large to read end-to-end. Withholds approval without asserting a defect. |

`COMMENT` exists so incomplete coverage is never laundered into an approval. Reaching for it is
correct when the honest answer is "I could not check the part that matters."

## Finding entries

Each numbered finding carries three things. A finding missing any of them is not ready to post:

1. **Location** — `path/to/file.ext:42`, or `path/to/file.ext` plus a heading when there is no
   meaningful line. Clickable and specific; "in the parser" is not a location.
2. **What is wrong** — the rule violated, the bug, or the conflict. Name the mechanism: what input
   reaches what sink, which two statements contradict each other, which branch is unreachable.
   "This looks fragile" is not a finding.
3. **A proposed fix** — concrete enough to act on. When several fixes are defensible, name two or
   three and say which you would pick and why. A finding with no proposed fix is an
   observation; either work it into a proposal or drop it.

Structural findings additionally carry `[regression]` or `[simplification]` as the **first token**,
per `rubric-universal.md`. Non-structural findings are untagged.

Empty sections read `None.` — never delete the heading. A `### Blockers` section reading `None.` is
an assertion the reviewer made; a missing one is ambiguous.

## Tone contract

This lands on someone else's work, under a real identity, permanently. The review posture is the
same as any outsider review — read what is there, not what was meant — with additions that only
matter when the author is another person:

- **Critique the artifact, never the author.** "This function does X" — never "you did X", never a
  characterization of the author's care, skill, or attention.
- **Uncertainty about intent is a question, not a defect.** When the code could be deliberate,
  ask in `### Open questions` instead of asserting it is wrong. Being confidently wrong at someone
  costs more than one extra round-trip.
- **Technical reasoning for every finding.** Name the rule, the conflict, or the failing input. A
  finding a maintainer cannot verify from the comment alone is not actionable.
- **No performative praise.** No "great work", no "nice approach", no softening preamble before a
  Blocker. `### Load-bearing strengths` is where genuine positives go, and it stays **required** —
  it tells the author what not to break in revision, which is load-bearing information, not
  flattery.
- **Say what was not checked.** An unstated gap reads as coverage. The scope paragraph is where
  honesty about depth lives.
- **State which tiers applied.** A finding's authority depends on the standard that produced it.

## Worked example

```markdown
<!-- pr-round:v1 lane=review head=e128257 -->
## 🔴 REQUEST CHANGES — one unbounded read on an untrusted path

**Reviewed:** e128257 on `feat/ingest-retry` · 4 files, +212/-31 · standard: tier 1 + tier 2 (no project rubric)

Read `client.py`, `retry.py`, and both new tests end-to-end. Traced the retry path by hand against
the documented backoff defaults. Did **not** run the suite — no fixture data in the checkout — so
the test assertions are reviewed by reading only.

### Blockers

1. `client.py:88` — `resp.read()` is called with no size limit on a response from a
   caller-supplied URL. A large or hostile endpoint exhausts memory before the timeout fires; the
   timeout bounds latency, not bytes. Pass an explicit `amt` and fail closed past the cap, or
   stream into a bounded buffer.

### Important

1. `retry.py:24` — the backoff cap is read from config but never validated. A negative value makes
   `min()` return it and the loop retries with no delay. Add a `>= 0` guard at load time so the
   failure surfaces at startup rather than under load.
2. [simplification] `client.py:140` — `_build_headers()` forwards to `dict(base)` and adds nothing.
   Inlining it at the two call sites removes a hop without losing anything. (Obvious and singular, so
   it lands in Important, not Suggestions — `rubric-universal.md` § Overridable defaults owns that
   placement rule.)

### Suggestions

1. [simplification] `retry.py:12–40` — the `pending` / `retrying` / `exhausted` booleans encode one
   three-state machine and the code defends the cross-field invariants by hand. A single
   `state` enum would delete them, but it touches several call sites — a real reframe, not a singular
   move, so it lands here rather than in Important.

### Load-bearing strengths

- Retry policy is separated from transport, so the transport layer carries no policy state.
- The backoff-cap edge case has a test that would fail without the change.

### Open questions for the author

- Is the cap intended to interact with the global request timeout, or are they independent budgets?
  `retry.py:31` reads as though one bounds the other, but nothing enforces it.
```

## Common deviations

| Deviation | Why it fails |
|---|---|
| `## Blockers` instead of `### Blockers` | Breaks the section structure readers and tooling both key on |
| Omitting `### Blockers` when the count is zero | Ambiguous — write `None.` |
| A finding with no proposed fix | Not actionable; it is an observation |
| Approving without a scope paragraph that supports it | An approval whose coverage is unstated is unearned — use `COMMENT` |
| Bulleted findings instead of numbered | Findings get referenced by number in replies |
| Softening a Blocker with praise | Buries the finding the author most needs to see |
