# Bootstrap: setting up the CAMELS lab on your machine

This runbook takes a new lab member from nothing to a working CAMELS Claude Code / Cowork environment.
("Cowork" is the lab's name for a Claude Code working session — same tool, our shorthand.) It covers the
**shared layer** every member needs — workspace layout, the core repos, lab conventions, the lab plugins,
and the two `CLAUDE.md` templates that wire it all together.

Estimated time: ~20 minutes plus repo clone / Python environment time.

Throughout, `<DEV_ROOT>` is the single directory you clone all lab repos into. Pick one and use it
consistently:
- **Windows (reference setup):** `C:\Users\<you>\Development`
- **macOS / Linux:** `~/Development`

The shell blocks below write `<DEV_ROOT>` literally. To make them copy-pasteable, set it as a variable
once in your shell and substitute mentally (or use `$DEV_ROOT` in the POSIX blocks):
- **Windows (PowerShell):** `$DEV_ROOT = "$HOME\Development"`
- **macOS / Linux:** `export DEV_ROOT=~/Development`

---

## 0. Prerequisites

| Need | Why | Check |
|---|---|---|
| **Claude Code / Cowork on Claude Max** | The lab runs inference via Claude Max, not a metered API key | `claude --version` |
| **Git** | Clone repos, run the workflow | `git --version` |
| **GitHub CLI (`gh`)** | PR workflow, private-repo auth | `gh auth status` |
| **Python 3 + a virtual-env tool** | `LSCA` is Python + PyTorch | `python --version` |
| **GitHub access to private lab repos** | `LSCA`, `Global_Pathways` may be private | request access from the lab manager, Watson Blair (<watsonwblair@gmail.com>) |

> The lab's primary reference setup is **Windows 11 + PowerShell**. Each step below gives the
> macOS / Linux equivalent. Where they differ it's almost always **junction (Windows) vs symlink (Unix)**
> and **path separators**.

---

## 1. Create your lab workspace

**Windows (PowerShell):**
```powershell
New-Item -ItemType Directory -Force "$HOME\Development"
Set-Location "$HOME\Development"
```

**macOS / Linux:**
```bash
mkdir -p ~/Development && cd ~/Development
```

---

## 2. Clone the core repos

The core bootstrap set is the two **active** research repos plus the two **tooling** repos. Foundational
and paused repos (`Vibe_App`, `cs627`, `FCM_Analysis`, …) are cloned on demand when a question sends you
upstream — see the lineage section of the dev-root `CLAUDE.md`.

```bash
# from <DEV_ROOT>
git clone https://github.com/WatsonWBlair/LSCA.git
git clone https://github.com/WatsonWBlair/Global_Pathways.git
git clone https://github.com/WatsonWBlair/lab-rules.git
git clone https://github.com/WatsonWBlair/lab-claude-plugins.git
```

If `LSCA` or `Global_Pathways` 404s, you don't have access yet — request it from the lab manager, Watson
Blair (<watsonwblair@gmail.com>), with your GitHub username.

---

## 3. Wire lab-rules into Cowork

Lab-wide conventions live in `lab-rules/.claude/rules/`. Cowork picks them up when they appear at
`<DEV_ROOT>/.claude/rules/`. Link — don't copy — so a `git pull` of `lab-rules` keeps you current.

**Windows (PowerShell) — junction, no admin required:**
```powershell
New-Item -ItemType Directory -Force "$HOME\Development\.claude"
cmd /c mklink /J "$HOME\Development\.claude\rules" "$HOME\Development\lab-rules\.claude\rules"
```

**macOS / Linux — symlink:**
```bash
mkdir -p ~/Development/.claude
ln -s ~/Development/lab-rules/.claude/rules ~/Development/.claude/rules
```

Verify the link resolves: a session opened at `<DEV_ROOT>` should load `01-workflow.md` and
`02-data-protection.md`.

---

## 4. Install the CLAUDE.md templates

Two layers, two files. Both templates ship in this repo under `templates/`.

### 4a. Personal-global (your persona, applies in every session, every project)

Copy `templates/global-CLAUDE.template.md` to your personal Claude config and fill in the `<...>`
placeholders in the **About Me** block. Keep everything below it (the lab operating philosophy) close to
verbatim.

- **Windows:** copy to `C:\Users\<you>\.claude\CLAUDE.md`
- **macOS / Linux:** copy to `~/.claude/CLAUDE.md`

```bash
# macOS / Linux example
cp ~/Development/lab-rules/templates/global-CLAUDE.template.md ~/.claude/CLAUDE.md
# then edit the About Me block
```

> If you already have a personal `~/.claude/CLAUDE.md`, **merge** rather than overwrite — fold in the
> Ethics → Memory sections, keep your own About Me.

### 4b. Dev-root orientation (lab map, applies when you open a session at `<DEV_ROOT>`)

Copy `templates/dev-root-CLAUDE.template.md` to `<DEV_ROOT>/.claude/CLAUDE.md` and adjust paths.

```bash
# macOS / Linux example
cp ~/Development/lab-rules/templates/dev-root-CLAUDE.template.md ~/Development/.claude/CLAUDE.md
```

**How the layers compose:** global (you) → dev-root (lab map) → per-repo `CLAUDE.md` (project specifics),
most-specific wins. Per-repo rules extend or override lab rules; see this repo's `README.md` for override
semantics.

---

## 5. Install the lab plugins

The lab's Claude Code plugins (e.g. the PR-review loop) ship from the `lab-claude-plugins` marketplace.
From inside a Claude Code session:

```
/plugin marketplace add WatsonWBlair/lab-claude-plugins
/plugin install pr-review-loop@lab-claude-plugins
```

Run `/reload-plugins` to apply, then `/plugin` to confirm it's listed.

> **Also install `superpowers`.** The methods in [`WORKING-WITH-CLAUDE.md`](WORKING-WITH-CLAUDE.md) and §8
> below lean on the `superpowers` plugin's process skills (brainstorming, writing-plans,
> subagent-driven-development, verification-before-completion, …). It's a **separate** plugin, not part of
> the `lab-claude-plugins` marketplace — install it the same way (`/plugin marketplace add` its source,
> then `/plugin install`). If you don't have the marketplace source, ask the lab manager.

