## Index

| id | title | owner | size | status |
|---|---|---|---|---|
| B1 | Done thing | Kiara | S | done |
| B2 | Orphan ready | — | M | ready |
| B3 | Working | Watson | M | in-progress |
| B4 | Big one | Arya | L | inbox |
| B5 | Ready unblocked | Kiara | S | ready |
| B6 | Ready blocked | Jean | S | ready |

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

## B2 — Orphan ready

- **Problem:** nobody owns it
- **Who it helps:** groomers
- **Value:** visibility
- **Owner:** —
- **Rough size:** M
- **Done when:** `docs/b.md` merged
- **Depends on:** —
- **Status:** ready

## B3 — Working

- **Problem:** in flight
- **Who it helps:** the team
- **Value:** steady
- **Owner:** Watson
- **Rough size:** M
- **Done when:** `scripts/c.py --check` passes
- **Depends on:** —
- **Status:** in-progress

## B4 — Big one

- **Problem:** too large
- **Who it helps:** future us
- **Value:** big payoff
- **Owner:** Arya
- **Rough size:** L
- **Done when:** `docs/d.md` split into items
- **Depends on:** —
- **Status:** inbox

## B5 — Ready unblocked

- **Problem:** next up
- **Who it helps:** the team
- **Value:** quick win
- **Owner:** Kiara
- **Rough size:** S
- **Done when:** `scripts/e.py` lands
- **Depends on:** B1
- **Status:** ready

## B6 — Ready blocked

- **Problem:** waiting on B3
- **Who it helps:** the team
- **Value:** sequenced
- **Owner:** Jean
- **Rough size:** S
- **Done when:** `docs/f.md` lands
- **Depends on:** B3
- **Status:** ready
