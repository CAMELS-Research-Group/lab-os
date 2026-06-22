//! IAS client evaluation feature — ONNX inference + scoring. Fleshed out by
//! later tasks (CL-15 decoder, CL-17 alignment, CL-19 orchestrator).
//!
//! Per ADD §3.10 the feature owns a single error enum
//! ([`EvaluationError`]); per ADD §6.3 the [`Phonemizer`] trait is the
//! seam used by every downstream test in the pipeline.

pub mod alignment;
pub mod allophones;
pub mod chunk_orchestrator;
pub mod commands;
pub mod ctc_decode;
pub mod error;
pub mod feedback;
pub mod model_download;
pub mod orchestrator;
pub mod phonemizer;
pub mod reference_ipa;
pub mod thresholds;

pub use alignment::{
    align_to_reference, AlignmentResult, OccurrenceCertainty, PhonemeSummary,
};
pub use chunk_orchestrator::{
    evaluate_chunked, ChunkParams, ChunkProgress, EvaluationContext, PartialChunkResult,
};
pub use ctc_decode::{ctc_greedy_decode, CtcDecodeOutput};
pub use error::EvaluationError;
pub use feedback::{generate_feedback, lookup_articulation, ArticulationEntry, FeedbackEntry};
pub use phonemizer::{Logits, OnnxPhonemizer, Phonemizer};
pub use reference_ipa::{get_passage, load_expected_phonemes, load_passage, ExpectedPhoneme};
pub use allophones::{load_bundled as load_bundled_allophones, AllophoneError, AllophoneMap};
pub use thresholds::{ThresholdTable, V1_TARGET_PHONEMES};
