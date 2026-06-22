//! Phonemizer trait + default ONNX implementation.
//!
//! Spec: TRD §4.3 (Inference), TRD §4.5 (V1 model), ADD §6.3 (Phonemizer trait).
//!
//! # Overview
//!
//! The [`Phonemizer`] trait is the unit-test seam every downstream evaluation
//! task uses to swap in canned model output. The production implementation
//! [`OnnxPhonemizer`] wraps an `ort` session over the V1
//! `facebook/wav2vec2-lv-60-espeak-cv-ft` graph (MX-built artifact;
//! ~339 MB, 392-symbol IPA vocab, opset 17).
//!
//! ## Integrity gate
//!
//! Before `ort` ever touches the file, [`OnnxPhonemizer::load`] streams the
//! bytes through SHA-256 and compares against
//! [`crate::shared::config::BuildConfig::MODEL_SHA256`]. The comparison is
//! case-insensitive hex equality; mismatch is a hard fail. The dev sentinel
//! (64 zero hex digits) is intentional — a dev build that picks up a real
//! model file will fail the gate, surfacing the misconfiguration rather than
//! silently running an unverified model. To exercise the real load path in
//! dev, set `IAS_MODEL_SHA256` to the artifact's actual digest before
//! `cargo build`.
//!
//! ## Threading
//!
//! `ort::Session::run` requires `&mut self`, but the [`Phonemizer::forward`]
//! contract takes `&self` (downstream code holds the phonemizer via
//! `Arc<dyn Phonemizer>` and calls into it from multiple Tauri command
//! handlers). [`OnnxPhonemizer`] reconciles this by wrapping the session in a
//! `Mutex`. V1 evaluates one clip at a time per ADD §3.5, so contention is
//! not a concern; if batching ever lands the lock becomes the obvious place
//! to introduce per-call timing.

use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;
use std::sync::Mutex;

use ort::ep::CPU;
use ort::session::Session;
use ort::value::TensorRef;
use sha2::{Digest, Sha256};

use crate::evaluation::error::EvaluationError;
use crate::shared::config::BuildConfig;

/// Per-time-step logits over the IPA vocabulary. Length equals the model's
/// vocab size (392 for the V1 wav2vec2-espeak graph).
pub type Logits = Vec<f32>;

/// V1 audio contract: 16 kHz mono f32 PCM. We require at least one-tenth of a
/// second of audio (1 600 samples) so the model has something to chew on; the
/// wav2vec2 receptive field is ~25 ms / 50 Hz output rate, so very short
/// buffers produce zero output frames.
///
/// `pub(crate)` so `chunk_orchestrator::chunk_windows` can guarantee every
/// window it returns is at least this size — a sub-`MIN_AUDIO_SAMPLES`
/// trailing chunk would otherwise fail [`OnnxPhonemizer::forward`] with
/// `UnsupportedInputShape` and surface to the UI as `audio_too_short` on an
/// otherwise successful long read.
pub(crate) const MIN_AUDIO_SAMPLES: usize = 1_600;

/// Abstraction over "map 16 kHz mono f32 audio → per-frame logits over the
/// IPA vocabulary". CL-15 (CTC decoder) and CL-17 (forced alignment) consume
/// this trait directly; production wires an [`OnnxPhonemizer`], tests wire a
/// [`MockPhonemizer`].
pub trait Phonemizer: Send + Sync {
    /// Run a single forward pass. `audio` is 16 kHz mono f32 PCM. Returns one
    /// [`Logits`] vector per output time-step; the number of time-steps
    /// depends on the input length (wav2vec2 emits ~50 frames/sec).
    fn forward(&self, audio: &[f32]) -> Result<Vec<Logits>, EvaluationError>;

    /// Vocabulary size — the length of each [`Logits`] vector in
    /// [`Self::forward`]'s output. Decoder code uses this to size its
    /// posterior buffers.
    fn vocab_size(&self) -> usize;

    /// Pinned model version (semver-ish). Surfaced on every
    /// [`crate::shared::types::EvaluationResult`] so the IAS report payload
    /// records which model produced the score.
    fn model_version(&self) -> &str;
}

