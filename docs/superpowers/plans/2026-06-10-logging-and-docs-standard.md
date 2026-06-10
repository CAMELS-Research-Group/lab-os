# Logging & Documentation Standard — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Tasks use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the §11 deliverables of the [logging & documentation standard spec](../specs/2026-06-10-logging-and-docs-standard-design.md) — rules, lifecycle docs, templates, and three adherence Actions — in the lab-rules repo.

**Architecture:** Enforceable rules live in `.claude/rules/` (loaded into every Cowork session and every PR review); narrative lives in root docs; templates make compliance copyable; reusable `workflow_call` Actions + Python (stdlib-only) scripts do mechanical enforcement, with lab-rules consuming its own Actions from day one.

**Tech Stack:** Markdown, GitHub Actions (`workflow_call`, `ubuntu-latest`), Python 3.11 stdlib scripts.

**Plan format note (lab rule):** tasks specify *what* the implementation must satisfy, not *how*. No literal code; the implementing agent owns structure, function names, and test design. The only code blocks are shell commands in Verification lines.

**Branch:** `docs/logging-and-docs-standard` (spec already committed there). All tasks land on this branch; one PR at the end.

---

## Phase A — rules and core docs

### Task 1: Core rule file `03-logging-and-docs.md`

**Files:**
- Create: `.claude/rules/03-logging-and-docs.md`

**Depends on:** —

