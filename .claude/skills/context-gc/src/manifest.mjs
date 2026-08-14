// context-gc — manifest assembly module.
//
// Owns formatting and byte-bounding ONLY. Consumes the already-normalized source outputs
// (`files` from git.mjs, `tasks` from transcript.mjs, `objective`/`decisions` from ollama.mjs
// via recover.mjs) and never touches raw record shape — that seam belongs to the source
// modules. Byte-cap enforcement lives in exactly one place: this module.
//
// `files`/`tasks` are the DETERMINISTIC FLOOR — always sourceable, never model-inferred.
// `objective`/`decisions` are ENRICHED — model-inferred by a local Ollama call (ollama.mjs) and
// the softest content in the manifest, so they render under one visible tag
// (`MODEL_INFERRED_TAG`, applied exactly once here — the only place the manifest names the
// enrichment source) and are the FIRST content dropped under byte-cap pressure, before the
// floor. `sources` is deliberately an object (not positional args) so it can carry both without
// changing this function's signature.
//
// Pure and stateless: every call re-derives the manifest fresh from whatever `sources` it is
// given. Nothing is persisted, read back, or accumulated across calls, so repeated compactions
// within one session never compound stale state — deduping is unnecessary by construction rather
// than handled after the fact.

const HEADER = '# Context manifest (pre-compaction floor)';

// The ONE place the manifest names the enrichment source. objective/decisions are model-inferred
// (local Ollama, see ollama.mjs) — never verified fact — so every enriched line renders under
// this single shared tag rather than a per-field inline string.
const MODEL_INFERRED_TAG = 'model-inferred — local Ollama; verify before treating as fact';

// Appended to a section whose entries were partially dropped by the byte cap, so a trimmed list
// is never read as a complete enumeration. Only ever appended when at least one entry survives:
// a section trimmed to nothing is omitted entirely, which is unambiguous on its own and keeps a
// pathological cap reachable (a marker on an empty section would consume bytes forever).
const TRUNCATION_MARKER_PREFIX = '- … ';

// The fewest bytes an entry line can occupy in the rendered document: `- M: ` — the shortest form
// `renderFileLine` can emit (a one-character status and an empty path) — plus the newline that
// joins it to its neighbour. Used ONLY as a safe lower bound when pre-bounding the deterministic
// arrays in `buildManifest`. Understating it is harmless (a looser ceiling, still O(cap));
// overstating it would discard an entry that could have survived, so it stays conservative.
const MIN_ENTRY_BYTES = 6;

function renderTruncationMarker(droppedCount) {
  // Deliberately terse. This line competes with real entries for the same byte cap, so every
  // word it spends is an entry it may cost; "… N more" carries the whole signal.
  return `${TRUNCATION_MARKER_PREFIX}${droppedCount} more`;
}

/**
 * Counts the real entry lines in a rendered manifest — list items that carry content, excluding
 * any truncation marker. This is the quantity the byte cap is spent on, and therefore the one
 * the two competing trims in `buildManifest` are compared by.
 *
 * @param {string} markdown
 * @returns {number}
 */
function countEntryLines(markdown) {
  if (typeof markdown !== 'string' || markdown === '') return 0;
  return markdown
    .split('\n')
    .filter((line) => line.startsWith('- ') && !line.startsWith(TRUNCATION_MARKER_PREFIX))
    .length;
}

/**
 * Flattens any value destined for a rendered line to a single line of text.
 *
 * Every rendered line interpolates a string that this module did not author — model-inferred
 * enrichment from ollama.mjs, task content from the harness transcript, paths from git. A value
 * containing a newline would otherwise forge manifest STRUCTURE: an embedded `## Files in flight`
 * heading inside an enriched decision renders below `MODEL_INFERRED_TAG` but reads to the
 * resuming agent as deterministic floor, which is exactly the distinction the tag exists to
 * draw. Collapsing whitespace here is the one place that guarantee is enforced, so no caller can
 * bypass it.
 *
 * @param {unknown} value
 * @returns {string}
 */
function flattenToLine(value) {
  return typeof value === 'string' ? value.replace(/\s+/g, ' ').trim() : '';
}

/**
 * Renders one file entry as a single markdown list line.
 * @param {{path: string, status: string}} file
 * @returns {string}
 */
function renderFileLine(file) {
  const rawStatus = flattenToLine(file && file.status);
  // Degrades to the honest `unknown` rather than guessing `modified`: this line sits in the
  // deterministic floor, where an invented status would be indistinguishable from a sourced one.
  const status = rawStatus !== '' ? rawStatus : 'unknown';
  const filePath = flattenToLine(file && file.path);
  return `- ${status}: ${filePath}`;
}

/**
 * Renders one task entry as a single markdown list line.
 * @param {{content: string, status: string}} task
 * @returns {string}
 */
function renderTaskLine(task) {
  const rawStatus = flattenToLine(task && task.status);
  const status = rawStatus !== '' ? rawStatus : 'unknown';
  const content = flattenToLine(task && task.content);
  return `- [${status}] ${content}`;
}

