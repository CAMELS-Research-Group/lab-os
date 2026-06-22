//! Greedy CTC decoder that retains the per-frame posterior distribution.
//!
//! Spec: ADD §3.1 (evaluation feature), TRD T-INF-4 → T-INF-6.
//!
//! # Why retain posteriors
//!
//! CTC argmax alone produces only the most-likely token at each frame, which
//! is enough to recover the IPA sequence but discards the confidence signal
//! the alignment + certainty stage (CL-17) depends on. This decoder therefore
//! returns **both** the collapsed token sequence **and** the per-frame softmax
//! distribution, so downstream consumers can index the column corresponding to
//! any reference phoneme without re-running the model.
//!
//! # Reference parity
//!
//! The token-sequence path mirrors the Python reference in
//! `tools/model_export/exporter/validate.py::ctc_greedy_decode` byte-for-byte
//! so client-side decoding agrees with the validation that gated the bundled
//! ONNX artifact. Two invariants are load-bearing:
//!
//! 1. **Argmax ties resolve to the lowest index** — matches `np.argmax`'s
//!    first-occurrence rule. We implement this with a strict `>` comparison.
//! 2. **Blank tokens reset the repeat-collapse state** — `[A, blank, A]`
//!    decodes to `[A, A]`, not `[A]`. The Python reference comment makes this
//!    explicit; we mirror it here.

use crate::evaluation::error::EvaluationError;
use crate::evaluation::phonemizer::Logits;

/// Output of [`ctc_greedy_decode`]: the collapsed CTC token sequence and the
/// per-frame softmax distribution over the full vocabulary.
///
/// `posteriors` has shape `(T, vocab_len)` — the same width as the input
/// `logits`. CL-17 picks columns by reference-phoneme id, so retaining the
/// full distribution (rather than a top-k) avoids needing to re-run inference
/// when alignment chooses which columns matter.
#[derive(Debug, Clone, PartialEq)]
pub struct CtcDecodeOutput {
    /// Decoded IPA token ids after argmax → collapse-consecutive-repeats →
    /// remove-blanks. Empty for silence / all-blank input.
    pub tokens: Vec<usize>,

    /// Per-frame softmax over the vocabulary; `posteriors[t][v]` is the
    /// probability mass the model assigns to vocab id `v` at frame `t`.
    /// Each inner row sums to 1.0 within f32 tolerance.
    pub posteriors: Vec<Vec<f32>>,
}

/// Greedy CTC decode that retains the per-frame posterior distribution.
///
/// # Parameters
///
/// * `logits` — slice of per-frame logit vectors. Each frame's length must
///   equal `vocab.len()`.
/// * `vocab` — IPA vocabulary; only its length is consulted (for shape
///   validation and to size the posterior buffer).
/// * `blank_idx` — CTC blank token id. Must be `< vocab.len()`.
///
/// # Returns
///
/// A [`CtcDecodeOutput`] carrying the collapsed token sequence and the
/// per-frame softmax distribution. An empty input returns empty `tokens` and
/// empty `posteriors`.
///
/// # Errors
///
/// Returns [`EvaluationError::BlankIndexOutOfRange`] if `blank_idx >= vocab.len()`.
/// Returns [`EvaluationError::CtcShapeMismatch`] if any frame's length differs
/// from `vocab.len()`, or if `vocab` is empty while `logits` is non-empty.
/// This mirrors the `ValueError` raised by the Python reference on malformed
/// input.
pub fn ctc_greedy_decode(
    logits: &[Logits],
    vocab: &[String],
    blank_idx: usize,
) -> Result<CtcDecodeOutput, EvaluationError> {
    let vocab_len = vocab.len();

    if logits.is_empty() {
        return Ok(CtcDecodeOutput {
            tokens: Vec::new(),
            posteriors: Vec::new(),
        });
    }

    if vocab_len == 0 {
        return Err(EvaluationError::CtcShapeMismatch {
            vocab_len: 0,
            frame_len: logits[0].len(),
            frame_index: 0,
        });
    }

    if blank_idx >= vocab_len {
        return Err(EvaluationError::BlankIndexOutOfRange {
            blank_idx,
            vocab_len,
        });
    }

    let mut tokens: Vec<usize> = Vec::new();
    let mut posteriors: Vec<Vec<f32>> = Vec::with_capacity(logits.len());
    let mut prev: Option<usize> = None;

    for (t, frame) in logits.iter().enumerate() {
        if frame.len() != vocab_len {
            return Err(EvaluationError::CtcShapeMismatch {
                vocab_len,
                frame_len: frame.len(),
                frame_index: t,
            });
        }

        let argmax = argmax_lowest_index(frame);
        posteriors.push(softmax(frame));

        if argmax == blank_idx {
            // Blank resets the repeat-collapse state — see module docs.
            prev = None;
            continue;
        }
        if Some(argmax) != prev {
            tokens.push(argmax);
        }
        prev = Some(argmax);
    }

    Ok(CtcDecodeOutput { tokens, posteriors })
}

