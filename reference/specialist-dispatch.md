# Specialist review dispatch — triggers, caps, merge contract

**Single owning source** for how lab review skills dispatch the specialist review agents in
`.claude/agents/`. Do not duplicate any table below elsewhere. The originating design bundle
(`2026-07-23-specialist-review-agents`, decisions C1–C4, D3, D7, D9, D11, D12) lives in the lab's
workspace fork and is **not** carried here; this file is the operative contract, and it is
self-contained — nothing below requires reading that bundle.

**Consumers (in place — no skill copies any table from this file):**

- `pr-round` (`.claude/skills/pr-round/`) — review lane only. The remediate lane does not dispatch
  specialists.
- `pr-review-loop` — single-PR loop review dispatch, every pass; the multi-PR conductor path does
  **not** dispatch specialists. That skill is **not yet carried in lab-os** (it is maintained in the
  lab's workspace forks), so this row describes the contract it honours where it is present.

**Specialists are report-only.** No specialist edits any file; remediation belongs to the
coordinating skill's existing flow (spec C4).

## Roster

| Agent | Dimension | Status |
|---|---|---|
| `.claude/agents/pr-test-analyzer.md` | Test coverage quality | active |
| `.claude/agents/silent-failure-hunter.md` | Error handling / silent failures | active |
| `.claude/agents/type-design-analyzer.md` | Type design / invariants | active |
| `.claude/agents/comment-analyzer.md` | Code comments / docstring rot | active |
| doc-tier trio (`slop-hunter`, `spec-plan-analyzer`, `info-design-reviewer`) | AI / ENG / Public doc tiers | Phase 2 — not yet vendored; trigger rows land with them |

## Trigger table (code specialists)

Deterministic predicates, evaluable from `git diff --name-status` plus the diff hunks alone — no
repo execution, no reviewer judgment. Evaluate against the **pinned diff the pass reviews** (the PR
diff at the reviewed head sha); re-review cycles re-evaluate against the remaining diff, so the
panel shrinks as a loop converges. A predicate fires on **added or modified** lines/files only —
pure deletions dispatch nothing.

**Executable-source file set** (used below): any changed path with extension
`.py .pyi .js .mjs .cjs .ts .tsx .jsx .go .rs .java .kt .c .cc .cpp .h .hpp .cs .rb .php .swift
.scala .sh .bash .ps1 .psm1 .sql .r .jl .lua .pl`. Markdown, plain text, `_specs/**`, and pure
config data (`.json .yaml .toml .ini` with no embedded script) are **not** executable source.

| Specialist | Predicate (dispatch when…) |
|---|---|
| `pr-test-analyzer` | Any executable-source file is added or modified (`git diff --name-status` shows `A`/`M`, or `R` with content edits — `R<100>` — on a path in the set above; a pure rename `R100` has no added/modified lines and does not fire). |
| `silent-failure-hunter` | Any added/changed hunk line matches an error-handling token: `try` / `except` / `catch` / `finally` / `rescue` / `.catch(` / `Result<` / `unwrap` / `panic` / `raise` / `throw` / `on_error` / `-ErrorAction` / `set -e` / `trap ` / `2>/dev/null` / `2>$null` / `\|\| true` / optional-chaining-with-swallow (`?.` or `??` introduced alongside a removed error branch). |
| `type-design-analyzer` | Any added/changed hunk line introduces or alters a type definition: `class` / `interface` / `enum` / `struct` / `trait` / `protocol` / `type <Name> =` / `dataclass` / `TypedDict` / `NamedTuple` / schema definitions (`pydantic`, `zod`, `.proto` messages). |
| `comment-analyzer` | Comment density over threshold: added comment/docstring lines (language comment leaders: `#`, `//`, `/* … */`, `"""…"""`, `'''…'''`, `<!-- -->` inside source files) are **≥ 15% of added lines AND ≥ 10 lines**, or any docstring block is added or modified. Counts code-file comments only — prose docs are doc-tier surface (Phase 2). |

A pass whose diff fires no predicate dispatches no specialists — the generalist review runs alone,
exactly as before this reference existed.

## Cap, model tier, escape hatch

- **Per-pass cap: 4.** If more than four predicates fire (possible once the Phase-2 trio lands),
  dispatch the four whose trigger evidence spans the most changed files; name the undispatched
  dimension(s) in the pass output (see Degradation — an intentional skip is still a named skip).
- **Model tier: Opus, high effort** — spec D7, per the DeepSWE cost-effectiveness finding
  (generalizing the lab model-default table is tracked separately in the workspace fork). The
  coordinating skill sets this at dispatch; agent bodies stay `model: inherit` so the tier is owned
  here, not in vendored files.
- **Escape hatch: `--no-specialists`.** Both consuming skills accept the flag; it skips trigger
  evaluation and dispatches the generalist only. The pass output states that specialists were
  disabled by flag.
- Specialists dispatch **in parallel with the generalist** (same message / same wave), never
  serially after it (spec D6).

## Finding schema (spec C2)

Every specialist returns findings in this shape; free-form prose outside it is **ignored by the
merge stage**:

- **Severity:** `Blocker` / `Important` / `Suggestion`.
- **Classification:** `mechanical` / `design-pin` (per the consuming skill's
  `classify-blockers.md`).
- **Key:** `<file>::<symbol>` for code findings (same convention as the `[simplification]`
  `target:` key); bare `<file path>` for doc findings.
- **Taxonomy citation:** the `reference/code-quality-taxonomy.md` class, where one applies;
  omitted (not invented) where none does. **The taxonomy is not carried in lab-os** — it is a
  manifest-synced asset whose byte source is the lab's workspace fork. Where it does not resolve,
  no class applies, so the citation is simply absent: this is the schema's existing "omitted where
  none does" path, not a degraded mode, and the agent bodies already phrase it conditionally.
- **Evidence pointer:** one line — file + line/heading and what was observed.

## Merge / dedup rules (spec D9)

The coordinating skill's parse step gains a **merge stage**; everything downstream of it (routing,
ledger, close-out, verdicts, consent) is unchanged.

