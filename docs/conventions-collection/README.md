# Conventions Collection — Team Review Draft

**Status:** Draft — gathering team feedback before authoring  
**Date:** 2026-07-16 / 2026-07-17  
**Sources:** 7 documents from 4 contributors (Jean, Watson, Kiara, Arya)  
**Interactive preview:** [preview.html](preview.html) (download and open locally, dark-theme tabbed view)  
**Findings presentation:** [findings-presentation.html](findings-presentation.html) (download and open locally, arrow keys to advance; the takeaways in seven slides)

---

## What this is

Four contributors independently reviewed their workspaces, project logs, and working patterns to surface conventions, patterns, and tools worth promoting into the Lab-OS handbook. This PR collects all seven source documents into one place so the team can:

1. **Comment on individual entries** — agree, disagree, propose rewording
2. **Flag de-duplication** — several items appear in 2–3 sources independently (strongest promote signal)
3. **Propose landing order** — some entries have dependencies; the team should agree on sequencing
4. **Decide what needs authoring vs. copying** — some items are ready-to-copy files; others need to be written from scratch

> **This PR deliberately contains no changes to rules, templates, or the handbook itself.** It is a review surface only. Promotion PRs follow after the team agrees on the list.

---

## Source documents

| # | Document | Author | Method | Entries |
|---|----------|--------|--------|---------|
| 1 | conventions-to-promote-2026-07-16 | Jean | Deep extraction (5 parallel readers on two internal application records) | 13 |
| 2 | lab-os-promotion-review (full) | Watson | Fork-vs-upstream diff + machine-wide sweep of all lab codebases | 38 |
| 3 | lab-os-promotion-review (fork-only) | Watson | Fork-only subset of #2 (entries 1–17 identical) | 17 |
| 4 | lab-os-promotion-review-jean | Jean | Fleet/CAMELS workspace review (4 parallel readers + 2nd pass) | 28 |
| 5 | claude-conventions-jean | Jean | Operator-side ("my side of the keyboard") | ~6 sections |
| 6 | steering-conventions | Kiara | Operator-side steering conventions | ~6 sections |
| 7 | arya-steering | Arya | Operator-side steering conventions | ~6 sections |

**De-duplication notes:**
- Doc #3 is a **strict subset** of Doc #2 (entries 1–17 byte-identical; machine-wide addendum 18–38 absent)
- `claude-conventions-jean.md` has an exact duplicate in Downloads (byte-identical)

---

## Convergent findings (independently derived in 2+ sources — highest-confidence promotes)

These items appeared independently across sources without either author naming the same thing. Independent reinvention is the strongest signal a convention should be ratified.

| Convention | Sources | Status |
|-----------|---------|--------|
| **Dependency seam** — every external dep behind one config-swappable interface | Jean-deep A1 | NEEDS AUTHORING |
| **Soft-degrade by default** — failure degrades to a working path, never crashes | Jean-deep A2 | NEEDS AUTHORING |
| **Fail-fast config guards** — invariants as hard boot errors | Jean-deep B1 | NEEDS AUTHORING |
| **Mocked tests ≠ live verification** — green suite proves wiring, not live value | Jean-deep C1, Jean-workspace #1, Watson #21 | SHARPEN existing |
| **Evidence over assertion** — probe primary source, distrust even your own claims | Jean-deep C3, Jean-workspace #3 | SHARPEN existing |
| **"Done" means pushed** — commit + push together, verify the push landed | Jean-deep D1, Watson #5 | NEEDS AUTHORING |
| **One-writer-per-tree** — each tree has exactly one owner; cross-tree = handoff | Jean-workspace #10 | NEEDS AUTHORING (at lab altitude) |
| **4-part status summary** — Verified / Not yet verified / Caveats / Next step | Jean-workspace #21 | NEEDS AUTHORING |

---

## All entries by theme

### Theme A — Structural Resilience & Engineering