/**
 * Renders one decision/open-thread entry as a single markdown list line.
 * @param {string} decision
 * @returns {string}
 */
function renderDecisionLine(decision) {
  return `- ${flattenToLine(decision)}`;
}

/**
 * Renders the enriched (model-inferred) section, or `null` when there is nothing to show — an
 * absent/empty `objective` AND an empty `decisions` list omit the section entirely, the same
 * "no wasted bytes on a placeholder" pattern the deterministic sections use. Both fields render
 * under the single `MODEL_INFERRED_TAG` heading — there is no per-field tag string.
 * @param {string|null} objective
 * @param {string[]} decisions
 * @returns {string|null}
 */
function renderEnrichedSection(objective, decisions) {
  const lines = [];
  const flatObjective = flattenToLine(objective);
  if (flatObjective !== '') lines.push(`**Objective:** ${flatObjective}`);
  if (decisions.length > 0) {
    lines.push('**Decisions / open threads:**', ...decisions.map(renderDecisionLine));
  }
  if (lines.length === 0) return null;
  return [`## Session context (${MODEL_INFERRED_TAG})`, ...lines].join('\n');
}

/**
 * Renders the full manifest markdown from (possibly already-trimmed) `files`/`tasks` arrays and
 * `objective`/`decisions` enrichment. A section is omitted entirely when it has nothing to show
 * (no wasted bytes on a placeholder), and the whole document collapses to `''` when everything is
 * empty — never a bare/lonely header. Deterministic sections render first (files, then tasks);
 * the enriched section renders last, reflecting its lower priority under the byte cap (see
 * `buildManifest`).
 * A partially-trimmed deterministic section carries a truncation marker naming how many entries
 * were dropped, so a short list is never mistaken for a complete one.
 *
 * @param {Array<{path: string, status: string}>} files
 * @param {Array<{content: string, status: string}>} tasks
 * @param {string|null} objective
 * @param {string[]} decisions
 * @param {{files: number, tasks: number}} [dropped] entries removed by the byte cap, per section
 * @returns {string}
 */
function render(files, tasks, objective, decisions, dropped = { files: 0, tasks: 0 }) {
  const sections = [];

  if (files.length > 0) {
    const lines = ['## Files in flight', ...files.map(renderFileLine)];
    if (dropped.files > 0) lines.push(renderTruncationMarker(dropped.files));
    sections.push(lines.join('\n'));
  }
  if (tasks.length > 0) {
    const lines = ['## Tasks', ...tasks.map(renderTaskLine)];
    if (dropped.tasks > 0) lines.push(renderTruncationMarker(dropped.tasks));
    sections.push(lines.join('\n'));
  }
  const enrichedSection = renderEnrichedSection(objective, decisions);
  if (enrichedSection) sections.push(enrichedSection);

  if (sections.length === 0) return '';

  return [HEADER, ...sections].join('\n\n') + '\n';
}

/**
 * Assembles the manifest — `sources.files` + `sources.tasks` (the deterministic floor) plus
 * optional `sources.objective` + `sources.decisions` (model-inferred enrichment) — into a
 * compact markdown string no larger than `maxBytes` UTF-8 bytes.
 *
 * Over-cap handling drops WHOLE lowest-priority lines until the result fits — never a mid-line
 * or mid-character split. Priority, softest-first: enriched `decisions` (popped one at a time),
 * then the enriched `objective` (dropped whole), then `tasks` (popped one at a time), then
 * `files` (popped one at a time) — files are highest priority (what's in flight must survive),
 * decisions/objective are lowest (model-inferred, not verified fact) per the "guaranteed floor"
 * requirement: enrichment must never crowd out the deterministic floor. Bytes are measured with
 * `Buffer.byteLength(str, 'utf8')`, never JS string length (UTF-16 code units), so multibyte
 * content (emoji, non-ASCII paths) is bounded correctly. If even the empty-of-content header
 * cannot fit (a pathological `maxBytes`), the result falls back to `''`, which is always ≤ any
 * non-negative cap. A partially-trimmed `files`/`tasks` section carries a truncation marker, so
 * the floor is never silently presented as a complete enumeration. The marker is preferred even
 * when it costs an entry line — an unmarked short list makes a completeness claim, which is the
 * more expensive error. It is dropped ONLY at a cap so tight that paying for it would leave no
 * entries at all. A section trimmed away entirely is omitted, making no completeness claim.
 *
 * The byte bound is enforced only for a finite `maxBytes`. A non-finite cap (`undefined`, `NaN`,
 * `Infinity`) returns the untrimmed manifest — callers in this plugin always pass the integer
 * `config.mjs` resolves, so this path is a defensive escape rather than a supported mode.
 *
 * Pure and stateless: does not read, write, or merge with any prior manifest; does not mutate any
 * `sources` field. A fresh call with the same inputs always returns the same output.
 *
 * @param {{files: Array<{path: string, status: string}>, tasks: Array<{content: string, status: string}>,
 *   objective?: string|null, decisions?: string[]}} sources
 * @param {number} maxBytes
 * @returns {string}
 */
