//! Cross-platform audio capture — cpal-based recording at 16 kHz mono f32.
//!
//! Architecture ported from `spike/rust-poc/src/{recorder,resample}.rs` per
//! ADD §3.1 ("recording carve-out") and `spike/FINDINGS.md` §4. The spike
//! sources remain in tree as a CLI tool for operator-recorded reference
//! fixtures; this module is the library form the production client uses
//! end-to-end.
//!
//! Layout:
//!
//! - [`CpalAdapter`] — cpal device + stream lifecycle (`new`, `start_capture`,
//!   `pause`, `resume`, `stop`).
//! - [`AudioBuffer`] — in-memory 16 kHz mono f32 buffer returned by `stop()`.
//! - [`resample_to_16k`] — pure rubato sinc-resampler wrapper.
//! - [`MicrophoneError`] — feature-owned error type, bubbles into
//!   [`crate::shared::error::AppError`] via `#[from]`.
//! - [`SessionLifecycle`] — session state machine (ADD §3.3) coordinating the
//!   adapter across the read-evaluate-review cycle; held in
//!   [`crate::AppState`] behind a `Mutex` (CL-13).
//! - [`commands`] — Tauri `#[command]` surface (CL-13).

mod buffer;
pub mod commands;
mod cpal_adapter;
mod error;
mod resample;
mod session;

pub use buffer::{AudioBuffer, SAMPLE_RATE};
pub use cpal_adapter::CpalAdapter;
pub use error::MicrophoneError;
pub use resample::{resample_to_16k, TARGET_RATE};
pub use session::{SessionLifecycle, SessionState};
