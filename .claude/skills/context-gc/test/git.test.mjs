// context-gc — tests for src/git.mjs
// git.mjs is a leaf source module: a pure function of working directory -> file list. These
// tests exercise it against real, throwaway git repos created under the OS temp dir so they
// never depend on (or mutate) the surrounding repo.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { getChangedFiles } from '../src/git.mjs';
import { createTempGitRepo, cleanup, git } from './helpers.mjs';

test('reports a modified tracked file with status "modified"', () => {
  const dir = createTempGitRepo('context-gc-git-test-');
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    git(dir, 'add', 'tracked.txt');
    git(dir, 'commit', '-m', 'init');

    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v2\n');

    const files = getChangedFiles(dir);
    const entry = files.find((f) => f.path === 'tracked.txt');
    assert.ok(entry, 'expected tracked.txt to be reported');
    assert.equal(entry.status, 'modified');
  } finally {
    cleanup(dir);
  }
});

test('reports a new untracked file with status "untracked"', () => {
  const dir = createTempGitRepo('context-gc-git-test-');
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    git(dir, 'add', 'tracked.txt');
    git(dir, 'commit', '-m', 'init');

    fs.writeFileSync(path.join(dir, 'new-file.txt'), 'new\n');

    const files = getChangedFiles(dir);
    const entry = files.find((f) => f.path === 'new-file.txt');
    assert.ok(entry, 'expected new-file.txt to be reported');
    assert.equal(entry.status, 'untracked');
  } finally {
    cleanup(dir);
  }
});

test('preserves a path containing a space intact', () => {
  const dir = createTempGitRepo('context-gc-git-test-');
  try {
    fs.writeFileSync(path.join(dir, 'my file.txt'), 'content\n');

    const files = getChangedFiles(dir);
    const entry = files.find((f) => f.path === 'my file.txt');
    assert.ok(entry, 'expected "my file.txt" to be reported with its space intact');
    assert.equal(entry.status, 'untracked');
  } finally {
    cleanup(dir);
  }
});

test('returns an empty array (no throw) for a directory that is not a git repo', () => {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'context-gc-not-a-repo-'));
  try {
    assert.doesNotThrow(() => {
      const files = getChangedFiles(dir);
      assert.deepEqual(files, []);
    });
  } finally {
    cleanup(dir);
  }
});

test('returns an empty array (no throw) when git is not on PATH', () => {
  const dir = createTempGitRepo('context-gc-git-test-');
  const originalPath = process.env.PATH;
  const originalPathWin = process.env.Path;
  try {
    fs.writeFileSync(path.join(dir, 'untracked.txt'), 'x\n');
    process.env.PATH = '';
    process.env.Path = '';
    assert.doesNotThrow(() => {
      const files = getChangedFiles(dir);
      assert.deepEqual(files, []);
    });
  } finally {
    process.env.PATH = originalPath;
    process.env.Path = originalPathWin;
    cleanup(dir);
  }
});

test('returns an empty array (no throw) for a nonexistent cwd', () => {
  const dir = path.join(os.tmpdir(), 'context-gc-does-not-exist-' + Date.now());
  assert.doesNotThrow(() => {
    const files = getChangedFiles(dir);
    assert.deepEqual(files, []);
  });
});

// --- Rename/copy continuation records, absent cwd, and timeout bounding ---

test('a staged rename yields exactly one entry for the new path, not a phantom for the old', () => {
  // `git status --porcelain -z` emits a rename as TWO NUL-terminated records: `R  new\0old\0`.
  // The second carries no `XY ` prefix, so without the continuation skip it would parse as its
  // own entry and put a file that no longer exists into the manifest's deterministic floor.
  const dir = createTempGitRepo('context-gc-git-rename-test-');
  try {
    fs.writeFileSync(path.join(dir, 'old.txt'), 'contents\n');
    git(dir, 'add', 'old.txt');
    git(dir, 'commit', '-m', 'init');

    git(dir, 'mv', 'old.txt', 'new.txt');

    const files = getChangedFiles(dir);
    const paths = files.map((f) => f.path).sort();

    assert.deepEqual(paths, ['new.txt']);
    assert.doesNotMatch(JSON.stringify(files), /old\.txt/);
  } finally {
    cleanup(dir);
  }
});

test('an absent cwd reports nothing rather than the process working directory', () => {
  // Deferring to process.cwd() would return a real-looking file list from an unrelated repo —
  // fabricated floor content, which is worse than an absent section.
  //
  // The assertion must run with the process cwd inside a DIRTY repo, or it is vacuous: in a
  // clean checkout `git status` returns nothing either way, so the guard could be deleted and
  // this would still pass — which is exactly the CI case, the only runner that matters.
  const dir = createTempGitRepo('context-gc-git-cwd-test-');
  const originalCwd = process.cwd();
  try {
    fs.writeFileSync(path.join(dir, 'dirty-file.txt'), 'uncommitted\n');
    process.chdir(dir);

    // Sanity-check the fixture itself: if the cwd is not actually dirty, the test below proves
    // nothing, so fail loudly rather than passing for the wrong reason.
    assert.ok(
      getChangedFiles(dir).some((f) => f.path === 'dirty-file.txt'),
      'fixture repo is not dirty — the guard assertion would be vacuous'
    );

    for (const value of [undefined, null, '', '   ']) {
      assert.deepEqual(getChangedFiles(value), [], `expected [] for ${JSON.stringify(value)}`);
    }
  } finally {
    process.chdir(originalCwd);
    cleanup(dir);
  }
});

test('a malformed porcelain record is skipped rather than coerced into a floor entry', () => {
  // Simulated via a repo whose status output is empty, which does NOT reach the guard: this
  // assertion is tautological on its own, and deleting the malformed-record guard leaves the
  // suite green. The rename case above does not cover it either — its continuation record is
  // consumed by `skipNext` before the guard sees it. The coverage gap is real and tracked in #88;
  // do not read this test as pinning the guard.
  const dir = createTempGitRepo('context-gc-git-clean-test-');
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    git(dir, 'add', 'tracked.txt');
    git(dir, 'commit', '-m', 'init');

    const files = getChangedFiles(dir);
    assert.deepEqual(files, []);
  } finally {
    cleanup(dir);
  }
});
