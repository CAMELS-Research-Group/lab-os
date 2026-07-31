# Logging

All lab repos. Rationale: `PR-LIFECYCLE.md`. Doc tiers, budgets, single-source: `04-docs.md`.

## Log altitudes

| Altitude | Anchor | Contents |
|---|---|---|
| Lab | `<DEV_ROOT>/project_log.md` | Cross-repo: tooling, infra, conventions, lab formation |
| Project | `<repo>/project_log.md` | Decisions outliving any one plan; irreversible/external events; direction changes |
| Spec-log | `_specs/<repo>/<DATE>-<handle>/log.md` | Bundle's single log: load-bearing decisions, discarded alternatives, and in-flight deviations/gate evidence the plan would otherwise discard |

The planning bundle is `_specs/<repo>/<DATE>-<handle>/{prd,spec,plan,log}.md` — `<DATE>` is the plan's `Date:` header, `<handle>` a short kebab-case slug; file contract: `04-docs.md` §ENG (chore/docs-only bundles omit `spec.md`). The plan carries no Execution Log; `log.md` absorbs it. `spec.md` is the bundle's "what is still true" surface; `log.md` the history — the Standing-Decisions/entries split at bundle altitude.

Test: matters after the plan ships? → project. Bundle-scoped (decisions, alternatives, deviations, gate evidence)? → spec-log. Cross-repo? → lab.

Closure: post-merge evidence (deploy green, runtime checks, branch cleanup) goes to a comment on that PR, never a trailing entry in any log. Anything bigger routes per Entry triggers.

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

- ≤1,500 bytes/entry — binds project-log entries only (spec-logs are grep-only, unbudgeted per `04-docs.md`); extra → PR body or spec
- `YYYY-MM-DD HH:MM` stamps are US Eastern, at all altitudes
- Count-free — no counts that restale
- PR# is the durable ref; never a squash SHA (untracked lab log: paths/URLs)
- No `Status:` field — currency lives in the index

## Immutability & supersession

Entries immutable once the PR merges. Reversal/revision = new entry with `Supersedes:`; never edit the old. Same PR removes the superseded index line; history keeps both. Factual fixes (typo'd PR#): PR with `log-lint:override` label + reason in body — never silent edits.

## File structure & overflow

`templates/docs/project_log.template.md` is normative (`log-lint` parses it): title + pointer to this standard; Standing Decisions index — one line per still-binding decision, hot window and archive alike (the "what is still true" surface, read first), date+subject match the entry header verbatim, created in the entry's PR (events: no index line); entries reverse-chron, top-insert, each preceded by `---` + blank line (conflict: keep both blocks, reorder by header timestamp).

Cap (whole file): 15 KB at project altitude, 40 KB at lab (§Log altitudes). Entry over cap → CI warns (never blocks); a dedicated `chore: archive log overflow` PR moves oldest entries to `project_log_archive.md` — prepended as a block, order preserved, byte-identical modulo EOL. Archive: grep-only, cap-exempt; still-binding archived decisions keep index lines, re-pointed.

**Spec-log overflow:** `_specs/<repo>/<DATE>-<handle>/log.md` has no Standing Decisions index and no cap enforcement; it is grep-only from the start. Spec-log entries append chronologically (oldest-first, bottom-insert — contrast project-log entries, which are reverse-chron, top-insert). The Entry format applies, but the ≤1,500-byte entry cap does not bind here and the `Refs: #PR` requirement is relaxed: spec-log entries are often written before a PR exists, so Refs may carry a bundle-relative path or be omitted until the PR lands. When a `log.md` grows unwieldy, overflow oldest entries to `_specs/<repo>/<DATE>-<handle>/log_archive.md` co-located in the same bundle — same block-prepend, byte-identical rule as project-log overflow. Spec-logs do not feed the project-log index.