---

## 6. Set up the active repos

Each repo's own `README.md` / `CLAUDE.md` is the authority. Minimum to get `LSCA` runnable:

```bash
cd <DEV_ROOT>/LSCA
# create and activate a virtual environment, then:
pip install -r requirements.txt   # or follow LSCA/README.md if it differs
```

`Global_Pathways` consumes the `camels` package built from `LSCA` and is in spec phase — read its
`CLAUDE.md` before touching code.

---

## 7. Verify the setup

| Check | Expected |
|---|---|
| Open a Cowork session at `<DEV_ROOT>` | Loads dev-root `CLAUDE.md` **and** `lab-rules` `.claude/rules/*` |
| Open a session inside `<DEV_ROOT>/LSCA` | Additionally loads `LSCA/CLAUDE.md` |
| `/plugin` | Lists `pr-review-loop@lab-claude-plugins` |
| Ask Claude "what are the lab's commit-message rules?" | Answers from `01-workflow.md` (e.g. `feat:`, `fix:`, lowercase subject) |
| Your global `CLAUDE.md` | About Me block reflects **you**, not the template placeholders |

**If a check fails:** rules not loading → re-check the junction/symlink in §3 (does `<DEV_ROOT>/.claude/rules`
resolve to the `lab-rules` copy?). Plugin not listed → re-run §5 (`/plugin marketplace add` then
`/plugin install`, then `/reload-plugins`). Template placeholders still showing → finish §4a's edit.

---

## 8. Practice: get comfortable with Subagent-Driven Development

Before driving real research work, get fluent with the lab's preferred Claude workflow —
**Subagent-Driven Development** (plan → dispatch independent tasks to subagents → review). The lab uses
**`mission-control`** (a local-first FastAPI + React dashboard) as the practice ground for this: it has
tests and a CI gate, so the review loop is real without research stakes.

This is an **individual** practice project — set up your own copy (or your own equivalent app) and use it
to learn the loop. It is **not** part of the shared bootstrap clone set, and you are not working in the
lab manager's instance. Pair it with the `superpowers:subagent-driven-development` and
`superpowers:writing-plans` skills.

Before you start, read **[`WORKING-WITH-CLAUDE.md`](WORKING-WITH-CLAUDE.md)** — the lab's established
methods and best practices (code-free plans, verification discipline, review discipline, autonomous-loop
safety, communication discipline). Most of it was earned by hitting a failure mode and correcting it;
reading it first saves you the rediscovery.

---

## Keeping current

- `git pull` `lab-rules` periodically — the junction/symlink means new rules apply immediately, no re-link.
- `git pull` `lab-claude-plugins`, then `/plugin marketplace update` to pick up plugin changes.
- When lab conventions change, the change lands in `lab-rules` first; your local link stays the
  source-of-truth.
