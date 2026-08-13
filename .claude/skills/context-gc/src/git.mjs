// context-gc — git file source module.
//
// Leaf source module: a pure function of working directory -> file list. No manifest,
// formatting, or byte-cap concerns leak in here (seam) — callers get raw {path, status}
// entries and decide what to do with them. Needs no tunables, so unlike other modules in this
// plugin it does not read `getConfig()`.

import { spawnSync } from 'node:child_process';

// Wall-clock ceiling on the `git status` call. Generous enough for a cold large repo, short
// enough that a pathological hang cannot stall session resume.
const GIT_TIMEOUT_MS = 5000;

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
 * A record that does not match the `XY<space>path` shape is SKIPPED rather than coerced. These
 * entries land in the manifest's deterministic floor, so a porcelain format change must cost a
 * missing section, never a fabricated path presented as sourced fact.
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

    if (x === 'R' || x === 'C' || y === 'R' || y === 'C') {
      skipNext = true;
    }

    // `XY<space>` prefix plus at least one path character; anything shorter or malformed is not
    // a record this parser understands, and guessing at it would fabricate a floor entry.
    if (record.length <= 3 || record[2] !== ' ') continue;

    const recordPath = record.slice(3);
    const status = x === '?' && y === '?' ? 'untracked' : 'modified';
    files.push({ path: recordPath, status });
  }

  return files;
}

/**
 * Returns the modified and untracked files in the git working tree rooted at `cwd`, derived
 * from `git status --porcelain -z`.
 *
 * Fail-open: not a git repo, `git` absent from PATH, a non-zero git exit, a spawn error, or a
 * git invocation that exceeds `GIT_TIMEOUT_MS` all resolve to an empty array — this function
 * never throws.
 *
 * The timeout matters as much as the error handling: this runs synchronously on the session
 * resume path, and `git status` can block indefinitely on index-lock contention or a slow
 * network filesystem. Without it the fail-open contract would cover errors but not hangs, and a
 * hook that blocks resume is worse than one that no-ops — the same reasoning that gives the
 * Ollama call its deadline.
 *
 * An absent `cwd` is treated as "no repo to inspect" rather than deferring to the process
 * working directory, which would report a plausible-looking file list from an unrelated repo.
 *
 * @param {string} cwd
 * @returns {Array<{path: string, status: 'modified' | 'untracked'}>}
 */
export function getChangedFiles(cwd) {
  if (typeof cwd !== 'string' || cwd.trim() === '') return [];

  let result;
  try {
    result = spawnSync('git', ['status', '--porcelain', '-z'], {
      cwd,
      encoding: 'utf8',
      timeout: GIT_TIMEOUT_MS,
    });
  } catch {
    return [];
  }

  if (!result || result.error || result.status !== 0 || typeof result.stdout !== 'string') {
    return [];
  }

  return parsePorcelainZ(result.stdout);
}
