## Summary

Wire the retry policy into the upload client.

## Type of change

- [x] `feat` — new feature or capability

## Changes

- `src/upload.py`: exponential backoff on 5xx responses

## Motivation / context

Nightly uploads failed on transient registry errors.

## Verification

```
python -m pytest tests/test_upload.py
# 9 passed

## Related

- [x] Log entries finalized (verified against final diff, index updated)
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
