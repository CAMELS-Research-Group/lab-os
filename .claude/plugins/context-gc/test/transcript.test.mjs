// context-gc — tests for src/transcript.mjs
// transcript.mjs is the fragile seam: the only module that knows the raw `.jsonl` transcript
// shape. These tests exercise it against synthetic fixtures under test/fixtures/ whose field
// names/shapes are derived from a real (uncommitted) transcript (log 2026-07-03 04:02) — no real
// transcript content is committed. Every fixture read must come back normalized to
// format-neutral `{role,text}` tail entries and a plain `tasks` list; never raw record shape.
import { test } from 'node:test';
import assert from 'node:assert/strict';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { readTranscript } from '../src/transcript.mjs';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const fixture = (name) => path.join(__dirname, 'fixtures', name);

test('normalizes string-form message.content and windows to the records preceding the marker', () => {
  const { tail, tasks } = readTranscript(fixture('string-content.jsonl'), 2);
  assert.deepEqual(tail, [
    { role: 'user', text: 'record 3' },
    { role: 'assistant', text: 'record 4' },
  ]);
  assert.deepEqual(tasks, []);
});

test('clamps the window to the start of the file when tailRecords exceeds available records', () => {
  const { tail } = readTranscript(fixture('string-content.jsonl'), 100);
  assert.deepEqual(tail, [
    { role: 'user', text: 'record 1' },
    { role: 'assistant', text: 'record 2' },
    { role: 'user', text: 'record 3' },
    { role: 'assistant', text: 'record 4' },
  ]);
});

test('never includes records after the compaction marker in the tail', () => {
  const { tail } = readTranscript(fixture('string-content.jsonl'), 100);
  assert.ok(
    !tail.some((entry) => entry.text === 'post-compaction record'),
    'post-compaction record must not appear in the tail'
  );
});

test('normalizes array-form content: joins text blocks, drops tool_use blocks', () => {
  const { tail } = readTranscript(fixture('array-content.jsonl'), 3);
  assert.deepEqual(tail, [
    { role: 'user', text: 'start' },
    { role: 'assistant', text: 'Let me check that.\nOne sec.' },
    { role: 'user', text: 'continue' },
  ]);
});

test('extracts tasks from the most recent TodoWrite record in the tail', () => {
  const { tasks } = readTranscript(fixture('todo-write.jsonl'), 4);
  assert.deepEqual(tasks, [
    { content: 'task A', status: 'completed' },
    { content: 'task B', status: 'in_progress' },
  ]);
});

test('a later unparseable TaskCreate block does not override an earlier parseable TodoWrite', () => {
  const { tasks } = readTranscript(fixture('todo-then-taskcreate.jsonl'), 4);
  assert.deepEqual(tasks, [
    { content: 'task A', status: 'completed' },
    { content: 'task B', status: 'in_progress' },
  ]);
});

test('a later TodoWrite that clears the list wins over an earlier non-empty TodoWrite', () => {
  const { tasks } = readTranscript(fixture('todo-write-then-cleared.jsonl'), 4);
  assert.deepEqual(tasks, []);
});

test('multiple isCompactSummary markers: tail is taken before the LAST one', () => {
  const { tail } = readTranscript(fixture('multiple-markers.jsonl'), 2);
  assert.deepEqual(tail, [
    { role: 'user', text: 'after first, before second' },
    { role: 'assistant', text: 'ack2' },
  ]);
});

test('missing isCompactSummary marker degrades to empty', () => {
  const result = readTranscript(fixture('no-marker.jsonl'), 10);
  assert.deepEqual(result, { tail: [], tasks: [] });
});

test('missing transcript file degrades to empty, never throws', () => {
  assert.doesNotThrow(() => {
    const result = readTranscript(fixture('does-not-exist.jsonl'), 10);
    assert.deepEqual(result, { tail: [], tasks: [] });
  });
});

test('an unparseable line degrades the whole read to empty, never throws', () => {
  assert.doesNotThrow(() => {
    const result = readTranscript(fixture('garbage-line.jsonl'), 10);
    assert.deepEqual(result, { tail: [], tasks: [] });
  });
});

test('downstream never sees raw record shape: tail entries carry only role and text', () => {
  const { tail } = readTranscript(fixture('array-content.jsonl'), 3);
  for (const entry of tail) {
    assert.deepEqual(Object.keys(entry).sort(), ['role', 'text'].sort());
    assert.ok(entry.role === 'user' || entry.role === 'assistant');
    assert.equal(typeof entry.text, 'string');
  }
});

test('a torn trailing line is tolerated: intact preceding records still produce a non-empty result', () => {
  const { tail, tasks } = readTranscript(fixture('torn-trailing-line.jsonl'), 4);
  assert.deepEqual(tail, [
    { role: 'user', text: 'start task' },
    { role: 'assistant', text: '' },
  ]);
  assert.deepEqual(tasks, [{ content: 'task A', status: 'completed' }]);
});

test('a corrupt line that is not the last non-blank line still degrades the whole read (conservative behavior preserved)', () => {
  const result = readTranscript(fixture('garbage-line.jsonl'), 10);
  assert.deepEqual(result, { tail: [], tasks: [] });
});

test('an all-valid transcript with no torn line still yields its exact expected tasks (regression guard)', () => {
  const { tasks } = readTranscript(fixture('todo-write.jsonl'), 4);
  assert.deepEqual(tasks, [
    { content: 'task A', status: 'completed' },
    { content: 'task B', status: 'in_progress' },
  ]);
});

test('a trailing torn line with no intact records yields empty-but-non-null, degrading to EMPTY_RESULT without throwing', () => {
  assert.doesNotThrow(() => {
    const result = readTranscript(fixture('only-torn-line.jsonl'), 10);
    assert.deepEqual(result, { tail: [], tasks: [] });
  });
});

test('is a pure function of its arguments: does not read config or env', () => {
  const before = process.env.CONTEXT_GC_TAIL_RECORDS;
  process.env.CONTEXT_GC_TAIL_RECORDS = '999';
  try {
    const { tail } = readTranscript(fixture('string-content.jsonl'), 2);
    assert.equal(tail.length, 2);
  } finally {
    if (before === undefined) delete process.env.CONTEXT_GC_TAIL_RECORDS;
    else process.env.CONTEXT_GC_TAIL_RECORDS = before;
  }
});
