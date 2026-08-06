## Index

| id | title | owner | size | status |
|---|---|---|---|---|
| B1 | Done thing | Kiara | S | done |
| B2 | Ready thing | Watson | M | ready |
| B3 | Blocked thing | Arya | S | ready |

## Items

## B1 — Done thing

- **Problem:** was broken
- **Who it helps:** the team
- **Value:** shipped value
- **Owner:** Kiara
- **Rough size:** S
- **Done when:** `scripts/a.py` exists
- **Depends on:** —
- **Status:** done

## B2 — Ready thing

- **Problem:** next up
- **Who it helps:** the team
- **Value:** quick win
- **Owner:** Watson
- **Rough size:** M
- **Done when:** `scripts/b.py` lands
- **Depends on:** B1
- **Status:** ready

## B3 — Blocked thing

- **Problem:** waiting on B2
- **Who it helps:** the team
- **Value:** sequenced
- **Owner:** Arya
- **Rough size:** S
- **Done when:** `docs/c.md` lands
- **Depends on:** B2
- **Status:** ready
