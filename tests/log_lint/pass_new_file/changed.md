# <repo-name> — project log

Format: lab standard, `lab-rules/.claude/rules/03-logging.md`. Skeleton per
`lab-rules/templates/project_log.template.md` (normative — `log-lint` parses this structure).
The `## Standing Decisions` and `## Entries` headings are load-bearing lint anchors: exact
text, one each, never renamed. Entry headers are the only other `##` headings allowed.

## Standing Decisions

<!-- One line per still-binding decision — hot window and archive alike. This index is the
     "what is still true" surface; read it first. Line grammar (exact; — is U+2014, · is U+00B7):
       - YYYY-MM-DD HH:MM — <subject> · #<PR-or-archive-link>
     Date + subject must match the entry header verbatim — log-lint keys index lines to entry
     headers on that string. Add the line in the same PR as its decision entry; events get no
     index line. Remove a superseded line in the same PR as its superseding entry. -->

- 2026-01-01 12:34 — EXAMPLE: replace with the first real decision · #1

## Entries

<!-- Reverse-chronological, TOP-INSERT: a PR's new entries form one contiguous block at the
     head of this region, internally date-ordered (non-strict descending; same-timestamp ties
     allowed). Each entry is preceded by `---` on its own line, then a blank line.
     Merge conflict here: keep both blocks, reorder entries by header timestamp.
     Entries are immutable once their PR merges — revise via a new entry with `Supersedes:`.
     Whole file capped at 15 KB; overflow moves oldest entries to project_log_archive.md via a
     dedicated `chore: archive log overflow` PR. -->

---

## 2026-01-01 12:34 — EXAMPLE: replace with the first real decision

**Decision:** <what was decided / what happened — whole entry ≤ 1,500 bytes; detail beyond that belongs in the PR body or the spec>
**Why:** <the load-bearing rationale>
**Alternatives:** <only when real ones were weighed — what was rejected and why>
**Supersedes:** 2025-12-31 09:00 — EXAMPLE: an older decision this one replaces   <!-- only on superseding entries; the same PR removes the superseded index line -->
**Refs:** #1
