# lab-os — main PRD (standing)

**Scope:** lab-os · **Living** — exempt from byte budgets.
**Spec:** [spec.md](./spec.md) · **Design:** [design.md](./design.md) ·
**Plan:** [plan.md](./plan.md) · **Log:** [log.md](./log.md)

---

## Problem

The lab's repos are worked by several members and many agent sessions. Without one owning
source, conventions drift per repo and per member: each repo grows its own commit format,
its own review bar, its own idea of what "done" means, and every agent session inherits
whichever copy it happens to be sitting next to. Re-deriving the conventions per repo is
both wasted work and a silent correctness risk — an agent that reads a stale copy enforces
a standard the lab no longer holds.

lab-os exists to be that owning source: the canonical bytes of the lab-wide conventions,
the review tooling that applies them, and the CI that scores adherence.

## Success criteria (standing)

- Every lab repo's `0x` rules are byte-identical to this repo's, modulo the one-line sync
  header, and drift is detectable mechanically rather than by reading.
- A convention change has exactly one place it can be authored and one path it travels:
  staged in Caravan, reviewed there, round-tripped here, vendored out.
- The adherence gates (`docs-budget`, `merge-bar-check`, `log-lint`) are callable by any
  member repo as reusable workflows, so no repo carries a second copy of the check.
  `backlog-lint` and `backlog-views` are the same shape but lab-os-only in practice — the
  lab-wide `BACKLOG.md` lives here, and `standards.yml`'s downstream caller block omits them.
- A reader arriving cold can answer "what is implemented here right now" from
  [design.md](./design.md) without reconstructing it from history.

## Scope

**In:** the base `0x` rules · the specialist review agents and shared skills · the adherence
CI workflows and their scripts · the doc and planning templates · the handbook site · the
lab-wide backlog · `reference/` contracts that rules and skills derive from.

**Out:**

- Per-repo rules (`10+`) and per-repo gate commands — owned by the member repo.
- Project/product code. lab-os holds conventions and tooling, not applications.
- The `_specs` archive of the pre-Caravan dev home. `WatsonWBlair/Agentic_Workspace` stays
  live as the browsable archive and nothing there is deleted — D16, owned by that fork's
  `_specs/lab-os/2026-08-06-spec-home-migration/decisions.md` §D16; this scope governs
  lab-os's own bundles only.
- Staging of rule edits. Caravan is the staging surface — D17, same register §D17, carried
  forward in Caravan's `project_log.md` at 2026-08-10 19:40 (Caravan #2); this repo receives
  the round-trip.

## Constraints

- **Byte budgets** bind the always-loaded tier and `project_log.md`. Which surfaces are
  bound, the numbers, and the WARN/FAIL thresholds are owned by
  [`04-docs.md`](../../../.claude/rules/04-docs.md) §Tiers & budgets and are read there,
  never restated here; `docs_budget.py` is the enforcer, not the source.
- **Upstream owns the canonical bytes; Caravan stages the edits.** A rule is never edited
  first here — it lands here as a reviewed round-trip.
- **Approval gates** apply to anything posted under a member's own identity, spend above a
  member's stated ceiling, destructive operations on shared state, and gated-data exposure.
- **Project-altitude decisions live in `project_log.md`**, not in this bundle. The main
  bundle links to them and never restates them.

## Open questions

- **Do the pre-Caravan dev home's terminal bundles enter this register, and how?**
  `WatsonWBlair/Agentic_Workspace` holds the bundles that document most of the lab's current
  toolchain, a majority of them terminal. Three shapes are open: adopt wholesale, adopt
  selectively (only decisions still binding on lab-os), or leave them addressable by pointer
  into the fork and its git history. Nothing is assumed. *Trigger:* an owner decision; until
  then [spec.md](./spec.md) registers only decisions folded from lab-os's own bundles.
- **Does the register file keep the name `spec.md`?** The fork's `spec-home-migration` bundle
  renamed it `decisions.md` and that rename has not landed here. `spec.md` is the ratified
  name and the one this bundle uses. *Trigger:* Caravan backlog item
  [B7](https://github.com/CAMELS-Research-Group/Caravan/blob/main/BACKLOG.md#b7--re-vendor-when-the-bundle-decision-file-rename-lands-upstream)
  (Caravan's `B<n>` namespace, not this repo's).
- **Is the `2026-07-31-timeboxing` bundle actually still active?** Its PRD reads
  `Status: active — in review, PR #66`, but #66 is merged. If it is done, it is the first
  bundle to fold. *Trigger:* its owner's call — finality is hindsight, not an agent's read.

- **Is lab-os a workspace root, or a member repo whose main bundle belongs at `_specs/main/`?**
  `03-logging.md` reserves the `<repo>` path segment for workspace-root altitude, while this
  bundle records Caravan as the shared dev home — which would read lab-os as a member repo.
  Current resolution: keep `_specs/lab-os/main/`, matching the sibling `2026-07-31-timeboxing`
  bundle; one path convention per repo beats a split tree. *Trigger:* a ruling or a
  `03-logging.md` change that settles lab-os's altitude, or lab-os ceasing to be worked as its
  own root. Correcting it later is a cross-file rename — the recovery command in
  [spec.md](./spec.md), the `_specs/lab-os/` entry in [design.md](./design.md), and every fold
  path — so it is cheaper to decide than to drift.
