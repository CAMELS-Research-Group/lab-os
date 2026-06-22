//! Tauri command surface for the recording feature.
//!
//! Spec: ADD §3.6 (recording commands), ADD §3.3 (session state machine),
//! FRD F-PSF-6 (practice-again loop).
//!
//! Five commands, all thin wrappers over the lock-guarded [`SessionLifecycle`]:
//!
//! - [`start_session`] — Idle | Reviewing → Recording. Returns the fresh
//!   [`SessionId`].
//! - [`pause_session`] — Recording → Paused.
//! - [`resume_session`] — Paused → Recording.
//! - [`cancel_session`] — Recording | Paused | Error → Idle. Discards the
//!   audio buffer.
//! - [`end_session`] — Recording | Paused → Evaluating. Stashes the finalized
//!   [`AudioBuffer`] in the lifecycle for CL-19 to pick up.
//!
//! ## Event emission
//!
//! `start_session` constructs a level callback that forwards the cpal RMS
//! values to the frontend via Tauri's `app.emit("recording:level", …)`. The
//! callback is invoked at ~10 Hz from the cpal audio thread; the closure is
//! `Send + Sync + 'static` because the [`tauri::AppHandle`] is `Send + Sync`
//! and `Clone`.
//!
//! ## Deferred wiring
//!
//! `end_session` does not yet dispatch the audio buffer to the evaluation
//! orchestrator (CL-19 has not landed). The buffer sits in the lifecycle's
//! `last_audio` slot until CL-19's orchestrator picks it up via
//! [`SessionLifecycle::take_audio`]. See the `TODO(CL-19)` marker below.

use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::recording::SessionLifecycle;
use crate::shared::error::AppError;
use crate::shared::types::SessionId;

pub use crate::AppState;

const LIFECYCLE_LOCK_POISONED_MSG: &str = "session lifecycle lock poisoned";

/// Wire payload for `recording:level`. Mirrors the `{ rms: number }` shape the
/// frontend will deserialize via Tauri's `listen` API.
#[derive(Debug, Clone, Serialize)]
struct RecordingLevelEvent {
    rms: f32,
}

fn lock_lifecycle(
    lifecycle: &Mutex<SessionLifecycle>,
) -> Result<std::sync::MutexGuard<'_, SessionLifecycle>, AppError> {
    lifecycle
        .lock()
        .map_err(|_| AppError::InvalidState(LIFECYCLE_LOCK_POISONED_MSG.to_string()))
}

#[tauri::command]
pub async fn start_session(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<SessionId, AppError> {
    let mut lifecycle = lock_lifecycle(&state.lifecycle)?;

    // `AppHandle` is `Send + Sync + Clone`. We clone it into the level
    // callback so the closure satisfies `Fn(f32) + Send + Sync + 'static`.
    let app_for_callback = app.clone();
    let on_level = move |rms: f32| {
        // Best-effort emit; if the webview is closed mid-capture this fails
        // and we drop the level update silently. Logging at the audio-thread
        // callback site would risk allocation per frame, so we swallow.
        let _ = app_for_callback.emit("recording:level", RecordingLevelEvent { rms });
    };

    lifecycle.start_session(on_level)
}

#[tauri::command]
pub async fn pause_session(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let mut lifecycle = lock_lifecycle(&state.lifecycle)?;
    lifecycle.pause()
}

#[tauri::command]
pub async fn resume_session(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let mut lifecycle = lock_lifecycle(&state.lifecycle)?;
    lifecycle.resume()
}

#[tauri::command]
pub async fn cancel_session(state: tauri::State<'_, AppState>) -> Result<(), AppError> {
    let mut lifecycle = lock_lifecycle(&state.lifecycle)?;
    lifecycle.cancel()
}

#[tauri::command]
pub async fn end_session(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    // Single lock scope: stop capture, pull the audio + current session id,
    // then release the lock before spawning the evaluation task. The spawned
    // task acquires the locks it needs (db, lifecycle) on its own.
    let (session_id, audio) = {
        let mut lifecycle = lock_lifecycle(&state.lifecycle)?;
        lifecycle.end()?;
        let session_id = lifecycle
            .current_session_id()
            .cloned()
            .ok_or_else(|| {
                AppError::InvalidState("end_session: no current session_id after end()".into())
            })?;
        let audio = lifecycle.take_audio().ok_or_else(|| {
            AppError::InvalidState("end_session: no pending audio after end()".into())
        })?;
        (session_id, audio)
    };

    // Fire-and-forget. The orchestrator owns the rest of the lifecycle:
    // chunked inference + eval:progress, persistence + eval:done, or
    // eval:error + lifecycle → Error on failure. See
    // `crate::evaluation::orchestrator` for the full state diagram.
    crate::evaluation::orchestrator::run_evaluation(session_id, audio, app);
    Ok(())
}
