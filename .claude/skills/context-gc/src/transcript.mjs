// context-gc — transcript source module.
//
// THE FRAGILE SEAM: this is the ONLY module in the plugin that knows the internal transcript
// `.jsonl` shape — an undocumented, harness-owned format that can change without notice. Every
// fact this module learns about that shape is normalized away before it leaves: callers get
// format-neutral `{ role, text }` tail records and a plain `tasks` list, never a raw record. If
// the harness changes the `.jsonl` shape tomorrow, this is the only file that has to change.
//
// Real shape (verified empirically against real sessions, not from documentation): JSONL, one
// record per line;
// top-level `type` is `"user"`, `"assistant"`, or a compaction-summary record; text lives at
// `message.content`, which is either a plain string or an array of blocks (`{type:"text",text}`
// and/or `{type:"tool_use",name,input}`); the compaction marker is a record carrying
// `isCompactSummary: true`; task state is a `tool_use` block named `TodoWrite`, `TaskCreate`, or
// `TaskUpdate` (only `TodoWrite`'s `input.todos[]` shape — `{content,status,activeForm}` — is
// confirmed; the other two, and any `TodoWrite` missing `input.todos`, are unparseable). Task
// extraction takes the most recent *parseable* `TodoWrite`, empty or not — so an intentional
// `TodoWrite({todos: []})` correctly wins over a stale non-empty list, at the cost of not being
// able to tell "the assistant cleared its list" apart from "no task tool ran here" — both read
// as `[]` to callers.

import { readFileSync } from 'node:fs';

const TASK_TOOL_NAMES = new Set(['TodoWrite', 'TaskCreate', 'TaskUpdate']);
// Deep-frozen, not shallow: `readTranscript`'s docstring calls the function pure, and this
// singleton is returned on every degraded read in the process. Left shallow, a caller that pushed
// onto the returned `tail` would silently poison every later degraded read.
const EMPTY_RESULT = Object.freeze({ tail: Object.freeze([]), tasks: Object.freeze([]) });

// Marks a line that did not parse as JSON, held at its original index so the read stays
// position-accurate. Never leaves this module: `readTranscript` either rejects the read outright
// or confines the sentinel to a region no caller sees.
const CORRUPT = Object.freeze({ __contextGcCorruptRecord: true });

// Marks a TORN trailing record — the last non-blank line, unparseable because the harness was
// still appending when this read happened. Distinct from CORRUPT: a torn tail is expected at the
// moment a SessionStart(compact) hook fires, whereas mid-file corruption is not. Retained
// positionally (rather than dropped) so `readTranscript` can reason about what it might have
// been — it is the single record most likely to have BEEN the newest compaction summary.
const TORN = Object.freeze({ __contextGcTornRecord: true });

/**
 * Normalizes a raw `message.content` value (string, block array, or anything else) to a plain
 * string: a string passes through as-is; an array joins its `type: "text"` blocks' `text` with
 * `"\n"` (non-text blocks, e.g. `tool_use`, are dropped); anything else normalizes to `""`.
 */
function normalizeContent(content) {
  if (typeof content === 'string') return content;
  if (Array.isArray(content)) {
    return content
      .filter((block) => block && block.type === 'text' && typeof block.text === 'string')
      .map((block) => block.text)
      .join('\n');
  }
  return '';
}

/**
 * Normalizes a window of raw records to format-neutral `{ role, text }` entries. Only
 * `type: "user"` / `type: "assistant"` records become tail entries; anything else (a stray
 * summary-type record, a malformed entry) is silently skipped rather than surfaced raw.
 */
function normalizeTail(window) {
  const tail = [];
  for (const record of window) {
    if (!record || (record.type !== 'user' && record.type !== 'assistant')) continue;
    // An older typed marker inside the window would otherwise put a PREVIOUS cycle's compaction
    // summary into both the tail and the Ollama prompt.
    if (record.isCompactSummary === true) continue;
    const content = record.message && record.message.content;
    tail.push({ role: record.type, text: normalizeContent(content) });
  }
  return tail;
}

/**
 * Normalizes one task-tool `tool_use` block's `input` to a plain `{content, status}` list, or
 * `null` if the block cannot be parsed as task state. Only `TodoWrite`'s `input.todos[]` shape is
 * confirmed: a block whose `input.todos` is an array normalizes to that array (possibly empty —
 * an intentional cleared list is a valid, parseable result, not a parse failure). A block
 * carrying no `input.todos` array is unparseable and normalizes to `null`.
 *
 * This function does NOT inspect `block.name`; filtering to the task tools happens upstream in
 * `extractTasks` via `TASK_TOOL_NAMES`. In practice only `TodoWrite` is known to carry the
 * `todos` shape, but a `TaskCreate`/`TaskUpdate` block that did would parse here too.
 */
