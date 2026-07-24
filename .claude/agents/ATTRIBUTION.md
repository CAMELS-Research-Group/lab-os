# Specialist review agents — attribution & provenance

Provenance for the vendored agent bodies in this directory. Dispatch contract (triggers, cap, model
tier, merge rules) is owned by `reference/specialist-dispatch.md` and is not restated here.

## Anthropic, `pr-review-toolkit` plugin (Apache-2.0)

Four agents in `.claude/agents/` are vendored from Anthropic's **`pr-review-toolkit`** plugin
(official Claude Code plugin marketplace, `claude-plugins-official`; author: Anthropic,
support@anthropic.com). The plugin's `plugin.json` carries no version field; the vendored bodies
were taken from the marketplace cache as of 2026-07-23. Upstream license: **Apache-2.0** — full
text carried at `.claude/agents/LICENSE-pr-review-toolkit` per its terms.

| Agent | Origin | Changed? |
|---|---|---|
| `pr-test-analyzer` | `pr-review-toolkit/agents/pr-test-analyzer.md` | rewired — lab contract block added (report-only, C2 finding schema with `file::symbol` keys, taxonomy citations, diff scope); criticality ratings mapped to Blocker/Important/Suggestion; analytical core preserved |
| `silent-failure-hunter` | `pr-review-toolkit/agents/silent-failure-hunter.md` | rewired — as above; upstream project-specific logging/tooling facts (named logging functions, error-ID registry) and persona examples removed; severity vocabulary mapped from CRITICAL/HIGH/MEDIUM |
| `type-design-analyzer` | `pr-review-toolkit/agents/type-design-analyzer.md` | rewired — as above; taxonomy Classes 6/7 cited where applicable; four-axis ratings retained as analysis context |
| `comment-analyzer` | `pr-review-toolkit/agents/comment-analyzer.md` | rewired — as above; ownership boundary vs the doc-tier specialists stated (code comments/docstrings here; prose docs → `slop-hunter`) |

Not vendored: `code-reviewer` (redundant with the review skills' own outsider generalist) and
`code-simplifier` (an editor — it collides with the report → classify → remediate flow; its value is
covered by the `[simplification]` rubric class).

## Taxonomy citations

The agent bodies cite `reference/code-quality-taxonomy.md` classes **where one applies**. That
taxonomy is a manifest-synced asset whose byte source is the lab's workspace fork and it is not
carried in lab-os; where it does not resolve, no class applies and the citation is simply absent.
The bodies are phrased conditionally for exactly this reason and are vendored here verbatim —
byte-identical to the workspace-fork copies, so the two do not drift.
