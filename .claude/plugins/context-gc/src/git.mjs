// context-gc — git file source module.
//
// Leaf source module: a pure function of working directory -> file list. No manifest,
// formatting, or byte-cap concerns leak in here (seam) — callers get raw {path, status}
// entries and decide what to do with them. Needs no tunables, so unlike other modules in this
// plugin it does not read `getConfig()`.

import { spawnSync } from 'node:child_process';

/**
 * Parses the NUL-delimited output of `git status --porcelain -z` into `{path, status}` entries.
 *
 * Each record is `XY<space>path`, where X/Y are the two status-code columns. `-z` disables the
 * quoting/escaping regular porcelain applies to paths, so paths (including ones containing
 * spaces) come through intact. Rename/copy records (X or Y === 'R'/'C') are followed by an
 * extra NUL-terminated "from" path with no status prefix — that continuation record is skipped
 * so it isn't mistaken for its own entry.
 *
 * Per the plugin's interface, only two status categories are distinguished: '??' (untracked)
 * vs. everything else that appears in the output ('modified') — per-XY-code fidelity is not
 * needed downstream.
 *
 * @param {string} raw
 * @returns {Array<{path: string, status: 'modified' | 'untracked'}>}
 */
function parsePorcelainZ(raw) {
  const records = raw.split('\0').filter((record) => record.length > 0);
  const files = [];
  let skipNext = false;

  for (const record of records) {
    if (skipNext) {
      skipNext = false;
      continue;
    }

    const x = record[0];
    const y = record[1];
    const recordPath = record.slice(3);

    if (x === 'R' || x === 'C' || y === 'R' || y === 'C') {
      skipNext = true;
    }

    const status = x === '?' && y === '?' ? 'untracked' : 'modified';
    files.push({ path: recordPath, status });
  }

  return files;
}

/**
 * Returns the modified and untracked files in the git working tree rooted at `cwd`, derived
 * from `git status --porcelain -z`.
 *
 * Fail-open: not a git repo, `git` absent from PATH, a non-zero git exit, or a spawn error all
 * resolve to an empty array — this function never throws.
 *
 * @param {string} cwd
 * @returns {Array<{path: string, status: 'modified' | 'untracked'}>}
 */
export function getChangedFiles(cwd) {
  let result;
  try {
    result = spawnSync('git', ['status', '--porcelain', '-z'], { cwd, encoding: 'utf8' });
  } catch {
    return [];
  }

  if (!result || result.error || result.status !== 0 || typeof result.stdout !== 'string') {
    return [];
  }

  return parsePorcelainZ(result.stdout);
}

export default getChangedFiles;
