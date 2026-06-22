//! CL-24 — first-run model download with streamed digest verification.
//!
//! Spec: TRD §4.8 (T-DST-1/2/3 — weights are not bundled; fetched once from a
//! pinned HTTPS URL and verified against a build-time SHA-256), ADD §3.6 (the
//! `start_first_run_model_download` command), ADD §3.7 (the `model_download:*`
//! event surface). The frontend contract already exists
//! (`app/src/ipc/commands.ts::startFirstRunModelDownload`,
//! `app/src/ipc/events.ts::{listenModelDownloadProgress,…Done,…Error}`); this
//! module is the missing Rust side.
//!
//! # Shape
//!
//! The 339 MB model cannot ship in the installer, so on first launch the
//! client streams it from [`BuildConfig::MODEL_URL`] into
//! `<model_cache_dir>/<MODEL_URL terminal segment>` (the exact path
//! [`crate::evaluation::orchestrator::resolve_model_path`] returns), hashing as
//! it goes. The digest is checked **before** the file is committed to the cache
//! path: bytes land in a sibling `*.part` temp file, and only a digest match
//! renames it into place (T-DST-2). A mismatch deletes the partial.
//!
//! # Testability
//!
//! The logic-bearing pieces are pure and unit-tested: [`stream_to_file`] (the
//! write + hash + progress loop, exercised with a canned in-memory stream),
//! [`commit_or_reject`] (rename-on-match / delete-on-mismatch), and
//! [`is_cached_valid`] (idempotency). The only untested glue is the reqwest GET
//! in [`open_response`] — the same split the `update` module makes between its
//! tested `check_for_update_impl` and its thin `ReqwestFetcher`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use futures_util::StreamExt;
use sha2::{Digest, Sha256};
use tauri::Emitter;

use crate::shared::config::BuildConfig;
use crate::shared::error::AppError;
use crate::AppState;

/// Event channel names — mirror `app/src/ipc/events.ts` (`EVT_MODEL_DOWNLOAD_*`).
const EVT_PROGRESS: &str = "model_download:progress";
const EVT_DONE: &str = "model_download:done";
const EVT_ERROR: &str = "model_download:error";

/// Buffered read size when re-hashing an already-cached file. Matches the 64 KiB
/// window used by the phonemizer's `sha256_of_file`.
const HASH_BUF_BYTES: usize = 64 * 1024;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Failure modes of the first-run download. These surface to the frontend as
/// the `error` string of the `model_download:error` event (ADD §3.7), not as an
/// [`AppError`] — the command returns immediately and the download runs async,
/// so the only `AppError` the command itself can return is the re-entrancy
/// `InvalidState`.
#[derive(Debug, thiserror::Error)]
pub enum ModelDownloadError {
    /// The HTTP GET failed, returned a non-success status, or the body stream
    /// errored mid-transfer.
    #[error("network error: {0}")]
    Network(String),

    /// Writing the partial file or renaming it into the cache failed.
    #[error("io error: {0}")]
    Io(String),

    /// The fully-downloaded file's SHA-256 did not match
    /// [`BuildConfig::MODEL_SHA256`]. The partial is deleted before this is
    /// returned (T-DST-2).
    #[error("model digest mismatch: expected {expected}, got {actual}")]
    DigestMismatch { expected: String, actual: String },
}

// ---------------------------------------------------------------------------
// Pure, unit-tested pieces
// ---------------------------------------------------------------------------

