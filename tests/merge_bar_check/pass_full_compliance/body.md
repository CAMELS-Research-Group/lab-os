## Summary

Add the merge-bar-check adherence script and reusable workflow.

## Type of change

- [x] `feat` — new feature or capability

## Changes

- Add `scripts/merge_bar_check.py`
- Add `.github/workflows/merge-bar-check.yml`

## Motivation / context

Implements the adherence actions from the logging-and-docs standard design, section 9.

## Verification

```
python scripts/merge_bar_check.py --self-test
# exit 0, all fixtures pass
```

## Checklist

- [x] Scoped to a single concern (split if it spans multiple phases)
- [x] Commit messages follow the lab convention (`<type>[(<scope>)]: <subject>`, lowercase, imperative)
- [ ] Docs updated where required (CLAUDE.md / STANDARDS.md / `.claude/rules/` / READMEs — see `.claude/rules/01-workflow.md`)
- [x] No raw gated-dataset content, secrets, or binaries committed (see `.claude/rules/02-data-protection.md`)
- [ ] Derived artifacts (if any) passed the PII review checklist in `.claude/rules/02-data-protection.md`
- [x] Log entries finalized (verified against final diff, index updated)
- [ ] No loggable events in this PR
- [ ] Work-bundle archival included (slice declared done)

## Related

Closes #42
