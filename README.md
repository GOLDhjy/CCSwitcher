# CCSwitcher

`ccswitcher` is a Rust CLI for Claude Code model preset management.

It powers a global slash command `/switchmodel` with subcommands:

- `list`
- `current`
- `use <preset>`
- `add`
- `remove <preset>`

## What it changes

- Presets are stored in `~/.claudecode-switcher/config.json`
- Active preset is applied to `~/.claude/settings.json` under `env`

Updated environment keys:

- `ANTHROPIC_DEFAULT_HAIKU_MODEL`
- `ANTHROPIC_DEFAULT_SONNET_MODEL`
- `ANTHROPIC_DEFAULT_OPUS_MODEL`
- `ANTHROPIC_AUTH_TOKEN`
- `ANTHROPIC_BASE_URL`
- `API_TIMEOUT_MS` (optional)
- `MCP_TOOL_TIMEOUT` (optional)
- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC` (optional)
- `HTTP_PROXY` (optional)

`settings.json` writes are atomic and create a timestamped backup.

## Rust toolchain

This project pins Rust in `rust-toolchain.toml`:

- `1.92.0`

## Build

```bash
cargo build --release
```

If your project is on a filesystem that does not support Rust lock files, use:

```bash
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/ccswitcher-target cargo build --release
```

## CLI usage

```bash
ccswitcher list
ccswitcher current
ccswitcher use glm

ccswitcher add \
  --name glm-work \
  --provider glm \
  --base-url https://open.bigmodel.cn/api/anthropic \
  --auth-token your-token \
  --haiku GLM-4.7 \
  --sonnet GLM-4.7 \
  --opus GLM-4.7 \
  --http-proxy http://127.0.0.1:10809 \
  --api-timeout-ms 3000000 \
  --mcp-tool-timeout 30000 \
  --disable-nonessential-traffic true

ccswitcher remove glm-work
```

## Install slash command

```bash
bash scripts/install.sh
```

This installs:

- `~/.local/bin/ccswitcher`
- `~/.claude/commands/switchmodel.md`

After that, use in Claude Code:

- `/switchmodel list`
- `/switchmodel current`
- `/switchmodel use glm-work`
- `/switchmodel add` (interactive question flow in chat)
- `/switchmodel remove glm-work`

## Security note

Auth tokens are stored in plain text by design for this version.
Rotate tokens if they were exposed and keep file permissions restricted.
