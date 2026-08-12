// context-gc — tests for src/config.mjs
// config.mjs is the single canonical owner of tunables (env names + defaults); these tests
// exercise it via an injected env object so they never touch the real process.env.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { getConfig } from '../src/config.mjs';

const DEFAULTS = {
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

test('the generation timeout default is the generous 20s value (not the old 8s cap)', () => {
  // Regression guard for the Task-8 decision: the single flat cap was too short under
  // live-compaction load; the resume budget is intentionally 20s so enrichment lands.
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
