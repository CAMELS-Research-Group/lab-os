# Workflow: Commits, PRs, and Documentation

Applies across all WatsonWBlair lab repos.

## Commit Messages

Format: `<type>[(<scope>)]: <subject>`

| Prefix | When to use |
|--------|-------------|
| `feat` | New feature or capability |
| `fix` | Bug fix |
| `refactor` | Structural change, no behavior change |
| `docs` | Documentation only |
| `test` | Test additions or fixes |
| `chore` | Maintenance, dependency updates |
| `ci` | CI/CD config, GitHub Actions workflows, release/deploy automation |
| `merge` | Merge conflict resolution |
| `report` | Generated analysis or results |

- Lowercase subject, no trailing period
- Active voice, present tense ("add" not "added")
- Under 72 characters
- Scope optional for subsystem clarity
- Body optional — use for "why" when the diff doesn't make it obvious

## Pull Request Workflow

- Use `.github/pull_request_template.md` for every PR. No template in the repo? Draft one and ask before adopting.
- Fill all template sections — preserve checklist items verbatim, tick only the ones that apply
- Pass the filled template as `--body` via HEREDOC to `gh pr create`
- Scope each PR to a single concern — split if it spans multiple phases

## Merge Bar

All of the following, verified at merge time:

1. **Gate green, run unpiped.** The gate is the repo's designated verification command per its `CLAUDE.md` (e.g. `invoke check`); unpiped because piping (`gate | tail`) swallows the exit code and lets a red gate look green. No gate (docs-only repo)? The PR template's Verification section is the gate.
2. PR template complete; checklist items ticked only where true
3. Review findings resolved, or routed to issues (a finding meeting an entry trigger in `03-logging.md` additionally gets a log entry)
4. **Log cleanup done** (entry rules: `03-logging.md`): this PR's loggable events have entries; entries verified against the final diff; compressed to budget; refs filled (PR#); no edits to pre-existing entries; Standing Decisions index updated — line added per new standing decision, superseded lines removed; log over cap → overflow chore PR filed
5. Doc-sync triggers checked (Documentation Update Triggers below)
6. Single concern; bundle archival rides only when the owner declares the slice done

## Documentation Update Triggers

- **CLAUDE.md**: when commands, data sources, env vars, or workflow change
- **STANDARDS.md + matching `.claude/rules/` file**: when conventions or rules change
- **Subsystem READMEs**: when a module's API, responsibility, or usage patterns change
- **README.md**: when setup steps, project structure, or developer workflow change

Docs out of sync with the code = not done.