1. **Dedup keys.** Code findings: `file::symbol` + category (the taxonomy class, or the
   specialist's dimension where untagged — which is every finding wherever the taxonomy is not
   carried, so dedup stays well-defined without it). Doc findings: file class per the ownership map
   (Phase 2).
2. **Same key + same category** (generalist vs specialist, or two specialists): one finding.
   Keep the higher severity; credit both sources in the merged finding text.
3. **Same symbol, different category** (cross-category collision): the merge stage makes a
   **recorded same-defect judgment** — same defect or distinct. When judged the same defect, the
   merged finding **records the absorption**: which finding absorbed which, both sources credited,
   so the posted review stays auditable. This is the one deliberately non-mechanical step; the
   recording is what keeps it honest.
4. **Severity authority (spec D11).** Specialist Blockers are full Blockers — they count toward
   the consuming skill's effective-Blocker gate exactly like generalist Blockers — subject only to
   the severity-ceiling knob below.
5. After the merge, findings flow into the consuming skill's existing machinery (severity sections,
   mechanical/design-pin routing, `[simplification]` ledger where applicable) with no further
   special-casing.

## Per-agent severity-ceiling knob (default: none)

A noisy dimension is demoted **here, by config**, never by editing a vendored agent body. A ceiling
caps that agent's findings at the named severity during the merge stage (a `Blocker` from a
`ceiling: Important` agent enters the merge as `Important`, and is annotated as demoted).

| Agent | Ceiling |
|---|---|
| `pr-test-analyzer` | none |
| `silent-failure-hunter` | none |
| `type-design-analyzer` | none |
| `comment-analyzer` | none |

## Degradation (spec C3, D12)

- **Loud, never silent.** A specialist that errors, times out, or returns schema-invalid output is
  reported in the pass output **and** in the posted review comment as a named not-run dimension
  (e.g. `specialists: test-coverage ✓, silent-failures ✗ (agent error — dimension not reviewed)`).
  The pass completes on the remaining findings; it never silently narrows.
- **Cap skips are named the same way** (see Cap above).
- **Missing this reference** (a `pr-round` run on a non-lab repo): degrade exactly like a missing
  rubric tier — the run proceeds specialist-less and the posted comment names the absent layer.
  One degradation pattern, not two.

## Per-repo public-tier path registry (Phase 2 — schema stated, not yet populated)

Doc-tier dispatch (Phase 2) classifies Public-tier surfaces via an explicit per-repo path registry
— deterministic, **fail-closed**: prose not listed here routes to `slop-hunter`, never to
`info-design-reviewer` by inference. Schema — one row per repo, glob paths relative to that repo's
root:

| Repo | Public-tier paths (globs) |
|---|---|
| _(empty until Phase 2 — T9 populates this repo's rows)_ | |
