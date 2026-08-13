// context-gc — tests for src/recover.mjs
// recover.mjs is the thin orchestrator that wires config -> git -> transcript -> manifest for a
// SessionStart(compact) hook payload. These tests call `recover(payload)` directly (no process
// spawned) against a real throwaway git repo (same pattern as git.test.mjs) and the existing
// synthetic transcript fixtures (same pattern as transcript.test.mjs), plus fault-injection cases
// that assert the orchestrator degrades rather than throws.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { recover } from '../src/recover.mjs';
import { createTempGitRepo, cleanup, git } from './helpers.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fixture = (name) => path.join(__dirname, 'fixtures', name);

test('source "compact" with a real repo + fixture transcript returns a manifest with changed files and tasks', () => {
  const dir = createTempGitRepo('context-gc-recover-test-');
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    git(dir, 'add', 'tracked.txt');
    git(dir, 'commit', '-m', 'init');
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v2\n');
    fs.writeFileSync(path.join(dir, 'new-file.txt'), 'new\n');

    const manifest = recover({
      source: 'compact',
      cwd: dir,
      transcript_path: fixture('todo-write.jsonl'),
    });

    assert.match(manifest, /tracked\.txt/);
    assert.match(manifest, /new-file\.txt/);
    assert.match(manifest, /task A/);
    assert.match(manifest, /task B/);
  } finally {
    cleanup(dir);
  }
});

test('source other than "compact" is a no-op: returns "" and never touches git or the transcript', () => {
  const dir = createTempGitRepo('context-gc-recover-test-');
  try {
    fs.writeFileSync(path.join(dir, 'untracked.txt'), 'x\n');

    for (const source of ['startup', 'resume', 'clear', undefined, '']) {
      const manifest = recover({
        source,
        cwd: dir,
        transcript_path: fixture('todo-write.jsonl'),
      });
      assert.equal(manifest, '');
    }
  } finally {
    cleanup(dir);
  }
});

test('missing payload entirely returns "" without throwing', () => {
  assert.doesNotThrow(() => {
    assert.equal(recover(undefined), '');
    assert.equal(recover(null), '');
  });
});

test('nonexistent transcript_path degrades to empty tasks but still reports git files (degrade, not empty)', () => {
  const dir = createTempGitRepo('context-gc-recover-test-');
  try {
    fs.writeFileSync(path.join(dir, 'orphan.txt'), 'x\n');

    const manifest = recover({
      source: 'compact',
      cwd: dir,
      transcript_path: path.join(dir, 'does-not-exist.jsonl'),
    });

    assert.match(manifest, /orphan\.txt/);
    assert.doesNotMatch(manifest, /## Tasks/);
  } finally {
    cleanup(dir);
  }
});

test('missing transcript_path field (undefined) degrades to empty tasks but still reports git files', () => {
  const dir = createTempGitRepo('context-gc-recover-test-');
  try {
    fs.writeFileSync(path.join(dir, 'orphan2.txt'), 'x\n');

    const manifest = recover({ source: 'compact', cwd: dir });

    assert.match(manifest, /orphan2\.txt/);
  } finally {
    cleanup(dir);
  }
});

test('non-git cwd degrades to empty files but still reports transcript tasks', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'context-gc-not-a-repo-'));
  try {
    const manifest = recover({
      source: 'compact',
      cwd: dir,
      transcript_path: fixture('todo-write.jsonl'),
    });

    assert.match(manifest, /task A/);
    assert.doesNotMatch(manifest, /## Files in flight/);
  } finally {
    cleanup(dir);
  }
});

test('both git and transcript absent (nothing in flight) returns "" (no-op)', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'context-gc-not-a-repo-'));
  try {
    const manifest = recover({
      source: 'compact',
      cwd: dir,
      transcript_path: path.join(dir, 'does-not-exist.jsonl'),
    });
    assert.equal(manifest, '');
  } finally {
    cleanup(dir);
  }
});

