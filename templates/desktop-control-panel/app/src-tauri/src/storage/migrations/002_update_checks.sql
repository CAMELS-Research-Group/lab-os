-- 002_update_checks.sql
-- Adds the opt-in update-checks column to the settings singleton.
-- CL-23-lite: update checks are off by default (DEFAULT 0 = false).
-- Existing rows receive the default value; the column is NOT NULL.

ALTER TABLE settings
  ADD COLUMN update_checks_enabled INTEGER NOT NULL DEFAULT 0;
