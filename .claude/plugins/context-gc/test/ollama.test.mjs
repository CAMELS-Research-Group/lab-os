// context-gc — tests for src/ollama.mjs
// ollama.mjs is the local-Ollama enrichment leaf: it must NEVER throw and must fail open to
// `null` on every failure mode (unreachable, non-200, network throw, timeout/abort, malformed or
// empty response). Every test here injects a fake `fetchImpl` — none require a live Ollama or
// touch the real network: every failure mode is driven through an injected fetchImpl.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { enrich } from '../src/ollama.mjs';

const TAIL = [
  { role: 'user', text: 'add retry logic to the fetch wrapper' },
  { role: 'assistant', text: 'decided to use exponential backoff with a 3-attempt cap' },
];

/** Builds a fetch-shaped success response resolving to `{response: JSON.stringify(body)}`. */
function jsonResponse(body) {
  return {
    ok: true,
    status: 200,
    json: async () => ({ response: JSON.stringify(body) }),
  };
}

test('success: returns {objective, decisions} parsed from the model response', async () => {
  const fetchImpl = async () => jsonResponse({
    objective: 'add retry logic to the fetch wrapper',
    decisions: ['use exponential backoff with a 3-attempt cap'],
  });

  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.deepEqual(result, {
    objective: 'add retry logic to the fetch wrapper',
    decisions: ['use exponential backoff with a 3-attempt cap'],
  });
});

test('success: honors the configured model/host and calls the /api/generate endpoint', async () => {
  let capturedUrl;
  let capturedBody;
  const fetchImpl = async (url, opts) => {
    capturedUrl = url;
    capturedBody = JSON.parse(opts.body);
    return jsonResponse({ objective: 'do the thing', decisions: [] });
  };

  await enrich({
    tail: TAIL,
    model: 'qwen2.5:7b',
    host: 'http://localhost:9999',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.equal(capturedUrl, 'http://localhost:9999/api/generate');
  assert.equal(capturedBody.model, 'qwen2.5:7b');
  assert.equal(capturedBody.stream, false);
  assert.equal(capturedBody.format, 'json');
});

test('grounding: the prompt sent to the model includes the tail content and grounding instructions', async () => {
  let capturedBody;
  const fetchImpl = async (url, opts) => {
    capturedBody = JSON.parse(opts.body);
    return jsonResponse({ objective: '', decisions: [] });
  };

  await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.match(capturedBody.prompt, /add retry logic to the fetch wrapper/);
  assert.match(capturedBody.prompt, /exponential backoff/);
  assert.match(capturedBody.prompt, /STRICTLY on the/i);
  assert.match(capturedBody.prompt, /never invent/i);
});

test('non-200: fails open to null', async () => {
  const fetchImpl = async () => ({ ok: false, status: 500, json: async () => ({}) });

  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.equal(result, null);
});

test('network throw / unreachable host: fails open to null, never throws', async () => {
  const fetchImpl = async () => {
    throw new Error('ECONNREFUSED 127.0.0.1:11434');
  };

  await assert.doesNotReject(async () => {
    const result = await enrich({
      tail: TAIL,
      model: 'llama3.2:3b',
      host: 'http://127.0.0.1:11434',
      timeoutMs: 1000,
      fetchImpl,
    });
    assert.equal(result, null);
  });
});

test('timeout/abort: fails open to null within the configured timeout, never hangs', async () => {
  // Never resolves on its own; only rejects when the AbortController fires, mirroring how the
  // real global `fetch` behaves against an aborted signal.
  let sawRealSignal = false;
  let abortFired = false;
  const fetchImpl = (url, opts) => new Promise((_resolve, reject) => {
    // Assert the wiring explicitly. Without this, removing `signal: controller.signal` from the
    // fetch options makes `opts.signal` undefined, the line below THROWS, enrich()'s outer catch
    // swallows it, and the test still sees null-and-fast — passing while a real hung Ollama
    // would stall session resume forever.
    sawRealSignal = opts && opts.signal instanceof AbortSignal;
    opts.signal.addEventListener('abort', () => {
      abortFired = true;
      const err = new Error('The operation was aborted');
      err.name = 'AbortError';
      reject(err);
    });
  });

  const start = Date.now();
  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 50,
    fetchImpl,
  });
  const elapsed = Date.now() - start;

  assert.ok(sawRealSignal, 'fetch was called without a real AbortSignal — the deadline is not wired');
  assert.ok(abortFired, 'the abort never fired — the timeout did not drive the cancellation');
  assert.equal(result, null);
  assert.ok(elapsed < 2000, `expected the abort to resolve quickly, took ${elapsed}ms`);
});

test('malformed response: model response text is not valid JSON fails open to null', async () => {
  const fetchImpl = async () => ({
    ok: true,
    status: 200,
    json: async () => ({ response: 'this is not json' }),
  });

  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.equal(result, null);
});

test('empty response: an empty string response fails open to null', async () => {
  const fetchImpl = async () => ({ ok: true, status: 200, json: async () => ({ response: '' }) });

  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.equal(result, null);
});

test('empty response body: response payload missing the "response" field fails open to null', async () => {
  const fetchImpl = async () => ({ ok: true, status: 200, json: async () => ({}) });

  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.equal(result, null);
});

