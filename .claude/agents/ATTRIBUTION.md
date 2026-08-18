# Specialist review agents — attribution & provenance

Provenance for the agent bodies in this directory — vendored and lab-authored alike. Dispatch
contract (triggers, cap, model tier, merge rules) is owned by `reference/specialist-dispatch.md`
and is not restated here.

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

## Lab-authored agents (no upstream)

| Agent | Origin | Derives from |
|---|---|---|
| `spec-plan-analyzer` | Lab-original — written for lab-os, not lifted from any plugin | The repo's own `.claude/rules/04-docs.md` § ENG document standards and `.claude/rules/03-logging.md`, read at review time |

No upstream license applies to this body; `LICENSE-pr-review-toolkit` covers the four vendored
agents only.

**It carries no restated checklist by design.** The lab's ENG document standards differ per repo
and change over time, so the agent resolves them from the rules in the repo under review and
derives its checks there — a body that restated them would violate `04-docs.md` § Single source and
would go stale at the next rules sync. Where those sources do not resolve, it returns a named
not-run dimension instead of reviewing against invented criteria (dispatch reference
§ Degradation).

## Taxonomy citations

Every agent body here cites `reference/code-quality-taxonomy.md` classes **where one applies**.
Ownership of that file runs the same direction as § Byte ownership below: upstream lab-os owns the
convention and the canonical bytes; Caravan (github.com/CAMELS-Research-Group/Caravan) is the
staging surface where its bytes are edited and where Caravan's `scripts/rules_sync.py`
(§ Manifest) vendors it into member repos. Wherever the file does not resolve — including a lab-os
clone that does not yet carry it — no class applies and the citation is simply absent. All bodies
are phrased conditionally for exactly this reason.

## Byte ownership

**lab-os is the byte master for every agent body in this directory and for the dispatch contract
they cite, `reference/specialist-dispatch.md`**: these copies are canonical,
and the workspace fork inherits them through the normal `git pull upstream main` direction — never
by a back-port. Identity with the fork copies is restored at each sync, not asserted between them:
between an upstream edit and the fork's next pull, the copies may legitimately differ, and the
resolution is always a pull, never the reverse. `spec-plan-analyzer` moves the same direction; it
originates **here**, in lab-os, which owns it. (Byte-master choice recorded per the PR #61 review.)
