#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="${INSTALL_BIN_DIR:-$HOME/.local/bin}"
CLAUDE_COMMANDS_DIR="${CLAUDE_COMMANDS_DIR:-$HOME/.claude/commands}"
TARGET_DIR="${CARGO_TARGET_DIR:-/tmp/ccswitcher-target}"

echo "Building ccswitcher..."
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$TARGET_DIR" cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

mkdir -p "$BIN_DIR"
cp "$TARGET_DIR/release/ccswitcher" "$BIN_DIR/ccswitcher"
chmod +x "$BIN_DIR/ccswitcher"

mkdir -p "$CLAUDE_COMMANDS_DIR"
cp "$ROOT_DIR/templates/switchmodel.md" "$CLAUDE_COMMANDS_DIR/switchmodel.md"

echo
echo "Installed binary: $BIN_DIR/ccswitcher"
echo "Installed slash command: $CLAUDE_COMMANDS_DIR/switchmodel.md"
echo
echo "If needed, add this to your shell rc:"
echo "  export PATH=\"$BIN_DIR:\$PATH\""
