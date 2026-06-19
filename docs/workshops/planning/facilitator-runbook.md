# Planning Session — Facilitator Runbook

**Status:** draft
**Audience:** Facilitators running the Planning part of the workshop program. Read alongside the
participant pages under `site/docs/workshops/planning/`. Participant materials contain the full
content; this runbook tells you how to hold the room.

---

## Before the session

Confirm participants have completed the [pre-work worksheet](../../../site/docs/workshops/planning/pre-work.md):
workspace set up, project idea captured. Anyone who has not done pre-work is not blocked from
attending, but you will need to give them five to ten minutes at the start of the journaling
segment to do it live. Budget for this if the cohort is likely under-prepared.

Have the [PRD-interrogation worksheet](../../../site/docs/workshops/planning/prd-interrogation-worksheet.md)
and [plan/decomposition worksheet](../../../site/docs/workshops/planning/plan-decomposition-worksheet.md)
open for reference so you can help participants navigate them during work time.

---

## Run-of-show

### Segment 1 — Framing (10 min)

**What:** Orient the cohort to what they are about to do and why the three-movement arc exists.

Cover briefly:
- The goal: leave with an execution-ready plan for their own project, not a generic exercise
- What "execution-ready" means concretely (the completion-checklist bar: measurable PRD + ordered,
  individually-verifiable task list)
- The three movements: capture the raw idea, interrogate it into a PRD, decompose into tasks
- This part's output is what Part 2 — Building runs against

Do not go deep on the interrogation technique yet — that is taught just-in-time before Segment 2.

**Timing note:** Keep this tight. Participants are antsy to get into their own projects. Five
minutes of framing and five minutes of questions is enough.

---

### Segment 2 — Movement 1: Capture the raw idea (20 min)

**What:** Participants externalize their project idea using one of the journaling styles.

Teach (5 min):
- Walk through the four journaling options (paragraph dump, question chain, voice-to-text,
  existing notes) and the single criterion for picking one: whatever gets the idea out of their
  head fastest
- The output is not a PRD. It is raw material — a few paragraphs, enough to be interrogated
- Explicitly name the anti-pattern: over-editing or trying to "get it right" at this stage

Participants work (15 min):
- Everyone writes independently. Remind them at the five-minute mark that they do not need
  more than a page
- Walk the room. If someone is staring at a blank page, ask: "What problem are you trying to
  solve, and who has it?" That is usually enough to unblock them

**Closing the segment:** Quick show of hands — who has something written down? Do not ask people
to share; just confirm everyone has something to work from. If a participant has nothing, they are
your first priority in Segment 3.

---

### Segment 3 — Movement 2: AI interrogation into a PRD (45 min)

**What:** Participants use the PRD-interrogation worksheet to have Claude ask the clarifying
questions until each PRD section is filled with concrete, falsifiable content.

Teach (10 min):
- Explain the interrogation technique: paste the raw capture, ask Claude to play the role of a
  demanding requirements interviewer, answer each question honestly, keep going until each section
  is concrete
- Walk through what a *finished* section looks like vs. a vague one — particularly success
  criteria (measurable vs. vibes) and out-of-scope (explicit list vs. "we'll figure it out")
- Show the PRD skeleton from the worksheet and where each answer lands
- Name the done condition: when Claude cannot find another meaningful clarifying question

Participants work (30 min):
- Everyone runs their interrogation independently. Circulate and check progress
- The most common place people stall is success criteria — they write something that sounds
  measurable but is actually a feeling ("it feels smooth," "users enjoy it"). Coach them to
  convert: "Can you check that in a year without asking anyone? What would you observe?"
- A participant who finishes early: ask them to read their success criteria aloud and find at
  least one that is still a vibe. There is almost always one

Check-in / debrief (5 min):
- Ask 2–3 participants to read one success criterion they wrote and say whether it is measurable
  or still fuzzy. This surfaces the distinction for the whole room without singling anyone out

---

### Highest-risk segment: Segment 3 — Idea too vague to interrogate

**The failure mode:** A participant's raw idea is so underspecified that Claude's questions loop
without surfacing concrete answers. The participant cannot answer "for whom?" or "what would
success look like?" because they have not thought that far. The interrogation spins.

**Why this is the highest-risk segment:** A participant stuck here has no PRD and cannot proceed
to decomposition. If left unaddressed, they go into Building with nothing to run against.

