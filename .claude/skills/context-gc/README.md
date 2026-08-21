# context-gc

Re-injects lost session state after Claude Code auto-compacts the conversation.

## What it does

Claude Code compacts the conversation automatically as it approaches the context limit. That
compaction is **harness-owned** — this plugin does not decide when it fires, does not shape what
the summary keeps, and cannot trigger `/compact` programmatically. There is no lever here for
*controlling* compaction.

What this plugin does instead is **recover** what compaction silently drops: which files were in
flight, the active task list, and (best-effort) the session's objective and the load-bearing
decisions behind it. It fires on `SessionStart` with `source: "compact"` — the hook Claude Code
runs right after a compaction, including an inline mid-session auto-compaction — and prints a
recovery manifest to stdout as `additionalContext`, the mechanism a `SessionStart` hook uses to add
text back into the model's context on resume.

Recovery, not control. If compaction never fires, this hook never runs.

## The manifest: a deterministic floor plus an enriched layer

The manifest has two tiers, sourced independently and rendered with a strict priority order under
the byte cap (see `CONTEXT_GC_MAX_BYTES` below):

- **Deterministic floor** — always sourceable, never model-inferred:
  - **Files in flight**: modified/untracked paths from `git status --porcelain -z` in the session's
    working tree.
  - **Tasks**: the most recent parseable task list (`TodoWrite`) found in the transcript window
    immediately preceding the compaction marker.
- **Enriched layer** — model-inferred, distilled by a **local** Ollama model over the same
  pre-compaction transcript tail:
  - **Objective**: a one-sentence read on what the session was working toward.
  - **Decisions / open threads**: short decisions-with-why, grounded in and traceable to the
    transcript excerpt the model was given.

Enriched content always renders under a single visible heading tag —
`model-inferred — local Ollama; verify before treating as fact` — so the resumed agent treats it as
a hint to verify, never as settled fact. Under byte pressure, enriched content is trimmed first
(decisions, then objective, then tasks, then files) — the deterministic floor is the last thing to
give.

### Enriched-only recovery on a clean tree (intentional)

If the git working tree is clean and no todos are in flight, but the transcript tail is rich (e.g.
a research or planning conversation with no file edits yet), the deterministic floor is empty —
but the manifest still ships, containing **only** the model-inferred objective and decisions. This
is deliberate: it is exactly where enrichment earns its place, recovering the thread of a
conversation that never touched a file. The content is still tagged model-inferred, so it is
recovered as a hint, not asserted as fact.

## Fail-open, always

Every source degrades independently and never blocks session resume:

- No git / not a repo / `git` not on PATH → files list degrades to empty.
- No `isCompactSummary` marker in the transcript, unreadable transcript, or unparseable task
  records → tasks (and the enrichment tail) degrade to empty.
- Ollama unreachable, slow past `CONTEXT_GC_TIMEOUT_MS`, erroring, or returning malformed output →
  the enriched layer degrades to absent; the deterministic floor ships unchanged.

A cold Ollama model load can routinely exceed the timeout on the *first* compaction of a session
(before the model is warm) — that is expected fail-open-to-floor behavior, not a bug.

The hook always injects whatever it has, and if every source comes up empty, it injects nothing
(no bare/lonely manifest header) and exits 0 either way — a hook that blocks or errors session
resume is worse than one that silently no-ops.

## Install

There is nothing to install. `context-gc` is **vendored**, like every other asset under
`.claude/skills/` — a clone or fork of this repo carries the plugin itself, with no marketplace to
add and no install step to run (`.claude/skills/ATTRIBUTION.md`).

It is a plugin, not a skill: the `.claude-plugin/plugin.json` manifest is what makes Claude Code
load this directory as `context-gc@skills-dir` rather than as a model-invocable skill. Only a
plugin can register a hook, which is the whole point — recovery has to fire on an event, not wait
to be chosen.

Run the standard asset link script once per machine to deploy it user-scope:

```
bash .claude/scripts/link-lab-assets.sh
```

That symlink is **required for always-on coverage**, not a convenience. A project-scope
skills-dir plugin loads only from the `.claude/skills/` of the directory Claude Code was started
in and does not walk up to the repository root, so a session opened in a nested `projects/<repo>`
clone would never see it. Linked into `~/.claude/skills/`, it loads at personal scope in every
session on the machine. The plugin's [wiring test](test/wiring.test.mjs) asserts the link script
discovers it, because a hook that is never registered is indistinguishable from one that had
nothing to recover.

## Enable / disable

The plugin ships one hook registration — a `SessionStart` hook matching `compact`, defined in
[`hooks/hooks.json`](hooks/hooks.json) (that file is the source of truth for the exact command;
it is not reproduced here, so the two cannot drift).

Discovery registers that hook, so it runs automatically on every `SessionStart(compact)` event —
no separate opt-in step, no command to run. Changes to `hooks/hooks.json` need `/reload-plugins`
or a restart to take effect; only `SKILL.md` edits are picked up live, and this plugin ships none.

To disable recovery:

```
claude plugin disable context-gc@skills-dir
```

There is no `uninstall`, because nothing was installed from a marketplace — removing the symlink
from `~/.claude/skills/` (or the directory itself) also stops it loading.

## Configuration

All configuration is via environment variables; every one is optional and falls back to its
default when unset, blank, or — for the numeric ones — unparseable, non-integer, or not positive.
Zero and negative values are rejected rather than honoured: each would silently disable a feature
(a `0` byte cap empties the manifest; a non-positive tail window empties the transcript read and
with it enrichment), which is indistinguishable from having nothing to recover. String values are
trimmed. `src/config.mjs` is the single place these are read and defaulted.

