// context-gc — tests for the plugin's WIRING rather than its logic.
//
// Every other test file imports the source modules by relative path, so all 100+ of them stay
// green if the hook command, the marketplace entry, or the plugin manifest points somewhere that
// no longer exists. That failure mode is the hardest one to notice in this plugin specifically:
// context-gc is deliberately fail-open, so a hook that never fires is indistinguishable from a
// session that simply had nothing to recover. These tests assert the declarations agree with the
// files on disk.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

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
  assert.doesNotThrow(() => readJson(path.join(repoRoot, '.claude-plugin/marketplace.json')));
});

test('the SessionStart hook command points at a file that exists', () => {
  // The one declaration that makes the plugin do anything at all. `${CLAUDE_PLUGIN_ROOT}` is
  // substituted by the harness at invocation time; resolving it against the plugin root here is
  // exactly what the harness will do.
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
  }
});

test('the hook matches the compaction event it exists to serve', () => {
  const hooks = readJson(path.join(pluginRoot, 'hooks/hooks.json'));
  const matchers = hooks.hooks.SessionStart.map((entry) => entry.matcher);
  assert.ok(matchers.includes('compact'), `expected a "compact" matcher, got ${matchers.join(', ')}`);
});

test('the marketplace entry resolves to this plugin directory', () => {
  const marketplace = readJson(path.join(repoRoot, '.claude-plugin/marketplace.json'));
  const entry = marketplace.plugins.find((p) => p.name === 'context-gc');

  assert.ok(entry, 'context-gc is not listed in the marketplace manifest');

  const resolved = path.resolve(repoRoot, entry.source);
  assert.equal(resolved, pluginRoot, 'marketplace source does not point at this plugin');
  assert.ok(
    fs.existsSync(path.join(resolved, '.claude-plugin/plugin.json')),
    'marketplace source resolves to a directory with no plugin manifest'
  );
});

test('the plugin version is declared once in effect: manifest and marketplace agree', () => {
  // Two independent copies of a version string drift silently; nothing else compares them.
  const plugin = readJson(path.join(pluginRoot, '.claude-plugin/plugin.json'));
  const marketplace = readJson(path.join(repoRoot, '.claude-plugin/marketplace.json'));
  const entry = marketplace.plugins.find((p) => p.name === 'context-gc');

  assert.equal(entry.version, plugin.version);
  assert.equal(entry.name, plugin.name);
});
