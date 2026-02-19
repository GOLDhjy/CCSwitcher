---
argument-hint: list | current | use <preset> | add | remove <preset> | reset
description: Manage model presets and switch Claude Code provider settings
allowed-tools: Bash, Read
---

# switchmodel

Use the local `ccswitcher` binary to manage model presets.

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
2. Provider (`glm` default, or `custom`)
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

## Notes

- If command fails, show stderr and suggest checking field values.
- Do not modify files directly in this slash command. All writes must go through `ccswitcher`.
