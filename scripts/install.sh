#!/usr/bin/env bash
set -euo pipefail

REPO_URL=""
DEFAULT_REPO_URL="https://github.com/GOLDhjy/CCSwitcher.git"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo)
      REPO_URL="${2:-}"
      shift 2
      ;;
    *)
      echo "Unknown argument: $1"
      echo "Usage: bash scripts/install.sh [--repo <git-url>]"
      exit 1
      ;;
  esac
done

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${INSTALL_BIN_DIR:-$HOME/.local/bin}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/ccswitcher-target}"

if [[ ! -f "$ROOT_DIR/Cargo.toml" ]]; then
  REPO_URL="${REPO_URL:-${CCSWITCHER_REPO_URL:-$DEFAULT_REPO_URL}}"

  TMP_DIR="$(mktemp -d)"
  trap 'rm -rf "$TMP_DIR"' EXIT
  echo "Cloning repository: $REPO_URL"
  git clone --depth 1 "$REPO_URL" "$TMP_DIR/CCSwitcher"
  exec bash "$TMP_DIR/CCSwitcher/scripts/install.sh"
fi

echo "Building ccswitcher..."
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

mkdir -p "$BIN_DIR"
cp "$TARGET_DIR/release/ccswitcher" "$BIN_DIR/ccswitcher"
chmod +x "$BIN_DIR/ccswitcher"

"$BIN_DIR/ccswitcher" install

echo
echo "Installed binary: $BIN_DIR/ccswitcher"
echo "Installed slash command via: $BIN_DIR/ccswitcher install"
echo
echo "If needed, add this to your shell rc:"
echo "  export PATH=\"$BIN_DIR:\$PATH\""
