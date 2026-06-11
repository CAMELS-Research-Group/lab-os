# Working with Claude — lab methods and best practices

Methods the lab has established **in practice** — most of them earned by hitting a failure mode and
correcting it. This is the "how we actually work with Claude effectively" companion to the hard rules in
`.claude/rules/`. Read it during onboarding; come back to it when a workflow feels off.

The operating *philosophy* (PRD-first, pushback, reversibility, review mode) lives in the global
`CLAUDE.md` template. This doc is about the *methods* — the repeatable workflows and the traps to avoid.

---

## 1. Process before code

- **Brainstorm before building.** For any creative work — a new feature, component, or behavior change —
  run the brainstorming step first (the `superpowers:brainstorming` skill). Process skills (brainstorm,
  debug) decide *how* to approach; implementation skills come second.
- **PRD before a non-trivial build.** Problem · Success criteria (measurable) · Scope (in / explicitly out)
  · Constraints · Plan (phased) · Open questions. No keys touched until sign-off. Silence isn't approval.
- **Check what exists first.** Search the lab repos, the upstream lineage, and installed tooling before
  proposing custom work. Extending something close-enough beats a new build.

## 2. Code-free implementation plans

Plans specify **what** the implementation must satisfy, not **how** to write it. Per task, six elements:

**Files** (Create / Modify, exact paths) · **Depends on** (task numbers) · **Spec** (link to the design-doc
section) · **Acceptance** (bulleted behaviors the implementation must demonstrate) · **Verification** (the
exact command the implementing agent runs to confirm done) · **Commit** (conventional-commit subject).

**No literal code, no test code, no TDD walkthroughs.** The behaviors *are* the test surface; the
implementing agent owns function names, fixtures, and assertions. The only code-blocks allowed are short
shell commands in `**Verification:**` lines.

**Why it pays off:** code-heavy plans conflate plan-authoring with implementation-authoring, rot faster
(literal API names drift before the code lands), and discourage the implementer from owning their tests.
Contract-and-behavior plans survive re-runs after a fix-round, which is exactly when a plan gets
re-executed. This **overrides** `superpowers:writing-plans`' show-code default.

## 3. Subagent-driven development

- **The plan is the human→agent handoff artifact.** Decomposition into Shape-B tasks
  (Context / Files / Depends / Spec / Acceptance / Verification / Commit) is *daytime* work — don't hand an
  agent a design spec and expect it to also decompose.
- **Subagents discard context and return only their report.** Design tasks so a subagent needs only its
  own brief to act, and so its returned report is the thing you actually need — not a transcript.
- **A backlog scaffolds the fan-out.** Useful schema, reusable across repos: a table
  (`Done | ID | Task | Phase | Agent | Depends on`), stable `PREFIX-N` IDs, an
  `agent-suitable: yes | partial | no` classification, a dependency DAG (`#N` same-plan, bold cross-plan),
  and **git-authoritative completion** (a task is done when the commit exists, not when an agent says so).

## 4. Verification discipline

The single most important lesson: **an agent's self-report is not evidence.**

- **Beware the optimistic narrator.** An agent will report success it didn't achieve. Re-run the gate
  yourself; don't trust the digest. (When the self-report *matches* independent evidence, that's the win
  condition — but you only know by checking.)
- **Green ≠ reviewed.** A passing gate proves "tests pass," not "the guarantees those tests claim are
  actually tested." Watch for **self-referential test gaps** — when the same agent wrote both the code and
  its tests, the guarantees are only proven for whatever path the agent chose to exercise.
- **Gate hygiene:** run the gate command **unpiped**. Piping (`gate | tail`) swallows the exit code and
  lets a red gate look green. A real failure mode — an agent committed on a red gate because the pipe
  masked it.
- **Credential / data paths are never gate-verified.** Mocked tests don't touch live OAuth / APIs. Those
  need a manual smoke test and a human review pass before merge, every time.
- Use `superpowers:verification-before-completion` before claiming anything is done — evidence before
  assertions, always.

## 5. Review discipline

- **Multi-agent first pass + audit pass.** Cheaper model for the first-pass review; escalate to the
  stronger model for an **audit pass** — both when the first pass returns *zero* findings (rubber-stamp
  risk) and to recheck first-pass findings (cheaper models over-state severities). The audit pass earns its
  cost: it has caught real latent bugs (e.g. a missing rollback on a failed commit) that the gate could not
  see, and corrected over-stated severities.
