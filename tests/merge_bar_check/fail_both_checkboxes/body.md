## Summary

Add retry wrapper around the webhook poster.

## Type of change

- [x] `feat` — new feature or capability

## Changes

- `src/notify.py`: exponential backoff, three attempts, on webhook POST

## Motivation / context

Transient Discord 5xx responses were dropping notifications.

## Verification

```
python -m pytest tests/test_notify.py
# 8 passed
```

## Checklist

- [x] Scoped to a single concern (split if it spans multiple phases)
- [x] Commit messages follow the lab convention (`<type>[(<scope>)]: <subject>`, lowercase, imperative)
- [ ] Docs updated where required (CLAUDE.md / STANDARDS.md / `.claude/rules/` / READMEs — see `.claude/rules/01-workflow.md`)
- [x] No raw gated-dataset content, secrets, or binaries committed (see `.claude/rules/02-data-protection.md`)
- [ ] Derived artifacts (if any) passed the PII review checklist in `.claude/rules/02-data-protection.md`
- [x] Log entries finalized (verified against final diff, index updated)
- [x] No loggable events in this PR
- [ ] Work-bundle archival included (slice declared done)

## Related

None.