function normalizeTaskBlock(block) {
  const todos = block.input && block.input.todos;
  if (!Array.isArray(todos)) return null;
  return todos
    .filter((todo) => todo && typeof todo.content === 'string')
    .map((todo) => ({ content: todo.content, status: todo.status }));
}

/**
 * Scans a window of raw records for task-tool `tool_use` blocks (name in `TodoWrite` /
 * `TaskCreate` / `TaskUpdate`), newest-first, and returns the normalized task list of the first
 * one that parses (i.e. the most recent parseable `TodoWrite`) — whether that list is empty or
 * not. An unparseable block (`TaskCreate`/`TaskUpdate`, which normalize to `null`) is skipped
 * rather than allowed to override an earlier, parseable `TodoWrite` block, so a trailing
 * unparseable block never discards valid task state. Trade-off: this can't distinguish "no task
 * tool ran" from "the assistant intentionally cleared its list" — both surface as `[]` — but it
 * does correctly prefer a more recent cleared list over a stale non-empty one. Returns `[]` when
 * no task-tool block in the window parses.
 *
 * A THIRD case currently folds in with those two: `normalizeTaskBlock` drops entries whose
 * `content` is not a string, so a `TodoWrite` whose entries ALL fail that filter (a harness
 * field rename, say) normalizes to `[]` — a successful parse — and outranks an older, still
 * valid list, surfacing to the reader as "the assistant cleared its list". Whether a block that
 * survives the filter empty should instead return `null`, so the previous valid list holds, is
 * an open question and not settled here.
 */
function extractTasks(window) {
  const taskBlocks = [];
  for (const record of window) {
    if (!record || record.type !== 'assistant') continue;
    const content = record.message && record.message.content;
    if (!Array.isArray(content)) continue;
    for (const block of content) {
      if (block && block.type === 'tool_use' && TASK_TOOL_NAMES.has(block.name)) {
        taskBlocks.push(block);
      }
    }
  }
  for (let i = taskBlocks.length - 1; i >= 0; i -= 1) {
    const tasks = normalizeTaskBlock(taskBlocks[i]);
    if (tasks !== null) return tasks;
  }
  return [];
}

/**
 * Resolves `tailRecords` to a non-negative integer window size, tolerating any garbage input
 * (missing, negative, non-numeric) by falling back to `0` rather than throwing. A `0` window
 * yields an empty tail AND empty tasks; `config.mjs` rejects non-positive values precisely so
 * this fallback is never reached through configuration.
 */
function resolveWindowSize(tailRecords) {
  return Number.isFinite(tailRecords) && tailRecords > 0 ? Math.floor(tailRecords) : 0;
}

/**
 * Reads and parses a transcript `.jsonl` file into an array of records, POSITION-PRESERVING:
 * a non-trailing line that fails to parse as JSON becomes a `CORRUPT` sentinel at its own index
 * rather than collapsing the read. Returns `null` (instead of throwing) only when the file is
 * missing or unreadable.
 *
 * A torn trailing line is retained as a distinct `TORN` sentinel rather than dropped: the harness
 * can still be appending when this reads, leaving the last line half-written, and that record is
 * the one most likely to have been the newest compaction summary.
 *
 * Keeping the sentinel in place is what lets `readTranscript` decide whether the corruption
 * actually matters. A transcript can hold tens of thousands of records while only the few
 * immediately preceding the last compaction marker are ever used; discarding the whole read for
 * a bad line in a region that would never have been consulted costs the user their task list and
 * tail for no gain. Corruption INSIDE the consulted window is still fatal to the read — see
 * `readTranscript`.
 */
function readRecords(transcriptPath) {
  let raw;
  try {
    raw = readFileSync(transcriptPath, 'utf8');
  } catch {
    return null;
  }

  const lines = raw.split(/\r?\n/).filter((line) => line.trim() !== '');
  const records = [];
  for (let i = 0; i < lines.length; i += 1) {
    try {
      records.push(JSON.parse(lines[i]));
    } catch {
      // A torn LAST line is the expected mid-append case; anything else is real corruption.
      records.push(i === lines.length - 1 ? TORN : CORRUPT);
    }
  }
  return records;
}

