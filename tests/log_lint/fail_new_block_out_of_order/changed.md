# fixture-repo — project log

Format: lab standard, `lab-rules/.claude/rules/03-logging.md`.

## Standing Decisions

- 2026-03-02 14:10 — adopt widget pipeline · #12
- 2026-02-01 09:30 — pin runtime to 3.11 · #8

## Entries

---

## 2026-04-04 16:20 — cut v0.3 release

**Decision:** Tagged and published v0.3.
**Why:** Milestone agreed for the April check-in.
**Refs:** #19

---

## 2026-04-05 11:00 — newer entry placed below an older one

**Decision:** The new head block is ascending instead of descending.
**Why:** Exercises the non-strict descending order rule.
**Refs:** #21

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
