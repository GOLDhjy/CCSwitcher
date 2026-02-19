use clap::{Args, Parser, Subcommand};

use crate::config::{FlagConfig, ModelConfig, NetworkConfig, Preset, TimeoutConfig};
use crate::provider::ProviderKind;

#[derive(Debug, Parser)]
#[command(
    name = "ccswitcher",
    version,
    about = "Manage Claude Code model presets and switch provider config"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List all presets.
    List,
    /// Show active preset details.
    Current,
    /// Switch to a preset and apply it to ~/.claude/settings.json.
    Use {
        /// Preset name to activate.
        preset: String,
    },
    /// Add a preset.
    Add(AddArgs),
    /// Remove a preset.
    Remove {
        /// Preset name to remove.
        preset: String,
    },
    /// Clear model/provider env overrides and return to Claude official defaults.
    #[command(name = "reset-official", visible_alias = "reset")]
    ResetOfficial,
    /// Install /switchmodel slash command template into ~/.claude/commands.
    Install,
}

#[derive(Debug, Args)]
pub struct AddArgs {
    /// Unique preset name.
    #[arg(long)]
    pub name: String,
    /// Provider type.
    #[arg(long, value_enum, default_value_t = ProviderKind::Custom)]
    pub provider: ProviderKind,
    /// Anthropic-compatible base URL.
    #[arg(long)]
    pub base_url: String,
    /// Provider auth token.
    #[arg(long)]
    pub auth_token: String,
    /// Default model for ANTHROPIC_DEFAULT_HAIKU_MODEL.
    #[arg(long)]
    pub haiku: String,
    /// Default model for ANTHROPIC_DEFAULT_SONNET_MODEL.
    #[arg(long)]
    pub sonnet: String,
    /// Default model for ANTHROPIC_DEFAULT_OPUS_MODEL.
    #[arg(long)]
    pub opus: String,
    /// Optional HTTP proxy URL.
    #[arg(long)]
    pub http_proxy: Option<String>,
    /// Optional API timeout in milliseconds.
    #[arg(long)]
    pub api_timeout_ms: Option<String>,
    /// Optional MCP tool timeout in milliseconds.
    #[arg(long)]
    pub mcp_tool_timeout: Option<String>,
    /// Optional flag for CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC.
    #[arg(long)]
    pub disable_nonessential_traffic: Option<bool>,
}

impl AddArgs {
    pub fn into_name_and_preset(self) -> (String, Preset) {
        let name = self.name.trim().to_owned();
        let disable_nonessential_traffic = self
            .disable_nonessential_traffic
            .map(|value| value.to_string());

        let preset = Preset {
            provider: self.provider,
            base_url: self.base_url.trim().to_owned(),
            auth_token: self.auth_token.trim().to_owned(),
            models: ModelConfig {
                haiku_model: self.haiku.trim().to_owned(),
                sonnet_model: self.sonnet.trim().to_owned(),
                opus_model: self.opus.trim().to_owned(),
            },
            network: self.http_proxy.map(|http_proxy| NetworkConfig {
                http_proxy: Some(http_proxy.trim().to_owned()),
            }),
            timeouts: Some(TimeoutConfig {
                api_timeout_ms: self.api_timeout_ms.map(|v| v.trim().to_owned()),
                mcp_tool_timeout: self.mcp_tool_timeout.map(|v| v.trim().to_owned()),
            })
            .filter(|t| t.api_timeout_ms.is_some() || t.mcp_tool_timeout.is_some()),
            flags: disable_nonessential_traffic.map(|value| FlagConfig {
                disable_nonessential_traffic: Some(value),
            }),
        };

        (name, preset)
    }
}
