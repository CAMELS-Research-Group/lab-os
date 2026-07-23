# Finding classification — mechanical vs design-pin

> **Vendored copy.** Adapted from the `pr-review-loop` skill's file of the same name, taken once and
> reshaped for a **single round**: the original's age ledger, per-pass decay, and cross-pass
> fingerprint keys are removed because this skill makes one pass and stops. The two copies are
> independent and are expected to drift; neither is canonical for the other.

Both lanes use this tree. The **remediate lane** walks it to decide whether to auto-fix a finding or
hand it back for a decision. The **review lane** walks it to label each finding for the PR's author,
so a maintainer can see at a glance which findings are a small edit and which need them to choose.

Conservative default: **ambiguous → design-pin.** A false ask costs one round-trip. A false auto-fix
ships a wrong change under the reviewer's name.

## Decision tree

```
For each finding:

  0. Would the edit assert a DECISION STATE (ratified / approved / agreed / signed off /
     superseded), or land in a SHARED MULTI-OWNER document (design authority, spec,
     ledger, decision record)?
     YES -> DESIGN-PIN, and stop. Size does not redeem it: a one-word edit that
            claims a group decided something is still that claim.
     NO  -> continue

  1. Is there a hard rule violation with a single text-level fix?
     (e.g. "subject is 84 chars, rule says <= 72" -> trim it)
     YES -> MECHANICAL
     NO  -> continue

  2. Is it a missing path / missing edge / wording typo / count inconsistency?
     (e.g. "the file list names handler.py but the acceptance also asserts validator.py"
       -> add the missing path)
     YES -> MECHANICAL
     NO  -> continue

  3. Does the finding enumerate 2+ resolution options?
     YES -> continue to (4)
     NO  -> if a clear simplest-defensible fix exists -> MECHANICAL; else DESIGN-PIN

  4. Among the enumerated options, is one OBVIOUSLY simplest
     AND free of downstream redesign implications?
     YES -> MECHANICAL (apply that option)
     NO  -> DESIGN-PIN

  5. Still ambiguous -> DESIGN-PIN (conservative fall-through)
```

## Mechanical markers

- Hard rule violation with a single text-level fix (length caps, size limits, anchor format)
- A missing path that the surrounding text explicitly asserts
- A dead cross-reference resolving to a clearly-renamed target
- A missing dependency edge already discoverable from other declared edges
- Wording typo — singular/plural, off-by-one count, repeated word
- A future-tense reference to something already closed
- A heading level wrong where the format pins the level
- Two enumerated options where one is simplest-defensible with no downstream redesign

## Design-pin markers

- Option A vs option B where both carry meaningfully different downstream implications — consumer
  surface, performance characteristic, CI topology, schema shape
- "Pin a shape" framing: a vague type or signature must become concrete and the choice affects
  multiple consumers
- Conflicting interpretations the change inherited and the spec does not pre-resolve
- Ambiguity affecting more than one component or downstream change
- A missing consent surface where the fix is "ask about X" and X is a real choice
- Prose that explicitly says "pick one" or "implementer's choice with downstream impact"
- **The edit asserts a decision state** — that something was ratified, approved, agreed, accepted,
  signed off, superseded, or settled. The edit may be a one-word wording change and still be a claim
  about what a group decided, which no amount of local text inspection can verify.
- **The edit lands in a shared multi-owner document** — a design-authority doc, spec, ledger,
  standard, or decision record whose readers treat it as the record rather than as prose. The same
  correction is mechanical in a module docstring and a design-pin here, because the blast radius is
  everyone who reads the file as authoritative.

## Structural findings

Structural findings are tagged by the rubric (`rubric-universal.md`, plus any higher tier). **Which
section a tag lands in, and whether it gates, is the rubric's to say** — `rubric-universal.md` §
Overridable defaults owns that rule, and a higher tier may override it there. Do not restate it here;
a second copy would disagree with a tier-2 or tier-3 override without either file noticing, and the
review verdict is derived mechanically from the Blocker count.

This file owns only the mechanical-vs-design-pin call, which is the same tree above for both tags:

- **`[regression]`** — classify through the tree unchanged: a regression with one text-level fix is
  mechanical; one with several defensible shapes and downstream cost is a design-pin. The tag adds no
  special path.
