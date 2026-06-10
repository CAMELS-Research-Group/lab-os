## Summary

Refactor the loader module; no behavior change.

## Type of change

- [x] `refactor` — structural change, no behavior change

## Changes

- Split `src/loader.py` into `src/loader/core.py` and `src/loader/io.py`

## Motivation / context

Loader had grown past 500 lines; split per the module-size convention.

## Checklist

- [x] Scoped to a single concern (split if it spans multiple phases)
- [x] Commit messages follow the lab convention (`<type>[(<scope>)]: <subject>`, lowercase, imperative)
- [ ] Docs updated where required (CLAUDE.md / STANDARDS.md / `.claude/rules/` / READMEs — see `.claude/rules/01-workflow.md`)
- [x] No raw gated-dataset content, secrets, or binaries committed (see `.claude/rules/02-data-protection.md`)
- [ ] Derived artifacts (if any) passed the PII review checklist in `.claude/rules/02-data-protection.md`
- [ ] Log entries finalized (verified against final diff, index updated)
- [x] No loggable events in this PR
- [ ] Work-bundle archival included (slice declared done)

## Related

None.
