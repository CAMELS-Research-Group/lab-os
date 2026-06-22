//! Evaluation orchestrator + Tauri event emission (CL-19).
//!
//! Spec: planning task CL-19. This module is the integration seam that wires
//! all of CL-14 through CL-18 into the Tauri lifecycle:
//!
//! - takes the [`AudioBuffer`] [`crate::recording::commands::end_session`]
//!   stashes when a learner stops recording;
//! - drives a chunked phonemizer + single-pass alignment loop (see
//!   `run_chunked_phonemizer_then_align`); each chunk emits an
//!   `eval:progress` event after its forward pass completes. The
//!   chunk_orchestrator's `evaluate_chunked` remains in-tree for future
//!   progressive-UI work, but the production path no longer uses it
//!   (per-chunk alignment fails Viterbi feasibility when T_chunk < K);
//! - on completion: generates rule-based feedback (CL-18), persists the
//!   [`EvaluationResult`] to the `sessions` table, and emits `eval:done` with
//!   the full result + feedback list;
//! - on failure: emits `eval:error { session_id, kind, message }` and
//!   transitions the lifecycle to [`crate::recording::SessionState::Error`].
//!
//! # NO upload at eval:done
//!
//! Per the FRD F-RPT-1 "one report per cycle" contract, the upload-queue row
//! is enqueued on results-leave (CL-22), NOT here. This module writes the
//! `sessions` row and stops.
//!
//! # Threading
//!
//! [`run_evaluation`] returns synchronously after spawning a `tokio::task` so
//! the calling command (`end_session`) does not block. The orchestrator runs
//! on the Tauri-managed tokio runtime; the [`tauri::AppHandle`] is cloned in.
//!
//! # Vocab + threshold sourcing
//!
//! Both `threshold_table_v1.json` and `model_vocab_v1.json` are bundled via
//! `tauri.conf.json::bundle.resources` and resolved at orchestrator startup
//! through `BaseDirectory::Resource`. The vocab JSON is the extracted
//! `ipa_vocabulary` array from `tools/model_export/build/manifest.json` — see
//! `resources/model_vocab_v1.json` for the on-disk shape.
//!
//! # Persistence shape
//!
//! [`persist_evaluation_result`] writes one row to `sessions` with the v0.4
//! fields populated (`difficulty_level`, `difficulty_thresholds_json`,
//! `threshold_table_version`). The `learner_rating` / `learner_note` /
//! `feedback_submitted_at` columns remain NULL — the post-session rating UI
//! fills those later (out of scope for CL-19).
//!
//! The `os_major` field is set to the literal `"unknown"` — a deliberate V1
//! shortcut to avoid pulling in the `sysinfo` crate. CL-23 (updater) is the
//! natural place to refine this once we have a reason to discriminate by OS
//! version.

use std::path::PathBuf;

use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Emitter, Manager};

use crate::evaluation::alignment::align_to_reference;
use crate::evaluation::chunk_orchestrator::{
    self, merge_occurrence, ChunkParams, EvaluationContext, PartialChunkResult, PooledOccurrence,
};
use crate::evaluation::ctc_decode::ctc_greedy_decode;
use crate::evaluation::error::EvaluationError;
use crate::evaluation::feedback::{generate_feedback, FeedbackEntry};
use crate::evaluation::phonemizer::{OnnxPhonemizer, Phonemizer};
use crate::evaluation::reference_ipa::{load_expected_phonemes, load_passage, ExpectedPhoneme};
use crate::evaluation::thresholds::ThresholdTable;
use crate::recording::{AudioBuffer, MicrophoneError};
use crate::settings::commands::get_settings_impl;
use crate::shared::config::BuildConfig;
use crate::shared::error::AppError;
use crate::shared::types::{
    DifficultyLevel, EvaluationResult, Passage, PhonemeThresholds, SessionId, Settings,
};
use crate::storage::Connection;
use crate::AppState;

// ---------------------------------------------------------------------------
// Wire payloads
// ---------------------------------------------------------------------------

/// Wire payload for `eval:progress`. Stages emitted:
///
/// - `"chunk"` — per-chunk progress from the inner orchestrator.
/// - `"stopping"` — initial transition emitted before the chunked loop starts.
/// - `"building_feedback"` — emitted after the loop, before persistence.
/// - `"persisting"` — emitted right before the `sessions` INSERT.
///
/// `partial_result` is currently always `None` on every stage: the production
/// path runs the phonemizer in chunks but defers a single-pass alignment to
/// after the final chunk, so a progressive partial isn't available at chunk-
/// emission time. The field is preserved on the wire shape for the future
/// progressive-UI path (see [`WireChunkPartial`]).
#[derive(Debug, Clone, Serialize)]
pub struct ProgressPayload {
    pub session_id: SessionId,
    pub stage: String,
    pub pct: f32,
    pub partial_result: Option<WireChunkPartial>,
}

/// Serializable mirror of [`PartialChunkResult`]. Reserved for the future
/// progressive-UI path; the current production flow never populates it (see
/// [`ProgressPayload::partial_result`]). The conversion is kept so a future
/// progressive-alignment helper can emit real partials without re-deriving
/// the wire shape.
#[derive(Debug, Clone, Serialize)]
pub struct WireChunkPartial {
    pub flagged_phonemes_so_far: Vec<crate::shared::types::FlaggedPhoneme>,
    pub mean_certainty_so_far: std::collections::HashMap<String, f64>,
}

impl From<PartialChunkResult> for WireChunkPartial {
    fn from(p: PartialChunkResult) -> Self {
        Self {
            flagged_phonemes_so_far: p.flagged_phonemes_so_far,
            mean_certainty_so_far: p.mean_certainty_so_far,
        }
    }
}

/// Wire payload for `eval:done`.
#[derive(Debug, Clone, Serialize)]
pub struct DonePayload {
    pub result: EvaluationResult,
    pub feedback: Vec<FeedbackEntry>,
}

/// Wire payload for `eval:error`.
#[derive(Debug, Clone, Serialize)]
pub struct ErrorPayload {
    pub session_id: SessionId,
    pub kind: String,
    pub message: String,
}

// ---------------------------------------------------------------------------
// Public entry point
// ---------------------------------------------------------------------------

/// Bundled-resource paths (kept in lockstep with `tauri.conf.json::bundle.resources`).
const BUNDLED_THRESHOLD_TABLE_REL: &str = "resources/threshold_table_v1.json";
const BUNDLED_VOCAB_REL: &str = "resources/model_vocab_v1.json";
const BUNDLED_PASSAGE_REF_REL: &str = "passages/visiting_nyc.ipa.json";

/// Sentinel value for the `os_major` column. See module docs.
const OS_MAJOR_PLACEHOLDER: &str = "unknown";

/// Chunk width for the per-chunk phonemizer forward pass. SPIKE-15 measured
/// wav2vec2's attention as O(T²); 10 s at 16 kHz keeps the working set bounded
/// to ~0.7 GB/window — well inside the NF-MEM-1 budget for 8 GB pilot laptops.
/// Single-pass alignment over the concatenated posteriors is sub-millisecond
/// regardless of T, so no cross-chunk continuity is needed here (overlap = 0).
const CHUNK_SECONDS: f64 = 10.0;

/// Entry point invoked from `recording::commands::end_session`. Spawns a
/// `tokio::task` so the call returns immediately. The spawned task drives the
/// full evaluation pipeline + persistence + event emission.
pub fn run_evaluation(session_id: SessionId, audio: AudioBuffer, app_handle: AppHandle) {
    let app = app_handle.clone();
    tokio::spawn(async move {
        // The pipeline is CPU-bound (ONNX inference + alignment); run on a
        // blocking worker so the async runtime stays responsive. The closure
        // owns the app handle and audio buffer.
        let session_for_error = session_id.clone();
        let app_for_error = app.clone();
        let result = tokio::task::spawn_blocking(move || {
            run_inner_blocking(session_id, audio, app)
        })
        .await;

        match result {
            Ok(Ok(())) => { /* eval:done already emitted */ }
            Ok(Err(e)) => {
                handle_failure(&app_for_error, &session_for_error, &e);
            }
            Err(join_err) => {
                let err = AppError::InvalidState(format!(
                    "evaluation task panicked: {join_err}"
                ));
                handle_failure(&app_for_error, &session_for_error, &err);
            }
        }
    });
}