/**
 * Reads a Claude Code transcript and derives the pre-compaction tail + current task state.
 *
 * Locates the LAST record carrying `isCompactSummary: true` (a session may hold several
 * compactions; the most recent one wins), takes the `tailRecords` records immediately
 * PRECEDING it as the window, normalizes that window to format-neutral `{role,text}` tail
 * entries, and extracts task state from the window's most recent task-tool record.
 *
 * Fail-open — the complete list of degradations to `{ tail: [], tasks: [] }`; this enumeration is
 * exhaustive and load-bearing, so keep it in step with the body. Never throws.
 *   1. a missing or unreadable file;
 *   2. no `isCompactSummary` marker anywhere in the file;
 *   3. an unparseable line WITHIN the consulted window;
 *   4. an unparseable line NEWER than the selected marker (it may have been a newer marker, and
 *      serving the previous cycle as current is the failure this module must not have);
 *   5. a TORN trailing record when intact records sit between it and the selected marker — i.e.
 *      the session ran past that compaction, so the marker is not the one just written.
 *
 * Tolerated, NOT degraded: an unparseable line OLDER than the selected marker (far history the
 * plugin would never have read — it cannot hide a newer marker), and a torn trailing record where
 * the marker is the newest intact record (the ordinary mid-append case this hook fires during).
 *
 * Task state comes from the window's most recent PARSEABLE task-tool record.
 *
 * Pure function of its arguments — does not read config or env; callers (e.g. `recover.mjs`)
 * pass `getConfig().tailRecords` explicitly.
 *
 * @param {string} transcriptPath
 * @param {number} tailRecords
 * @returns {{tail: Array<{role: 'user'|'assistant', text: string}>, tasks: Array<{content: string, status: string | undefined}>}}
 */
export function readTranscript(transcriptPath, tailRecords) {
  const records = readRecords(transcriptPath);
  if (!records) return EMPTY_RESULT;

  let markerIndex = -1;
  for (let i = records.length - 1; i >= 0; i -= 1) {
    if (records[i] && records[i].isCompactSummary === true) {
      markerIndex = i;
      break;
    }
  }
  if (markerIndex === -1) return EMPTY_RESULT;

  // An unreadable record NEWER than the selected marker may itself have been a newer compaction
  // marker. If it was, everything below reports the previous cycle's tail and task list as the
  // current pre-compaction floor — stale state presented as fact, with nothing marking it stale.
  // That is the one failure this module must not have. Anything unreadable OLDER than the marker
  // stays tolerated: it cannot hide a newer marker, and that is what the positional sentinels buy.
  //
  // A TORN TRAILING record is handled separately and more permissively, by the loop below.
  // `readRecords` RETAINS it as the `TORN` sentinel rather than dropping it (see that constant's
  // docstring) precisely so this loop can judge it positionally: the harness is legitimately
  // mid-append at the moment this hook fires, so treating every torn tail as corruption would
  // forfeit recovery in the common case. The loop degrades on a torn tail only when an intact
  // record sits between it and the marker. The residual, accepted deliberately: a torn trailing
  // record with NO intact record after the marker is tolerated, so if that torn record was itself
  // a newer compaction summary, an older marker is still selected.
  let sawIntactAfterMarker = false;
  for (let i = markerIndex + 1; i < records.length; i += 1) {
    const record = records[i];

    // Mid-file corruption newer than the marker is always fatal: it may itself have been a newer
    // compaction summary, in which case everything below reports the PREVIOUS cycle's state as
    // current — stale content in the deterministic floor, with nothing marking it stale.
    if (record === CORRUPT) return EMPTY_RESULT;

    // A torn tail is fatal only when the marker we selected is NOT the newest intact record.
    // The discriminator is whether the session demonstrably continued past that compaction:
    //   · marker is the newest intact record (nothing intact after it) — the torn tail is a
    //     record being appended after the compaction that just fired, and the marker is current.
    //     Recovery proceeds; this is the common case the tolerance exists for.
    //   · intact records sit between the marker and the torn tail — the session ran on past that
    //     compaction, so the marker is demonstrably not the one just written, and the torn record
    //     is a plausible newer summary. Degrade rather than serve a previous cycle as current.
    // Sniffing the torn text for `isCompactSummary` would narrow this but not close it, since the
    // tear can fall before that key appears.
    if (record === TORN) {
      if (sawIntactAfterMarker) return EMPTY_RESULT;
      continue;
    }

    sawIntactAfterMarker = true;
  }

  const windowSize = resolveWindowSize(tailRecords);
  const startIndex = Math.max(0, markerIndex - windowSize);
  const window = records.slice(startIndex, markerIndex);

  // Corruption inside the region actually consulted is still fatal to the read: a window with a
  // hole in it would silently present an incomplete tail as a complete one.
  if (window.some((record) => record === CORRUPT)) return EMPTY_RESULT;

  return { tail: normalizeTail(window), tasks: extractTasks(window) };
}