- **Review catches what the gate cannot.** Rollback/atomicity bugs with no test to trigger them,
  self-referential coverage gaps, credential-path assumptions — these are review findings, not gate
  findings. A human / multi-agent review before merge is **load-bearing, not optional**.
- **Outsider's eye.** Review what's there, not what's meant. If you helped author it, declare the
  conflict-of-interest up front and harden the review accordingly.
- **For a requested review, the review *is* the deliverable.** Synthesize and post it; don't ask
  separately for permission to post the thing you were asked to produce. (Unsolicited posts under your name
  still need the approval gate — see the rules.)
- **PR template and bypass:** see `PR-LIFECYCLE.md` for the end-to-end lifecycle (template usage,
  merge bar, solo-maintainer bypass). The hard rule on filling the template and scoping each PR to
  a single concern is in `.claude/rules/01-workflow.md`.

## 6. Autonomous / overnight loops

When letting work run unattended, the target is: **wake to either a gate-green increment or a clean halt
plus an actionable digest** — never "wake to a finished feature you haven't checked."

- **Halt contract.** The loop must have an explicit escape hatch: halt and report, don't press on. Phrase
  the completion signal as *handoff/digest-written* (true at any terminal state — done OR documented halt),
  not as task-success — otherwise a stuck agent churns to the iteration cap trying to earn a success it
  can't reach.
- **Budget / retry caps + wall-clock kill.** A session **cannot** read its own plan-usage % in-session
  (opaque by design). "Stop at X% of quota" is not buildable. Use `--max-iterations` plus a wall-clock kill
  as the backstop, and cap conservatively on early runs since iterations don't map cleanly to quota.
- **Test the halt path before trusting a run.** An unattended run that only ended because the agent
  finished naturally has never exercised its escape hatch — verify the halt actually works (missing
  dependencies have silently broken stop-hooks).
- **Forbid hard-to-reverse ops in the mandate.** No rebase / amend / force-push / history rewrites inside
  an autonomous loop — require halt-and-report instead. Start branches from the correct base.
- **Human-gated authorization is correct, not a nuisance.** Launching an autonomous loop trips the safety
  classifier by design. The agent must **not** retry, reframe, or route around the denial — the human
  authorizes it deliberately via an explicit allowlist. Keep that posture.

## 7. Communication discipline

- **Overclaim scrub on external-facing writing.** For funder pitches, marketing copy, public sites,
  outreach — make a *dedicated* pass that checks every load-bearing word against current evidence. Flag
  each claim with "is this earned?" High-risk words: *foundational, first, only, state-of-the-art,
  real-time, generalizes, platform, production-ready, scalable, shipped*. Separate what something *currently
  does* from what it's *designed to do*; any number needs a source, and if the benchmark hasn't run, say
  "pending."
- **Partners vs prospects.** Don't pitch shared values *at* partners (collaborators, advisors) — they
  already share them; pitching signals you've miscategorized them. Selling-point language is for prospects
  (institutions, customers, funders) who haven't bought in. A single draft can do both: the test is "am I
  pitching this *recipient* on something we already agree on?" — not whether selling-point words appear.

## 8. Memory and note-taking

- **Continuous capture.** Log context, decisions, and open threads as they happen. Load-bearing decisions
  get the *why*. Log altitudes, routing, and entry format are in `.claude/rules/03-logging.md`.
- **Checkpoint triggers:** before switching domains, before a long chat compacts, before a context-heavy
  subagent handoff, at any decision worth re-deriving later.
- **Long-lived facts go to auto-memory** (user / feedback / project / reference), indexed by `MEMORY.md` —
  one fact per file. This very doc is distilled from accumulated `feedback`-type memories; when you discover
  a durable working preference, capture it there so it survives the session.

---

## The skills that back these methods

The `superpowers` plugin provides process skills that operationalize most of the above —
`brainstorming`, `writing-plans`, `subagent-driven-development`, `dispatching-parallel-agents`,
`test-driven-development`, `systematic-debugging`, `verification-before-completion`,
`requesting-code-review`, `receiving-code-review`, `finishing-a-development-branch`. Lab-specific
conventions (like the code-free plan format) **override** a skill's default where they conflict —
user/lab instructions win over skill defaults.
