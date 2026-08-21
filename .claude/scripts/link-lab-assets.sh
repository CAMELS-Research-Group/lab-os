#!/usr/bin/env bash
# Symlink this dev home's lab Claude assets (skills, commands, workflows) into the
# per-machine ~/.claude, so they load in every session regardless of the working
# directory. Project .claude/ assets are auto-discovered only up to the enclosing git
# repo root, and a nested project repo's own root blocks the walk-up to the dev home.
# Symlinks (not copies) keep this repo the single source of truth — edit here, live
# everywhere. Idempotent: refreshes stale links, never clobbers a real file or dir.
#
# Windows precondition: MSYS/Git-Bash emits real native symlinks only when
# MSYS=winsymlinks:nativestrict is set (done below) AND the shell holds the symlink
# privilege (Developer Mode enabled, or an elevated shell). Without it `ln -s` would
# deep-copy and exit 0, silently breaking the single-source contract — so the script
# probes the capability once up front and aborts with actionable guidance. macOS and
# Linux are unaffected.
#
# Usage:
#   bash .claude/scripts/link-lab-assets.sh             # link into ~/.claude
#   bash .claude/scripts/link-lab-assets.sh --dry-run   # show what would change
#   CLAUDE_HOME=/custom/.claude bash .claude/scripts/link-lab-assets.sh
#
# Run this on every machine you clone to — ~/.claude is per-machine and not committed.
# The links take effect in the next Claude Code session (asset lists load at startup).

set -euo pipefail

# On Windows/MSYS, make `ln -s` create real native symlinks — and, crucially, error
# out instead of silently deep-copying when the symlink privilege is missing. No-op
# on macOS/Linux.
export MSYS=winsymlinks:nativestrict

DRY_RUN=0
if (( $# > 1 )); then
  printf 'Too many arguments.\nUsage: %s [--dry-run]\n' "$0" >&2
  exit 2
fi
case "${1:-}" in
  "")        ;;
  --dry-run) DRY_RUN=1 ;;
  *) printf 'Unknown argument: %s\nUsage: %s [--dry-run]\n' "$1" "$0" >&2
     exit 2 ;;
esac

# This dev home's .claude/ dir is the parent of this script's directory — derived,
# never hardcoded, so the script is portable across clone paths and machines.
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SRC="$(cd "$SCRIPT_DIR/.." && pwd)"
DEST="${CLAUDE_HOME:-$HOME/.claude}"

linked=0
refreshed=0
skipped=0
skipped_targets=()

# Actionable guidance printed whenever a real symlink could not be created (the
# single failure message, shared by the preflight probe and the per-link asserts).
# Optional $1: the destination path that came back a copy instead of a symlink.
symlink_help() {
  {
    printf 'ERROR: could not create a native symlink'
    [[ -n "${1:-}" ]] && printf ' at %s' "$1"
    printf ' (got a copy instead).\n'
    printf 'On Windows, native symlinks require ONE of:\n'
    printf '  - Developer Mode enabled (Settings > Privacy & security > For developers), or\n'
    printf '  - running this shell elevated (Run as administrator).\n'
    printf 'Enable one and re-run. macOS/Linux need no extra privilege.\n'
  } >&2
}

# Preflight: prove this host can make a real symlink before touching ~/.claude, so a
# missing privilege surfaces as ONE clear message up front rather than once per asset
# mid-run. With MSYS=winsymlinks:nativestrict a privilege-less `ln -s` errors (caught
# by the `&&`); the `[[ -L ]]` check additionally catches any host that copied silently.
preflight_symlink_check() {
  local probe_dir probe_target probe_link
  probe_dir="$(mktemp -d)"
  probe_target="$probe_dir/target"
  probe_link="$probe_dir/link"
  : > "$probe_target"
  if ln -s "$probe_target" "$probe_link" 2>/dev/null && [[ -L "$probe_link" ]]; then
    rm -rf "$probe_dir"
    return 0
  fi
  rm -rf "$probe_dir"
  symlink_help "$probe_link"
  exit 1
}

