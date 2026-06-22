//! Session lifecycle state machine.
//!
//! Spec: ADD §3.3 (session state machine), ADD §3.6 (recording commands),
//! FRD F-PSF-6 (practice-again loop).
//!
//! A **session** is one read-evaluate cycle. Learners typically run several per
//! sitting; the "practice again" path (Reviewing → Recording) loops within the
//! same Tauri process, minting a fresh [`SessionId`] for each new cycle while
//! preserving prior cycle ids in [`SessionLifecycle::completed_session_ids`].
//!
//! ## States
//!
//! Six states per ADD §3.3:
//!
//! - [`SessionState::Idle`] — default post-FirstRun; no active capture.
//! - [`SessionState::Recording`] — cpal is capturing; `recording:level` events
//!   are firing.
//! - [`SessionState::Paused`] — capture paused (not discarded).
//! - [`SessionState::Evaluating`] — capture stopped, audio handed off (or
//!   waiting to be handed off) to the evaluation orchestrator.
//! - [`SessionState::Reviewing`] — results shown to the learner; the cycle's
//!   single report is queued on leaving this state (FRD F-RPT-1).
//! - [`SessionState::Error`] — unrecoverable mid-capture failure (e.g. cpal
//!   stream error → device disconnect). Recovery is `cancel_session` → Idle.
//!
//! ## Threading
//!
//! `CpalAdapter` holds a `cpal::Stream` which is `!Send` on macOS while a
//! stream is alive. The lifecycle is therefore designed to live behind a
//! `Mutex<SessionLifecycle>` accessed from the Tauri command thread only; the
//! mutex guards the lifecycle from concurrent mutation, and the `!Send` stream
//! never crosses thread boundaries because every command body locks, mutates,
//! and unlocks on the same thread (Tauri serializes command invocations onto
//! its async runtime worker). See [`crate::AppState`] for the wrapping.
//!
//! ## Deferred wiring
//!
//! CL-13 is the analysis-first re-sequence's last task before the evaluation
//! block. CL-19 (evaluation orchestrator) and CL-22 (reporting worker) have
//! not landed yet. Two sites carry `TODO` markers:
//!
//! - `end_session` stashes the finalized [`AudioBuffer`] in `last_audio` and
//!   transitions to Evaluating. CL-19's orchestrator will drain `last_audio`
//!   via [`SessionLifecycle::take_audio`] when it lands. Until then the
//!   command returns `Ok(())` and the buffer is held in memory.
//! - The Reviewing → Recording transition discards `last_audio` (the cycle's
//!   audio is already evaluated; the per-cycle report would be queued by
//!   the reporting worker before mintage of the new id). CL-22 will hook the
//!   `// TODO(CL-22)` site for the report-queue signal.

use uuid::Uuid;

use crate::recording::{AudioBuffer, CpalAdapter, MicrophoneError};
use crate::shared::error::AppError;
use crate::shared::types::SessionId;

/// Six states per ADD §3.3 (see module docstring for transition diagram).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionState {
    Idle,
    Recording,
    Paused,
    Evaluating,
    Reviewing,
    Error,
}

/// In-memory session lifecycle. Single-cycle live; prior cycle ids are kept
/// in `completed_session_ids` for the multi-cycle "practice again" loop.
///
/// The `adapter` field is `Option<CpalAdapter>` because (1) idle/paused/etc
/// states do not need it, and (2) `CpalAdapter` is `!Send` while a stream is
/// alive on some platforms, so we want construction and lifecycle to happen
/// on the command thread. The wrapping `Mutex<SessionLifecycle>` in
/// [`crate::AppState`] enforces single-thread access at runtime.
pub struct SessionLifecycle {
    state: SessionState,
    current_session_id: Option<SessionId>,
    completed_session_ids: Vec<SessionId>,
    adapter: Option<CpalAdapter>,
    last_audio: Option<AudioBuffer>,
    /// Last error message captured by `mark_error`. Surfaced in logs and as
    /// the recovery path's context; cleared on `cancel` back to Idle.
    last_error_message: Option<String>,
}

impl Default for SessionLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionLifecycle {
    /// Construct an idle, cycle-history-empty lifecycle.
    pub fn new() -> Self {
        Self {
            state: SessionState::Idle,
            current_session_id: None,
            completed_session_ids: Vec::new(),
            adapter: None,
            last_audio: None,
            last_error_message: None,
        }
    }

    // ----- accessors -----

    pub fn state(&self) -> SessionState {
        self.state
    }

