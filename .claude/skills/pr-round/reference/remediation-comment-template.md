# Remediation handoff comment template

The shape of the **single** comment a remediation round posts on your own pull request, at Step 5c,
once the whole round is complete — mechanical fixes applied, decisions made and applied, gate run.

The marker literal is owned by `review-comment-template.md` § Machine marker and is **never spelled
out here** — this file only pins the two fields it sets, `lane=remediate` and `head` = the final sha.
Restating the literal would give the `pr-round:v1` token a second definition site, and a version bump
that landed in only one of them would silently break the staleness filter.

## What this comment is for

It is the **handoff artifact between remediator and reviewer.** A round ends by handing the PR back
for re-review, and this comment is what the next reviewer reads first. It must answer, without them
opening the diff:

- What feedback was taken in, and was any of it missed?
- What changed, and at which commit?
- What judgment calls were made, and on what reasoning?
- What is still open, and why?
- Is this ready for another look?

**Exactly one per round.** Nothing is posted mid-round. A comment written before decisions are
applied would be stale by the time the round ends and would leave resolved items reading as open.

## Structure

```markdown
<marker per review-comment-template.md, with lane=remediate and head=<final sha7>>
## <emoji> <READINESS> — <one-line summary>

**Ingested:** <sources and counts> · **Head:** `<sha7>` · **Gate:** <result>

### Addressed

<numbered; each: finding -> file:line -> what changed. Or `None.`>

### Decisions made

<numbered; each: the choice, the reasoning, what was applied. Or `None.`>

### Still open

<numbered; each: the item and why it is still open. Or `None.`>

### Not actioned

<numbered; each: the finding and the reason it was left. Or `None.`>
```

## Readiness verdict

Mirrors the review lane's verdict so both halves of the loop speak the same language. The trigger is
mechanical.

| Verdict | Emoji | Trigger |
|---|---|---|
| `READY FOR RE-REVIEW` | ✅ | Every ingested finding is addressed or explicitly not-actioned, nothing deferred, gate green. |
| `PARTIALLY ADDRESSED` | ⚠️ | Gate green and work landed, but something is deferred or still open. The PR moved; it is not finished. |
| `BLOCKED` | 🔴 | The gate is red, the gate could not run at all (rung 3), a push failed, or the round could not complete. **Nothing was pushed** in any of those cases — say what broke and where the work is. |

A round that fixed nothing and had nothing to fix is `READY FOR RE-REVIEW` with `None.` throughout —
that is a real and useful outcome, and the comment should say so plainly rather than imply activity.

## Header fields

- **Ingested** — where the feedback came from and how much, e.g.
  `2 review bodies, 7 inline comments, 1 issue comment, 1 failing check`. This is the coverage audit:
  a reader can tell whether a finding was missed or never seen.
- **Head** — the short sha the round **ended** on, after decisions were applied. Required whenever
  anything was pushed, so every claim below is checkable against `git log`. Nothing pushed →
  `unchanged — no commit`.
- **Gate** — the verification's real result **and which rung produced it** (`PROMPT.md` § 4.3 step 5
  owns the ladder). Never omit it; never imply green by silence. A gate reported green must have been
  run unpiped — a piped gate swallows the exit code and a red one reads as green.

  | Rung | Written as |
  |---|---|
  | 1 — repo's designated gate | `full — green` · `full — red (<summary>)` |
  | 2 — scoped substitute | `scoped — green (<exact commands>; uncovered: <what a full gate would have covered>)` |
  | 3 — gate exists, could not run | `not run (<reason>)` — nothing was pushed |
  | 4 — repo defines no gate | `none (repo defines none) — unverified` |

  **A scoped gate is never written as plain `green`.** The fallback rungs are sanctioned precisely
  because they are disclosed; a rung-2 result that reads like a rung-1 result converts a partial
  verification into a full-verification claim, which is the failure the ladder exists to prevent. Name
  the commands and name the gap — a reader deciding whether to re-review needs to know what was not
  checked.

## Section rules

### Addressed

One entry per fix, naming the originating finding, the file and line, and what actually changed.
Specific enough to diff the claim against: "tightened validation" is not an entry; "added a `>= 0`
guard at load time so a negative cap fails at startup" is.

This section covers **both** mechanical fixes and the edits that came out of decisions — a reader
does not care which phase produced a change. `### Decisions made` explains the *reasoning* for the
subset that needed one.

