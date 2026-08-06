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

- `lane` is `review` or `remediate` — the only two comments this skill posts. Step 6 asks questions
  and Step 8 posts nothing, so no third value is ever emitted.
- `head` is the short sha actually reviewed — not the branch tip at posting time, which may have
  moved.
- HTML comments do not render, so the marker is invisible to readers and reliable for tooling.

## Structure

In this order. Headings are `###` for the sections that stay open; the three collapsed sections carry
their name in the `<summary>` instead. Do not omit a section — an empty one carries information.

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

<details><summary><b>Suggestions</b> — <n>, or none</summary>

<numbered; or `None.`>
</details>

<details><summary><b>Load-bearing strengths</b> — <n>, or none</summary>

<bulleted — what is working and should survive revision>
</details>

<details><summary><b>Open questions for the author</b> — <n>, or none</summary>

<bulleted — what could not be resolved by reading, phrased as questions>
</details>
```

**Collapse is display-only; it drops nothing.** `### Suggestions`, `### Load-bearing strengths`, and
`### Open questions for the author` open collapsed so a reader sees the verdict line, the header, the
scope paragraph, and the two severity sections that gate the merge first, then expands what they
need — a full review comment routinely runs long, and the author's first question is "what must I fix",
which the open part answers. Collapse changes the default scroll length, not what must be present:
`None.` is still written rather than the section omitted, `### Load-bearing strengths` is still
**required** (§ Tone contract), and every finding still carries its location, mechanism, and proposed
fix inside the collapsed block.

**Collapse is not a licence to write more.** It settles where bytes sit on the page; § Proportionality
settles whether they are written at all, and the two are independent. A collapsed section still costs
the reviewer the attention that produced it, and still arrives in full in email notifications and in
any agent that ingests the comment downstream — neither honours `<details>`. Hiding a bloated section
is not the same as not bloating it.

**The verdict line, the `**Reviewed:**` line, the scope paragraph, `### Blockers`, and `### Important`
stay open.** They are the act-on-it-at-a-glance surface — the verdict, how much weight the review
carries, and everything the author has to address — so hiding them would defeat the point. The
`<summary>` for each collapsed section carries a **count** (or "none"), so the reader decides whether
to expand from the summary alone rather than having to open it to learn it is empty.

## Verdict vocabulary

Three tokens. The trigger is mechanical — the verdict follows the Blocker count, it is not a judgment
call layered on top — with **one override**, stated in its own row below, that a caller sets
deliberately and the skill never applies on its own.

| Verdict | Emoji | Trigger |
|---|---|---|
| `REQUEST CHANGES` | 🔴 | One or more Blockers. |
| `APPROVE` | ✅ | Zero Blockers **and** the reviewer read enough to say so — the scope paragraph must support it. |
| `COMMENT` | 💬 | Zero Blockers but coverage was partial: a generated artifact, a stack the reviewer could not evaluate, or a diff too large to read end-to-end. Withholds approval without asserting a defect. |
| `COMMENT` **(override)** | 💬 | `--comment-only` is set. Forces `COMMENT` regardless of Blocker count; the findings — Blockers included — still post in full, only the formal verdict is withheld. |

`COMMENT` exists so incomplete coverage is never laundered into an approval. Reaching for it is
correct when the honest answer is "I could not check the part that matters."

**An `APPROVE` must state explicitly whether anything was executed.** This lane reads and reasons; it
runs no gate — the rung ladder belongs to the remediate lane. So unless the scope paragraph names
something the reviewer actually ran, an `APPROVE` says in those words that no verification was
executed and the approval is from reading alone. Silence reads as "I ran it and it passed", the one
claim this verdict cannot make on its own.

**The `--comment-only` override exists because the mechanical derivation assumes the reviewer is the
merge authority.** A formal `APPROVE` or `REQUEST CHANGES` is a merge-gating act on most repositories;
a contributor without write access who posts one overstates their standing, however sound the
findings. The flag lets them post the full review and withhold the verdict claim, rather than choosing
between an overreaching verdict and no review at all. It is the only thing that decouples the token
from the Blocker count, and it never fires unless the caller passes it — so the base rule stays purely
mechanical for everyone else.

## Finding entries

Each numbered finding carries three things. A finding missing any of them is not ready to post:

1. **Location** — `path/to/file.ext:42`, or `path/to/file.ext` plus a heading when there is no
   meaningful line. Clickable and specific; "in the parser" is not a location.
2. **What is wrong** — the rule violated, the bug, or the conflict. Name the mechanism: what input
   reaches what sink, which two statements contradict each other, which branch is unreachable.
   "This looks fragile" is not a finding.
