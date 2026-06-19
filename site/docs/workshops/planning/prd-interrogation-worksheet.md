---
title: PRD Interrogation Worksheet
description: The interrogation prompts and PRD skeleton for Movement 2 of the Planning part — let Claude ask the clarifying questions until every section is concrete.
---

# PRD Interrogation Worksheet

This worksheet is for **Movement 2** of the [Planning part](./index.md): letting Claude interrogate
you into a solid PRD instead of you front-loading a finished spec.

You bring your raw idea (from the pre-work or from Movement 1 journaling). Claude asks the
clarifying questions. You fill in the PRD skeleton below as the answers emerge. By the end, you
have a written PRD you can hand to Movement 3 for decomposition.

---

## How to use this worksheet

1. Open a Claude session alongside this worksheet.
2. Paste your raw idea and one of the **interrogation prompts** below.
3. Answer each question Claude asks honestly. Write the answers directly into the PRD skeleton.
4. Keep going until no section of the skeleton has a vague or missing answer.
5. Check the **done condition** at the bottom before moving on.

Work through the PRD skeleton one section at a time. You do not need to fill them in order — follow
where the conversation leads.

---

## Interrogation prompts

Choose one to start the dialogue. All three reach the same place; pick the one that matches how
you tend to think.

---

**Prompt A — Open interrogation**

Paste your raw idea, then add:

> "I have described a project idea above. I want you to play the role of a demanding product
> requirements interviewer. Ask me clarifying questions — one or two at a time — until you are
> confident you can write a precise product requirements document covering: the exact problem and
> who has it, measurable success criteria, explicit in-scope and out-of-scope items, and the
> binding constraints. Do not write the PRD yet. Start by asking me whatever you most need to
> clarify."

---

**Prompt B — Section-by-section**

Paste your raw idea, then add:

> "I have described a project idea above. Walk me through a product requirements interview,
> section by section: first surface the problem and who has it, then help me define measurable
> success criteria, then draw the scope boundary (what is explicitly in and what is explicitly
> out), then identify the binding constraints. Ask me clarifying questions in each section until
> the content is specific and falsifiable before moving on. One or two questions at a time."

---

**Prompt C — Challenge mode**

Paste your raw idea, then add:

> "I want you to stress-test this project idea as a requirements interviewer. Challenge every
> claim I make. If a success criterion is not measurable, push back and make me make it
> measurable. If scope is unclear, ask me to draw the line. If I name a constraint I have not
> examined, ask me to justify it. Keep asking until the requirements are concrete enough that a
> developer who has never spoken to me could act on them. Start with whatever you think is the
> weakest part of what I have described."

---

## Guidance for the dialogue

- **Answer honestly.** "I do not know yet" is a valid answer — it surfaces an open question or a
  constraint you have not examined. Write it into the PRD as an open question rather than guessing.
- **Do not front-load.** If you already have a strong opinion about how to build it, set it aside
  for now. The interrogation is about surfacing what the requirements actually are, not defending
  a solution.
- **Push back if a question feels off.** If Claude asks something that does not apply to your
  project, say so and explain why — that explanation often surfaces a relevant constraint.
- **Do not stop early.** The most common mistake is stopping when the PRD *feels* done. Keep going
  until each section has a specific, falsifiable answer — not a vibe.

---

## PRD skeleton

Fill this in as the dialogue progresses. Each section has a prompt to guide what belongs there.

---

### Problem

*What is broken, for whom, and why does it matter? A concrete statement, not a solution pitch.*

> What problem does this project solve?
> Who has this problem — specifically?
> Why does it matter to them? What does the problem cost them (time, effort, errors, risk)?
> What is the current workaround, and why is it insufficient?

**Your answer:**

&nbsp;

---

### Success criteria

*Measurable and falsifiable — something you can check, not something you can feel.*

> What would you observe — specifically — that would tell you this project succeeded?
> If you checked in three months, what would be different or better in a way you could measure?
> For each criterion you name: can you put a number or a yes/no check on it? If not, push until
> you can.

**Criteria (list each one):**

- [ ] &nbsp;
- [ ] &nbsp;
- [ ] &nbsp;

*Each criterion must be falsifiable: "users complete the core task in under two minutes" is
checkable. "It feels fast" is not.*

---

### Scope

*What is explicitly in — and what is explicitly out. The out-of-scope list is as important as the
in-scope list.*

> What are the two or three things this project must do to satisfy the success criteria?
> What are the things that sound related but are NOT in this version?
> Where is the edge — what does "done for now" look like versus "done eventually"?

**In scope:**

- &nbsp;
- &nbsp;

**Out of scope (explicit):**

- &nbsp;
- &nbsp;

*If you cannot name at least one thing that is explicitly out of scope, the boundary is not drawn
yet. Go back and ask the question again.*

---

### Constraints

*What bounds the work — hard limits, not preferences.*

> What are you working with that you cannot change (time, budget, data access, tools, approvals,
> infrastructure)?
> What would block you from starting tomorrow — and is that a real constraint or an assumption?
> What dependencies does this project have on people, systems, or resources outside your control?

**Constraints (list each one):**

- &nbsp;
- &nbsp;

---

### Open questions

*Things you do not know yet that the project depends on. Better to name them now than discover
them mid-execution.*

**Open questions:**

- &nbsp;

---

## Done condition

The interrogation is done when you can answer **yes** to every item below by inspecting your
PRD skeleton, not by judgment.

- [ ] **Problem** is a concrete statement of what is broken and for whom — not a description of
      a solution.
- [ ] **Success criteria** are each measurable and falsifiable. You could hand them to someone
      who was not in the conversation and they would agree on whether each criterion is met.
- [ ] **Scope** has at least one item explicitly in and at least one item explicitly out. The
      out-of-scope list makes clear what is not being built in this version.
- [ ] **Constraints** names the real limits on the work — not wishes, not preferences, but things
      that actually bound what you can do.
- [ ] No section is vague or missing. If a section has a placeholder or "TBD," that is an open
      question — name it as one and decide whether it blocks execution.

When every item is checked, your PRD is ready. Take it to the
[plan/decomposition worksheet](./plan-decomposition-worksheet.md) to break it into tasks.

If you are working through the completion checklist later, the "execution-ready" bar for the PRD
section maps directly to these done-condition items: specific, falsifiable success criteria and an
explicit out-of-scope list are the non-negotiable parts.