/// Late-arrival fetch of a persisted [`EvaluationResult`]. Returns `Ok(None)`
/// for an unknown `session_id`.
pub fn get_evaluation_result_impl(
    conn: &Connection,
    session_id: &str,
) -> Result<Option<EvaluationResult>, AppError> {
    let inner = conn.as_inner();
    let row = inner
        .query_row(
            "SELECT session_id, started_at, ended_at, duration_seconds, \
                    phoneme_attempts_json, difficulty_level, difficulty_thresholds_json, \
                    threshold_table_version, reattempt_counts_json, model_version \
             FROM sessions WHERE session_id = ?1",
            params![session_id],
            |row| {
                Ok(PersistedRow {
                    session_id: row.get::<_, String>(0)?,
                    started_at: row.get::<_, String>(1)?,
                    ended_at: row.get::<_, String>(2)?,
                    duration_seconds: row.get::<_, i64>(3)?,
                    phoneme_attempts_json: row.get::<_, String>(4)?,
                    difficulty_level: row.get::<_, String>(5)?,
                    difficulty_thresholds_json: row.get::<_, String>(6)?,
                    threshold_table_version: row.get::<_, i64>(7)?,
                    reattempt_counts_json: row.get::<_, String>(8)?,
                    model_version: row.get::<_, String>(9)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                // sentinel — translated to None below
                e
            }
            other => other,
        });

    match row {
        Ok(persisted) => Ok(Some(rehydrate_evaluation_result(persisted)?)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Storage(crate::storage::StorageError::from(e))),
    }
}

// ---------------------------------------------------------------------------
// Persistent row decode + rehydration
// ---------------------------------------------------------------------------

struct PersistedRow {
    session_id: String,
    started_at: String,
    ended_at: String,
    duration_seconds: i64,
    phoneme_attempts_json: String,
    difficulty_level: String,
    difficulty_thresholds_json: String,
    threshold_table_version: i64,
    reattempt_counts_json: String,
    model_version: String,
}

fn rehydrate_evaluation_result(row: PersistedRow) -> Result<EvaluationResult, AppError> {
    let phoneme_attempts = serde_json::from_str(&row.phoneme_attempts_json)
        .map_err(|e| AppError::InvalidState(format!("phoneme_attempts_json decode failed: {e}")))?;
    let difficulty_level = parse_difficulty_level(&row.difficulty_level)?;
    let difficulty_thresholds: PhonemeThresholds =
        serde_json::from_str(&row.difficulty_thresholds_json).map_err(|e| {
            AppError::InvalidState(format!("difficulty_thresholds_json decode failed: {e}"))
        })?;
    let reattempt_counts_by_sentence: Vec<u32> = serde_json::from_str(&row.reattempt_counts_json)
        .map_err(|e| AppError::InvalidState(format!("reattempt_counts_json decode failed: {e}")))?;

    Ok(EvaluationResult {
        session_id: SessionId(row.session_id),
        started_at: row.started_at,
        ended_at: row.ended_at,
        duration_seconds: row.duration_seconds as f64,
        phoneme_attempts,
        difficulty_level,
        difficulty_thresholds,
        threshold_table_version: row.threshold_table_version as i32,
        reattempt_counts_by_sentence,
        // Both `flagged_phonemes_ordered` and `highest_error_phoneme` are
        // returned empty/None on rehydration. The V1 late-arrival caller (a
        // React Results screen that recovers after a page reload) recomputes
        // them client-side from `phoneme_attempts` + `difficulty_thresholds`.
        // Post-V1, either persist the derived columns or call a shared
        // rerank helper here (see `build_flagged_ordered` in chunk_orchestrator).
        flagged_phonemes_ordered: Vec::new(),
        highest_error_phoneme: None,
        model_version: row.model_version,
    })
}

fn parse_difficulty_level(s: &str) -> Result<DifficultyLevel, AppError> {
    match s {
        "gentle" => Ok(DifficultyLevel::Gentle),
        "standard" => Ok(DifficultyLevel::Standard),
        "strict" => Ok(DifficultyLevel::Strict),
        other => Err(AppError::InvalidState(format!(
            "sessions.difficulty_level has unknown value {other:?}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// run_inner: blocking pipeline
// ---------------------------------------------------------------------------

fn run_inner_blocking(
    session_id: SessionId,
    audio: AudioBuffer,
    app: AppHandle,
) -> Result<(), AppError> {
    // 1. Resolve bundled resources.
    let threshold_path = resolve_resource(&app, BUNDLED_THRESHOLD_TABLE_REL)?;
    let vocab_path = resolve_resource(&app, BUNDLED_VOCAB_REL)?;
    let passage_ref_path = resolve_resource(&app, BUNDLED_PASSAGE_REF_REL)?;

    let threshold_table = ThresholdTable::load(&threshold_path)
        .map_err(|e| AppError::Inference(e))?;
    let vocab = load_vocab(&vocab_path)?;
    let allophones = crate::evaluation::load_bundled_allophones(&vocab)
        .map_err(|e| AppError::Inference(EvaluationError::AllophoneMapLoad(e.to_string())))?;
    let reference = load_expected_phonemes(&passage_ref_path)
        .map_err(|e| AppError::Inference(e))?;
    let passage = load_passage(&passage_ref_path)
        .map_err(|e| AppError::Inference(e))?;

    // Short-read guard. wav2vec2 produces 50 frames/sec at 16 kHz (one frame
    // per 320 input samples). The forced alignment in `align_to_reference`
    // requires T ≥ K — if the learner stops the recording before the
    // wall-clock duration buys at least K frames, the alignment fails as
    // `AlignmentInfeasible` and surfaces as a generic `scoring_failed` to the
    // UI. Bail early with the typed `audio_too_short` kind so Results can
    // render an actionable banner. The 1.2× headroom matches the
    // chunk-sizing math below — Viterbi's blank-augmented path inserts
    // intermediate blanks between labels, so a comfortable margin avoids
    // tripping the guard on a borderline-length read.
    let frame_samples = 320_usize; // 16_000 Hz / 50 frames-per-sec
    let min_samples = (reference.len() as f64 * 1.2 * frame_samples as f64).ceil() as usize;
    if audio.samples().len() < min_samples {
        return Err(AppError::Inference(
            EvaluationError::UnsupportedInputShape {
                got_len: audio.samples().len(),
                min_len: min_samples,
            },
        ));
    }

    // 2. Read settings from the DB.
    let settings = {
        let state: tauri::State<'_, AppState> = app.state();
        let conn = state
            .db
            .lock()
            .map_err(|_| AppError::InvalidState("db lock poisoned".into()))?;
        get_settings_impl(&conn)?
    };

    let thresholds = threshold_table.resolve_for_level(settings.difficulty.clone());
    let threshold_table_version = threshold_table.version();
    let difficulty_level = settings.difficulty.clone();
    let sentence_count = passage_sentence_count(&passage);

    // 3. Load the ONNX phonemizer. The DLL must already be initialised — see
    //    `crate::run()` for the `ort::init_from` call site. If init was
    //    skipped or failed, the load below will surface as RuntimeFailure.
    let model_path = resolve_model_path(&app)?;
    let phonemizer = OnnxPhonemizer::load(&model_path)
        .map_err(|e| AppError::Inference(e))?;

    // Now we have everything; signal the UI that the pipeline is about to
    // start the chunk loop. Per the task spec, stage transitions emit their
    // own progress events; we use the "stopping" stage name to align with
    // the spec's terminology (the recording has stopped; evaluation starts).
    emit_progress(
        &app,
        &session_id,
        "stopping",
        0.0,
        None,
    );

    // 4. Build evaluation context + run chunked evaluation.
    let started_at = chrono::Utc::now().to_rfc3339();
    let ended_at = started_at.clone();

    let context = build_evaluation_context(
        &session_id,
        audio.samples(),
        &started_at,
        &ended_at,
        &reference,
        &vocab,
        &thresholds,
        &allophones,
        difficulty_level.clone(),
        threshold_table_version,
        sentence_count,
    );

    // Chunk the phonemizer (the O(T²) attention cost is here), concatenate
    // posteriors, then run alignment ONCE over the full posterior buffer.
    // SPIKE-15 measured alignment + per-phoneme certainty as sub-ms regardless
    // of T, so single-pass alignment over the full buffer is the right shape
    // for both correctness (no per-chunk K > T_chunk Viterbi failures) and
    // memory (per-call working set bounded by the 10 s chunk).
    let evaluation_result = run_chunked_phonemizer_then_align(
        &context,
        &phonemizer as &dyn Phonemizer,
        &app,
        &session_id,
    )
    .map_err(|e| AppError::Inference(e))?;

    // 5. Generate feedback from the flagged-phoneme map. CL-18's
    //    generate_feedback consumes a HashMap<String, u32>; we feed it the
    //    flagged_count per symbol from the result.
    emit_progress(&app, &session_id, "building_feedback", 1.0, None);

    let flagged_map: std::collections::HashMap<String, u32> = evaluation_result
        .phoneme_attempts
        .0
        .iter()
        .filter(|(_, roll)| roll.flagged > 0)
        .map(|(sym, roll)| (sym.clone(), roll.flagged))
        .collect();
    let feedback = generate_feedback(&flagged_map);

    // 6. Persist.
    emit_progress(&app, &session_id, "persisting", 1.0, None);
    {
        let state: tauri::State<'_, AppState> = app.state();
        let mut conn = state
            .db
            .lock()
            .map_err(|_| AppError::InvalidState("db lock poisoned".into()))?;
        persist_evaluation_result(&mut conn, &evaluation_result, &settings)?;
    }

    // 7. Transition lifecycle Evaluating → Reviewing.
    {
        let state: tauri::State<'_, AppState> = app.state();
        let mut lifecycle = state
            .lifecycle
            .lock()
            .map_err(|_| AppError::InvalidState("lifecycle lock poisoned".into()))?;
        // The state machine has its own state-check; if a `cancel` raced us
        // it'll reject the transition. We surface that as a logged warning
        // rather than promoting to a hard error: by the time mark_evaluation_complete
        // fails, eval:done is the next thing we emit, and the UI can recover.
        if let Err(e) = lifecycle.mark_evaluation_complete() {
            log::warn!(
                "mark_evaluation_complete refused (session {:?}): {e}",
                evaluation_result.session_id
            );
        }
    }

    // 8. Emit eval:done.
    let payload = build_done_payload(evaluation_result, feedback);
    let _ = app.emit("eval:done", &payload);

    Ok(())
}

// ---------------------------------------------------------------------------
// Chunked-phonemizer / single-pass-align flow
// ---------------------------------------------------------------------------

/// Run the phonemizer in non-overlapping 10 s chunks, concatenate the
/// per-chunk posteriors, then run forced alignment ONCE over the full
/// posterior buffer. Thin wrapper that closes over the Tauri [`AppHandle`]
/// so per-chunk progress fires as `eval:progress` events.
///
/// Why this shape: SPIKE-15 measured wav2vec2's attention as O(T²) and
/// projected ~4.4 GB working-set on a 90 s clip — beyond the NF-MEM-1 budget
/// for 8 GB pilot laptops. Splitting the forward passes keeps each call's
/// working set bounded by the 10 s chunk. The alignment + certainty stage
/// stays sub-millisecond regardless of T (also per SPIKE-15), so single-pass
/// over the concatenated posteriors is both feasible and correct: feeding the
/// FULL reference into a per-chunk alignment would fail Viterbi feasibility
/// whenever T_chunk < K, and a chunk-local sub-reference is the wrong
/// projection (boundaries cut mid-word).
fn run_chunked_phonemizer_then_align(
    context: &EvaluationContext,
    phonemizer: &dyn Phonemizer,
    app: &AppHandle,
    session_id: &SessionId,
) -> Result<EvaluationResult, EvaluationError> {
    let session_id_for_cb = session_id.clone();
    let app_for_cb = app.clone();
    let on_progress = move |pct: f32| {
        // Per-chunk emit fires AFTER the phonemizer's forward pass but BEFORE
        // alignment, so there is no progressive partial yet. `None` signals
        // "no progressive result available" rather than "progressive result
        // is empty" — the distinction matters when `evaluate_chunked` (still
        // in tree) starts populating real partials on the progressive-UI path.
        emit_progress(
            &app_for_cb,
            &session_id_for_cb,
            "chunk",
            pct,
            None,
        );
    };
    run_chunked_phonemizer_then_align_inner(context, phonemizer, on_progress)
}

/// Test seam for [`run_chunked_phonemizer_then_align`]: the same flow, but
/// takes an `FnMut(f32)` progress sink instead of emitting Tauri events.
/// Production wraps this with an emitter closure; tests record into a `Vec`.
fn run_chunked_phonemizer_then_align_inner<F: FnMut(f32)>(
    context: &EvaluationContext,
    phonemizer: &dyn Phonemizer,
    mut on_progress: F,
) -> Result<EvaluationResult, EvaluationError> {
    let params = ChunkParams {
        chunk_seconds: CHUNK_SECONDS,
        overlap_seconds: 0.0,
    };

    let total_samples = context.audio.len();
    let duration_seconds = total_samples as f64 / chunk_orchestrator::SAMPLE_RATE_HZ;
    let windows = chunk_orchestrator::chunk_windows(total_samples, &params);

    // Empty audio: match `evaluate_chunked`'s empty-audio contract — zero
    // attempts, no progress events.
    if windows.is_empty() {
        let pool: std::collections::HashMap<(usize, usize), PooledOccurrence> =
            std::collections::HashMap::new();
        return Ok(chunk_orchestrator::build_final_result(
            &pool,
            context,
            duration_seconds,
            phonemizer.model_version(),
        ));
    }

    let total_chunks = windows.len();
    let mut all_posteriors: Vec<Vec<f32>> = Vec::new();

    for (chunk_index, window) in windows.iter().enumerate() {
        let chunk_audio = &context.audio[window.start_sample..window.end_sample];
        let logits = phonemizer.forward(chunk_audio)?;
        let decoded = ctc_greedy_decode(&logits, context.vocab, chunk_orchestrator::BLANK_IDX)?;
        all_posteriors.extend(decoded.posteriors);

        // Emit per-chunk progress after the forward pass completes. The
        // partial-result payload is empty (alignment has not yet run).
        let pct = (chunk_index + 1) as f32 / total_chunks as f32;
        on_progress(pct);
    }

    // Single-pass alignment over the concatenated posterior buffer.
    let alignment = align_to_reference(
        &all_posteriors,
        context.reference,
        context.vocab,
        context.thresholds,
        context.allophones,
    )?;

    // Build pool from the alignment's per-symbol occurrences. Each
    // `(word_index, position_in_word)` key is unique in a single-pass
    // alignment, so `merge_occurrence` here degenerates to a plain insert;
    // reusing the helper keeps the pool shape consistent with the
    // `evaluate_chunked` flow.
    let mut pool: std::collections::HashMap<(usize, usize), PooledOccurrence> =
        std::collections::HashMap::new();
    for (symbol, summary) in alignment.per_symbol.iter() {
        for occ in &summary.occurrences {
            merge_occurrence(&mut pool, symbol, occ);
        }
    }

    Ok(chunk_orchestrator::build_final_result(
        &pool,
        context,
        duration_seconds,
        phonemizer.model_version(),
    ))
}

// ---------------------------------------------------------------------------
// Failure handling
// ---------------------------------------------------------------------------

fn handle_failure(app: &AppHandle, session_id: &SessionId, err: &AppError) {
    let kind = error_kind(err);
    let message = err.to_string();
    log::warn!(
        "evaluation failed for session {:?}: kind={kind} message={message}",
        session_id
    );
    let payload = ErrorPayload {
        session_id: session_id.clone(),
        kind: kind.to_string(),
        message,
    };
    let _ = app.emit("eval:error", &payload);

    // Best-effort transition: Evaluating → Error. Use the cpal-tied
    // DeviceDisconnected stand-in because mark_error wants a MicrophoneError.
    // This is a known impedance mismatch between the recording state machine
    // and the evaluation feature; refactoring the state machine to accept a
    // generic error is out of scope for CL-19 (see project_log.md follow-up).
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut lifecycle) = state.lifecycle.lock() {
            if matches!(
                lifecycle.state(),
                crate::recording::SessionState::Evaluating
                    | crate::recording::SessionState::Recording
                    | crate::recording::SessionState::Paused
            ) {
                let _ = lifecycle.mark_error(MicrophoneError::DeviceDisconnected);
            }
        }
    }
}

/// Map an `AppError` to a wire-friendly kind discriminator. Mirrors the
/// `error_kind` exposed via the `AppError::kind()` method but narrows the
/// inference variants so the UI can route specific causes (e.g. model load
/// vs runtime failure vs persistence failure).
fn error_kind(err: &AppError) -> &'static str {
    match err {
        AppError::Inference(e) => match e {
            EvaluationError::ModelNotFound(_) | EvaluationError::ModelDigestMismatch { .. } => {
                "model_load_failed"
            }
            EvaluationError::ThresholdTableLoad(_) => "threshold_load_failed",
            EvaluationError::AllophoneMapLoad(_) => "allophone_load_failed",
            EvaluationError::BundledReferenceLoad(_)
            | EvaluationError::BundledReferenceInventoryMismatch { .. } => "reference_load_failed",
            EvaluationError::ReferenceSymbolNotInVocab { .. } => "vocab_mismatch",
            EvaluationError::AlignmentInfeasible { .. }
            | EvaluationError::ChunkInfeasible { .. }
            | EvaluationError::CtcShapeMismatch { .. }
            | EvaluationError::BlankIndexOutOfRange { .. } => "scoring_failed",
            EvaluationError::UnsupportedInputShape { .. } => "audio_too_short",
            EvaluationError::RuntimeFailure(_) => "inference_runtime",
        },
        AppError::Storage(_) => "persistence_failed",
        AppError::InvalidState(_) => "invalid_state",
        AppError::Config(_) => "config",
        _ => "evaluation_failed",
    }
}

// ---------------------------------------------------------------------------
// Helpers — extracted so they're unit-testable without a Tauri runtime
// ---------------------------------------------------------------------------

/// Pure helper to build the [`EvaluationContext`] from the orchestrator's
/// inputs. Exists so the wiring can be exercised without a Tauri runtime.
#[allow(clippy::too_many_arguments)]
pub(crate) fn build_evaluation_context<'a>(
    session_id: &SessionId,
    audio: &'a [f32],
    started_at: &str,
    ended_at: &str,
    reference: &'a [ExpectedPhoneme],
    vocab: &'a [String],
    thresholds: &'a PhonemeThresholds,
    allophones: &'a crate::evaluation::AllophoneMap,
    difficulty_level: DifficultyLevel,
    threshold_table_version: i32,
    sentence_count: usize,
) -> EvaluationContext<'a> {
    EvaluationContext {
        audio,
        session_id: session_id.clone(),
        started_at: started_at.to_string(),
        ended_at: ended_at.to_string(),
        reference,
        vocab,
        thresholds,
        allophones,
        difficulty_level,
        threshold_table_version,
        sentence_count,
    }
}

/// Build the `eval:done` wire payload.
pub(crate) fn build_done_payload(
    result: EvaluationResult,
    feedback: Vec<FeedbackEntry>,
) -> DonePayload {
    DonePayload { result, feedback }
}

/// Persist an [`EvaluationResult`] to the `sessions` table. The orchestrator
/// calls this once per cycle, immediately before emitting `eval:done`.
///
/// Failure modes: any SQLite error bubbles via [`AppError::Storage`].
pub(crate) fn persist_evaluation_result(
    conn: &mut Connection,
    result: &EvaluationResult,
    settings: &Settings,
) -> Result<(), AppError> {
    let inner = conn.as_inner_mut();

    // cumulative_session_count = current row count + 1 (the row we're about
    // to insert). Computed before the INSERT so the new row carries its own
    // serial number.
    let prior_count: i64 = inner
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .map_err(|e| AppError::Storage(crate::storage::StorageError::from(e)))?;
    let cumulative_session_count = prior_count + 1;

    let phoneme_attempts_json = serde_json::to_string(&result.phoneme_attempts)
        .map_err(|e| AppError::InvalidState(format!("phoneme_attempts encode failed: {e}")))?;
    let difficulty_thresholds_json = serde_json::to_string(&result.difficulty_thresholds)
        .map_err(|e| {
            AppError::InvalidState(format!("difficulty_thresholds encode failed: {e}"))
        })?;
    let reattempt_counts_json = serde_json::to_string(&result.reattempt_counts_by_sentence)
        .map_err(|e| AppError::InvalidState(format!("reattempt_counts encode failed: {e}")))?;

    let difficulty_level_str = match result.difficulty_level {
        DifficultyLevel::Gentle => "gentle",
        DifficultyLevel::Standard => "standard",
        DifficultyLevel::Strict => "strict",
    };

    // duration_seconds is INTEGER in the schema; round to nearest.
    let duration_int = result.duration_seconds.round() as i64;

    inner
        .execute(
            "INSERT INTO sessions (
                session_id, started_at, ended_at, duration_seconds,
                l1_at_session, regional_variety_at_session, phoneme_attempts_json,
                difficulty_level, difficulty_thresholds_json, threshold_table_version,
                reattempt_counts_json, cumulative_session_count,
                app_version, model_version, os_family, os_major
             ) VALUES (
                ?1, ?2, ?3, ?4,
                ?5, ?6, ?7,
                ?8, ?9, ?10,
                ?11, ?12,
                ?13, ?14, ?15, ?16
             )",
            params![
                result.session_id.0,
                result.started_at,
                result.ended_at,
                duration_int,
                settings.l1,
                settings.regional_variety,
                phoneme_attempts_json,
                difficulty_level_str,
                difficulty_thresholds_json,
                result.threshold_table_version as i64,
                reattempt_counts_json,
                cumulative_session_count,
                BuildConfig::APP_VERSION,
                result.model_version,
                std::env::consts::OS,
                // TODO(post-v1): refine os_major via sysinfo or tauri::api (CL-23).
                OS_MAJOR_PLACEHOLDER,
            ],
        )
        .map_err(|e| AppError::Storage(crate::storage::StorageError::from(e)))?;

    Ok(())
}

