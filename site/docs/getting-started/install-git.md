---
sidebar_position: 3
title: Install Git
---

# Install Git

Git copies ("clones") repositories to your machine and tracks every change you make. You need it before
you can run the bootstrap prompt.

**Official source:** [git-scm.com/downloads](https://git-scm.com/downloads)

## Windows

**Option A — winget (recommended if you have Windows 10 1709+ or Windows 11):**

```bash title="Terminal"
winget install --id Git.Git -e --source winget
```

**Option B — installer:** download the Windows installer from
[git-scm.com/download/win](https://git-scm.com/download/win) and run it. Accept the defaults unless
you have a reason to change them.

After either option, close and reopen PowerShell before running the version check below.

## macOS

**Option A — Xcode Command Line Tools (simplest):**

```bash title="Terminal"
xcode-select --install
```

A dialog appears asking you to install the tools. Click **Install**. Git is included.

**Option B — Homebrew (if you already have it):**

```bash title="Terminal"
brew install git
```

## Linux

**Debian / Ubuntu:**

```bash title="Terminal"
sudo apt update && sudo apt install git
```

**Fedora / RHEL / CentOS:**

```bash title="Terminal"
sudo dnf install git
```

For other distributions, see [git-scm.com/download/linux](https://git-scm.com/download/linux).

## Confirm the install

```bash title="Terminal"
git --version
```

Expected output (version number may differ):

```
git version 2.47.0
```

If you see a version number, Git is installed and ready. If you see "command not found", close and reopen
your terminal, then try again. If it still fails, revisit the install step for your OS above.
