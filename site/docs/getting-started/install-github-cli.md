---
sidebar_position: 4
title: Install GitHub CLI
---

# Install GitHub CLI

The GitHub CLI (`gh`) is the tool you use to open pull requests, clone repositories, and interact with
GitHub from your terminal. You need it before you can run the bootstrap prompt.

**Official source:** [cli.github.com](https://cli.github.com)

## Windows

**Option A — winget (recommended):**

```bash title="Terminal"
winget install --id GitHub.cli -e --source winget
```

**Option B — installer:** download the `.msi` installer from
[github.com/cli/cli/releases/latest](https://github.com/cli/cli/releases/latest) and run it.

After either option, close and reopen PowerShell.

## macOS

**Homebrew:**

```bash title="Terminal"
brew install gh
```

No Homebrew? Install it first from [brew.sh](https://brew.sh), or download the macOS `.pkg` directly from
[github.com/cli/cli/releases/latest](https://github.com/cli/cli/releases/latest).

## Linux

**Debian / Ubuntu (official apt repository):**

```bash title="Terminal"
(type -p wget >/dev/null || (sudo apt update && sudo apt-get install wget -y)) \
  && sudo mkdir -p -m 755 /etc/apt/keyrings \
  && out=$(mktemp) && wget -nv -O$out https://cli.github.com/packages/githubcli-archive-keyring.gpg \
  && cat $out | sudo tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
  && sudo chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
  && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | sudo tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
  && sudo apt update \
  && sudo apt install gh -y
```

**Fedora / RHEL / CentOS:**

```bash title="Terminal"
sudo dnf install 'dnf-command(config-manager)' && sudo dnf config-manager --add-repo https://cli.github.com/packages/rpm/gh-cli.repo && sudo dnf install gh
```

For other Linux distributions, see [cli.github.com/manual/installation](https://cli.github.com/manual/installation).

## Authenticate with GitHub

After installing `gh`, run these two commands (same on every platform):

```bash title="Terminal"
gh auth login
```

This opens a browser window where you sign in to GitHub. Follow the prompts — it takes about a minute.

```bash title="Terminal"
gh auth setup-git
```

This tells Git to reuse your GitHub sign-in instead of asking for a password every time.

<details>
<summary>New to this? What gh auth actually does</summary>

GitHub needs to know who you are before it lets you push code or open pull requests.
`gh auth login` opens a browser window where you sign in to GitHub once; after that, the `gh` tool
remembers you. `gh auth setup-git` tells Git — the separate tool that actually moves code — to
reuse that same sign-in instead of asking for a password every time. Once `gh auth status` says
"Logged in", both tools work on your behalf.

</details>

## Confirm authentication

```bash title="Terminal"
gh auth status
```

Expected output includes:

```
Logged in to github.com as <your-username>
```

If you see an error instead, re-run `gh auth login` and follow the prompts again.
