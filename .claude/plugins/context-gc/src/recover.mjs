// context-gc — recovery entrypoint (SessionStart hook).
//
// Thin orchestrator ONLY: parse the stdin payload, guard on `source === "compact"`, call each
// source module (config -> git -> transcript -> [ollama] -> manifest), and print the
// additionalContext JSON. No sourcing or formatting logic lives here (locality) — that belongs
// to git.mjs, transcript.mjs, ollama.mjs, and manifest.mjs respectively. This module owns
// exactly the wiring between them.
//
// `recover(payload)` is the pure(-ish), fully SYNCHRONOUS, deterministic-only orchestration
// function — files + tasks, no Ollama — that Task 5's tests already call directly and continue
// to rely on unchanged. `recoverEnriched(payload, options?)` is the async orchestrator that adds
// local-Ollama enrichment ON TOP of the same deterministic sources: it is the ONLY thing that
// changes behavior in this file for Task 6, added rather than grafted onto `recover()`, so the
// deterministic-only path stays trivially available (and trivially testable with no `await`) as
// its own guaranteed floor — never entangled with the enrichment leaf's async/network surface.
// `main()` is the thin process-facing shell (stdin -> recoverEnriched -> stdout) that only runs
// when this file is the process entry point, never on import; it uses the enriched path because
// that is what a real hook run should attempt.
//
// Replaces the Task 0 spike (which proved additionalContext survives a real compact + resume);
// that proof is why this module trusts the same hookSpecificOutput/additionalContext shape.

import { readFileSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { getConfig } from './config.mjs';
import { getChangedFiles } from './git.mjs';
import { readTranscript } from './transcript.mjs';
import { enrich } from './ollama.mjs';
import { buildManifest } from './manifest.mjs';

/**
 * Calls `getChangedFiles(cwd)`, degrading to `[]` on any unexpected throw. `getChangedFiles` is
 * already documented fail-open (never throws), but this call is still wrapped per the
 * architectural constraint that each source call degrades only its own field, not the run — a
 * defense-in-depth seam, not a sign the source contract is expected to break.
 */
function safeGetChangedFiles(cwd) {
  try {
    return getChangedFiles(cwd);
  } catch {
    return [];
  }
}

/**
 * Calls `readTranscript(transcriptPath, tailRecords)`, degrading to `{tail: [], tasks: []}` on
 * any unexpected throw (same defense-in-depth rationale as `safeGetChangedFiles`).
 */
function safeReadTranscript(transcriptPath, tailRecords) {
  try {
    return readTranscript(transcriptPath, tailRecords);
  } catch {
    return { tail: [], tasks: [] };
  }
}

/**
 * Gathers the deterministic sources (config, changed files, transcript tail/tasks) for one
 * payload — the shared first step of both `recover()` and `recoverEnriched()`. Not itself
 * wrapped in a try/catch: each source call already degrades independently
 * (`safeGetChangedFiles`/`safeReadTranscript`), and the remaining surface (`payload.cwd` /
 * `payload.transcript_path` property access, `getConfig()`) is cheap enough that callers wrap
 * this whole call in their own fail-open `try`/`catch` rather than duplicating one here.
 * @param {{cwd?: string, transcript_path?: string}} payload
 * @returns {{config: object, files: Array, tail: Array, tasks: Array}}
 */
function gatherDeterministicSources(payload) {
  const config = getConfig();
  const files = safeGetChangedFiles(payload.cwd);
  const { tail, tasks } = safeReadTranscript(payload.transcript_path, config.tailRecords);
  return { config, files, tail, tasks };
}

/**
 * Derives the recovery manifest string for one SessionStart hook payload — DETERMINISTIC ONLY,
 * no Ollama. This is the guaranteed floor: it never awaits anything and never touches the
 * network, so it stays trivially callable/testable synchronously. `recoverEnriched()` below adds
 * local-Ollama enrichment on top of the same sources; it does not replace this function.
 *
 * No-op (returns `''`) when `payload.source !== 'compact'`, or when there is nothing in flight
 * to report (both sources empty — `buildManifest` itself collapses that case to `''`). Otherwise
 * returns the deterministic markdown manifest built from the current git working-tree state and
 * the pre-compaction transcript tail/tasks.
 *
 * Never throws: the whole body is wrapped so any unexpected failure (e.g. `getConfig()` itself
 * misbehaving, a payload shape nobody anticipated) degrades to `''` rather than propagating —
 * this is the orchestrator's own fail-open floor, on top of (not instead of) each source module's
 * own fail-open contract.
 *
 * @param {{source?: string, cwd?: string, transcript_path?: string}} payload
 * @returns {string}
 */
export function recover(payload) {
  try {
    if (!payload || payload.source !== 'compact') return '';

    const { config, files, tasks } = gatherDeterministicSources(payload);
    const manifest = buildManifest({ files, tasks }, config.maxBytes);

    return typeof manifest === 'string' ? manifest : '';
  } catch {
    return '';
  }
}

/**
 * Calls `enrich()`, degrading to `null` on any unexpected throw (same defense-in-depth rationale
 * as `safeGetChangedFiles`/`safeReadTranscript`) — `enrich()` is already documented fail-open
 * (see ollama.mjs), but this wrapper keeps the "each source degrades only its own field, never
 * the run" contract enforced uniformly at this orchestration layer regardless of how carefully
 * the leaf itself is written.
 * @param {Array<{role: string, text: string}>} tail
 * @param {{ollamaModel: string, ollamaHost: string, timeoutMs: number}} config
 * @param {typeof fetch} [fetchImpl]
 */
async function safeEnrich(tail, config, fetchImpl) {
  try {
    return await enrich({
      tail,
      model: config.ollamaModel,
      host: config.ollamaHost,
      timeoutMs: config.timeoutMs,
      ...(fetchImpl ? { fetchImpl } : {}),
    });
  } catch {
    return null;
  }
}

/**
 * Same as `recover()`, but additionally attempts local-Ollama enrichment (objective +
 * decisions/why, tagged model-inferred by `manifest.mjs`) and merges it into the manifest when
 * available.
 *
 * Enrichment runs strictly AFTER the deterministic sources are gathered, and `buildManifest` is
 * called exactly ONCE with everything it has (per the "single-call, pure manifest" constraint —
 * this function never renders a manifest twice or patches enrichment into an already-rendered
 * string). `enrich()` is a fail-open leaf (see ollama.mjs): any failure there (Ollama
 * unreachable, non-200, timeout, malformed/empty response) resolves to `null`/`undefined`, in
 * which case `buildManifest` is called with the deterministic `{files, tasks}` only, so the
 * shipped manifest is byte-for-byte what `recover()` alone would have produced.
 *
 * `fetchImpl` (in `options`) is test-only dependency injection threaded through to `enrich()`;
 * real callers (`main()` below) omit it so `enrich()` uses the real global `fetch`.
 *
 * @param {{source?: string, cwd?: string, transcript_path?: string}} payload
 * @param {{fetchImpl?: typeof fetch}} [options]
 * @returns {Promise<string>}
 */
export async function recoverEnriched(payload, options = {}) {
  try {
    if (!payload || payload.source !== 'compact') return '';

    const { config, files, tail, tasks } = gatherDeterministicSources(payload);
    const enriched = await safeEnrich(tail, config, options.fetchImpl);

    const sources = enriched && typeof enriched === 'object'
      ? { files, tasks, objective: enriched.objective, decisions: enriched.decisions }
      : { files, tasks };

    const manifest = buildManifest(sources, config.maxBytes);
    return typeof manifest === 'string' ? manifest : '';
  } catch {
    return '';
  }
}

/**
 * Reads and parses the hook's stdin JSON payload, degrading to `{}` (never throwing) on a
 * missing/empty stdin or unparseable JSON — mirrors the fail-open posture of every module this
 * file orchestrates.
 */
function readStdinPayload() {
  try {
    const raw = readFileSync(0, 'utf8');
    return JSON.parse(raw || '{}');
  } catch {
    return {};
  }
}

/**
 * Process-facing entry point: reads the stdin payload, derives the manifest via
 * `recoverEnriched()` (deterministic sources + a best-effort local-Ollama enrichment pass), and
 * (only when non-empty) writes the `SessionStart` `additionalContext` JSON to stdout. Always
 * exits 0 — a hook that blocks or errors session resume is worse than one that silently no-ops.
 * `recoverEnriched()` is itself fully fail-open (see its own docs), so an absent/slow/erroring
 * Ollama here degrades quietly to the same deterministic manifest `recover()` would have
 * produced — this function does not need its own Ollama-specific fallback.
 *
 * Async (unlike Task 5's `main()`) because `recoverEnriched()` awaits the Ollama HTTP call.
 * Deliberately never calls `process.exit()`: on POSIX (Linux/macOS), a `process.stdout.write()`
 * to a piped fd is ASYNCHRONOUS (per Node's "note on process I/O"; Windows pipes are the
 * synchronous case), so `process.exit()` immediately after `write()` can terminate the process
 * before the OS finishes flushing — silently truncating the `additionalContext` JSON the harness
 * reads (the manifest can run up to `CONTEXT_GC_MAX_BYTES`, far past a single-chunk write).
 * `process.exitCode = 0` sets the eventual exit status without forcing an early exit; with stdin
 * fully consumed (`readFileSync(0, ...)` above) and no other pending handles, the event loop
 * drains stdout and the process exits naturally on its own once the write completes.
 */
async function main() {
  let manifest = '';
  try {
    manifest = await recoverEnriched(readStdinPayload());
  } catch {
    manifest = '';
  }

  if (typeof manifest === 'string' && manifest !== '') {
    try {
      process.stdout.write(JSON.stringify({
        hookSpecificOutput: { hookEventName: 'SessionStart', additionalContext: manifest },
      }));
    } catch {
      // Writing the output failed unexpectedly; fall through to the unconditional exit-0 below
      // rather than let a stdout error surface as a non-zero exit that could block resume.
    }
  }

  process.exitCode = 0;
}

const isEntryPoint = process.argv[1] !== undefined
  && import.meta.url === pathToFileURL(process.argv[1]).href;

if (isEntryPoint) {
  main();
}

export default recover;
