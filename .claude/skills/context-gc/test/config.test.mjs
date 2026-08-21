// context-gc — tests for src/config.mjs
// config.mjs is the single canonical owner of tunables (env names + defaults); these tests
// exercise it via an injected env object so they never touch the real process.env.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { getConfig } from '../src/config.mjs';

const DEFAULTS = {
  debug: false,
  ollamaModel: 'hermes3:8b',
  ollamaHost: 'http://127.0.0.1:11434',
  tailRecords: 40,
  timeoutMs: 20000,
  maxBytes: 4000,
};

test('resolves documented defaults when env is empty', () => {
  const config = getConfig({});
  assert.deepEqual(config, DEFAULTS);
});

test('resolves documented defaults when env has none of the CONTEXT_GC_* keys', () => {
  const config = getConfig({ PATH: '/usr/bin', HOME: '/home/x' });
  assert.deepEqual(config, DEFAULTS);
});

test('the generation timeout default is a generous 20s, not a short cap', () => {
  // Regression guard: a shorter flat cap does not survive live-compaction load, so enrichment
  // silently never lands. The session-resume budget is deliberately 20s.
  assert.equal(getConfig({}).timeoutMs, 20000);
});

test('applies env overrides for every tunable', () => {
  const config = getConfig({
    CONTEXT_GC_OLLAMA_MODEL: 'qwen2.5:7b',
    CONTEXT_GC_OLLAMA_HOST: 'http://localhost:9999',
    CONTEXT_GC_TAIL_RECORDS: '100',
    CONTEXT_GC_TIMEOUT_MS: '15000',
    CONTEXT_GC_MAX_BYTES: '8000',
  });
  assert.deepEqual(config, {
    debug: false,
    ollamaModel: 'qwen2.5:7b',
    ollamaHost: 'http://localhost:9999',
    tailRecords: 100,
    timeoutMs: 15000,
    maxBytes: 8000,
  });
});

test('blank string env values fall back to defaults (string tunables)', () => {
  const config = getConfig({
    CONTEXT_GC_OLLAMA_MODEL: '',
    CONTEXT_GC_OLLAMA_HOST: '   ',
  });
  assert.equal(config.ollamaModel, DEFAULTS.ollamaModel);
  assert.equal(config.ollamaHost, DEFAULTS.ollamaHost);
});

test('blank string env values fall back to defaults (numeric tunables)', () => {
  const config = getConfig({
    CONTEXT_GC_TAIL_RECORDS: '',
    CONTEXT_GC_TIMEOUT_MS: '   ',
    CONTEXT_GC_MAX_BYTES: '',
  });
  assert.equal(config.tailRecords, DEFAULTS.tailRecords);
  assert.equal(config.timeoutMs, DEFAULTS.timeoutMs);
  assert.equal(config.maxBytes, DEFAULTS.maxBytes);
});

test('non-numeric env values fall back to the default rather than throwing', () => {
  assert.doesNotThrow(() => {
    const config = getConfig({
      CONTEXT_GC_TAIL_RECORDS: 'not-a-number',
      CONTEXT_GC_TIMEOUT_MS: 'NaN',
      CONTEXT_GC_MAX_BYTES: 'eight-thousand',
    });
    assert.equal(config.tailRecords, DEFAULTS.tailRecords);
    assert.equal(config.timeoutMs, DEFAULTS.timeoutMs);
    assert.equal(config.maxBytes, DEFAULTS.maxBytes);
  });
});

test('non-integer numeric env values fall back to the default', () => {
  const config = getConfig({
    CONTEXT_GC_TAIL_RECORDS: '40.5',
    CONTEXT_GC_TIMEOUT_MS: '8000.7',
    CONTEXT_GC_MAX_BYTES: 'Infinity',
  });
  assert.equal(config.tailRecords, DEFAULTS.tailRecords);
  assert.equal(config.timeoutMs, DEFAULTS.timeoutMs);
  assert.equal(config.maxBytes, DEFAULTS.maxBytes);
});

test('getConfig() with no argument reads process.env without throwing', () => {
  assert.doesNotThrow(() => getConfig());
});

test('does not mutate the env object passed in', () => {
  const env = { CONTEXT_GC_TAIL_RECORDS: '7' };
  const before = JSON.stringify(env);
  getConfig(env);
  assert.equal(JSON.stringify(env), before);
});

// --- Range validation and whitespace handling ---

test('a zero or negative integer tunable falls back to its default', () => {
  // Each of these silently disables a feature if passed through: a 0 byte cap empties the
  // manifest forever, a non-positive tail window empties the transcript read and with it
  // enrichment. A typo must cost the default, not the plugin.
  for (const value of ['0', '-1', '-4000']) {
    const config = getConfig({
      CONTEXT_GC_MAX_BYTES: value,
      CONTEXT_GC_TAIL_RECORDS: value,
      CONTEXT_GC_TIMEOUT_MS: value,
    });
    assert.equal(config.maxBytes, DEFAULTS.maxBytes, `maxBytes for ${value}`);
    assert.equal(config.tailRecords, DEFAULTS.tailRecords, `tailRecords for ${value}`);
    assert.equal(config.timeoutMs, DEFAULTS.timeoutMs, `timeoutMs for ${value}`);
  }
});

test('a positive integer tunable is still honoured (the range check is not a blanket reject)', () => {
  const config = getConfig({ CONTEXT_GC_MAX_BYTES: '1', CONTEXT_GC_TAIL_RECORDS: '7' });
  assert.equal(config.maxBytes, 1);
  assert.equal(config.tailRecords, 7);
});

test('a string tunable is trimmed, so stray whitespace cannot build a malformed URL', () => {
  const config = getConfig({
    CONTEXT_GC_OLLAMA_HOST: '  http://127.0.0.1:11434  ',
    CONTEXT_GC_OLLAMA_MODEL: '\tqwen2.5:7b\n',
  });
  assert.equal(config.ollamaHost, 'http://127.0.0.1:11434');
  assert.equal(config.ollamaModel, 'qwen2.5:7b');
});

test('an integer tunable surrounded by whitespace still parses', () => {
  assert.equal(getConfig({ CONTEXT_GC_MAX_BYTES: '  512  ' }).maxBytes, 512);
});
