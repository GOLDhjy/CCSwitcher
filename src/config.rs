use std::{collections::BTreeMap, fs, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    errors::{AppError, Result},
    fsutil,
    paths::AppPaths,
    provider::ProviderKind,
};

const CURRENT_CONFIG_VERSION: u32 = 1;
const DEFAULT_GLM_PRESET_NAME: &str = "glm";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitcherConfig {
    pub version: u32,
    pub active_preset: Option<String>,
    pub presets: BTreeMap<String, Preset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub provider: ProviderKind,
    pub base_url: String,
    pub auth_token: String,
    pub models: ModelConfig,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub network: Option<NetworkConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeouts: Option<TimeoutConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flags: Option<FlagConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub haiku_model: String,
    pub sonnet_model: String,
    pub opus_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http_proxy: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub api_timeout_ms: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mcp_tool_timeout: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlagConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disable_nonessential_traffic: Option<String>,
}

impl Default for SwitcherConfig {
    fn default() -> Self {
        let mut presets = BTreeMap::new();
        presets.insert(DEFAULT_GLM_PRESET_NAME.to_owned(), default_glm_preset());
        Self {
            version: CURRENT_CONFIG_VERSION,
            active_preset: None,
            presets,
        }
    }
}

pub fn load_or_init(paths: &AppPaths) -> Result<SwitcherConfig> {
    if !paths.config_path.exists() {
        let cfg = SwitcherConfig::default();
        save(paths, &cfg)?;
        return Ok(cfg);
    }

    load_existing_config(&paths.config_path).map(ensure_builtin_glm_preset)
}

pub fn save(paths: &AppPaths, config: &SwitcherConfig) -> Result<()> {
    fsutil::write_json_atomic(&paths.config_path, config)
}

impl Preset {
    pub fn validate_ready(&self, preset_name: &str) -> Result<()> {
        if self.base_url.trim().is_empty() {
            return Err(AppError::PresetIncomplete {
                preset: preset_name.to_owned(),
                field: "base_url",
            });
        }
        if self.auth_token.trim().is_empty() {
            return Err(AppError::PresetIncomplete {
                preset: preset_name.to_owned(),
                field: "auth_token",
            });
        }
        if self.models.haiku_model.trim().is_empty() {
            return Err(AppError::PresetIncomplete {
                preset: preset_name.to_owned(),
                field: "models.haiku_model",
            });
        }
        if self.models.sonnet_model.trim().is_empty() {
            return Err(AppError::PresetIncomplete {
                preset: preset_name.to_owned(),
                field: "models.sonnet_model",
            });
        }
        if self.models.opus_model.trim().is_empty() {
            return Err(AppError::PresetIncomplete {
                preset: preset_name.to_owned(),
                field: "models.opus_model",
            });
        }
        Ok(())
    }
}

fn load_existing_config(path: &Path) -> Result<SwitcherConfig> {
    let raw = fs::read_to_string(path).map_err(|err| AppError::io(path, err))?;
    let cfg: SwitcherConfig =
        serde_json::from_str(&raw).map_err(|err| AppError::json(path, err))?;
    if cfg.version != CURRENT_CONFIG_VERSION {
        return Err(AppError::UnsupportedConfigVersion(cfg.version));
    }
    Ok(cfg)
}

fn ensure_builtin_glm_preset(mut cfg: SwitcherConfig) -> SwitcherConfig {
    cfg.presets
        .entry(DEFAULT_GLM_PRESET_NAME.to_owned())
        .or_insert_with(default_glm_preset);
    cfg
}

fn default_glm_preset() -> Preset {
    Preset {
        provider: ProviderKind::Glm,
        base_url: "https://open.bigmodel.cn/api/anthropic".to_owned(),
        auth_token: String::new(),
        models: ModelConfig {
            haiku_model: "GLM-4.7".to_owned(),
            sonnet_model: "GLM-4.7".to_owned(),
            opus_model: "GLM-4.7".to_owned(),
        },
        network: None,
        timeouts: None,
        flags: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_contains_glm_preset() {
        let cfg = SwitcherConfig::default();
        let glm = cfg
            .presets
            .get(DEFAULT_GLM_PRESET_NAME)
            .expect("missing glm");
        assert_eq!(glm.base_url, "https://open.bigmodel.cn/api/anthropic");
        assert_eq!(glm.models.sonnet_model, "GLM-4.7");
    }

    #[test]
    fn validate_ready_rejects_missing_required_fields() {
        let preset = Preset {
            provider: ProviderKind::Glm,
            base_url: String::new(),
            auth_token: "token".to_owned(),
            models: ModelConfig {
                haiku_model: "GLM-4.7".to_owned(),
                sonnet_model: "GLM-4.7".to_owned(),
                opus_model: "GLM-4.7".to_owned(),
            },
            network: None,
            timeouts: None,
            flags: None,
        };

        let err = preset.validate_ready("glm").expect_err("expected error");
        assert!(matches!(
            err,
            AppError::PresetIncomplete {
                preset,
                field: "base_url"
            } if preset == "glm"
        ));
    }
}
