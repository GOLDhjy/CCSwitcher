# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`ccswitcher` is a Rust CLI tool that manages Claude Code model presets. It provides a global slash command `/switchmodel` that allows switching between different API providers (e.g., GLM, custom Anthropic-compatible endpoints) and model configurations.

**Key behavior**: The tool modifies `~/.claude/settings.json` to set environment variables like `ANTHROPIC_BASE_URL`, `ANTHROPIC_AUTH_TOKEN`, and model names (`ANTHROPIC_DEFAULT_HAIKU_MODEL`, etc.). Presets are stored in `~/.claudecode-switcher/config.json`.

## Build Commands

```bash
# Standard release build
cargo build --release

# If the filesystem doesn't support Rust lock files (e.g., networked drive)
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/ccswitcher-target cargo build --release

# Run tests
cargo test

# Run a specific test
cargo test test_name

# CLI testing (uses built binary in target/debug/)
./target/debug/ccswitcher list
```

## Installation & Distribution

The install script (`scripts/install.sh`) supports two modes:
- Binary mode: Downloads prebuilt release from GitHub
- Source mode: Builds locally with `cargo build --release`

Installation places:
- Binary: `~/.local/bin/ccswitcher`
- Slash command template: `~/.claude/commands/switchmodel.md`

Releases are automated via GitHub Actions (`.github/workflows/release.yml`) on version tags like `v*`.

## Architecture

### Module Structure

- **`cli.rs`**: Clap-based CLI argument parsing. Defines `Commands` enum and `AddArgs` struct.
- **`config.rs`**: `SwitcherConfig` and `Preset` structs with JSON serialization. Handles preset validation.
- **`settings.rs`**: Modifies Claude's `settings.json`. Applies preset values to the `env` map, preserving non-env fields.
- **`paths.rs`**: Path resolution using environment variables: `CCSWITCHER_HOME` (defaults to `~/.claudecode-switcher`), `CLAUDE_HOME` (defaults to `~/.claude`).
- **`fsutil.rs`**: Atomic file writes and timestamped backups. Uses temp-file + rename pattern for safety.
- **`provider.rs`**: `ProviderKind` enum (`Glm`, `Custom`).
- **`errors.rs`**: `AppError` enum with `thiserror` for error display and context.
- **`lib.rs`**: Main command handlers (`use_preset`, `add_preset`, etc.) and interactive TUI menu.

### Environment Variables Modified by Presets

When applying a preset, these env keys are set in `~/.claude/settings.json`:
- `ANTHROPIC_DEFAULT_HAIKU_MODEL`, `ANTHROPIC_DEFAULT_SONNET_MODEL`, `ANTHROPIC_DEFAULT_OPUS_MODEL`
- `ANTHROPIC_AUTH_TOKEN`, `ANTHROPIC_BASE_URL`
- `API_TIMEOUT_MS`, `MCP_TOOL_TIMEOUT`
- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`
- `HTTP_PROXY`

### Slash Command Integration

The `/switchmodel` slash command is generated from `templates/switchmodel.md`. Installation also adds a permission rule `Bash(ccswitcher:*)` to `~/.claude/settings.local.json` so Claude can run the binary without prompts.

## Testing Strategy

- Unit tests are embedded in module files (e.g., `config.rs`, `settings.rs`)
- Integration tests are in `tests/cli_flow.rs` using `assert_cmd` and `tempfile`
- Tests isolate filesystem paths via `CCSWITCHER_HOME` and `CLAUDE_HOME` environment variables

## Rust Toolchain

Pinned to `stable` via `rust-toolchain.toml`. Edition 2024.

## Security Note

Auth tokens are stored in plain text in `~/.claudecode-switcher/config.json`. This is a known limitation noted in the README.
