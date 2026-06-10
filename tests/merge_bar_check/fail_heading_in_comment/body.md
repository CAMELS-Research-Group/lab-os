## Summary

Speed up the dataset manifest scan.

## Type of change

- [x] `refactor` — structural change, no behavior change

## Changes

- `src/manifest.py`: single-pass scan instead of per-file stat calls

## Motivation / context

Manifest builds were the slowest step of every training run.

## Verification

```
python -m pytest tests/test_manifest.py
# 14 passed
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

Nothing linked yet <!-- ## Related --> but issues may follow.
