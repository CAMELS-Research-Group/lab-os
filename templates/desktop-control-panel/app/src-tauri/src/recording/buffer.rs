//! In-memory audio buffer at the rate this module commits to (16 kHz mono f32).
//!
//! `AudioBuffer` is the library-form output of [`crate::recording::CpalAdapter::stop`]:
//! capture samples have been downmixed to mono and resampled to 16 kHz. The
//! type holds no rate field — the invariant is encoded in the constructor
//! visibility: only the `recording` module can build an `AudioBuffer` from
//! raw samples via [`AudioBuffer::from_samples`], so any `AudioBuffer` the
//! rest of the app sees has been through the resample-to-16kHz path.

/// Sample rate of every `AudioBuffer` returned by this module.
pub const SAMPLE_RATE: u32 = 16_000;

#[derive(Debug, Clone, Default)]
pub struct AudioBuffer {
    samples: Vec<f32>,
}

impl AudioBuffer {
    /// Constructs an empty buffer. Useful as a sentinel before capture starts.
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    /// Constructs from raw 16 kHz mono f32 samples. Visible only inside the
    /// `recording` module so callers outside it cannot bypass the resampler.
    pub(super) fn from_samples(samples: Vec<f32>) -> Self {
        Self { samples }
    }

    /// Borrow the underlying samples. They are mono f32 at [`SAMPLE_RATE`].
    pub fn samples(&self) -> &[f32] {
        &self.samples
    }

    /// Sample rate is fixed at [`SAMPLE_RATE`] by construction.
    pub fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    /// Duration in seconds, computed from the sample count.
    pub fn duration_seconds(&self) -> f64 {
        self.samples.len() as f64 / SAMPLE_RATE as f64
    }

    /// `true` when the buffer holds no samples.
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// Number of samples (i.e. frames; mono).
    pub fn len(&self) -> usize {
        self.samples.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn audiobuffer_new_is_empty() {
        let buf = AudioBuffer::new();
        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.duration_seconds(), 0.0);
    }

    #[test]
    fn audiobuffer_is_empty_reports_correctly() {
        let empty = AudioBuffer::from_samples(Vec::new());
        assert!(empty.is_empty(), "empty buffer should report is_empty == true");

        let full = AudioBuffer::from_samples(vec![0.1, 0.2, 0.3]);
        assert!(!full.is_empty(), "non-empty buffer should report is_empty == false");
        assert_eq!(full.len(), 3);
    }

    #[test]
    fn audiobuffer_duration_seconds_correct() {
        // 16 000 samples @ 16 kHz == 1.0 s exactly.
        let one_second = AudioBuffer::from_samples(vec![0.0; SAMPLE_RATE as usize]);
        assert_eq!(one_second.duration_seconds(), 1.0);

        // Half a second.
        let half_second = AudioBuffer::from_samples(vec![0.0; (SAMPLE_RATE / 2) as usize]);
        assert_eq!(half_second.duration_seconds(), 0.5);
    }

    #[test]
    fn audiobuffer_sample_rate_is_16k() {
        let buf = AudioBuffer::new();
        assert_eq!(buf.sample_rate(), 16_000);
        assert_eq!(SAMPLE_RATE, 16_000);
    }

    #[test]
    fn audiobuffer_samples_accessor_returns_inserted_data() {
        let data = vec![0.25_f32, -0.5, 0.75];
        let buf = AudioBuffer::from_samples(data.clone());
        assert_eq!(buf.samples(), data.as_slice());
    }
}