| ID | Entry | Tag | Source |
|----|-------|-----|--------|
| A1 | Dependency seam — every external dep behind one config-swappable interface | NEEDS AUTHORING | Jean-deep |
| A2 | Soft-degrade by default — failure degrades, never crashes | NEEDS AUTHORING | Jean-deep |
| B1 | Fail-fast config guards — invariants as hard boot errors | NEEDS AUTHORING | Jean-deep |
| B2 | Deletion completeness — real delete purges all stores + external identity | NEEDS AUTHORING | Jean-deep |
| B3 | LLM provider choice is a data-protection decision | NEEDS AUTHORING | Jean-deep |
| J5 | Structural-invariant-over-promise | NEEDS AUTHORING | Jean-workspace |
| J13 | Enforce-at-authoring-time — guards over sweeps | NEEDS AUTHORING | Jean-workspace |
| J14 | Multi-writer JSON-store hardening contract | NEEDS AUTHORING | Jean-workspace |
| J15 | Hardcode-then-parameterize | NEEDS AUTHORING | Jean-workspace |
| J16 | Local-LLM structured-JSON output contract | NEEDS AUTHORING | Jean-workspace |

### Theme B — Verification & Honesty

| ID | Entry | Tag | Source |
|----|-------|-----|--------|
| C1 | Mocked tests are green-but-meaningless for the live path | SHARPEN | Jean-deep + workspace |
| C2 | Mutation-prove a lock/regression test actually "bites" | SHARPEN | Jean-deep |
| C3 | Evidence over assertion — including the agent's own claims | SHARPEN | Jean-deep + workspace |
| J1 | Watched-live bar — green tests are not the greenlight | NEEDS AUTHORING | Jean-workspace |
| J2 | Adversarial pre-ship second-set-of-eyes | NEEDS AUTHORING | Jean-workspace |
| J4 | Honesty labels — ship the real, label the rest | PARTIAL | Jean-workspace |
| J16b | Local-vs-hosted model selection rule | NEEDS AUTHORING | Jean-workspace |
| J16c | A review loop cannot audit taste | NEEDS AUTHORING | Jean-workspace |

### Theme C — Gates & Authority

| ID | Entry | Tag | Source |
|----|-------|-----|--------|
| D1 | "Done" means pushed — commit-and-push together | NEEDS AUTHORING | Jean-deep + Watson |
| J6 | Machine-boundary per-action gate / local-private default | PARTIAL | Jean-workspace |
| J7 | Agent authority ladder | NEEDS AUTHORING | Jean-workspace |
| J8 | Gated-builder contract | COPY | Jean-workspace |
| J9 | Subscription-token automation boundary | NEEDS AUTHORING | Jean-workspace |

### Theme D — Coordination

| ID | Entry | Tag | Source |
|----|-------|-----|--------|
| J10 | One-writer-per-tree / one-owner-per-concern | NEEDS AUTHORING | Jean-workspace |
| J11 | Pull-not-push + "suggested owner ≠ claimed owner" | NEEDS AUTHORING | Jean-workspace |
| J12 | Hub-not-mesh manager charter | NEEDS AUTHORING | Jean-workspace |
| W15 | Fork-and-own extraction posture | NEEDS AUTHORING | Watson |
| W16 | Gate-sitting pattern for autonomous runs | NEEDS AUTHORING | Watson |

### Theme E — Skills, Tools & Starters

| ID | Entry | Tag | Source |
|----|-------|-----|--------|
| W1 | Agent-runtime guardrail rule (rule 05) | COPY | Watson |
| W6 | pr-review-loop skill + `/pr-review-loop` | COPY | Watson |
| W7 | logging-automation skill + `/log` | COPY | Watson |
| W8 | Code-quality taxonomy + planning/execution pair | COPY | Watson |
| W9 | prompt-optimization skill + `/optimize-prompt` | COPY | Watson |
| W10 | Vendored engineering-discipline skills (8, MIT) | COPY | Watson |
| W11 | slop-review skill | COPY | Watson |
| W12 | context-gc plugin | COPY | Watson |
| W13 | Reusable workflow scripts + tracking convention | COPY | Watson |
| W17 | Overnight handoff brief convention | NEEDS AUTHORING | Watson |
| W38 | Edge/service starter (hardware-flexible inference) | COPY | Watson |

### Theme F — CI, Release & Security

