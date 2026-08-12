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
 */
function resolveString(rawValue, fallback) {
  if (rawValue === undefined || rawValue === null) return fallback;
  const trimmed = String(rawValue).trim();
  return trimmed === '' ? fallback : rawValue;
}

/**
 * Resolves an integer tunable: missing/blank, non-numeric, non-finite, or non-integer values
 * all fall back to `fallback` rather than throwing.
 */
function resolveInt(rawValue, fallback) {
  if (rawValue === undefined || rawValue === null) return fallback;
  const trimmed = String(rawValue).trim();
  if (trimmed === '') return fallback;
  const parsed = Number(trimmed);
  return Number.isInteger(parsed) ? parsed : fallback;
}

/**
 * Resolves the context-gc plugin config from an env-like object (defaults to `process.env`).
 * This is the ONLY function in the plugin that reads environment variables; every other
 * module receives its config already resolved.
 *
 * Env vars (all optional, documented defaults):
 * - `CONTEXT_GC_OLLAMA_MODEL`  (string)  default "hermes3:8b"
 * - `CONTEXT_GC_OLLAMA_HOST`   (string)  default "http://127.0.0.1:11434"
 * - `CONTEXT_GC_TAIL_RECORDS`  (integer) default 40
 * - `CONTEXT_GC_TIMEOUT_MS`    (integer) default 20000
 * - `CONTEXT_GC_MAX_BYTES`     (integer) default 4000
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

export default getConfig;
