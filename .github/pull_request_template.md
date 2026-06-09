<!--
Lab PR template. Fill every section; tick only the checklist items that apply.
Keep the PR scoped to a single concern — split if it spans multiple phases.
-->

## Summary

<!-- One or two sentences: what this PR does and why. -->

## Type of change

<!-- Match the commit prefix (see .claude/rules/01-workflow.md). Tick one. -->

- [ ] `feat` — new feature or capability
- [ ] `fix` — bug fix
- [ ] `refactor` — structural change, no behavior change
- [ ] `docs` — documentation only
- [ ] `test` — test additions or fixes
- [ ] `chore` — maintenance, dependency updates
- [ ] `ci` — CI/CD config, GitHub Actions workflows, release/deploy automation
- [ ] `merge` — merge conflict resolution
- [ ] `report` — generated analysis or results

## Changes

<!-- Bullet the concrete changes. Reference file paths where useful. -->

-

## Motivation / context

<!-- The "why". Link the design doc, issue, or decision this implements. -->

## Verification

<!-- The exact command(s) a reviewer runs to confirm this works, and the observed result.
     For docs-only PRs: note that links resolve and content is accurate. -->

```
# Docs-only: links resolve, content accurate. Code repos: paste the gate command + result.
```

## Checklist

- [ ] Scoped to a single concern (split if it spans multiple phases)
- [ ] Commit messages follow the lab convention (`<type>[(<scope>)]: <subject>`, lowercase, imperative)
- [ ] Docs updated where required (CLAUDE.md / STANDARDS.md / `.claude/rules/` / READMEs — see `.claude/rules/01-workflow.md`)
- [ ] No raw gated-dataset content, secrets, or binaries committed (see `.claude/rules/02-data-protection.md`)
- [ ] Derived artifacts (if any) passed the PII review checklist in `.claude/rules/02-data-protection.md`

## Related

<!-- Linked issues / PRs. Use "Closes #N" to auto-close on merge. -->
