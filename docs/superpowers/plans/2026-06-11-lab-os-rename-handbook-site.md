# lab-os Rename + Handbook Site Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Track progress via each task's `Commit:` subject — git is authoritative.

**Goal:** Rename the repo to `lab-os`, stand up a Docusaurus handbook site as the human-facing front door at `watsonwblair.github.io/lab-os`, and ship the play-test kit so 1–3 lab members can run the full onboarding arc.

**Architecture:** Docusaurus 3 (classic preset, TypeScript config) lives in `site/` inside this repo; a GitHub Action builds on PRs touching `site/**` and deploys to GitHub Pages on push to main. The site becomes the owning home for human-facing docs (D3); root markdown becomes pointer stubs; `.claude/rules/` stays AI-tier and untouched. The Docusaurus build's broken-link check is the link-rot gate.

**Tech Stack:** Docusaurus 3, Node 20+, GitHub Pages, GitHub Actions. Existing lab CI (`docs-budget`, `log-lint`, `merge-bar-check`) unchanged.

**Spec** (read before claiming any task): [2026-06-11-lab-os-rename-handbook-site-design.md](../specs/2026-06-11-lab-os-rename-handbook-site-design.md)

## How to consume a task in this plan

- **Files** lists exact paths to create/modify. Stick to them.
- **Depends on** lists task numbers that must complete first.
- **Spec** links the design-doc section that governs the task. Read it before starting.
- **Acceptance** is the bulleted set of behaviors/properties the result must demonstrate. The behaviors *are* the review surface; you choose wording, page structure, and exact phrasing.
- **Verification** is the exact command(s) that must pass before opening the PR.
- **Commit** is the conventional-commit subject for the PR title.

Content tasks (Phase 3) produce prose, not code: "Acceptance" bullets there describe what a reader must be able to do or find. If an acceptance bullet is ambiguous against the spec, ask rather than guess.

**Not in this plan**: friction fixes and structural rework are written *during* the play-test window from real friction data; the play-test-launch log entry is written when Watson invites testers.

