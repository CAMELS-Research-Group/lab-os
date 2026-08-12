// context-gc — tests for src/ollama.mjs
// ollama.mjs is the local-Ollama enrichment leaf: it must NEVER throw and must fail open to
// `null` on every failure mode (unreachable, non-200, network throw, timeout/abort, malformed or
// empty response). Every test here injects a fake `fetchImpl` — none require a live Ollama or
// touch the real network, per the task's hermeticity requirement.
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
  const fetchImpl = (url, opts) => new Promise((_resolve, reject) => {
    opts.signal.addEventListener('abort', () => {
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