**Spec:** [§4 Project-log standard](../specs/2026-06-10-logging-and-docs-standard-design.md#4-project-log-standard), [§6 Single-source](../specs/2026-06-10-logging-and-docs-standard-design.md#6-single-source--derived-views), [§7 Tiers and budgets](../specs/2026-06-10-logging-and-docs-standard-design.md#7-documentation-tiers-and-context-budgets), [§10 ENG-tier document standards](../specs/2026-06-10-logging-and-docs-standard-design.md#10-eng-tier-document-standards-prd-design-plan)

Context: this is the always-loaded, review-time-enforced core; it must be terse — rationale belongs in the spec and PR-LIFECYCLE.md, not here.

**Acceptance:**
- States the three log altitudes with their anchors and the altitude test (§4.1), including the lab-altitude caveat in compressed form
- States the three entry triggers and the full routing table (§4.2), including the pause/retirement README-banner parenthetical
- Shows the canonical entry format block (§4.3) with the byte budget, count-free phrasing rule, PR-number-as-ref rule, and no-Status rule
- States immutability + supersession (§4.4) including the `log-lint:override` escape hatch
- States the file-structure contract (§4.5) by pointing at `templates/project_log.template.md` as normative, and the hot-window cap + overflow-chore-PR mechanism (§4.6)
- States the single-source rule incl. the public→private generalized-restatement form (§6)
- States the three tiers in one compact table and the context budgets with warn/fail multipliers (§7)
- States the ENG-tier document standards: PRD required elements + no-embedded-decision-log rule, design-doc contents + status line, code-free plan elements + Execution Log section (§10)
- States the rules numbering convention: lab `0x-*`, per-repo `10+` (D12)
- File is ≤ 5,120 bytes (its own §7.2 budget) — dense reference style, no narrative

**Verification:**
```powershell
(Get-Item .claude/rules/03-logging-and-docs.md).Length -le 5120
```
Plus: confirm every §4.2 routing-table row and every §7.2 budget row from the spec appears (manual diff against spec sections).

**Commit:** `feat: add logging and docs standard rule`

---

### Task 2: Merge bar in `01-workflow.md`

**Files:**
- Modify: `.claude/rules/01-workflow.md`

**Depends on:** 1

**Spec:** [§8.2 Merge bar](../specs/2026-06-10-logging-and-docs-standard-design.md#82-merge-bar-hard-rule-added-to-01-workflowmd)

**Acceptance:**
- New "Merge bar" section reproduces all six §8.2 items, including: gate defined as the repo's designated verification command per its CLAUDE.md, the unpiped requirement with its one-line reason, the docs-only fallback (PR template Verification section), the full log-cleanup item (entries verified against final diff, compressed, refs filled, no edits to pre-existing entries, index updated, overflow chore PR filed if over cap), and bundle-archival-on-declared-done
- Existing commit-message, PR-workflow, and doc-update-trigger sections are unchanged
- Log-cleanup item cross-references `03-logging-and-docs.md` rather than restating entry rules (single-source)
- File stays ≤ 5,120 bytes

**Verification:**
```powershell
(Get-Item .claude/rules/01-workflow.md).Length -le 5120; git diff HEAD~1 -- .claude/rules/01-workflow.md
```
Diff shows only additions (no modified/deleted existing lines).

**Commit:** `feat: add merge bar to workflow rules`

---

### Task 3: `PR-LIFECYCLE.md`

**Files:**
- Create: `PR-LIFECYCLE.md`

**Depends on:** 1, 2

**Spec:** [§8 PR lifecycle](../specs/2026-06-10-logging-and-docs-standard-design.md#8-pr-lifecycle-merge-bar-solo-maintainer-bypass), [§4.6 overflow](../specs/2026-06-10-logging-and-docs-standard-design.md#46-hot-window-and-overflow), [§1 (the four sources)](../specs/2026-06-10-logging-and-docs-standard-design.md#1-problem)

Context: the onboarding-read narrative companion — rationale lives here so the rules files can stay terse.

**Acceptance:**
- Narrates the full lifecycle: branch → PR from template → automated review (naming pr-review-agent's cron role and pr-review-loop's remediation role, linking `pr-review-agent/SPEC.md` and the plugin) → remediation → merge bar (linking `01-workflow.md`, not restating it) → merge mechanics (squash, bundle archival, branch delete)
- Codifies the solo-maintainer bypass exactly per §8.3: independent multi-agent review (first pass + audit pass per lab model defaults) posted to the PR before admin bypass, merge note referencing it
- Explains overflow/archive mechanics (§4.6) including the dedicated chore-PR pattern and the smart-distillation of still-binding decisions into the Standing Decisions index
- Explains *why* behind the load-bearing choices (immutability/supersession, byte budgets, warn-not-block overflow) in a short rationale section linking the spec
- Tone/format: ENG tier (§7.1) — skimmable, stable heading anchors
- Reader test: a new lab member could take a PR from branch to merge using only this doc plus the files it links

**Verification:**
```powershell
git grep -n "01-workflow.md" PR-LIFECYCLE.md; git grep -n "SPEC.md" PR-LIFECYCLE.md
```
Both return hits; manual read-through confirms no merge-bar restatement.

**Commit:** `docs: add end-to-end PR lifecycle guide`

---

### Task 4: Lab `TROUBLESHOOTING.md`

**Files:**
- Create: `TROUBLESHOOTING.md`

**Depends on:** —

**Spec:** [§4.2 routing](../specs/2026-06-10-logging-and-docs-standard-design.md#42-entry-triggers-and-routing-table), [§7.1 grep-only tier](../specs/2026-06-10-logging-and-docs-standard-design.md#71-tiers-defined-by-reader), [§11 deliverables row](../specs/2026-06-10-logging-and-docs-standard-design.md#11-deliverables-map)

**Acceptance:**
- Header states the contract: grep-only surface, looked up by symptom, never read whole; one symptom-titled section per gotcha
- Seeded with the known cross-platform gotchas, each as symptom → cause → resolution: line-ending normalization (`autocrlf` clones vs LF files, incl. the byte-comparison implication for log archival), junction (Windows) vs symlink (Unix) for the rules link, path-separator differences in docs/commands, PowerShell vs POSIX quoting in shell snippets
- Setup *steps* are not duplicated from BOOTSTRAP.md — entries link to it where the fix is "redo a setup step"
- States where new entries come from (the §4.2 routing rule for expensive findings)

**Verification:**
```powershell
git grep -in "autocrlf" TROUBLESHOOTING.md; git grep -in "junction" TROUBLESHOOTING.md
```
Both return hits.

**Commit:** `docs: add lab troubleshooting doc seeded with cross-platform gotchas`

---

### Task 5: PR template checkboxes

**Files:**
- Modify: `.github/pull_request_template.md`

**Depends on:** 2

**Spec:** [§11 deliverables row](../specs/2026-06-10-logging-and-docs-standard-design.md#11-deliverables-map), [§9 merge-bar-check](../specs/2026-06-10-logging-and-docs-standard-design.md#9-adherence-actions)

**Acceptance:**
- Adds two **separate, mutually exclusive** checkboxes: "Log entries finalized (verified against final diff, index updated)" and "No loggable events in this PR"
- Adds "Work-bundle archival included (slice declared done)" as an applicable-only item
- Does not duplicate the existing docs-updated checklist item; all pre-existing template content preserved verbatim
- Checkbox wording is exactly what Task 11's `merge-bar-check` script will match (this template is the source of truth for that script's expectations)

**Verification:**
```powershell
git diff HEAD~1 -- .github/pull_request_template.md
```
Diff shows only added lines; both new checkbox texts present.

**Commit:** `feat: add log-cleanup checkboxes to PR template`

---

## Phase B — templates

### Task 6: Project-log template (normative)

**Files:**
- Create: `templates/project_log.template.md`

**Depends on:** 1

**Spec:** [§4.3 entry format](../specs/2026-06-10-logging-and-docs-standard-design.md#43-entry-format), [§4.5 file structure](../specs/2026-06-10-logging-and-docs-standard-design.md#45-file-structure)

Context: this file is **normative** — `log-lint` (Task 9) parses exactly this structure, so its skeleton is a contract, not an example.

**Acceptance:**
- Skeleton top-to-bottom per §4.5: title + one-line pointer to the lab standard; Standing Decisions index section with the exact line grammar (`- YYYY-MM-DD HH:MM — <subject> · #<PR-or-archive-link>`); entries region delimiter; one example entry showing the full §4.3 format including the optional `Supersedes:` line
- Index/entries delimiting is unambiguous for a parser (a fixed heading or separator a script can anchor on — implementer chooses, then Task 9 consumes the same choice)
- Example entry demonstrates the date+subject key matching between its header and its index line
- Placeholder text instructs: top-insert, `---` separator before each entry, conflict resolution = keep both blocks reordered by timestamp

**Verification:**
```powershell
git grep -n "Supersedes" templates/project_log.template.md; git grep -nE "[0-9]{4}-[0-9]{2}-[0-9]{2} [0-9]{2}:[0-9]{2}" templates/project_log.template.md
```
Both return hits.

**Commit:** `feat: add normative project log template`

---

### Task 7: Per-repo CLAUDE.md seed template

**Files:**
- Create: `templates/repo-CLAUDE.template.md`

**Depends on:** 1

**Spec:** [§7 tiers and budgets](../specs/2026-06-10-logging-and-docs-standard-design.md#7-documentation-tiers-and-context-budgets)

**Acceptance:**
- Section scaffold for a per-repo CLAUDE.md: what-this-repo-is, commands, architecture orientation, boundaries/invariants, conventions pointer (to lab rules + repo rules `10+`)
- Inline guidance comments tell the author what belongs in each section and what to route to ENG-tier docs instead (the budget-preserving move)
- AI-tier writing standard stated at top as author guidance: dense, deterministic, count-free, zero narrative
- Template itself (with guidance comments) is ≤ 4,096 bytes, so a filled copy starts well inside the 8,192-byte budget
- Consistent with the existing `templates/global-CLAUDE.template.md` / `dev-root-CLAUDE.template.md` naming and tone

**Verification:**
```powershell
(Get-Item templates/repo-CLAUDE.template.md).Length -le 4096
```

**Commit:** `feat: add per-repo CLAUDE.md seed template`

---

### Task 8: Work-bundle + PRD templates

**Files:**
- Create: `templates/work-bundle/design.template.md`
- Create: `templates/work-bundle/plan.template.md`
- Create: `templates/PRD.template.md`

**Depends on:** 1

**Spec:** [§5 work-artifact lifecycle](../specs/2026-06-10-logging-and-docs-standard-design.md#5-work-artifact-lifecycle-bundling--archival), [§10 ENG-tier document standards](../specs/2026-06-10-logging-and-docs-standard-design.md#10-eng-tier-document-standards-prd-design-plan)

**Acceptance:**
- `design.template.md`: status line at top (draft / reviewed / superseded-by), then problem / decisions-with-rationale-and-rejected-alternatives / known-gaps sections per §10
- `plan.template.md`: per-task six-element skeleton (Files / Depends on / Spec / Acceptance / Verification / Commit), a no-literal-code reminder, and the `## Execution Log` section with a one-line entry grammar for deviations and gate evidence
- `PRD.template.md`: the six required elements (Problem · Success criteria · Scope in/out · Constraints · Plan phased · Open questions) plus a visible note that decisions live in `project_log.md`, never embedded in the PRD
- Each template carries a one-line header pointing at the bundle lifecycle (`docs/work/` → `completed/` / `abandoned/`) per §5
- ENG-tier style: skimmable, stable anchors

**Verification:**
```powershell
Get-ChildItem templates/work-bundle/, templates/PRD.template.md; git grep -n "Execution Log" templates/work-bundle/plan.template.md
```
All three files exist; Execution Log section present.

**Commit:** `feat: add work-bundle and PRD templates`

---

## Phase C — adherence Actions

### Task 9: `log-lint` script + fixtures + workflow

**Files:**
- Create: `scripts/log_lint.py`
- Create: `tests/log_lint/` (fixture pairs: baseline + changed log, valid and violating)
- Create: `.github/workflows/log-lint.yml`

**Depends on:** 6

**Spec:** [§9 Adherence Actions](../specs/2026-06-10-logging-and-docs-standard-design.md#9-adherence-actions) (incl. the immutability algorithm), [§4.3–4.6](../specs/2026-06-10-logging-and-docs-standard-design.md#43-entry-format)

Context: the immutability algorithm is fully specified in §9 — entry-set comparison keyed by header, baseline = target-branch HEAD, archive reappearance byte-identical modulo EOL, index region exempt.

**Acceptance:**
- Script takes a baseline log, a changed log, and optionally the archive file; parses both into entry sets keyed by `## YYYY-MM-DD HH:MM — <subject>` headers using the Task 6 template structure
- Passes: well-formed new entries as one contiguous block at the head of the entries region, non-strict descending timestamps; index-only changes; archive moves where removed entries reappear in the archive byte-identical modulo EOL normalization
- Fails (distinct, named violations): malformed entry header; entry over 1,500 bytes; new entry inserted mid-history; pre-existing entry modified; entry removed without verbatim archive reappearance; index line whose date+subject key matches no entry (hot window or archive)
- Index region is exempt from immutability checks
- Workflow (`workflow_call`): runs only when the PR touches `project_log.md` or its archive; computes baseline from the PR's target-branch HEAD; honors the `log-lint:override` PR label — skips enforcement but **fails if the label is present and the PR body lacks an override reason**
- Fixtures cover every pass/fail behavior above; Python 3.11 stdlib only
- Script exits 0/1 with one human-readable line per violation (file, entry header, violation name)

**Verification:**
```powershell
python scripts/log_lint.py --self-test
```
Exit 0; self-test runs every fixture pair and asserts the expected verdicts (implementer wires `--self-test` to the fixtures so CI and humans share one entry point).

**Commit:** `feat: add log-lint adherence action`

---

### Task 10: `docs-budget` script + workflow

**Files:**
- Create: `scripts/docs_budget.py`
- Create: `tests/docs_budget/` (fixtures: under-budget, 1.0–1.5×, over-1.5× files)
- Create: `.github/workflows/docs-budget.yml`

**Depends on:** 1

**Spec:** [§7.2 context budgets](../specs/2026-06-10-logging-and-docs-standard-design.md#72-context-budgets-always-loaded--first-read-ai-surfaces), [§9](../specs/2026-06-10-logging-and-docs-standard-design.md#9-adherence-actions)

**Acceptance:**
- Measures byte sizes against the §7.2 budgets: `CLAUDE.md` 8,192; each `.claude/rules/*.md` 5,120; `project_log.md` 15,360
- Between 1.0× and 1.5× → warning (GitHub warning annotation), exit 0; above 1.5× → exit 1 — unless warn-only mode
- `workflow_call` input `enforce` (default **false** — the §7.2 warn-only-until-first-green posture); `enforce: false` never fails, only warns
- Missing surfaces are skipped silently (not every repo has every file)
- Junction/symlink-aware: a rules dir that resolves outside the repo is skipped (mission-control's junction case)
- Fixtures cover all three zones and both modes; stdlib only; output names each file, its size, its budget, and its zone

**Verification:**
```powershell
python scripts/docs_budget.py --self-test
```
Exit 0; self-test asserts zone classification and both-mode behavior across fixtures.

**Commit:** `feat: add docs-budget adherence action`

---

### Task 11: `merge-bar-check` script + workflow

**Files:**
- Create: `scripts/merge_bar_check.py`
- Create: `tests/merge_bar_check/` (fixture PR bodies + changed-file lists)
- Create: `.github/workflows/merge-bar-check.yml`

**Depends on:** 5

**Spec:** [§9](../specs/2026-06-10-logging-and-docs-standard-design.md#9-adherence-actions)

**Acceptance:**
- Script takes the PR body and the PR's changed-file list; verifies all required template sections (headings from `.github/pull_request_template.md`) are present
- When any changed file matches the code-path globs: exactly one of the two log checkboxes (Task 5 wording) must be ticked — neither or both → fail with a message naming the rule
- When no changed file matches the globs: checkbox state is not enforced; section presence still is
- `workflow_call` input for the glob set; documented default = everything except `*.md`, `docs/**`, `.github/**`
- Fixtures cover: full-compliance pass, missing section, neither checkbox, both checkboxes, docs-only PR skip; stdlib only

**Verification:**
```powershell
python scripts/merge_bar_check.py --self-test
```
Exit 0 across all fixtures.

**Commit:** `feat: add merge-bar-check adherence action`

---

### Task 12: lab-rules self-caller

**Files:**
- Create: `.github/workflows/standards.yml`

**Depends on:** 9, 10, 11

**Spec:** [§9](../specs/2026-06-10-logging-and-docs-standard-design.md#9-adherence-actions) ("lab-rules adds its own caller in phase 1"), [§11](../specs/2026-06-10-logging-and-docs-standard-design.md#11-deliverables-map)

**Acceptance:**
- On `pull_request`, invokes all three reusable workflows against lab-rules itself (local `uses:` references — no cross-repo checkout needed here; this file also serves as the copyable example for phase-2 caller YAMLs in other repos)
- `docs-budget` called with `enforce: false` initially (flips after first green, per §7.2)
- A comment block documents what a downstream repo's caller looks like (checkout of lab-rules + `uses:` reference), so phase 2 is copy-paste

**Verification:**
```powershell
gh workflow list; gh run list --workflow standards.yml --limit 1
```
After the PR is opened (Task 13+ PR), the run appears and completes; all three jobs green.

**Commit:** `ci: lab-rules consumes its own adherence actions`

---

## Phase D — integration

### Task 13: Migrate lab-rules' own project log + log this initiative

**Files:**
- Modify: `project_log.md`
- Create (if overflow requires; not expected): `project_log_archive.md`

**Depends on:** 6, 9

Context: lab-rules' log has a single entry in the old format; once Task 12's self-caller runs, `log-lint` checks this file — it must comply, and migrating it is the standard's first live exercise.

**Spec:** [§4.5](../specs/2026-06-10-logging-and-docs-standard-design.md#45-file-structure), [§12 rollout](../specs/2026-06-10-logging-and-docs-standard-design.md#12-rollout-phase-2-separate-effort) (this is the lab-rules instance, pulled into phase 1)

**Acceptance:**
- `project_log.md` adopts the Task 6 template head (pointer line + Standing Decisions index + entries region)
- The existing 2026-06-09 onboarding-project entry is preserved (content unchanged, reformatted header is acceptable for this one-time migration — noted in the PR body under the `log-lint:override` reason if the lint flags it)
- A new entry records this standard's adoption: decision, why, refs (the spec path + this PR) — written per §4.3, within byte budget, with its Standing Decisions index line
- `python scripts/log_lint.py` accepts the migrated file (self-consistency: the standard's own repo passes its own lint)

**Verification:**
```powershell
python scripts/log_lint.py --baseline <git show origin/main:project_log.md> --changed project_log.md
```
Exit 0 (implementer wires exact CLI flags; intent: migrated log passes the lint against the pre-migration baseline, or the PR carries the documented override).

**Commit:** `docs: migrate lab-rules project log to the new standard`

---

### Task 14: Trim `WORKING-WITH-CLAUDE.md` to pointers

**Files:**
- Modify: `WORKING-WITH-CLAUDE.md`

**Depends on:** 1, 2, 3

**Spec:** [§11 deliverables row](../specs/2026-06-10-logging-and-docs-standard-design.md#11-deliverables-map)

**Acceptance:**
- §5: only the bypass and PR-template bullets become pointers (to `PR-LIFECYCLE.md` / `01-workflow.md`); the audit-pass, outsider's-eye, and review-is-the-deliverable bullets are **retained verbatim**
- §8: log-location prose replaced with a pointer to `03-logging-and-docs.md`; checkpoint triggers and auto-memory guidance **retained verbatim**
- No other section touched

**Verification:**
```powershell
git diff HEAD~1 -- WORKING-WITH-CLAUDE.md
```
Diff confined to §5 and §8; retained bullets absent from the deletion side of the diff.

**Commit:** `docs: point working-with-claude at the new standard docs`

---

### Task 15: README "What's here" update

**Files:**
- Modify: `README.md`

**Depends on:** 1, 3, 4

**Spec:** [§11 deliverables row](../specs/2026-06-10-logging-and-docs-standard-design.md#11-deliverables-map)

**Acceptance:**
- "What's here" lists the new rule file, `PR-LIFECYCLE.md`, `TROUBLESHOOTING.md`, the new templates, and the adherence Actions (one line each, linking the file)
- "How repos consume it" gains one sentence + link for the phase-2 caller-YAML pattern (pointing at `standards.yml` as the example)
- Scope-discipline paragraph unchanged

**Verification:**
```powershell
git grep -n "PR-LIFECYCLE" README.md; git grep -n "standards.yml" README.md
```
Both return hits.

**Commit:** `docs: update readme for logging and docs standard`

---

## Execution Log

*(Deviations from this plan, implementation-altitude calls, and gate evidence land here as tasks execute — one line each: `YYYY-MM-DD HH:MM · task N · what/why`.)*

- 2026-06-10 16:16 · task 1 · gate: `(Get-Item .claude/rules/03-logging-and-docs.md).Length` = 5,116 ≤ 5,120, unpiped; all §4.2 routing rows + §7.2 budget rows confirmed; spec review needed 2 rounds (5 rules lost to compression, restored)
- 2026-06-10 16:18 · task 4 · gate: `git grep` autocrlf + junction both hit, unpiped; spec review 2 rounds — removed command blocks duplicated from BOOTSTRAP.md §3/§5 per the doc's own no-setup-steps contract
- 2026-06-10 16:25 · task 2 · gate: 2,746 B ≤ 5,120; diff 11+/0− (additions only), unpiped; altitude call: spec-internal "(§4.6)"/"(§5)" citations not transplanted into the rules file — overflow mechanics reachable via item 4's `03-logging-and-docs.md` pointer, bundle lifecycle lands in PR-LIFECYCLE.md (task 3)
- 2026-06-10 16:27 · task 3 · gate: both required greps hit (01-workflow.md ×4, SPEC.md ×2), unpiped; spec review's two factual findings rejected after independent check (.github/pull_request_template.md exists; §4–5 citation correct — self-report-is-not-evidence lives in §4)
- 2026-06-10 16:29 · task 5 · gate: diff 5+/0− additions-only, all three checkbox texts verbatim per spec §11; spec review caught a dangling "(§5 of 03-logging-and-docs.md)" comment ref — re-pointed at PR-LIFECYCLE.md Merge mechanics