test('a payload shaped to blow up internals (non-string cwd/transcript_path) returns "" without throwing', () => {
  assert.doesNotThrow(() => {
    const manifest = recover({ source: 'compact', cwd: 12345, transcript_path: { not: 'a string' } });
    assert.equal(typeof manifest, 'string');
  });
});

test('spawned as a real process, a large manifest reaches stdout uncorrupted (regression: process.exit() racing an async pipe flush on Windows)', () => {
  const repoDir = createTempGitRepo('context-gc-recover-test-');
  const scriptPath = path.join(__dirname, '..', 'src', 'recover.mjs');
  let transcriptDir;
  try {
    fs.writeFileSync(path.join(repoDir, 'tracked.txt'), 'v1\n');
    git(repoDir, 'add', 'tracked.txt');
    git(repoDir, 'commit', '-m', 'init');
    fs.writeFileSync(path.join(repoDir, 'tracked.txt'), 'v2\n');
    fs.writeFileSync(path.join(repoDir, 'new-file.txt'), 'new\n');

    // Build a transcript whose most recent TodoWrite carries thousands of long tasks, so the
    // rendered manifest runs to hundreds of KB. Payload size is the whole point: a manifest small
    // enough to clear the pipe buffer completes its `write()` synchronously and proves nothing.
    // Only a payload too large for one synchronous write exposes the
    // async-flush-vs-`process.exit()` race on macOS.
    transcriptDir = fs.mkdtempSync(path.join(os.tmpdir(), 'context-gc-recover-large-'));
    const transcriptPath = path.join(transcriptDir, 'large.jsonl');
    const TASK_COUNT = 3000;
    const todos = [];
    for (let i = 0; i < TASK_COUNT; i += 1) {
      todos.push({
        content: `task-${i}-${'x'.repeat(180)}`,
        status: i % 2 === 0 ? 'pending' : 'completed',
        activeForm: `Doing ${i}`,
      });
    }
    const lines = [
      JSON.stringify({ type: 'user', message: { role: 'user', content: 'start task' } }),
      JSON.stringify({
        type: 'assistant',
        message: {
          role: 'assistant',
          content: [{ type: 'tool_use', name: 'TodoWrite', input: { todos } }],
        },
      }),
      JSON.stringify({ type: 'summary', isCompactSummary: true }),
    ];
    fs.writeFileSync(transcriptPath, `${lines.join('\n')}\n`);

    const payload = { source: 'compact', cwd: repoDir, transcript_path: transcriptPath };
    const bigMaxBytes = '5000000';

    // Expected value: call the pure `recover()` in-process, temporarily overriding
    // CONTEXT_GC_MAX_BYTES so getConfig() resolves the same cap the child process will use.
    // This isolates the comparison to exactly what main()'s stdout/process-exit handling might
    // corrupt, not a difference in config resolution between parent and child.
    const savedMaxBytes = process.env.CONTEXT_GC_MAX_BYTES;
    process.env.CONTEXT_GC_MAX_BYTES = bigMaxBytes;
    let expectedManifest;
    try {
      expectedManifest = recover(payload);
    } finally {
      if (savedMaxBytes === undefined) delete process.env.CONTEXT_GC_MAX_BYTES;
      else process.env.CONTEXT_GC_MAX_BYTES = savedMaxBytes;
    }

    // Sanity check on the fixture itself: fail loudly here (not via a confusing downstream
    // assertion) if the generated manifest is not actually large.
    assert.ok(
      Buffer.byteLength(expectedManifest, 'utf8') > 300_000,
      `fixture manifest should be large (was ${Buffer.byteLength(expectedManifest, 'utf8')} bytes)`,
    );

    const result = spawnSync(process.execPath, [scriptPath], {
      input: JSON.stringify(payload),
      encoding: 'utf8',
      // main() runs recoverEnriched(), which attempts a real Ollama call unless
      // told otherwise. Point CONTEXT_GC_OLLAMA_HOST at a loopback port nothing listens on so
      // this test's byte-for-byte comparison against the deterministic-only `recover()` stays
      // valid regardless of whether the host machine happens to have a real Ollama running —
      // a fast, offline-safe ECONNREFUSED, not a DNS lookup.
      env: { ...process.env, CONTEXT_GC_MAX_BYTES: bigMaxBytes, CONTEXT_GC_OLLAMA_HOST: 'http://127.0.0.1:65533' },
      maxBuffer: 10 * 1024 * 1024,
    });

    assert.equal(result.status, 0, `child exited ${result.status}, stderr: ${result.stderr}`);

    let parsed;
    assert.doesNotThrow(() => {
      parsed = JSON.parse(result.stdout);
    }, `stdout was not valid JSON — likely truncated (received ${Buffer.byteLength(result.stdout, 'utf8')} bytes)`);

    const actualManifest = parsed.hookSpecificOutput.additionalContext;
    assert.equal(
      Buffer.byteLength(actualManifest, 'utf8'),
      Buffer.byteLength(expectedManifest, 'utf8'),
      'stdout additionalContext byte length must match the untruncated manifest',
    );
    assert.equal(actualManifest, expectedManifest);
  } finally {
    cleanup(repoDir);
    if (transcriptDir) cleanup(transcriptDir);
  }
});