/// Stream `stream` into `temp_path`, updating a running SHA-256 and invoking
/// `on_progress(bytes_done, bytes_total)` after each chunk. Returns the
/// lowercase hex digest of everything written.
///
/// Generic over the chunk type (`AsRef<[u8]>`) and the stream error so tests
/// can drive it with `futures_util::stream::iter` of `Vec<u8>` and production
/// can hand it a `reqwest` `bytes_stream()` directly.
pub(crate) async fn stream_to_file<S, B, E>(
    mut stream: S,
    temp_path: &Path,
    bytes_total: u64,
    mut on_progress: impl FnMut(u64, u64),
) -> Result<String, ModelDownloadError>
where
    S: futures_util::Stream<Item = Result<B, E>> + Unpin,
    B: AsRef<[u8]>,
    E: std::fmt::Display,
{
    use std::io::Write as _;

    let mut file =
        std::fs::File::create(temp_path).map_err(|e| ModelDownloadError::Io(e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut bytes_done: u64 = 0;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| ModelDownloadError::Network(e.to_string()))?;
        let bytes = chunk.as_ref();
        file.write_all(bytes)
            .map_err(|e| ModelDownloadError::Io(e.to_string()))?;
        hasher.update(bytes);
        bytes_done += bytes.len() as u64;
        on_progress(bytes_done, bytes_total);
    }

    file.flush().map_err(|e| ModelDownloadError::Io(e.to_string()))?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Gate the digest, then either commit the temp file to `dest_path` (rename on
/// match) or delete it and return [`ModelDownloadError::DigestMismatch`]. The
/// comparison is case-insensitive hex equality, matching the phonemizer's load
/// gate. The rename is the commit point — nothing reaches the cache path until
/// the digest is verified (T-DST-2).
pub(crate) fn commit_or_reject(
    temp_path: &Path,
    dest_path: &Path,
    actual_sha256: &str,
    expected_sha256: &str,
) -> Result<(), ModelDownloadError> {
    let actual = actual_sha256.to_ascii_lowercase();
    let expected = expected_sha256.to_ascii_lowercase();
    if actual != expected {
        // Best-effort cleanup; a leftover *.part is harmless (overwritten next
        // attempt) but we remove it so a failed download leaves no debris.
        let _ = std::fs::remove_file(temp_path);
        return Err(ModelDownloadError::DigestMismatch { expected, actual });
    }
    std::fs::rename(temp_path, dest_path).map_err(|e| ModelDownloadError::Io(e.to_string()))
}

/// True when `dest_path` already holds a file whose SHA-256 equals
/// `expected_sha256` — the idempotency check that lets a re-launch skip the
/// download entirely. Any read error (missing file, permission) is treated as
/// "not valid → (re)download", never as an error.
pub(crate) fn is_cached_valid(dest_path: &Path, expected_sha256: &str) -> bool {
    match sha256_of_file(dest_path) {
        // `sha256_of_file` returns lowercase hex (`format!("{:x}")`), so only
        // `expected` needs normalizing here.
        Ok(actual) => actual == expected_sha256.to_ascii_lowercase(),
        Err(_) => false,
    }
}

/// Streamed SHA-256 of a file as lowercase hex. Local copy of the phonemizer's
/// helper (kept private there) so this module has no cross-feature dependency
/// on `evaluation::phonemizer` internals.
fn sha256_of_file(path: &Path) -> std::io::Result<String> {
    use std::io::Read as _;

    let file = std::fs::File::open(path)?;
    let mut reader = std::io::BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buf = [0u8; HASH_BUF_BYTES];
    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Sibling temp path the download streams into before the digest gate. Appends
/// `.part` to the destination filename so it sits in the same directory (making
/// the final rename atomic on the same volume).
fn part_path(dest_path: &Path) -> PathBuf {
    let mut name = dest_path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    dest_path.with_file_name(name)
}

// ---------------------------------------------------------------------------
// Reqwest glue (thin, untested at the unit level)
// ---------------------------------------------------------------------------

/// GET `url`, following redirects, and return the success [`reqwest::Response`]
/// (caller reads `content_length()` then `bytes_stream()`). Returning the
/// `Response` rather than the stream avoids naming reqwest's re-exported
/// `Bytes` type, so this module needs no direct `bytes` dependency.
async fn open_response(url: &str) -> Result<reqwest::Response, ModelDownloadError> {
    reqwest::get(url)
        .await
        .map_err(|e| ModelDownloadError::Network(e.to_string()))?
        .error_for_status()
        .map_err(|e| ModelDownloadError::Network(e.to_string()))
}

/// Full first-run download: idempotency check → stream to temp → digest gate →
/// commit. `on_progress` is forwarded to [`stream_to_file`]. Returns `Ok(())`
/// both when the model is freshly downloaded and when a valid copy was already
/// cached.
pub(crate) async fn download_model(
    url: &str,
    expected_sha256: &str,
    dest_path: &Path,
    on_progress: impl FnMut(u64, u64),
) -> Result<(), ModelDownloadError> {
    // Idempotent: a present, digest-valid file short-circuits with no network.
    // Runs before `create_dir_all` below — safe because `is_cached_valid`
    // treats a missing file or directory as "not cached" (`Err(_) => false`),
    // never an error, so the clean-machine cache-miss path falls through here.
    if is_cached_valid(dest_path, expected_sha256) {
        return Ok(());
    }

    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| ModelDownloadError::Io(e.to_string()))?;
    }

    let temp = part_path(dest_path);
    let response = open_response(url).await?;
    // 0 when the server omits Content-Length — the UI renders bytes_total == 0
    // as an indeterminate spinner rather than a percentage.
    let total = response.content_length().unwrap_or(0);
    let stream = std::pin::pin!(response.bytes_stream());
    let actual = stream_to_file(stream, &temp, total, on_progress).await?;
    commit_or_reject(&temp, dest_path, &actual, expected_sha256)
}

// ---------------------------------------------------------------------------
// Tauri command
// ---------------------------------------------------------------------------

/// `start_first_run_model_download` (ADD §3.6). Validates that no download is
/// already running (returns [`AppError::InvalidState`] if one is), then spawns
/// the streamed download and returns immediately. Progress, completion, and
/// failure are reported asynchronously via the `model_download:*` events.
#[tauri::command]
pub async fn start_first_run_model_download(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), AppError> {
    // Re-entrancy guard: reject a second concurrent invocation rather than
    // racing two writers onto the same `*.part`. swap returns the prior value;
    // if it was already true, a download is in flight.
    if state.model_download_in_progress.swap(true, Ordering::SeqCst) {
        return Err(AppError::InvalidState(
            "model download already in progress".into(),
        ));
    }

    let dest = match crate::evaluation::orchestrator::resolve_model_path(&app) {
        Ok(p) => p,
        Err(e) => {
            // Releasing the guard on the early-return path keeps a config
            // failure from wedging the feature for the rest of the process.
            state.model_download_in_progress.store(false, Ordering::SeqCst);
            return Err(e);
        }
    };

    let url = BuildConfig::MODEL_URL;
    let expected = BuildConfig::MODEL_SHA256;
    let in_progress: Arc<AtomicBool> = state.model_download_in_progress.clone();

    tokio::spawn(async move {
        let progress_app = app.clone();
        // Throttle progress emits. `stream_to_file` invokes the callback after
        // every reqwest chunk (8–64 KiB), so an un-throttled emit fires tens of
        // thousands of `model_download:progress` events for the ~339 MB model —
        // enough to saturate the Tauri event channel and stall the very progress
        // bar it feeds on low-end pilot hardware. Emit only when progress crosses
        // a ≥1% delta (or ≥1 MiB when the server omits Content-Length), plus the
        // first chunk (immediate paint) and the final byte (so the bar reaches
        // 100%). The unit-tested `stream_to_file` signature is unchanged — the
        // throttle lives entirely in this production callback.
        const MIN_DELTA_BYTES: u64 = 1024 * 1024; // 1 MiB floor when total unknown
        let mut last_emitted: u64 = 0;
        let outcome = download_model(url, expected, &dest, move |bytes_done, bytes_total| {
            let step = if bytes_total > 0 {
                (bytes_total / 100).max(1)
            } else {
                MIN_DELTA_BYTES
            };
            let is_final = bytes_total > 0 && bytes_done >= bytes_total;
            if last_emitted == 0 || bytes_done - last_emitted >= step || is_final {
                last_emitted = bytes_done;
                let _ = progress_app.emit(
                    EVT_PROGRESS,
                    ProgressPayload {
                        bytes_done,
                        bytes_total,
                    },
                );
            }
        })
        .await;

        // Always release the guard before emitting the terminal event so a
        // retry after an error path is never blocked by a stuck flag.
        in_progress.store(false, Ordering::SeqCst);

        match outcome {
            Ok(()) => {
                let _ = app.emit(EVT_DONE, DonePayload {});
            }
            Err(e) => {
                log::error!("first-run model download failed: {e}");
                let _ = app.emit(EVT_ERROR, ErrorPayload { error: e.to_string() });
            }
        }
    });

    Ok(())
}

/// `model_download:progress` payload — mirrors `ModelDownloadProgressEvent`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProgressPayload {
    pub bytes_done: u64,
    pub bytes_total: u64,
}

