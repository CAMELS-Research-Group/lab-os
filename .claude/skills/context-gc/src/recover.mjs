// context-gc — recovery entrypoint (SessionStart hook).
//
// Thin orchestrator ONLY: parse the stdin payload, guard on `source === "compact"`, call each
// source module (config -> git -> transcript -> [ollama] -> manifest), and print the
// additionalContext JSON. No sourcing or formatting logic lives here (locality) — that belongs
// to git.mjs, transcript.mjs, ollama.mjs, and manifest.mjs respectively. This module owns
// exactly the wiring between them.
//
// `recover(payload)` is the pure(-ish), fully SYNCHRONOUS, deterministic-only orchestration
// function — files + tasks, no Ollama. `recoverEnriched(payload, options?)` is the async
// orchestrator that adds local-Ollama enrichment ON TOP of the same deterministic sources; it is
// added alongside `recover()` rather than grafted onto it, so the deterministic-only path stays
// trivially available (and trivially testable with no `await`) as its own guaranteed floor,
// never entangled with the enrichment leaf's async/network surface. `main()` is the thin
// process-facing shell (stdin -> recoverEnriched -> stdout) that only runs when this file is the
// process entry point, never on import; it uses the enriched path because that is what a real
// hook run should attempt.
//
// The `hookSpecificOutput`/`additionalContext` shape below is the one verified to survive a real
// compact-and-resume cycle end to end; it is not inferred from documentation alone.

