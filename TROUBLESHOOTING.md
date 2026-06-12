# TROUBLESHOOTING.md — lab-os

**Contract:** grep-only surface. Look up by symptom; do not read whole.
One section per gotcha, titled as the symptom you would search for.
Setup *steps* live in [`site/docs/getting-started.md`](site/docs/getting-started.md) (Manual path); entries here are failure modes, not walkthroughs.

**Adding entries:** an expensive finding or gotcha routes here (not to the project log) per the
§4.2 routing rule in
[`docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md`](docs/superpowers/specs/2026-06-10-logging-and-docs-standard-design.md).
When you hit something costly enough that the next person should not have to rediscover it, add a
section below: symptom as heading, cause, resolution.

---

## Rules files not loading in Cowork session (`01-workflow.md` / `02-data-protection.md` not visible)

**Cause:** The junction (Windows) or symlink (Unix) linking
`<DEV_ROOT>/.claude/rules` → `lab-os/.claude/rules` was not created, was created pointing at
the wrong target, or was deleted.

**Resolution:** Re-run the link step from
[Getting Started — Manual §3](site/docs/getting-started.md#3-wire-lab-os-into-cowork).

Verify: open a Cowork session at `<DEV_ROOT>` and ask "what are the lab's commit-message rules?" —
it should answer from `01-workflow.md`.

---

## Junction creation fails on Windows ("Access denied" or "file already exists")

**Cause (access denied):** `mklink /J` is called from PowerShell directly. The `/J` flag requires
the command to run through `cmd.exe`.

**Cause (file already exists):** A previous (possibly broken) junction or directory already exists
at that path.

**Resolution:**
- Always prefix with `cmd /c`: `cmd /c mklink /J "<target>" "<source>"`
- If the path already exists: `Remove-Item -Force "$HOME\Development\.claude\rules"` then re-run.
  If it is a directory (not a junction), use `Remove-Item -Recurse -Force`.

Note: junctions on Windows behave like directories to most tools; `Get-Item` will show
`LinkType: Junction` if the link exists and is valid.

---

## Symlink creation fails on macOS / Linux (permission error or already exists)

**Cause:** Target path already exists (stale symlink or directory), or the parent `.claude/`
directory was not created first.

**Resolution:**
```bash
rm -f ~/Development/.claude/rules          # remove stale symlink
```
Then re-run the link step from
[Getting Started — Manual §3](site/docs/getting-started.md#3-wire-lab-os-into-cowork).

---

## Log archive diff fails or log-lint reports non-identical entries across platforms

**Cause:** `autocrlf` or editor defaults write CRLF line endings on Windows and LF on macOS/Linux.
When an archived entry from a Windows clone is compared byte-for-byte against the original from a
Unix clone, the bytes differ even though the content is identical.

**Implication for log archival:** the `log-lint` immutability check requires archived entries to
be **byte-identical modulo EOL normalization** — meaning CRLF and LF are treated as equivalent
when comparing a pre-existing entry against its copy in `project_log_archive.md`. If your tooling
performs a raw byte comparison without EOL normalization, it will false-positive on cross-platform
moves. The `log-lint` script normalizes before comparing; external scripts must do the same.

**Resolution:**
- Ensure your repo has `.gitattributes` with `* text=auto` (or explicit `*.md text eol=lf`) so
  Git normalizes to LF on commit regardless of `core.autocrlf` setting.
- If a manual archive move was done on a Windows machine and the diff is noisy: re-open the
  affected file in an editor configured to write LF, or run:
  ```powershell
  (Get-Content "project_log_archive.md") | Set-Content -Encoding utf8 "project_log_archive.md"
  ```
  Note: PowerShell's `Set-Content` defaults to CRLF in Windows PowerShell 5.1; pipe through
  `[System.IO.File]::WriteAllText` for guaranteed LF, or use Git's `git add --renormalize`.
- The canonical fix: `git add --renormalize .` after confirming `.gitattributes` is set, then
  commit. This re-encodes all tracked text files to their declared endings.

---

## Shell command in docs fails on Windows but works on macOS / Linux (path separator)

**Cause:** Docs written for POSIX use forward-slash path separators (`/`) and POSIX-style
relative paths. Windows PowerShell accepts `/` in most contexts (it is not the issue), but the
**directory separator in string interpolation and some native CLI tools** defaults to `\`, and
scripts that embed paths in strings (grep patterns, Python `os.path`, config files) can break.

**Resolution:**
- In PowerShell, use `Join-Path` rather than string concatenation for paths:
  `Join-Path $HOME "Development\lab-os"` rather than `"$HOME/Development/lab-os"`.
- In Python code targeting both platforms: use `pathlib.Path` throughout; never concatenate
  separators by hand.
- In docs and shell snippets targeting both platforms: forward slash (`/`) is safe for
  PowerShell and is the lab's preferred separator in documentation. Only use `\` in
  PowerShell-specific blocks and mark them `# PowerShell`.

---

## Shell snippet in docs works on bash/zsh but fails in PowerShell (quoting / variable syntax)

**Cause:** POSIX shell and PowerShell have incompatible quoting and variable-expansion rules:

| Behavior | POSIX (`bash`/`zsh`) | PowerShell |
|---|---|---|
| Variable expansion | `$VAR` or `${VAR}` | `$env:VAR` (environment) / `$var` (local) |
| Single quotes | Literal — no expansion | Literal — no expansion (same) |
| Double quotes | Expands `$VAR`, `` ` `` escapes | Expands `$var`, backtick `` ` `` is escape char |
| Command substitution | `$(cmd)` or `` `cmd` `` | `$(cmd)` (subexpression) |
| Multiline strings | heredoc `<<'EOF'` | `@'...'@` here-string (closing `'@` must be at column 0) |
| Exit-code check | `$?` (0 = success) | `$?` (`$true` = success) or `$LASTEXITCODE` for native exes |

**Resolution:**
- Lab docs provide **both** blocks when the commands are non-trivial, labelled `# PowerShell`
  and `# macOS / Linux` — match the pattern in
  [`site/docs/getting-started.md`](site/docs/getting-started.md).
- When writing a single snippet intended for both: prefer constructs that work in both
  (`$(cmd)` subexpressions, forward-slash paths, explicit quoting of paths with spaces). Test
  on both before committing.
- The most common source of confusion: `$HOME` works in both, but `~` expands reliably in
  PowerShell only in a limited set of contexts (e.g. `Set-Location ~` works; string
  interpolation `"$HOME"` is safer than `"~\path"`).

---

## Plugin not listed after `/plugin install` (`pr-review-loop` or `superpowers` absent)

**Cause:** `/plugin install` was run but `/reload-plugins` was not, or the marketplace source
was not added first.

**Resolution:** Re-run the full sequence from
[Getting Started — Manual §5](site/docs/getting-started.md#5-install-the-lab-plugins), then confirm with `/plugin`.

---

## `CLAUDE.md` template placeholders still showing ("About Me" says `<your name>`)

**Cause:** The template was copied but the `<...>` placeholders in the About Me block were not
filled in.

**Resolution:** Open `~/.claude/CLAUDE.md` (Windows: `C:\Users\<you>\.claude\CLAUDE.md`), fill
in every `<...>` placeholder. See
[Getting Started — Manual §4a](site/docs/getting-started.md#4a-personal-global-your-persona-applies-in-every-session-every-project).