/// Resolve a bundled resource via Tauri's `BaseDirectory::Resource` lookup,
/// surfacing failures as [`EvaluationError::BundledReferenceLoad`].
fn resolve_resource(app: &AppHandle, rel: &str) -> Result<PathBuf, AppError> {
    app.path()
        .resolve(rel, BaseDirectory::Resource)
        .map_err(|e| {
            AppError::Inference(EvaluationError::BundledReferenceLoad(format!(
                "could not resolve bundled resource {rel}: {e}"
            )))
        })
}

/// Resolve the model file under the runtime model cache directory. The
/// filename is derived from `MODEL_URL` (see [`BuildConfig::model_filename`]),
/// so the path the CL-24 download writes to and the path the orchestrator
/// loads from are the same by construction. `pub(crate)` so
/// [`crate::evaluation::model_download`] resolves the identical destination.
pub(crate) fn resolve_model_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let cfg = crate::shared::config::RuntimeConfig::from_app_handle(app)?;
    Ok(cfg.model_cache_dir.join(crate::shared::config::BuildConfig::model_filename()))
}

/// On-disk shape of the bundled vocab JSON. Mirrors the script that produces
/// it from `tools/model_export/build/manifest.json::ipa_vocabulary`.
#[derive(Debug, Deserialize)]
struct OnDiskVocab {
    #[serde(default)]
    #[allow(dead_code)]
    version: i32,
    vocabulary: Vec<String>,
}

