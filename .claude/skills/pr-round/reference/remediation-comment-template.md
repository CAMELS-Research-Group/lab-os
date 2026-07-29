# Remediation handoff comment template

The shape of the **single** comment a remediation round posts on your own pull request, at Step 9,
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

<details><summary><b>Decisions made</b> — <n>, or none</summary>

<numbered; each: the choice, the reasoning, what was applied. Or `None.`>
</details>

<details><summary><b>Still open</b> — <n>, or none</summary>

<numbered; each: the item and why it is still open. Or `None.`>
</details>

<details><summary><b>Not actioned</b> — <n>, or none</summary>

<numbered; each: the finding and the reason it was left. Or `None.`>
</details>
```

**Collapse is display-only; it drops nothing.** `### Decisions made`, `### Still open`, and
`### Not actioned` open collapsed so a reader sees the verdict line, the header, and `### Addressed`
first, then expands what they need — a handoff comment routinely runs long, and the reviewer's first
question is "did it move and is it ready", which the open part answers. Every rule for those sections
below is unchanged: a finding still appears in exactly one, `None.` is still written rather than the
section omitted, and the § Section rules coverage guarantee ("a finding appearing nowhere reads as
missed") still binds inside the collapsed block. What changed is the default scroll length, not what
must be present.

**`### Addressed` and the `##` header line stay open.** They are the ready-at-a-glance surface —
whether the round moved the PR, and to what commit — so hiding them would defeat the point. The
`<summary>` for each collapsed section carries a **count** (or "none"), so the reader decides whether
to expand from the summary alone rather than having to open it to learn it is empty.

## Readiness verdict

Mirrors the review lane's verdict so both halves of the loop speak the same language. The trigger is
mechanical.

| Verdict | Emoji | Trigger |
|---|---|---|
| `READY FOR RE-REVIEW` | ✅ | Every ingested finding is addressed or explicitly not-actioned, every returned design-pin decided, nothing deferred, gate green. |
| `PARTIALLY ADDRESSED` | ⚠️ | Gate green and work landed, **every returned design-pin was put to the operator**, and something they deferred — or an out-of-band follow-up — is still open. The PR moved; it is not finished. |
| `BLOCKED` | 🔴 | The gate is red, the gate could not run at all (rung 3), a push failed, a returned design-pin was **never asked** (§ Still open, class 3), or the round could not complete. Say what broke and where the work is. Except in the never-asked case, **nothing was pushed**; a round that pushed fixes and still failed to ask says both. |

**`PARTIALLY ADDRESSED` is not reachable on un-asked pins.** The distinction is the point: partial
means the operator saw every pin and chose to leave some, which is a finished round with open items.
A pin nobody asked about is the round failing to do its job, and grading it ⚠️ launders that failure
into a normal outcome — which is precisely how a queue of pins gets carried silently from round to
round. It grades 🔴 and the comment names it.

A round that fixed nothing and had nothing to fix is `READY FOR RE-REVIEW` with `None.` throughout —
that is a real and useful outcome, and the comment should say so plainly rather than imply activity.

## Header fields

- **Ingested** — where the feedback came from and how much, e.g.
  `2 review bodies, 7 inline comments, 1 issue comment, 1 failing check`. This is the coverage audit:
  a reader can tell whether a finding was missed or never seen.
- **Head** — the short sha the round **ended** on, after decisions were applied. Required whenever
  anything was pushed, so every claim below is checkable against `git log`. Nothing pushed →
  `unchanged — no commit`.
- **Gate** — the verification's real result **and which rung produced it** (`PROMPT.md` § 5.3 step 5
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
decision" section — anything unresolved was deferred (`### Still open`, class 1), was never a
decision (`### Not actioned`), or was never asked at all (`### Still open`, class 3 — a round
failure, and the only case where something genuinely still awaits the operator).

### Still open

Items that remain. **Every entry opens with its class**, because the three are different facts and a
reader who cannot tell them apart cannot tell a finished round from a failed one:

| Class | Written as | What it means |
|---|---|---|
| 1 — **Deferred by you** | `**Deferred by you.**` | The pin was put to the operator at Step 6 — individually, or in aggregate through the volume guard — and they chose to leave it. A decision was made. A legitimate outcome. |
| 2 — **Out-of-band** | `**Out-of-band.**` | The resolution lies outside what a remediation agent does (`PROMPT.md` § 5.3 step 8): a review thread to reply to, a collaborator to ask, a repo setting to change, work too large for the round. |
| 3 — **⚠️ Never asked** | `**⚠️ Never asked — this round failed to put this to you.**` | The pin was returned by the remediate agent and the round ended without asking about it (`PROMPT.md` § Step 6 step 6): the run was interrupted, a question errored, the batches stopped short. **No decision exists.** |

Each entry says why it is open and what would close it. For class 3, "what would close it" is
*asking* — name the reason the ask did not happen, and state that the item is still undecided.

**Class 3 is never written as class 1.** A deferral is the operator's decision; an un-asked pin is
the skill's failure to obtain one. Labelling the second as the first reports a choice nobody made,
and it is the exact defect this split exists to prevent — the reader's next move differs (accept the
deferral, versus answer the question that was skipped), so the label has to carry it. A class-3 entry
also forces the readiness verdict to `BLOCKED` (§ Readiness verdict), so a comment carrying one and
grading ⚠️ contradicts itself.

