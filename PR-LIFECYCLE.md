# PR Lifecycle

The end-to-end story of a lab PR: branch → review → remediation → merge bar → merge. This is the
onboarding narrative — it explains *how the pieces fit and why*. The enforceable rules live in
[`.claude/rules/01-workflow.md`](.claude/rules/01-workflow.md) (commits, PR workflow, **Merge
Bar**); this doc links them rather than restating them.

It consolidates what previously lived in four places: the cron reviewer's design
(`pr-review-agent/SPEC.md`), the `pr-review-loop` plugin docs, the
[Working with Claude → Review](site/docs/working-with-claude/review.md) stage, and the solo-maintainer bypass pattern,
which was practiced but unwritten until now.

**At a glance:**

```
branch → PR from template → automated review ⇄ remediation → merge bar → squash merge
                                                                  ↑          (+ bundle archival,
                                                    solo-maintainer bypass     branch delete)
                                                    when peer review impossible
```

## 1. Branch

Branch from the default branch, one branch per concern. Commit messages follow the
conventional-commit format in [`01-workflow.md`](.claude/rules/01-workflow.md). If the work is a
slice with a design, its planning bundle (`_specs/<repo>/<DATE>-<handle>/`, holding
`prd`/`spec`/`plan`) already exists or is created here — its shape and lifecycle are owned by
[`04-docs.md` § Bundle lifecycle & the main bundle](.claude/rules/04-docs.md).

## 2. Open the PR from the template

Every PR uses the repo's `.github/pull_request_template.md` — fill all sections, tick checklist
items only where true, pass the filled body via HEREDOC to `gh pr create`. The template is
load-bearing: it encodes the commit-type, doc-update, and data-protection gates so they surface
at review time instead of after merge. Rules:
[`01-workflow.md`](.claude/rules/01-workflow.md).

## 3. Automated review

Two automated reviewers with distinct roles:

- **`pr-review-agent`** — the standing cron reviewer. A sibling repo in the lab workspace
  (`<DEV_ROOT>/pr-review-agent/SPEC.md` is its design doc;
  [github.com/WatsonWBlair/pr-review-agent](https://github.com/WatsonWBlair/pr-review-agent)). It
  runs 2×/day and posts an external-perspective review to every open PR with a new head SHA, body
  prefixed `[automated review by lab-pr-reviewer]`. Its job is the outside-the-problem look: fresh
  context, adversarial prompt, no inherited author framing. It loads `lab-os/.claude/rules/*.md`
  into its review prompt, so it is also the *semantic* enforcement arm — it checks what CI
  parsers cannot (was a decision actually logged? is a restated fact owned elsewhere?).
- **`pr-review-loop`** — the on-demand remediation driver. A Claude Code plugin from the lab
  marketplace ([github.com/WatsonWBlair/lab-claude-plugins](https://github.com/WatsonWBlair/lab-claude-plugins);
  `/plugin install pr-review-loop@lab-claude-plugins`). Where the cron agent *finds* issues on its
  own schedule, the loop *drives* a PR through review → fix → re-review cycles until the merge bar
  is met, mostly hands-off.

Either path satisfies the review step; on an active PR they compose (the cron agent's findings
feed the loop's next cycle). The review *method* — multi-agent first pass + audit pass,
outsider's eye, self-report-is-not-evidence — is the
[Working with Claude → Verify](site/docs/working-with-claude/verify.md) and
[Review](site/docs/working-with-claude/review.md) stages.

## 4. Remediation

Findings are resolved on the branch or explicitly routed to GitHub issues — never silently
dropped. Treat review feedback with rigor, not performative agreement: verify a finding is real
before fixing it, and push back with evidence when it isn't.
Each remediation push gets a fresh review pass (the cron agent re-reviews new SHAs automatically;
the loop re-reviews by design).

## 5. The merge bar

The merge bar is the hard rule — six conditions verified at merge time, defined in
[`01-workflow.md` → Merge Bar](.claude/rules/01-workflow.md). In short: gate green (run unpiped),
template complete, findings resolved or routed, **a durable decision with no bundle behind it
added directly to the scope's main-bundle `spec.md`**, doc-sync triggers checked, single concern —
plus, on a bundle-backed slice declared done, the PRD `Status:` flip. Read the rule file for the
authoritative list; nothing merges below it.

## 6. Solo-maintainer bypass

GitHub blocks self-approval, so a sole maintainer cannot satisfy a required-review branch rule.
The bypass is codified, not informal:

When the required peer review is impossible (sole maintainer; self-approval blocked), an
**independent multi-agent review** — first pass + audit pass per the lab model defaults
(cheaper-model first pass, stronger-model audit;
[Working with Claude → Review](site/docs/working-with-claude/review.md))
— must be completed and **posted to the PR** before admin bypass. The merge note references the
posted review. This codifies the practice established on Global_Pathways #139–#142.

The point: bypass skips the *mechanism* (a human approval click), never the *review*. The posted
review is the durable evidence that the merge bar was held to.

## 7. Merge mechanics

1. **Squash merge.** One commit per PR on the default branch; the PR number is the durable
   reference (which is why a decision section records `#<PR>` and never a squash SHA — the SHA
   doesn't exist until after the section is written).
2. **The fold rides** — if this PR is the one where the owner declares a slice done, the
   bundle's PRD `Status:` flips to `complete` and the bundle folds file-to-file into the scope's
   main bundle, its directory deleted, in the same PR (or its own `chore:` PR later; finality is
   hindsight). Git history is the archival record — see
   [`04-docs.md` § Bundle lifecycle & the main bundle](.claude/rules/04-docs.md).
3. **Delete the branch.**

## Why it works this way

- **Byte budgets, not line counts.** Line budgets are gamed by simply not wrapping; bytes are
  what actually fill an agent's context window, so bytes are what's measured (per rule file, and
  in aggregate across the always-loaded tier).
- **PR# as the durable ref.** Squash merging rewrites SHAs; PR numbers survive. Recording a SHA
  in a decision section would be wrong the moment the merge button is pressed.