/// Argmax with first-occurrence tie-break — matches `np.argmax`.
///
/// Strict `>` ensures an equal value does NOT displace the running best, so
/// the lowest index wins when multiple positions share the maximum logit.
fn argmax_lowest_index(frame: &[f32]) -> usize {
    let mut best_idx = 0usize;
    let mut best_val = frame[0];
    for (i, &v) in frame.iter().enumerate().skip(1) {
        if v > best_val {
            best_val = v;
            best_idx = i;
        }
    }
    best_idx
}

/// Numerically stable softmax: `exp(x - max) / sum(exp(x - max))`.
fn softmax(frame: &[f32]) -> Vec<f32> {
    let max = frame
        .iter()
        .copied()
        .fold(f32::NEG_INFINITY, |acc, x| if x > acc { x } else { acc });
    let mut out: Vec<f32> = frame.iter().map(|&x| (x - max).exp()).collect();
    let sum: f32 = out.iter().sum();
    if sum > 0.0 {
        for v in out.iter_mut() {
            *v /= sum;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // Helpers ---------------------------------------------------------------

    /// Build a vocab of N placeholder strings; the decoder only uses `len()`.
    fn vocab(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("v{i}")).collect()
    }

    /// One-hot frame: argmax == `which`, all other positions a uniform low.
    fn one_hot(vocab_len: usize, which: usize) -> Vec<f32> {
        let mut f = vec![-10.0_f32; vocab_len];
        f[which] = 10.0;
        f
    }

    /// Argmax of a posterior row — used to assert the decoded-token /
    /// posterior-argmax agreement invariant.
    fn argmax(row: &[f32]) -> usize {
        argmax_lowest_index(row)
    }

    // 1. Collapse repeats ---------------------------------------------------

    #[test]
    fn collapses_consecutive_duplicates() {
        // [1, 1, 2, 2, 2, 1] with blank=0 -> [1, 2, 1]
        let vocab = vocab(4);
        let logits: Vec<Logits> = vec![
            one_hot(4, 1),
            one_hot(4, 1),
            one_hot(4, 2),
            one_hot(4, 2),
            one_hot(4, 2),
            one_hot(4, 1),
        ];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();
        assert_eq!(out.tokens, vec![1, 2, 1]);
    }

    // 2. Blank removal ------------------------------------------------------

    #[test]
    fn removes_blanks() {
        // [1, blank, 2] -> [1, 2]
        let vocab = vocab(4);
        let logits: Vec<Logits> = vec![one_hot(4, 1), one_hot(4, 0), one_hot(4, 2)];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();
        assert_eq!(out.tokens, vec![1, 2]);
    }

    // 3. All-blank -> empty -------------------------------------------------

    #[test]
    fn all_blank_yields_empty_tokens() {
        let vocab = vocab(4);
        let logits: Vec<Logits> = vec![one_hot(4, 0); 6];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();
        assert!(out.tokens.is_empty());
        // Posteriors are still retained for every frame.
        assert_eq!(out.posteriors.len(), 6);
    }

    // 4. Alternating tokens (no collapse across non-equal) -----------------

    #[test]
    fn alternating_tokens_are_preserved() {
        let vocab = vocab(4);
        let logits: Vec<Logits> = vec![
            one_hot(4, 1),
            one_hot(4, 2),
            one_hot(4, 1),
            one_hot(4, 2),
        ];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();
        assert_eq!(out.tokens, vec![1, 2, 1, 2]);
    }

    // 5. Blank resets repeat state -- the load-bearing CTC invariant ------

    #[test]
    fn blank_resets_repeat_state() {
        // [A, blank, A] -> [A, A], NOT [A]. If this is wrong, downstream
        // alignment is wrong.
        let vocab = vocab(4);
        let logits: Vec<Logits> = vec![one_hot(4, 1), one_hot(4, 0), one_hot(4, 1)];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();
        assert_eq!(out.tokens, vec![1, 1]);
    }

    // 6. Non-duplicate transitions preserved -------------------------------

    #[test]
    fn distinct_tokens_pass_through() {
        let vocab = vocab(4);
        let logits: Vec<Logits> = vec![one_hot(4, 1), one_hot(4, 2), one_hot(4, 3)];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();
        assert_eq!(out.tokens, vec![1, 2, 3]);
    }

    // 7. Posteriors are valid distributions --------------------------------

    #[test]
    fn posteriors_are_valid_distributions() {
        // Asymmetric, arbitrary logits — not one-hot.
        let vocab = vocab(5);
        let logits: Vec<Logits> = vec![
            vec![0.1, 1.2, -0.3, 0.7, 2.5],
            vec![-1.0, 0.0, 3.0, 0.5, 0.5],
            vec![2.0, 2.0, 1.0, 0.0, -1.0],
        ];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();

        assert_eq!(out.posteriors.len(), logits.len());
        for (t, row) in out.posteriors.iter().enumerate() {
            assert_eq!(row.len(), 5);
            let sum: f32 = row.iter().sum();
            assert!((sum - 1.0).abs() < 1e-5, "frame {t} sum={sum} not ~1.0");
            // argmax-of-posterior matches argmax-of-logits.
            assert_eq!(argmax(row), argmax_lowest_index(&logits[t]));
        }
    }

    // 8. Posterior argmax agrees with decoded tokens -----------------------

    #[test]
    fn posterior_argmax_collapses_to_decoded_tokens() {
        // Same fixture as test 7 plus a few more frames mixing blanks and
        // repeats so the agreement invariant gets a non-trivial path.
        let vocab = vocab(5);
        let logits: Vec<Logits> = vec![
            vec![0.1, 1.2, -0.3, 0.7, 2.5], // argmax 4
            vec![0.1, 1.2, -0.3, 0.7, 2.5], // argmax 4 (collapse)
            vec![5.0, 0.0, 0.0, 0.0, 0.0],  // argmax 0 (blank, reset)
            vec![0.1, 1.2, -0.3, 0.7, 2.5], // argmax 4 (re-emit)
            vec![-1.0, 0.0, 0.0, 3.0, 0.5], // argmax 3
        ];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();

        // Manually re-decode from the retained posteriors and compare.
        let mut redecoded: Vec<usize> = Vec::new();
        let mut prev: Option<usize> = None;
        for row in &out.posteriors {
            let a = argmax(row);
            if a == 0 {
                prev = None;
                continue;
            }
            if Some(a) != prev {
                redecoded.push(a);
            }
            prev = Some(a);
        }
        assert_eq!(redecoded, out.tokens);
        assert_eq!(out.tokens, vec![4, 4, 3]);
    }

    // 9. Tie-break = lowest index ------------------------------------------

    #[test]
    fn argmax_tie_break_uses_lowest_index() {
        // Two indices share the max logit; the lower index must win.
        let vocab = vocab(4);
        let frame = vec![0.0, 5.0, 5.0, 1.0];
        let logits: Vec<Logits> = vec![frame.clone()];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();
        assert_eq!(out.tokens, vec![1], "lowest tied index (1) must win, not 2");

        // Also exercise tie at the blank position vs a real token.
        let blank_tie = vec![5.0, 5.0, 0.0, 0.0];
        let out2 = ctc_greedy_decode(&[blank_tie], &vocab, 0).unwrap();
        assert!(out2.tokens.is_empty(), "blank wins the tie at index 0");
    }

    // 10. Fixture parity with the Python reference -------------------------

    /// Hand-verified parity with `ctc_greedy_decode` in
    /// `tools/model_export/exporter/validate.py`. Equivalent Python:
    ///
    /// ```python
    /// import numpy as np
    /// from exporter.validate import ctc_greedy_decode
    /// logits = np.array([
    ///     [5.0, 0.1, 0.2, 0.3],  # argmax 0 (blank)
    ///     [0.1, 4.0, 0.2, 0.3],  # argmax 1
    ///     [0.1, 4.0, 0.2, 0.3],  # argmax 1 (collapse)
    ///     [5.0, 0.1, 0.2, 0.3],  # argmax 0 (blank, resets prev)
    ///     [0.1, 4.0, 0.2, 0.3],  # argmax 1 (re-emit after blank)
    ///     [0.1, 0.2, 4.0, 0.3],  # argmax 2
    ///     [0.1, 0.2, 0.3, 4.0],  # argmax 3
    ///     [5.0, 0.1, 0.2, 0.3],  # argmax 0 (blank)
    /// ])
    /// ctc_greedy_decode(logits)  # -> [1, 1, 2, 3]
    /// ```
    #[test]
    fn matches_python_reference_on_fixture() {
        let vocab = vocab(4);
        let logits: Vec<Logits> = vec![
            vec![5.0, 0.1, 0.2, 0.3],
            vec![0.1, 4.0, 0.2, 0.3],
            vec![0.1, 4.0, 0.2, 0.3],
            vec![5.0, 0.1, 0.2, 0.3],
            vec![0.1, 4.0, 0.2, 0.3],
            vec![0.1, 0.2, 4.0, 0.3],
            vec![0.1, 0.2, 0.3, 4.0],
            vec![5.0, 0.1, 0.2, 0.3],
        ];
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();
        assert_eq!(out.tokens, vec![1, 1, 2, 3]);
        assert_eq!(out.posteriors.len(), 8);
        for row in &out.posteriors {
            let s: f32 = row.iter().sum();
            assert!((s - 1.0).abs() < 1e-5);
        }
    }

    // Shape-validation paths -----------------------------------------------

    #[test]
    fn empty_logits_yields_empty_output() {
        let vocab = vocab(4);
        let logits: Vec<Logits> = Vec::new();
        let out = ctc_greedy_decode(&logits, &vocab, 0).unwrap();
        assert!(out.tokens.is_empty());
        assert!(out.posteriors.is_empty());
    }

    #[test]
    fn frame_width_mismatch_returns_error() {
        let vocab = vocab(4);
        let logits: Vec<Logits> = vec![vec![0.0; 4], vec![0.0; 3]];
        let err = ctc_greedy_decode(&logits, &vocab, 0).unwrap_err();
        match err {
            EvaluationError::CtcShapeMismatch {
                vocab_len,
                frame_len,
                frame_index,
            } => {
                assert_eq!(vocab_len, 4);
                assert_eq!(frame_len, 3);
                assert_eq!(frame_index, 1);
            }
            other => panic!("expected CtcShapeMismatch, got {other:?}"),
        }
    }

    #[test]
    fn blank_idx_out_of_range_returns_error() {
        let vocab = vocab(4);
        let logits: Vec<Logits> = vec![vec![0.0; 4]];
        let err = ctc_greedy_decode(&logits, &vocab, 4).unwrap_err();
        match err {
            EvaluationError::BlankIndexOutOfRange {
                blank_idx,
                vocab_len,
            } => {
                assert_eq!(blank_idx, 4);
                assert_eq!(vocab_len, 4);
            }
            other => panic!("expected BlankIndexOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn empty_vocab_with_non_empty_logits_returns_shape_mismatch() {
        // Non-empty logits against a zero-length vocab cannot be decoded; the
        // pre-loop guard reports vocab_len=0 and points at the first frame.
        let vocab: Vec<String> = Vec::new();
        let logits: Vec<Logits> = vec![vec![0.0; 3]];
        let err = ctc_greedy_decode(&logits, &vocab, 0).unwrap_err();
        match err {
            EvaluationError::CtcShapeMismatch {
                vocab_len,
                frame_len,
                frame_index,
            } => {
                assert_eq!(vocab_len, 0);
                assert_eq!(frame_len, 3);
                assert_eq!(frame_index, 0);
            }
            other => panic!("expected CtcShapeMismatch, got {other:?}"),
        }
    }
}
