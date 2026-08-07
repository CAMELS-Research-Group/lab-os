# <Slice title> — spec log

<!-- Bundle spec-log altitude — the single per-bundle log. A FLAT CHRONOLOGICAL STREAM of
     entries in the canonical Entry format (.claude/rules/03-logging.md §Entry format). One
     stream, no sections: load-bearing decisions (with alternatives weighed), discarded design
     detail worth surviving, plan deviations, implementation-altitude calls, and gate evidence
     are all ordinary entries here. plan.md carries no Execution Log; that detail lands here as
     entries like any other.

     Order: oldest-first, BOTTOM-INSERT (03-logging.md §File structure & overflow) — contrast
     project_log.md, which is reverse-chron, top-insert.

     Relaxations at this altitude: no Standing Decisions index, no cap enforcement, and the
     per-entry byte cap does not bind; Refs may carry a bundle-relative path or be omitted
     until the PR lands. Overflow goes to log_archive.md co-located in this bundle.

     Routing (owned by .claude/rules/04-docs.md §ENG document standards, restated here at
     point of use): known gaps are NOT entries here — they live in the PRD's "Open questions". A
     decision's binding current-state resolution lives once in this bundle's spec.md; this log
     is the history of how it got there. Decisions that outlive the slice → project_log.md.

     Spec: .claude/rules/03-logging.md -->

**Date:** YYYY-MM-DD · **Repo:** <repo> · **Slice:** <slug>

---

## YYYY-MM-DD HH:MM — <subject, one line>

**Decision:** <what was decided or happened>
**Why:** <load-bearing rationale — not a restatement of the decision>
**Alternatives:** <only when real ones were weighed — "considered X, chose Y because Z">
**Supersedes:** <YYYY-MM-DD HH:MM — subject> <!-- superseding entries only -->
**Refs:** <!-- #PR, a bundle-relative path, or omit until the PR lands -->

---

## YYYY-MM-DD HH:MM — <subject, one line>

**Decision:**
**Why:**
**Refs:**

---

<!-- Append the next entry BELOW this line, oldest-first / BOTTOM-INSERT. Delete any
     placeholder entry left unfilled. -->
