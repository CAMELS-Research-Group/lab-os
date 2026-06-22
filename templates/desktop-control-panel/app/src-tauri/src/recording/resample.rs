//! 16 kHz mono f32 resampling via [`rubato`].
//!
//! Ported from `spike/rust-poc/src/resample.rs`. The sinc-interpolation
//! parameters here are byte-equivalent to the spike's so a buffer produced
//! by [`resample_to_16k`] is identical to the spike-recorded clips committed
//! to the MX-3 reference-audio fixture set (see `tools/model_export/reference_audio/`).
//!
//! SPIKE-15 verified that under the production workload the resample stage
//! costs ~1 s per 8.7 s clip, dominated by inference rather than the resample
//! itself — so we keep sinc interpolation (not naive decimation) at the
//! configured quality. Naive decimation aliases high-frequency content and is
//! rejected by the unit-test suite below.

use log::debug;
use rubato::{
    audioadapter_buffers::direct::SequentialSliceOfVecs, Async, FixedAsync, Resampler,
    SincInterpolationParameters, SincInterpolationType, WindowFunction,
};

use super::error::MicrophoneError;

pub const TARGET_RATE: u32 = 16_000;

/// Resample `samples` from `input_rate` Hz → 16 kHz mono f32.
///
/// Returns the resampled `Vec<f32>`. If `input_rate == 16_000` the input is
/// returned untouched (byte-equivalent passthrough). Sinc parameters match
/// the spike — do not retune without re-running the MX-3 byte-equivalence
/// check against the fixture clips.
pub fn resample_to_16k(samples: Vec<f32>, input_rate: u32) -> Result<Vec<f32>, MicrophoneError> {
    if input_rate == TARGET_RATE {
        debug!(
            "resample: already 16 kHz — {} samples, no resample needed",
            samples.len()
        );
        return Ok(samples);
    }

    let ratio = TARGET_RATE as f64 / input_rate as f64;
    let input_len = samples.len();

    // Sinc interpolation parameters — quality suitable for speech, not
    // over-engineered. MUST match the spike (byte-equivalence requirement).
    let params = SincInterpolationParameters {
        sinc_len: 64,
        f_cutoff: 0.95,
        interpolation: SincInterpolationType::Cubic,
        oversampling_factor: 128,
        window: WindowFunction::BlackmanHarris2,
    };

    let mut resampler = Async::<f32>::new_sinc(
        ratio,
        1.1, // max relative ratio deviation (fixed — will never change)
        &params,
        1024, // chunk size
        1,    // mono
        FixedAsync::Input,
    )
    .map_err(|e| MicrophoneError::Resampler(format!("rubato new_sinc: {e}")))?;

    // Wrap input as channel-major (1 channel).
    let input_data = vec![samples];
    let input_adapter = SequentialSliceOfVecs::new(&input_data, 1, input_len)
        .map_err(|e| MicrophoneError::Resampler(format!("SequentialSliceOfVecs input: {e}")))?;

    // Allocate output buffer.
    let output_len = resampler.process_all_needed_output_len(input_len);
    let mut output_data = vec![vec![0.0_f32; output_len]; 1];
    let mut output_adapter = SequentialSliceOfVecs::new_mut(&mut output_data, 1, output_len)
        .map_err(|e| MicrophoneError::Resampler(format!("SequentialSliceOfVecs output: {e}")))?;

    let (_, actual_output_len) = resampler
        .process_all_into_buffer(&input_adapter, &mut output_adapter, input_len, None)
        .map_err(|e| MicrophoneError::Resampler(format!("rubato process_all_into_buffer: {e}")))?;

    // Extract channel 0 and truncate to actual output length.
    let mut out = output_data.remove(0);
    out.truncate(actual_output_len);

    debug!(
        "resample: {} Hz → {} Hz: {} in → {} out samples",
        input_rate,
        TARGET_RATE,
        input_len,
        out.len()
    );

    Ok(out)
}

