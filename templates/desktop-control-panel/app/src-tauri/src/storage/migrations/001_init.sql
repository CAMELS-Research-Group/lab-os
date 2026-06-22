-- 001_init.sql (committed; first migration)
-- IAS client SQLite schema per ADD §3.8.
--
-- Four tables: install_identity, settings, sessions, upload_queue.
-- install_identity and settings are singletons (CHECK id = 1). sessions
-- carries the v0.4 named-difficulty fields (difficulty_level +
-- difficulty_thresholds_json + threshold_table_version). upload_queue holds
-- the at-rest outbound queue with FK ON DELETE CASCADE to sessions.

CREATE TABLE install_identity (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  uuid TEXT NOT NULL UNIQUE,
  consent_granted_at TEXT,
  consent_revoked_at TEXT,
  registered_at TEXT,
  schema_version INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  l1 TEXT NOT NULL DEFAULT '',
  regional_variety TEXT,
  difficulty TEXT NOT NULL DEFAULT 'gentle'
    CHECK (difficulty IN ('gentle', 'standard', 'strict')),
  schema_version INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE sessions (
  session_id TEXT PRIMARY KEY,
  started_at TEXT NOT NULL,
  ended_at TEXT NOT NULL,
  duration_seconds INTEGER NOT NULL,
  l1_at_session TEXT NOT NULL,
  regional_variety_at_session TEXT,
  phoneme_attempts_json TEXT NOT NULL,
  difficulty_level TEXT NOT NULL,
  difficulty_thresholds_json TEXT NOT NULL,
  threshold_table_version INTEGER NOT NULL,
  reattempt_counts_json TEXT NOT NULL,
  cumulative_session_count INTEGER NOT NULL,
  app_version TEXT NOT NULL,
  model_version TEXT NOT NULL,
  os_family TEXT NOT NULL,
  os_major TEXT NOT NULL,
  learner_rating INTEGER,
  learner_note TEXT,
  feedback_submitted_at TEXT
);
CREATE INDEX idx_sessions_started ON sessions(started_at);

CREATE TABLE upload_queue (
  queue_id INTEGER PRIMARY KEY AUTOINCREMENT,
  session_id TEXT NOT NULL,
  payload_kind TEXT NOT NULL CHECK (payload_kind IN ('session_report')),
  payload_json TEXT NOT NULL,
  queued_at TEXT NOT NULL,
  last_attempt_at TEXT,
  next_attempt_at TEXT,
  attempt_count INTEGER NOT NULL DEFAULT 0,
  last_error TEXT,
  FOREIGN KEY (session_id) REFERENCES sessions(session_id) ON DELETE CASCADE
);
CREATE INDEX idx_upload_queue_next_attempt ON upload_queue(next_attempt_at);
-- FK on upload_queue.session_id is enforced by SQLite, but it doesn't auto-create a child-side index.
-- This index keeps ON DELETE CASCADE scans cheap.
CREATE INDEX idx_upload_queue_session ON upload_queue(session_id);
