## Summary

Clarify the archive-overflow steps in PR-LIFECYCLE.md and fix two stale links.

## Type of change

- [x] `docs` — documentation only

## Changes

- `PR-LIFECYCLE.md`: rewrite the overflow walkthrough
- `docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md`: fix anchors
- `.github/workflows/ci.yml`: comment-only clarification of the gate step

## Motivation / context

Two readers tripped over the same ambiguous sentence; links broke after the section rename.

## Verification

```
# Docs-only: links resolve, content accurate.
```

## Checklist

- [x] Scoped to a single concern (split if it spans multiple phases)
- [x] Commit messages follow the lab convention (`<type>[(<scope>)]: <subject>`, lowercase, imperative)
- [x] Docs updated where required (CLAUDE.md / STANDARDS.md / `.claude/rules/` / READMEs — see `.claude/rules/01-workflow.md`)
- [x] No raw gated-dataset content, secrets, or binaries committed (see `.claude/rules/02-data-protection.md`)
- [ ] Derived artifacts (if any) passed the PII review checklist in `.claude/rules/02-data-protection.md`
- [ ] Log entries finalized (verified against final diff, index updated)
- [ ] No loggable events in this PR
- [ ] Work-bundle archival included (slice declared done)

## Related

None.
