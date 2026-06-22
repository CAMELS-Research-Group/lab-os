//! Cross-platform audio capture via [`cpal`].
//!
//! Ported from `spike/rust-poc/src/recorder.rs`, restructured from a CLI
//! binary into a library type. The audio-processing path (mono downmix,
//! buffer accumulation, resample-to-16kHz at stop) is byte-equivalent to the
//! spike — the spike's clips committed under `tools/model_export/reference_audio/`
//! are the comparison fixture (acceptance bullet 4 of CL-12).
//!
//! What changed vs the spike:
//! - The interactive `clap` CLI + WAV/FLAC writers are gone; this module is
//!   library-only.
//! - The `--max-secs` timeout / stdin-Enter dance is owned by the caller
//!   (CL-13's session-lifecycle commands); the adapter just runs until
//!   `stop()`.
//! - Errors are mapped from `anyhow::Error` to the typed
//!   [`MicrophoneError`] variants.
//! - A 10 Hz level (RMS) callback is exposed for the UI's recording indicator
//!   per the task context — the spike didn't surface a level meter.
//! - `pause()` / `resume()` wrap `cpal::Stream::pause()` / `play()`; the
//!   spike's CLI never paused mid-take.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BuildStreamError, DefaultStreamConfigError, PauseStreamError, PlayStreamError, Stream};
use log::{debug, warn};

use super::buffer::AudioBuffer;
use super::error::MicrophoneError;
use super::resample::resample_to_16k;

/// Throttle the level callback to ~10 Hz so the UI indicator stays smooth
/// without flooding the IPC channel.
const LEVEL_CALLBACK_INTERVAL: Duration = Duration::from_millis(100);

/// State machine for the adapter. Captures the operations that are valid
/// from each state so `pause` / `resume` / `stop` reject invalid transitions
/// without panicking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    Capturing,
    Paused,
    Stopped,
}

/// Cross-platform cpal audio capture adapter.
///
/// Lifecycle:
///
/// ```text
/// new() → start_capture() → [ pause() <-> resume() ]* → stop() → AudioBuffer
/// ```
///
/// `new()` does not open the device; capture begins on `start_capture()`.
/// `stop()` finalizes the resample-to-16kHz step and returns the buffer; the
/// adapter is single-use after `stop()`.
///
/// Not `Send` once a stream is active on platforms where the `Stream` is
/// `!Send` (e.g. macOS); construction and lifecycle are expected to happen on
/// the same thread (the Tauri command thread in CL-13).
pub struct CpalAdapter {
    state: State,
    stream: Option<Stream>,
    /// Shared with the cpal data callback. Holds the raw (pre-resample),
    /// pre-downmix samples in the native input rate, mono.
    buffer: Arc<Mutex<Vec<f32>>>,
    /// Mid-capture errors observed by the cpal error_callback. Surfaced on
    /// `stop()` so a device-disconnect doesn't silently produce a clean clip.
    stream_error: Arc<Mutex<Option<String>>>,
    /// Native sample rate of the input device, captured at `start_capture()`.
    /// Used to drive the resample-to-16kHz step at `stop()` time.
    native_rate: u32,
}

impl CpalAdapter {
    /// Constructs an idle adapter. Does NOT open the audio device — capture
    /// begins on [`start_capture`](Self::start_capture).
    pub fn new() -> Result<Self, MicrophoneError> {
        Ok(Self {
            state: State::Idle,
            stream: None,
            buffer: Arc::new(Mutex::new(Vec::new())),
            stream_error: Arc::new(Mutex::new(None)),
            native_rate: 0,
        })
    }

