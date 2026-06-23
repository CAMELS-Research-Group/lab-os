# Desktop Control-Panel Starter

A domain-free Tauri desktop control-panel walking skeleton: a React frontend +
a Rust (Tauri 2) backend with a small, generic capability spine — first-run
gate, settings (theme + opt-in update checks), local SQLite app data, an
about/version surface, and an optional self-update banner. It deliberately
carries **no product domain logic**; fork it and build your own app surface on
top.

## Rebrand after forking

This starter ships with explicit placeholder identity so a fork can find and
replace every branding point by grepping for three tokens:

| Token | Where it appears | Replace with |
|-------|------------------|--------------|
| `[DEFAULT]` | Human-facing display names — window title, app title, `productName`, `index.html` `<title>`, UI header/brand copy | Your app's display name |
| `default-app` | Machine identifiers — npm `package.json` `name`, Cargo crate `name`, local-storage key namespace | Your package/crate name (kebab-case) |
| `com.example.default` | Tauri bundle `identifier` in `src-tauri/tauri.conf.json` | Your reverse-DNS bundle id |

```bash
# From the repo root — see every branding point to change:
grep -rn -e '\[DEFAULT\]' -e 'default-app' -e 'com.example.default' .
```

`tauri.conf.json` and `package.json` are strict JSON (no inline comments), so
their placeholder values are documented here rather than in the files.

## Prerequisites

- Node 20+ and npm
- Rust stable (`rustup`) and the platform build deps:
  - **Windows:** the MSVC build tools (Desktop development with C++)
  - **macOS:** Xcode command-line tools
  - **Linux:** the standard Tauri 2 system deps (webkit2gtk, etc.)

## Run / build / test

```bash
cd app
npm install               # one time

npm run tauri dev         # run the full desktop app (first launch builds Rust; 3-8 min)
npm run dev               # frontend only, in a browser at http://localhost:1420

npm run build             # frontend production build (tsc + vite)
npm test                  # frontend unit tests (vitest)

# Rust backend:
cargo test  --manifest-path src-tauri/Cargo.toml --lib
cargo build --manifest-path src-tauri/Cargo.toml --lib

# Full desktop bundle (.msi / .dmg):
npm run tauri build                  # uses safe defaults (updater disabled)
npm run tauri:build:release          # loads .env for release config (see below)
```

## Releases and auto-update (optional, configure post-fork)

Auto-update is **optional and off by default**. The app builds and runs with no
release host configured: the updater's manifest URL and the release download
page both fall back to an unresolvable `example.invalid` sentinel (RFC 6761),
and the update check is additionally gated behind an opt-in setting that is off
by default. So a fresh fork builds clean, makes no network calls, and the update
check reports "no update" — there is no required release host and nothing breaks
if it is left unset.

To enable auto-update for your fork, configure the release host **after
forking**. There is a single place to set it: the build-time environment
variables in **`app/.env`** (template: `app/.env.example`), which feed
`BuildConfig` in `app/src-tauri/src/shared/config.rs` and the release download
page in `app/src-tauri/src/update/commands.rs`:

- `UPDATER_MANIFEST_URL` — your release feed (e.g. a GitHub Releases
  `latest.json`)
- `RELEASE_DOWNLOAD_PAGE_URL` — the page the "Download update" button opens
- `UPDATER_PUBKEY` — updater signing public key, if used
- `APP_VERSION` — optional; defaults to the Cargo package version

```bash
cd app
cp .env.example .env       # then edit .env with your host
npm run tauri:build:release
```

The CI release workflow lives at `.github/workflows/app-release.yml` (repo
root). It is **fork-portable** — it builds and publishes to *your* repo's GitHub
Releases automatically (`${{ github.repository }}`), so there is no publish target
to repoint. Before the gated publish job will run, a fork must:
- set a `RELEASE_TOKEN` secret (contents:write on your repo), bound to a `release`
  Environment;
- provide `app/tools/release/gen-latest-json.mjs` — a small Node (stdlib)
  generator that writes each channel's `latest.json`. It is **not bundled** (the
  source app's was deployment-specific); the build legs work without it, but the
  publish leg is non-functional until you add or replace it.

## What this is not

No ESL/pronunciation/domain logic, no bundled ML model, no backend service
dependency — those were stripped from the source app. Companion docs at the
starter root (`STRIPPED.md`, `EXTENSION-POINTS.md`) cover the provenance of what
was removed and how to add command modules, screens, migrations, and a release
host.
