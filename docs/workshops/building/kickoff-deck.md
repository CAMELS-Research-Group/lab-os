# Building With Claude — kickoff deck

**Status:** draft
**Audience:** Facilitator delivering the Building kickoff. Slide copy + speaker notes for the ~30-minute
talk that precedes the live workspace-setup demo. Mixed cohort (newcomers + assessors); the demo is
follow-along.

**How to read this:** slide numbers match the deck's footer (`building-with-claude-kickoff.pptx`, 27
slides). Act-divider slides are included so you never lose your place. Each content slide has
**On-slide** (the lede + sparse bullets that appear — keep slides light) and **Notes** (what you say,
~30–90 s). Times are guides; the talk lands near 30 min, with the demo taking the rest of the session.

| Act | Title | Slides | ~Time |
|---|---|---|---|
| 0 | Open | 1–2 | 2 min |
| 1 | Development Foundations | 3 (divider) – 5 | 5 min |
| 2 | Build | 6 (divider) – 9 | 6 min |
| 3 | Verify | 10 (divider) – 13 | 7 min |
| 4 | Review | 14 (divider) – 17 | 5 min |
| 5 | Project Setup | 18 (divider) – 21 | 5 min |
| — | Transition | 22 (divider) – 23 | 1 min |
| — | Live Demo | 24 (divider) – 27 | rest of session |

---

## Act 0 — Open

### S1 · Title

**On-slide:**
- **Building With Claude**
- Agentic, spec-driven development — the lab's way
- A 30-minute primer on trusting autonomous coding agents, then a hands-on workspace setup
- *CAMELS Research Group*

**Notes:** Welcome. The next half hour is how we build software here — not by hand-coding every line,
but by handing work to an AI agent that carries it out, and knowing how to trust what comes back. Then
we set up your workspace live, together. You'll leave with a working agentic workspace on your own
machine and the mental model for using it.

### S2 · The deal

**On-slide:**
- *Half talk, half hands-on — you leave with a working setup, not just notes.*
- First 30 minutes: how we hand work to an AI agent and trust the result
- Then we set up your workspace live — you do it on your own machine, alongside me
- Have ready: a Claude subscription, Git, the GitHub CLI, and Claude Code

**Notes:** Two halves. First I talk: the concepts. Then we all open a terminal and set up your
workspace step by step. You'll need four things installed already: a Claude subscription, Git, the
GitHub CLI, and Claude Code. If you're missing one, flag it now or pair with a neighbour.

---

## Act 1 — Development Foundations

### S3 · Divider — Act 1 · Development Foundations

**On-slide:** Act 1 · Development Foundations — *Why Building*

**Notes (cue):** Frame the half-hour: before we touch a terminal, two ideas — what changed, and where
today sits in the lifecycle.

### S4 · The shift

**On-slide:**
- *Agentic AI doesn't just answer — it does the work, which makes trust the real problem.*
- Old mode: the AI answers questions, you act on them
- New mode: the agent edits files, runs commands, and opens pull requests
- The hard part is no longer capability — it's trusting what it did without re-checking every line
- **The skill we build today: let go, then verify**

**Notes:** You've used AI that answers questions. Agentic development is different: the AI carries out
tasks. Once it's doing the work, the hard problem flips — it's no longer "can it do this" but "can I
trust that it did, without re-reading every line." That's the skill we build today: let go, then
verify.

### S5 · The map

**On-slide:**
- *Today follows the lifecycle's middle — Build, Verify, Review — the loop you'll run all session.*
- Lifecycle: Brainstorm → Specify → Plan → **Build → Verify → Review** → Close
- Build executes the plan; Verify is the automated checkpoint; Review is the human one
- Those three — Build, Verify, Review — are exactly what this session drills
- **You can't learn this from slides: you have to watch the agent be confidently wrong, then catch what the gate didn't**

**Notes:** The lifecycle has seven stages. Today we live in the middle three: Build executes the plan,
Verify is the automated checkpoint, and Review is the human checkpoint. Those three are the loop this
session drills. It's the part you can't learn from slides — the lesson is catching what the gate can't,
the moment the agent is confidently wrong, and you have to see that happen.

---

## Act 2 — Build

### S6 · Divider — Act 2 · Build

**On-slide:** Act 2 · Build — *Hand the work off*

**Notes (cue):** This act is the hand-off — how you delegate cleanly, in which mode, and inside what
safety rail. Verify and Review (the next two acts) are how you earn the trust to let go.

### S7 · The meta-skill

**On-slide:**
- *Delegating well: hand off cleanly, then check — stay across the work without burning your own context window on every step.*
- Micromanaging: correcting every step — you never actually delegated
- Blind trust: taking the agent's word and shipping its mistakes
- The middle path: hand off, let it run, then verify with something that can't lie
- **Build sets up the work; Verify and Review are how you trust it**

