//! Feature-local error type for the evaluation pipeline.
//!
//! Per ADD §3.10 each feature module owns a single `thiserror`-derived enum
//! that bubbles into [`crate::shared::error::AppError`] via `#[from]`. The
//! evaluation feature covers two cooperating subsystems — threshold-table
//! loading (CL-9a) and phonemizer inference (CL-14) — so the variants below
//! span both concerns. Downstream tasks (CL-17 alignment, CL-19 orchestrator)
//! extend this enum rather than introducing parallel error types.

use thiserror::Error;

/// Errors produced by the evaluation pipeline.
#[derive(Debug, Error)]
pub enum EvaluationError {
    /// The bundled threshold table failed to load, parse, or validate.
    /// CL-19 surfaces this as a graceful startup failure rather than
    /// silently falling back to defaults.
    #[error("threshold table load failed: {0}")]
    ThresholdTableLoad(String),

    /// The ONNX model file was not present at the resolved cache path.
    /// CL-19 (orchestrator) is responsible for download + placement; if the
    /// file is missing at load time the client cannot evaluate audio and the
    /// frontend surfaces a terminal banner.
    #[error("model file not found at {0}")]
    ModelNotFound(String),

    /// The on-disk model's SHA-256 did not match
    /// [`crate::shared::config::BuildConfig::MODEL_SHA256`]. This is the
    /// integrity gate that makes the client refuse a swapped model — see
    /// TRD §4.8 ("Model URL + digest pinning") and the dev-sentinel rationale
    /// on [`crate::shared::config::BuildConfig`].
    #[error("model digest mismatch: expected {expected}, got {actual}")]
    ModelDigestMismatch { expected: String, actual: String },

    /// The `ort` session failed to load, validate, or run the graph. The
    /// inner message is the verbatim `ort` error string so log triage can
    /// distinguish opset / operator / shape issues without re-running.
    #[error("inference runtime failure: {0}")]
    RuntimeFailure(String),

    /// The audio buffer handed to [`crate::evaluation::Phonemizer::forward`]
    /// was the wrong shape for the V1 model (16 kHz mono f32). The
    /// `got_len` field carries the actual sample count so callers can log
    /// the gap; the expected minimum is described in the variant message.
    #[error(
        "unsupported audio input shape (got len {got_len} samples; expected ≥ {min_len} mono f32 samples @ 16 kHz)"
    )]
    UnsupportedInputShape { got_len: usize, min_len: usize },

    /// A logits frame handed to [`crate::evaluation::ctc_greedy_decode`] did
    /// not match the supplied vocabulary length. `frame_index` identifies the
    /// offending frame in the input sequence (0 when the mismatch is detected
    /// before the decode loop, e.g. empty-vocab with non-empty logits).
    /// Mirrors the `ValueError` raised by the Python reference in
    /// `tools/model_export/exporter/validate.py`.
    #[error(
        "ctc shape mismatch: vocab_len {vocab_len}, frame_len {frame_len} (frame index {frame_index})"
    )]
    CtcShapeMismatch {
        vocab_len: usize,
        frame_len: usize,
        frame_index: usize,
    },

    /// The supplied CTC blank index was `>= vocab.len()`, so the decoder
    /// cannot distinguish blank frames from real tokens. Surfaced separately
    /// from [`Self::CtcShapeMismatch`] so triage logs do not conflate a
    /// configuration error with a per-frame width problem. Mirrors the
    /// `ValueError` raised by the Python reference in
    /// `tools/model_export/exporter/validate.py`.
    #[error("ctc blank index out of range: blank_idx {blank_idx}, vocab_len {vocab_len}")]
    BlankIndexOutOfRange { blank_idx: usize, vocab_len: usize },

    /// The bundled reference IPA artifact (`passages/<name>.ipa.json`) failed
    /// to load, parse, or validate. See
    /// [`crate::evaluation::reference_ipa`] for the on-disk shape; failure
    /// modes include missing file, malformed JSON, parallel `ipa`/`is_target`
    /// arrays of differing lengths, and any non-IO read failure.
    #[error("bundled reference IPA load failed: {0}")]
    BundledReferenceLoad(String),

    /// The bundled reference IPA artifact's `v1_target_phonemes` array did not
    /// match [`crate::evaluation::V1_TARGET_PHONEMES`] exactly. This catches a
    /// stale bundle (committed before an inventory revision) and a stale
    /// inventory constant (revised in code without a regenerated bundle) in
    /// the same check — both surface here so the operator regenerates the
    /// bundle before shipping. Hard-fail; do not silently tag against the
    /// wrong inventory.
    ///
    /// `espeak_ng_version` and `espeak_voice` are surfaced verbatim from the
    /// bundle so a drift log line auto-attributes which generator emitted the
    /// stale inventory.
    #[error(
        "bundled reference IPA inventory mismatch (espeak-ng {espeak_ng_version}, voice {espeak_voice}): bundle has {bundle:?}, code expects {expected:?}"
    )]
    BundledReferenceInventoryMismatch {
        bundle: Vec<String>,
        expected: Vec<String>,
        espeak_ng_version: String,
        espeak_voice: String,
    },

    /// The blank-augmented Viterbi path against the reference returned no
    /// finite end log-probability, or the input shape was structurally
    /// impossible (empty reference, more reference labels than frames). The
    /// `detail` field carries the specific shape mismatch so triage logs
    /// distinguish a degenerate input from a posterior/reference mismatch.
    ///
    /// CL-19 surfaces this as a per-utterance scoring failure; the
    /// orchestrator may degrade to skipping the utterance rather than
    /// failing the whole session.
    #[error("forced alignment infeasible: {detail}")]
    AlignmentInfeasible { detail: String },

    /// A symbol in the bundled reference IPA does not appear in the model's
    /// vocabulary. This is a build-time inconsistency between the espeak-ng
    /// passage bundle (CL-16) and the wav2vec2 export's `ipa_vocabulary`
    /// (`tools/model_export/build/manifest.json`); the operator regenerates
    /// one to match the other rather than ignoring the symbol at runtime.
    #[error("reference IPA symbol {symbol:?} not in model vocabulary")]
    ReferenceSymbolNotInVocab { symbol: String },

    /// The chunk-window parameters handed to
    /// [`crate::evaluation::chunk_orchestrator::evaluate_chunked`] cannot
    /// produce a valid window schedule. Surfaced for `overlap_seconds >=
    /// chunk_seconds`, non-positive `chunk_seconds`, or non-finite values —
    /// any case where the hop / window math would diverge.
    #[error("chunk-window parameters infeasible: {detail}")]
    ChunkInfeasible { detail: String },

    /// The bundled allophone map failed to load, parse, or validate against
    /// the current model vocabulary. Distinct from
    /// [`Self::ThresholdTableLoad`] so triage logs can distinguish a
    /// vocabulary-mismatch from a threshold-schema problem.
    #[error("allophone map load failed: {0}")]
    AllophoneMapLoad(String),
}
