/**
 * IPC payload types — TypeScript side of the contract.
 *
 * Hand-mirrored with `app/src-tauri/src/shared/types.rs` (CL-4 / Rust side).
 *
 * If you change a type here, change the matching struct in
 * `src-tauri/src/shared/types.rs` in the same PR. Codegen for a single source
 * of truth is post-V1.
 *
 * Field names are snake_case throughout — no camelCase remapping at the IPC
 * boundary (Rust's serde rename_all = "snake_case" controls the wire format).
 *
 * SCOPE (CL-25 — full ADD §3.6 + §3.7 surface):
 * The CL-11 minimum subset (BRIDGE-1) covered the BRIDGE-1 read-path:
 * `SessionId`, `DifficultyLevel`, `Passage`, `Settings`, `AttemptRollup`,
 * `PhonemeAttempts`, `PhonemeThresholds`, `FlaggedPhoneme`,
 * `EvaluationResult`, `FirstRunPhase`. CL-25 closes the contract by landing
 * the deferred set: `InstallState`, `ConsentState`, `SessionSummary`,
 * `PhonemeTrend`, `QueueStatus`, `LastTerminalError`, `TerminalErrorCode`,
 * `UpdateInfo`, `TrendDirection`. Some of these have no Rust handler yet
 * (CL-7 / CL-8 / CL-10 / CL-20-22 / CL-23-lite) — the contract still lands
 * so the React surface can compile against the full ADD §3.6 / §3.7 shape;
 * runtime calls to handlers that don't exist surface as `IpcError` from the
 * command wrapper, which is the correct behaviour.
 *
 * Spec: ADD §3.6 (command payloads), ADD §3.7 (event payloads).
 */

// ---------------------------------------------------------------------------
// Session identifiers
// ---------------------------------------------------------------------------

/**
 * Client-generated v4 UUID for a single practice session. Mirrors the Rust
 * `SessionId` newtype (`#[serde(transparent)]` over `String`) — appears as a
 * bare JSON string on the wire.
 */
export type SessionId = string;

// ---------------------------------------------------------------------------
// Install / consent (CL-7 / CL-8)
// ---------------------------------------------------------------------------

/**
 * Tri-state consent posture. Mirrors the Rust enum's snake_case serialization
 * (`#[serde(rename_all = "snake_case")]`).
 *
 * Wire shape: `"pending" | "granted" | "revoked"`.
 *
 * CL-8 deferral: the consent state-machine writes are stubbed at the Rust
 * boundary until CL-8 lands. The type is published now so the consent screen
 * and settings surface can compile.
 */
export type ConsentState = 'pending' | 'granted' | 'revoked';

/**
 * Per-install identity and consent posture. Mirrors `InstallState` in
 * `shared/types.rs`. `uuid` is the v4 UUID created at first-run consent
 * accept.
 */
export type InstallState = {
  uuid: string;
  consent_state: ConsentState;
};

// ---------------------------------------------------------------------------
// Difficulty level + thresholds
// ---------------------------------------------------------------------------

/**
 * Named difficulty level. Resolves to a per-phoneme threshold map via the
 * bundled threshold table.
 */
export type DifficultyLevel = 'gentle' | 'standard' | 'strict';

/**
 * Per-phoneme certainty cutoff map for the active difficulty level. Keys are
 * IPA symbols; values are in `[0, 1]`.
 */
export type PhonemeThresholds = Record<string, number>;

// ---------------------------------------------------------------------------
// Passage (read-aloud content)
// ---------------------------------------------------------------------------

/** One word of the passage plus its bundled reference IPA sequence. */
export type ExpectedIpaPerWord = {
  word: string;
  ipa: string[];
};

/**
 * The passage the learner reads aloud, plus the bundled reference IPA
 * sequence. `text` is the canonical written form; `expected_ipa_per_word` is
 * what the inference pipeline aligns against.
 */