**Execution:** subagent-driven-development (confirmed by Watson on PR #12, 2026-06-11). Tester launch is gated on Task 16 — testers read scrubbed pages.

## Phase structure

| Phase | Theme | Day | Tasks |
|---|---|---|---|
| 1 | Rename | 1 | 1–3 |
| 2 | Site infrastructure | 1 | 4–5 |
| 3 | Content MVP | 1–2 | 6–13 |
| 4 | Play-test kit | 2 | 14 |
| 5 | Stakeholder polish | 2–3 | 15–16 |

PR grouping per the merge bar (single concern each): Task 2 is one PR; Tasks 4–5 one PR; Tasks 6–12 land as one content PR or a small number of page-group PRs at the executor's discretion (each page is independently reviewable); Task 13 one PR; Task 14 one PR. Tasks 1 and 3 are operational — no commits.

## File structure (new/changed surfaces)

```
site/
├── package.json / package-lock.json
├── docusaurus.config.ts          # baseUrl /lab-os/, org WatsonWBlair
├── sidebars.ts
├── src/pages/index.md            # landing (Task 6)
└── docs/
    ├── getting-started.md        # Task 7 (absorbs BOOTSTRAP.md)
    ├── working-with-claude.md    # Task 8 (absorbs WORKING-WITH-CLAUDE.md)
    ├── onboarding-project.md     # Task 9 (absorbs orphaned ONBOARDING-PROJECT.md)
    ├── rules-explained.md        # Task 10
    ├── repo-setup.md             # Task 11
    ├── play-testing.md           # Task 12
    └── tooling-tour.md           # Task 15
.github/workflows/deploy-site.yml # Task 5
.github/ISSUE_TEMPLATE/friction.yml # Task 14
README.md / BOOTSTRAP.md / WORKING-WITH-CLAUDE.md  # slimmed/stubbed (Task 13)
```

---

# Phase 1 — Rename

## Task 1: Rename the repo and re-point local infrastructure

**Context:** Operational, no commit. Shared-state change (repo settings) — approved via the signed-off spec; do the junction step between Cowork sessions because a stale junction silently drops rule-loading.

**Files:** none (GitHub settings + local filesystem)

**Depends on:** (none — first task)

**Spec:** [§3 Rename mechanics, steps 1–4](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#3-rename-mechanics)

**Acceptance:**
- GitHub repo is `WatsonWBlair/lab-os`; old `lab-rules` URLs redirect
- Local folder is `C:\Users\watso\Development\lab-os`
- Junction `C:\Users\watso\Development\.claude\rules` resolves to `Development\lab-os\.claude\rules` and lists the four rule files
- Local git remote points at the canonical `lab-os` URL; fetch works

**Verification:**
```powershell
gh repo view WatsonWBlair/lab-os --json name
Get-ChildItem C:\Users\watso\Development\.claude\rules   # four 0x-*.md files
git -C C:\Users\watso\Development\lab-os remote -v; git -C C:\Users\watso\Development\lab-os fetch
```

**Commit:** none (operational)

## Task 2: In-repo reference sweep + rename log entry

**Files:**
- Modify: `README.md`, `BOOTSTRAP.md`, `WORKING-WITH-CLAUDE.md`, `PR-LIFECYCLE.md`
- Modify: `.claude/rules/01-workflow.md` … `04-docs.md` (only where they self-reference the repo name)
- Modify: `templates/dev-root-CLAUDE.template.md`, `templates/global-CLAUDE.template.md`, `templates/repo-CLAUDE.template.md`, `templates/project_log.template.md` (whichever name lab-rules)
- Modify: `project_log.md` (new event entry; events get no Standing Decisions index line)
- Modify: `C:\Users\watso\Development\project_log.md` (lab-altitude event entry — outside this repo, no commit, honor-system rules apply)

**Depends on:** 1

**Spec:** [§3 steps 5 + logging](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#3-rename-mechanics)

**Acceptance:**
- No live doc in the repo refers to `lab-rules` as the current name; historical artifacts (existing specs/plans, merged log entries, archived content) stay as written
- Junction/checkout snippets in README and BOOTSTRAP use the new paths and still work copy-pasted
- Repo log entry records the rename as an irreversible/external event (Why: name described one slice; phase 2 makes renaming expensive later); lab log gets the cross-repo counterpart with absolute-path refs
- `git grep -niI "lab-rules"` hits only historical artifacts (docs/superpowers/**, project_log.md pre-existing entries) — list any other hit and justify or fix

**Verification:**
```powershell
git grep -niI "lab-rules" -- ':!docs/superpowers' ':!project_log.md'   # expect: no output
python scripts/log_lint.py --base main; python scripts/docs_budget.py --root .
```

**Commit:** `docs: sweep lab-rules references to lab-os`

## Task 3: Update live session-steering files

**Context:** Operational, no commit — these files live outside the repo but steer every Cowork session; same-day per spec.

**Files:**
- Modify: `C:\Users\watso\.claude\CLAUDE.md` (lab-tooling bullet, doc-update-triggers pointer)
- Modify: `C:\Users\watso\Development\.claude\CLAUDE.md` (lab-tooling section, repo table)
- Modify: Claude memory files referencing lab-rules under `C:\Users\watso\.claude\projects\C--Users-watso-Development\memory\` (and the lab-rules project memory dir)

**Depends on:** 1

**Spec:** [§3 step 6](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#3-rename-mechanics)

**Acceptance:**
- Both CLAUDE.md files name `lab-os` with the new path; junction description matches Task 1 reality
- Memory files mentioning the repo by name are updated; no memory recommends a dead path

**Verification:**
```powershell
Select-String -Path C:\Users\watso\.claude\CLAUDE.md,C:\Users\watso\Development\.claude\CLAUDE.md -Pattern "lab-rules"   # expect: no output
```

**Commit:** none (files outside repo)

---

# Phase 2 — Site infrastructure

## Task 4: Scaffold the Docusaurus site

**Files:**
- Create: `site/` via `create-docusaurus` (classic preset, TypeScript)
- Modify: `.gitignore` (add `site/node_modules/`, `site/build/`, `site/.docusaurus/`)

**Depends on:** 1 (baseUrl bakes in the new name)

**Spec:** [§4 Site architecture](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#4-site-architecture)

**Acceptance:**
- `docusaurus.config.ts`: `url` `https://watsonwblair.github.io`, `baseUrl` `/lab-os/`, `organizationName` `WatsonWBlair`, `projectName` `lab-os`, `onBrokenLinks: 'throw'`
- Blog plugin disabled; scaffold boilerplate pages/images that won't be reused are removed
- Production build succeeds from a clean install; no scaffold-default "Docusaurus" branding remains in navbar/footer/title

**Verification:**
```powershell
cd site; npm ci; npm run build   # exit 0, no broken-link errors
```

**Commit:** `feat(site): scaffold docusaurus handbook site`

## Task 5: Build + Pages deploy workflow

**Context:** Enabling GitHub Pages (Settings → Pages → GitHub Actions source) is a one-time shared-state change — covered by the signed-off spec, done by Watson or with him present.

**Files:**
- Create: `.github/workflows/deploy-site.yml`

**Depends on:** 4

**Spec:** [§4 Site architecture](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#4-site-architecture)

**Acceptance:**
- On `pull_request` touching `site/**`: build-only job (the link-check gate); on `push` to main touching `site/**`: build + deploy to GitHub Pages via the official `actions/deploy-pages` flow with the required permissions and a concurrency group
- Site responds 200 at `https://watsonwblair.github.io/lab-os/` after the first main deploy
- Workflow does not trigger on non-site changes

**Verification:**
```powershell
gh run list --workflow deploy-site.yml --limit 3   # build green on PR, deploy green on main
Invoke-WebRequest https://watsonwblair.github.io/lab-os/ -Method Head   # 200
```

**Commit:** `ci: build and deploy handbook site to github pages`

---

# Phase 3 — Content MVP

## Task 6: Landing page

**Files:**
- Create: `site/src/pages/index.md` (replacing scaffold homepage)

**Depends on:** 4

**Spec:** [§5 IA — Landing](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#5-information-architecture-mvp)

**Acceptance:**
- A reader with zero lab context leaves knowing: what the CAMELS Research Group is (one short paragraph, public-tier — no codenames, no overclaims per 04-docs), what lab-os is (the lab's operating system: conventions, onboarding, tooling), and what to do next
- Single prominent "Start here" call-to-action linking to Getting Started
- Written fresh — no text lifted from AI-tier docs

**Verification:**
```powershell
cd site; npm run build
```

**Commit:** `feat(site): landing page`

## Task 7: Getting Started — Claude-guided setup

**Files:**
- Create: `site/docs/getting-started.md`
- Source: `BOOTSTRAP.md` (content absorbed; stub happens in Task 13)

**Depends on:** 4

**Spec:** [§5 IA — Getting Started](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#5-information-architecture-mvp), [D6](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#2-decisions)

**Acceptance:**
- Page spine is the guided flow: prerequisites (accounts, installs, Claude subscription) → install Claude Code → paste the bootstrap prompt → verify the result
- The bootstrap prompt is a single copy-paste block that instructs Claude to: interview the member and personalize `templates/global-CLAUDE.template.md` into their `~/.claude/CLAUDE.md`; seed the dev-root CLAUDE.md from its template; clone the core repos; wire the rules junction (Windows + macOS/Linux variants); and run a verification pass at the end. The prompt references the templates by repo URL — it never inlines template content (single-source, §6)
- BOOTSTRAP.md's step-by-step survives as a manual-path section for members who prefer doing it by hand; cross-platform coverage is not lost in migration
- A "verify your setup" section gives the member concrete success checks (rules visible in a session, junction resolves, templates found)

**Verification:**
```powershell
cd site; npm run build
```

**Commit:** `feat(site): getting started with claude-guided setup`

## Task 8: Working with Claude — methods page

**Files:**
- Create: `site/docs/working-with-claude.md`
- Source: `WORKING-WITH-CLAUDE.md`

**Depends on:** 4

**Spec:** [§5 IA — Working with Claude](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#5-information-architecture-mvp)

**Acceptance:**
- Content parity with the source doc modulo human-tier edits (headings, intro framing, link fixes)
- Insider-reference scan applied (§10): no reference assumes access to private docs or unexplained lab history; anything load-bearing gets one sentence of context or a link
- Where the doc restates rule content, it links the owning rule file ("source of truth: `01-workflow.md`")

**Verification:**
```powershell
cd site; npm run build
```

**Commit:** `feat(site): working-with-claude methods page`

## Task 9: Onboarding project page (rescue from orphaned branch)

**Context:** `ONBOARDING-PROJECT.md` exists only on the unmerged `docs/onboarding-project` branch; its content moves straight into the site and never lands at repo root. Branch deletion at the end is destructive — get Watson's explicit OK before deleting.

**Files:**
- Create: `site/docs/onboarding-project.md`
- Source: `git show docs/onboarding-project:ONBOARDING-PROJECT.md`

**Depends on:** 4

**Spec:** [§5 IA — Onboarding Project](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#5-information-architecture-mvp), [§8 risks](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#8-sequencing)

**Acceptance:**
- Full ONBOARDING-PROJECT.md content migrated (capability checklist, fixed workflow spine, stack-as-discovery-surface, deployment-tradeoff writeup requirement, 2-week timebox)
- The branch's README and project_log deltas are *not* brought over (superseded by this initiative)
- Page links the repo-setup runbook (Task 11) as the project's first step
- After the content PR merges: orphan branch deleted with Watson's explicit confirmation

**Verification:**
```powershell
cd site; npm run build
git diff --stat docs/onboarding-project -- ONBOARDING-PROJECT.md   # reviewer cross-checks content parity
```

**Commit:** `feat(site): onboarding project page`

## Task 10: Rules, explained

**Files:**
- Create: `site/docs/rules-explained.md`

**Depends on:** 4

**Spec:** [§5 IA — Rules explained](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#5-information-architecture-mvp), [§6 single-source map](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#6-single-source-map-per-04-docs)

**Acceptance:**
- One short section per rule file (01-workflow, 02-data-protection, 03-logging, 04-docs): what it governs, why it exists, the one or two things a new member most needs to internalize
- Every section names and links its rule file as source of truth; restates generalized, never copies rule text verbatim (robust to rule edits per 04-docs)
- A reader can answer: "where do commit conventions live, what can I never commit, when do I write a log entry, which doc owns a fact"

**Verification:**
```powershell
cd site; npm run build
```

**Commit:** `feat(site): rules-explained tour`

## Task 11: Repo-setup runbook

**Files:**
- Create: `site/docs/repo-setup.md`

**Depends on:** 4

**Spec:** [§5 IA — Setting up a new repo](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#5-information-architecture-mvp)

**Acceptance:**
- First-cut checklist for creating a lab-conformant repo: create repo → `.gitignore` basics → seed `CLAUDE.md` from `templates/repo-CLAUDE.template.md` → seed `project_log.md` from `templates/project_log.template.md` → PR template → caller YAML pointer marked "phase 2 — not yet required"
- Each step links its template/source rather than inlining content
- Framed as the sandbox project's first step (the play-test exercises it); explicitly labeled first-cut, expected to grow from friction data

**Verification:**
```powershell
cd site; npm run build
```

**Commit:** `feat(site): repo-setup runbook`

## Task 12: How to play-test

**Files:**
- Create: `site/docs/play-testing.md`

**Depends on:** 4

**Spec:** [§7 Play-test protocol](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#7-play-test-protocol)

**Acceptance:**
- States the arc (landing → guided setup → methods → sandbox-project brainstorm/spec) and the success bar (working environment incl. personalized global CLAUDE.md + a sandbox-project spec, unassisted)
- The >15-minute stall rule: file a friction issue first, then ask Watson; links the friction template
- End-of-arc retro prompt: what confused, what was missing, what you'd cut

**Verification:**
```powershell
cd site; npm run build
```

**Commit:** `feat(site): play-test guide`

## Task 13: Pointer stubs, README slim-down, nav, decision log entry

**Files:**
- Modify: `BOOTSTRAP.md`, `WORKING-WITH-CLAUDE.md` (→ ≤10-line pointer stubs to their site pages)
- Modify: `README.md` (AI/CI-consumer reference + prominent site link; "New to the lab? → site")
- Modify: `site/sidebars.ts` (nav order: Getting Started → Working with Claude → Onboarding Project → Rules → Repo Setup → Play-testing)
- Modify: `project_log.md` (standing decision: site owns human-facing docs — entry + Standing Decisions index line)

**Depends on:** 7, 8, 9, 10, 11, 12

**Spec:** [§5 after-migration](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#5-information-architecture-mvp), [§6](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#6-single-source-map-per-04-docs)

**Acceptance:**
- Stubs state where the content went and why, link the site page, and contain no content of their own
- README retains: junction setup, caller-YAML consumption pattern, override semantics, scope discipline; loses migrated human-facing narrative
- Log entry records D3 as a standing decision (Why + rejected alternatives from the spec); index line added, header/date match verbatim
- All budgets green after the slim-down

**Verification:**
```powershell
python scripts/docs_budget.py --root .; python scripts/log_lint.py --base main
cd site; npm run build
```

**Commit:** `docs: root docs become pointers to the handbook site`

---

# Phase 4 — Play-test kit

## Task 14: Friction issue template + label

**Files:**
- Create: `.github/ISSUE_TEMPLATE/friction.yml`

**Depends on:** (none — independent of the site)

**Spec:** [§7 Play-test protocol](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#7-play-test-protocol)

**Acceptance:**
- GitHub issue form with fields: where I was (page/step) · what I expected · what happened · severity (dropdown); auto-applies the `playtest` label
- `playtest` label exists on the repo
- Opening a new issue on GitHub shows the form rendered (malformed YAML falls back to blank issue — that's the failure signal)

**Verification:**
```powershell
gh label list --repo WatsonWBlair/lab-os | Select-String playtest
gh api repos/WatsonWBlair/lab-os/contents/.github/ISSUE_TEMPLATE/friction.yml -q .name   # exists on main after merge
```

**Commit:** `chore: add play-test friction issue template`

---

# Phase 5 — Stakeholder polish (gates tester launch)

## Task 15: Tooling tour page

**Files:**
- Create: `site/docs/tooling-tour.md`
- Modify: `site/sidebars.ts` (append after Play-testing)

**Depends on:** 4

**Spec:** [§5 IA — Tooling tour](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#5-information-architecture-mvp)

**Acceptance:**
- Covers, one section each: the three adherence Actions (what each enforces, where it runs, what a red check means for a contributor); the `templates/` directory (what each template seeds); the pr-review agent (what it does, that it posts via bot identity, where its SPEC lives); the lab-claude-plugins marketplace (install one-liner)
- Written for both stakeholders ("what infrastructure does this lab run") and new members ("what will act on my PRs"); every section links its source of truth rather than restating mechanics
- No section promises behavior the tooling doesn't have (overclaim scrub applies here too)

**Verification:**
```powershell
cd site; npm run build
```

**Commit:** `feat(site): tooling tour page`

## Task 16: Public-tier jargon scrub (launch gate)

**Context:** Editorial pass only — wording, defined-at-first-use, overclaims. Structural rework deliberately waits for play-test friction data.

**Files:**
- Modify: `site/src/pages/index.md`, `site/docs/*.md` (all pages from Tasks 6–12 and 15)

**Depends on:** 6, 7, 8, 9, 10, 11, 12, 15

**Spec:** [§5](../specs/2026-06-11-lab-os-rename-handbook-site-design.md#5-information-architecture-mvp), 04-docs public-tier standard

**Acceptance:**
- Every page passes the 04-docs public-tier bar: jargon defined at first use or removed; no internal codenames without introduction; overclaim-scrubbed; no page depends on private-doc access to make sense
- Outsider's-eye read (per Watson's review standard): a reader with zero lab context can follow every page; anything that required insider knowledge is fixed or cut, not excused
- No structural changes (page splits, nav reordering, new sections) — those wait for friction data

**Verification:**
```powershell
cd site; npm run build
```

**Commit:** `docs(site): public-tier jargon scrub`

---

## Execution Log

<!-- Plan deviations, implementation calls, gate evidence. Archives with the plan. -->

- 2026-06-11 18:30 · task 1 · GitHub rename, remote re-point, and junction recreate done in-session. Local folder rename blocked: the session process itself holds the `lab-rules` handle (the plan's between-sessions caveat, hit from inside). Deferred to post-session handoff; junction temporarily targets the `lab-rules` path and gets recreated against `lab-os` together with the folder rename.
- 2026-06-11 18:50 · task 2 (deviation) · Sweep scope exceeded the plan's file list: the three reusable workflows carry functional self-repo gates (`$GITHUB_REPOSITORY = "WatsonWBlair/lab-rules"` decides which copy of each lint script runs) plus `repository:`/`path:` checkout refs — broken by the rename, swept in a dedicated `ci:` commit. Test-fixture sweep verified: `log_lint --self-test` 20/20, `merge_bar_check --self-test` 9/9, `docs_budget --self-test` all checks passed, `docs_budget --root .` all OK.
- 2026-06-11 · task 1 (gate evidence) · Post-session manual steps verified next session: folder renamed to `Development\lab-os` (`lab-rules` gone), junction `Development\.claude\rules` resolves to `lab-os\.claude\rules` and lists the four rule files, remote at `lab-os` URL, fetch clean. Task 1 acceptance fully met.
- 2026-06-11 19:10 · task 4 (gate evidence + review fixes) · Docusaurus 3.10.1 scaffold on `feat/site-infra`; `npm ci; npm run build` exit 0. Spec review: compliant; scaffold logo/favicon knowingly kept for a later branding pass. Quality review fixes applied: scaffold README → npm dev notes; `future.faster: true` enabled (the dep was installed but inert — note: 3.10 renamed `experimental_faster` → `faster`); scaffold `deploy` script dropped (conflicts with the Actions deploy path).
- 2026-06-11 19:10 · task 5 (gate evidence + deviations) · `deploy-site.yml` authored; YAML parse + structure check green; runtime verification (green runs, site 200) deferred to post-merge + Pages enablement per plan. Deviations from the starter Pages flow, all reviewed: `configure-pages`/`upload-pages-artifact` step-gated to push-to-main so PR builds stay green before Pages is enabled; per-ref concurrency for PR builds (single shared group would cancel queued PR checks); workflow self-path added to the path filters. Review hardening: `markdown.hooks.onBrokenMarkdownLinks: 'throw'` added so the link gate covers md-file links too — verified by inserting a broken link (build fails) and removing it (build passes).
- 2026-06-11 20:30 · tasks 6–13 (gate evidence + deviations) · Content MVP on `feat/site-content` (stacked on `feat/site-infra`), subagent-driven with per-task spec + quality reviews and fix commits. Build green throughout (`onBrokenLinks` + `onBrokenMarkdownLinks` both `'throw'`); docs-budget all green; log-lint clean on the D3 entry. Deviations, all reviewed: landing page is `index.mdx` (needs the `Link` import; scaffold `index.tsx`/`index.module.css` deleted); placeholder `site/docs/index.md` deleted at task 13 with the landing CTA re-pointed to `/docs/getting-started`; getting-started's guided flow gained gh-auth, plugin-install, and parent-dir steps the acceptance didn't enumerate (play-test stall risks found in review); task 13 scope grew to re-point TROUBLESHOOTING.md/PR-LIFECYCLE.md/dev-root-template refs whose BOOTSTRAP/WORKING-WITH-CLAUDE anchors died with the stubs (plan gap). Final cross-page review fixed two contradictions (next-steps bypassing the onboarding project; repo-setup naming a nonexistent secret-scanning workflow). Outstanding at PR time: `#TBD` ×2 in project_log.md (fill with content-PR number); task 9 orphan-branch deletion gated on Watson; task 14 must merge before testers are invited (play-testing page references the friction template).
- 2026-06-11 21:15 · tasks 6–13 (re-scope, Watson) · Conciseness pass over the content pages on Watson's feedback: human-facing docs were too long; they exist to start and ground conversation, with detail living in linked sources. working-with-claude −35%, onboarding-project −25%, rules-explained −21%, getting-started −17% (floor set by protected functional content: the bootstrap prompt, dual-platform commands, verify table). getting-started reordered so the recommended path ends at Next steps and the manual path is a trailing appendix (`#manual-path` → `#appendix-manual-path`; only referrer was the page's own intro). All TROUBLESHOOTING/PR-LIFECYCLE-cited anchors verified unchanged. Knowingly dropped: the §3 backlog-schema details (column set, ID/DAG notation) — this page was their only home; rehome in a template if needed. Build, docs-budget green throughout.
- 2026-06-11 22:30 · tasks 6–13 (direction change, Watson — supersedes parts of spec §1/§5/D6 framing) · White-label pivot from live review: the site is NOT a CAMELS lab introduction — it's a skills-building exercise (learn spec-driven development, set up an agentic workspace); play-testers bring their own brands. Applied this round: chrome de-branded (tagline, footer); landing rewritten as the TLDR page (lab section dropped); getting-started rewritten single-path (guided only — manual appendix, lab-repo cloning, access-request flow, Keeping-current all dropped; verify section now OS tabs; file → .mdx for the Tabs import); TROUBLESHOOTING junction refs re-pointed to README. Remaining pages await Watson's page-by-page review; spec amendment due when the round completes. Known residuals flagged: dev-root template still names the old clone set; play-testing still lists "lab-repo access" as a tester prerequisite.
- 2026-06-12 00:00 · tasks 4–13 merged + task 9 closeout · PR #14 and #15 squash-merged to main with Watson's go; Pages enabled by Watson (source: GitHub Actions); first deploys green; site live at watsonwblair.github.io/lab-os. Squash gotcha for the record: merging #14 broke ancestry for stacked #15 — GitHub closed it on base-branch deletion instead of re-targeting; recovered by restoring the base branch at its old tip, reopen → re-target main → resolve add/add conflicts to the branch versions → merge. Orphan branch `docs/onboarding-project` deleted with Watson's explicit confirmation (content lives at /docs/onboarding-project), along with stale local branches from merged work.
- 2026-06-12 00:05 · task 14 (gate evidence) · `friction.yml` field set verified 1:1 against play-testing.md's promises (four fields, severity cosmetic→fully-blocked with a "Stalled — 15+ minutes" tier, `playtest` auto-label); label created on the repo; YAML parses; build + docs-budget green. Two white-label residuals folded in: dev-root template clone-set line self-contained again; "lab-repo access" dropped from tester prerequisites. PR #16.
- 2026-06-12 01:30 · task 15 (gate evidence + framing call) · Tooling tour on `feat/site-tooling-tour` (worktree), subagent-driven. Framing call Watson left open, resolved: the tour stays lab-truthful — white-label applies to journey pages; reference pages describe the real repo, framed "what this repo runs on itself." Spec review compliant, with accuracy spot-checks against the workflow headers and lint scripts (warn-only vs enforce, override semantics, checkbox logic). pr-review agent SPEC repo is private → section written self-contained, no hyperlink, per the public-tier rule. Quality review drove 8 wording fixes (CI / reusable-workflow / caller glosses, work-bundle plain language). Deviation: no literal "covered separately" line existed in rules-explained — the cross-link landed at its CI parenthetical instead. Build green.
- 2026-06-12 01:45 · task 16 (gate evidence) · Public-tier jargon scrub across index + all seven docs pages, editorial only, same branch (the scrub covers task 15's page). Zero heading changes verified mechanically — all PR-LIFECYCLE/TROUBLESHOOTING-cited anchors intact. Spec review compliant; its 5 residual-jargon advisories fixed; final whole-branch review READY, 4 minors polished. Call flagged for Watson at PR: the index third bullet he requested ("mission-control surface you built yourself") is now "small dashboard-style app you built yourself" — the codename was unintroduced on the landing page; onboarding-project still introduces `mission-control` and the journey reads connected. Build + docs-budget green.
