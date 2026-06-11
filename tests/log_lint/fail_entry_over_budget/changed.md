# fixture-repo — project log

Format: lab standard, `lab-rules/.claude/rules/03-logging.md`.

## Standing Decisions

- 2026-04-06 10:00 — adopt long-winded persistence layer · #24
- 2026-03-02 14:10 — adopt widget pipeline · #12
- 2026-02-01 09:30 — pin runtime to 3.11 · #8

## Entries

---

## 2026-04-06 10:00 — adopt long-winded persistence layer

**Decision:** Adopt the long-winded persistence layer for every artifact the lab produces, including the intermediate embeddings, the consolidated training manifests, the per-run evaluation summaries, the cross-validation fold assignments, the calibration curves, the ablation grids, and the assorted bookkeeping files that accumulate around each experiment as it moves from a notebook sketch to a reproducible pipeline stage with pinned dependencies and recorded seeds.
**Why:** The previous storage arrangement scattered artifacts across machine-local scratch directories, two different cloud buckets with inconsistent prefixes, and a handful of external drives, which made every reproduction attempt begin with an archaeology session. Consolidating behind one interface removes that recurring cost, and the verbose configuration surface, while heavy, is at least explicit about retention, encryption, and lifecycle rules for each artifact class, which the lab's data-protection rules require us to be able to demonstrate on demand to any collaborator or reviewer who asks.
**Alternatives:** Keeping the scratch-directory convention with a nightly sync script — rejected because the sync script itself became a source of silent divergence whenever a machine was offline during its window. Adopting the minimal key-value store — rejected because it pushed retention policy into application code, exactly where it kept being forgotten, and because its access-control story amounted to a single shared credential that everyone would inevitably paste into a notebook someday.
**Refs:** #24

---

## 2026-03-02 14:10 — adopt widget pipeline

**Decision:** Use the widget pipeline for ingest.
**Why:** Lowest maintenance for a one-person lab.
**Alternatives:** Hand-rolled ETL — rejected, too much upkeep.
**Refs:** #12

---

## 2026-02-01 09:30 — pin runtime to 3.11

**Decision:** Pin CI runtime to Python 3.11.
**Why:** Matches the oldest supported GitHub runner default.
**Refs:** #8
