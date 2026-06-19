# Working with Claude rework — SDD-lifecycle reframe — design

Status: reviewed — approved by Watson 2026-06-13 (brainstorm + spec sign-off).

Second per-page round of the handbook content rework. Applies the backbone conventions
(`2026-06-13-handbook-backbone-conventions-design.md`, working reference `site/AUTHORING.md`):
hybrid depth model (folder + sub-pages + collapsibles), Terminal-vs-Claude command marking,
link-out-plus-fill-gaps zero-tech support.

## 1. Problem

`site/docs/working-with-claude.md` is a flat list of eight numbered "discipline" sections
(Process before code, Code-free plans, Subagent-driven development, Verification, Review,
Autonomous loops, Communication, Memory). The numbering is arbitrary, the sections don't tell a
reader *when* in their work each method applies, and the heaviest material (autonomous loops, the
full subagent contract) is jammed into one page behind `<details>` blocks. A new member can't read
it as a procedure — it reads as a checklist of lessons, not a way of working. It needs to become
the **spec-driven-development (SDD) lifecycle the lab actually runs**, so a reader walks the stages
in order and knows what each stage demands.

## 2. Decisions

| # | Decision | Alternatives rejected |
|---|---|---|
| D1 | **Reframe as a 7-stage SDD lifecycle**: Brainstorm → Specify → Plan → Build → Verify → Review → Close. Verify is the *automated* checkpoint; Review is the *human* checkpoint | Keep the flat numbered list (no when-does-this-apply signal); fewer/more stages (7 maps cleanly onto the existing eight sections + global philosophy) |
| D2 | **Communication + Memory become a cross-cutting "Throughout" section**, not lifecycle stages — they apply at every stage | Leave them as terminal stages 7–8 (misleads: they aren't a final step) |
| D3 | **Convert to a folder**: `working-with-claude/index.md` (lifecycle overview) + sub-pages for the heavy stages; URL stays `/docs/working-with-claude` | Keep one page (can't carry per-stage depth without `<details>` sprawl) |
| D4 | **Sub-page inventory**: `plan.md`, `build.md`, `autonomous-loops.md`, `verify.md`, `review.md`. Brainstorm / Specify / Close stay **inline** in the overview (light stages) | A sub-page per stage (Brainstorm/Specify/Close too thin to stand alone); no sub-pages (heavy stages overload the overview) |
| D5 | **Positive-framed discipline**; name a failure mode **only** where the discipline is meaningless without it (e.g. "an agent's self-report is not evidence" is *why* the Verify gate exists) | A standalone "anti-patterns" catalog (reads as a list of scary stories, not a method) |
| D6 | **Framing**: the lab's *discovered* way of working, owned by the working group, open to change; new members do a **manual pass** through the lifecycle before independent work | Present as fixed law (contradicts "earned in practice, still evolving") |
| D7 | **Update every moved anchor's referrers in the same commit** (anchor constraint) | Defer referrer fixes (breaks the build — `onBrokenLinks: 'throw'`) |

## 3. Structure

```
site/docs/working-with-claude/
  index.md            # lifecycle overview: the 7 stages at a glance; Brainstorm/Specify/Close
                      #   inline; the cross-cutting "Throughout" (Communication + Memory)
  plan.md             # Plan stage — code-free implementation plans
  build.md            # Build stage — subagent-driven development
  autonomous-loops.md # Build, deep dive — autonomous / overnight loop safety contract
  verify.md           # Verify stage — the automated checkpoint
  review.md           # Review stage — the human checkpoint
```

### 3.1 `index.md` (overview)

- **Reframed lead-in** (D6): the lab's discovered SDD lifecycle, owned by the working group and
  still evolving; new members walk it manually once before independent work. Keep the existing
  pointers: hard rules live in `.claude/rules/`; the operating *philosophy* (PRD-first, pushback,
  reversibility, review mode) lives in the global `CLAUDE.md` template; several stages lean on the
  `superpowers` plugin's process skills, and where a lab convention conflicts with a skill default,
  the lab convention wins.
- **The lifecycle at a glance** — the 7 stages named in order, one line each, with Verify flagged
  as the automated checkpoint and Review as the human checkpoint. Heavy stages link inward
  (→ Plan, → Build, → Verify, → Review).
- **Brainstorm** (inline) — `superpowers:brainstorming` before any creative work; process skills
  decide *how* before implementation skills. Check what exists first (search lab repos + lineage
  per the dev-root `CLAUDE.md`) — extending close-enough beats building new.
- **Specify** (inline) — PRD before a non-trivial build: Problem · Success criteria · Scope ·
  Constraints · Plan · Open questions. No code until sign-off; silence isn't approval.
- **Plan** (inline summary + link) — one-paragraph "what a code-free plan is", → [Plan](./plan.md).
- **Build** (inline summary + link) — → [Build](./build.md); autonomous loops called out as the
  deep dive → [Autonomous loops](./autonomous-loops.md).
- **Verify** (inline summary + link) — the automated checkpoint, → [Verify](./verify.md).
- **Review** (inline summary + link) — the human checkpoint, → [Review](./review.md).
- **Close** (inline) — log decisions/threads as they happen (which log, what earns one, format:
  source of truth `03-logging.md`); checkpoint before domain switches / compaction / subagent
  handoff; durable preferences → auto-memory.
- **Throughout: Communication & Memory** (D2) — cross-cutting section: overclaim scrub on
  external-facing writing; partners-vs-prospects; continuous capture / checkpointing (the memory
  bullets that aren't Close-specific). Framed as applying at every stage, not as a final step.

### 3.2 Sub-pages (current content, relocated and lightly expanded)

- **`plan.md`** — current §2 (Code-free implementation plans) verbatim-in-substance: the six
  elements, the no-code rule, the `superpowers:writing-plans` override, source-of-truth link.
- **`build.md`** — current §3 (Subagent-driven development): the plan-is-the-handoff-artifact
  point, plus the `<details>` content (context boundaries, backlog mechanics) promoted to prose.
  Links to `autonomous-loops.md` as the deep dive for unattended runs.
- **`autonomous-loops.md`** — current §6 (Autonomous / overnight loops): the wake-to-gate-or-halt
  target, the halt contract, plus the `<details>` content (budget caps, halt-path testing,
  forbidden ops, human-gated authorization) promoted to prose.
- **`verify.md`** — current §4 (Verification discipline): self-report-isn't-evidence (the D5
  failure-mode that justifies the stage), optimistic narrator, green≠reviewed, run-unpiped,
  credential/data paths never gate-verified, `superpowers:verification-before-completion`.
- **`review.md`** — current §5 (Review discipline): multi-agent first pass + audit pass,
  review-catches-what-the-gate-cannot, outsider's eye, review-is-the-deliverable,
  PR-template/merge-bar/solo-maintainer links.

No new *facts* are introduced; this is a re-organization. Each sub-page keeps its existing
source-of-truth links (`01-workflow.md`, `04-docs.md`, `PR-LIFECYCLE.md`, the global template).

## 4. Anchor / referrer updates (D7)

Moving sections from one page into folder sub-pages changes their URLs. Every referrer below is
updated **in the same commit**; the build (`onBrokenLinks: 'throw'`) is the enforcement.

| Old target | New target | Referrers to update |
|---|---|---|
| `working-with-claude.md#4-verification-discipline` | `working-with-claude/verify.md` | `PR-LIFECYCLE.md` §3 (line ~60) |
| `working-with-claude.md#5-review-discipline` | `working-with-claude/review.md` | `PR-LIFECYCLE.md` §intro (line ~11), §6 (line ~88) |
| `/docs/working-with-claude#2-code-free-implementation-plans` | `/docs/working-with-claude/plan` | `onboarding-project.md` (Axis-2 step 3, line ~52) |
| `/docs/working-with-claude#6-autonomous--overnight-loops` | `/docs/working-with-claude/autonomous-loops` | `onboarding-project.md` (Axis-1 row 5, line ~39) |
| `site/docs/working-with-claude.md` (page) | `site/docs/working-with-claude/index.md` | `WORKING-WITH-CLAUDE.md` (repo-root pointer, line ~8) |

Page-level links with no anchor that resolve to the category index need no change: `play-testing.md`
(line ~20), `getting-started/index.mdx` (line ~178), `onboarding-project.md` (line ~18) all link
`/docs/working-with-claude`, which still resolves to the folder overview. The visible "§4–5" /
"§5" text in `PR-LIFECYCLE.md` is reworded to name the **Verify** / **Review** stages, since the
section numbers no longer exist.

At `onboarding-project.md`'s two cited spots, the surrounding link text is **lightly reworded** to
match the new stage names (Watson call 2026-06-13): "autonomous loops" and "lab plan format" link
phrasing is aligned to the Plan stage / Autonomous-loops sub-page. This is the only prose touched
in that file — the broader Onboarding Workshop reframe remains its own round.

`TROUBLESHOOTING.md` is named as a potential referrer in `AUTHORING.md` §4 but currently cites no
`working-with-claude` anchor (confirmed by grep) — no change needed.

## 5. Sidebar

In `sidebars.ts`, the flat `'working-with-claude'` entry (under "Get started") becomes a nested
category labeled "Working with Claude", `collapsed: true`, `link` → `working-with-claude/index`,
with the five sub-pages as `items` in lifecycle order: `plan`, `build`, `autonomous-loops`,
`verify`, `review`. Mirrors the Getting Started nesting from #26.

## 6. Command marking

Apply the convention retroactively where this round touches a command block. This page is mostly
prose; the few inline references to slash-commands/skills (`/init`, `superpowers:*`) are named in
running text, not fenced blocks, so no `title=` is added unless a block is introduced. No legend is
needed (no Terminal/Claude-session command-block mix).

## 7. Scope

**In:** the folder conversion; the overview reframe (7 stages + Throughout); the five sub-pages
(relocated existing content, `<details>` promoted to prose); the anchor/referrer updates (§4); the
sidebar nesting; this spec + its plan.

**Out:** the Onboarding Workshop reframe (its own round — only its two cited anchors, plus a light
reword of their surrounding link text, are touched here); any change to the rules in `.claude/rules/` or the global template; new methods or facts
(this is a reorganization, not a content expansion); the other handbook pages' rounds.

## 8. Verification

- `cd site && npm run build` (unpiped) passes — `onBrokenLinks: 'throw'` confirms the folder URL,
  every sub-page link, and all updated referrers resolve; MDX valid.
- `python scripts/docs_budget.py --root .` passes (each sub-page well under the per-file budget;
  splitting one page into six *reduces* per-file size).
- Manual: the overview reads as a walkable lifecycle (7 stages in order, Verify=automated /
  Review=human flagged); Communication + Memory read as cross-cutting, not a final step; sidebar
  shows a collapsed "Working with Claude" with the five sub-pages in lifecycle order; the old
  flat page is gone (no route conflict).

## 9. Known gaps

- Whether `index.md` vs `index.mdx` for the overview matters: `.md` is parsed as MDX by Docusaurus
  and the page uses only `<details>` (valid MDX), so `.md` is chosen for consistency with the
  sub-pages; confirmed by the build during implementation.
- The exact prose for promoting each `<details>` block to body text is settled during
  implementation — substance is fixed (it's the current text), phrasing is the implementer's.
- `WORKING-WITH-CLAUDE.md` is a repo-root convenience pointer; if its source-link line proves
  redundant with the live Pages URL it could later be trimmed, but that's out of scope here — this
  round only repoints it.