3. **A proposed fix** — concrete enough to act on. A finding with no proposed fix is an
   observation; either work it into a proposal or drop it. Weighing two or three defensible fixes in
   the open belongs to Blockers, where the choice is load-bearing and the author has to make it;
   below that tier, name the one you would pick and stop.

Structural findings additionally carry `[regression]` or `[simplification]` as the **first token**,
per `rubric-universal.md`. Non-structural findings are untagged.

Empty sections read `None.` — never delete the heading. A `### Blockers` section reading `None.` is
an assertion the reviewer made; a missing one is ambiguous. A collapsed section is written the same
way: `None.` inside the `<details>` block, with `none` in its `<summary>` count — the block is never
dropped for being empty.

## Proportionality

Length follows severity. Left unstated it inverts, because a Suggestion is easier to write at length
than a Blocker is to prove: the tier that costs the reviewer least attracts the most prose, and the
finding the author actually has to act on ends up outweighed by the ones they do not.

| Section | Default shape |
|---|---|
| `### Blockers` | Full apparatus — location, mechanism, fix, and the alternatives where the choice is real. Spend here. |
| `### Important` | Location, mechanism, and the fix you would pick. No alternatives menu. |
| `### Suggestions` | A sentence or two each. One needing a paragraph to motivate is an Important finding, or is not ready. |
| Scope paragraph | Two or three sentences: read end-to-end, verified by running, not examined. A coverage disclosure, not a narrative of the reading. |
| `### Load-bearing strengths` | Bullets, one line each. What must survive revision — not why it is admirable. |
| `### Open questions for the author` | The question, plus the line that prompted it. |

**Defaults, not caps.** An intricate Blocker earns the space it needs, and a diff genuinely examined
three ways earns a longer scope paragraph. What is never earned is reaching a length because the
section exists — a section with little to say says it briefly, and the `None.` rule already
establishes that an honest empty section is a real answer.

**The `mechanical` / `design-pin` label carries what the no-menu rule strips.** An Important finding
naming a single fix is indistinguishable in prose from a mechanical one, so the classification cannot
be recovered from the write-up — and nothing downstream tries to. The label rides the review lane's
structured return (`PROMPT.md` 5.2 step 3), the only place it is authoritative
(`classify-blockers.md` § Step 3 honours an emitted label); the comment body carries no label token
and needs none. Keep the label on the finding in the return, and the one-fix rule above stays safe to
follow.

**The test is relative, not absolute.** If the Suggestions outrun the Blockers, the review is
miscalibrated whatever its findings are worth. That comparison is quicker to run against a draft
than any byte count, and it catches the failure that matters: a merge-gating finding buried under
agreeable ones.

The reader is deciding what to change. Every sentence that does not move that decision competes with
the one that does.

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
  Blocker. `### Load-bearing strengths` is where genuine positives go, and it stays **required** on
  every verdict — it tells the author what not to break in revision, which is load-bearing
  information, not flattery. Collapsing it (§ Structure) changes where it sits on the page, not
  whether it is written.
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

<details><summary><b>Suggestions</b> — 1</summary>

1. [simplification] `retry.py:12–40` — the `pending` / `retrying` / `exhausted` booleans encode one
   three-state machine and the code defends the cross-field invariants by hand. A single
   `state` enum would delete them, but it touches several call sites — a real reframe, not a singular
   move, so it lands here rather than in Important.
</details>

<details><summary><b>Load-bearing strengths</b> — 2</summary>

- Retry policy is separated from transport, so the transport layer carries no policy state.
- The backoff-cap edge case has a test that would fail without the change.
</details>

<details><summary><b>Open questions for the author</b> — 1</summary>

- Is the cap intended to interact with the global request timeout, or are they independent budgets?
  `retry.py:31` reads as though one bounds the other, but nothing enforces it.
</details>
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
| Collapsing `### Blockers`, `### Important`, the scope paragraph, or the `##` header line | They are the act-on-it-at-a-glance surface; only the three lower sections collapse, and the author must not have to expand anything to learn what blocks the merge |
| Using collapse to omit a section rather than default-hide it | `<details>` changes default visibility, not presence; a section still reads `None.` when empty and still carries every finding it owns — a dropped finding is dropped whether or not it was behind a `<summary>` |
| A `<summary>` without its count | The count is what lets a reader skip an empty section unopened; a bare section name forces them to expand it to find nothing |
| Suggestions that outrun the Blockers | Inverts the budget (§ Proportionality); the merge-gating finding ends up outweighed by the ones the author can ignore |
| An alternatives menu on an Important finding or a Suggestion | Below Blocker tier the choice is not load-bearing; name the fix you would pick and stop |
| A scope paragraph that narrates the reading rather than disclosing coverage | It exists so the reader can weigh the review, not to evidence effort |
