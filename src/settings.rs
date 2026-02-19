use std::fs;

use serde_json::{Map, Value, json};

use crate::{
    config::Preset,
    errors::{AppError, Result},
    fsutil,
    paths::AppPaths,
};

const OVERRIDE_ENV_KEYS: [&str; 9] = [
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "API_TIMEOUT_MS",
    "MCP_TOOL_TIMEOUT",
    "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
    "HTTP_PROXY",
];

pub fn apply_preset(paths: &AppPaths, preset: &Preset) -> Result<()> {
    let mut root = load_settings_root(paths)?;
    let env = ensure_env_map(paths, &mut root)?;

    set_env(
        env,
        "ANTHROPIC_DEFAULT_HAIKU_MODEL",
        Some(&preset.models.haiku_model),
    );
    set_env(
        env,
        "ANTHROPIC_DEFAULT_SONNET_MODEL",
        Some(&preset.models.sonnet_model),
    );
    set_env(
        env,
        "ANTHROPIC_DEFAULT_OPUS_MODEL",
        Some(&preset.models.opus_model),
    );
    set_env(env, "ANTHROPIC_AUTH_TOKEN", Some(&preset.auth_token));
    set_env(env, "ANTHROPIC_BASE_URL", Some(&preset.base_url));

    set_env(
        env,
        "HTTP_PROXY",
        preset
            .network
            .as_ref()
            .and_then(|network| network.http_proxy.as_deref()),
    );
    set_env(
        env,
        "API_TIMEOUT_MS",
        preset
            .timeouts
            .as_ref()
            .and_then(|timeouts| timeouts.api_timeout_ms.as_deref()),
    );
    set_env(
        env,
        "MCP_TOOL_TIMEOUT",
        preset
            .timeouts
            .as_ref()
            .and_then(|timeouts| timeouts.mcp_tool_timeout.as_deref()),
    );
    set_env(
        env,
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
        preset
            .flags
            .as_ref()
            .and_then(|flags| flags.disable_nonessential_traffic.as_deref()),
    );

    fsutil::backup_if_exists(&paths.settings_path)?;
    fsutil::write_json_atomic(&paths.settings_path, &root)
}

pub fn reset_to_official(paths: &AppPaths) -> Result<()> {
    let mut root = load_settings_root(paths)?;
    let env = ensure_env_map(paths, &mut root)?;
    for key in OVERRIDE_ENV_KEYS {
        env.remove(key);
    }

    fsutil::backup_if_exists(&paths.settings_path)?;
    fsutil::write_json_atomic(&paths.settings_path, &root)
}

fn load_settings_root(paths: &AppPaths) -> Result<Value> {
    if !paths.settings_path.exists() {
        return Ok(json!({ "env": {} }));
    }

    let raw = fs::read_to_string(&paths.settings_path)
        .map_err(|err| AppError::io(&paths.settings_path, err))?;
    let root: Value =
        serde_json::from_str(&raw).map_err(|err| AppError::json(&paths.settings_path, err))?;
    Ok(root)
}

fn ensure_env_map<'a>(paths: &AppPaths, root: &'a mut Value) -> Result<&'a mut Map<String, Value>> {
    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| AppError::invalid_json_root(&paths.settings_path))?;
    root_obj
        .entry("env".to_owned())
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| AppError::invalid_json_root(&paths.settings_path))
}

