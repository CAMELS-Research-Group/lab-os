// context-gc — tests for src/recover.mjs's recoverEnriched()
// recoverEnriched() is the async orchestrator that layers local-Ollama enrichment on top of the
// same deterministic sources `recover()` uses. These tests never touch a live Ollama or the real
// network — every case injects a fake `fetchImpl` through `recoverEnriched(payload, {fetchImpl})`.
// Coverage: enrichment success merges tagged fields into the manifest; enrichment failure (every
// mode) ships the deterministic floor byte-for-byte unchanged; cap-trim drops enriched content
// before the deterministic floor under byte pressure; `recover()` itself never calls the network.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { recover, recoverEnriched } from '../src/recover.mjs';
import { createTempGitRepo, cleanup, git } from './helpers.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fixture = (name) => path.join(__dirname, 'fixtures', name);

/** Builds a fetch-shaped success response resolving to `{response: JSON.stringify(body)}`. */
function jsonResponse(body) {
  return { ok: true, status: 200, json: async () => ({ response: JSON.stringify(body) }) };
}

/** Runs `fn` with `process.env.CONTEXT_GC_MAX_BYTES` temporarily set, restoring it afterward. */
async function withMaxBytes(maxBytes, fn) {
  const saved = process.env.CONTEXT_GC_MAX_BYTES;
  process.env.CONTEXT_GC_MAX_BYTES = String(maxBytes);
  try {
    return await fn();
  } finally {
    if (saved === undefined) delete process.env.CONTEXT_GC_MAX_BYTES;
    else process.env.CONTEXT_GC_MAX_BYTES = saved;
  }
}

test('enrichment success: merges tagged objective + decisions into the manifest, floor intact', async () => {
  const dir = createTempGitRepo('context-gc-recover-enriched-test-');
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    git(dir, 'add', 'tracked.txt');
    git(dir, 'commit', '-m', 'init');
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v2\n');

    const fetchImpl = async () => jsonResponse({
      objective: 'wire up the recovery manifest',
      decisions: ['prefer local Ollama over a metered API for privacy and $0 cost'],
    });

    const manifest = await recoverEnriched(
      { source: 'compact', cwd: dir, transcript_path: fixture('todo-write.jsonl') },
      { fetchImpl },
    );

    // Deterministic floor still present.
    assert.match(manifest, /tracked\.txt/);
    assert.match(manifest, /task A/);
    // Enriched content present and visibly tagged as model-inferred in exactly one heading.
    assert.match(manifest, /wire up the recovery manifest/);
    assert.match(manifest, /prefer local Ollama over a metered API/);
    assert.match(manifest, /model-inferred/i);
    assert.equal((manifest.match(/model-inferred/gi) || []).length, 1);
  } finally {
    cleanup(dir);
  }
});

test('enrichment failure (network throw): ships the deterministic floor byte-for-byte unchanged', async () => {
  const dir = createTempGitRepo('context-gc-recover-enriched-test-');
  try {
    fs.writeFileSync(path.join(dir, 'orphan.txt'), 'x\n');

    const payload = { source: 'compact', cwd: dir, transcript_path: fixture('todo-write.jsonl') };
    const fetchImpl = async () => { throw new Error('ECONNREFUSED'); };

    const deterministic = recover(payload);
    const enriched = await recoverEnriched(payload, { fetchImpl });

    assert.equal(enriched, deterministic);
    assert.doesNotMatch(enriched, /model-inferred/i);
  } finally {
    cleanup(dir);
  }
});

test('enrichment failure (non-200): ships the deterministic floor byte-for-byte unchanged', async () => {
  const dir = createTempGitRepo('context-gc-recover-enriched-test-');
  try {
    fs.writeFileSync(path.join(dir, 'orphan.txt'), 'x\n');
    const payload = { source: 'compact', cwd: dir, transcript_path: fixture('todo-write.jsonl') };
    const fetchImpl = async () => ({ ok: false, status: 404, json: async () => ({}) });

    const deterministic = recover(payload);
    const enriched = await recoverEnriched(payload, { fetchImpl });

    assert.equal(enriched, deterministic);
  } finally {
    cleanup(dir);
  }
});

test('enrichment failure (malformed JSON response): ships the deterministic floor unchanged', async () => {
  const dir = createTempGitRepo('context-gc-recover-enriched-test-');
  try {
    fs.writeFileSync(path.join(dir, 'orphan.txt'), 'x\n');
    const payload = { source: 'compact', cwd: dir, transcript_path: fixture('todo-write.jsonl') };
    const fetchImpl = async () => ({ ok: true, status: 200, json: async () => ({ response: 'not json' }) });

    const deterministic = recover(payload);
    const enriched = await recoverEnriched(payload, { fetchImpl });

    assert.equal(enriched, deterministic);
  } finally {
    cleanup(dir);
  }
});

test('enrichment failure (timeout/abort): ships the deterministic floor unchanged, does not hang', async () => {
  const dir = createTempGitRepo('context-gc-recover-enriched-test-');
  try {
    fs.writeFileSync(path.join(dir, 'orphan.txt'), 'x\n');
    const payload = { source: 'compact', cwd: dir, transcript_path: fixture('todo-write.jsonl') };
    const fetchImpl = (url, opts) => new Promise((_resolve, reject) => {
      opts.signal.addEventListener('abort', () => {
        const err = new Error('aborted');
        err.name = 'AbortError';
        reject(err);
      });
    });

    await withMaxBytes(4000, async () => {
      process.env.CONTEXT_GC_TIMEOUT_MS = '50';
      try {
        const start = Date.now();
        const deterministic = recover(payload);
        const enriched = await recoverEnriched(payload, { fetchImpl });
        const elapsed = Date.now() - start;

        assert.equal(enriched, deterministic);
        assert.ok(elapsed < 2000, `expected a fast fail-open, took ${elapsed}ms`);
      } finally {
        delete process.env.CONTEXT_GC_TIMEOUT_MS;
      }
    });
  } finally {
    cleanup(dir);
  }
});

