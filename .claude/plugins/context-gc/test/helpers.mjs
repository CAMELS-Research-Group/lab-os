// context-gc — shared test helpers.
//
// `createTempGitRepo` is load-bearing hermeticity, not incidental scaffolding: it is what pins
// `user.email`, `user.name`, `commit.gpgsign` and the default branch so the suite behaves the
// same on a machine with an opinionated global git config as on a bare CI runner. It lives in
// exactly one place so a future hermeticity fix cannot land in some callers and miss others —
// a divergence that would surface as a flake on one developer's machine only.
//
// The guarantee only holds for git invocations that actually USE the hermetic env, so tests run
// their own git commands through the exported `git()` helper below. It does NOT extend to the
// `spawnSync` inside `src/git.mjs` (the code under test, which correctly inherits the real
// environment) — a global config that changes what `git status` reports can still red those
// assertions. That failure is loud (a red suite), never a false green.
import { spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';

// Global and system git config are pointed at the null device rather than merely overridden
// per-key, so a setting nobody thought to override cannot leak into a test repo.
const HERMETIC_ENV = {
  ...process.env,
  GIT_CONFIG_GLOBAL: os.devNull,
  GIT_CONFIG_SYSTEM: os.devNull,
};

/**
 * Creates a throwaway git repo under the OS temp dir with a local identity configured (and commit
 * signing disabled) so `git commit` works unattended regardless of the host's git config. Returns
 * the repo's absolute path; pass it to `cleanup` when done.
 *
 * @param {string} [prefix] mkdtemp prefix, so a failing test names the suite that created the dir
 * @returns {string} absolute path to the new repo
 */
export function createTempGitRepo(prefix = 'context-gc-test-') {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), prefix));

  git(dir, 'init', '--initial-branch=main');
  git(dir, 'config', 'user.email', 'context-gc-test@example.com');
  git(dir, 'config', 'user.name', 'context-gc test');
  git(dir, 'config', 'commit.gpgsign', 'false');
  return dir;
}

/**
 * Runs a git command inside `dir` under the same hermetic environment `createTempGitRepo` uses.
 *
 * Tests must route their own `git add`/`commit`/`mv` calls through this rather than calling
 * `spawnSync('git', …)` directly: the hermetic env is what keeps an opinionated global git config
 * (a `core.excludesFile`, `status.showUntrackedFiles=no`) from turning the suite red on one
 * machine only. A direct call silently opts out of it.
 *
 * @param {string} dir repo created by `createTempGitRepo`
 * @param {...string} args git arguments
 */
export function git(dir, ...args) {
  return spawnSync('git', args, { cwd: dir, encoding: 'utf8', env: HERMETIC_ENV });
}

/**
 * Removes a repo created by `createTempGitRepo`. Safe to call on an already-removed path.
 * @param {string} dir
 */
export function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}