fn set_env(env: &mut Map<String, Value>, key: &str, value: Option<&str>) {
    match value {
        Some(v) => {
            env.insert(key.to_owned(), Value::String(v.to_owned()));
        }
        None => {
            env.remove(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use serde_json::Value;
    use tempfile::TempDir;

    use super::*;
    use crate::{
        config::{FlagConfig, ModelConfig, NetworkConfig, TimeoutConfig},
        provider::ProviderKind,
    };

    #[test]
    fn apply_preset_preserves_non_env_fields_and_updates_env() {
        let tmp = TempDir::new().expect("tempdir");
        let ccs_home = tmp.path().join("ccswitcher");
        let claude_home = tmp.path().join("claude");
        fs::create_dir_all(&ccs_home).expect("ccswitcher home");
        fs::create_dir_all(&claude_home).expect("claude home");
        let settings_path = claude_home.join("settings.json");

        fs::write(
            &settings_path,
            r#"{
  "enabledPlugins": {"foo": true},
  "env": {"OLD_KEY": "old-value"}
}"#,
        )
        .expect("write settings");

        let preset = Preset {
            provider: ProviderKind::Glm,
            base_url: "https://open.bigmodel.cn/api/anthropic".to_owned(),
            auth_token: "secret".to_owned(),
            models: ModelConfig {
                haiku_model: "GLM-4.7".to_owned(),
                sonnet_model: "GLM-4.7".to_owned(),
                opus_model: "GLM-4.7".to_owned(),
            },
            network: Some(NetworkConfig {
                http_proxy: Some("http://127.0.0.1:10809".to_owned()),
            }),
            timeouts: Some(TimeoutConfig {
                api_timeout_ms: Some("3000000".to_owned()),
                mcp_tool_timeout: Some("30000".to_owned()),
            }),
            flags: Some(FlagConfig {
                disable_nonessential_traffic: Some("true".to_owned()),
            }),
        };

        let paths = AppPaths {
            config_path: tmp.path().join("ccswitcher/config.json"),
            claude_home,
            settings_path: settings_path.clone(),
            settings_local_path: tmp.path().join("claude/settings.local.json"),
        };

        apply_preset(&paths, &preset).expect("apply preset");
        let parsed: Value =
            serde_json::from_str(&fs::read_to_string(settings_path).expect("read")).expect("json");

        assert_eq!(parsed["enabledPlugins"]["foo"], Value::Bool(true));
        assert_eq!(parsed["env"]["ANTHROPIC_AUTH_TOKEN"], "secret");
        assert_eq!(
            parsed["env"]["ANTHROPIC_BASE_URL"],
            "https://open.bigmodel.cn/api/anthropic"
        );
        assert_eq!(parsed["env"]["MCP_TOOL_TIMEOUT"], "30000");
    }

    #[test]
    fn reset_to_official_clears_only_override_keys() {
        let tmp = TempDir::new().expect("tempdir");
        let claude_home = tmp.path().join("claude");
        fs::create_dir_all(&claude_home).expect("claude home");
        let settings_path = claude_home.join("settings.json");

        fs::write(
            &settings_path,
            r#"{
  "enabledPlugins": {"foo": true},
  "env": {
    "ANTHROPIC_AUTH_TOKEN": "x",
    "ANTHROPIC_BASE_URL": "https://open.bigmodel.cn/api/anthropic",
    "ANTHROPIC_DEFAULT_SONNET_MODEL": "GLM-4.7",
    "EXISTING_KEY": "keep-me"
  }
}"#,
        )
        .expect("write settings");

        let paths = AppPaths {
            config_path: tmp.path().join("ccswitcher/config.json"),
            claude_home,
            settings_path: settings_path.clone(),
            settings_local_path: tmp.path().join("claude/settings.local.json"),
        };

        reset_to_official(&paths).expect("reset");
        let parsed: Value =
            serde_json::from_str(&fs::read_to_string(settings_path).expect("read")).expect("json");

        assert_eq!(parsed["enabledPlugins"]["foo"], Value::Bool(true));
        assert_eq!(
            parsed["env"]["EXISTING_KEY"],
            Value::String("keep-me".to_owned())
        );
        assert_eq!(parsed["env"]["ANTHROPIC_AUTH_TOKEN"], Value::Null);
        assert_eq!(parsed["env"]["ANTHROPIC_BASE_URL"], Value::Null);
        assert_eq!(parsed["env"]["ANTHROPIC_DEFAULT_SONNET_MODEL"], Value::Null);
    }
}