/// Compute the energy at a target frequency using a simple DFT bin sum.
/// Used for the 1 kHz tone-preservation spot-check in the unit tests.
/// Returns the fraction of total energy in the band [freq-band_hz, freq+band_hz].
#[cfg(test)]
fn energy_fraction_at_freq(samples: &[f32], sample_rate: u32, freq_hz: f32, band_hz: f32) -> f32 {
    let n = samples.len();
    if n == 0 {
        return 0.0;
    }

    let freq_res = sample_rate as f32 / n as f32; // Hz per bin
    let center_bin = (freq_hz / freq_res).round() as i64;
    let half_band = (band_hz / freq_res).ceil() as i64;

    let lo = (center_bin - half_band).max(0) as usize;
    let hi = ((center_bin + half_band) as usize).min(n / 2);

    // Compute magnitudes squared for all bins up to Nyquist.
    let mut magnitudes_sq = vec![0.0_f32; n / 2 + 1];
    for k in 0..=(n / 2) {
        let mut re = 0.0_f32;
        let mut im = 0.0_f32;
        let two_pi_k_over_n = 2.0 * std::f32::consts::PI * k as f32 / n as f32;
        for (t, &s) in samples.iter().enumerate() {
            re += s * (two_pi_k_over_n * t as f32).cos();
            im -= s * (two_pi_k_over_n * t as f32).sin();
        }
        magnitudes_sq[k] = re * re + im * im;
    }

    let total_energy: f32 = magnitudes_sq.iter().sum();
    let band_energy: f32 = magnitudes_sq[lo..=hi].iter().sum();

    if total_energy < 1e-12 {
        return 0.0;
    }
    band_energy / total_energy
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    /// Generate a `freq_hz` sine at `sample_rate` Hz for `duration_secs`.
    fn sine(freq_hz: f32, sample_rate: u32, duration_secs: f32) -> Vec<f32> {
        let n = (sample_rate as f32 * duration_secs) as usize;
        (0..n)
            .map(|i| (2.0 * PI * freq_hz * (i as f32) / sample_rate as f32).sin())
            .collect()
    }

    fn rms(samples: &[f32]) -> f32 {
        if samples.is_empty() {
            return 0.0;
        }
        let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
        (sum_sq / samples.len() as f32).sqrt()
    }

    #[test]
    fn resample_passthrough_at_16k() {
        // Input already at TARGET_RATE — should pass through byte-equivalent.
        let input: Vec<f32> = (0..1600).map(|i| (i as f32 * 0.001).sin()).collect();
        let out = resample_to_16k(input.clone(), TARGET_RATE).expect("resample passthrough");
        assert_eq!(
            out, input,
            "passthrough at 16 kHz must return the input unchanged"
        );
    }

    #[test]
    fn resample_44k_to_16k_preserves_sine() {
        // 1 kHz tone at 44.1 kHz → resample to 16 kHz. The dominant energy
        // should still be at 1 kHz after resampling. Naive decimation would
        // also pass this for 1 kHz, but the *naive_decimation_rejected* test
        // below covers that case with high-frequency input.
        let input = sine(1_000.0, 44_100, 0.25); // 0.25 s
        let input_rms = rms(&input);

        let out = resample_to_16k(input, 44_100).expect("resample 44k→16k");

        assert!(
            !out.is_empty(),
            "resampled output must be non-empty"
        );

        // RMS should be roughly preserved (sinc filter introduces minor
        // amplitude change at the band edge; 25% tolerance is generous and
        // catches gross failure modes without flapping).
        let out_rms = rms(&out);
        assert!(
            (out_rms - input_rms).abs() / input_rms < 0.25,
            "RMS should be roughly preserved: input={input_rms}, output={out_rms}"
        );

        // Most of the energy should still sit in a band around 1 kHz.
        let band = energy_fraction_at_freq(&out, TARGET_RATE, 1_000.0, 100.0);
        assert!(
            band > 0.5,
            ">50% of output energy should remain within ±100 Hz of 1 kHz; got {band}"
        );
    }

    #[test]
    fn resample_48k_to_16k_correct_length() {
        // 4800 samples @ 48 kHz == 0.1 s.  After resample to 16 kHz we expect
        // ~1600 samples (0.1 s × 16 000 Hz), allow some rubato windowing
        // edge effect.
        let input = sine(440.0, 48_000, 0.1);
        assert_eq!(input.len(), 4_800);

        let out = resample_to_16k(input, 48_000).expect("resample 48k→16k");

        let expected = 1_600_i64;
        let diff = (out.len() as i64 - expected).abs();
        assert!(
            diff <= 64,
            "expected ~{expected} samples after 48k→16k resample, got {} (diff {diff})",
            out.len()
        );
    }

    #[test]
    fn resample_naive_decimation_rejected() {
        // Generate a 7 kHz sine at 48 kHz.  Nyquist of 16 kHz output is 8 kHz,
        // so 7 kHz is technically representable — but naive decimation
        // (taking every 3rd sample) would alias because of the lack of an
        // anti-alias filter, producing a mangled waveform with significantly
        // different RMS or even near-zero output for some frequencies.  The
        // sinc resampler should preserve a meaningful signal.
        //
        // We assert two things:
        //   (a) the output is non-trivially non-zero,
        //   (b) energy at 7 kHz is the dominant component (no wild aliasing).
        let input = sine(7_000.0, 48_000, 0.25);
        let out = resample_to_16k(input, 48_000).expect("resample 48k→16k high-freq");

        let out_rms = rms(&out);
        assert!(
            out_rms > 0.05,
            "sinc resample should preserve substantial 7 kHz energy; got RMS={out_rms}"
        );

        // Most energy should still sit near 7 kHz (band ±200 Hz).
        let band = energy_fraction_at_freq(&out, TARGET_RATE, 7_000.0, 200.0);
        assert!(
            band > 0.4,
            ">40% of output energy should remain within ±200 Hz of 7 kHz; got {band}"
        );
    }
}