test('source other than "compact": returns "" without ever calling the transport', async () => {
  const dir = createTempGitRepo('context-gc-recover-enriched-test-');
  try {
    let called = false;
    const fetchImpl = async () => {
      called = true;
      return jsonResponse({ objective: 'x', decisions: [] });
    };

    for (const source of ['startup', 'resume', undefined]) {
      const manifest = await recoverEnriched(
        { source, cwd: dir, transcript_path: fixture('todo-write.jsonl') },
        { fetchImpl },
      );
      assert.equal(manifest, '');
    }
    assert.equal(called, false);
  } finally {
    cleanup(dir);
  }
});

test('missing payload entirely resolves to "" without throwing', async () => {
  await assert.doesNotReject(async () => {
    assert.equal(await recoverEnriched(undefined), '');
    assert.equal(await recoverEnriched(null), '');
  });
});

test('cap-trim: enriched content is dropped before the deterministic floor under byte pressure', async () => {
  const dir = createTempGitRepo('context-gc-recover-enriched-test-');
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    git(dir, 'add', 'tracked.txt');
    git(dir, 'commit', '-m', 'init');
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v2\n');

    const payload = { source: 'compact', cwd: dir, transcript_path: fixture('todo-write.jsonl') };
    const fetchImpl = async () => jsonResponse({
      objective: 'a fairly long model-inferred objective sentence to pad out the byte count',
      decisions: [
        'a fairly long decision with a lot of padding text describing the why in detail',
        'another fairly long decision, also padded, also describing the why in detail',
      ],
    });

    // Room for the deterministic floor (files + tasks) plus the header, but not enough left over
    // for the enriched section.
    const floorOnly = await recoverEnriched(payload, { fetchImpl: async () => null });
    const floorBytes = Buffer.byteLength(floorOnly, 'utf8');

    const manifest = await withMaxBytes(floorBytes + 10, () => recoverEnriched(payload, { fetchImpl }));

    assert.ok(Buffer.byteLength(manifest, 'utf8') <= floorBytes + 10);
    assert.match(manifest, /tracked\.txt/);
    assert.match(manifest, /task A/);
    assert.doesNotMatch(manifest, /model-inferred/i);
    assert.doesNotMatch(manifest, /fairly long/);
  } finally {
    cleanup(dir);
  }
});

test('honors the configured model/host/timeout from config.mjs (via env)', async () => {
  const dir = createTempGitRepo('context-gc-recover-enriched-test-');
  try {
    fs.writeFileSync(path.join(dir, 'orphan.txt'), 'x\n');
    const payload = { source: 'compact', cwd: dir, transcript_path: fixture('todo-write.jsonl') };

    let capturedUrl;
    let capturedModel;
    const fetchImpl = async (url, opts) => {
      capturedUrl = url;
      capturedModel = JSON.parse(opts.body).model;
      return jsonResponse({ objective: '', decisions: [] });
    };

    const savedModel = process.env.CONTEXT_GC_OLLAMA_MODEL;
    const savedHost = process.env.CONTEXT_GC_OLLAMA_HOST;
    process.env.CONTEXT_GC_OLLAMA_MODEL = 'qwen2.5:7b';
    process.env.CONTEXT_GC_OLLAMA_HOST = 'http://localhost:4321';
    try {
      await recoverEnriched(payload, { fetchImpl });
    } finally {
      if (savedModel === undefined) delete process.env.CONTEXT_GC_OLLAMA_MODEL;
      else process.env.CONTEXT_GC_OLLAMA_MODEL = savedModel;
      if (savedHost === undefined) delete process.env.CONTEXT_GC_OLLAMA_HOST;
      else process.env.CONTEXT_GC_OLLAMA_HOST = savedHost;
    }

    assert.equal(capturedUrl, 'http://localhost:4321/api/generate');
    assert.equal(capturedModel, 'qwen2.5:7b');
  } finally {
    cleanup(dir);
  }
});

test('a getter that throws when read still results in "" rather than propagating', async () => {
  const payload = { source: 'compact' };
  Object.defineProperty(payload, 'cwd', {
    get() {
      throw new Error('boom');
    },
  });
  await assert.doesNotReject(async () => {
    assert.equal(await recoverEnriched(payload), '');
  });
});

test('recover() itself never calls the network, even when enrichment would succeed', () => {
  // Sanity check on the seam: recover() (the deterministic-only function) takes no fetchImpl
  // and has no path to ollama.mjs at all — this is a structural assertion, not a mock check.
  const dir = createTempGitRepo('context-gc-recover-enriched-test-');
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    const manifest = recover({ source: 'compact', cwd: dir, transcript_path: fixture('todo-write.jsonl') });
    assert.equal(typeof manifest, 'string');
    assert.doesNotMatch(manifest, /model-inferred/i);
  } finally {
    cleanup(dir);
  }
});
