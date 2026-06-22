//! Errors produced by the recording layer.
//!
//! Per ADD §3.10, each feature module owns its own `thiserror`-derived error
//! enum and bubbles up into [`crate::shared::error::AppError`] via `#[from]`.
//!
//! Variants:
//!
//! - [`MicrophoneError::PermissionDenied`] — OS denied microphone access (TCC
//!   on macOS, privacy settings on Windows). Distinguishing this from a
//!   generic stream-build failure is platform-specific and is currently
//!   surfaced only when `default_input_device()` returns `None` AFTER a build
//!   attempt; see `cpal_adapter.rs` for the TODO.
//! - [`MicrophoneError::NoDevice`] — no default input device available (no
//!   mic connected, or the OS hasn't enumerated one yet).
//! - [`MicrophoneError::DeviceDisconnected`] — stream errored mid-capture;
//!   the device is most likely gone (USB unplug, sleep, etc.).
//! - [`MicrophoneError::ConfigUnsupported`] — the host/device combination
//!   refuses the requested f32 input config.
//! - [`MicrophoneError::StreamError`] — catch-all for cpal build/play/pause
//!   failures that don't map cleanly to the variants above.
//! - [`MicrophoneError::Resampler`] — rubato setup or runtime failure.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum MicrophoneError {
    #[error("microphone permission denied")]
    PermissionDenied,

    #[error("no audio input device available")]
    NoDevice,

    #[error("audio device disconnected during capture")]
    DeviceDisconnected,

    #[error("audio device config unsupported: {0}")]
    ConfigUnsupported(String),

    #[error("cpal stream error: {0}")]
    StreamError(String),

    #[error("resampler error: {0}")]
    Resampler(String),
}
