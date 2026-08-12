// context-gc — shared test helpers.
//
// `createTempGitRepo` is load-bearing hermeticity, not incidental scaffolding: it is what pins
// `user.email`, `user.name`, `commit.gpgsign` and the default branch so the suite behaves the
// same on a machine with an opinionated global git config as on a bare CI runner. It lives in
// exactly one place so a future hermeticity fix cannot land in some callers and miss others —
// a divergence that would surface as a flake on one developer's machine only.
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
  const git = (...args) => spawnSync('git', args, { cwd: dir, encoding: 'utf8', env: HERMETIC_ENV });

  git('init', '--initial-branch=main');
  git('config', 'user.email', 'context-gc-test@example.com');
  git('config', 'user.name', 'context-gc test');
  git('config', 'commit.gpgsign', 'false');
  return dir;
}

/**
 * Removes a repo created by `createTempGitRepo`. Safe to call on an already-removed path.
 * @param {string} dir
 */
export function cleanup(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
}
