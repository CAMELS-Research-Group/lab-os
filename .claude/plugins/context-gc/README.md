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

`context-gc` is distributed through the lab's org-owned plugin marketplace, declared at
`.claude-plugin/marketplace.json` in this repository. Add the marketplace once, then install:

```
/plugin marketplace add CAMELS-Research-Group/lab-os
/plugin install context-gc@lab-os
```

## Enable / disable

The plugin ships one hook registration — a `SessionStart` hook matching `compact`, defined in
[`hooks/hooks.json`](hooks/hooks.json) (that file is the source of truth for the exact command;
it is not reproduced here, so the two cannot drift).

Installing the plugin registers that hook, so it runs automatically on every
`SessionStart(compact)` event — no separate opt-in step, no command to run. To disable recovery,
disable or remove the plugin so Claude Code no longer registers the hook.

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

- **Node** — the hook's only command is `node …`, making it the plugin's one hard, non-degradable
  dependency: unlike git and Ollama below, its absence fails the hook invocation itself rather than
  degrading a field. It is normally present wherever Claude Code runs, and the plugin needs no
  external npm dependencies.
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