**Notes:** The one thing to remember, flanked by two failure modes. Micromanaging: you hover and
correct every step — you never actually delegated. Blind trust: you take the agent's word and ship its
mistakes. The middle path is to hand off cleanly, let it run, then check. Build is the hand-off; the
next two stages, Verify and Review, are how you earn the trust to let go.

### S8 · Three execution modes

**On-slide:**
- *You move from close supervision to full delegation as you earn trust in your gate.*
- Monitored sequential — one task at a time, you check each result (how you learn what good looks like)
- Autonomous — hand off a whole task, don't intervene; you verify the one result when it's done
- Multi-agent — hand off a batch in parallel; the coordinating agent verifies each, you own the gate it clears
- **You don't start in third gear — you earn it by proving your gate holds**

**Notes:** Three gears; you shift up as you earn trust. Monitored: one task at a time, you check each
result — how you learn what good output looks like. Autonomous: hand off a whole task, don't
intervene, then you verify the one result. Multi-agent: hand off a batch in parallel and the
coordinating agent verifies each — your job shifts to owning the gate it clears and sampling the diffs.
You don't start in third gear — you earn it by proving your gate holds.

### S9 · Worktree isolation

**On-slide:**
- *Autonomous runs happen in a throwaway worktree, so a bad run can't damage your work.*
- A worktree is a separate working copy on a disposable branch
- A run that goes wrong leaves your main branch untouched — you just delete it
- Set up with `git worktree add`; see them with `git worktree list`
- **Containing the downside is what makes letting go feel safe**

**Notes:** The safety rail that makes letting go feel OK. A git worktree is a separate working copy on
a throwaway branch. Autonomous runs happen there, so a run that goes off the rails leaves your main
branch untouched — you delete the worktree and move on. Containing the downside is what lets you
actually let go.

---

## Act 3 — Verify

### S10 · Divider — Act 3 · Verify

**On-slide:** Act 3 · Verify — *Prove it yourself*

**Notes (cue):** The automated checkpoint. What a gate is, why green isn't enough, how to verify for
yourself, and the predictable ways runs fail.

### S11 · Quality gates — green isn't reviewed

**On-slide:**
- *A quality gate is the single pass/fail command that makes delegation safe — but a pass means the
  tests ran, not that the work is right.*
- One command — tests, a build, or a typecheck — exit code is the source of truth, no interpretation
- Write it down before any autonomous run: no gate, no autonomy
- Green means 'the tests passed' — not 'the change is correct'
- One agent writing both code and tests makes green a closed loop that only confirms its own assumptions
- **Green is necessary, never sufficient — Verify clears the gate, Review the rest**

**Notes:** Two ideas, one slide. First, what a gate is: a single command — tests, a build, a typecheck
— that returns pass or fail with no argument, exit code zero or not. Write it down before you hand
anything off; no gate, no autonomy, because otherwise your only check is reading all the code yourself,
which defeats the point. Second, the trap: green means the gate passed — that's all, not that the work
is right. And when the same agent writes the code and the tests, the tests only confirm its own
assumptions agree with themselves — a closed loop. Green is necessary, never sufficient. Verify clears
the gate; Review, next, clears what green can't speak to.

### S12 · Verify, don't trust

**On-slide:**
- *Let the coordinating agent re-verify at scale — but the gate result is the evidence, never the agent's word.*
- Have the coordinating agent re-verify every sub-agent result — reports turn optimistic across context boundaries
- The gate runs unpiped — piping swallows the exit code and makes red look green
- You still read `git diff HEAD` on what matters, not the agent's summary of it
- **Scope creep hides in the diff; it's invisible in the summary**

**Notes:** The honest division of labor: the coordinating agent runs the gate on every task — nobody's
checking all of them by hand. Your job is that it runs unpiped (piping hides the exit code, so a red
gate reads green — I did exactly that setting this up, caught it, re-ran clean) and that you still read
the actual diff on what matters, not the agent's prose summary. Scope creep lives in the diff and is
invisible in the summary.

### S13 · The failure-mode catalogue

**On-slide:**
- *Autonomous runs fail in predictable ways — and catching a failure is the goal, not the
  embarrassment.*
- Scope creep — does more than you asked
- Claims-done-but-isn't — reports done, the gate says no (the headline lesson)
- Won't-let-go (you, hovering) 
- Environment Stalls
- Verification Errors
- **A caught failure is the system working — the worst outcome is an uncaught one**