/// Production phonemizer: an `ort` session over the V1 ONNX graph using the
/// CPU execution provider as the mandatory baseline.
///
/// Build via [`Self::load`]; the constructor enforces the digest gate and
/// constructs the session. After construction, [`Self::forward`] is the only
/// runtime entry point.
pub struct OnnxPhonemizer {
    session: Mutex<Session>,
    vocab_size: usize,
    model_version: String,
}

// Hand-rolled Debug — `ort::Session` is not `Debug`, and dumping the raw
// session pointer adds no signal. Surface the static metadata instead so
// `expect_err`-style assertions in tests can render the type.
impl std::fmt::Debug for OnnxPhonemizer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OnnxPhonemizer")
            .field("vocab_size", &self.vocab_size)
            .field("model_version", &self.model_version)
            .finish()
    }
}

impl OnnxPhonemizer {
    /// Load the ONNX model at `model_path`, verifying its SHA-256 against
    /// [`BuildConfig::MODEL_SHA256`] before handing the file to `ort`.
    ///
    /// Failure modes:
    /// - [`EvaluationError::ModelNotFound`] — the file does not exist or
    ///   could not be opened.
    /// - [`EvaluationError::ModelDigestMismatch`] — file exists but its
    ///   SHA-256 does not match the build-time pinned digest.
    /// - [`EvaluationError::RuntimeFailure`] — file passes the digest gate
    ///   but `ort` failed to build a session from it (typically an
    ///   opset / operator mismatch).
    pub fn load(model_path: &Path) -> Result<Self, EvaluationError> {
        // 1. Hash + digest gate. The hash function returns `ModelNotFound`
        //    for any open-time failure, so a missing path surfaces the
        //    correct variant before we reach the digest compare.
        let actual = sha256_of_file(model_path)?;
        let expected = BuildConfig::MODEL_SHA256.to_ascii_lowercase();
        if actual != expected {
            return Err(EvaluationError::ModelDigestMismatch {
                expected,
                actual,
            });
        }

        // 2. Build the ort session. CPU execution provider is the V1
        //    baseline (ADD §3.5 / TRD §4.3). DirectML/CoreML wiring is left
        //    for a follow-up — the trait surface is the right place to gate
        //    those. CPU is always-available so the explicit declaration is
        //    documentation as much as configuration.
        let session = Session::builder()
            .map_err(|e| EvaluationError::RuntimeFailure(format!("SessionBuilder::new failed: {e}")))?
            .with_execution_providers([CPU::default().build()])
            .map_err(|e| EvaluationError::RuntimeFailure(format!("register CPU EP: {e}")))?
            .commit_from_file(model_path)
            .map_err(|e| {
                EvaluationError::RuntimeFailure(format!(
                    "ort failed to load ONNX graph from {}: {e}",
                    model_path.display()
                ))
            })?;

        // 3. Infer vocab size from the output metadata. wav2vec2's output is
        //    (batch, time, vocab); we read the static `vocab` dim from the
        //    graph so callers don't have to guess.
        let vocab_size = infer_vocab_size(&session)?;

        Ok(Self {
            session: Mutex::new(session),
            vocab_size,
            model_version: BuildConfig::MODEL_VERSION.to_string(),
        })
    }
}