/// Load the model vocabulary JSON from `path`. Surfaces parse / read errors
/// as [`EvaluationError::BundledReferenceLoad`] — the vocab is a sibling
/// build-time artifact, so reusing the variant keeps the error surface tight.
pub(crate) fn load_vocab(path: &std::path::Path) -> Result<Vec<String>, AppError> {
    let bytes = std::fs::read(path).map_err(|e| {
        AppError::Inference(EvaluationError::BundledReferenceLoad(format!(
            "could not read vocab {}: {}",
            path.display(),
            e
        )))
    })?;
    let parsed: OnDiskVocab = serde_json::from_slice(&bytes).map_err(|e| {
        AppError::Inference(EvaluationError::BundledReferenceLoad(format!(
            "could not parse vocab {}: {}",
            path.display(),
            e
        )))
    })?;
    if parsed.vocabulary.is_empty() {
        return Err(AppError::Inference(EvaluationError::BundledReferenceLoad(
            format!("vocab {} is empty", path.display()),
        )));
    }
    Ok(parsed.vocabulary)
}

/// Best-effort sentence count for `passage`. We split on `.`, `?`, `!`
/// boundaries and filter empties; the value is only used to size the
/// `reattempt_counts_by_sentence` placeholder vec (V1 ships zeros — single
/// attempt path).
fn passage_sentence_count(passage: &Passage) -> usize {
    passage
        .text
        .split(|c: char| matches!(c, '.' | '?' | '!'))
        .filter(|s| !s.trim().is_empty())
        .count()
        .max(1)
}

/// Best-effort `eval:progress` emit. Failure (e.g. webview closed) is
/// swallowed — progress events are advisory.
fn emit_progress(
    app: &AppHandle,
    session_id: &SessionId,
    stage: &str,
    pct: f32,
    partial: Option<WireChunkPartial>,
) {
    let payload = ProgressPayload {
        session_id: session_id.clone(),
        stage: stage.to_string(),
        pct,
        partial_result: partial,
    };
    let _ = app.emit("eval:progress", &payload);
}