**Notes:** Five ways runs go wrong — you'll meet some today. Scope creep: does more than you asked.
Claims-done-but-isn't: reports done, the gate says no — the headline. Won't-let-go: that's you,
hovering. Environment stalls. Task-too-big: if you can't state one "done when," split it. The reframe:
a caught failure is the system working as designed — the worst outcome is an uncaught one.

---

## Act 4 — Review

### S14 · Divider — Act 4 · Review

**On-slide:** Act 4 · Review — *Read it as an outsider*

**Notes (cue):** The human checkpoint — what the gate structurally can't see, how to read it, and why
it's not optional.

### S15 · Where verification stops

**On-slide:**
- *Some failures are invisible to any gate — those are exactly what review exists to catch.*
- Credential and live-data paths: mocked tests never touch them
- Failure-recovery and edge cases: rarely exercised by the suite
- Intent vs. implementation: does it do what was actually meant?
- **Green clears the gate; a human clears what the gate can't reach**

**Notes:** Verification has a ceiling. Anything behind a real credential or a live data path is never
hit by mocked tests. Failure recovery — what happens when the network dies mid-call — is rarely in the
suite. And no gate judges whether the code does what was actually meant. None of that is a bug to fix
in the gate; it's the boundary of what automation can see. Past it, a human reads the work. That's
review.

### S16 · Review like an outsider

**On-slide:**
- *Review reads what's actually there, as an outsider would — so run it in a context that never saw
  the work being built.*
- Have Claude review in a fresh context — implementation intent can't color a reviewer that never built the thing
- If understanding the change needs context only the author's session holds, that's a finding
- Self-review runs softer — hold work you (or your agent) produced to a higher bar, and declare authorship
- **Write a review standard: what you inspect before you trust the output**

**Notes:** Review is the human checkpoint — but you can run it in a fresh Claude context that never
built the work, so implementation intent can't color it. Read what the code says, not what was
intended — memory papers over real gaps. If a reader would need context they don't have, that's a
finding, not a free pass. Authorship softens scrutiny exactly when you need it most, so work you or
your agent produced gets the higher bar — declare it. You own the review standard: the specific things
inspected before any autonomous output is accepted.

### S17 · Review is load-bearing

**On-slide:**
- *Pre-merge review isn't optional, and it isn't one pass — it's a loop until nothing load-bearing is
  open.*
- Each pass surfaces findings — fix them now, or route to a tracked issue; never silently defer
- Re-review until the bar is clear, and record findings and their resolution on the PR
- A second perspective hardens it — a teammate, or the lab's automated reviewer's outside read
- **You'll run this loop today: review → fix → re-check, then merge**

**Notes:** Review is load-bearing — it's the step that catches what the gate structurally can't, so
it's not a nicety you skip under time pressure. And it's iterative: a pass surfaces findings, you
resolve each — fix it now, or route it to an issue with a reason, recording findings and their
resolution on the PR — then review again until the merge bar is clear. A second set of eyes makes
it stronger: a teammate, or the lab's automated reviewer that posts an outside read on open PRs but
never merges. You'll run exactly this loop in the demo.

---

## Act 5 — Project Setup

### S18 · Divider — Act 5 · Project Setup

**On-slide:** Act 5 · Project Setup — *Your workspace + the rules*

**Notes (cue):** Shift from concepts to the thing they're about to stand up: the working surface, the
rules it loads, and the fact that the lab runs those rules on itself.

### S19 · Your working surface

**On-slide:**
- *Your workspace is your lab-os fork — layered context files plus the lab's rules, loaded every session.*
- Three `CLAUDE.md` layers: global (you) → dev-home/fork (workspace) → project (nested repo) — specific wins
- The rules live in your fork; `git pull upstream` keeps them current (junction stays as the multi-repo power-user path)
- Two plugins add capabilities: superpowers and pr-review-loop
- **This is the surface you'll stand up in the demo**

**Notes:** This is what you're about to set up — your own fork of lab-os as your dev home. Three
layers of context files Claude reads every session: who you are (global), your dev-home/fork
(workspace), each nested project (repo) — most specific wins. The rules live natively in your fork;
`git pull upstream` keeps them current. (If you'd rather run many repos under one home, the junction
model is the power-user alternative.) Plus two plugins. This is the surface; in a minute you stand it
up.

### S20 · The four rules

**On-slide:**
- *Four lab-wide rules load into every session so the agent works the way we do.*
- 01 Workflow — commit style, pull requests, and the bar a change clears to merge
- 02 Data protection — license-restricted data never enters a repo; no secrets or binaries
- 03 Logging — record decisions and their why, not a diary; merged entries are immutable
- 04 Docs — every fact has exactly one owning document

**Notes:** The rules every session loads. Workflow: commit and PR shape, and the merge bar. Data
protection: license-restricted, human-subject data never enters a repo — the one to internalise first.
Logging: record decisions and their why, not a diary; merged entries are immutable. Docs: every fact
has one owning document. You don't memorise these — they load and the agent applies them.

