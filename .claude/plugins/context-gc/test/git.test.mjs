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

/**
 * Creates a throwaway git repo under the OS temp dir with a local identity configured (and
 * commit signing disabled) so `git commit` works unattended regardless of the host's global
 * git config. Returns the repo's absolute path.
 */
function createTempGitRepo() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'context-gc-git-test-'));
  spawnSync('git', ['init'], { cwd: dir, encoding: 'utf8' });
  spawnSync('git', ['config', 'user.email', 'context-gc-test@example.com'], { cwd: dir });
  spawnSync('git', ['config', 'user.name', 'context-gc test'], { cwd: dir });
  spawnSync('git', ['config', 'commit.gpgsign', 'false'], { cwd: dir });
  return dir;
}

function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}

test('reports a modified tracked file with status "modified"', () => {
  const dir = createTempGitRepo();
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    spawnSync('git', ['add', 'tracked.txt'], { cwd: dir });
    spawnSync('git', ['commit', '-m', 'init'], { cwd: dir });

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
  const dir = createTempGitRepo();
  try {
    fs.writeFileSync(path.join(dir, 'tracked.txt'), 'v1\n');
    spawnSync('git', ['add', 'tracked.txt'], { cwd: dir });
    spawnSync('git', ['commit', '-m', 'init'], { cwd: dir });

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
  const dir = createTempGitRepo();
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
  const dir = createTempGitRepo();
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
