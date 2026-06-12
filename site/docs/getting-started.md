---
sidebar_position: 1
title: Getting Started
description: Set up a working CAMELS lab environment — Claude-guided in one paste, or step-by-step by hand.
---

# Getting Started

This page takes a new lab member from nothing to a working CAMELS Claude Code / Cowork environment.
("Cowork" is the lab's name for a Claude Code working session — same tool, our shorthand.)

Either path produces the same workspace; repo-specific Python environment setup lives in
[Manual §6](#6-set-up-the-active-repos) for both.

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
| **GitHub CLI (`gh`)** | PR workflow, private-repo auth | `gh --version` |
| **Python 3 + a virtual-env tool** | `LSCA` is Python + PyTorch | `python --version` |
| **GitHub access to private lab repos** | `LSCA`, `Global_Pathways` may be private | request access from the lab manager, Watson Blair ([watsonwblair@gmail.com](mailto:watsonwblair@gmail.com)), with your GitHub username |

After installing `gh`, authenticate before you do anything else:

```powershell
# Windows (PowerShell)
gh auth login
gh auth setup-git
```

```bash
# macOS / Linux
gh auth login
gh auth setup-git
```

Confirm with `gh auth status` — it should show "Logged in to github.com".

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
   into <DEV_ROOT>. Before cloning, confirm `gh auth status` shows me
   logged in. If I'm authenticated and a clone still 404s, I don't have
   access yet — tell me to request it from the lab manager, Watson Blair
   (watsonwblair@gmail.com), with my GitHub username, and move on.
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
   copy, so a git pull of lab-os keeps me current. First create
   <DEV_ROOT>/.claude if it doesn't exist, then:
   - Windows (PowerShell, no admin needed):
     cmd /c mklink /J "<DEV_ROOT>\.claude\rules" "<DEV_ROOT>\lab-os\.claude\rules"
   - macOS / Linux:
     ln -s <DEV_ROOT>/lab-os/.claude/rules <DEV_ROOT>/.claude/rules
7. Install the lab plugins: run `/plugin marketplace add WatsonWBlair/lab-claude-plugins`
   then `/plugin install pr-review-loop@lab-claude-plugins`. Also install
   `/plugin install superpowers@claude-plugins-official` (superpowers is on
   the official marketplace that ships with Claude Code — no marketplace add
   needed). (These are slash commands I run in Claude Code myself — walk me
   through them rather than running them for me.)
8. Run a verification pass and show me the results:
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
| `/plugin` | lists `pr-review-loop@lab-claude-plugins` and `superpowers@claude-plugins-official` |
| Junction / symlink — **Windows (PowerShell):** `Get-Item "$HOME\Development\.claude\rules" \| Select-Object LinkType, Target` | `LinkType` = Junction; `Target` = path ending in `lab-os\.claude\rules` |
| Junction / symlink — **macOS / Linux:** `ls -l ~/Development/.claude/rules` | Arrow pointing to `lab-os/.claude/rules` |
| Open a Cowork session at `<DEV_ROOT>` | Loads dev-root `CLAUDE.md` **and** the `lab-os` rules |
| Open a session inside `<DEV_ROOT>/LSCA` (if you cloned LSCA) | Additionally loads `LSCA/CLAUDE.md` |
| Ask Claude "what are the lab's commit-message rules?" | Answers from `01-workflow.md` (e.g. `feat:`, `fix:`, lowercase subject) |
| Your global `~/.claude/CLAUDE.md` | About Me block reflects **you** — no `<...>` template placeholders left |

**If a check fails:** junction command fails → re-do the wiring step (Manual §3). Junction resolves fine
but rules question is unanswered → confirm the session was opened AT `<DEV_ROOT>`. Template placeholders
still showing → finish personalizing the global template.

## Manual path

The same setup by hand. Set `$DEV_ROOT` once in your shell and all blocks below use it automatically:

- **Windows (PowerShell):** `$DEV_ROOT = "$HOME\Development"` (adjust if you picked a different root)
- **macOS / Linux:** `DEV_ROOT=~/Development`

### 1. Create your lab workspace

**Windows (PowerShell):**

```powershell
$DEV_ROOT = "$HOME\Development"
New-Item -ItemType Directory -Force $DEV_ROOT
Set-Location $DEV_ROOT
```

**macOS / Linux:**

```bash
DEV_ROOT=~/Development
mkdir -p $DEV_ROOT && cd $DEV_ROOT
```

### 2. Clone the core repos

The core bootstrap set is the two **active** research repos plus the two **tooling** repos. Foundational
and paused repos (`Vibe_App`, `cs627`, `FCM_Analysis`, …) are cloned on demand when a question sends you
upstream — see the lineage section of the dev-root `CLAUDE.md`.

First confirm you are authenticated:

```powershell
# Windows (PowerShell)
gh auth status
```

```bash
# macOS / Linux
gh auth status
```

If not logged in, run `gh auth login` and `gh auth setup-git` before continuing.

**Windows (PowerShell):**

```powershell
git clone https://github.com/WatsonWBlair/LSCA.git "$DEV_ROOT\LSCA"
git clone https://github.com/WatsonWBlair/Global_Pathways.git "$DEV_ROOT\Global_Pathways"
git clone https://github.com/WatsonWBlair/lab-os.git "$DEV_ROOT\lab-os"
git clone https://github.com/WatsonWBlair/lab-claude-plugins.git "$DEV_ROOT\lab-claude-plugins"
```

**macOS / Linux:**

```bash
git clone https://github.com/WatsonWBlair/LSCA.git $DEV_ROOT/LSCA
git clone https://github.com/WatsonWBlair/Global_Pathways.git $DEV_ROOT/Global_Pathways
git clone https://github.com/WatsonWBlair/lab-os.git $DEV_ROOT/lab-os
git clone https://github.com/WatsonWBlair/lab-claude-plugins.git $DEV_ROOT/lab-claude-plugins
```

If `LSCA` or `Global_Pathways` 404s: first confirm `gh auth status` shows you are logged in. If
authenticated and it still 404s, you don't have access yet — request it from the lab manager, Watson
Blair ([watsonwblair@gmail.com](mailto:watsonwblair@gmail.com)), with your GitHub username.

### 3. Wire lab-os into Cowork

Lab-wide conventions live in `lab-os/.claude/rules/`. Cowork picks them up when they appear at
`<DEV_ROOT>/.claude/rules/`. Link — don't copy — so a `git pull` of `lab-os` keeps you current.

**Windows (PowerShell) — junction, no admin required:**

```powershell
New-Item -ItemType Directory -Force "$DEV_ROOT\.claude"
cmd /c mklink /J "$DEV_ROOT\.claude\rules" "$DEV_ROOT\lab-os\.claude\rules"
```

**macOS / Linux — symlink:**

```bash
mkdir -p $DEV_ROOT/.claude
ln -s $DEV_ROOT/lab-os/.claude/rules $DEV_ROOT/.claude/rules
```

Verify the link resolves:

- **Windows (PowerShell):** `Get-Item "$DEV_ROOT\.claude\rules" | Select-Object LinkType, Target`
  — expect `LinkType` = Junction and `Target` ending in `lab-os\.claude\rules`
- **macOS / Linux:** `ls -l $DEV_ROOT/.claude/rules`
  — expect an arrow pointing to `lab-os/.claude/rules`

If the junction command fails, re-do this step. If the junction resolves fine but a session at
`<DEV_ROOT>` doesn't see the rules, confirm the session was opened AT `<DEV_ROOT>` (not a subdirectory).

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
cp $DEV_ROOT/lab-os/templates/global-CLAUDE.template.md ~/.claude/CLAUDE.md
# then edit the About Me block
```

> If you already have a personal `~/.claude/CLAUDE.md`, **merge** rather than overwrite — fold in the
> Ethics → Memory sections, keep your own About Me.

#### 4b. Dev-root orientation (lab map, applies when you open a session at `<DEV_ROOT>`)

Copy `templates/dev-root-CLAUDE.template.md` to `<DEV_ROOT>/.claude/CLAUDE.md` and adjust paths.

```bash
# macOS / Linux example
cp $DEV_ROOT/lab-os/templates/dev-root-CLAUDE.template.md $DEV_ROOT/.claude/CLAUDE.md
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

Restart your Claude Code session to apply, then `/plugin` to confirm it's listed.

> **Also install `superpowers`.** The lab's working methods (see
> [Working with Claude](/docs/working-with-claude)) lean
> on the `superpowers` plugin's process skills (brainstorming, writing-plans, subagent-driven-development,
> verification-before-completion, …). Superpowers is on the official plugin marketplace that ships with
> Claude Code — no marketplace add needed:
>
> ```text
> /plugin install superpowers@claude-plugins-official
> ```