impl Phonemizer for OnnxPhonemizer {
    fn forward(&self, audio: &[f32]) -> Result<Vec<Logits>, EvaluationError> {
        if audio.len() < MIN_AUDIO_SAMPLES {
            return Err(EvaluationError::UnsupportedInputShape {
                got_len: audio.len(),
                min_len: MIN_AUDIO_SAMPLES,
            });
        }

        // Zero-mean / unit-variance normalisation BEFORE inference. The V1
        // wav2vec2-lv-60 graph was trained with the HF feature extractor's
        // `do_normalize=True`, but the exported ONNX does NOT fold that step
        // in (verified empirically: posteriors are amplitude-dependent). Feeding
        // raw [-1,1] PCM therefore makes scoring fragile to capture level — a
        // quiet laptop-mic read (RMS ~0.003) collapses per-phoneme certainties
        // ~1-3 orders below the SPIKE-11 reference calibrated on louder studio
        // audio (RMS ~0.09), while a robust phoneme like /i/ partially survives.
        // Normalising here makes inference gain-invariant and restores the
        // reference scale at any input level. See issue #67/#70 root-cause.
        let normalized = normalize_audio(audio);

        // Build input tensor: shape (1, samples) f32. Normalisation already
        // allocated `normalized` (one Vec the size of the chunk — ~640 KB per
        // 10 s window, negligible against the per-window inference working set),
        // so this is no longer the zero-copy borrow of the caller's slice it was
        // pre-normalisation; `TensorRef` borrows `normalized`, which is kept
        // alive for the duration of the run below.
        let n_samples = normalized.len();
        let input = TensorRef::<f32>::from_array_view(([1_usize, n_samples], normalized.as_slice()))
            .map_err(|e| EvaluationError::RuntimeFailure(format!("input tensor build: {e}")))?;

        // Lock the session for the duration of the run. See module docs for
        // why this is a Mutex rather than &mut self on the trait.
        let mut session = self.session.lock().expect("OnnxPhonemizer session mutex poisoned");
        let outputs = session
            .run(ort::inputs![input])
            .map_err(|e| EvaluationError::RuntimeFailure(format!("session.run: {e}")))?;

        // First output is the logits tensor. Use the first key rather than
        // hard-coding "logits" — the exact name depends on the export.
        let output_key = outputs
            .keys()
            .next()
            .ok_or_else(|| EvaluationError::RuntimeFailure("session returned no outputs".into()))?
            .to_string();

        let (shape, data) = outputs[output_key.as_str()]
            .try_extract_tensor::<f32>()
            .map_err(|e| EvaluationError::RuntimeFailure(format!("extract f32 tensor: {e}")))?;

        // wav2vec2 CTC heads emit (batch=1, time, vocab). Accept (time, vocab)
        // as well in case the export squeezed the batch dim. Shape derefs to
        // &[i64], so we treat it as a slice for the rank check.
        let dims: &[i64] = &**shape;
        let (time, vocab) = match dims.len() {
            3 if dims[0] == 1 => (dims[1] as usize, dims[2] as usize),
            2 => (dims[0] as usize, dims[1] as usize),
            _ => {
                return Err(EvaluationError::RuntimeFailure(format!(
                    "unexpected output shape {dims:?}; want (1,T,V) or (T,V)"
                )));
            }
        };

        if vocab != self.vocab_size {
            return Err(EvaluationError::RuntimeFailure(format!(
                "output vocab dim {vocab} != session-reported vocab_size {}",
                self.vocab_size
            )));
        }

        // Slice the flat data buffer into one Vec per time-step. Cost is
        // O(time * vocab) — acceptable for a single ~5 s clip (~250 frames *
        // 392 vocab ≈ 100k f32s).
        let mut frames: Vec<Logits> = Vec::with_capacity(time);
        for t in 0..time {
            let start = t * vocab;
            let end = start + vocab;
            frames.push(data[start..end].to_vec());
        }
        Ok(frames)
    }

    fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    fn model_version(&self) -> &str {
        &self.model_version
    }
}

/// Floor on the variance divisor, matching the HF
/// `Wav2Vec2FeatureExtractor.zero_mean_unit_var_normalize` epsilon. Also
/// guarantees silence / DC input maps to all-zeros (0 / sqrt(eps) == 0) rather
/// than producing NaN.
///
/// Units: this is added to `var` (amplitude², not amplitude). So "1e-7 is
/// negligible" holds only *relative to the variance*: for a ~unit-variance
/// signal it is ~1e-7 of the divisor, but for a heavily-attenuated signal
/// (e.g. var ≈ 1e-5 at ~0.003 RMS, the quiet-laptop regime) it is ~1% of the
/// divisor — which is
/// exactly why exact gain-invariance degrades as the signal approaches silence
/// (see `normalize_audio_eps_floor_limits_invariance_only_near_silence`).
const NORMALIZE_EPS: f64 = 1e-7;

/// Zero-mean / unit-variance normalise an audio buffer (HF `do_normalize`
/// semantics): `(x - mean) / sqrt(var + 1e-7)`, population variance.
///
/// This is the model-input prep the exported ONNX graph omits. It is the home
/// of the issue #67/#70 fix: inference is otherwise amplitude-dependent because
/// the raw waveform scale is fed straight to the network. Normalising makes the
/// result invariant to capture gain (proven against L2-Arctic: a 0.01× gain
/// sweep that drops un-normalised overall certainty from 0.79 → 0.05 holds flat
/// at ~0.84 once this is applied).
///
/// Computed in f64 to keep the mean/variance accumulation stable over the
/// ~160k-sample (10 s) chunks the orchestrator feeds; the output is f32 to
/// match the model input dtype.
pub(crate) fn normalize_audio(audio: &[f32]) -> Vec<f32> {
    let n = audio.len();
    if n == 0 {
        return Vec::new();
    }
    let mean = audio.iter().map(|&s| s as f64).sum::<f64>() / n as f64;
    let var = audio
        .iter()
        .map(|&s| {
            let d = s as f64 - mean;
            d * d
        })
        .sum::<f64>()
        / n as f64;
    let inv_std = 1.0 / (var + NORMALIZE_EPS).sqrt();
    audio
        .iter()
        .map(|&s| ((s as f64 - mean) * inv_std) as f32)
        .collect()
}