test('a getter that throws when read still results in "" rather than propagating', () => {
  const payload = { source: 'compact' };
  Object.defineProperty(payload, 'cwd', {
    get() {
      throw new Error('boom');
    },
  });
  assert.doesNotThrow(() => {
    assert.equal(recover(payload), '');
  });
});

// --- Process boundary: the exit-0 contract under hostile stdin ---

test('the hook exits 0 and emits nothing for every malformed stdin payload', () => {
  // main()'s loudest stated contract is "always exits 0 — a hook that blocks or errors session
  // resume is worse than one that silently no-ops". Happy-path spawns cannot establish it: the
  // contract is about the payloads nobody designed for. A hook that exits non-zero on a shape the
  // harness happens to send would block resume for every user at once, so the table below drives
  // the malformed shapes directly rather than assuming they cannot occur.
  const entry = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'src', 'recover.mjs');

  const payloads = [
    ['empty stdin', ''],
    ['whitespace only', '   \n  '],
    ['unparseable JSON', 'not json at all'],
    ['JSON null', 'null'],
    ['JSON array', '[1,2,3]'],
    ['no source key', '{"cwd":"/tmp"}'],
    ['non-compact source', '{"source":"startup"}'],
    ['source with hostile types', '{"source":"compact","cwd":42,"transcript_path":{"a":1}}'],
  ];

  for (const [label, input] of payloads) {
    const result = spawnSync(process.execPath, [entry], {
      input,
      encoding: 'utf8',
      env: { ...process.env, CONTEXT_GC_OLLAMA_HOST: 'http://127.0.0.1:1' },
    });

    assert.equal(result.status, 0, `${label}: expected exit 0, got ${result.status}`);
    assert.equal(result.stdout, '', `${label}: expected no stdout`);
  }
});

test('an empty manifest writes nothing at all, not an empty additionalContext envelope', () => {
  // A clean tree with no transcript has nothing to recover. Emitting an envelope with an empty
  // string would put a meaningless block into every resumed session's context.
  const entry = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'src', 'recover.mjs');
  const dir = createTempGitRepo('context-gc-recover-clean-');
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    git(dir, 'add', 'tracked.txt');
    git(dir, 'commit', '-m', 'init');

    const result = spawnSync(process.execPath, [entry], {
      input: JSON.stringify({ source: 'compact', cwd: dir }),
      encoding: 'utf8',
      env: { ...process.env, CONTEXT_GC_OLLAMA_HOST: 'http://127.0.0.1:1' },
    });

    assert.equal(result.status, 0);
    assert.equal(result.stdout, '');
  } finally {
    cleanup(dir);
  }
});