/// `model_download:done` payload — empty object (`ModelDownloadDoneEvent`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DonePayload {}

/// `model_download:error` payload — mirrors `ModelDownloadErrorEvent`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorPayload {
    pub error: String,
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write as _;
    use tempfile::TempDir;

    /// SHA-256 of the bytes `b"hello world"` (lowercase hex). Precomputed so
    /// the digest-gate tests don't depend on re-deriving the expected value
    /// from the same code under test.
    const HELLO_SHA256: &str =
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9";

    fn canned_stream(chunks: Vec<&'static [u8]>) -> impl futures_util::Stream<Item = Result<Vec<u8>, std::io::Error>> {
        futures_util::stream::iter(
            chunks.into_iter().map(|c| Ok(c.to_vec())).collect::<Vec<_>>(),
        )
    }

    #[tokio::test]
    async fn stream_to_file_writes_bytes_and_returns_digest() {
        let dir = TempDir::new().unwrap();
        let temp = dir.path().join("m.onnx.part");
        let stream = canned_stream(vec![b"hello ", b"world"]);

        let digest = stream_to_file(Box::pin(stream), &temp, 11, |_, _| {})
            .await
            .expect("stream completes");

        assert_eq!(digest, HELLO_SHA256);
        assert_eq!(std::fs::read(&temp).unwrap(), b"hello world");
    }

    #[tokio::test]
    async fn stream_to_file_reports_cumulative_progress() {
        let dir = TempDir::new().unwrap();
        let temp = dir.path().join("m.onnx.part");
        let stream = canned_stream(vec![b"hello ", b"world"]);

        let mut samples: Vec<(u64, u64)> = Vec::new();
        stream_to_file(Box::pin(stream), &temp, 11, |done, total| {
            samples.push((done, total))
        })
        .await
        .unwrap();

        // One callback per chunk, byte counts accumulate, total is constant.
        assert_eq!(samples, vec![(6, 11), (11, 11)]);
    }

    #[tokio::test]
    async fn stream_to_file_surfaces_stream_error() {
        let dir = TempDir::new().unwrap();
        let temp = dir.path().join("m.onnx.part");
        let stream = futures_util::stream::iter(vec![
            Ok(b"partial".to_vec()),
            Err(std::io::Error::new(std::io::ErrorKind::ConnectionReset, "dropped")),
        ]);

        let err = stream_to_file(Box::pin(stream), &temp, 99, |_, _| {})
            .await
            .expect_err("mid-stream error propagates");
        assert!(matches!(err, ModelDownloadError::Network(_)));
    }

    #[test]
    fn commit_or_reject_renames_on_match() {
        let dir = TempDir::new().unwrap();
        let temp = dir.path().join("m.onnx.part");
        let dest = dir.path().join("m.onnx");
        std::fs::write(&temp, b"hello world").unwrap();

        commit_or_reject(&temp, &dest, HELLO_SHA256, HELLO_SHA256).expect("commits");

        assert!(dest.exists(), "destination created on digest match");
        assert!(!temp.exists(), "temp consumed by the rename");
        assert_eq!(std::fs::read(&dest).unwrap(), b"hello world");
    }

    #[test]
    fn commit_or_reject_is_case_insensitive() {
        let dir = TempDir::new().unwrap();
        let temp = dir.path().join("m.onnx.part");
        let dest = dir.path().join("m.onnx");
        std::fs::write(&temp, b"hello world").unwrap();

        commit_or_reject(&temp, &dest, &HELLO_SHA256.to_uppercase(), HELLO_SHA256)
            .expect("uppercase actual still matches");
        assert!(dest.exists());
    }

    #[test]
    fn commit_or_reject_deletes_partial_on_mismatch() {
        let dir = TempDir::new().unwrap();
        let temp = dir.path().join("m.onnx.part");
        let dest = dir.path().join("m.onnx");
        std::fs::write(&temp, b"hello world").unwrap();

        let wrong = "0".repeat(64);
        let err = commit_or_reject(&temp, &dest, HELLO_SHA256, &wrong)
            .expect_err("digest mismatch rejected");

        match err {
            ModelDownloadError::DigestMismatch { expected, actual } => {
                assert_eq!(expected, wrong);
                assert_eq!(actual, HELLO_SHA256);
            }
            other => panic!("expected DigestMismatch, got {other:?}"),
        }
        assert!(!dest.exists(), "nothing committed to cache on mismatch");
        assert!(!temp.exists(), "partial deleted on mismatch");
    }

    #[test]
    fn is_cached_valid_true_for_matching_file() {
        let dir = TempDir::new().unwrap();
        let dest = dir.path().join("m.onnx");
        let mut f = std::fs::File::create(&dest).unwrap();
        f.write_all(b"hello world").unwrap();

        assert!(is_cached_valid(&dest, HELLO_SHA256));
        assert!(is_cached_valid(&dest, &HELLO_SHA256.to_uppercase()));
    }

    #[test]
    fn is_cached_valid_false_for_missing_or_wrong_file() {
        let dir = TempDir::new().unwrap();
        let dest = dir.path().join("m.onnx");

        // Missing file → not valid, not an error.
        assert!(!is_cached_valid(&dest, HELLO_SHA256));

        std::fs::write(&dest, b"different bytes").unwrap();
        assert!(!is_cached_valid(&dest, HELLO_SHA256));
    }

    #[tokio::test]
    async fn download_model_short_circuits_when_already_cached() {
        // A valid cached file means download_model returns Ok without ever
        // touching the network — open_response's `example.invalid` URL would
        // fail if it were reached, so success proves the short-circuit.
        let dir = TempDir::new().unwrap();
        let dest = dir.path().join("m.onnx");
        std::fs::write(&dest, b"hello world").unwrap();

        download_model("https://unreachable.invalid/m.onnx", HELLO_SHA256, &dest, |_, _| {})
            .await
            .expect("cached file short-circuits the download");
    }

    #[test]
    fn part_path_appends_suffix_in_same_dir() {
        let p = part_path(Path::new("/cache/models/ias-model-0.1.0.onnx"));
        assert_eq!(
            p,
            Path::new("/cache/models/ias-model-0.1.0.onnx.part")
        );
    }
}