- **`[simplification]`** — same tree, with one added rule: auto-fix only when the simpler shape is
  **obvious and singular** — inline a thin wrapper, call the existing canonical helper, extract one
  repeated block. Anything that reframes a state model, resequences a flow, or collapses a layer
  multiple callers traverse is a **design-pin**, because more than one defensible target shape exists.

Because this skill makes a single pass, a design-pin simplification is asked about **once**. If it is
deferred, it is recorded in the PR comment and left — there is no later round to raise it again.

## Worked examples

### 1. Mechanical — hard rule, single fix

> "12 of 19 commit subjects exceed the 72-char cap. Longest is 84. The rule is unambiguous."

Tree: (1) hard rule with a single text-level fix → **MECHANICAL**. Trim each subject, preserving the
load-bearing terms.

*Teaches:* when the finding names its own fix as mechanical, trust it.

### 2. Mechanical — despite three-option framing

> "Task 5 needs a `max_retries` field, but Task 1 doesn't declare it and Task 5 doesn't list
> `config/models.py` as modified. Either add it to Task 1, or list the file in Task 5, or drop the
> field option."

Tree: (1) no. (2) close — a missing declaration is path-adjacent. (3) three options. (4) option 1 is
purely additive with no downstream change; option 3 would rework Task 5's prose → **MECHANICAL**,
apply option 1.

*Teaches:* "either X or Y or Z" does not mean design-pin. Walk the options and look for one that is
obviously additive.

### 3. Design-pin — options touch a user-facing surface

> "The acceptance asserts CI check names the workflow ships differently. Either fix the acceptance to
> match the existing jobs, or require the split in acceptance and files."

Tree: (1) no. (2) no. (3) two options. (4) option 1 changes the user-facing gate count; option 2
changes CI topology and billing. Neither is clearly safer → **DESIGN-PIN**.

*Teaches:* when options touch a user-facing surface — CI topology, consumer API, schema shape — even
a mechanical-looking fix is a pin.

### 4. `[simplification]` — obvious shape, mechanical

> "[simplification] `client.ts` — `getUser()` is a one-line wrapper around `api.fetch('/user')` that
> adds no validation, caching, or error mapping."

An identity abstraction with a single call shape is a mechanical delete. Inline it at the call sites.

### 5. `[simplification]` — restructure, design-pin

> "[simplification] `reducer.ts` — the `pending` / `loading` / `inFlight` booleans encode one
> three-state machine; a single `status` enum would delete the cross-field invariants."

A state-model reframe with multiple call sites reading the booleans and more than one defensible
target shape → **DESIGN-PIN**. Asked once, with "apply now" and "defer" as options. Deferred means
recorded in the comment and left.

### 6. Design-pin — a wording fix that asserts a decision state

> "§Scope still calls S1 and S2 provisional. They were ratified as-written on the 2026-07-18 ledger —
> update the wording to match."

Tree: (0) the edit asserts a **ratification state**, in a **design-authority doc** → **DESIGN-PIN**,
and the tree stops there. Every later step would have said mechanical: it is a two-word wording change
with an unambiguous target, and the finding names its own fix.

It fails because the claim is not checkable from the text being edited. The finding asserts a
ratification; whether it happened lives in the ledger, and that ledger may say ratification is pending
a round that has not run. Applying it writes a false decision-state claim into the document readers
treat as the record of decisions — and the wrongness is invisible in the diff, which shows only a
tidy wording correction.

*Teaches:* classify by **what the edit claims**, not by how large it is. "Small, unambiguous, and
names its own fix" is exactly the shape of the most expensive false auto-fix in this file. When the
edit's correctness depends on a fact outside the file, the operator confirms the fact.

### 7. `[regression]` — ordinary tree

> "[regression] `pipeline.ts` — the export-CSV special case now sits in the shared `serialize()` that
> every output format traverses."

The fix — lift the branch back to the export-CSV caller — is one structural move with no competing
shapes → **MECHANICAL**. Had it been "pick one of three homes, each with downstream cost," it would
have been a design-pin. The tag marks it a hard Blocker; it does not change how it classifies.