| ID | Entry | Tag | Source |
|----|-------|-----|--------|
| W18 | Bot-identity subsystem (short-lived token mint, OS-level secret wrapping) | COPY | Watson |
| W19 | Comms-authorization tier policy | COPY | Watson |
| W21 | Local verification-gate convention (`invoke check`) | COPY | Watson |
| W22 | Hygiene gates: commit hook + PR-scoped quality CI | COPY | Watson |
| W24 | Fixture-lock drift-guard gate | COPY | Watson |
| W25 | Architectural-fitness test: import-boundary guard | COPY | Watson |
| W26 | Gated, digest-verified release template | COPY | Watson |

### Theme G — Data & Reproducibility

| ID | Entry | Tag | Source |
|----|-------|-----|--------|
| W27 | Training-run reproducibility conventions | COPY | Watson |
| W28 | Idempotent data-wrangling contract | COPY | Watson |
| W29 | Checksummed data/model manifest + verified setup | COPY | Watson |
| J17 | Sensitive-values single-store contract | COPY | Jean-workspace |
| J18 | Provenance / attestation standard | COPY | Jean-workspace |

### Theme H — Docs, Templates & Formats

| ID | Entry | Tag | Source |
|----|-------|-----|--------|
| W2 | Spec-log altitude + dated planning bundles + lifecycle | COPY | Watson |
| W3 | GLOSSARY.md as a first-read doc type | COPY | Watson |
| W4 | Cross-platform peer-clone convention | COPY | Watson |
| W30 | Evidence-first status + demo-honesty doc conventions | COPY | Watson |
| W31 | Integration-contract doc type | COPY | Watson |
| W33 | Cost-tracking ledger format | COPY | Watson |
| J21 | 4-part status-summary format | NEEDS AUTHORING | Jean-workspace |
| J25 | Adversarial verdict-doc format | COPY | Jean-workspace |
| J26 | Prompt-quality bar for agent-emitted work items | NEEDS AUTHORING | Jean-workspace |
| J24 | Journaling-compliance staleness check | NEEDS AUTHORING | Jean-workspace |

### Operator-Side Conventions (Jean + Kiara)

These are "my side of the keyboard" — how the human steers the agent. Not handbook rules, but shared vocabulary worth documenting.

**Jean's patterns:** "do it for me" / "don't act, tell me" mode-switching · "go" / "yes" per-action authorization · "make me a prompt for FM" delegation · "ask me one question at a time" · "why are you celebrating" honesty pushback · /roundtable, /broadcast, /fleet, /compact rituals

**Kiara's patterns:** "resume" / "autonomous, task-by-task" mode-switching · "go" + rider ("stop only for genuine forks") · "consider that a standing instruction" promotion · "hold on" wrong-assumption interrupt · "confirm with evidence" / "tell me what correct looks like" honesty · full sprint-close verification ritual

---

## Already upstream — do NOT re-promote

These are ratified in lab-os already. Cite the `.claude/rules/0x-*` file as canonical.

- SDD lifecycle, PRD/Design-doc/Plan formats, Verification checkpoint, Review checkpoint
- Build = delegate to subagents, Autonomous-loop safety contract
- Merge bar + PR lifecycle + commits, Logging standard, Docs standard
- Data protection, Operating philosophy
- White-label desktop starter (PR #41), CLAUDE.local.md overlay (issue #53)
- Rules 01-04 (byte-identical via symlink), 07-network-boundaries (promoted from Jean's workspace)
- Overclaim-scrub / "gate green, run unpiped"

---

## How to comment

1. **Agree with an entry** — react with 👍 or leave a short "+1 — we use this too"
2. **Disagree or flag a concern** — comment on the specific line with your reasoning
3. **Propose a merge** — if two entries are the same thing, comment "merge with [ID]"
4. **Suggest landing order** — Watson proposed: W2 → W3 → W7 → W10 → everything else
5. **Claim authorship** — if you want to write the rule file for a NEEDS AUTHORING entry, comment "I'll author this"

---

## Suggested landing order (Watson's dependency analysis)

```
W2 (spec-log/bundles) → W3 (GLOSSARY) → W7 (/log) → W10 (engineering skills, depends on W3 + W7) → everything else in any order
```

Entries W1, W5, W6, W8, W9, W11, W12, W13, W14 are independent.  
Entries W15–W17 are authoring tasks with no file dependency.  
W17 reads best landed after W6 (references pr-review-loop as the quality bar).

---

*This document was assembled from the source reviews by Cursor Fleet Agent on 2026-07-17. No changes to rules, templates, or the handbook were made.*
