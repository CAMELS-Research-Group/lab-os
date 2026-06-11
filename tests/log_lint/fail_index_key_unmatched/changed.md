# fixture-repo — project log

Format: lab standard, `lab-rules/.claude/rules/03-logging.md`.

## Standing Decisions

- 2026-03-02 14:10 — adopt widget pipeline · #12
- 2026-02-02 09:00 — a decision that was never logged · #9
- 2026-02-01 09:30 — pin runtime to 3.11 · #8

## Entries

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
