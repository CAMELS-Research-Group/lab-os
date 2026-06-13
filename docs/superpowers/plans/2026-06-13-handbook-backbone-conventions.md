# Handbook Backbone Authoring-Conventions — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Git is authoritative for progress.

**Goal:** Ship `site/AUTHORING.md` — the living contributor reference capturing the three handbook authoring conventions — so every subsequent page round applies them consistently.

**Architecture:** One ENG-tier Markdown reference in the site repo. It restates the spec's decisions as actionable, copy-paste authoring rules and links back to the spec as the rationale source. No site code changes this round; the conventions are applied per page in later rounds.

**Tech Stack:** Markdown; Docusaurus 3.10 (code-block `title=` support, `<details>` collapsibles, category-index doc structure).

**Spec references** (read before claiming the task):
- [Design doc](../specs/2026-06-13-handbook-backbone-conventions-design.md) — the conventions and rationale
- [01-workflow.md](../../../.claude/rules/01-workflow.md) — commit/PR conventions
- [04-docs.md](../../../.claude/rules/04-docs.md) — doc tiers, single-source rule

## How to consume a task

- **Files** lists exact paths. **Spec** links the governing section. **Acceptance** is the behaviors the
  doc must demonstrate. **Verification** is the exact command(s) to run. **Commit** is the PR title.
- The spec and this plan are already committed on `feat/handbook-backbone`; the PR carries them plus the
  task below. Ask Watson before creating/merging the PR.

---

## Task 1: Author the conventions reference

**Context:** A single contributor-facing reference so page authors don't re-derive the conventions from the spec each time; the spec owns the decisions, this owns the how-to.

**Files:**
- Create: `site/AUTHORING.md`

**Depends on:** (none)

**Spec:** [§3–§6](../specs/2026-06-13-handbook-backbone-conventions-design.md#3-depth-model-d1)

**Acceptance:**
- Opens with a one-line purpose and a pointer naming the spec as the source of truth for *why* (single-source per `04-docs.md`).
- **Depth model section** documents the hybrid model with a copy-paste folder-layout example (`docs/<section>/index.md` overview + `docs/<section>/<subtopic>.md` sub-pages), states that the sidebar category label links to the overview, and gives the sub-page-vs-collapsible test verbatim from spec §3 (self-contained procedure/topic someone would land on or link to → sub-page; 2–5 sentence tied-to-one-spot aside → `<details>` collapsible).
- **Command-marking section** shows the exact code-block syntax for both contexts (a fenced block titled `Terminal` and one titled `Claude session`), and includes the one-time legend snippet authors paste near the top of mixed pages. The examples render correctly (valid Docusaurus code-block title syntax).
- **Zero-tech section** states the link-out-plus-fill-gaps rule: link canonical install/signup docs; write a lab "from scratch" sub-page only where the official path is confusing or has lab-specific steps; screenshots allowed where they materially help, accepting refresh cost.
- **Heading-anchor constraint** is noted: page rounds that move/rename headings cited by `PR-LIFECYCLE.md` / `TROUBLESHOOTING.md` (the `working-with-claude` anchors) update the referrers in the same commit.
- Skimmable (headings per convention, examples in code blocks); within the ENG-tier spirit of `04-docs.md` (no rigid byte budget applies to `site/AUTHORING.md`, but keep it tight).
- Contains no `<...>` placeholders left unfilled and no TODO markers.

**Verification:**
```bash
cd site && npm run build
python scripts/docs_budget.py --root .
```
Expected: build succeeds (the file is repo-level Markdown, not a built doc page, so it must not break the build); docs_budget passes. Manually confirm `site/AUTHORING.md` contains one copy-paste example per convention and a working link to the spec.

**Commit:** `docs(handbook): add AUTHORING.md authoring-conventions reference`

---

## Self-review

- **Spec coverage:** §3 depth model → Task 1 depth section; §4 command marking → command-marking section; §5 zero-tech → zero-tech section; §6 (the reference itself) → the whole task; §3 migration/anchor note → heading-anchor constraint bullet. §5 D5 (per-page inventories deferred) and the page reworks are out of scope per spec §8 — no task, correctly.
- **Placeholder scan:** none.
- **Consistency:** file path `site/AUTHORING.md` matches spec §6; commit subject matches the lab `docs(scope): subject` convention.