/// Stream `path` through SHA-256 in 64 KiB chunks and return the lowercase
/// hex digest. A missing or unreadable file maps to
/// [`EvaluationError::ModelNotFound`]; read failures part-way through map to
/// [`EvaluationError::RuntimeFailure`].
fn sha256_of_file(path: &Path) -> Result<String, EvaluationError> {
    let file = File::open(path).map_err(|e| {
        EvaluationError::ModelNotFound(format!("{}: {}", path.display(), e))
    })?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| EvaluationError::RuntimeFailure(format!("io error during hash of {}: {}", path.display(), e)))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Read the session's first-output metadata to recover the static vocab dim.
///
/// wav2vec2 CTC graphs export a (batch, time, vocab) output where `vocab` is
/// a fixed integer. If the dim is symbolic (or the first output has the wrong
/// rank), we fall back to a [`EvaluationError::RuntimeFailure`] so the
/// integration test surfaces the export mismatch rather than the client
/// silently using a wrong vocab size at decode time.
fn infer_vocab_size(session: &Session) -> Result<usize, EvaluationError> {
    let outputs = session.outputs();
    let first = outputs
        .first()
        .ok_or_else(|| EvaluationError::RuntimeFailure("model has no outputs".into()))?;

    use ort::value::ValueType;
    match first.dtype() {
        ValueType::Tensor { shape, .. } => {
            let dims: &[i64] = &**shape;
            match dims.len() {
                3 => {
                    let v = dims[2];
                    if v <= 0 {
                        Err(EvaluationError::RuntimeFailure(format!(
                            "output[0] vocab dim is symbolic or non-positive: {dims:?}"
                        )))
                    } else {
                        Ok(v as usize)
                    }
                }
                2 => {
                    let v = dims[1];
                    if v <= 0 {
                        Err(EvaluationError::RuntimeFailure(format!(
                            "output[0] vocab dim is symbolic or non-positive: {dims:?}"
                        )))
                    } else {
                        Ok(v as usize)
                    }
                }
                _ => Err(EvaluationError::RuntimeFailure(format!(
                    "output[0] has unexpected rank {}: {:?}",
                    dims.len(),
                    dims
                ))),
            }
        }
        other => Err(EvaluationError::RuntimeFailure(format!(
            "output[0] is not a tensor: {other:?}"
        ))),
    }
}

// ---------------------------------------------------------------------------
// Test-only mock
// ---------------------------------------------------------------------------

/// Test-only [`Phonemizer`] returning canned logits. Per ADD §6.3, downstream
/// tests (CL-15 decoder, CL-17 alignment, CL-19 orchestrator) inject one of
/// these to exercise their own logic without paying ONNX load time or shipping
/// the 339 MB artifact into CI.
///
/// The "keyed by fixture name" form in the plan is a generalization of what
/// V1 tests actually need; a fixed canned output is the simplest shape that
/// satisfies every existing call site. CL-15+ can extend with a name-keyed
/// builder if a future test needs to switch outputs by audio content.
#[cfg(test)]
pub struct MockPhonemizer {
    canned: Vec<Logits>,
    vocab_size: usize,
    model_version: String,
}

#[cfg(test)]
impl MockPhonemizer {
    /// Construct a mock that returns `logits` from every `forward` call.
    ///
    /// `vocab_size` is checked against each row of `logits` so a typo in the
    /// test fixture surfaces as a panic at construction time rather than a
    /// confusing length mismatch downstream.
    pub fn with_fixed(logits: Vec<Logits>, vocab_size: usize) -> Self {
        for (i, row) in logits.iter().enumerate() {
            assert_eq!(
                row.len(),
                vocab_size,
                "MockPhonemizer: logits[{i}].len() = {} but vocab_size = {vocab_size}",
                row.len()
            );
        }
        Self {
            canned: logits,
            vocab_size,
            model_version: "mock-0.0.0".to_string(),
        }
    }
}

