# Logging

All lab repos. Rationale: `PR-LIFECYCLE.md`. Doc tiers, budgets, single-source: `04-docs.md`.

## Log altitudes

| Altitude | Anchor | Contents |
|---|---|---|
| Lab | `<DEV_ROOT>/project_log.md` | Cross-repo: tooling, infra, conventions, lab formation |
| Project | `<repo>/project_log.md` | Decisions outliving any one plan; irreversible/external events; direction changes |
| Plan-execution | `## Execution Log` in plan doc | Plan deviations, implementation calls, gate evidence; archives with the plan |

Test: matters after the plan ships? → project. Only how the plan ran? → plan-execution. Cross-repo? → lab.

Closure: a plan-execution log closes with the PR that ships it — post-merge evidence (deploy green, runtime checks, branch cleanup) goes to a comment on that PR, never a trailing entry held for a future PR; anything bigger routes per Entry triggers.

Lab caveat: no git/CI; honor-system — immutability begins once a newer entry exists; Refs = absolute paths/URLs, not PR#; archive when adding over cap.

## Entry triggers

Log only for:

1. **Load-bearing decision** — real alternatives; reversal changes direction/architecture
2. **Irreversible/external event** — release, migration, secret rotation, org/repo change, data published
3. **Direction change / re-scope** — pivot, pause, reactivation, supersession of a spec or plan (pause/retire a project → README top banner "Status: paused YYYY-MM-DD — see lab log")

Else routes:

| Information | Home |
|---|---|
| Deviation from approved plan | Plan doc `## Execution Log` |
| Expensive finding/gotcha | `TROUBLESHOOTING.md` or GitHub issue |
| Open work, follow-ups, review findings | GitHub issues (trigger-meeting findings also logged) |
| Bare status ("merged, smoke passed") | PR comment |
| Session narrative/what-I-did | PR body |
| Long-lived people/preference facts | Auto-memory |

## Entry format

```
## YYYY-MM-DD HH:MM — <subject, one line>

**Decision:** <what was decided/happened>
**Why:** <load-bearing rationale>
**Alternatives:** <only when real ones weighed>
**Supersedes:** <YYYY-MM-DD HH:MM — subject> <!-- superseding entries only -->
**Refs:** #<PR>, <absolute paths or URLs>
```

- ≤1,500 bytes/entry; extra → PR body or spec
- Count-free — no counts that restale
- PR# is the durable ref; never a squash SHA (lab altitude: paths/URLs)
- No `Status:` field — currency lives in the index

## Immutability & supersession

Entries immutable once the PR merges. Reversal/revision = new entry with `Supersedes:`; never edit the old. Same PR removes the superseded index line; history keeps both. Factual fixes (typo'd PR#): PR with `log-lint:override` label + reason in body — never silent edits.

## File structure & overflow

`templates/project_log.template.md` is normative (`log-lint` parses it): title + pointer to this standard; Standing Decisions index — one line per still-binding decision, hot window and archive alike (the "what is still true" surface, read first), date+subject match the entry header verbatim, created in the entry's PR (events: no index line); entries reverse-chron, top-insert, each preceded by `---` + blank line (conflict: keep both blocks, reorder by header timestamp).

Cap: 15 KB whole file. Entry over cap → CI warns (never blocks); a dedicated `chore: archive log overflow` PR moves oldest entries to `project_log_archive.md` — prepended as a block, order preserved, byte-identical modulo EOL. Archive: grep-only, cap-exempt; still-binding archived decisions keep index lines, re-pointed.