    pub fn current_session_id(&self) -> Option<&SessionId> {
        self.current_session_id.as_ref()
    }

    pub fn completed_session_ids(&self) -> &[SessionId] {
        &self.completed_session_ids
    }

    pub fn last_error_message(&self) -> Option<&str> {
        self.last_error_message.as_deref()
    }

    /// `true` when an `AudioBuffer` is stashed awaiting evaluation pickup.
    /// Set by `end()`; cleared by `take_audio()` or by a `cancel` from Error.
    pub fn has_pending_audio(&self) -> bool {
        self.last_audio.is_some()
    }

    /// Consume the stashed `AudioBuffer` left by `end()`. Used by the
    /// (forthcoming) CL-19 evaluation orchestrator to pick up the buffer
    /// after `end_session` has returned. Returns `None` if no buffer is
    /// pending.
    pub fn take_audio(&mut self) -> Option<AudioBuffer> {
        self.last_audio.take()
    }

    // ----- transitions -----

    /// Idle → Recording or Reviewing → Recording (the "practice again" loop,
    /// FRD F-PSF-6). Constructs a fresh [`CpalAdapter`], starts capture, and
    /// emits `recording:level` events via `on_level`.
    ///
    /// When called from Reviewing, the current cycle's id is pushed to
    /// `completed_session_ids` and a new id is minted. This is the only path
    /// that grows `completed_session_ids`.
    ///
    /// `// TODO(CL-22): notify reporting worker before starting fresh cycle.`
    /// CL-22 will hook the cycle-leave signal here (FRD F-RPT-1 — report is
    /// queued on leaving Reviewing).
    pub fn start_session<L>(&mut self, on_level: L) -> Result<SessionId, AppError>
    where
        L: Fn(f32) + Send + Sync + 'static,
    {
        // State-machine check first — bail before touching the audio device.
        self.guard_start_session_state()?;

        // Build the adapter and begin capture. If `start_capture` fails the
        // adapter is dropped here and state is unchanged.
        let mut adapter = CpalAdapter::new().map_err(AppError::from)?;
        adapter.start_capture(on_level).map_err(AppError::from)?;

        let new_id = mint_session_id();
        self.apply_start_session(new_id.clone(), Some(adapter));
        Ok(new_id)
    }

    /// Recording → Paused. No-op-with-success not exposed; if already Paused,
    /// the adapter is already paused and we treat it as a transition error to
    /// help surface logic mistakes in the frontend (a no-op `pause` from
    /// `Paused` would mask repeated taps that should be ignored at the UI
    /// layer).
    pub fn pause(&mut self) -> Result<(), AppError> {
        if self.state != SessionState::Recording {
            return Err(invalid_state("pause", self.state));
        }
        if let Some(adapter) = self.adapter.as_mut() {
            adapter.pause().map_err(AppError::from)?;
        }
        self.state = SessionState::Paused;
        Ok(())
    }

    /// Paused → Recording.
    pub fn resume(&mut self) -> Result<(), AppError> {
        if self.state != SessionState::Paused {
            return Err(invalid_state("resume", self.state));
        }
        if let Some(adapter) = self.adapter.as_mut() {
            adapter.resume().map_err(AppError::from)?;
        }
        self.state = SessionState::Recording;
        Ok(())
    }

    /// Recording | Paused | Error → Idle. Drops the audio buffer (the cycle
    /// is being discarded), clears the current session id, and tears down the
    /// adapter. `Error → Idle` is the documented recovery path per the
    /// controller amendments to CL-13.
    pub fn cancel(&mut self) -> Result<(), AppError> {
        match self.state {
            SessionState::Recording | SessionState::Paused | SessionState::Error => {
                // Drop the adapter — its `Drop` impl tears down the cpal
                // stream even if we never called `stop()`.
                drop(self.adapter.take());
                self.last_audio = None;
                self.current_session_id = None;
                self.last_error_message = None;
                self.state = SessionState::Idle;
                Ok(())
            }
            other => Err(invalid_state("cancel", other)),
        }
    }