    /// Opens the default input device and begins capture.
    ///
    /// `on_level` is invoked at ~10 Hz with the RMS level of the most recent
    /// audio window, normalized to roughly `[0.0, 1.0]` (the value is the
    /// raw RMS of f32 samples in `[-1.0, 1.0]`; UIs typically map this to a
    /// dB scale or a 0–100 indicator). The callback runs on the cpal audio
    /// thread; it must not block.
    pub fn start_capture<L>(&mut self, on_level: L) -> Result<(), MicrophoneError>
    where
        L: Fn(f32) + Send + Sync + 'static,
    {
        if self.state != State::Idle {
            return Err(MicrophoneError::StreamError(format!(
                "start_capture called in invalid state: {:?}",
                self.state
            )));
        }

        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(MicrophoneError::NoDevice)?;

        let config = device
            .default_input_config()
            .map_err(map_default_config_err)?;

        let native_rate = config.sample_rate();
        let channels = config.channels() as usize;

        #[allow(deprecated)]
        let device_name = device.name().unwrap_or_else(|_| "<unknown>".into());
        debug!(
            "cpal: opening input device '{}' at {} Hz, {} ch",
            device_name, native_rate, channels
        );

        // Reset shared state in case this is a recycled adapter (defensive;
        // the state machine should already have prevented this).
        if let Ok(mut buf) = self.buffer.lock() {
            buf.clear();
        }
        if let Ok(mut slot) = self.stream_error.lock() {
            *slot = None;
        }

        let buffer_for_cb = Arc::clone(&self.buffer);
        let stream_error_for_data = Arc::clone(&self.stream_error);

        // Level-callback throttling state.  `Arc<Mutex<Instant>>` because the
        // closure must be `Fn`, not `FnMut`, for cpal.
        let last_level_at = Arc::new(Mutex::new(Instant::now()));
        let on_level = Arc::new(on_level);

        let stream = device
            .build_input_stream(
                &config.into(),
                {
                    let on_level = Arc::clone(&on_level);
                    let last_level_at = Arc::clone(&last_level_at);
                    move |data: &[f32], _| {
                        // Mono downmix — preserved from the spike.
                        let mono: Vec<f32> = if channels == 1 {
                            data.to_vec()
                        } else {
                            data.chunks(channels)
                                .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                                .collect()
                        };

                        // Append; surface poisoning via the stream-error slot
                        // instead of panicking in the audio thread (matches
                        // the spike's defensive handling).
                        match buffer_for_cb.lock() {
                            Ok(mut g) => g.extend_from_slice(&mono),
                            Err(_) => {
                                if let Ok(mut slot) = stream_error_for_data.lock() {
                                    if slot.is_none() {
                                        *slot = Some("buffer mutex poisoned".to_owned());
                                    }
                                }
                            }
                        }

                        // Throttled level callback (~10 Hz).
                        if let Ok(mut last) = last_level_at.lock() {
                            if last.elapsed() >= LEVEL_CALLBACK_INTERVAL {
                                *last = Instant::now();
                                let rms = compute_rms(&mono);
                                (on_level)(rms);
                            }
                        }
                    }
                },
                {
                    let stream_error_for_err = Arc::clone(&self.stream_error);
                    move |err| {
                        warn!("cpal stream error: {err}");
                        if let Ok(mut slot) = stream_error_for_err.lock() {
                            if slot.is_none() {
                                *slot = Some(format!("{err}"));
                            }
                        }
                    }
                },
                None,
            )
            .map_err(map_build_stream_err)?;

        stream.play().map_err(map_play_stream_err)?;

        self.stream = Some(stream);
        self.native_rate = native_rate;
        self.state = State::Capturing;

        Ok(())
    }

    /// Pauses capture without releasing the device.
    pub fn pause(&mut self) -> Result<(), MicrophoneError> {
        match self.state {
            State::Capturing => {
                if let Some(stream) = self.stream.as_ref() {
                    stream.pause().map_err(map_pause_stream_err)?;
                }
                self.state = State::Paused;
                Ok(())
            }
            State::Paused => Ok(()), // idempotent no-op
            other => Err(MicrophoneError::StreamError(format!(
                "pause called in invalid state: {other:?}"
            ))),
        }
    }

    /// Resumes capture from the device.
    pub fn resume(&mut self) -> Result<(), MicrophoneError> {
        match self.state {
            State::Paused => {
                if let Some(stream) = self.stream.as_ref() {
                    stream.play().map_err(map_play_stream_err)?;
                }
                self.state = State::Capturing;
                Ok(())
            }
            State::Capturing => Ok(()), // idempotent no-op
            other => Err(MicrophoneError::StreamError(format!(
                "resume called in invalid state: {other:?}"
            ))),
        }
    }

    /// Stops capture, finalizes the resample-to-16 kHz step, returns the
    /// buffer.  The adapter is single-use after `stop()`.
    pub fn stop(&mut self) -> Result<AudioBuffer, MicrophoneError> {
        if matches!(self.state, State::Stopped | State::Idle) {
            return Err(MicrophoneError::StreamError(format!(
                "stop called in invalid state: {:?}",
                self.state
            )));
        }

        // Drop the stream first so the cpal audio thread stops calling our
        // data callback before we drain the buffer.
        drop(self.stream.take());
        self.state = State::Stopped;

        // If the cpal error_callback flagged a mid-capture failure, surface
        // it as DeviceDisconnected — the most likely cause of a mid-capture
        // error is device unplug or OS audio teardown.  The message is
        // logged at the callback site (warn!) rather than embedded in the
        // variant; CL-13's session-lifecycle layer logs the variant + context.
        let stream_err = self
            .stream_error
            .lock()
            .ok()
            .and_then(|g| g.clone());
        if stream_err.is_some() {
            return Err(MicrophoneError::DeviceDisconnected);
        }

        let raw = match self.buffer.lock() {
            Ok(mut g) => std::mem::take(&mut *g),
            Err(poisoned) => poisoned.into_inner().clone(),
        };

        let resampled = resample_to_16k(raw, self.native_rate)?;
        Ok(AudioBuffer::from_samples(resampled))
    }
}