### 6. Set up the active repos

Each repo's own `README.md` / `CLAUDE.md` is the authority. Minimum to get `LSCA` runnable:

```powershell
# Windows (PowerShell)
cd "$DEV_ROOT\LSCA"
# create and activate a virtual environment, then:
pip install -r requirements.txt   # or follow LSCA/README.md if it differs
```

```bash
# macOS / Linux
cd "$DEV_ROOT/LSCA"
# create and activate a virtual environment, then:
pip install -r requirements.txt   # or follow LSCA/README.md if it differs
```

`Global_Pathways` consumes the `camels` package built from `LSCA` and is in spec phase — read its
`CLAUDE.md` before touching code.

Then run the checks in [Step 3 — Verify your setup](#step-3--verify-your-setup), plus: `/plugin` lists
`pr-review-loop@lab-claude-plugins` (if not, re-run §5 — `/plugin marketplace add`, `/plugin install`,
then restart your Claude Code session).

## Next steps

- **Read [Working with Claude](/docs/working-with-claude)**
  — the lab's established methods and best practices (code-free plans, verification discipline, review
  discipline, autonomous-loop safety, communication discipline). Most of it was earned by hitting a
  failure mode and correcting it; reading it first saves you the rediscovery.
- **Start the [Onboarding Project](/docs/onboarding-project)** — a two-week, throwaway sandbox build
  in your own disposable repo that has you practice the full lab workflow (brainstorm → spec →
  code-free plan → subagent-driven build → review → log) on low-stakes code. It is the structured
  practice vehicle for everything the methods page describes; you choose your own stack.

## Keeping current

- `git pull` `lab-os` periodically — the junction/symlink means new rules apply immediately, no re-link.
- `git pull` `lab-claude-plugins`, then `/plugin marketplace update` to pick up plugin changes.
- When lab conventions change, the change lands in `lab-os` first; your local link stays the
  source-of-truth.
