-- 003_feedback.sql
-- App-level free-form feedback captured from the "Give Feedback" modal.
--
-- Standalone (not session-scoped): the feedback control is now a global header
-- entry point, so a submission is not tied to any one practice session. Stored
-- ON DEVICE ONLY for V1 — nothing is transmitted. A future opt-in egress path
-- (FRD amendment required) would read from this table; the schema is shaped to
-- make that flip additive (no PII columns, just the note + optional rating +
-- timestamp).

CREATE TABLE feedback (
  id           INTEGER PRIMARY KEY AUTOINCREMENT,
  rating       INTEGER,             -- optional 1..5; NULL when the user only left a note
  note         TEXT,                -- free-form suggestion text; NULL when only a rating
  submitted_at TEXT NOT NULL        -- RFC3339 timestamp of capture
);