export function buildManifest(sources, maxBytes) {
  const trim = (withMarkers) => {
    const files = sources && Array.isArray(sources.files) ? sources.files.slice() : [];
    const tasks = sources && Array.isArray(sources.tasks) ? sources.tasks.slice() : [];
    let objective = sources && typeof sources.objective === 'string' && sources.objective.trim() !== ''
      ? sources.objective
      : null;
    let decisions = sources && Array.isArray(sources.decisions)
      ? sources.decisions.filter((d) => typeof d === 'string' && d.trim() !== '')
      : [];

    const dropped = { files: 0, tasks: 0 };

    // Bound the deterministic arrays BEFORE the tier loop below. That loop pops ONE entry and
    // re-renders the WHOLE document per drop, so it costs O(n²) in whatever `git status` emitted.
    // Nothing upstream bounds n: `git.mjs`'s `GIT_TIMEOUT_MS` caps how long git may RUN, not how
    // much it may EMIT, and a mass reformat, an EOL/`.gitattributes` flip, or a generated tracked
    // tree reaches five figures routinely. Unbounded, this pure function alone stalls session
    // resume for tens of seconds (measured on this tree: 20,000 entries → ~34 s) — the exact
    // outcome every other slow path here is capped to prevent, and worse than git and Ollama
    // combined.
    //
    // Output-neutral by construction, which is why this is a bound and not a policy change: the
    // tier loop drops from the END, so whatever survives is a PREFIX of the array, and no entry
    // past `maxBytes / MIN_ENTRY_BYTES` can survive a cap of `maxBytes`. Entries removed here are
    // credited to the same `dropped` counters the loop uses, so the truncation marker still names
    // the true total and never understates what was withheld.
    //
    // RESIDUAL, stated rather than silently left: the bound's strength tracks `maxBytes`, it is
    // not constant. The loop is still O(ceiling²), and `ceiling` is `maxBytes / MIN_ENTRY_BYTES` —
    // ~667 entries at the 4,000-byte default (immaterial), but `CONTEXT_GC_MAX_BYTES` is
    // operator-settable with no upper limit, so a very large cap re-opens the window for a working
    // tree large enough to overflow it. An absolute entry cap would close it, but would also
    // change what a large cap renders, forfeiting the output-neutrality above — the property that
    // makes this a safe bound rather than a policy change. Closing it properly means making the
    // trim itself sub-quadratic (bulk-drop by prefix byte sums instead of one entry per
    // re-render), which is a larger change than this bound.
    if (Number.isFinite(maxBytes)) {
      const ceiling = Math.max(0, Math.ceil(maxBytes / MIN_ENTRY_BYTES));
      if (files.length > ceiling) dropped.files += files.splice(ceiling).length;
      if (tasks.length > ceiling) dropped.tasks += tasks.splice(ceiling).length;
    }

    const counts = () => (withMarkers ? dropped : { files: 0, tasks: 0 });
    const rerender = () => render(files, tasks, objective, decisions, counts());

    let markdown = rerender();

    if (!Number.isFinite(maxBytes)) return markdown;

    // The trim priority is stated ONCE, here, as data in softest-first order — not encoded four
    // times in near-identical control flow. Adding or reordering a tier is an edit to this list;
    // the loop below never changes. `canDrop` reports whether the tier still has something to
    // give up, `drop` gives up exactly one unit of it.
    const TIERS = [
      { canDrop: () => decisions.length > 0, drop: () => { decisions = decisions.slice(0, -1); } },
      { canDrop: () => objective !== null, drop: () => { objective = null; } },
      { canDrop: () => tasks.length > 0, drop: () => { tasks.pop(); dropped.tasks += 1; } },
      { canDrop: () => files.length > 0, drop: () => { files.pop(); dropped.files += 1; } },
    ];

    const overCap = () => Buffer.byteLength(markdown, 'utf8') > maxBytes;

    for (const tier of TIERS) {
      while (overCap() && tier.canDrop()) {
        tier.drop();
        markdown = rerender();
      }
      if (!overCap()) break;
    }

    return overCap() ? '' : markdown;
  };

  // A truncation marker costs bytes against the same cap as the content it describes, so the two
  // compete directly and the trade has to be chosen rather than assumed.
  //
  // The MARKED result is preferred whenever it carries any content at all, even though it can
  // cost an entry line. Rationale: an unmarked short list is read as a COMPLETE list — the
  // resuming agent concludes those are all the files in flight — and this plugin's whole posture
  // is that absent or qualified output beats confidently wrong output. One fewer path, plus
  // "… N more", beats one more path presented as the whole truth.
  //
  // The unmarked trim is the fallback for the degenerate case only: a cap so tight that paying
  // for the marker leaves no entries at all, where the marker would be describing an emptiness
  // the reader can already see.
  const marked = trim(true);
  return countEntryLines(marked) > 0 ? marked : trim(false);
}
