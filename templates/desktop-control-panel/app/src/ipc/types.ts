/**
 * IPC payload types — TypeScript side of the contract.
 *
 * Hand-mirrored with `src-tauri/src/shared/types.rs` (Rust side). If you change
 * a type here, change the matching struct in `src-tauri/src/shared/types.rs` in
 * the same PR. Codegen for a single source of truth is post-V1.
 *
 * Field names are snake_case throughout — no camelCase remapping at the IPC
 * boundary (Rust's serde rename_all = "snake_case" controls the wire format).
 *
 * SCOPE: the generic-shell command surface. The IPC layer binds exactly the
 * surviving Rust commands — settings, theme, update, and app-data — and nothing
 * else. The former ESL contract (read-aloud reference content, sound-level
 * analysis, recording, and reporting types) was removed with the ESL UI strip.
 */

// ---------------------------------------------------------------------------
// Theme
// ---------------------------------------------------------------------------

/**
 * App theme preference. Mirrors the Rust enum's snake_case serialization.
 *
 * Wire shape: `"system" | "light" | "dark"`.
 */
export type Theme = 'system' | 'light' | 'dark';

// ---------------------------------------------------------------------------
// Settings
// ---------------------------------------------------------------------------

/**
 * User-mutable settings persisted across sessions. Mirrors `Settings` in
 * `shared/types.rs`.
 *
 * `report_uploads_enabled` gates optional report uploads; `update_checks_enabled`
 * gates periodic network egress to check for a new client version (off by
 * default — the user opts in via the Settings screen).
 */
export type Settings = {
  theme: Theme;
  report_uploads_enabled: boolean;
  update_checks_enabled: boolean;
};

// ---------------------------------------------------------------------------
// Update info
// ---------------------------------------------------------------------------

/**
 * Information about an available client update. `version` and `notes` are
 * `null` when no update is pending.
 */
export type UpdateInfo = {
  available: boolean;
  version: string | null;
  notes: string | null;
};

// ---------------------------------------------------------------------------
// Compile-time exhaustiveness checks
// ---------------------------------------------------------------------------

// Compile-time discrimination check for Theme. If a member is dropped from the
// union without updating the mapped-type below, `tsc` errors because the
// `Record<Theme, true>` key set would no longer be satisfied by the literal
// object. No runtime cost; tree-shaken in the Vite build.
const _ThemeExhaustive: Record<Theme, true> = {
  system: true,
  light: true,
  dark: true,
};
void _ThemeExhaustive;