### S21 · We run it on ourselves

**On-slide:**
- *The lab runs these rules on its own repo — including the review step you just saw.*
- Three CI checks lint every PR: log-lint, docs-budget, merge-bar-check
- An automated reviewer posts an outside-perspective read on open PRs — but never merges
- Lab tooling ships through a public plugin marketplace
- **The way we teach you to work is the way the lab actually works**

**Notes:** Credibility, briefly — this matters most to anyone evaluating how the lab works. The
handbook repo runs its own rules in CI — three checks that lint every PR. A separate automated
reviewer posts an outside-perspective read on open PRs — the Review step, automated — but never
approves or merges; a human owns that. And lab tooling ships through a public marketplace. The way
we teach you to work is the way the lab actually works.

---

## Transition

### S22 · Divider — Stand up your workspace

**On-slide:** Transition · Stand up your workspace — *From the concepts to your own setup*

**Notes (cue):** Beat change. Concepts are done; the rest is hands-on. Get everyone into a terminal.

### S23 · Stand up your workspace

**On-slide:**
- *Time to build — one pasted prompt sets up your whole workspace, with you confirming each step.*
- Have installed: Claude Pro/Max, Git, GitHub CLI, Claude Code
- One prompt has Claude interview you, then set everything up
- It confirms before every file write — you stay in control
- **Follow along on your own machine**

**Notes:** That's the concepts. Now we build. Open a terminal. You need those four things installed.
I'll paste one prompt that has Claude interview me and set everything up, confirming before each file
write. Do it with me on your own machine.

---

## Live Demo — follow-along workspace bootstrap

### S24 · Divider — Live Demo

**On-slide:** Live Demo · Follow-along workspace bootstrap — *Set it up, then run the Build → Verify →
Review loop*

**Notes (cue):** Switch to the terminal. From here you narrate while the cohort runs each step on their
own machines.

### S25 · Bootstrap — run of show

**On-slide** (source of truth: the handbook's Getting Started page):
- *You paste one prompt; it forks lab-os and configures everything, confirming each step. The fork is the prompt's own first job — you don't set it up beforehand.*
1. Prereq check: `claude` / `git` / `gh --version`; confirm you're logged in
2. Paste the one bootstrap prompt — it runs the rest, confirming each step
3. → forks lab-os to your account + clones the fork as your dev home (clone = fallback)
4. → personalizes global config + seeds the dev-home one (rules native, no junction)
5. → cleans the fork: fresh project log, drops the design history + the site
6. → re-homes your plan to the fork's `_plans/`, then installs the two plugins
7. → verifies: rules resolve, remotes right, no placeholders
8. Then your first `git worktree` in the project — your autonomous-run surface

**Notes:** Source of truth is the handbook's Getting Started page. The point participants miss: they
paste ONE prompt and Claude forks + clones lab-os itself (step 3) and reads the templates from that
fork — nobody sets it up by hand first. Confirm before every fork/clone/delete/move. `git pull
upstream` is how the fork stays current — no junction to manage. You root your work at the dev home
(the fork); the verify pass proves it took — a fresh session there answers "what are the lab's
commit-message rules?" from `01-workflow.md`.

### S26 · The payoff — run the loop

**On-slide:**
- *Now apply the whole talk: hand the sample plan to the agent, then Verify and Review each result.*
- Point Claude at `docs/workshops/building/sample-plan.md` in your fork
- Build: hand off Task 1 autonomously; Verify: run the gate yourself, unpiped
- Review: read the diff against your standard before you'd merge
- Scale: hand off Tasks 2 + 3 as a wave → verify and review each
- **Task 2's build passes, but only your eye confirms the branding — the whole loop, live**

**Notes:** This ties the talk to the doing — the full Build → Verify → Review loop. Hand off Task 1,
run the gate yourself unpiped, then read the diff against your review standard before you'd merge. Then
Tasks 2 and 3 as a wave, each verified and reviewed. The "green is not reviewed" beat lands live on
Task 2's branding — the build passes, but only your eye confirms it. The Task 3 answer key is on hand
if a run needs a known-good reference.

### S27 · Backup — if the demo breaks

**On-slide:**
- *A broken run is teaching material — narrate it, recover, and keep moving.*
- A failed autonomous run is content — walk the failure-mode catalogue, then recover
- Setup stalls → pair people up and keep the room moving
- Short on time → run only Task 1 live; Tasks 2 + 3 become the first homework

**Notes:** A failed run is the most instructive moment available — don't rush past it. Don't let one
machine's setup eat everyone's time. If you're short on time, Task 1 live is enough; the rest becomes
the first homework rep.