# link_one <absolute-source> <destination>
link_one() {
  local src="$1" dest="$2" name
  name="$(basename "$dest")"

  if [[ -L "$dest" ]]; then
    # Existing symlink: leave it if already correct, else refresh.
    if [[ "$(readlink "$dest")" == "$src" ]]; then
      return 0
    fi
    if (( DRY_RUN == 0 )); then
      ln -sfn "$src" "$dest"
      [[ -L "$dest" ]] || { symlink_help "$dest"; exit 1; }
    fi
    printf '  refresh  %s\n' "$name"
    refreshed=$((refreshed + 1))
    return 0
  fi

  if [[ -e "$dest" ]]; then
    printf '  SKIP     %s (real file/dir already present — not overwriting)\n' "$name"
    skipped=$((skipped + 1))
    skipped_targets+=("$dest")
    return 0
  fi

  if (( DRY_RUN == 0 )); then
    ln -s "$src" "$dest"
    [[ -L "$dest" ]] || { symlink_help "$dest"; exit 1; }
  fi
  printf '  link     %s\n' "$name"
  linked=$((linked + 1))
}

# Fail fast if this host can't make real symlinks — before announcing or touching
# ~/.claude. Runs in dry-run too, so a preview also confirms feasibility.
preflight_symlink_check

SUFFIX=""
if (( DRY_RUN )); then SUFFIX="   (dry run — no changes made)"; fi
printf 'Linking lab assets%s\n  from: %s\n  into: %s\n\n' "$SUFFIX" "$SRC" "$DEST"

# Skills: directories holding a SKILL.md, plus skills-dir PLUGINS — directories holding a
# .claude-plugin/plugin.json, which Claude Code loads as <name>@skills-dir with no marketplace and
# no install step. Both are linked; anything else (ATTRIBUTION.md, stray files) is skipped.
#
# The plugin arm is load-bearing, not tidiness. A plugin's hooks are registered by the harness
# rather than invoked by the model, and at PROJECT scope a skills-dir plugin loads only from the
# .claude/skills/ of the directory Claude Code started in — it does not walk up to the repo root,
# so a session opened in a nested projects/<repo> clone would never see it. This symlink is what
# puts it in ~/.claude/skills/ at personal scope, where it loads in every session on the machine.
# A plugin whose whole job is to fire on an event nobody triggers by hand fails silently when it
# is missed, so discovery is asserted by that plugin's own wiring test rather than left to review.
if [[ -d "$SRC/skills" ]]; then
  echo "skills:"
  (( DRY_RUN )) || mkdir -p "$DEST/skills"
  for d in "$SRC"/skills/*/; do
    [[ -f "${d}SKILL.md" || -f "${d}.claude-plugin/plugin.json" ]] || continue
    link_one "${d%/}" "$DEST/skills/$(basename "$d")"
  done
fi

# Commands: *.md files.
if [[ -d "$SRC/commands" ]]; then
  echo "commands:"
  (( DRY_RUN )) || mkdir -p "$DEST/commands"
  for f in "$SRC"/commands/*.md; do
    [[ -e "$f" ]] || continue
    link_one "$f" "$DEST/commands/$(basename "$f")"
  done
fi

# Workflows: all regular files.
if [[ -d "$SRC/workflows" ]]; then
  echo "workflows:"
  (( DRY_RUN )) || mkdir -p "$DEST/workflows"
  for f in "$SRC"/workflows/*; do
    [[ -f "$f" ]] || continue
    link_one "$f" "$DEST/workflows/$(basename "$f")"
  done
fi

printf '\nDone: %d linked, %d refreshed, %d skipped.\n' "$linked" "$refreshed" "$skipped"
if (( skipped > 0 )); then
  # A skipped target is a real file/dir squatting where a symlink belongs (e.g. a
  # stale pre-symlink copy). The script never clobbers it; instead it prints the
  # exact back-up-then-rerun commands so the operator remediates deliberately. The
  # backup is moved to a dedicated, non-scanned dir (NOT a sibling `.bak`): a
  # `.bak` left in skills/ or workflows/ is itself discovered as a stray asset, so
  # the backup lands under $DEST/_pre-symlink-backups/ keyed by parent--name.
  {
    printf '\nTo link a skipped target, back it up (recoverable) then re-run this script:\n'
    printf '  mkdir -p %q\n' "$DEST/_pre-symlink-backups"
    for t in "${skipped_targets[@]}"; do
      printf '  mv %q %q\n' "$t" \
        "$DEST/_pre-symlink-backups/$(basename "$(dirname "$t")")--$(basename "$t")"
    done
    printf '  bash %q\n' "$SCRIPT_DIR/$(basename "${BASH_SOURCE[0]}")"
  } >&2
fi