export type Passage = {
  text: string;
  expected_ipa_per_word: ExpectedIpaPerWord[];
};

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/**
 * User-mutable settings persisted across sessions. `difficulty` is the **named
 * level** (not a scalar pre-v0.4 number); per-phoneme cutoffs are derived at
 * evaluation time and surfaced on `EvaluationResult`.
 *
 * `update_checks_enabled` gates periodic network egress to check for a new
 * client version. Off by default (CL-23-lite); the user opts in via the
 * Settings screen.
 */
export type Settings = {
  l1: string;
  regional_variety: string | null;
  difficulty: DifficultyLevel;
  report_uploads_enabled: boolean;
  update_checks_enabled: boolean;
};

// ---------------------------------------------------------------------------
// Evaluation results
// ---------------------------------------------------------------------------

/**
 * Rollup of attempts for a single phoneme within a session. `mean_certainty`
 * is `null` when the phoneme had zero non-fallback occurrences (no real-frame
 * certainty score produced). See `AttemptRollup` in `shared/types.rs` for the
 * CL-17 fallback-occurrence filter rationale.
 */
export type AttemptRollup = {
  occurrences: number;
  flagged: number;
  mean_certainty: number | null;
};

/**
 * Map of IPA symbol to its per-session `AttemptRollup`. Wire shape is a flat
 * JSON object (e.g. `{ "θ": { ... }, "ð": { ... } }`).
 */
export type PhonemeAttempts = Record<string, AttemptRollup>;

/**
 * Single entry in `EvaluationResult.flagged_phonemes_ordered`. Carries the
 * example-word copy so the results screen does not have to look it up.
 */
export type FlaggedPhoneme = {
  phoneme: string;
  example_word: string;
  flag_count: number;
  mean_certainty: number;
};

/**
 * Full result payload for a completed session. Carries per-phoneme certainty
 * (FRD F-EVL-3) and the three v0.4 fields (`difficulty_level`,
 * `difficulty_thresholds`, `threshold_table_version`) applied at evaluation
 * time. The legacy single-scalar `difficulty_threshold` is intentionally
 * absent.
 *
 * Spec: ADD §3.6.
 */
export type EvaluationResult = {
  session_id: SessionId;
  started_at: string; // ISO 8601 UTC
  ended_at: string; // ISO 8601 UTC
  duration_seconds: number;
  phoneme_attempts: PhonemeAttempts;
  difficulty_level: DifficultyLevel;
  difficulty_thresholds: PhonemeThresholds;
  threshold_table_version: number;
  reattempt_counts_by_sentence: number[];
  flagged_phonemes_ordered: FlaggedPhoneme[];
  highest_error_phoneme: string | null;
  model_version: string;
};

// ---------------------------------------------------------------------------
// Feedback (CL-18)
// ---------------------------------------------------------------------------

/**
 * One rule-based feedback card for a flagged phoneme. Mirrors
 * `evaluation::feedback::FeedbackEntry` on the Rust side and is emitted as part
 * of the `eval:done` payload (alongside the `EvaluationResult`).
 *
 * Extension to the CL-11 minimum subset: BRIDGE-1 needs to render this on the
 * Results screen. Keep the wire shape (snake_case) and the optional
 * `learn_more_url` (V1: always `null`; the field is reserved for post-V1 deep
 * links into the IAS pedagogy site).
 */
export type FeedbackEntry = {
  phoneme: string;
  example_word: string;
  /**
   * Single plain-language "how to make this sound" paragraph. Collapsed from
   * the former tongue_placement / lip_shape / voicing / airflow split per IAS
   * review (2026-06). Mirrors `ArticulationEntry.mouth_shape` on the Rust side.
   */
  mouth_shape: string;
  /**
   * Two contrasting words for the "Say this pair" practice line (e.g.
   * "light / right"). Empty when the phoneme has no natural word pair (/ʒ/,
   * /dʒ/, /z/) — the Results screen omits the pair line.
   */
  minimal_pair: string;
  flag_count: number;
  learn_more_url: string | null;
};

// ---------------------------------------------------------------------------
// History / progress (CL-20 / CL-21)
// ---------------------------------------------------------------------------

/**
 * Compact summary of a completed session, used in the history list. Mirrors
 * `SessionSummary` in `shared/types.rs`.
 */
