---
sidebar_position: 1
title: Getting Started
description: Set up a working CAMELS lab environment — Claude-guided in one paste, or step-by-step by hand.
---

# Getting Started

This page takes a new lab member from nothing to a working CAMELS Claude Code / Cowork environment.
("Cowork" is the lab's name for a Claude Code working session — same tool, our shorthand.)

There are two paths to the same result:

1. **Claude-guided (recommended):** install Claude Code, paste one prompt, and Claude interviews you and
   performs the setup with you, confirming each step.
2. **[Manual path](#manual-path):** the same steps by hand, for members who prefer it.

Either way, the setup produces: a lab workspace directory with the core repos cloned, your personal
`~/.claude/CLAUDE.md` (your persona, loads in every session), a dev-root `CLAUDE.md` (the lab map), and a
link that loads the lab rules into every session. Estimated time: ~20 minutes plus repo clone time.

Throughout, `<DEV_ROOT>` is the single directory you clone all lab repos into. Pick one and use it
consistently:

- **Windows (reference setup):** `C:\Users\<you>\Development`
- **macOS / Linux:** `~/Development`

## Prerequisites

| Need | Why | Check |
|---|---|---|
| **A Claude subscription (Claude Max)** | The lab runs inference via Claude Max, not a metered API key | log in at [claude.ai](https://claude.ai) |
| **Git** | Clone repos, run the workflow | `git --version` |
| **GitHub CLI (`gh`)** | PR workflow, private-repo auth | `gh auth status` |
| **Python 3 + a virtual-env tool** | `LSCA` is Python + PyTorch | `python --version` |
| **GitHub access to private lab repos** | `LSCA`, `Global_Pathways` may be private | request access from the lab manager, Watson Blair ([watsonwblair@gmail.com](mailto:watsonwblair@gmail.com)), with your GitHub username |

> The lab's primary reference setup is **Windows 11 + PowerShell**. Every step below gives the
> macOS / Linux equivalent. Where they differ it's almost always **junction (Windows) vs symlink (Unix)**
> and **path separators**.

## Step 1 — Install Claude Code

Follow the official install instructions at
[docs.claude.com/en/docs/claude-code/setup](https://docs.claude.com/en/docs/claude-code/setup), then
confirm:

```bash
claude --version
```

Run `claude` once and log in with your Claude Max account when prompted.

## Step 2 — Paste the bootstrap prompt

Open a terminal in your home directory, start a session with `claude`, and paste the entire block below.
Claude will interview you first, then perform the setup with you — it should confirm before every step
that creates, overwrites, or links files. The lab templates in the `lab-os` repo
([templates/](https://github.com/WatsonWBlair/lab-os/tree/main/templates)) stay the source of truth; the
prompt has Claude clone the repo and read them from there.

```text
I'm a new member of the CAMELS Research Group setting up my machine for lab
work. Walk me through setup interactively. Rules for the whole session:

- Interview me BEFORE writing anything: my name, role in the lab, career
  stage / background, what I'll be working on, my stack and primary shell,
  my working style, my OS (Windows / macOS / Linux), my time zone, and where
  I want my lab workspace directory (<DEV_ROOT> — default
  C:\Users\<me>\Development on Windows, ~/Development on macOS/Linux).
- Confirm with me before any step that creates, overwrites, or links files.
  If a target file already exists (especially ~/.claude/CLAUDE.md), show me
  a merge plan and get my OK — never silently overwrite.
- The lab-os templates are the source of truth. Read them from the cloned
  repo and follow their embedded instructions; do not improvise their
  content from memory.

Then do the following, in order, confirming each step with me:

1. Create my lab workspace directory <DEV_ROOT> if it doesn't exist.
2. Clone https://github.com/WatsonWBlair/lab-os.git into <DEV_ROOT> first —
   the setup templates live in its templates/ directory.
3. Ask me which other lab repos I have access to (the core set is LSCA,
   Global_Pathways, and lab-claude-plugins, all under
   github.com/WatsonWBlair; some are private). Clone the ones I confirm
   into <DEV_ROOT>. If a clone 404s, I don't have access yet — tell me to
   request it from the lab manager, Watson Blair (watsonwblair@gmail.com),
   with my GitHub username, and move on.
4. Personalize <DEV_ROOT>/lab-os/templates/global-CLAUDE.template.md into
   my personal global config at ~/.claude/CLAUDE.md (Windows:
   C:\Users\<me>\.claude\CLAUDE.md). Fill the About Me block from the
   interview; keep everything below it (Ethics through Memory system) close
   to verbatim; remove the template's instructional blockquote. If I
   already have a global CLAUDE.md, merge instead of overwriting — keep my
   About Me, fold in the lab sections.
5. Seed <DEV_ROOT>/.claude/CLAUDE.md from
   <DEV_ROOT>/lab-os/templates/dev-root-CLAUDE.template.md, adjusting paths
   to my machine and removing the instructional blockquote.
6. Link the lab rules so sessions at <DEV_ROOT> load them — link, don't
   copy, so a git pull of lab-os keeps me current:
   - Windows (PowerShell, no admin needed):
     cmd /c mklink /J "<DEV_ROOT>\.claude\rules" "<DEV_ROOT>\lab-os\.claude\rules"
   - macOS / Linux:
     ln -s <DEV_ROOT>/lab-os/.claude/rules <DEV_ROOT>/.claude/rules
7. Run a verification pass and show me the results:
   - <DEV_ROOT>/.claude/rules resolves to lab-os/.claude/rules and lists
     the rule files (01-workflow.md, 02-data-protection.md, ...)
   - ~/.claude/CLAUDE.md exists with no remaining <...> placeholders
   - <DEV_ROOT>/.claude/CLAUDE.md exists
   - every repo I confirmed in step 3 is cloned
   Then tell me to start a fresh session at <DEV_ROOT> and ask it "what are
   the lab's commit-message rules?" — the answer should come from
   01-workflow.md.

If anything fails, stop and explain before moving on. When everything
passes, summarize what was set up and where.
```

## Step 3 — Verify your setup

Whichever path you took, all of these should hold:

| Check | Expected |
|---|---|
| `<DEV_ROOT>/.claude/rules` | Resolves to `lab-os/.claude/rules` and lists the rule files (`01-workflow.md`, `02-data-protection.md`, …) |
| Open a Cowork session at `<DEV_ROOT>` | Loads dev-root `CLAUDE.md` **and** the `lab-os` rules |
| Open a session inside `<DEV_ROOT>/LSCA` | Additionally loads `LSCA/CLAUDE.md` |
| Ask Claude "what are the lab's commit-message rules?" | Answers from `01-workflow.md` (e.g. `feat:`, `fix:`, lowercase subject) |
| Your global `~/.claude/CLAUDE.md` | About Me block reflects **you** — no `<...>` template placeholders left |

**If a check fails:** rules not loading → re-check the junction/symlink (does `<DEV_ROOT>/.claude/rules`
resolve to the `lab-os` copy?). Template placeholders still showing → finish personalizing the global
template. Session not seeing the dev-root `CLAUDE.md` → confirm it lives at `<DEV_ROOT>/.claude/CLAUDE.md`
and the session was opened at `<DEV_ROOT>`.

## Manual path

The same setup by hand. The shell blocks write `<DEV_ROOT>` literally — set it once in your shell and
substitute:

- **Windows (PowerShell):** `$DEV_ROOT = "$HOME\Development"`
- **macOS / Linux:** `export DEV_ROOT=~/Development`

### 1. Create your lab workspace

**Windows (PowerShell):**

```powershell
New-Item -ItemType Directory -Force "$HOME\Development"
Set-Location "$HOME\Development"
```

**macOS / Linux:**

```bash
mkdir -p ~/Development && cd ~/Development
```

### 2. Clone the core repos

The core bootstrap set is the two **active** research repos plus the two **tooling** repos. Foundational
and paused repos (`Vibe_App`, `cs627`, `FCM_Analysis`, …) are cloned on demand when a question sends you
upstream — see the lineage section of the dev-root `CLAUDE.md`.

```bash
# from <DEV_ROOT>
git clone https://github.com/WatsonWBlair/LSCA.git
git clone https://github.com/WatsonWBlair/Global_Pathways.git
git clone https://github.com/WatsonWBlair/lab-os.git
git clone https://github.com/WatsonWBlair/lab-claude-plugins.git
```

If `LSCA` or `Global_Pathways` 404s, you don't have access yet — request it from the lab manager, Watson
Blair ([watsonwblair@gmail.com](mailto:watsonwblair@gmail.com)), with your GitHub username.

### 3. Wire lab-os into Cowork

Lab-wide conventions live in `lab-os/.claude/rules/`. Cowork picks them up when they appear at
`<DEV_ROOT>/.claude/rules/`. Link — don't copy — so a `git pull` of `lab-os` keeps you current.

**Windows (PowerShell) — junction, no admin required:**

```powershell
New-Item -ItemType Directory -Force "$HOME\Development\.claude"
cmd /c mklink /J "$HOME\Development\.claude\rules" "$HOME\Development\lab-os\.claude\rules"
```

**macOS / Linux — symlink:**

```bash
mkdir -p ~/Development/.claude
ln -s ~/Development/lab-os/.claude/rules ~/Development/.claude/rules
```

Verify the link resolves: a session opened at `<DEV_ROOT>` should load `01-workflow.md` and
`02-data-protection.md`.

### 4. Install the CLAUDE.md templates

Two layers, two files. Both templates ship in the `lab-os` repo under
[templates/](https://github.com/WatsonWBlair/lab-os/tree/main/templates).

#### 4a. Personal-global (your persona, applies in every session, every project)

Copy `templates/global-CLAUDE.template.md` to your personal Claude config and fill in the `<...>`
placeholders in the **About Me** block. Keep everything below it (the lab operating philosophy) close to
verbatim.

- **Windows:** copy to `C:\Users\<you>\.claude\CLAUDE.md`
- **macOS / Linux:** copy to `~/.claude/CLAUDE.md`

```bash
# macOS / Linux example
cp ~/Development/lab-os/templates/global-CLAUDE.template.md ~/.claude/CLAUDE.md
# then edit the About Me block
```

> If you already have a personal `~/.claude/CLAUDE.md`, **merge** rather than overwrite — fold in the
> Ethics → Memory sections, keep your own About Me.

#### 4b. Dev-root orientation (lab map, applies when you open a session at `<DEV_ROOT>`)

Copy `templates/dev-root-CLAUDE.template.md` to `<DEV_ROOT>/.claude/CLAUDE.md` and adjust paths.

```bash
# macOS / Linux example
cp ~/Development/lab-os/templates/dev-root-CLAUDE.template.md ~/Development/.claude/CLAUDE.md
```

**How the layers compose:** global (you) → dev-root (lab map) → per-repo `CLAUDE.md` (project specifics),
most-specific wins. Per-repo rules extend or override lab rules; see the
[lab-os README](https://github.com/WatsonWBlair/lab-os#readme) for override semantics.

### 5. Install the lab plugins

The lab's Claude Code plugins (e.g. the PR-review loop) ship from the `lab-claude-plugins` marketplace.
From inside a Claude Code session:

```text
/plugin marketplace add WatsonWBlair/lab-claude-plugins
/plugin install pr-review-loop@lab-claude-plugins
```

Run `/reload-plugins` to apply, then `/plugin` to confirm it's listed.

> **Also install `superpowers`.** The lab's working methods (see
> [WORKING-WITH-CLAUDE.md](https://github.com/WatsonWBlair/lab-os/blob/main/WORKING-WITH-CLAUDE.md)) lean
> on the `superpowers` plugin's process skills (brainstorming, writing-plans, subagent-driven-development,
> verification-before-completion, …). It's a **separate** plugin, not part of the `lab-claude-plugins`
> marketplace — install it the same way (`/plugin marketplace add` its source, then `/plugin install`).
> If you don't have the marketplace source, ask the lab manager.

### 6. Set up the active repos

Each repo's own `README.md` / `CLAUDE.md` is the authority. Minimum to get `LSCA` runnable:

```bash
cd <DEV_ROOT>/LSCA
# create and activate a virtual environment, then:
pip install -r requirements.txt   # or follow LSCA/README.md if it differs
```

`Global_Pathways` consumes the `camels` package built from `LSCA` and is in spec phase — read its
`CLAUDE.md` before touching code.

Then run the checks in [Step 3 — Verify your setup](#step-3--verify-your-setup), plus: `/plugin` lists
`pr-review-loop@lab-claude-plugins` (if not, re-run §5 — `/plugin marketplace add`, `/plugin install`,
`/reload-plugins`).

## Next steps

- **Read [WORKING-WITH-CLAUDE.md](https://github.com/WatsonWBlair/lab-os/blob/main/WORKING-WITH-CLAUDE.md)**
  — the lab's established methods and best practices (code-free plans, verification discipline, review
  discipline, autonomous-loop safety, communication discipline). Most of it was earned by hitting a
  failure mode and correcting it; reading it first saves you the rediscovery.
- **Practice Subagent-Driven Development** (plan → dispatch independent tasks to subagents → review)
  before driving real research work. The lab uses `mission-control` (a local-first FastAPI + React
  dashboard) as the practice ground: it has tests and a CI gate, so the review loop is real without
  research stakes. This is an **individual** practice project — set up your own copy (or your own
  equivalent app); it is not part of the shared bootstrap clone set. Pair it with the
  `superpowers:subagent-driven-development` and `superpowers:writing-plans` skills.

## Keeping current

- `git pull` `lab-os` periodically — the junction/symlink means new rules apply immediately, no re-link.
- `git pull` `lab-claude-plugins`, then `/plugin marketplace update` to pick up plugin changes.
- When lab conventions change, the change lands in `lab-os` first; your local link stays the
  source-of-truth.
