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
// Pure and stateless (the "dedupe" the commit subject refers to): every call re-derives the
// manifest fresh from whatever `sources` it is given. Nothing is persisted, read back, or
// accumulated across calls — repeated compactions never compound stale state (log 2026-07-03,
// resolves the PRD's within-session dedupe question in favor of re-derive-fresh).

const HEADER = '# Context manifest (pre-compaction floor)';

// The ONE place the manifest names the enrichment source. objective/decisions are model-inferred
// (local Ollama, see ollama.mjs) — never verified fact — so every enriched line renders under
// this single shared tag rather than a per-field inline string.
const MODEL_INFERRED_TAG = 'model-inferred — local Ollama; verify before treating as fact';

/**
 * Renders one file entry as a single markdown list line.
 * @param {{path: string, status: string}} file
 * @returns {string}
 */
function renderFileLine(file) {
  const status = typeof file.status === 'string' && file.status ? file.status : 'modified';
  const filePath = typeof file.path === 'string' ? file.path : '';
  return `- ${status}: ${filePath}`;
}

/**
 * Renders one task entry as a single markdown list line.
 * @param {{content: string, status: string}} task
 * @returns {string}
 */
function renderTaskLine(task) {
  const status = typeof task.status === 'string' && task.status ? task.status : 'unknown';
  const content = typeof task.content === 'string' ? task.content : '';
  return `- [${status}] ${content}`;
}

/**
 * Renders one decision/open-thread entry as a single markdown list line.
 * @param {string} decision
 * @returns {string}
 */
function renderDecisionLine(decision) {
  return `- ${typeof decision === 'string' ? decision : ''}`;
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
  if (objective) lines.push(`**Objective:** ${objective}`);
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
 * @param {Array<{path: string, status: string}>} files
 * @param {Array<{content: string, status: string}>} tasks
 * @param {string|null} objective
 * @param {string[]} decisions
 * @returns {string}
 */
function render(files, tasks, objective, decisions) {
  const sections = [];

  if (files.length > 0) {
    sections.push(['## Files in flight', ...files.map(renderFileLine)].join('\n'));
  }
  if (tasks.length > 0) {
    sections.push(['## Tasks', ...tasks.map(renderTaskLine)].join('\n'));
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
 * non-negative cap.
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
  const files = sources && Array.isArray(sources.files) ? sources.files.slice() : [];
  const tasks = sources && Array.isArray(sources.tasks) ? sources.tasks.slice() : [];
  let objective = sources && typeof sources.objective === 'string' && sources.objective.trim() !== ''
    ? sources.objective
    : null;
  let decisions = sources && Array.isArray(sources.decisions)
    ? sources.decisions.filter((decision) => typeof decision === 'string' && decision.trim() !== '')
    : [];

  let markdown = render(files, tasks, objective, decisions);

  if (!Number.isFinite(maxBytes)) return markdown;

  while (Buffer.byteLength(markdown, 'utf8') > maxBytes && decisions.length > 0) {
    decisions = decisions.slice(0, -1);
    markdown = render(files, tasks, objective, decisions);
  }

  while (Buffer.byteLength(markdown, 'utf8') > maxBytes && objective !== null) {
    objective = null;
    markdown = render(files, tasks, objective, decisions);
  }

  while (Buffer.byteLength(markdown, 'utf8') > maxBytes && tasks.length > 0) {
    tasks.pop();
    markdown = render(files, tasks, objective, decisions);
  }

  while (Buffer.byteLength(markdown, 'utf8') > maxBytes && files.length > 0) {
    files.pop();
    markdown = render(files, tasks, objective, decisions);
  }

  if (Buffer.byteLength(markdown, 'utf8') > maxBytes) return '';

  return markdown;
}

export default buildManifest;
