## Summary

Fix off-by-one in the budget byte counter.

## Type of change

- [x] `fix` — bug fix

## Changes

- `src/budget.py`: count bytes, not characters, when comparing against the cap

## Motivation / context

Multi-byte characters made files look smaller than they are on disk.

## Verification

```
python -m pytest tests/test_budget.py
# 12 passed
```

## Checklist

- [x] Scoped to a single concern (split if it spans multiple phases)
- [x] Commit messages follow the lab convention (`<type>[(<scope>)]: <subject>`, lowercase, imperative)
- [ ] Docs updated where required (CLAUDE.md / STANDARDS.md / `.claude/rules/` / READMEs — see `.claude/rules/01-workflow.md`)
- [x] No raw gated-dataset content, secrets, or binaries committed (see `.claude/rules/02-data-protection.md`)
- [ ] Derived artifacts (if any) passed the PII review checklist in `.claude/rules/02-data-protection.md`
- [ ] Log entries finalized (verified against final diff, index updated)
- [ ] No loggable events in this PR
- [ ] Work-bundle archival included (slice declared done)

## Related

Closes #17