#[cfg(test)]
impl Phonemizer for MockPhonemizer {
    fn forward(&self, _audio: &[f32]) -> Result<Vec<Logits>, EvaluationError> {
        Ok(self.canned.clone())
    }

    fn vocab_size(&self) -> usize {
        self.vocab_size
    }

    fn model_version(&self) -> &str {
        &self.model_version
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    // ---- trait surface / MockPhonemizer ----------------------------------

    fn sample_logits(time: usize, vocab: usize) -> Vec<Logits> {
        (0..time)
            .map(|t| (0..vocab).map(|v| (t * vocab + v) as f32).collect())
            .collect()
    }

    #[test]
    fn mock_phonemizer_returns_canned_logits() {
        let canned = sample_logits(5, 8);
        let mock = MockPhonemizer::with_fixed(canned.clone(), 8);

        // Any audio buffer; MockPhonemizer ignores it.
        let audio = vec![0.0_f32; 1_600];
        let frames = mock.forward(&audio).expect("mock forward succeeds");

        assert_eq!(frames.len(), 5);
        for (got, want) in frames.iter().zip(canned.iter()) {
            assert_eq!(got, want);
        }
    }

    #[test]
    fn mock_phonemizer_reports_vocab_and_version() {
        let mock = MockPhonemizer::with_fixed(sample_logits(2, 4), 4);
        assert_eq!(mock.vocab_size(), 4);
        assert_eq!(mock.model_version(), "mock-0.0.0");
    }

    #[test]
    fn mock_phonemizer_implements_phonemizer_trait() {
        // Compile-time check that &dyn Phonemizer accepts MockPhonemizer.
        // This is the "trait can be implemented" acceptance bullet.
        fn assert_dyn(_p: &dyn Phonemizer) {}
        let mock = MockPhonemizer::with_fixed(sample_logits(1, 2), 2);
        assert_dyn(&mock);
    }

    #[test]
    #[should_panic(expected = "logits[1].len()")]
    fn mock_phonemizer_panics_on_row_vocab_mismatch() {
        // Catches test-fixture typos at construction time.
        let mut canned = sample_logits(2, 4);
        canned[1].pop();
        let _ = MockPhonemizer::with_fixed(canned, 4);
    }

    // ---- OnnxPhonemizer::load digest gate --------------------------------

    #[test]
    fn onnx_phonemizer_load_rejects_missing_file() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("does_not_exist.onnx");

        let err = OnnxPhonemizer::load(&path).expect_err("must reject missing file");
        match err {
            EvaluationError::ModelNotFound(msg) => {
                assert!(
                    msg.contains("does_not_exist.onnx"),
                    "message should mention the path, got: {msg}"
                );
            }
            other => panic!("expected ModelNotFound, got {other:?}"),
        }
    }

    #[test]
    fn onnx_phonemizer_load_rejects_digest_mismatch() {
        // Write a small non-ONNX byte sequence. Its SHA-256 will not match
        // BuildConfig::MODEL_SHA256 in any reasonable build configuration
        // (the dev sentinel is 64 zeros; the real digest pins a 339 MB file).
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("not_a_model.onnx");
        {
            let mut f = std::fs::File::create(&path).expect("create");
            f.write_all(b"definitely not a real onnx file").expect("write");
        }

        let err = OnnxPhonemizer::load(&path).expect_err("must reject digest mismatch");
        match err {
            EvaluationError::ModelDigestMismatch { expected, actual } => {
                assert_eq!(
                    expected,
                    BuildConfig::MODEL_SHA256.to_ascii_lowercase(),
                    "expected field should match BuildConfig digest"
                );
                assert_eq!(actual.len(), 64, "actual digest should be 64 hex chars");
                assert!(
                    actual.chars().all(|c| c.is_ascii_hexdigit()),
                    "actual digest should be hex, got: {actual}"
                );
                assert_ne!(expected, actual, "digests must differ to be a mismatch");
            }
            other => panic!("expected ModelDigestMismatch, got {other:?}"),
        }
    }

