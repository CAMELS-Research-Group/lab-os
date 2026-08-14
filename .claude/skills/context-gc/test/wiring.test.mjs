// context-gc — tests for the plugin's WIRING rather than its logic.
//
// Every other test file imports the source modules by relative path, so all 100+ of them stay
// green if the hook command, the plugin manifest, or the discovery path points somewhere that no
// longer exists. That failure mode is the hardest one to notice in this plugin specifically:
// context-gc is deliberately fail-open, so a hook that never fires is indistinguishable from a
// session that simply had nothing to recover. These tests assert the declarations agree with the
// files on disk.
//
// lab-os distributes this plugin WITHOUT a marketplace: the directory is vendored under
// `.claude/skills/`, where Claude Code discovers any folder carrying `.claude-plugin/plugin.json`
// as a plugin (`context-gc@skills-dir`) with no install step, and `link-lab-assets.sh` symlinks it
// into `~/.claude/skills/` so it loads at personal scope in every session on the machine. That
// makes the discovery preconditions below load-bearing in a way a marketplace entry never was:
// nothing else in the repo notices if the plugin stops being found.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import { execFileSync, spawnSync } from 'node:child_process';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createTempGitRepo, cleanup, git } from './helpers.mjs';

const pluginRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const repoRoot = path.resolve(pluginRoot, '..', '..', '..');

const readJson = (p) => JSON.parse(fs.readFileSync(p, 'utf8'));

test('every wiring manifest is valid JSON', () => {
  for (const relative of [
    '.claude-plugin/plugin.json',
    'hooks/hooks.json',
  ]) {
    const full = path.join(pluginRoot, relative);
    assert.ok(fs.existsSync(full), `${relative} is missing`);
    assert.doesNotThrow(() => readJson(full), `${relative} is not valid JSON`);
  }
});

