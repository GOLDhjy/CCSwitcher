# CCSwitcher

[English README](./README.md)

`ccswitcher` 是一个用于管理 Claude Code 模型预设的 Rust CLI 工具。

它提供全局斜杠命令 `/switchmodel`，包含以下子命令：

- `list`
- `current`
- `use <preset>`
- `add`
- `remove <preset>`
- `reset`（`reset-official` 的别名）

## 它会修改什么

- 预设配置保存在 `~/.claudecode-switcher/config.json`
- 当前预设会写入 `~/.claude/settings.json` 的 `env` 字段

会更新以下环境变量：

- `ANTHROPIC_DEFAULT_HAIKU_MODEL`
- `ANTHROPIC_DEFAULT_SONNET_MODEL`
- `ANTHROPIC_DEFAULT_OPUS_MODEL`
- `ANTHROPIC_AUTH_TOKEN`
- `ANTHROPIC_BASE_URL`
- `API_TIMEOUT_MS`（可选）
- `MCP_TOOL_TIMEOUT`（可选）
- `CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC`（可选）
- `HTTP_PROXY`（可选）

`settings.json` 采用原子写入，并自动创建带时间戳的备份。

## Rust 工具链

本项目通过 `rust-toolchain.toml` 跟踪最新稳定版 Rust：

- `stable`

## 构建

```bash
cargo build --release
```

如果你的项目目录所在文件系统不支持 Rust lock file，可使用：

```bash
CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/ccswitcher-target cargo build --release
```

## CLI 用法

```bash
ccswitcher list
ccswitcher current
ccswitcher use glm
ccswitcher reset-official

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

## 安装（推荐，无需 Rust）

```bash
curl -fsSL https://raw.githubusercontent.com/GOLDhjy/CCSwitcher/main/scripts/install.sh | bash
```

该命令会下载你平台对应的最新预编译二进制，然后安装：

- `~/.local/bin/ccswitcher`
- `~/.claude/commands/switchmodel.md`（`/switchmodel` 命令模板）

安装指定版本：

```bash
curl -fsSL https://raw.githubusercontent.com/GOLDhjy/CCSwitcher/main/scripts/install.sh | bash -s -- --version v0.1.0
```

如果你的平台没有可用二进制，可切换为源码模式：

```bash
curl -fsSL https://raw.githubusercontent.com/GOLDhjy/CCSwitcher/main/scripts/install.sh | bash -s -- --source
```

查看发布包：[`GitHub Releases`](https://github.com/GOLDhjy/CCSwitcher/releases)

## 安装（本地源码仓库）

```bash
bash scripts/install.sh --source
```

安装完成后，可在 Claude Code 中使用：

- `/switchmodel list`
- `/switchmodel current`
- `/switchmodel use glm-work`
- `/switchmodel add`（在对话中交互式提问）
- `/switchmodel remove glm-work`
- `/switchmodel reset`

## 安全说明

当前版本会以明文保存 Auth Token。
如有泄露风险，请及时轮换 Token，并确保相关文件权限受控。
