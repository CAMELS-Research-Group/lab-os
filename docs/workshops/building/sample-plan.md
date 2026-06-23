# Building — sample plan

**Status:** draft
**Audience:** Participants who reach Building without a plan of their own, and facilitators who need a
fallback. ENG-tier, not published to the handbook site.

**Using this plan.** This file ships in your fork. From the fork's root, start Claude and point it
here — for example: *"Read `docs/workshops/building/sample-plan.md` and run Task 1 end to end, then
stop."* Treat each task as one hand-off; verify with the gate before moving on. You read this from the
CLI, not the handbook site — it is a plan to execute, not a page to browse.

---

If you reached the Building part without an execution-ready plan of your own — for example, you have
just set up your workspace and forked this handbook — use this plan. It is a small, low-stakes plan
you run against **your own fork of this repository**, so you can practice the three execution modes
(monitored sequential → autonomous → scaling) on work that actually changes something you own.

It is also a worked example of what *execution-ready* looks like: a short PRD, then three tasks that
are each small enough to finish and verify on their own, each with a single verification command and
its dependencies marked. Read it the way you would read a plan you wrote yourself.

> **Prefer your own project.** This plan exists so nobody is blocked. If you brought a real project
> and a plan of your own, run *that* — it teaches the same skills on work you care more about. The
> Building participant page (`../../../site/docs/workshops/building/index.md`) is the source of truth
> for the method.

---

## What you build

Three changes to your fork, in plain terms:

1. **Re-home the fork's identity** — make the handbook deploy as *yours*, not a copy of the upstream.
2. **Brand it** — give the handbook your own name, colour, mark, and front-page copy.
3. **Add a planning surface** — a small backlog-generation workflow page plus the files it writes to,
   so your fork has a place to turn raw ideas into tracked work.

---

## PRD

**Problem.** A participant who finished setup but has no plan of their own has nothing to hand to an
autonomous run. Practising the execution modes needs a plan whose tasks are real, small, and
individually verifiable.

**Success criteria.**

- You hand at least one task to an autonomous run, then prove it with the single gate command — not
  the agent's self-report.
- You hand off a two-task wave and verify **each** result separately.
- After each run you can point to the diff and say what actually changed.

**Scope.**

- **In:** three changes to your own fork, listed above.
- **Out:** the upstream's deployment, anyone else's fork, and any production planning tool. This is a
  teaching echo of a planning workflow, not a real backlog system.

**Constraints.**

- Every task is verifiable by one command whose exit code is the source of truth.
- No task is larger than one autonomous sitting; if one starts to sprawl, that is a finding — split it.
- Node ≥ 20. Run `npm ci` once inside `site/` before your first build.

**Notes / assumptions.** You forked `WatsonWBlair/lab-os` to your own GitHub account and cloned it
into your workspace. The strings you replace in Task 1 are the **upstream's** identity, which your
fork inherited — replacing them is the point, not a leak.

---

## The verification gate

One command, run from the repository root, exit code is the truth:

```shell
cd site && npm run build
```

The site config sets the build to **throw on any broken link or broken Markdown link**, so a passing
build is a real check: it proves every page still resolves. It does **not** prove your branding looks
good or your copy reads well — that is the "green is not reviewed" lesson, and you will feel it
directly in Task 2, where the build passes but only your eye can tell the colour is right.

---

## The wave map

```
Task 1  Re-home identity         (no dependencies)        ── run first
   │
   ▼
Task 2  Branding                 (depends on Task 1)   ┐
                                                       ├── run as a parallel wave
Task 3  Planning surface         (independent)         ┘
```

The dependencies are honest, and they are the point of the sequencing exercise:

- **Task 2 depends on Task 1** because both edit `site/docusaurus.config.ts`. Running them at the same
  time would collide. Task 1 has to land first.
- **Task 3 depends on nothing.** It touches a different set of files entirely. It *could* run first;
  scheduling it alongside Task 2 just makes a clean two-task wave for the scaling exercise.

So Tasks 2 and 3 form a wave you hand off together once Task 1 is done — they touch **disjoint files**,
so they will not interfere, and you verify each one on its own.

---

## Task 1 — Re-home the fork's identity

**Files:**
- Modify: `site/docusaurus.config.ts`
- Modify: `README.md`

**Depends on:** —
**Agent-suitable:** yes — this is the monitored-sequential / first-autonomous rep.

