-- 001_init.sql (committed; first migration)
-- Generic app-state spine for the desktop-control-panel starter.
--
-- Two singleton tables: install_identity and settings (both CHECK id = 1).
-- install_identity carries the per-install uuid + consent/registration
-- timestamps. settings carries a small GENERIC example preference set
-- (`theme`) to demonstrate the get/set round-trip; product layers extend it.
--
-- This is the SPINE only — the migration engine + singleton-state shape are
-- kept so a downstream app can demonstrate the data-lifecycle. No
-- application-domain tables live here; an app adds those in its own
-- migrations.

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
  theme TEXT NOT NULL DEFAULT 'system'
    CHECK (theme IN ('system', 'light', 'dark')),
  schema_version INTEGER NOT NULL DEFAULT 1
);