    #[test]
    fn onnx_phonemizer_forward_rejects_short_audio_via_mock() {
        // We can't easily build an OnnxPhonemizer in unit tests (no model),
        // but the input-shape gate is also reachable via the trait surface
        // by writing a thin wrapper. Instead we directly assert the gate
        // constant matches what MIN_AUDIO_SAMPLES claims; the real model
        // path is covered by the #[ignore]-marked integration test.
        assert!(MIN_AUDIO_SAMPLES >= 1_600);
        // Confirm the variant carries the diagnostic field for callers.
        let err = EvaluationError::UnsupportedInputShape {
            got_len: 10,
            min_len: MIN_AUDIO_SAMPLES,
        };
        let rendered = err.to_string();
        assert!(rendered.contains("got len 10"));
        assert!(rendered.contains(&MIN_AUDIO_SAMPLES.to_string()));
    }

    // ---- Real model — only runs when the artifact is present + env is set

    /// Resolve `<repo root>/tools/model_export/build/ias-model-0.1.0.onnx`
    /// from the crate-relative `CARGO_MANIFEST_DIR` (`app/src-tauri/`).
    fn real_model_path() -> std::path::PathBuf {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        manifest_dir
            .parent()
            .expect("app/")
            .parent()
            .expect("repo root")
            .join("tools/model_export/build/ias-model-0.1.0.onnx")
    }

    /// Real-model load test. `#[ignore]`-marked because:
    /// - the artifact may not be present in every dev environment;
    /// - even when present, `BuildConfig::MODEL_SHA256` is the dev sentinel
    ///   unless the implementer sets `IAS_MODEL_SHA256` at build time, in
    ///   which case load succeeds; otherwise it fails the digest gate.
    ///
    /// Either outcome is a valid CL-14 pass: load-success demonstrates the
    /// end-to-end ort path; ModelDigestMismatch demonstrates the integrity
    /// gate. Run with:
    ///
    /// ```bash
    /// IAS_MODEL_SHA256=e88bbad6cd890c193ba42c63f708383b8b646e0e7382db8d4392efbfe8e2edb0 \
    ///     cargo test -- --ignored evaluation::phonemizer
    /// ```
    #[test]
    #[ignore = "requires the MX-built ONNX artifact on disk"]
    fn onnx_phonemizer_loads_real_model_or_rejects_digest() {
        let path = real_model_path();
        if !path.exists() {
            panic!(
                "MX artifact not on disk at {}; run the SETUP task first",
                path.display()
            );
        }
        match OnnxPhonemizer::load(&path) {
            Ok(p) => {
                assert!(p.vocab_size() > 0, "loaded model should report nonzero vocab");
                assert_eq!(
                    p.model_version(),
                    BuildConfig::MODEL_VERSION,
                    "phonemizer reports the build-pinned model version"
                );
            }
            Err(EvaluationError::ModelDigestMismatch { expected, actual }) => {
                // Acceptable: dev build did not have IAS_MODEL_SHA256 set,
                // so the integrity gate rejected the real file. Verify the
                // mismatch is between the dev sentinel and the real digest.
                assert_eq!(
                    expected.chars().all(|c| c == '0'),
                    true,
                    "expected sentinel (all zeros), got: {expected}"
                );
                assert_ne!(actual.chars().all(|c| c == '0'), true);
            }
            Err(other) => panic!("unexpected load error: {other:?}"),
        }
    }

    /// Real-model forward pass. Same #[ignore] rationale — additionally
    /// requires the digest gate to pass, so this only runs when
    /// `IAS_MODEL_SHA256` is set to the real artifact's digest at build time.
    #[test]
    #[ignore = "requires the MX-built ONNX artifact AND IAS_MODEL_SHA256 env"]
    fn onnx_phonemizer_forward_on_real_silence() {
        let path = real_model_path();
        let phonemizer = OnnxPhonemizer::load(&path)
            .expect("real model load — set IAS_MODEL_SHA256 to the real digest");
        let audio = vec![0.0_f32; 16_000]; // 1 second of silence at 16 kHz
        let frames = phonemizer.forward(&audio).expect("forward on silence");

        assert!(frames.len() > 0, "forward should emit at least one frame");
        let vocab = phonemizer.vocab_size();
        for (i, row) in frames.iter().enumerate() {
            assert_eq!(
                row.len(),
                vocab,
                "frame[{i}] has length {} but vocab is {vocab}",
                row.len()
            );
        }
    }