**Spec:** [PRD → Scope](#prd), [the verification gate](#the-verification-gate)

Context: a fresh fork still declares the upstream as its owner and deploy target. Until you re-home
it, "Edit this page" links and the published URL point at the original, not at you.

**Acceptance:**

- In `site/docusaurus.config.ts`, the deploy identity points at **your** GitHub account and fork:
  `organizationName`, `projectName`, `url`, `baseUrl`, the `editUrl` base, and the navbar and footer
  GitHub links no longer name `WatsonWBlair` / `watsonwblair` / the upstream repo.
- In `README.md`, the top identity line names your own workspace rather than the upstream owner. (The
  `WatsonWBlair/lab-os` references inside the "How repos consume it" examples are about the upstream
  on purpose — leave those.)
- No `WatsonWBlair` or `watsonwblair` literal remains in `site/docusaurus.config.ts`.
- The site still builds.

**Verification:**

```shell
cd site && npm run build && cd .. && ! grep -nE 'WatsonWBlair|watsonwblair' site/docusaurus.config.ts && echo OK
```

**Commit:** `chore: re-home fork identity to my account`

---

## Task 2 — Branding and personalization

**Files:**
- Modify: `site/docusaurus.config.ts`
- Modify: `site/src/css/custom.css`
- Modify: `site/src/pages/index.mdx`
- Modify: `site/static/img/logo.svg`
- Modify: `site/static/img/favicon.ico`

**Depends on:** 1
**Agent-suitable:** yes

**Spec:** [PRD → Scope](#prd), [the verification gate](#the-verification-gate)

Context: this is where you make the handbook visibly yours. It is also a clean "green is not reviewed"
moment — the build cannot see a colour, so the gate passes whether or not the branding actually looks
right.

**Acceptance:**

- The handbook `title` and `tagline` in `site/docusaurus.config.ts`, the navbar title, and the footer
  name your own project, not `lab-os`.
- `site/src/css/custom.css` sets a primary-colour palette of your own (both the `:root` and the
  `[data-theme='dark']` blocks), distinct from the upstream `#5b54e8`.
- `site/src/pages/index.mdx` front-page copy is yours — the headline and intro paragraph describe
  your project, not the upstream's.
- The logo and favicon are your own mark (a deliberate placeholder is fine), not the upstream's.
- The site builds.

**Verification:**

```shell
cd site && npm run build && cd .. && ! grep -nE "title: 'lab-os'" site/docusaurus.config.ts && ! grep -nE '#5b54e8' site/src/css/custom.css && echo OK
```

The greps prove the upstream defaults are **gone** — not that your replacements look good. Open the
built site and judge that with your eyes. That gap is the lesson.

**Commit:** `feat: brand the handbook for my project`

---

## Task 3 — Add a planning surface

**Files:**
- Create: `site/docs/planning/backlog.md`
- Modify: `site/sidebars.ts`
- Create: `BACKLOG.md`
- Create: `templates/backlog-item.template.md`

**Depends on:** — (independent; schedule it in the wave with Task 2 for the scaling exercise)
**Agent-suitable:** yes

**Spec:** [PRD → Scope](#prd), the Planning part (`../../../site/docs/workshops/planning/index.md`)

Context: this gives your fork a place to turn a raw idea into a tracked, sized piece of work — a small
echo of the Planning part's idea-to-plan flow, living in your own handbook.

**Acceptance:**

- `site/docs/planning/backlog.md` is a new handbook page that documents a repeatable
  idea-to-backlog-item workflow: how to take a raw idea and turn it into a structured item (problem,
  who it helps, rough size, a sketch of how you would verify it done). It **links** the Planning part
  rather than restating it.
- The new page is registered in `site/sidebars.ts` (a wrong id fails the build, so the gate proves the
  wiring is correct).
- `BACKLOG.md` exists at the repository root with an index section and an items section, and at least
  one worked example item.
- `templates/backlog-item.template.md` gives the fill-in shape a backlog item follows.
- Optionally, the page includes a copy-paste prompt that has Claude interview you and draft an item in
  the template's shape — the on-thesis version of "a surface that facilitates backlog generation."
- The site builds with no broken links.

**Verification:**

```shell
cd site && npm run build && cd .. && test -f BACKLOG.md && test -f templates/backlog-item.template.md && test -f site/docs/planning/backlog.md && echo OK
```

**Commit:** `feat: add a backlog-generation planning surface`

---

## How to run this across the exercises

This plan is built to carry you through all five Building exercises
(`../../../site/docs/workshops/building/exercises.md`):

| Exercise | What you do with this plan |
|---|---|
| 1 — Plan to roadmap | This plan **is** the roadmap. Read each task and confirm you can state its "done when" before running anything. |
| 2 — Sequencing & dependencies | Study [the wave map](#the-wave-map). Confirm *why* Task 2 depends on Task 1 and Task 3 does not — and why that makes Tasks 2 + 3 a safe parallel wave. |
| 3 — Worktree isolation | Create a throwaway worktree off your fork before any autonomous run, so a bad run can't damage your main branch. |
| 4 — First autonomous execution | Hand off **Task 1** end to end. Let it run. Then run the gate yourself, unpiped, and read `git diff HEAD` — not the agent's summary. |
| 5 — Scaling execution | Hand off **Tasks 2 and 3** as one wave. Verify **each** separately — do not run the gate once and call both done. |

A run where the gate fails and you catch it is a success: you just proved verify-don't-trust. Record
it and continue from where it failed.