| Variable | Default | Controls |
|---|---|---|
| `CONTEXT_GC_OLLAMA_MODEL` | `hermes3:8b` | The local Ollama model used for enrichment. An ~8B model is the default for fidelity: smaller models are more prone to reporting a *superseded* decision as current on a corrected-decision tail. Override with a smaller/faster model (e.g. `llama3.2:3b`) if you prefer lower latency and accept softer recovery. |
| `CONTEXT_GC_OLLAMA_HOST` | `http://127.0.0.1:11434` | Base URL of the local Ollama server (`/api/generate` is appended). |
| `CONTEXT_GC_TAIL_RECORDS` | `40` | Number of transcript records immediately preceding the compaction marker to read for task state and to feed the enrichment prompt. |
| `CONTEXT_GC_TIMEOUT_MS` | `20000` | Milliseconds before the Ollama call is aborted; a timeout degrades the enriched layer only (see Fail-open above). The budget is generous on purpose: it is the resume-stall ceiling, reached only when the model is genuinely slow (cold load, or the machine busy mid-compaction) — an *unreachable* Ollama still fails in milliseconds via connection-refused, never waiting out this cap. |
| `CONTEXT_GC_MAX_BYTES` | `4000` | Byte cap (UTF-8) on the rendered manifest; content is trimmed whole-line, softest (enriched) first, until it fits. |

## Requirements

- **Node 18+** — the hook's only command is `node …`, making it the plugin's one hard,
  non-degradable dependency: unlike git and Ollama below, its absence fails the hook invocation
  itself rather than degrading a field. It is normally present wherever Claude Code runs, and the
  plugin needs no external npm dependencies. **18** is the floor because the enrichment call uses
  the global `fetch`, unflagged from Node 18.0.0; nothing in `src/` needs anything newer.

  Two higher numbers apply to *developing* the plugin, not to running it, and are stated here so
  all three are in one place: the test command — `node --test "test/*.test.mjs"` from this
  directory, the form
  [`.github/workflows/plugin-tests.yml`](../../../.github/workflows/plugin-tests.yml) runs — needs
  **Node 21+**, because passing a glob to `--test` is a Node 21 feature (on 18–20 the pattern is
  taken as a literal path and matches nothing); and that workflow pins **Node 22**, so 22 is the
  only version the suite is actually verified on. A user running the hook is bound by 18 alone.
- **Git** — for the deterministic files layer; its absence degrades that field only, it doesn't
  block the hook.
- **Local Ollama** (optional) — for the enriched layer only. Pull the default model once with
  `ollama pull hermes3:8b` (or point `CONTEXT_GC_OLLAMA_MODEL` at a model you already have). Without
  Ollama (not installed, not running, or the configured model not pulled), the hook still ships the
  deterministic floor. A warm 8B model answers well inside the default 20s
  timeout; the *first* compaction of a session may cold-load past the timeout and fail open to the
  floor.

## Diagnostics

Every failure in this plugin degrades silently by design — that is what keeps a broken hook from
blocking session resume. The cost is that a genuinely broken plugin looks exactly like a session
with nothing to recover. Set `CONTEXT_GC_DEBUG=1` to have each degradation report itself on
stderr (which does not affect the hook's exit status and does not enter model context):

```
CONTEXT_GC_DEBUG=1
```

Reported stages include `git.getChangedFiles`, `transcript.readTranscript`, `ollama.enrich`,
`readStdinPayload`, and `stdout`. This is the first thing to reach for if the manifest stops
appearing, or if the Tasks section goes missing after a Claude Code upgrade — the transcript
format is harness-owned and undocumented, so a format change degrades this plugin quietly.

## Data protection

The injected manifest **re-enters the Claude session context on resume** — that is its entire
function. Its content therefore carries the same gated-data review as any other session content
(`.claude/rules/02-data-protection.md`): if the session touched gated-dataset material, that
material can resurface in the re-injected manifest the same way it could in any other part of the
conversation.

The Ollama distillation itself runs entirely on-machine — no metered API, no external network call
beyond the local Ollama host. Running Ollama locally avoids adding a *second* external sink; it
does not avoid the pre-existing sink, which is the Claude session itself that the manifest
re-enters by design.

### Scope of `05-agent-runtime.md`

`.claude/rules/05-agent-runtime.md` is a HARD RULE binding any lab-os asset that **hosts a
guardrailed local coding-agent runtime**. This plugin is the first lab-os asset with a local-model
surface at all, so the reading is stated here rather than left for each reader to re-derive: **it
does not bind.** No runtime is hosted — there is no agent engine, no control seam, and no run
state. Nothing here spawns `claude`; the sole hook command is `node src/recover.mjs`. The one model
call is a single read-only distillation of a transcript tail into text — one shot, no tools, no
loop, and no path by which the model's output reaches anything but the manifest string. The model
is local Ollama, so the rule's `$0-by-construction` constraint is met by construction too: no
Max/Pro OAuth token is driven through a third-party tool, and there is no metered API key to
introduce.

**The growth trigger.** That reading holds only while model output is *rendered*, never *acted on*.
The moment any change lets the enriched layer drive an action — invoking a tool, spawning a
process, writing a file, choosing what runs next — this asset is hosting a runtime and
`05-agent-runtime.md` binds it in full, including the halt contract, per-run caps, the fail-closed
permission allowlist, and the bot-identity requirement. Treat that as a review gate on any change
that gives `src/ollama.mjs`'s output a consumer other than `buildManifest`.