**Recovery move:** Pull up their situation as a live example for the whole cohort — with their
permission, or with an anonymized version if they decline. Treat the vagueness as a finding, not
a failure: "This is what a too-vague idea looks like in interrogation. Let's workshop it."

Run a live interrogation with the cohort: you play the participant, Claude plays the interviewer,
and the room watches you answer the questions honestly, including "I do not know yet." Model what
it looks like to name an open question and keep moving rather than stalling for the perfect answer.

After the live example, return the participant to their interrogation. In most cases, watching the
live run is enough to unblock them — they see that "I do not know yet" is a valid and useful
answer, not evidence that their idea is bad.

If a participant truly cannot form a project idea at all, offer them a worked example: "Pick a
small tool you have always wanted but does not exist — something that would save you thirty minutes
a week." This is almost always enough to produce raw material.

---

### Segment 4 — Movement 3: Decompose the PRD into an executable task list (35 min)

**What:** Participants take their filled PRD and break it into an ordered, individually-verifiable
task list using the plan/decomposition worksheet.

Teach (5 min):
- The goal: a task list an autonomous run can act on, one task at a time
- The granularity rule: each task is one sitting, one verification step. If you cannot finish and
  verify it in one session, split it
- What a verification note is and is not: a command you run, a file that exists, a behavior you
  can trigger — not "Claude says it is done"
- Dependency order: mark what blocks what, group into waves

Participants work (25 min):
- Circulate and look for tasks that are still too large (typically anything framed as "implement
  the whole X") — ask: "What would you check to know this is done?" If the answer has multiple
  unrelated parts, it needs to be split
- Also look for verification notes that are still vague ("it works," "it looks right"). Coach
  toward something you can check without interpretation

Check-in (5 min):
- Ask participants to count how many tasks they have that do not yet have a verification note.
  Those are the outstanding items before they move to the completion checklist

---

### Segment 5 — Completion checklist + wrap (15 min)

**What:** Participants self-attest completion using the completion checklist, then close the session.

Work (10 min):
- Participants work through the [completion checklist](../../../site/docs/workshops/planning/completion-checklist.md)
  independently. Each item is a yes/no check against the artifact
- Anyone who finds an unchecked item: that is their remaining work, not a failure. Name it, note
  it, and address it before Building

Wrap (5 min):
- Share the homework prompt: generate as many specs and plans as possible before the Building
  session. Reps build speed; the capacity-tracking sheet lets them see it
- Confirm the hand-off: bring the completion checklist and task list to Building. The first
  exercise in Building works from the task list directly

---

## Facilitation do / don't

**Do:**
- Coach the interrogation technique — model what "I do not know yet" looks like as a valid answer
- Walk the room during work time and check progress, especially on success criteria
- Use the "vague idea" failure mode as a teaching moment for the whole cohort, not a problem
  to solve in private
- Enforce the done condition in Segment 3: measurable success criteria and explicit out-of-scope
  are not optional
- Let participants reach their own answers. Ask questions; do not supply the answer

**Don't:**
- Write the PRD for a participant who is stuck — interrogate them instead, or pull them into
  a live example with the whole cohort
- Accept "it feels right" or "users will enjoy it" as a success criterion without pushing back
- Skip the check-in moments at the end of Segments 3 and 4 — they surface shared confusion that
  would otherwise stay private
- Let Segment 2 (raw capture) run long. If someone is still writing at the 15-minute mark,
  tell them to stop where they are — incomplete raw material is enough to interrogate
- Rush the decomposition segment to make up time. A participant who leaves with a PRD but no
  verified task list cannot start Building

---

## Common questions from participants

**"Can I use a project I have already started building?"** Yes. Use the raw capture to describe
the current state and what the next slice of work is. The interrogation will surface whether the
existing work has a clear success criterion or has been built without one.

**"My idea is too big for this."** Good — that is exactly what decomposition is for. The
Planning part does not require the plan to cover the full project; it requires that *this slice*
be execution-ready. Scope the PRD to the first meaningful chunk the participant can run in Part 2.

**"I don't have Claude Code / my setup isn't working."** The interrogation step works in the
Claude desktop app or web interface; the terminal is not required until Building. Redirect them
to the Getting Started page for setup help and continue with the session — they can run the
interrogation in the browser while fixing their environment before Building.

**"Can my success criterion be 'the thing is built'?"** No. That is a scope statement, not a
criterion. Push back: built to what standard? Verifiable how? Who can check it, and what do they
look for?