import { readFileSync, realpathSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
import { getConfig, isDebugEnabled } from './config.mjs';
import { getChangedFiles } from './git.mjs';
import { readTranscript } from './transcript.mjs';
import { enrich } from './ollama.mjs';
import { buildManifest } from './manifest.mjs';

/**
 * Reports a degradation on stderr when the debug flag is enabled (`CONTEXT_GC_DEBUG=1`).
 *
 * Fail-open and fail-SILENT are separate choices, and only the first is forced by the exit-0
 * contract: a SessionStart hook's stderr does not affect its exit status and does not enter model
 * context, so there is room for a diagnostic without risking session resume. Without one, the
 * fragile transcript seam degrading after a harness format change is indistinguishable from "no
 * tasks were in flight", and a misconfigured Ollama host is indistinguishable from "the model
 * found nothing" — permanently, and for every user.
 *
 * Off by default: a hook that chatters on every resume is its own problem.
 *
 * @param {string} stage the module or step that degraded
 * @param {unknown} [detail] optional error or value, rendered best-effort
 */
function reportDegradation(stage, detail) {
  try {
    if (!isDebugEnabled()) return;
    // Only the error's NAME/CODE is rendered, never its message: V8 embeds a prefix of the
    // offending input in JSON parse errors, and the stdin payload carries `cwd`,
    // `transcript_path`, and session identifiers.
    const label = detail && (detail.code || detail.name);
    process.stderr.write(`[context-gc] degraded at ${stage}${label ? `: ${label}` : ''}\n`);
  } catch {
    // A diagnostic that throws must never become the failure it is reporting on.
  }
}

/**
 * Calls `getChangedFiles(cwd)`, degrading to `[]` on any unexpected throw. `getChangedFiles` is
 * already documented fail-open (never throws), but this call is still wrapped per the
 * architectural constraint that each source call degrades only its own field, not the run — a
 * defense-in-depth seam, not a sign the source contract is expected to break.
 */
function safeGetChangedFiles(cwd) {
  try {
    return getChangedFiles(cwd);
  } catch (error) {
    reportDegradation('git.getChangedFiles', error);
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
  } catch (error) {
    reportDegradation('transcript.readTranscript', error);
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
 * own fail-open contract. The failure is reported on the `CONTEXT_GC_DEBUG` channel, mirroring
 * `recoverEnriched()`, so degrading here is not also silent.
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
  } catch (error) {
    reportDegradation('recover', error);
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
  } catch (error) {
    reportDegradation('ollama.enrich', error);
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
  } catch (error) {
    reportDegradation('recoverEnriched', error);
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
  } catch (error) {
    reportDegradation('readStdinPayload', error);
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
 * Async because `recoverEnriched()` awaits the Ollama HTTP call.
 *
 * Deliberately never calls `process.exit()`. Per Node's "note on process I/O", writes to a PIPE
 * are synchronous on Linux and Windows but ASYNCHRONOUS on macOS — and a hook's stdout is always
 * a pipe. So `process.exit()` immediately after `write()` can terminate the process before the OS
 * finishes flushing, silently truncating the `additionalContext` JSON the harness reads (the
 * manifest can run up to `CONTEXT_GC_MAX_BYTES`, far past a single-chunk write). macOS is the
 * platform that loses the race; the guard is unconditional because the cost of being wrong about
 * the platform is a corrupt manifest.
 *
 * `process.exitCode = 0` sets the eventual exit status without forcing an early exit; with stdin
 * fully consumed (`readFileSync(0, ...)` above) and no other pending handles, the event loop
 * drains stdout and the process exits naturally once the write completes.
 */
async function main() {
  // Stream I/O errors on process.stdout/stderr (EPIPE, a closed read end, a destroyed pipe) are
  // delivered as an 'error' EVENT on every platform, never as a throw from write() — so the
  // try/catch below cannot see THAT class. (A synchronous throw from write() is a different
  // class — an already-destroyed stream, an invalid argument — and IS catchable; the catch below
  // exists for it. The two are complementary, not contradictory.) With no listener,
  // Node promotes the event to an uncaught exception and a non-zero exit: precisely the
  // resume-blocking outcome this function exists to avoid. (This is unrelated to the sync/async
  // split above, which governs FLUSH timing, not error delivery — the listener is unconditional
  // insurance, and gating it behind a platform check would reintroduce the failure elsewhere.)
  //
  // stderr needs the same guard as stdout: the diagnostic channel writes there, so without it a
  // closed stderr would let the reporter added to make failures visible become one.
  process.stdout.on('error', () => {
    reportDegradation('stdout');
  });
  process.stderr.on('error', () => {
    // Nothing to report the failure ON at this point; swallowing is the only non-fatal option.
  });

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
    } catch (error) {
      // A synchronous throw from write() (e.g. an already-destroyed stream). Stream 'error'
      // events are covered by the listener above; between them the write cannot exit non-zero.
      reportDegradation('stdout.write', error);
    }
  }

  process.exitCode = 0;
}

/**
 * Whether this module is the process entry point — i.e. the hook was invoked, rather than the
 * module imported by a test.
 *
 * Compares REALPATHS as a fallback, and that is the whole point of the function. Node resolves an
 * ESM entry module through symlinks before setting `import.meta.url`, but leaves `process.argv[1]`
 * exactly as the process was handed it. `link-lab-assets.sh` deploys this plugin as a DIRECTORY
 * SYMLINK at `~/.claude/skills/context-gc`, and that symlink is what makes the hook always-on — so
 * on every real installation `argv[1]` is the symlinked path while `import.meta.url` is the repo
 * realpath. A plain string compare is `false` there, `main()` never runs, and the process exits 0
 * having written nothing.
 *
 * That failure is invisible by construction: a hook that never fires is indistinguishable from a
 * session with nothing to recover, and `CONTEXT_GC_DEBUG=1` says nothing either, because the
 * diagnostic lives inside the `main()` that never ran. It is pinned by a wiring test that spawns
 * the plugin THROUGH the deployed symlink, because no amount of testing the module in place can
 * observe it.
 *
 * The plain comparison is tried first and the realpath comparison is the fallback, so an
 * unstattable `argv[1]` can only ever forfeit the symlink fix — it can never make this stricter
 * than the plain compare was.
 *
 * @returns {boolean}
 */
function resolveIsEntryPoint() {
  const invokedAs = process.argv[1];
  if (invokedAs === undefined) return false;
  if (import.meta.url === pathToFileURL(invokedAs).href) return true;
  try {
    return import.meta.url === pathToFileURL(realpathSync(invokedAs)).href;
  } catch {
    return false;
  }
}

const isEntryPoint = resolveIsEntryPoint();

if (isEntryPoint) {
  // main() is async, so a rejection escaping it would become an unhandled rejection and a
  // non-zero exit — the one outcome the always-exit-0 contract exists to prevent. Nothing inside
  // is expected to reject; this is the backstop that makes "always exits 0" unconditional.
  main().catch(() => {
    process.exitCode = 0;
  });
}
