---
sidebar_position: 1
title: Getting Started
description: Set up a working CAMELS lab environment — Claude-guided in one paste, or step-by-step by hand.
---

# Getting Started

From nothing to a working CAMELS Claude Code / Cowork environment ("Cowork" is the lab's shorthand
for a Claude Code working session). Two paths, same result:

1. **Claude-guided (recommended):** install Claude Code, paste one prompt; Claude interviews you and
   performs the setup, confirming each step.
2. **[Manual path](#appendix-manual-path):** the same steps by hand, in the appendix below.

Either way you end with: a lab workspace with the core repos cloned, your personal
`~/.claude/CLAUDE.md` (your persona, loads in every session), a dev-root `CLAUDE.md` (the lab map),
and a link that loads the lab rules into every session. ~20 minutes plus clone time.

`<DEV_ROOT>` is the single directory all lab repos clone into. Pick one and use it consistently —
**Windows (reference setup):** `C:\Users\<you>\Development` · **macOS / Linux:** `~/Development`.

## Prerequisites

| Need | Why | Check |
|---|---|---|
| **Claude subscription (Claude Max)** | Lab inference runs via Max, not a metered API key | log in at [claude.ai](https://claude.ai) |
| **Git** | Clone repos, run the workflow | `git --version` |
| **GitHub CLI (`gh`)** | PR workflow, private-repo auth | `gh --version` |
| **Python 3 + a virtual-env tool** | `LSCA` is Python + PyTorch | `python --version` |
| **Access to private lab repos** | `LSCA`, `Global_Pathways` may be private | request from the lab manager, Watson Blair ([watsonwblair@gmail.com](mailto:watsonwblair@gmail.com)), with your GitHub username |

After installing `gh`, authenticate first (same commands on every platform):

```bash
gh auth login
gh auth setup-git
```

Confirm with `gh auth status` — it should show "Logged in to github.com".

<details>
<summary>New to this? What `gh auth` actually does</summary>

GitHub needs to know who you are before it lets you download (clone) the lab's private code.
`gh auth login` opens a browser window where you sign in to GitHub once; after that, the `gh` tool
remembers you. `gh auth setup-git` tells Git — the separate tool that actually downloads code — to
reuse that same sign-in instead of asking for a password every time. Once `gh auth status` says
"Logged in", both tools can reach the private lab repos on your behalf.

</details>

> The lab's reference setup is **Windows 11 + PowerShell**; every step gives the macOS / Linux
> equivalent. Differences are almost always junction-vs-symlink and path separators.

## Step 1 — Install Claude Code

Follow [docs.claude.com/en/docs/claude-code/setup](https://docs.claude.com/en/docs/claude-code/setup),
confirm with `claude --version`, then run `claude` once and log in with your Claude Max account.

## Step 2 — Paste the bootstrap prompt

Start a session with `claude` in your home directory and paste the entire block below. Claude
interviews you first, then performs the setup, confirming before every step that creates,
overwrites, or links files. The lab templates in
[lab-os/templates/](https://github.com/WatsonWBlair/lab-os/tree/main/templates) stay the source of
truth — the prompt has Claude clone the repo and read them from there.

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
| **Windows (PowerShell):** `Get-Item "$HOME\Development\.claude\rules" \| Select-Object LinkType, Target` | `LinkType` = Junction; `Target` ends in `lab-os\.claude\rules` |
| **macOS / Linux:** `ls -l ~/Development/.claude/rules` | Arrow pointing to `lab-os/.claude/rules` |
| Cowork session at `<DEV_ROOT>` | Loads dev-root `CLAUDE.md` **and** the `lab-os` rules |
| Session inside `<DEV_ROOT>/LSCA` (if cloned) | Additionally loads `LSCA/CLAUDE.md` |
| Ask "what are the lab's commit-message rules?" | Answers from `01-workflow.md` (e.g. `feat:`, `fix:`, lowercase subject) |
| Your global `~/.claude/CLAUDE.md` | About Me reflects **you** — no `<...>` placeholders left |

**If a check fails:** junction command fails → re-do Manual §3. Junction resolves but the rules
question goes unanswered → confirm the session was opened AT `<DEV_ROOT>`, not a subdirectory.
Placeholders still showing → finish personalizing the global template.

## Next steps

- **[Working with Claude](/docs/working-with-claude)** — the lab's established methods; most were
  earned by hitting a failure mode, and reading them first saves the rediscovery.
- **[Onboarding Project](/docs/onboarding-project)** — a two-week throwaway sandbox build that has
  you practice the full lab workflow on low-stakes code.

## Keeping current

- `git pull` `lab-os` periodically — the link means new rules apply immediately.
- `git pull` `lab-claude-plugins`, then `/plugin marketplace update` for plugin changes.

## Appendix: manual path

The same setup by hand, for members who prefer doing it step by step. Set `$DEV_ROOT` once and all
blocks below use it:
**Windows (PowerShell):** `$DEV_ROOT = "$HOME\Development"` · **macOS / Linux:** `DEV_ROOT=~/Development`.

### 1. Create your lab workspace

Create the `<DEV_ROOT>` directory and move into it.

<details>
<summary>Commands — Windows and macOS/Linux</summary>

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

</details>

### 2. Clone the core repos

The core set is the two active research repos plus the two tooling repos; foundational and paused
repos clone on demand (see the lineage section of the dev-root `CLAUDE.md`). Confirm `gh auth status`
shows you logged in first.

<details>
<summary>Commands — Windows and macOS/Linux</summary>

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

</details>

If a clone 404s while `gh auth status` shows you logged in, you don't have access yet — request it
from the lab manager, Watson Blair ([watsonwblair@gmail.com](mailto:watsonwblair@gmail.com)), with
your GitHub username.

### 3. Wire lab-os into Cowork

Cowork picks up the lab rules when they appear at `<DEV_ROOT>/.claude/rules/`. Link — don't copy —
so a `git pull` of `lab-os` keeps you current.

<details>
<summary>Commands and link check — Windows and macOS/Linux</summary>

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

Verify: **Windows** `Get-Item "$DEV_ROOT\.claude\rules" | Select-Object LinkType, Target` (expect
Junction, target ending `lab-os\.claude\rules`); **macOS / Linux** `ls -l $DEV_ROOT/.claude/rules`
(expect an arrow to `lab-os/.claude/rules`). If the link resolves but a session doesn't see the
rules, confirm the session was opened AT `<DEV_ROOT>`, not a subdirectory.

</details>

<details>
<summary>New to this? What the junction/symlink does</summary>

A junction (Windows) or symlink (macOS/Linux) is a folder that is really just a signpost pointing
at another folder. Here, `<DEV_ROOT>/.claude/rules` doesn't hold its own copy of the lab rules —
it points at the copy inside the `lab-os` repo on your machine. When the lab updates its rules and
you run `git pull` in `lab-os`, the signpost automatically shows the new version, with nothing to
re-copy. That's why the instructions say "link, don't copy."

</details>

### 4. Install the CLAUDE.md templates

Two layers, two files, both from
[templates/](https://github.com/WatsonWBlair/lab-os/tree/main/templates).

<details>
<summary>New to this? The three CLAUDE.md layers</summary>

`CLAUDE.md` files are instruction notes that Claude reads automatically at the start of every
session. The lab uses three, stacked from general to specific: your **global** file (who you are
and how you like to work — applies everywhere), the **dev-root** file (a map of the lab's projects —
applies when you work in your lab workspace), and a **per-repo** file (details of one project —
applies inside that project). Claude reads every layer that applies to where you opened the
session, and when two disagree, the most specific one wins. This step installs the first two; each
repo ships its own third.

</details>

#### 4a. Personal-global (your persona, applies in every session, every project)

Copy `templates/global-CLAUDE.template.md` to `~/.claude/CLAUDE.md` (Windows:
`C:\Users\<you>\.claude\CLAUDE.md`). Fill the `<...>` placeholders in **About Me**; keep everything
below it close to verbatim.

<details>
<summary>Commands — macOS/Linux example</summary>

```bash
# macOS / Linux example
cp $DEV_ROOT/lab-os/templates/global-CLAUDE.template.md ~/.claude/CLAUDE.md
# then edit the About Me block
```

</details>

> Already have a personal `~/.claude/CLAUDE.md`? **Merge**, don't overwrite — keep your About Me,
> fold in the Ethics → Memory sections.

#### 4b. Dev-root orientation (lab map, applies at `<DEV_ROOT>`)

Copy `templates/dev-root-CLAUDE.template.md` to `<DEV_ROOT>/.claude/CLAUDE.md` and adjust paths.

<details>
<summary>Commands — macOS/Linux example</summary>

```bash
# macOS / Linux example
cp $DEV_ROOT/lab-os/templates/dev-root-CLAUDE.template.md $DEV_ROOT/.claude/CLAUDE.md
```

</details>

The layers compose global (you) → dev-root (lab map) → per-repo `CLAUDE.md`, most-specific wins —
override semantics: [lab-os README](https://github.com/WatsonWBlair/lab-os#readme).

### 5. Install the lab plugins

From inside a Claude Code session, add the lab marketplace and install the plugins; restart your
session to apply, then `/plugin` to confirm both are listed.

<details>
<summary>The slash commands, and where each plugin comes from</summary>

```text
/plugin marketplace add WatsonWBlair/lab-claude-plugins
/plugin install pr-review-loop@lab-claude-plugins
/plugin install superpowers@claude-plugins-official
```

`superpowers` is on the official marketplace that ships with Claude Code (no marketplace add) and
backs the lab's working methods — see [Working with Claude](/docs/working-with-claude).

</details>

### 6. Set up the active repos

Each repo's own `README.md` / `CLAUDE.md` is the authority. Minimum to get `LSCA` runnable: create
and activate a virtual environment, then install its requirements.

<details>
<summary>Commands — Windows and macOS/Linux</summary>

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

</details>

<details>
<summary>New to this? Virtual environments in one paragraph</summary>

A virtual environment is a private folder of Python packages that belongs to one project. Without
one, every Python project on your machine shares a single pile of installed packages, and two
projects that need different versions of the same package break each other. Creating one
(`python -m venv .venv`) and activating it tells your terminal "use this project's pile for now."
Make one per project, activate it whenever you work there, and `pip install` will only affect that
project.

</details>

`Global_Pathways` is in spec phase — read its `CLAUDE.md` before touching code. Then run the
checks in [Step 3 — Verify your setup](#step-3--verify-your-setup).

