# Logging

All lab repos. Rationale: `PR-LIFECYCLE.md`. Doc tiers, budgets, single-source: `04-docs.md`.

## Log altitudes

| Altitude | Anchor | Contents |
|---|---|---|
| Lab | `<DEV_ROOT>/project_log.md` | Cross-repo: tooling, infra, conventions, lab formation |
| Project | `<repo>/project_log.md` | Decisions outliving any one plan; irreversible/external events; direction changes |
| Spec-log | `_specs/<repo>/<DATE>-<handle>/log.md` | Bundle's single log: load-bearing decisions, discarded alternatives, and in-flight deviations/gate evidence the plan would discard |

The planning bundle is `_specs/<repo>/<DATE>-<handle>/{prd,spec,plan,log}.md` — `<DATE>` is the plan's `Date:` header, `<handle>` a short kebab-case slug; the `<repo>` segment is workspace-root altitude only — member repos omit it; file contract: `04-docs.md` §ENG (chore/docs-only bundles omit `spec.md`; code-touching slices add `design.md`). The plan carries no Execution Log; `log.md` absorbs it. `spec.md` is the bundle's "what is still true" surface; `log.md` the history — the same split at bundle altitude.

Test: matters after the plan ships? → project. Bundle-scoped (decisions, alternatives, deviations, gate evidence)? → spec-log. Cross-repo? → lab.

Closure: post-merge evidence (deploy green, runtime checks, branch cleanup) goes to a PR comment, never a trailing entry in any log. Anything bigger routes per Entry triggers.

Lab caveat: an untracked lab log is honor-system — immutability begins once a newer entry exists; archive when adding over cap. Where `<DEV_ROOT>` is a CI repo, its cap is CI-scored like any other surface.

## Entry triggers

Log only for:

1. **Load-bearing decision** — real alternatives; reversal changes direction/architecture
2. **Irreversible/external event** — release, migration, secret rotation, org/repo change, data published
3. **Direction change / re-scope** — pivot, pause, reactivation, supersession of a spec or plan (pause/retire a project → README top banner "Status: paused YYYY-MM-DD — see lab log")

Else routes:

| Information | Home |
|---|---|
| Deviation from approved plan | Bundle `log.md` (spec-log) |
| Expensive finding/gotcha | `TROUBLESHOOTING.md` or GitHub issue |
| Open work, follow-ups, review findings (repo-scoped) | GitHub issues (trigger-meeting findings also logged) |
| Cross-repo / lab-level open work | [upstream `BACKLOG.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/BACKLOG.md) (item shape: [`templates/backlog-item.template.md`](https://github.com/CAMELS-Research-Group/lab-os/blob/main/templates/backlog-item.template.md)) |
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

- ≤1,500 bytes/entry — project-log entries only (spec-logs are grep-only, unbudgeted); extra → PR body or spec
- `YYYY-MM-DD HH:MM` stamps are US Eastern, at all altitudes
- Count-free — no counts that restale
- PR# is the durable ref; never a squash SHA (untracked lab log: paths/URLs)
- No `Status:` field — currency lives in the index

## Immutability & supersession

Entries immutable once the PR merges. Reversal/revision = new entry with `Supersedes:`; never edit the old. Same PR removes the superseded index line; history keeps both. Factual fixes (typo'd PR#): PR with `log-lint:override` label + reason in body — never silent edits.

## File structure & overflow

`templates/docs/project_log.template.md` is normative (`log-lint` parses it) and owns the grammar: a `## Standing Decisions` index (the "what is still true" surface, read first), then `## Entries`, reverse-chron, top-insert.

Cap (whole file): 15 KB at project altitude, 40 KB at lab (§Log altitudes). Entry over cap → CI warns (never blocks); a dedicated `chore: archive log overflow` PR moves oldest entries to `project_log_archive.md` — prepended as a block, order preserved, byte-identical modulo EOL. Archive: grep-only, cap-exempt; still-binding archived decisions keep index lines, re-pointed.

**Spec-log overflow:** the bundle `log.md` has no Standing Decisions index and no cap enforcement; it is grep-only from the start. Spec-log entries append chronologically (oldest-first, bottom-insert — contrast project-log's reverse-chron top-insert). The Entry format applies, but `Refs: #PR` is relaxed: a bundle-relative path, or omitted until the PR lands. Overflow oldest entries to `log_archive.md` in the bundle — same block-prepend, byte-identical rule as above. Spec-logs do not feed the project-log index. On a terminal bundle status the whole bundle — log included — folds into the scope's main bundle and is deleted (`04-docs.md` §Bundle lifecycle); git history keeps the full text.
