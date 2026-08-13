// context-gc — config module.
//
// Single canonical owner of tunables (env names + defaults). No other module in this plugin
// reads `process.env` directly — everything downstream calls `getConfig()` (or is handed a
// resolved config object) instead. Keeping the parsing here means there is exactly one place
// to change a default or add a new `CONTEXT_GC_*` variable.

const DEFAULTS = Object.freeze({
  ollamaModel: 'hermes3:8b',
  ollamaHost: 'http://127.0.0.1:11434',
  tailRecords: 40,
  timeoutMs: 20000,
  maxBytes: 4000,
});

/**
 * Resolves a string tunable: missing or blank (empty/whitespace-only) falls back to `fallback`.
 * Returns the TRIMMED value — surrounding whitespace in an env var is always a typo, and an
 * untrimmed host would go on to build a malformed URL whose failure is indistinguishable from
 * "Ollama isn't running".
 */
function resolveString(rawValue, fallback) {
  if (rawValue === undefined || rawValue === null) return fallback;
  const trimmed = String(rawValue).trim();
  return trimmed === '' ? fallback : trimmed;
}

/**
 * Resolves an integer tunable: missing/blank, non-numeric, non-finite, non-integer, or
 * out-of-range values all fall back to `fallback` rather than throwing.
 *
 * Every tunable this plugin resolves is a positive quantity — a byte cap, a record count, a
 * timeout. Zero and negatives are therefore rejected rather than passed through: each one
 * silently disables a whole feature with no diagnostic (`CONTEXT_GC_MAX_BYTES=0` makes the
 * manifest permanently empty; a non-positive `CONTEXT_GC_TAIL_RECORDS` empties the transcript
 * window and, with it, enrichment). A typo should cost the default, not the plugin.
 */
function resolveInt(rawValue, fallback) {
  if (rawValue === undefined || rawValue === null) return fallback;
  const trimmed = String(rawValue).trim();
  if (trimmed === '') return fallback;
  const parsed = Number(trimmed);
  if (!Number.isInteger(parsed) || parsed <= 0) return fallback;
  return parsed;
}

/**
 * Resolves the context-gc plugin config from an env-like object (defaults to `process.env`).
 * This is the ONLY function in the plugin that reads environment variables; every other
 * module receives its config already resolved.
 *
 * Env vars (all optional; `DEFAULTS` above owns the values, this list only names the vars):
 * - `CONTEXT_GC_OLLAMA_MODEL`  (string)
 * - `CONTEXT_GC_OLLAMA_HOST`   (string)
 * - `CONTEXT_GC_TAIL_RECORDS`  (positive integer)
 * - `CONTEXT_GC_TIMEOUT_MS`    (positive integer)
 * - `CONTEXT_GC_MAX_BYTES`     (positive integer)
 *
 * @param {NodeJS.ProcessEnv} [env]
 * @returns {{ollamaModel: string, ollamaHost: string, tailRecords: number, timeoutMs: number, maxBytes: number}}
 */
export function getConfig(env = process.env) {
  return {
    ollamaModel: resolveString(env.CONTEXT_GC_OLLAMA_MODEL, DEFAULTS.ollamaModel),
    ollamaHost: resolveString(env.CONTEXT_GC_OLLAMA_HOST, DEFAULTS.ollamaHost),
    tailRecords: resolveInt(env.CONTEXT_GC_TAIL_RECORDS, DEFAULTS.tailRecords),
    timeoutMs: resolveInt(env.CONTEXT_GC_TIMEOUT_MS, DEFAULTS.timeoutMs),
    maxBytes: resolveInt(env.CONTEXT_GC_MAX_BYTES, DEFAULTS.maxBytes),
  };
}
