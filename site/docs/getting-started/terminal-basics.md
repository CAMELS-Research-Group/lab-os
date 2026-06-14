---
sidebar_position: 2
title: Terminal Basics
---

# Terminal Basics

A terminal (also called a command line or shell) is a text window where you type a command and press
**Enter** to run it. The computer executes the command and prints the result. That's the whole model.

You don't need to memorize commands — the handbook always shows exactly what to type.

## How to open a terminal

### Windows — PowerShell

1. Press the **Windows key**, type `PowerShell`, and click **Windows PowerShell** in the results.
2. A blue (or dark) window opens with a `PS C:\...>` prompt. Type here.

> **Tip:** You can also right-click the Start button and choose **Terminal** or **Windows PowerShell**
> directly.

### macOS — Terminal

1. Press **Cmd + Space** to open Spotlight, type `Terminal`, and press **Enter**.
2. A window opens with a `%` or `$` prompt. Type here.

> **Tip:** Terminal lives in **Applications → Utilities → Terminal** if you prefer to find it in Finder.

### Linux — Terminal app

Open your desktop's terminal emulator. Common names: **Terminal**, **Konsole**, **GNOME Terminal**,
**xterm**. You can usually find it in your app menu or right-click the desktop.

## Reading command blocks

Throughout this handbook, commands you type in a terminal look like this:

```bash title="Terminal"
echo "hello"
```

Type the text (not including the `$` or `>` prompt if shown), press **Enter**, and watch the output.

## When something goes wrong

- **Nothing happens / cursor blinks:** the command is still running. Wait, or press **Ctrl + C** to cancel.
- **"command not found":** the tool isn't installed yet, or the name was mistyped. The handbook's install
  pages cover each tool.
- **Permission denied:** you may need to run as administrator (Windows: right-click PowerShell → "Run as
  administrator") or use `sudo` (macOS/Linux) before the command.
