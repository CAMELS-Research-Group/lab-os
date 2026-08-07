# Timeboxing — Quick Reference

The at-a-glance version of [`timeboxing.md`](timeboxing.md) (the full standard, v1.0).
Calibration rows go in [`timebox_calibration.md`](timebox_calibration.md).

## The idea in three sentences

Work expands to fill whatever time you give it, so give it a fixed box.
When the box runs out, shrink the deliverable — don't stretch the clock.
Write down how long things *actually* took, and set future boxes from that
record instead of from optimism.

## Starting a session

Say one line out loud (or type it) before touching anything:

> **Goal:** _what I'm making_ · **Box:** _N minutes_ · **Done when:** _the exit criterion_

If you can't fit the goal in one line, spend the first 10 minutes scoping — not producing.

## How long is the box?

| I'm about to… | Session type (log this name) | Box |
|---|---|---|
| Draft a PRD or spec | `PRD / spec first draft` | **90 min** |
| Fix review findings | `Review remediation` | **45 min** |
| Research something | `Research spike` | **25 min** |
| Debug something | `Debugging` | **30 min, then escalate** |
| Make a decision | `Decision session` | **15 min each** |

[`timeboxing.md`](timeboxing.md) § Default boxes by session type owns every value in this
table — the session-type names, the box lengths, and the exit criteria alike.
This page is a copy kept for scanning speed, so when the two disagree, that one
wins and this one is the bug. Write the middle column verbatim in your
calibration row so the loop can group by it; exit criteria are definitions of
done and live in that same table rather than here.

## During the session

- **One artifact per box.** Anything shiny you discover goes on a "next box" list, not into the current one.
- **Questions that aren't yours to answer** go into the artifact's Open Questions section. Don't spend box time deciding for the requester.

## When the timer fires

Work through this list in order — each step is only allowed if the one above it failed:

1. **Ship it.** Exit criterion met? Done. Log the row and stop.
2. **Shrink it.** Cut the artifact down to what's finishable right now. A smaller done thing beats a bigger half-thing.
3. **Extend once.** Max half the original box, and write down *why*. A second overrun means the budget was wrong — end the sitting and re-decide how much time this artifact deserves.

Legitimate exceptions (no guilt, but log the overrun):

- **Waiting on a sign-off** — that ends when the requester decides, not when a timer fires.
- **Live repro on screen** — capture the state first, then stop.
- **Genuine flow on the stated goal** — finish the thought. The box kills drift, not momentum.

## After the box: one calibration row

Append one line to the table in [`timebox_calibration.md`](timebox_calibration.md), newest last:

| Date | Session type | Artifact | Planned | Actual | Exit met? | Note |
|---|---|---|---|---|---|---|
| 2026-07-24 | PRD / spec first draft | encoders-v2 prd.md | 90 | 110 | no | extended +20, scope hammered Open Qs |

- **Planned / Actual** in minutes.
- **Exit met?** — judged *at the timer*, before any extension.
- **Note** — optional: "scope hammered", "extended +N", "interrupted", …

That's it. No prose, no justification — it's telemetry, not a diary. (This is
also why it lives here and not in `project_log.md`: the log standard bans bare
status rows.)

## Every few weeks

Skim the calibration table. If a session type keeps blowing its box (PRDs
"planned 90, actual 120" three times in a row), **the box is wrong, not you** —
raise the default and update **both** [`timeboxing.md`](timeboxing.md) and
the "How long is the box?" table above — the box lengths are the one value
both carry, and they desync silently otherwise.

## Bigger than one sitting?

That's an *appetite*, not a box: decide up front how many sittings the whole
artifact is worth ("this bundle gets three sittings"). When they're spent, the
work doesn't auto-renew — it goes back for an explicit yes/no on more time.