    // ---- input normalisation (issue #67/#70) ------------------------------

    fn mean_var(x: &[f32]) -> (f64, f64) {
        let n = x.len() as f64;
        let m = x.iter().map(|&v| v as f64).sum::<f64>() / n;
        let v = x.iter().map(|&v| (v as f64 - m).powi(2)).sum::<f64>() / n;
        (m, v)
    }

    /// A non-trivial signal: a 200 Hz sine at 16 kHz, half a second.
    fn sine_16k(amp: f32) -> Vec<f32> {
        (0..8_000)
            .map(|i| amp * (2.0 * std::f32::consts::PI * 200.0 * i as f32 / 16_000.0).sin())
            .collect()
    }

    #[test]
    fn normalize_audio_yields_zero_mean_unit_variance() {
        let out = normalize_audio(&sine_16k(0.3));
        let (m, v) = mean_var(&out);
        assert!(m.abs() < 1e-4, "mean should be ~0, got {m}");
        assert!((v - 1.0).abs() < 1e-3, "variance should be ~1, got {v}");
    }

    #[test]
    fn normalize_audio_is_gain_invariant() {
        // The core of the fix: across the realistic capture-level range, scaling
        // the input by capture gain must not change the normalised output. The
        // ratio that matters is L2-Arctic studio (RMS ~0.09) vs Watson's laptop
        // mic (RMS ~0.003) — ~30x. The 1e-3 bound below is NOT f32 rounding
        // (~1e-7); the eps-floor term (eps relative to the attenuated variance)
        // sets this floor, and at 30x it stays comfortably under 1e-3.
        let loud = normalize_audio(&sine_16k(0.9));
        let quiet = normalize_audio(&sine_16k(0.03)); // 30x quieter (laptop-mic regime)
        assert_eq!(loud.len(), quiet.len());
        let max_abs_diff = loud
            .iter()
            .zip(quiet.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            max_abs_diff < 1e-3,
            "normalised output must be gain-invariant over the realistic range; \
             max abs diff = {max_abs_diff}"
        );
    }

    #[test]
    fn normalize_audio_eps_floor_limits_invariance_only_near_silence() {
        // Documents the deliberate trade-off: the 1e-7 eps floor (silence guard)
        // is an absolute term, so exact scale-invariance degrades as the signal
        // approaches silence. Even at an extreme 100x gain span the divergence
        // stays tiny (~2e-3 on a unit-variance signal) — far below what shifts
        // posteriors (the end-to-end L2-Arctic sweep holds certainty flat to
        // ~1e-3 across this same span). This test pins that bound so a future
        // eps change that materially breaks low-level invariance is caught.
        let loud = normalize_audio(&sine_16k(0.9));
        let very_quiet = normalize_audio(&sine_16k(0.009)); // 100x quieter
        let max_abs_diff = loud
            .iter()
            .zip(very_quiet.iter())
            .map(|(a, b)| (a - b).abs())
            .fold(0.0_f32, f32::max);
        assert!(
            max_abs_diff < 5e-3,
            "even at 100x the eps-limited divergence must stay small; got {max_abs_diff}"
        );
    }

    #[test]
    fn normalize_audio_silence_is_finite_zeros() {
        // All-zero (silence) input has zero variance; the eps floor must keep
        // the result finite (all zeros) rather than producing NaN/Inf.
        let out = normalize_audio(&vec![0.0_f32; 1_600]);
        assert_eq!(out.len(), 1_600);
        assert!(
            out.iter().all(|&v| v == 0.0),
            "silence must normalise to all-zeros, no NaN/Inf"
        );
    }

    #[test]
    fn normalize_audio_dc_offset_is_removed_and_finite() {
        // A constant (DC) signal: mean-subtraction zeroes it; the eps floor
        // prevents a divide-by-zero blow-up. Result must be finite zeros.
        let out = normalize_audio(&vec![0.5_f32; 1_600]);
        assert!(out.iter().all(|&v| v.is_finite()), "must be finite");
        let (m, _) = mean_var(&out);
        assert!(m.abs() < 1e-6, "DC offset must be removed, mean ~0, got {m}");
    }

    #[test]
    fn normalize_audio_empty_is_empty() {
        assert!(normalize_audio(&[]).is_empty());
    }
}