impl Drop for CpalAdapter {
    fn drop(&mut self) {
        // Ensure the cpal stream is torn down even if `stop()` was never
        // called (panic mid-capture, command future dropped, etc.).
        if let Some(stream) = self.stream.take() {
            drop(stream);
        }
    }
}

/// RMS over a window of f32 samples in `[-1.0, 1.0]`.
fn compute_rms(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return 0.0;
    }
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

// ---------------------------------------------------------------------------
// cpal error mapping. cpal exposes typed errors for each call site; we fan
// them in to the small surface MicrophoneError exposes per CL-12.
// ---------------------------------------------------------------------------

fn map_default_config_err(err: DefaultStreamConfigError) -> MicrophoneError {
    match err {
        DefaultStreamConfigError::DeviceNotAvailable => MicrophoneError::NoDevice,
        DefaultStreamConfigError::StreamTypeNotSupported => {
            MicrophoneError::ConfigUnsupported(format!("{err}"))
        }
        other => MicrophoneError::StreamError(format!("default_input_config: {other}")),
    }
}

fn map_build_stream_err(err: BuildStreamError) -> MicrophoneError {
    match err {
        BuildStreamError::DeviceNotAvailable => MicrophoneError::NoDevice,
        BuildStreamError::StreamConfigNotSupported => {
            MicrophoneError::ConfigUnsupported(format!("{err}"))
        }
        // TODO: distinguish permission-denied from generic stream-build
        // failure per platform; see ADD §3.9 fallback. On macOS a TCC denial
        // surfaces here as BackendSpecific; on Windows the OS surface is
        // similarly opaque. CL-12 leaves this as a documented gap; the
        // DEMO-5 simulated-mode fallback in the frontend handles the user
        // experience for now and CL-13 will add the session-lifecycle
        // retry/permission UI.
        other => MicrophoneError::StreamError(format!("build_input_stream: {other}")),
    }
}

fn map_play_stream_err(err: PlayStreamError) -> MicrophoneError {
    match err {
        PlayStreamError::DeviceNotAvailable => MicrophoneError::DeviceDisconnected,
        other => MicrophoneError::StreamError(format!("stream play: {other}")),
    }
}

fn map_pause_stream_err(err: PauseStreamError) -> MicrophoneError {
    match err {
        PauseStreamError::DeviceNotAvailable => MicrophoneError::DeviceDisconnected,
        other => MicrophoneError::StreamError(format!("stream pause: {other}")),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compute_rms_zero_for_silence() {
        let silence = vec![0.0_f32; 100];
        assert_eq!(compute_rms(&silence), 0.0);
    }

    #[test]
    fn compute_rms_correct_for_constant_signal() {
        // Constant +0.5 → RMS == 0.5.
        let signal = vec![0.5_f32; 1000];
        let rms = compute_rms(&signal);
        assert!((rms - 0.5).abs() < 1e-6, "expected ~0.5, got {rms}");
    }

    #[test]
    fn compute_rms_empty_slice_is_zero() {
        assert_eq!(compute_rms(&[]), 0.0);
    }

    /// Device-dependent smoke test. Marked `#[ignore]` so CI runners without
    /// audio hardware don't fail; run locally with
    /// `cargo test recording::cpal_adapter -- --ignored --nocapture`.
    #[test]
    #[ignore]
    fn cpal_adapter_records_some_audio() {
        let mut adapter = CpalAdapter::new().expect("adapter constructs");
        adapter
            .start_capture(|_level| {})
            .expect("start_capture should succeed if a device is present");
        std::thread::sleep(std::time::Duration::from_millis(200));
        let buffer = adapter.stop().expect("stop should succeed");
        assert!(
            !buffer.is_empty(),
            "captured buffer should not be empty after 200 ms"
        );
        assert_eq!(buffer.sample_rate(), 16_000);
    }

    #[test]
    fn pause_in_idle_state_errors() {
        let mut adapter = CpalAdapter::new().expect("adapter constructs");
        let result = adapter.pause();
        assert!(result.is_err(), "pause from Idle should error");
    }

    #[test]
    fn resume_in_idle_state_errors() {
        let mut adapter = CpalAdapter::new().expect("adapter constructs");
        let result = adapter.resume();
        assert!(result.is_err(), "resume from Idle should error");
    }

    #[test]
    fn stop_in_idle_state_errors() {
        let mut adapter = CpalAdapter::new().expect("adapter constructs");
        let result = adapter.stop();
        assert!(result.is_err(), "stop from Idle should error");
    }
}