export type SessionSummary = {
  session_id: SessionId;
  ended_at: string; // ISO 8601 UTC
  duration_seconds: number;
  flagged_count: number;
  highest_error_phoneme: string | null;
};

/**
 * Direction a phoneme's flag rate has moved over the observed window. Mirrors
 * the Rust enum's snake_case serialization.
 *
 * Wire shape: `"improving" | "worsening" | "flat"`.
 */
export type TrendDirection = 'improving' | 'worsening' | 'flat';

/** Cross-session trend for a single phoneme. */
export type PhonemeTrend = {
  phoneme: string;
  example_word: string;
  attempts_total: number;
  flagged_total: number;
  trend_direction: TrendDirection;
  sessions_observed: number;
  /**
   * Per-observed-session flag rate in [0,1], oldest→newest. Lower is better.
   * One entry per session counted in `sessions_observed`. Drives the sparkline.
   */
  session_flag_rate: number[];
};

// ---------------------------------------------------------------------------
// Upload queue status (CL-10)
// ---------------------------------------------------------------------------

/**
 * HTTP-status discriminator for the most recent terminal upload error.
 *
 * Wire shape: the literal string `"401" | "403" | "410"` (NOT the variant
 * name). The Rust enum uses `#[serde(rename = "401")]` etc. to force the
 * numeric-string serialization. Match the wire shape exactly when narrowing.
 */
export type TerminalErrorCode = '401' | '403' | '410';

/** Last terminal (non-retryable) upload error, plus the timestamp it occurred. */
export type LastTerminalError = {
  code: TerminalErrorCode;
  at: string; // ISO 8601 UTC
};

/** Snapshot of the report-upload queue. */
export type QueueStatus = {
  pending_count: number;
  last_attempt_at: string | null; // ISO 8601 UTC
  last_terminal_error: LastTerminalError | null;
};

// ---------------------------------------------------------------------------
// Update info (CL-23)
// ---------------------------------------------------------------------------

/**
 * Information about an available client update. `version` and `notes` are
 * `null` when no update is pending.
 */
export type UpdateInfo = {
  available: boolean;
  version: string | null;
  notes: string | null;
};

// ---------------------------------------------------------------------------
// First-run state machine
// ---------------------------------------------------------------------------

/**
 * First-run phase. String-literal union — PascalCase tags match the Rust enum
 * exactly (`#[serde(rename_all = "PascalCase")]`).
 *
 * V1 has no recovery-code variants — recovery codes are cut.
 */
export type FirstRunPhase =
  | 'WelcomePending'
  | 'L1Pending'
  | 'ConsentPending'
  | 'ModelDownloading'
  | 'Ready';

// ---------------------------------------------------------------------------
// Compile-time exhaustiveness checks
// ---------------------------------------------------------------------------

// Compile-time discrimination check for FirstRunPhase. If a member is dropped
// from the union without updating the mapped-type below, `tsc` errors because
// the `Record<FirstRunPhase, true>` key set would no longer be satisfied by
// the literal object. This catches drops (the high-risk drift mode) — adds
// without a corresponding entry below also fail, since the literal would be
// missing the new key. No runtime cost; tree-shaken in the Vite build.
const _FirstRunPhaseExhaustive: Record<FirstRunPhase, true> = {
  WelcomePending: true,
  L1Pending: true,
  ConsentPending: true,
  ModelDownloading: true,
  Ready: true,
};
void _FirstRunPhaseExhaustive;

const _ConsentStateExhaustive: Record<ConsentState, true> = {
  pending: true,
  granted: true,
  revoked: true,
};
void _ConsentStateExhaustive;

const _TrendDirectionExhaustive: Record<TrendDirection, true> = {
  improving: true,
  worsening: true,
  flat: true,
};
void _TrendDirectionExhaustive;

const _TerminalErrorCodeExhaustive: Record<TerminalErrorCode, true> = {
  '401': true,
  '403': true,
  '410': true,
};
void _TerminalErrorCodeExhaustive;

const _DifficultyLevelExhaustive: Record<DifficultyLevel, true> = {
  gentle: true,
  standard: true,
  strict: true,
};
void _DifficultyLevelExhaustive;