test('insufficient tail: a model response with empty objective and no decisions fails open to null', async () => {
  const fetchImpl = async () => jsonResponse({ objective: '', decisions: [] });

  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.equal(result, null);
});

test('empty tail: returns null without ever calling the transport', async () => {
  let called = false;
  const fetchImpl = async () => {
    called = true;
    return jsonResponse({ objective: 'x', decisions: [] });
  };

  const result = await enrich({
    tail: [],
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.equal(result, null);
  assert.equal(called, false);
});

test('missing/invalid timeout: fails open to null without calling the transport', async () => {
  // config.mjs is the sole owner of the timeout default; this leaf carries none, so a
  // missing/non-positive resolved timeout fails open BEFORE any generate call rather than
  // inventing a would-be-stale fallback.
  for (const timeoutMs of [undefined, 0, -1, NaN]) {
    let called = false;
    const fetchImpl = async () => {
      called = true;
      return jsonResponse({ objective: 'x', decisions: [] });
    };
    const result = await enrich({
      tail: TAIL,
      model: 'llama3.2:3b',
      host: 'http://127.0.0.1:11434',
      timeoutMs,
      fetchImpl,
    });
    assert.equal(result, null, `timeoutMs=${timeoutMs} should fail open`);
    assert.equal(called, false, `timeoutMs=${timeoutMs} should not call the transport`);
  }
});

test('missing host/model: fails open to null without throwing', async () => {
  await assert.doesNotReject(async () => {
    const result = await enrich({ tail: TAIL, model: '', host: '', timeoutMs: 1000, fetchImpl: async () => jsonResponse({}) });
    assert.equal(result, null);
  });
});

test('malformed args object: enrich({}) fails open to null without throwing', async () => {
  await assert.doesNotReject(async () => {
    const result = await enrich({});
    assert.equal(result, null);
  });
});

test('no args at all: enrich() fails open to null without throwing', async () => {
  await assert.doesNotReject(async () => {
    const result = await enrich();
    assert.equal(result, null);
  });
});

test('decisions filtered: non-string / blank entries in the decisions array are dropped', async () => {
  const fetchImpl = async () => jsonResponse({
    objective: 'ship the feature',
    decisions: ['keep it simple', '', '   ', 42, null, 'avoid a second dependency'],
  });

  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.deepEqual(result, {
    objective: 'ship the feature',
    decisions: ['keep it simple', 'avoid a second dependency'],
  });
});

// --- Prompt bounding, transport-level parse failure, and model-shape tolerance ---

test('the prompt is bounded regardless of tail size, keeping the NEWEST context', async () => {
  // PROMPT_TAIL_CHAR_CAP is the only bound on prompt size and is documented as holding
  // "regardless of tailRecords config". A regression slicing from the wrong end would still pass
  // every other test while feeding the model the OLDEST context — producing a stale objective
  // still rendered under the model-inferred tag.
  const huge = [
    { role: 'user', text: 'OLDEST-MARKER ' + 'x'.repeat(20000) },
    { role: 'assistant', text: 'y'.repeat(20000) + ' NEWEST-MARKER' },
  ];

  let captured = null;
  const fetchImpl = async (_url, options) => {
    captured = JSON.parse(options.body).prompt;
    return jsonResponse({ objective: 'o', decisions: [] });
  };

  await enrich({
    tail: huge,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.ok(captured, 'expected the prompt to be captured');
  assert.ok(captured.length < 20000, `prompt was not bounded: ${captured.length} chars`);
  assert.match(captured, /NEWEST-MARKER/);
  assert.doesNotMatch(captured, /OLDEST-MARKER/);
});

test('a body that fails to parse as JSON at the transport level fails open to null', async () => {
  // Distinct from the "malformed response" case, which returns a well-formed body containing bad
  // text. Here response.json() itself rejects — the failure class the catch comment names.
  const fetchImpl = async () => ({
    ok: true,
    status: 200,
    json: async () => { throw new SyntaxError('Unexpected token < in JSON at position 0'); },
  });

  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  assert.equal(result, null);
});

test('a model returning a JSON array or scalar instead of an object fails open to null', async () => {
  // A small local model emitting a bare array is an ordinary failure mode, not an adversarial one.
  for (const body of [['not', 'an', 'object'], 42, 'a bare string', null]) {
    const fetchImpl = async () => jsonResponse(body);
    const result = await enrich({
      tail: TAIL,
      model: 'llama3.2:3b',
      host: 'http://127.0.0.1:11434',
      timeoutMs: 1000,
      fetchImpl,
    });
    assert.equal(result, null, `expected null for ${JSON.stringify(body)}`);
  }
});

test('non-string objective and non-array decisions are each dropped, not propagated', async () => {
  const fetchImpl = async () => jsonResponse({ objective: 12345, decisions: 'not an array' });

  const result = await enrich({
    tail: TAIL,
    model: 'llama3.2:3b',
    host: 'http://127.0.0.1:11434',
    timeoutMs: 1000,
    fetchImpl,
  });

  // Either a fully-null result or one with both bad fields neutralised is acceptable; what must
  // never happen is a non-string objective or a string `decisions` reaching manifest.mjs.
  if (result !== null) {
    assert.ok(result.objective === null || typeof result.objective === 'string');
    assert.ok(Array.isArray(result.decisions));
  }
});
