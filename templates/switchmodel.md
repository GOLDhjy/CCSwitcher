---
argument-hint: list | current | use <preset> | add | remove <preset> | reset
description: Manage model presets and switch Claude Code provider settings
allowed-tools: ["Bash(ccswitcher:*)", "Read"]
---

# switchmodel

Use the local `ccswitcher` binary to manage model presets.

## Executable resolution

- Use global `ccswitcher` from PATH only.
- Do not use project-local `./bin/ccswitcher`.
- If command is unavailable, tell user to run the installer and ensure PATH includes `~/.local/bin`.

## Strict command dispatch

- For valid subcommands, treat `/switchmodel` as a strict CLI wrapper.
- If subcommand is missing, provide a numbered quick menu and ask the user to choose.
- If subcommand is invalid, show the same numbered menu.

When no valid subcommand is provided, respond with this exact style:

```
Select an action:
1. list presets
2. show current preset
3. use a preset
4. add a preset
5. remove a preset
6. reset to official

Reply with a number (1-6).
```

## Behavior

- `list`: run `ccswitcher list`
- `current`: run `ccswitcher current`
- `use <preset>`: run `ccswitcher use <preset>`
- `remove <preset>`: run `ccswitcher remove <preset>`
- `reset`: run `ccswitcher reset-official`
- `add`: collect values step by step, then run a single `ccswitcher add ...` command

## `add` interactive flow (required)

For `/switchmodel add`, ask one question at a time in this exact order:

1. Preset name
2. Provider tag (optional, defaults to `custom`; `glm` is accepted)
3. Base URL
4. Auth token
5. Default model name (applied to all tiers initially)
6. Ask whether user wants separate models for `haiku` / `sonnet` / `opus`
7. Optional HTTP proxy
8. Optional API timeout (ms)
9. Optional MCP tool timeout (ms)
10. Optional disable nonessential traffic (`true`/`false`)

After answers:

1. Show a confirmation summary (mask token as `****`).
2. Only after user confirms, run one `ccswitcher add` command.
3. Return CLI output directly.

For all non-`add` subcommands:

- Run exactly one `ccswitcher` command.
- Return its output directly.
- Do not add extra dialog.

For numbered menu replies:

- `1` -> run `ccswitcher list`
- `2` -> run `ccswitcher current`
- `3` -> run `ccswitcher list`, then present numbered preset choices and ask user to reply with a number; convert number to preset name, then run `ccswitcher use <preset>`
- `4` -> run the `add` interactive flow in this file
- `5` -> run `ccswitcher list`, then present numbered preset choices and ask user to reply with a number; convert number to preset name, then run `ccswitcher remove <preset>`
- `6` -> run `ccswitcher reset-official`

For preset-number selection in `3` and `5`:

- If no presets exist, tell the user there are no presets and ask them to use `4` to add one first.
- Display choices in this exact style:

```
Select a preset:
1. <preset-a>
2. <preset-b>

Reply with a number.
```

- Accept only a valid listed number.
- If the user replies with text instead of a number, ask for a number again.

## Notes

- If command fails, show stderr and suggest checking field values.
- Do not modify files directly in this slash command. All writes must go through `ccswitcher`.