An empty `### Addressed` means no fix landed; write `None.` and set Head to `unchanged — no commit`.
A comment implying work happened when none did is worse than no comment — it makes an untouched PR
look handled.

### Decisions made

Only judgment calls. Each entry gives the choice made, **why** that option over the alternatives, and
what was applied as a result. This is the section that survives longest: six months on, this is where
"why is it like this" gets answered.

By the time this comment posts, every decision here is **resolved**. There is no "awaiting your
decision" section — anything unresolved is either deferred (`### Still open`) or was never a decision
(`### Not actioned`).

### Still open

Items that remain: a decision deliberately deferred, work too large for this round, something blocked
on an external answer. Each says why it is open and what would close it. Deferred is a legitimate
outcome; silently dropping is not.

This section also carries **out-of-band follow-ups** — resolutions whose action lies outside what this
skill does (`PROMPT.md` § 4.3 step 8): an issue to file, a review thread to reply to, a collaborator
to ask, a repo setting to change. The skill posts one comment per PR and files nothing, so each entry
names the action and who takes it. Recording a follow-up here is not performing it, and the entry
should not read as though it were.

### Not actioned

Feedback seen and deliberately left — out of scope, superseded by a later comment, already fixed,
disagreed with (say so, and why). This is what makes the coverage claim honest: a finding appearing
nowhere reads as missed.

## Worked example

```markdown
<marker per review-comment-template.md, with lane=remediate and head=7c41a09>
## ⚠️ PARTIALLY ADDRESSED — 3 findings closed, 1 deferred to a follow-up

**Ingested:** 1 review body, 5 inline comments, 1 failing check · **Head:** `7c41a09` · **Gate:** scoped — green (`pytest tests/test_retry.py tests/test_client.py`, `ruff check .`; uncovered: the integration suite, which needs the staging credentials)

### Addressed

1. `retry.py:24` — added a `>= 0` guard on the backoff cap at load time, so a negative value fails
   at startup instead of silently disabling the delay.
2. `README.md:61` — corrected the documented default from `3` to `5`, matching the code since the
   config refactor.
3. `client.py:88` — bounded the response read at 10 MB and made it fail closed past the cap.

### Decisions made

1. **Unbounded `resp.read()` on a caller-supplied URL** (`client.py:88`) — capped at 10 MB and fail
   closed, rather than streaming into a bounded buffer. The cap matches the limit already enforced
   upstream at ingest, so the two agree and neither has to know about the other; streaming would
   have reshaped the return type and touched all three call sites for a payload size this client
   has never seen. If bulk responses become real, streaming is the follow-up.

### Still open

1. `pipeline.py` module size — deferred. It sits near the budget and this PR removes lines from it,
   so the split is not urgent, but the next feature that adds to it will cross. Closing it means
   choosing a seam, which is a bigger change than this PR should carry.
2. **Out-of-band — operator to action.** The reviewer's second comment asks for the streaming variant to
   be tracked rather than built here. This round does not file issues, so nothing was filed: the
   follow-up is to open one against `client.py` referencing decision 1 above.

### Not actioned

1. `parser.py:88` — reviewer flagged the nested conditional as hard to follow. Pre-existing; this PR
   does not touch that function. Out of scope for this round.
```

## Common deviations

| Deviation | Why it fails |
|---|---|
| Posting mid-round, before decisions are applied | Stale on arrival; leaves resolved items reading as open |
| More than one comment per round | The handoff stops being a single artifact; the reviewer has to reconstruct order |
| An "awaiting your decision" section | Decisions are resolved at 5a — anything left is deferred, and belongs in `### Still open` |
| Omitting the Head sha after pushing | The round becomes unverifiable |
| `### Addressed` populated with nothing pushed | Claims work that does not exist on the branch |
| Reporting the gate green without running it unpiped | A piped gate swallows the exit code; the claim is unfounded |
| Writing a scoped (rung-2) gate as plain `green` | Converts a partial check into a full-verification claim — the exact thing disclosure exists to prevent |
| An out-of-band follow-up phrased as though the round did it | The skill files nothing and replies to no thread; the reader assumes it was handled |
| A decision entry with no reasoning | The section's whole value is the *why*; the *what* is already in `### Addressed` |
| Findings appearing in no section | Reads as missed rather than considered |