test('the SessionStart hook command points at a file that exists', () => {
  // The one declaration that makes the plugin do anything at all. `${CLAUDE_PLUGIN_ROOT}` is
  // substituted by the harness at invocation time; resolving it against the plugin root here is
  // exactly what the harness will do. It resolves for a skills-dir plugin the same way it does
  // for a marketplace install — the plugin is discovered in place rather than copied into the
  // version cache, but it is a plugin either way.
  const hooks = readJson(path.join(pluginRoot, 'hooks/hooks.json'));
  const entries = hooks.hooks.SessionStart;

  assert.ok(Array.isArray(entries) && entries.length > 0, 'no SessionStart hook declared');

  const commands = entries.flatMap((entry) => entry.hooks.map((h) => h.command));
  assert.ok(commands.length > 0, 'no hook commands declared');

  for (const command of commands) {
    const match = command.match(/\$\{CLAUDE_PLUGIN_ROOT\}([^"']+)/);
    assert.ok(match, `hook command does not resolve against CLAUDE_PLUGIN_ROOT: ${command}`);
    const target = path.join(pluginRoot, match[1]);
    assert.ok(fs.existsSync(target), `hook command targets a missing file: ${target}`);
    // "A file that exists" is too weak: repointing the hook at any other module in the plugin
    // would satisfy it while the plugin silently stopped working. The entrypoint is the module
    // that actually exports the recovery orchestration.
    assert.equal(
      path.resolve(target),
      path.resolve(pluginRoot, 'src/recover.mjs'),
      'hook command does not target the recovery entrypoint'
    );
  }
});

test('the hook matches the compaction event it exists to serve', () => {
  const hooks = readJson(path.join(pluginRoot, 'hooks/hooks.json'));
  const matchers = hooks.hooks.SessionStart.map((entry) => entry.matcher);
  assert.ok(matchers.includes('compact'), `expected a "compact" matcher, got ${matchers.join(', ')}`);
});

test('the plugin is vendored where Claude Code discovers plugins without a marketplace', () => {
  // Discovery is by location: only a folder under a *skills* directory is scanned for a plugin
  // manifest. Moving this directory to `.claude/plugins/` — the layout a marketplace-distributed
  // copy uses, and the obvious place to file a plugin — silently ends discovery. Nothing else in
  // the repo fails when that happens, so it is asserted here.
  assert.equal(
    path.basename(path.dirname(pluginRoot)),
    'skills',
    'plugin is not under a skills directory, so it will never be discovered'
  );
  assert.equal(
    path.resolve(pluginRoot),
    path.resolve(repoRoot, '.claude/skills/context-gc'),
    'plugin is not at the vendored path the link script and CI both key on'
  );
});

test('the manifest name matches the directory that names the plugin', () => {
  // A skills-dir plugin is addressed as `<name>@skills-dir` — in `claude plugin disable`, in
  // `enabledPlugins`, and in the CI discovery glob. A manifest name that disagrees with the
  // directory leaves two different identifiers for one plugin.
  const plugin = readJson(path.join(pluginRoot, '.claude-plugin/plugin.json'));
  assert.equal(plugin.name, path.basename(pluginRoot));
});

test('the plugin ships no root SKILL.md, so it stays hooks-only', () => {
  // With no `skills/` directory and no `skills` manifest field, a `SKILL.md` at the plugin root
  // would be loaded as a model-invocable skill. This plugin's entire surface is one deterministic
  // SessionStart hook; it must not also present as something Claude chooses to invoke.
  //
  // This is a live hazard, not a hypothetical: `link-lab-assets.sh` discovers an asset by either a
  // `SKILL.md` or a `.claude-plugin/plugin.json`, so dropping in a stub `SKILL.md` is always the
  // cheaper-looking way to make a hooks-only plugin link — and it silently buys a second, invocable
  // surface along with the link.
  assert.ok(
    !fs.existsSync(path.join(pluginRoot, 'SKILL.md')),
    'a root SKILL.md would register this plugin as an invocable skill as well as a hook'
  );
});

test('the plugin version is declared', () => {
  // `plugin.json` is the single place this plugin's version lives — there is no marketplace entry
  // to agree with, so nothing else can catch it going missing. Asserted because `plugin details`
  // and every update path read it, and an absent version reads as an unversioned plugin rather
  // than an error.
  const plugin = readJson(path.join(pluginRoot, '.claude-plugin/plugin.json'));
  assert.equal(typeof plugin.version, 'string');
  assert.ok(plugin.version.length > 0);
});

test('link-lab-assets.sh actually links this plugin into the user scope', () => {
  // The always-on guarantee in one assertion. A project-scope skills-dir plugin loads only from
  // the `.claude/skills/` of the directory Claude Code was started in and does NOT walk up to the
  // repository root, so a session opened in a nested `projects/<repo>` clone would never see it.
  // The symlink into `~/.claude/skills/` is what makes it personal-scope and therefore
  // unconditional. If the link script stops discovering this directory, the plugin quietly
  // degrades to "fires only when you launch from the repo root" — and because it is fail-open,
  // nothing reports the degradation.
  const script = path.join(repoRoot, '.claude/scripts/link-lab-assets.sh');
  assert.ok(fs.existsSync(script), 'link-lab-assets.sh is missing');

  // A throwaway CLAUDE_HOME keeps the run off the developer's real ~/.claude. --dry-run writes
  // nothing regardless; the temp home is belt-and-braces so a future non-dry invocation here
  // cannot touch a real one.
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'context-gc-link-'));
  try {
    const out = execFileSync('bash', [script, '--dry-run'], {
      env: { ...process.env, CLAUDE_HOME: home },
      encoding: 'utf8',
    });
    assert.match(
      out,
      /^\s+(link|refresh)\s+context-gc$/m,
      `link-lab-assets.sh did not report linking context-gc:\n${out}`
    );
  } finally {
    fs.rmSync(home, { recursive: true, force: true });
  }
});

test('the plugin still produces a manifest when run THROUGH the deployed symlink', () => {
  // The one test that runs the plugin AS INSTALLED. Everything else in this suite — and every
  // other file — reaches the modules by relative path inside the repo tree, which is the one
  // arrangement no member ever runs.
  //
  // `link-lab-assets.sh` deploys this plugin as a DIRECTORY SYMLINK into ~/.claude/skills/, and
  // that symlink is what makes the hook always-on, because project scope does not reach nested
  // `projects/<repo>` sessions. Node resolves an ESM entry through symlinks before setting
  // `import.meta.url` but leaves `process.argv[1]` as given, so an entry-point guard that compares
  // the two as plain strings is FALSE on every real installation: the hook exits 0 having written
  // nothing, `CONTEXT_GC_DEBUG=1` prints nothing (the diagnostic is inside the main() that never
  // ran), and the result is indistinguishable from a session with nothing to recover. That
  // regression shipped past a full green suite once; only spawning through the link catches it.
  const script = path.join(repoRoot, '.claude/scripts/link-lab-assets.sh');
  assert.ok(fs.existsSync(script), 'link-lab-assets.sh is missing');

  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'context-gc-deployed-'));
  // Via the shared helper, not a local `git` wrapper: the helper owns the hermetic-env details in
  // one place precisely so a portability fix cannot land in some callers and miss others.
  const repo = createTempGitRepo('context-gc-deployed-repo-');
  try {
    // Real link, not --dry-run: the symlink itself is the thing under test.
    execFileSync('bash', [script], {
      env: { ...process.env, CLAUDE_HOME: home },
      encoding: 'utf8',
    });
    const linked = path.join(home, 'skills', 'context-gc');
    assert.ok(fs.existsSync(linked), 'link-lab-assets.sh did not deploy context-gc');
    assert.ok(
      fs.lstatSync(linked).isSymbolicLink(),
      'expected a symlink — the realpath/argv[1] divergence this test guards only exists through one'
    );

    // A dirty git repo gives the deterministic floor something real to report, so an empty
    // manifest cannot be mistaken for "nothing to say".
    fs.writeFileSync(path.join(repo, 'tracked.txt'), 'v1\n');
    git(repo, 'add', 'tracked.txt');
    git(repo, 'commit', '-qm', 'init');
    fs.writeFileSync(path.join(repo, 'tracked.txt'), 'v2\n');
    fs.writeFileSync(path.join(repo, 'untracked.txt'), 'new\n');

    const entry = path.join(linked, 'src', 'recover.mjs');
    const result = spawnSync(process.execPath, [entry], {
      input: JSON.stringify({
        source: 'compact',
        cwd: repo,
        transcript_path: path.join(repo, 'no-such-transcript.jsonl'),
      }),
      encoding: 'utf8',
      env: { ...process.env, CLAUDE_PLUGIN_ROOT: linked, CONTEXT_GC_OLLAMA_HOST: 'http://127.0.0.1:1' },
    });

    assert.equal(result.status, 0, `hook exited ${result.status}: ${result.stderr}`);
    assert.notEqual(
      result.stdout.trim(),
      '',
      'the hook wrote NOTHING when spawned through the deployed symlink — main() did not run'
    );
    const parsed = JSON.parse(result.stdout);
    assert.equal(parsed.hookSpecificOutput.hookEventName, 'SessionStart');
    assert.match(parsed.hookSpecificOutput.additionalContext, /## Files in flight/);
    assert.match(parsed.hookSpecificOutput.additionalContext, /untracked\.txt/);
  } finally {
    fs.rmSync(home, { recursive: true, force: true });
    cleanup(repo);
  }
});

