# Rubric layering — how the review standard is composed

A review standard is assembled from **three tiers** at dispatch time. This file owns the discovery
paths, the precedence rule, and what happens when a tier is absent. It is the only place those facts
are defined.

## The tiers

| Tier | Home | Owns |
|---|---|---|
| **1 — Universal** | `reference/rubric-universal.md`, beside this file | Correctness, security, error handling, tests, doc accuracy, the structural classes, and overridable defaults. Names no organization or repo. |
| **2 — Lab** | `<DEV_ROOT>/reference/code-quality-taxonomy.md` + `<DEV_ROOT>/.claude/rules/` | Conventions the working group applies to **every** project: the class catalog and its thresholds, commit format, doc tiers and budgets, logging rules, merge bar, data protection. |
| **3 — Project** | `<repo>/reference/code-quality-rubric.md`, in the **repo under review** | That project's stack and project-scoped concerns: framework idioms, module boundaries, performance envelopes, domain invariants. |

`<DEV_ROOT>` is the dev home this skill is authored in. `PROMPT.md` Step 0.1 owns its derivation —
dereference the skill root's symlink and strip the trailing `.claude/skills/pr-round` — and is the
only place that derivation is defined. It is **not** reachable under `~/.claude/`, which carries no
`reference/` directory; a `DEV_ROOT` that cannot be derived makes tier 2 absent, and the degradation
rule below applies. `<repo>` is the local checkout of
the repository the PR under review belongs to, **not** the session's working directory: reviewing a
PR in a nested project repo must read that repo's tier 3, not the dev home's.

Tier 2 is consumed **in place**, never copied. The taxonomy declares itself the single owning source
of the finding-class catalog and lists its derivers; this skill is one of them.

## Resolution

Resolve tiers in order 1 → 2 → 3, before dispatching any review. For each tier record **resolved**
(with the path) or **absent** (with the path that was checked). The resolved set travels with the
review brief, and the tier list is reported in the review comment — a reader must be able to tell
what standard was actually applied.

Tier 1 is always resolved; it ships with the skill. If tier 1 cannot be read, that is a broken
install, not a degraded run — stop and say so.

## Precedence

- **Higher tier wins on the same axis.** Tier 2's file-size budget replaces tier 1's default; tier
  3's replaces tier 2's. The review states which value it applied when a tier overrode a default.
- **Tiers otherwise accumulate.** A check present in any resolved tier is in scope. Tiers are
  additive by default and only collide where they set the same axis.
- **No tier silently deletes a lower tier's check.** A higher tier may relax or waive a lower-tier
  check only by saying so explicitly. An omission is not a waiver — if tier 3 simply does not mention
  test coverage, tier 1's test requirement still applies.
- **Conflicts are reported, not resolved by guess.** If two tiers set the same axis to incompatible
  values and neither declares precedence, apply the higher tier and note the conflict as a
  Suggestion so the owner can reconcile the documents.

## Degradation when a tier is absent

A missing tier **never fails the run**. Review proceeds with whatever resolved, and:

1. The review comment names which tiers applied and which were absent, so no reader mistakes a
   tier-1-only review for a full-standard one.
2. The roster summary emits the cold-start prompt below, once per missing tier per repo.

**The skill never writes a rubric into a repo it was asked to review.** Authoring standards is not a
side effect a review invocation is entitled to. The cold-start prompt hands that work back to a
human, who can run it deliberately and open its own pull request.

## Cold-start prompt (owning source)

Emit verbatim, substituting the bracketed values. This text is defined here and nowhere else.

```
Draft a project-specific code-quality rubric for [REPO] at reference/code-quality-rubric.md.

It is tier 3 of a three-tier review standard. Tier 1 (universal correctness, security, error
handling, tests, docs, structural classes) and tier 2 (organization-wide conventions) are already
covered — do NOT restate them. Tier 3 covers only what is specific to this project:

- Stack and framework idioms this repo commits to, and the anti-patterns that follow from them
- Module boundaries and seams a change must not cross without saying why
- Domain invariants a reviewer cannot infer from the code alone
- Performance, memory, or latency envelopes that gate a change
- Any tier-1 or tier-2 default this project deliberately overrides — state the new value and why

Read the repo first: its CLAUDE.md or equivalent, README, dependency manifest, test layout, and the
three largest source modules. Ground every rule in something you actually found; do not write
aspirational standards the codebase does not follow. Where the codebase is inconsistent, say so and
propose the shape to converge on rather than blessing the status quo.

Keep it under 200 lines. Every rule needs a concrete trigger a reviewer can check against a diff.
```

## Reporting contract

The review brief carries the resolved tier list; the review comment carries a one-line statement of
it. A review that does not say which tiers it applied is under-specified — a finding's authority
depends on which standard produced it, and a reader cannot weigh a finding whose source is unstated.