// ---------------------------------------------------------------------------
// Tests
//
// The orchestrator's wiring is split into testable helpers so we can exercise
// the load/persist/payload paths without a Tauri runtime:
//
// - `persist_evaluation_result` against an in-memory SQLite (mirrors the
//   pattern in `storage/connection.rs::tests`).
// - `build_evaluation_context` for the input-projection layer.
// - `build_done_payload` for the wire-shape assembly.
// - `error_kind` for the error-discriminator mapping.
// - `load_vocab` for the bundled-JSON loader, including the empty-vocab gate.
//
// Full end-to-end coverage (the AppHandle + tokio task path) is the
// `#[ignore]`-marked integration test below.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evaluation::allophones::AllophoneMap;
    use crate::evaluation::chunk_orchestrator::{evaluate_chunked, ChunkProgress};
    use crate::evaluation::phonemizer::{Logits, MockPhonemizer};
    use crate::shared::types::{
        AttemptRollup, DifficultyLevel, FlaggedPhoneme, PhonemeAttempts, PhonemeThresholds,
    };
    use std::collections::HashMap;
    use tempfile::TempDir;

    /// Build a minimal `AllophoneMap` for orchestrator tests. Every V1 target
    /// is mapped to itself (single-element allophone sets). The vocab is
    /// extended to include any V1 target symbols not already present.
    fn minimal_allophone_map_orch(vocab: &[String]) -> AllophoneMap {
        let all_v1_targets = crate::evaluation::thresholds::V1_TARGET_PHONEMES;
        let mut ext_vocab: Vec<String> = vocab.to_vec();
        for &t in all_v1_targets {
            if !ext_vocab.iter().any(|s| s == t) {
                ext_vocab.push(t.to_string());
            }
        }
        let pairs: String = all_v1_targets
            .iter()
            .map(|t| format!("{:?}: [{:?}]", t, t))
            .collect::<Vec<_>>()
            .join(", ");
        let json = format!(
            r#"{{
                "_header": {{
                    "schema_version": 1,
                    "source_spike": "test",
                    "source_file": "test",
                    "notes": "minimal allophone map for orchestrator tests"
                }},
                "allophones": {{ {pairs} }}
            }}"#
        );
        AllophoneMap::load(&json, &ext_vocab)
            .expect("minimal orchestrator allophone map must load")
    }

    fn open_tmp_conn() -> (TempDir, Connection) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("ias.db");
        let conn = Connection::new(&path).unwrap();
        (dir, conn)
    }

    fn sample_settings() -> Settings {
        Settings {
            l1: "spa".to_string(),
            regional_variety: Some("Caribbean".to_string()),
            difficulty: DifficultyLevel::Standard,
            report_uploads_enabled: true,
            update_checks_enabled: false,
        }
    }

    fn sample_evaluation_result(session_id: &str) -> EvaluationResult {
        let mut attempts = HashMap::new();
        attempts.insert(
            "θ".to_string(),
            AttemptRollup {
                occurrences: 4,
                flagged: 2,
                mean_certainty: Some(0.55),
            },
        );
        attempts.insert(
            "ɹ".to_string(),
            AttemptRollup {
                occurrences: 6,
                flagged: 0,
                mean_certainty: Some(0.91),
            },
        );

        let mut thresholds_map = HashMap::new();
        thresholds_map.insert("θ".to_string(), 0.70);
        thresholds_map.insert("ɹ".to_string(), 0.65);

        EvaluationResult {
            session_id: SessionId(session_id.to_string()),
            started_at: "2026-06-01T12:00:00Z".to_string(),
            ended_at: "2026-06-01T12:01:00Z".to_string(),
            duration_seconds: 60.0,
            phoneme_attempts: PhonemeAttempts(attempts),
            difficulty_level: DifficultyLevel::Standard,
            difficulty_thresholds: PhonemeThresholds(thresholds_map),
            threshold_table_version: 1,
            reattempt_counts_by_sentence: vec![0u32, 0u32, 0u32],
            flagged_phonemes_ordered: vec![FlaggedPhoneme {
                phoneme: "θ".to_string(),
                example_word: "think".to_string(),
                flag_count: 2,
                mean_certainty: 0.55,
            }],
            highest_error_phoneme: Some("θ".to_string()),
            model_version: "mock-0.0.0".to_string(),
        }
    }

    // ---- persistence -------------------------------------------------------

    #[test]
    fn persist_evaluation_result_inserts_row_with_v04_fields() {
        let (_dir, mut conn) = open_tmp_conn();
        let result = sample_evaluation_result("sess-A");
        let settings = sample_settings();

        persist_evaluation_result(&mut conn, &result, &settings).expect("persist");

        let inner = conn.as_inner();
        let (level, thresholds_json, version): (String, String, i64) = inner
            .query_row(
                "SELECT difficulty_level, difficulty_thresholds_json, threshold_table_version \
                 FROM sessions WHERE session_id = ?1",
                params!["sess-A"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .expect("row exists");
        assert_eq!(level, "standard");
        assert_eq!(version, 1);
        // thresholds_json round-trips into the IPC type.
        let _: PhonemeThresholds = serde_json::from_str(&thresholds_json).expect("decodable");

        // cumulative_session_count is the first row → 1.
        let cum: i64 = inner
            .query_row(
                "SELECT cumulative_session_count FROM sessions WHERE session_id = ?1",
                params!["sess-A"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(cum, 1);

        // duration_seconds was rounded into the INTEGER column.
        let duration: i64 = inner
            .query_row(
                "SELECT duration_seconds FROM sessions WHERE session_id = ?1",
                params!["sess-A"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(duration, 60);

        // model_version and l1_at_session round-trip from inputs.
        let (model_version, l1, regional_variety): (String, String, Option<String>) = inner
            .query_row(
                "SELECT model_version, l1_at_session, regional_variety_at_session \
                 FROM sessions WHERE session_id = ?1",
                params!["sess-A"],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(model_version, "mock-0.0.0");
        assert_eq!(l1, "spa");
        assert_eq!(regional_variety.as_deref(), Some("Caribbean"));

        // os_family is the build target's `consts::OS` and os_major is the
        // V1 placeholder.
        let (os_family, os_major): (String, String) = inner
            .query_row(
                "SELECT os_family, os_major FROM sessions WHERE session_id = ?1",
                params!["sess-A"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(os_family, std::env::consts::OS);
        assert_eq!(os_major, OS_MAJOR_PLACEHOLDER);
    }

    #[test]
    fn persist_two_sessions_advances_cumulative_count() {
        let (_dir, mut conn) = open_tmp_conn();
        let settings = sample_settings();

        let mut r1 = sample_evaluation_result("sess-1");
        r1.started_at = "2026-06-01T12:00:00Z".into();
        let mut r2 = sample_evaluation_result("sess-2");
        r2.started_at = "2026-06-01T12:05:00Z".into();

        persist_evaluation_result(&mut conn, &r1, &settings).expect("first persist");
        persist_evaluation_result(&mut conn, &r2, &settings).expect("second persist");

        let counts: Vec<i64> = {
            let inner = conn.as_inner();
            let mut stmt = inner
                .prepare(
                    "SELECT cumulative_session_count FROM sessions \
                     ORDER BY started_at ASC",
                )
                .unwrap();
            stmt.query_map([], |row| row.get::<_, i64>(0))
                .unwrap()
                .map(|r| r.unwrap())
                .collect()
        };
        assert_eq!(counts, vec![1, 2]);
    }

    #[test]
    fn persist_does_not_enqueue_upload_row() {
        // The task spec is explicit: NO upload at eval:done. CL-22 handles
        // the queue insert on Results-leave. This test guards against a
        // future refactor accidentally re-introducing the write here.
        let (_dir, mut conn) = open_tmp_conn();
        let result = sample_evaluation_result("sess-noupload");
        let settings = sample_settings();
        persist_evaluation_result(&mut conn, &result, &settings).expect("persist");

        let q_count: i64 = conn
            .as_inner()
            .query_row("SELECT COUNT(*) FROM upload_queue", [], |row| row.get(0))
            .unwrap();
        assert_eq!(
            q_count, 0,
            "CL-19 must NOT enqueue an upload_queue row; that's CL-22's job"
        );
    }

    // ---- late-arrival query ------------------------------------------------

    #[test]
    fn get_evaluation_result_for_missing_session_returns_none() {
        let (_dir, conn) = open_tmp_conn();
        let got = get_evaluation_result_impl(&conn, "nope").expect("query ok");
        assert!(got.is_none());
    }

    #[test]
    fn get_evaluation_result_returns_persisted_row() {
        let (_dir, mut conn) = open_tmp_conn();
        let result = sample_evaluation_result("sess-roundtrip");
        let settings = sample_settings();
        persist_evaluation_result(&mut conn, &result, &settings).expect("persist");

        let got = get_evaluation_result_impl(&conn, "sess-roundtrip")
            .expect("query ok")
            .expect("row exists");

        assert_eq!(got.session_id, result.session_id);
        assert_eq!(got.difficulty_level, DifficultyLevel::Standard);
        assert_eq!(got.threshold_table_version, 1);
        assert_eq!(got.model_version, "mock-0.0.0");
        // duration rounds to integer column, so 60.0 → 60.
        assert!((got.duration_seconds - 60.0).abs() < f64::EPSILON);
        // phoneme_attempts round-trip identical to the persisted value.
        assert_eq!(got.phoneme_attempts, result.phoneme_attempts);
        // difficulty_thresholds round-trip identical.
        assert_eq!(got.difficulty_thresholds, result.difficulty_thresholds);
    }

    // ---- evaluation context helper ----------------------------------------

    #[test]
    fn build_evaluation_context_populates_fields() {
        let session_id = SessionId("ctx-test".to_string());
        let audio = vec![0.0_f32; 16_000];
        let reference: Vec<ExpectedPhoneme> = Vec::new();
        let vocab: Vec<String> = vec!["<blank>".into(), "θ".into()];
        let mut th_map = HashMap::new();
        th_map.insert("θ".to_string(), 0.5);
        let thresholds = PhonemeThresholds(th_map);

        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:01:00Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            7,
            3,
        );

        assert_eq!(ctx.session_id, session_id);
        assert_eq!(ctx.audio.len(), 16_000);
        assert_eq!(ctx.started_at, "2026-06-01T12:00:00Z");
        assert_eq!(ctx.ended_at, "2026-06-01T12:01:00Z");
        assert_eq!(ctx.difficulty_level, DifficultyLevel::Standard);
        assert_eq!(ctx.threshold_table_version, 7);
        assert_eq!(ctx.sentence_count, 3);
    }

    // ---- payload assembly -------------------------------------------------

    #[test]
    fn build_done_payload_carries_v04_fields_and_feedback() {
        let result = sample_evaluation_result("sess-payload");
        let feedback = vec![FeedbackEntry {
            phoneme: "θ".into(),
            example_word: "think".into(),
            mouth_shape: "tip of tongue between teeth; no voicing".into(),
            minimal_pair: "thin / then".into(),
            flag_count: 2,
            learn_more_url: None,
        }];
        let payload = build_done_payload(result.clone(), feedback.clone());

        assert_eq!(payload.result.session_id, result.session_id);
        assert_eq!(payload.result.difficulty_level, result.difficulty_level);
        assert_eq!(
            payload.result.difficulty_thresholds, result.difficulty_thresholds,
        );
        assert_eq!(
            payload.result.threshold_table_version, result.threshold_table_version,
        );
        assert_eq!(payload.feedback.len(), 1);
        assert_eq!(payload.feedback[0].phoneme, "θ");

        // Wire-shape sanity: must serialize to a JSON object with both keys.
        let v: serde_json::Value = serde_json::to_value(&payload).unwrap();
        assert!(v["result"].is_object());
        assert!(v["feedback"].is_array());
        assert_eq!(v["result"]["difficulty_level"], serde_json::json!("standard"));
        assert_eq!(v["result"]["threshold_table_version"], serde_json::json!(1));
    }

    // ---- error-kind mapping -----------------------------------------------

    #[test]
    fn error_kind_maps_inference_subvariants() {
        let model_missing =
            AppError::Inference(EvaluationError::ModelNotFound("/missing".into()));
        assert_eq!(error_kind(&model_missing), "model_load_failed");

        let digest = AppError::Inference(EvaluationError::ModelDigestMismatch {
            expected: "a".into(),
            actual: "b".into(),
        });
        assert_eq!(error_kind(&digest), "model_load_failed");

        let threshold =
            AppError::Inference(EvaluationError::ThresholdTableLoad("bad".into()));
        assert_eq!(error_kind(&threshold), "threshold_load_failed");

        // AllophoneMapLoad must NOT collapse to threshold_load_failed — the
        // variant exists precisely so triage can tell a vocabulary-mismatch
        // from a threshold-schema problem.
        let allophones =
            AppError::Inference(EvaluationError::AllophoneMapLoad("missing symbol".into()));
        assert_eq!(error_kind(&allophones), "allophone_load_failed");

        let runtime = AppError::Inference(EvaluationError::RuntimeFailure("ort died".into()));
        assert_eq!(error_kind(&runtime), "inference_runtime");

        let bad_chunk =
            AppError::Inference(EvaluationError::ChunkInfeasible { detail: "bad".into() });
        assert_eq!(error_kind(&bad_chunk), "scoring_failed");

        let bad_input = AppError::Inference(EvaluationError::UnsupportedInputShape {
            got_len: 0,
            min_len: 1_600,
        });
        assert_eq!(error_kind(&bad_input), "audio_too_short");
    }

    #[test]
    fn error_kind_maps_storage_failure() {
        // Construct via the public constructor path used elsewhere.
        let err = AppError::Storage(crate::storage::StorageError::InvalidState("x".into()));
        assert_eq!(error_kind(&err), "persistence_failed");
    }

    // ---- vocab loader -----------------------------------------------------

    #[test]
    fn load_vocab_reads_committed_bundle() {
        // Use the committed bundle at app/src-tauri/resources/model_vocab_v1.json.
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources/model_vocab_v1.json");
        let vocab = load_vocab(&path).expect("bundled vocab must load");
        assert_eq!(
            vocab.len(),
            392,
            "MX manifest pins ipa_vocabulary at 392 symbols (wav2vec2-lv-60-espeak-cv-ft)"
        );
        assert_eq!(vocab[0], "<pad>", "blank/pad is index 0 by CTC convention");
    }

    #[test]
    fn load_vocab_rejects_empty_array() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("empty.json");
        std::fs::write(&path, r#"{"version":1,"vocabulary":[]}"#).expect("write");

        let err = load_vocab(&path).expect_err("must reject empty vocab");
        match err {
            AppError::Inference(EvaluationError::BundledReferenceLoad(msg)) => {
                assert!(
                    msg.contains("empty"),
                    "message should call out empty vocab, got: {msg}"
                );
            }
            other => panic!("expected BundledReferenceLoad, got {other:?}"),
        }
    }

    #[test]
    fn load_vocab_rejects_missing_file() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("nope.json");
        let err = load_vocab(&path).expect_err("must reject missing file");
        match err {
            AppError::Inference(EvaluationError::BundledReferenceLoad(msg)) => {
                assert!(msg.contains("could not read"), "got: {msg}");
            }
            other => panic!("expected BundledReferenceLoad, got {other:?}"),
        }
    }

    #[test]
    fn load_vocab_rejects_malformed_json() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("bad.json");
        std::fs::write(&path, b"{ not json").expect("write");
        let err = load_vocab(&path).expect_err("must reject malformed JSON");
        match err {
            AppError::Inference(EvaluationError::BundledReferenceLoad(msg)) => {
                assert!(msg.contains("could not parse"), "got: {msg}");
            }
            other => panic!("expected BundledReferenceLoad, got {other:?}"),
        }
    }

    // ---- progress / done wire-shape sanity --------------------------------

    #[test]
    fn progress_payload_serializes_with_snake_case_stage() {
        let payload = ProgressPayload {
            session_id: SessionId("sess-prog".to_string()),
            stage: "chunk".into(),
            pct: 0.5,
            partial_result: Some(WireChunkPartial {
                flagged_phonemes_so_far: vec![],
                mean_certainty_so_far: HashMap::new(),
            }),
        };
        let v = serde_json::to_value(&payload).unwrap();
        assert_eq!(v["session_id"], serde_json::json!("sess-prog"));
        assert_eq!(v["stage"], serde_json::json!("chunk"));
        assert_eq!(v["pct"], serde_json::json!(0.5));
        assert!(v["partial_result"].is_object());
    }

    #[test]
    fn error_payload_carries_kind_and_message() {
        let payload = ErrorPayload {
            session_id: SessionId("sess-err".to_string()),
            kind: "model_load_failed".into(),
            message: "file missing".into(),
        };
        let v = serde_json::to_value(&payload).unwrap();
        assert_eq!(v["session_id"], serde_json::json!("sess-err"));
        assert_eq!(v["kind"], serde_json::json!("model_load_failed"));
        assert_eq!(v["message"], serde_json::json!("file missing"));
    }

    // ---- mock-phonemizer integration: at-least-one progress before done ----
    //
    // The full orchestrator path can't run without an AppHandle (the spawned
    // task needs tauri::State + emit). Instead, we drive
    // `run_chunked_phonemizer_then_align_inner` directly with a recorded
    // callback — the same flow `run_chunked_phonemizer_then_align` uses, but
    // with an `FnMut(f32)` progress sink instead of a Tauri emitter.

    fn synthetic_logits_high_confidence(n_frames: usize, vocab_len: usize) -> Vec<Logits> {
        // Force decoder onto symbol-index-1 with logit 10; other vocab at 0.
        let mut out = Vec::with_capacity(n_frames);
        for _ in 0..n_frames {
            let mut row = vec![0.0_f32; vocab_len];
            row[1] = 10.0;
            out.push(row);
        }
        out
    }

    /// Logits that target an arbitrary vocab index per frame at high
    /// confidence (logit 10 on the chosen index, 0 elsewhere). The chunked
    /// flow softmaxes these per-frame inside `ctc_greedy_decode`.
    fn logit_frames_for_indices(
        per_frame_idx: &[usize],
        vocab_len: usize,
    ) -> Vec<Logits> {
        let mut out = Vec::with_capacity(per_frame_idx.len());
        for &idx in per_frame_idx {
            let mut row = vec![0.0_f32; vocab_len];
            row[idx] = 10.0;
            out.push(row);
        }
        out
    }

    /// Logits with a controllable per-frame target-index logit. Used to drive
    /// low-confidence posteriors (logit ~0.5 distributed over vocab) to test
    /// flagging behaviour without triggering the fallback path.
    fn logit_frames_low_confidence(
        per_frame_idx: &[usize],
        vocab_len: usize,
        target_logit: f32,
    ) -> Vec<Logits> {
        let mut out = Vec::with_capacity(per_frame_idx.len());
        for &idx in per_frame_idx {
            let mut row = vec![0.0_f32; vocab_len];
            row[idx] = target_logit;
            out.push(row);
        }
        out
    }

    fn ep(symbol: &str, is_target: bool, word_idx: usize, pos: usize) -> ExpectedPhoneme {
        ExpectedPhoneme {
            symbol: symbol.to_string(),
            is_target,
            word_index: word_idx,
            position_in_word: pos,
        }
    }

    /// A [`Phonemizer`] that returns different canned logits per call. Local
    /// declaration mirrors `chunk_orchestrator::tests::SequencedMockPhonemizer`
    /// so the orchestrator's chunked flow can be driven with distinct
    /// per-chunk posteriors.
    struct SequencedMockPhonemizer {
        rounds: std::sync::Mutex<std::collections::VecDeque<Vec<Logits>>>,
        vocab_size: usize,
        model_version: String,
    }

    impl SequencedMockPhonemizer {
        fn new(rounds: Vec<Vec<Logits>>, vocab_size: usize) -> Self {
            Self {
                rounds: std::sync::Mutex::new(rounds.into()),
                vocab_size,
                model_version: "mock-seq-orch-0.0.0".to_string(),
            }
        }
    }

    impl Phonemizer for SequencedMockPhonemizer {
        fn forward(&self, _audio: &[f32]) -> Result<Vec<Logits>, EvaluationError> {
            let mut q = self.rounds.lock().expect("rounds mutex poisoned");
            let next = q
                .pop_front()
                .expect("SequencedMockPhonemizer exhausted: forward called more than rounds");
            Ok(next)
        }
        fn vocab_size(&self) -> usize {
            self.vocab_size
        }
        fn model_version(&self) -> &str {
            &self.model_version
        }
    }

    /// A [`Phonemizer`] that returns canned logits for the first `succeed`
    /// chunks then errors. Used by `phonemizer_error_propagates`.
    struct ErrAfterPhonemizer {
        rounds: std::sync::Mutex<std::collections::VecDeque<Vec<Logits>>>,
        vocab_size: usize,
        model_version: String,
    }

    impl ErrAfterPhonemizer {
        fn new(rounds: Vec<Vec<Logits>>, vocab_size: usize) -> Self {
            Self {
                rounds: std::sync::Mutex::new(rounds.into()),
                vocab_size,
                model_version: "err-after-orch-0.0.0".to_string(),
            }
        }
    }

    impl Phonemizer for ErrAfterPhonemizer {
        fn forward(&self, _audio: &[f32]) -> Result<Vec<Logits>, EvaluationError> {
            let mut q = self.rounds.lock().expect("rounds mutex poisoned");
            match q.pop_front() {
                Some(next) => Ok(next),
                None => Err(EvaluationError::RuntimeFailure(
                    "synthetic forward failure on subsequent chunk".into(),
                )),
            }
        }
        fn vocab_size(&self) -> usize {
            self.vocab_size
        }
        fn model_version(&self) -> &str {
            &self.model_version
        }
    }

    #[test]
    fn mock_phonemizer_drives_progress_then_done_shape() {
        // 30s audio + default chunk params (10s, no overlap) → 3 chunks.
        // Mock phonemizer returns the same canned logits per chunk; the
        // chunked-then-aligned flow concatenates posteriors and runs
        // alignment once, producing the v0.4-shaped result.
        let vocab = vec!["<blank>".to_string(), "θ".to_string()];
        let reference = vec![ep("θ", true, 0, 0)];
        let mut th_map = HashMap::new();
        th_map.insert("θ".to_string(), 0.5);
        let thresholds = PhonemeThresholds(th_map);

        let phonemizer = MockPhonemizer::with_fixed(
            synthetic_logits_high_confidence(8, vocab.len()),
            vocab.len(),
        );
        let audio = vec![0.0_f32; 30 * 16_000];
        let session_id = SessionId("mock-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:30Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        let collected = std::sync::Arc::new(std::sync::Mutex::new(Vec::<f32>::new()));
        let collected_for_cb = std::sync::Arc::clone(&collected);
        let on_progress = move |pct: f32| {
            collected_for_cb.lock().unwrap().push(pct);
        };

        let result = run_chunked_phonemizer_then_align_inner(
            &ctx,
            &phonemizer as &dyn Phonemizer,
            on_progress,
        )
        .expect("evaluate ok");

        let events = collected.lock().unwrap();
        assert!(
            !events.is_empty(),
            "at least one chunk progress event must fire before done"
        );
        // pct ends at 1.0 within float epsilon.
        assert!((events.last().copied().unwrap() - 1.0).abs() < 1e-6);

        // v0.4 fields populated.
        assert_eq!(result.difficulty_level, DifficultyLevel::Standard);
        assert_eq!(result.threshold_table_version, 1);
        assert!(result.difficulty_thresholds.0.contains_key("θ"));

        // Done payload assembles cleanly.
        let payload = build_done_payload(result.clone(), Vec::new());
        let v: serde_json::Value = serde_json::to_value(&payload).unwrap();
        assert_eq!(v["result"]["difficulty_level"], serde_json::json!("standard"));
    }

    // ---- new tests for chunked-phonemizer / single-pass-align flow ---------

    #[test]
    fn chunked_phonemizer_aligns_each_chunk_to_corresponding_reference_position() {
        // Two-chunk read with distinct per-chunk canned logits. Reference is
        // [θ, ɹ]: the first chunk targets θ, the second chunk targets ɹ. If
        // the orchestrator concatenates posteriors in order, Viterbi will
        // align θ to chunk-1 frames and ɹ to chunk-2 frames — both spoken
        // with high confidence, so neither flags. This also indirectly proves
        // concatenation order: reverse the buffer and Viterbi can't satisfy
        // monotonic alignment of [θ, ɹ] over a [ɹ-frames, θ-frames] sequence,
        // forcing one (or both) labels onto low-certainty mismatched frames.
        let vocab = vec![
            "<blank>".to_string(),
            "θ".to_string(),
            "ɹ".to_string(),
        ];
        let reference = vec![
            ep("θ", true, 0, 0),
            ep("ɹ", true, 1, 0),
        ];
        let mut th_map = HashMap::new();
        th_map.insert("θ".to_string(), 0.5);
        th_map.insert("ɹ".to_string(), 0.5);
        let thresholds = PhonemeThresholds(th_map);

        // 20 s audio → exactly 2 chunks at 10 s default chunking.
        let audio = vec![0.0_f32; 20 * 16_000];
        let session_id = SessionId("concat-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:20Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        // Chunk 1: 8 frames on θ (vocab index 1). Chunk 2: 8 frames on ɹ
        // (vocab index 2). Concatenated buffer: 16 frames.
        let chunk1 = logit_frames_for_indices(&[1, 1, 1, 1, 1, 1, 1, 1], vocab.len());
        let chunk2 = logit_frames_for_indices(&[2, 2, 2, 2, 2, 2, 2, 2], vocab.len());
        let phonemizer = SequencedMockPhonemizer::new(vec![chunk1, chunk2], vocab.len());

        let result = run_chunked_phonemizer_then_align_inner(
            &ctx,
            &phonemizer as &dyn Phonemizer,
            |_pct| {},
        )
        .expect("evaluate ok");

        // Both reference labels should be present with non-zero certainty
        // (i.e. NOT from_fallback) and unflagged.
        let theta = result
            .phoneme_attempts
            .0
            .get("θ")
            .expect("θ attempts present");
        let rho = result
            .phoneme_attempts
            .0
            .get("ɹ")
            .expect("ɹ attempts present");
        assert_eq!(theta.occurrences, 1);
        assert_eq!(rho.occurrences, 1);
        assert_eq!(theta.flagged, 0);
        assert_eq!(rho.flagged, 0);
        let theta_cert = theta.mean_certainty.expect("θ mean certainty present");
        let rho_cert = rho.mean_certainty.expect("ɹ mean certainty present");
        assert!(theta_cert > 0.5, "θ should align high, got {theta_cert}");
        assert!(rho_cert > 0.5, "ɹ should align high, got {rho_cert}");
    }

    #[test]
    fn single_pass_alignment_covers_full_reference() {
        // Load-bearing property: chunking the phonemizer then aligning ONCE
        // over the concatenated posterior buffer remains feasible when the
        // per-chunk frame count is LESS than the reference label count. A
        // per-chunk alignment would fail Viterbi feasibility (K > T_chunk);
        // single-pass over the concatenated buffer succeeds because
        // T_concatenated ≥ K.
        //
        // Fixture: K = 600 distinct reference labels over 2 × 10 s chunks
        // (T_chunk = 500 frames each, T_concatenated = 1000). Chunk 1 covers
        // labels 0..299 (each frame strongly favouring its target vocab
        // index, padded with trailing blanks); chunk 2 covers labels
        // 300..599 the same way. Viterbi over the concatenated 1000-frame
        // posterior buffer must visit every one of the 600 labels.
        const K: usize = 600;
        const CHUNK_SAMPLES: usize = 10 * 16_000;
        const FRAMES_PER_CHUNK: usize = 500; // 10 s × 50 fps; matches default chunk_seconds

        // vocab: blank + sym_0 .. sym_{K-1}. Index of sym_i in vocab is i + 1.
        let mut vocab = Vec::with_capacity(K + 1);
        vocab.push("<blank>".to_string());
        for i in 0..K {
            vocab.push(format!("sym_{i}"));
        }
        let reference: Vec<ExpectedPhoneme> = (0..K)
            .map(|i| ep(&format!("sym_{i}"), true, i, 0))
            .collect();
        let mut th_map = HashMap::new();
        for i in 0..K {
            th_map.insert(format!("sym_{i}"), 0.5);
        }
        let thresholds = PhonemeThresholds(th_map);

        // Each chunk fits K/2 = 300 symbol-frames in the first 300 positions
        // and pads the remaining 200 frames with blanks (giving Viterbi slack
        // around the chunk boundary).
        let half = K / 2;
        let mut chunk1_indices = Vec::with_capacity(FRAMES_PER_CHUNK);
        for i in 0..half {
            chunk1_indices.push(i + 1);
        }
        for _ in 0..(FRAMES_PER_CHUNK - half) {
            chunk1_indices.push(0);
        }
        let mut chunk2_indices = Vec::with_capacity(FRAMES_PER_CHUNK);
        for i in half..K {
            chunk2_indices.push(i + 1);
        }
        for _ in 0..(FRAMES_PER_CHUNK - half) {
            chunk2_indices.push(0);
        }
        let chunk1 = logit_frames_for_indices(&chunk1_indices, vocab.len());
        let chunk2 = logit_frames_for_indices(&chunk2_indices, vocab.len());
        let phonemizer =
            SequencedMockPhonemizer::new(vec![chunk1, chunk2], vocab.len());

        // 20 s audio → exactly 2 chunks at the default 10 s chunk_seconds.
        let audio = vec![0.0_f32; 2 * CHUNK_SAMPLES];
        let session_id = SessionId("coverage-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:20Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        let result = run_chunked_phonemizer_then_align_inner(
            &ctx,
            &phonemizer as &dyn Phonemizer,
            |_pct| {},
        )
        .expect("evaluate ok");

        // Every one of the K labels was visited exactly once, scored high
        // enough to clear the 0.5 threshold. K > FRAMES_PER_CHUNK means a
        // per-chunk alignment would have failed; single-pass over the
        // concatenated buffer (T = 1000 ≥ K = 600) succeeds.
        assert_eq!(
            result.phoneme_attempts.0.len(),
            K,
            "every distinct reference symbol must produce an attempts entry"
        );
        for i in 0..K {
            let sym = format!("sym_{i}");
            let roll = result
                .phoneme_attempts
                .0
                .get(&sym)
                .unwrap_or_else(|| panic!("{sym} attempts present"));
            assert_eq!(roll.occurrences, 1, "{sym} occurrences");
            assert_eq!(roll.flagged, 0, "{sym} should not be flagged");
            assert!(
                roll.mean_certainty.is_some(),
                "{sym} mean_certainty should be present"
            );
        }

        // Nothing was flagged → no entries in flagged_phonemes_ordered.
        assert!(result.flagged_phonemes_ordered.is_empty());
        assert!(result.highest_error_phoneme.is_none());
    }

    #[test]
    fn aggregate_filter_silences_partial_read_flags() {
        // Co-located proof that the from_fallback filter applies end-to-end
        // through the orchestrator's single-pass-align path. Build a pool
        // directly (fabricating an alignment is brittle — Viterbi insists on
        // visiting every label state) and verify `aggregate_phoneme_attempts`
        // (which the helper calls via `build_final_result`) excludes
        // fallback occurrences from `flagged` and `mean_certainty`.
        //
        // This guards the load-bearing claim of the partial-read fix:
        // labels the learner never reached produce fallback occurrences;
        // those must NOT show up in the flagged list.
        let mut pool: std::collections::HashMap<(usize, usize), PooledOccurrence> =
            std::collections::HashMap::new();

        // Real, low-certainty occurrence of θ (below threshold) — must flag.
        pool.insert(
            (0, 0),
            PooledOccurrence {
                symbol: "θ".to_string(),
                certainty: 0.10,
                from_fallback: false,
                flagged: true,
            },
        );
        // Fallback occurrence of ɹ (unspoken in the read) — must NOT flag.
        pool.insert(
            (1, 0),
            PooledOccurrence {
                symbol: "ɹ".to_string(),
                certainty: 0.05,
                from_fallback: true,
                flagged: true,
            },
        );
        // Fallback occurrence of æ (also unspoken) — must NOT flag.
        pool.insert(
            (2, 0),
            PooledOccurrence {
                symbol: "æ".to_string(),
                certainty: 0.05,
                from_fallback: true,
                flagged: true,
            },
        );

        let vocab = vec![
            "<blank>".to_string(),
            "θ".to_string(),
            "ɹ".to_string(),
            "æ".to_string(),
        ];
        let reference = vec![
            ep("θ", true, 0, 0),
            ep("ɹ", true, 1, 0),
            ep("æ", true, 2, 0),
        ];
        let mut th_map = HashMap::new();
        for sym in ["θ", "ɹ", "æ"] {
            th_map.insert(sym.to_string(), 0.5);
        }
        let thresholds = PhonemeThresholds(th_map);
        let audio: Vec<f32> = Vec::new();
        let session_id = SessionId("partial-read-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:10Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        // Drive directly through the shared `build_final_result` helper —
        // same call site the orchestrator uses on completion.
        let result = chunk_orchestrator::build_final_result(
            &pool,
            &ctx,
            10.0,
            "mock-version-0.0.0",
        );

        // θ: real low-cert → flagged.
        let theta = result.phoneme_attempts.0.get("θ").expect("θ present");
        assert_eq!(theta.flagged, 1, "real low-cert θ must flag");

        // ɹ and æ: fallback → must NOT flag, and mean_certainty must be None.
        for sym in ["ɹ", "æ"] {
            let roll = result.phoneme_attempts.0.get(sym).expect("present");
            assert!(roll.occurrences > 0, "{sym} occurrences");
            assert_eq!(
                roll.flagged, 0,
                "{sym} fallback occurrence must be filtered from flagged"
            );
            assert!(
                roll.mean_certainty.is_none(),
                "{sym} mean_certainty must be None (only fallback contributors)"
            );
        }

        // Ordered list only has θ.
        let flagged_syms: Vec<&str> = result
            .flagged_phonemes_ordered
            .iter()
            .map(|f| f.phoneme.as_str())
            .collect();
        assert_eq!(flagged_syms, vec!["θ"]);
        assert_eq!(result.highest_error_phoneme.as_deref(), Some("θ"));
    }

    #[test]
    fn non_fallback_low_certainty_still_flagged() {
        // Guard against over-filtering: a real non-fallback occurrence with
        // certainty BELOW threshold must still flag. Target logit chosen so
        // post-softmax the dominant-index posterior sits well under 0.5.
        let vocab = vec!["<blank>".to_string(), "θ".to_string()];
        let reference = vec![ep("θ", true, 0, 0)];
        let mut th_map = HashMap::new();
        th_map.insert("θ".to_string(), 0.95);
        let thresholds = PhonemeThresholds(th_map);

        // Low-logit frames: target_logit = 0.5 → after softmax over 2-symbol
        // vocab, posterior ≈ 0.62. Below the 0.95 threshold but well above
        // any fallback noise.
        let frames = logit_frames_low_confidence(&[1, 1, 1, 1, 1, 1, 1, 1], vocab.len(), 0.5);
        let phonemizer = MockPhonemizer::with_fixed(frames, vocab.len());

        let audio = vec![0.0_f32; 10 * 16_000];
        let session_id = SessionId("low-cert-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:10Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        let result = run_chunked_phonemizer_then_align_inner(
            &ctx,
            &phonemizer as &dyn Phonemizer,
            |_pct| {},
        )
        .expect("evaluate ok");

        // θ must end up in flagged_phonemes_ordered: real alignment + low
        // certainty + below-threshold.
        let theta = result.phoneme_attempts.0.get("θ").expect("θ present");
        assert_eq!(theta.occurrences, 1);
        assert_eq!(
            theta.flagged, 1,
            "non-fallback low-certainty occurrence must still flag"
        );
        assert!(theta.mean_certainty.is_some());
        assert!(
            result
                .flagged_phonemes_ordered
                .iter()
                .any(|f| f.phoneme == "θ"),
            "θ must appear in flagged_phonemes_ordered"
        );
    }

    #[test]
    fn chunked_phonemizer_emits_progress_per_chunk() {
        // 30 s audio + default 10 s chunks → exactly 3 chunks. Each chunk
        // gets distinct canned logits via the sequenced mock. Assert 3
        // progress events fire with monotone `pct` ending at 1.0.
        let vocab = vec!["<blank>".to_string(), "θ".to_string()];
        let reference = vec![ep("θ", true, 0, 0)];
        let mut th_map = HashMap::new();
        th_map.insert("θ".to_string(), 0.5);
        let thresholds = PhonemeThresholds(th_map);

        let chunk_frames = |idx: usize| {
            // Place a symbol-frame in the middle of a sandwich of blanks.
            let mut per_frame = vec![0usize, 0];
            per_frame.push(1);
            for _ in 0..idx {
                per_frame.push(0);
            }
            logit_frames_for_indices(&per_frame, vocab.len())
        };
        let rounds = vec![chunk_frames(0), chunk_frames(1), chunk_frames(2)];
        let phonemizer = SequencedMockPhonemizer::new(rounds, vocab.len());

        let audio = vec![0.0_f32; 30 * 16_000];
        let session_id = SessionId("progress-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:30Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        let collected = std::sync::Arc::new(std::sync::Mutex::new(Vec::<f32>::new()));
        let collected_for_cb = std::sync::Arc::clone(&collected);
        let on_progress = move |pct: f32| {
            collected_for_cb.lock().unwrap().push(pct);
        };

        let _ = run_chunked_phonemizer_then_align_inner(
            &ctx,
            &phonemizer as &dyn Phonemizer,
            on_progress,
        )
        .expect("evaluate ok");

        let pcts = collected.lock().unwrap().clone();
        assert_eq!(pcts.len(), 3, "expected one progress event per chunk");
        for w in pcts.windows(2) {
            assert!(w[1] >= w[0], "pct must be monotone, got {:?}", pcts);
        }
        assert!(
            (pcts.last().copied().unwrap() - 1.0).abs() < 1e-6,
            "final pct must land at 1.0, got {pcts:?}"
        );
    }

    #[test]
    fn phonemizer_error_propagates() {
        // Phonemizer succeeds on chunk 1 of 3, then errors. The orchestrator
        // must return `Err(EvaluationError::RuntimeFailure)` and have emitted
        // progress only for chunk 1.
        let vocab = vec!["<blank>".to_string(), "θ".to_string()];
        let reference = vec![ep("θ", true, 0, 0)];
        let mut th_map = HashMap::new();
        th_map.insert("θ".to_string(), 0.5);
        let thresholds = PhonemeThresholds(th_map);

        let one_chunk = logit_frames_for_indices(&[1, 1, 1, 1], vocab.len());
        let phonemizer = ErrAfterPhonemizer::new(vec![one_chunk], vocab.len());

        let audio = vec![0.0_f32; 30 * 16_000];
        let session_id = SessionId("err-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:30Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        let collected = std::sync::Arc::new(std::sync::Mutex::new(Vec::<f32>::new()));
        let collected_for_cb = std::sync::Arc::clone(&collected);
        let on_progress = move |pct: f32| {
            collected_for_cb.lock().unwrap().push(pct);
        };

        let result = run_chunked_phonemizer_then_align_inner(
            &ctx,
            &phonemizer as &dyn Phonemizer,
            on_progress,
        );
        let err = result.expect_err("phonemizer error must propagate");
        match err {
            EvaluationError::RuntimeFailure(_) => {}
            other => panic!("expected RuntimeFailure, got {other:?}"),
        }

        // Only the first chunk's forward call succeeded → one progress event.
        let pcts = collected.lock().unwrap().clone();
        assert_eq!(
            pcts.len(),
            1,
            "progress should have fired for chunk 1 only, got {pcts:?}"
        );
    }

    #[test]
    fn phonemizer_error_on_first_chunk_propagates_with_no_progress() {
        // Phonemizer errors on chunk 1 of 3 (rounds = []). The orchestrator
        // must return Err(RuntimeFailure) without emitting any progress events
        // — per-chunk progress fires AFTER the forward pass succeeds.
        let vocab = vec!["<blank>".to_string(), "θ".to_string()];
        let reference = vec![ep("θ", true, 0, 0)];
        let mut th_map = HashMap::new();
        th_map.insert("θ".to_string(), 0.5);
        let thresholds = PhonemeThresholds(th_map);

        let phonemizer = ErrAfterPhonemizer::new(Vec::new(), vocab.len());

        let audio = vec![0.0_f32; 30 * 16_000];
        let session_id = SessionId("err-chunk1-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:30Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        let collected = std::sync::Arc::new(std::sync::Mutex::new(Vec::<f32>::new()));
        let collected_for_cb = std::sync::Arc::clone(&collected);
        let on_progress = move |pct: f32| {
            collected_for_cb.lock().unwrap().push(pct);
        };

        let result = run_chunked_phonemizer_then_align_inner(
            &ctx,
            &phonemizer as &dyn Phonemizer,
            on_progress,
        );
        let err = result.expect_err("phonemizer error must propagate");
        match err {
            EvaluationError::RuntimeFailure(_) => {}
            other => panic!("expected RuntimeFailure, got {other:?}"),
        }

        let pcts = collected.lock().unwrap().clone();
        assert!(
            pcts.is_empty(),
            "no progress should fire when chunk 1 fails before completing its forward pass, got {pcts:?}"
        );
    }

    #[test]
    fn chunked_phonemizer_empty_audio_returns_empty_result() {
        // Empty audio → `chunk_windows` returns []; the orchestrator's
        // `windows.is_empty()` defensive branch must short-circuit with a
        // zero-attempt EvaluationResult and emit no progress events. Without
        // this branch, a downstream `align_to_reference` call over empty
        // posteriors would either panic or error opaquely.
        let vocab = vec!["<blank>".to_string(), "θ".to_string()];
        let reference = vec![ep("θ", true, 0, 0)];
        let mut th_map = HashMap::new();
        th_map.insert("θ".to_string(), 0.5);
        let thresholds = PhonemeThresholds(th_map);

        let audio: Vec<f32> = Vec::new();
        let session_id = SessionId("empty-audio-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:00Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        let phonemizer = MockPhonemizer::with_fixed(
            logit_frames_for_indices(&[1], vocab.len()),
            vocab.len(),
        );

        let collected = std::sync::Arc::new(std::sync::Mutex::new(Vec::<f32>::new()));
        let collected_for_cb = std::sync::Arc::clone(&collected);
        let on_progress = move |pct: f32| {
            collected_for_cb.lock().unwrap().push(pct);
        };

        let result = run_chunked_phonemizer_then_align_inner(
            &ctx,
            &phonemizer as &dyn Phonemizer,
            on_progress,
        )
        .expect("empty audio must short-circuit Ok");

        assert!(result.phoneme_attempts.0.is_empty());
        assert!(result.flagged_phonemes_ordered.is_empty());
        assert!(result.highest_error_phoneme.is_none());
        assert_eq!(result.duration_seconds, 0.0);
        assert!(
            collected.lock().unwrap().is_empty(),
            "no progress events should fire on empty audio"
        );
    }

    // ---- evaluate_chunked error path (legacy coverage) --------------------
    //
    // The chunk_orchestrator's `evaluate_chunked` is kept in tree per the
    // fix-plan ("kept in tree, useful for future progressive-UI rework").
    // This test exercises the AppError-mapping boundary on the
    // `evaluate_chunked` side specifically; the new
    // `phonemizer_error_propagates` test covers the same property on the
    // chunked-phonemizer-then-single-pass-align flow.

    /// Phonemizer that always errors. Used to assert the error-mapping path.
    struct AlwaysErrPhonemizer;
    impl Phonemizer for AlwaysErrPhonemizer {
        fn forward(&self, _audio: &[f32]) -> Result<Vec<Logits>, EvaluationError> {
            Err(EvaluationError::RuntimeFailure("synthetic forward fail".into()))
        }
        fn vocab_size(&self) -> usize {
            2
        }
        fn model_version(&self) -> &str {
            "always-err-0.0.0"
        }
    }

    #[test]
    fn evaluate_chunked_phonemizer_error_maps_to_app_error_inference() {
        let vocab = vec!["<blank>".to_string(), "θ".to_string()];
        let reference = vec![ep("θ", true, 0, 0)];
        let mut th_map = HashMap::new();
        th_map.insert("θ".to_string(), 0.5);
        let thresholds = PhonemeThresholds(th_map);
        let audio = vec![0.0_f32; 16_000];
        let session_id = SessionId("err-session".to_string());
        let allophones = minimal_allophone_map_orch(&vocab);
        let ctx = build_evaluation_context(
            &session_id,
            &audio,
            "2026-06-01T12:00:00Z",
            "2026-06-01T12:00:01Z",
            &reference,
            &vocab,
            &thresholds,
            &allophones,
            DifficultyLevel::Standard,
            1,
            1,
        );

        let phonemizer = AlwaysErrPhonemizer;
        let on_progress = |_p: ChunkProgress| {};
        let result = evaluate_chunked(
            &ctx,
            &phonemizer as &dyn Phonemizer,
            ChunkParams::default(),
            on_progress,
        );
        let err = result.expect_err("forward error must propagate");
        let mapped = AppError::Inference(err);
        assert_eq!(error_kind(&mapped), "inference_runtime");
    }

    // ---- passage sentence count -------------------------------------------

    #[test]
    fn passage_sentence_count_handles_basic_punctuation() {
        let p = Passage {
            text: "Hello world. This is a test! Right?".to_string(),
            expected_ipa_per_word: Vec::new(),
        };
        assert_eq!(passage_sentence_count(&p), 3);
    }

    #[test]
    fn passage_sentence_count_floor_is_one() {
        // A passage without terminal punctuation still counts as ≥1 sentence
        // so the reattempt counts vec is never empty.
        let p = Passage {
            text: "Hello world".to_string(),
            expected_ipa_per_word: Vec::new(),
        };
        assert_eq!(passage_sentence_count(&p), 1);

        let empty = Passage {
            text: "".to_string(),
            expected_ipa_per_word: Vec::new(),
        };
        assert_eq!(passage_sentence_count(&empty), 1);
    }
}