    /// Recording | Paused → Evaluating. Stops the cpal stream, drains the
    /// final 16 kHz mono buffer, and stashes it in `last_audio` for the
    /// (forthcoming) evaluation orchestrator to pick up.
    ///
    /// `// TODO(CL-19): wire AudioBuffer dispatch to evaluation::orchestrator.`
    /// Once CL-19 lands, the command body will call into the orchestrator
    /// after this returns; for now the buffer sits in `last_audio` until a
    /// future cancel or evaluation-complete transition consumes it.
    pub fn end(&mut self) -> Result<(), AppError> {
        if !matches!(self.state, SessionState::Recording | SessionState::Paused) {
            return Err(invalid_state("end", self.state));
        }
        let mut adapter = self
            .adapter
            .take()
            .ok_or_else(|| AppError::InvalidState(
                "end called with no adapter present (lifecycle internal invariant violated)"
                    .to_string(),
            ))?;
        let buffer = adapter.stop().map_err(AppError::from)?;
        self.last_audio = Some(buffer);
        self.state = SessionState::Evaluating;
        Ok(())
    }

    /// Evaluating → Reviewing. Called by the (forthcoming) CL-19 evaluation
    /// orchestrator on `eval:done`. Exposed for testability and so the
    /// command layer can drive the transition once CL-19 lands.
    pub fn mark_evaluation_complete(&mut self) -> Result<(), AppError> {
        if self.state != SessionState::Evaluating {
            return Err(invalid_state("mark_evaluation_complete", self.state));
        }
        self.state = SessionState::Reviewing;
        Ok(())
    }

    /// Recording | Paused | Evaluating → Error. Called by the command layer
    /// or a background error handler when the cpal stream raises an
    /// unrecoverable mid-capture failure (most commonly
    /// [`MicrophoneError::DeviceDisconnected`]).
    ///
    /// Recovery path: emit `recording:error` (command layer) → `cancel` to
    /// drain back to Idle.
    pub fn mark_error(&mut self, source: MicrophoneError) -> Result<(), AppError> {
        if !matches!(
            self.state,
            SessionState::Recording | SessionState::Paused | SessionState::Evaluating
        ) {
            return Err(invalid_state("mark_error", self.state));
        }
        drop(self.adapter.take());
        self.last_audio = None;
        self.last_error_message = Some(source.to_string());
        self.state = SessionState::Error;
        Ok(())
    }

    // ----- internals -----

    /// State-only check for `start_session`. Separated from the adapter-wiring
    /// so tests can exercise the transition without an audio device.
    fn guard_start_session_state(&self) -> Result<(), AppError> {
        match self.state {
            SessionState::Idle | SessionState::Reviewing => Ok(()),
            other => Err(invalid_state("start_session", other)),
        }
    }

    /// State-only mutation for `start_session`. Tests inject `None` for the
    /// adapter; production passes `Some(adapter)` after `start_capture`
    /// succeeded.
    fn apply_start_session(&mut self, new_id: SessionId, adapter: Option<CpalAdapter>) {
        if self.state == SessionState::Reviewing {
            // FRD F-PSF-6: move the just-completed cycle into history before
            // minting the new id. The audio for the prior cycle is already
            // evaluated; discard any lingering buffer.
            // TODO(CL-22): notify reporting worker before starting fresh cycle.
            if let Some(prev_id) = self.current_session_id.take() {
                self.completed_session_ids.push(prev_id);
            }
            self.last_audio = None;
        }
        self.current_session_id = Some(new_id);
        self.adapter = adapter;
        self.last_error_message = None;
        self.state = SessionState::Recording;
    }
}

/// Mint a fresh v4 UUID as a `SessionId`. Centralized so tests can pin it.
fn mint_session_id() -> SessionId {
    SessionId(Uuid::new_v4().to_string())
}

/// Format the canonical `InvalidState` error for a rejected transition.
fn invalid_state(op: &str, from: SessionState) -> AppError {
    AppError::InvalidState(format!(
        "cannot {op} from {from:?}; transition not allowed by session state machine"
    ))
}