For classes 1 and 2: where Step 7 filed a follow-up issue for the item (own PRs, consent permitting),
the entry cites it — `tracked as #<N>`. Where no issue was filed, the entry names the action and who
takes it; recording a follow-up is not performing it, and the entry should not read as though it
were.

**Class 3 carries no issue number**, because Step 7 does not file for it (`PROMPT.md` § Step 7 files
deferrals and out-of-band items — decisions the operator made, and actions outside the round). An
un-asked pin is neither: it is this round's unfinished work, and filing it would move a visible
failure into a backlog where it reads as tracked. It stays in this comment and in the roster, and the
🔴 verdict is what gets it looked at.

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

<details><summary><b>Decisions made</b> — 1</summary>

1. **Unbounded `resp.read()` on a caller-supplied URL** (`client.py:88`) — capped at 10 MB and fail
   closed, rather than streaming into a bounded buffer. The cap matches the limit already enforced
   upstream at ingest, so the two agree and neither has to know about the other; streaming would
   have reshaped the return type and touched all three call sites for a payload size this client
   has never seen. If bulk responses become real, streaming is the follow-up.
</details>

<details><summary><b>Still open</b> — 2</summary>

1. **Deferred by you.** `pipeline.py` module size — you chose to leave it. It sits near the budget and
   this PR removes lines from it, so the split is not urgent, but the next feature that adds to it
   will cross. Closing it means choosing a seam, which is a bigger change than this PR should carry.
2. **Out-of-band — tracked as #87.** The reviewer's second comment asks for the streaming variant to
   be tracked rather than built here; this round filed the follow-up issue against `client.py`,
   referencing decision 1 above.
</details>

<details><summary><b>Not actioned</b> — 1</summary>

1. `parser.py:88` — reviewer flagged the nested conditional as hard to follow. Pre-existing; this PR
   does not touch that function. Out of scope for this round.
</details>
```

## Common deviations

| Deviation | Why it fails |
|---|---|
| Posting mid-round, before decisions are applied | Stale on arrival; leaves resolved items reading as open |
| More than one comment per round | The handoff stops being a single artifact; the reviewer has to reconstruct order |
| An "awaiting your decision" section | Step 6 drains the pin queue — anything left is deferred, out-of-band, or a never-asked round failure, and all three belong in `### Still open` under their own class label |
| A `### Still open` entry with no class label | Classes 1 and 3 read identically without it, which is exactly the confusion the labels exist to remove — one is your decision, the other is the round's failure |
| A never-asked pin written as "deferred" | Reports a decision the operator never made, and hides the round's own gap behind their name |
| ⚠️ `PARTIALLY ADDRESSED` on a comment carrying a class-3 entry | Self-contradictory: a never-asked pin is 🔴 `BLOCKED`, because the round did not finish the job it exists to do |
| Omitting the Head sha after pushing | The round becomes unverifiable |
| `### Addressed` populated with nothing pushed | Claims work that does not exist on the branch |
| Reporting the gate green without running it unpiped | A piped gate swallows the exit code; the claim is unfounded |
| Writing a scoped (rung-2) gate as plain `green` | Converts a partial check into a full-verification claim — the exact thing disclosure exists to prevent |
| An out-of-band follow-up phrased as though the round did it | The round replies to no thread and performs no action beyond Step 7's issue filing; an unfiled item that reads as handled is a dropped item |
| A decision entry with no reasoning | The section's whole value is the *why*; the *what* is already in `### Addressed` |
| Findings appearing in no section | Reads as missed rather than considered |
| Collapsing `### Addressed` or the `##` header line | They are the ready-at-a-glance surface; only the three lower sections collapse, and the reader must not have to expand anything to learn whether the round moved the PR |
| Using collapse to omit a section rather than default-hide it | `<details>` changes default visibility, not presence; a section still reads `None.` when empty and still carries every finding it owns — a dropped finding is dropped whether or not it was behind a `<summary>` |
| A `<summary>` without its count | The count is what lets a reader skip an empty section unopened; a bare section name forces them to expand it to find nothing |