test('link-lab-assets.sh skips a skills/ directory carrying neither marker', () => {
  // The negative half of the discovery predicate this PR widened. The positive half is asserted
  // above; with only that, replacing the whole predicate with `true` leaves the suite green, so
  // nothing pins that a plain directory under `.claude/skills/` stays unlinked. It must: the
  // directory is also where ATTRIBUTION.md and any future non-asset folder live, and a link
  // deploys its contents into every session on the machine.
  const script = path.join(repoRoot, '.claude/scripts/link-lab-assets.sh');
  const decoy = path.join(repoRoot, '.claude/skills', `not-an-asset-${process.pid}`);
  const home = fs.mkdtempSync(path.join(os.tmpdir(), 'context-gc-negative-'));
  fs.mkdirSync(decoy);
  fs.writeFileSync(path.join(decoy, 'notes.md'), 'neither a SKILL.md nor a plugin manifest\n');
  try {
    const out = execFileSync('bash', [script, '--dry-run'], {
      env: { ...process.env, CLAUDE_HOME: home },
      encoding: 'utf8',
    });
    assert.doesNotMatch(
      out,
      new RegExp(`\\b${path.basename(decoy)}\\b`),
      `a directory with neither SKILL.md nor .claude-plugin/plugin.json was reported as linked:\n${out}`
    );
  } finally {
    fs.rmSync(decoy, { recursive: true, force: true });
    fs.rmSync(home, { recursive: true, force: true });
  }
});
