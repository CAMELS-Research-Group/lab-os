// context-gc — local Ollama enrichment module.
//
// LEAF, FAIL-OPEN ONLY: this module attempts to distill a grounded, model-inferred
// `{objective, decisions}` summary from the normalized pre-compaction tail via a local Ollama
// `/api/generate` call. Every failure mode — host unreachable, non-2xx response, a network
// throw, a timeout/abort, or a response that isn't parseable/well-formed JSON — resolves to
// `null`; `enrich()` never throws. The only caller (`recover.mjs`'s `recoverEnriched`) already
// has a complete, shippable deterministic manifest before it calls this module, so a `null`
// result here costs the resumed agent an enrichment section, never the resume itself.
//
// Config values (model/host/timeoutMs) are passed in as plain arguments — this module never
// reads `process.env` (`config.mjs` is the sole owner of that). The HTTP transport is injectable
// via `fetchImpl` (defaults to the global `fetch`) so tests exercise every failure mode —
// including timeout/abort — without a live Ollama or real network access.
//
// The model-inferred TAG ITSELF is applied exactly once, in `manifest.mjs`'s render step: this
// module returns plain, untagged `{objective, decisions}` so the tag text is never duplicated
// inline across modules.

const GENERATE_PATH = '/api/generate';
const PROMPT_TAIL_CHAR_CAP = 12000; // bounds the prompt itself regardless of tailRecords config

/**
 * Renders the normalized tail as `role: text` lines for the prompt, truncated to
 * `PROMPT_TAIL_CHAR_CAP` characters (from the end — the most recent context matters most) so a
 * generously configured `tailRecords` can't build an unbounded prompt.
 * @param {Array<{role: string, text: string}>} tail
 * @returns {string}
 */
function renderTailForPrompt(tail) {
  const rendered = tail
    .map((entry) => `${entry.role}: ${typeof entry.text === 'string' ? entry.text : ''}`)
    .join('\n');
  return rendered.length > PROMPT_TAIL_CHAR_CAP
    ? rendered.slice(rendered.length - PROMPT_TAIL_CHAR_CAP)
    : rendered;
}

/**
 * Builds the `/api/generate` prompt: strict grounding instructions followed by the normalized
 * tail. The model is told to answer ONLY from the transcript given, to never invent a decision
 * it can't point to in the tail, and to return empty fields when the tail doesn't support a
 * claim. Grounding is enforced at the prompt level and paired with `temperature: 0` in the
 * request options (see `callOllama`) rather than by a post-hoc trace check: enriched fields must
 * be traceable to the tail they were derived from, and they always render under the
 * model-inferred tag (manifest.mjs) so a reader knows they are not verified fact.
 * @param {Array<{role: string, text: string}>} tail
 * @returns {string}
 */
function buildPrompt(tail) {
  return [
    'You are recovering context for a coding agent after its conversation history was',
    'compacted away. Base every claim STRICTLY on the transcript excerpt below — never invent a',
    'decision, task, or rationale that is not actually present in it. If the transcript does not',
    'clearly support an objective or any decisions, return empty values rather than guessing.',
    '',
    'Respond with ONLY a JSON object (no prose, no markdown code fences) matching exactly this',
    'shape: {"objective": "<one sentence, or empty string if unclear>", "decisions": ["<short',
    'decision or open thread, with its why, traceable to the transcript>", ...]}',
    '',
    'Transcript excerpt (oldest first):',
    renderTailForPrompt(tail),
  ].join('\n');
}

/**
 * Calls the local Ollama `/api/generate` endpoint (non-streaming, JSON-mode, low temperature)
 * and returns the model's raw `response` text, or `null` on any failure: non-2xx status,
 * timeout/abort (enforced via `AbortController` against `timeoutMs`), a network throw
 * (unreachable host, DNS failure, connection refused), or a response body that isn't JSON /
 * doesn't carry a string `response` field. Never throws.
 * @param {{host: string, model: string, prompt: string, timeoutMs: number, fetchImpl: typeof fetch}} args
 * @returns {Promise<string|null>}
 */
async function callOllama({ host, model, prompt, timeoutMs, fetchImpl }) {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const response = await fetchImpl(`${host}${GENERATE_PATH}`, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({
        model,
        prompt,
        stream: false,
        format: 'json',
        options: { temperature: 0 },
      }),
      signal: controller.signal,
    });

    if (!response || response.ok !== true) return null;

    const payload = await response.json();
    return payload && typeof payload.response === 'string' ? payload.response : null;
  } catch {
    // Covers: network throw (unreachable/DNS/refused), abort/timeout, and a body that fails to
    // parse as JSON at the transport level — all fold into the same fail-open outcome.
    return null;
  } finally {
    clearTimeout(timer);
  }
}

/**
 * Parses the model's raw `response` text into `{objective, decisions}`, or `null` when the text
 * is empty, isn't valid JSON, isn't an object, or carries neither a usable objective nor any
 * usable decisions — this is the "return empty/omit when the tail is insufficient" contract:
 * a model that (correctly) found nothing groundable in the tail produces the same `null` outcome
 * as a model that errored.
 * @param {string|null} responseText
 * @returns {{objective: string|null, decisions: string[]}|null}
 */
function parseEnrichment(responseText) {
  if (typeof responseText !== 'string' || responseText.trim() === '') return null;

  let parsed;
  try {
    parsed = JSON.parse(responseText);
  } catch {
    return null;
  }
  if (!parsed || typeof parsed !== 'object') return null;

  const objective = typeof parsed.objective === 'string' ? parsed.objective.trim() : '';
  const decisions = Array.isArray(parsed.decisions)
    ? parsed.decisions
        .filter((decision) => typeof decision === 'string' && decision.trim() !== '')
        .map((decision) => decision.trim())
    : [];

  if (objective === '' && decisions.length === 0) return null;

  return { objective: objective === '' ? null : objective, decisions };
}

/**
 * Attempts a local-Ollama distillation of the normalized `tail` into a grounded, untagged
 * `{objective, decisions}` summary. Fail-open across the board: an empty/missing `tail`, a
 * missing `host`/`model`, an unreachable Ollama, a non-2xx response, a timeout past `timeoutMs`
 * (a first-run cold model-load can routinely exceed a short timeout — that is expected fail-open
 * behavior, not a defect), or a malformed/empty model response all resolve to `null`. Never
 * throws — callers (`recover.mjs`) treat `null` as "nothing to add, ship the floor unchanged."
 *
 * `timeoutMs` is a REQUIRED resolved value, not defaulted here: `config.mjs` is the single owner
 * of the timeout default, so a missing/non-positive `timeoutMs` fails open to `null` rather than
 * this leaf inventing its own (would-be-stale) fallback.
 *
 * @param {{tail: Array<{role: string, text: string}>, model: string, host: string,
 *   timeoutMs: number, fetchImpl?: typeof fetch}} args
 * @returns {Promise<{objective: string|null, decisions: string[]}|null>}
 */
export async function enrich({ tail, model, host, timeoutMs, fetchImpl = fetch } = {}) {
  try {
    if (!Array.isArray(tail) || tail.length === 0) return null;
    if (typeof host !== 'string' || host.trim() === '') return null;
    if (typeof model !== 'string' || model.trim() === '') return null;
    if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) return null;

    const prompt = buildPrompt(tail);
    const responseText = await callOllama({ host, model, prompt, timeoutMs, fetchImpl });

    return parseEnrichment(responseText);
  } catch {
    return null;
  }
}