// ---------------------------------------------------------------------------
// Tests
//
// The state-machine logic is exercised directly via `guard_start_session_state`
// + `apply_start_session` (the test-friendly split). Adapter-dependent paths
// (real cpal capture) are not exercised here; they are covered by the
// `cpal_adapter` module's `#[ignore]` device smoke test.
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    /// Test helper: drive Idle → Recording without constructing a real
    /// `CpalAdapter`. Returns the freshly minted id.
    fn test_start(lc: &mut SessionLifecycle) -> SessionId {
        lc.guard_start_session_state().expect("state allows start");
        let id = mint_session_id();
        lc.apply_start_session(id.clone(), None);
        id
    }

    /// Helper: drive Recording → Evaluating without a real adapter. Mirrors
    /// `end()`'s state mutation; bypasses adapter teardown (no adapter held).
    fn test_end(lc: &mut SessionLifecycle) {
        assert!(
            matches!(lc.state, SessionState::Recording | SessionState::Paused),
            "test_end pre-condition: state was {:?}",
            lc.state,
        );
        // Stash a synthetic buffer so callers can observe `take_audio`.
        lc.last_audio = Some(AudioBuffer::new());
        lc.state = SessionState::Evaluating;
    }

    // --- construction ----------------------------------------------------

    #[test]
    fn new_lifecycle_starts_idle_with_empty_history() {
        let lc = SessionLifecycle::new();
        assert_eq!(lc.state(), SessionState::Idle);
        assert!(lc.current_session_id().is_none());
        assert!(lc.completed_session_ids().is_empty());
        assert!(!lc.has_pending_audio());
        assert!(lc.last_error_message().is_none());
    }

    // --- valid transitions -----------------------------------------------

    #[test]
    fn idle_to_recording_via_start_mints_fresh_id() {
        let mut lc = SessionLifecycle::new();
        let id = test_start(&mut lc);
        assert_eq!(lc.state(), SessionState::Recording);
        assert_eq!(lc.current_session_id(), Some(&id));
        assert!(lc.completed_session_ids().is_empty());
    }

    #[test]
    fn recording_to_paused_and_back_via_pause_resume() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);

        // No real adapter — pause/resume short-circuit to the state mutation.
        lc.pause().expect("pause from Recording");
        assert_eq!(lc.state(), SessionState::Paused);

        lc.resume().expect("resume from Paused");
        assert_eq!(lc.state(), SessionState::Recording);
    }

    #[test]
    fn recording_to_evaluating_via_end_stashes_buffer() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        test_end(&mut lc);
        assert_eq!(lc.state(), SessionState::Evaluating);
        assert!(lc.has_pending_audio(), "end should stash an AudioBuffer");

        // `take_audio` consumes it.
        let audio = lc.take_audio().expect("buffer was stashed");
        assert!(audio.is_empty(), "synthetic test buffer is empty");
        assert!(!lc.has_pending_audio());
    }

    #[test]
    fn paused_to_evaluating_via_end_is_valid() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        lc.pause().unwrap();
        // `end` from Paused is allowed per ADD §3.3 ("Exit to Evaluating from
        // Recording or Paused").
        assert!(matches!(lc.state(), SessionState::Paused));
        test_end(&mut lc);
        assert_eq!(lc.state(), SessionState::Evaluating);
    }

    #[test]
    fn evaluating_to_reviewing_via_mark_evaluation_complete() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        test_end(&mut lc);
        lc.mark_evaluation_complete()
            .expect("Evaluating → Reviewing");
        assert_eq!(lc.state(), SessionState::Reviewing);
    }

    #[test]
    fn reviewing_to_recording_via_start_is_practice_again() {
        let mut lc = SessionLifecycle::new();
        let first = test_start(&mut lc);
        test_end(&mut lc);
        lc.mark_evaluation_complete().unwrap();
        assert_eq!(lc.state(), SessionState::Reviewing);

        // "Practice again": Reviewing → Recording, fresh id, prior id archived.
        let second = test_start(&mut lc);
        assert_eq!(lc.state(), SessionState::Recording);
        assert_ne!(first, second, "fresh SessionId on practice-again");
        assert_eq!(
            lc.completed_session_ids(),
            &[first],
            "prior cycle id moved to completed_session_ids on practice-again"
        );
        assert_eq!(lc.current_session_id(), Some(&second));
    }

    #[test]
    fn recording_to_idle_via_cancel_discards_buffer() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        // Stash a fake pending buffer to confirm it's wiped.
        lc.last_audio = Some(AudioBuffer::new());

        lc.cancel().expect("cancel from Recording");
        assert_eq!(lc.state(), SessionState::Idle);
        assert!(lc.current_session_id().is_none());
        assert!(!lc.has_pending_audio(), "cancel must drop the audio buffer");
    }

    #[test]
    fn paused_to_idle_via_cancel_discards_buffer() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        lc.pause().unwrap();
        lc.last_audio = Some(AudioBuffer::new());

        lc.cancel().expect("cancel from Paused");
        assert_eq!(lc.state(), SessionState::Idle);
        assert!(!lc.has_pending_audio());
    }

    #[test]
    fn error_to_idle_via_cancel_drains_error_state() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        lc.mark_error(MicrophoneError::DeviceDisconnected)
            .expect("mark_error from Recording");
        assert_eq!(lc.state(), SessionState::Error);
        assert!(lc.last_error_message().is_some());

        lc.cancel().expect("cancel drains Error → Idle");
        assert_eq!(lc.state(), SessionState::Idle);
        assert!(
            lc.last_error_message().is_none(),
            "cancel should clear the captured error message"
        );
    }

    #[test]
    fn mark_error_from_evaluating_transitions_to_error() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        test_end(&mut lc);
        assert_eq!(lc.state(), SessionState::Evaluating);

        // E.g. an inference-thread cpal teardown surfaces an error mid-eval.
        lc.mark_error(MicrophoneError::DeviceDisconnected)
            .expect("mark_error from Evaluating");
        assert_eq!(lc.state(), SessionState::Error);
        // mark_error must also clear any pending audio so we don't try to
        // re-dispatch a buffer that came from a borked stream.
        assert!(!lc.has_pending_audio());
    }

    // --- invalid transitions ---------------------------------------------

    #[test]
    fn pause_from_idle_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        let err = lc.pause().expect_err("pause from Idle is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn resume_from_recording_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        let err = lc.resume().expect_err("resume from Recording is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn resume_from_idle_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        let err = lc.resume().expect_err("resume from Idle is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn end_from_idle_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        let err = lc.end().expect_err("end from Idle is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn end_from_reviewing_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        test_end(&mut lc);
        lc.mark_evaluation_complete().unwrap();
        let err = lc.end().expect_err("end from Reviewing is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn start_session_from_recording_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        let err = lc
            .guard_start_session_state()
            .expect_err("start from Recording is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn start_session_from_evaluating_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        test_end(&mut lc);
        let err = lc
            .guard_start_session_state()
            .expect_err("start from Evaluating is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn cancel_from_idle_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        let err = lc.cancel().expect_err("cancel from Idle is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn cancel_from_evaluating_returns_invalid_state() {
        // Per ADD §3.3.1, cancel during Evaluating is intentionally not
        // exposed in the UX; the lifecycle rejects it to match.
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        test_end(&mut lc);
        let err = lc.cancel().expect_err("cancel from Evaluating is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn cancel_from_reviewing_returns_invalid_state() {
        // Leaving Reviewing happens via start_session (practice again) or by
        // closing the surface (handled at the UI layer); `cancel` is not the
        // legal way to leave.
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        test_end(&mut lc);
        lc.mark_evaluation_complete().unwrap();
        let err = lc.cancel().expect_err("cancel from Reviewing is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn mark_evaluation_complete_from_recording_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        test_start(&mut lc);
        let err = lc
            .mark_evaluation_complete()
            .expect_err("mark_evaluation_complete from Recording is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    #[test]
    fn mark_error_from_idle_returns_invalid_state() {
        let mut lc = SessionLifecycle::new();
        let err = lc
            .mark_error(MicrophoneError::DeviceDisconnected)
            .expect_err("mark_error from Idle is invalid");
        assert!(matches!(err, AppError::InvalidState(_)));
    }

    // --- cycle history invariant -----------------------------------------

    #[test]
    fn three_cycle_practice_loop_preserves_distinct_ids() {
        let mut lc = SessionLifecycle::new();

        // Cycle 1: Idle → Recording → Evaluating → Reviewing.
        let c1 = test_start(&mut lc);
        test_end(&mut lc);
        lc.mark_evaluation_complete().unwrap();

        // Cycle 2: Reviewing → Recording (practice again) → ... → Reviewing.
        let c2 = test_start(&mut lc);
        test_end(&mut lc);
        lc.mark_evaluation_complete().unwrap();

        // Cycle 3: Reviewing → Recording (practice again).
        let c3 = test_start(&mut lc);

        // History invariant: prior two cycles archived, current is third.
        assert_eq!(
            lc.completed_session_ids(),
            &[c1.clone(), c2.clone()],
            "first two cycle ids preserved in history",
        );
        assert_eq!(lc.current_session_id(), Some(&c3));
        // All three distinct.
        assert_ne!(c1, c2);
        assert_ne!(c2, c3);
        assert_ne!(c1, c3);
    }

    // --- Arc<...> Send + Sync sanity (compile-time) ----------------------

    /// Compile-time check that callbacks of the production shape — `Arc`-wrapped
    /// closures that emit Tauri events — satisfy the bounds `start_session`
    /// requires (`Fn(f32) + Send + Sync + 'static`). Mirrors the closure shape
    /// used in `commands.rs`.
    #[test]
    fn level_callback_bounds_compile() {
        fn assert_send_sync<T: Fn(f32) + Send + Sync + 'static>(_: &T) {}
        let cb = Arc::new(|_rms: f32| {});
        assert_send_sync(&*cb);
    }
}
