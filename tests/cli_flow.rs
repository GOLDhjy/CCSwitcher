use std::fs;

use assert_cmd::Command;
use predicates::str::contains;
use serde_json::Value;
use tempfile::TempDir;

fn command_with_env(switcher_home: &std::path::Path, claude_home: &std::path::Path) -> Command {
    let mut cmd = assert_cmd::cargo::cargo_bin_cmd!("ccswitcher");
    cmd.env("CCSWITCHER_HOME", switcher_home)
        .env("CLAUDE_HOME", claude_home);
    cmd
}

#[test]
fn add_use_current_flow_updates_settings_and_preserves_other_fields() {
    let tmp = TempDir::new().expect("tempdir");
    let switcher_home = tmp.path().join("switcher-home");
    let claude_home = tmp.path().join("claude-home");
    fs::create_dir_all(&switcher_home).expect("switcher home");
    fs::create_dir_all(&claude_home).expect("claude home");

    let settings_path = claude_home.join("settings.json");
    fs::write(
        &settings_path,
        r#"{
  "enabledPlugins": {"existing-plugin": true},
  "alwaysThinkingEnabled": true,
  "env": {
    "EXISTING_KEY": "existing-value"
  }
}"#,
    )
    .expect("seed settings");

    command_with_env(&switcher_home, &claude_home)
        .args([
            "add",
            "--name",
            "team-glm",
            "--provider",
            "glm",
            "--base-url",
            "https://open.bigmodel.cn/api/anthropic",
            "--auth-token",
            "token-123",
            "--haiku",
            "GLM-4.7",
            "--sonnet",
            "GLM-4.7",
            "--opus",
            "GLM-4.7",
            "--api-timeout-ms",
            "3000000",
            "--mcp-tool-timeout",
            "30000",
        ])
        .assert()
        .success()
        .stdout(contains("Saved preset 'team-glm'."));

    command_with_env(&switcher_home, &claude_home)
        .args(["list"])
        .assert()
        .success()
        .stdout(contains("team-glm"));

    command_with_env(&switcher_home, &claude_home)
        .args(["use", "team-glm"])
        .assert()
        .success()
        .stdout(contains("Switched to preset 'team-glm'"));

    command_with_env(&switcher_home, &claude_home)
        .args(["current"])
        .assert()
        .success()
        .stdout(contains("Active preset: team-glm"))
        .stdout(contains("Provider: glm"));

    let settings: Value =
        serde_json::from_str(&fs::read_to_string(&settings_path).expect("read settings"))
            .expect("valid settings json");

    assert_eq!(
        settings["enabledPlugins"]["existing-plugin"],
        Value::Bool(true)
    );
    assert_eq!(settings["alwaysThinkingEnabled"], Value::Bool(true));
    assert_eq!(
        settings["env"]["ANTHROPIC_BASE_URL"],
        Value::String("https://open.bigmodel.cn/api/anthropic".to_owned())
    );
    assert_eq!(
        settings["env"]["ANTHROPIC_AUTH_TOKEN"],
        Value::String("token-123".to_owned())
    );
    assert_eq!(
        settings["env"]["ANTHROPIC_DEFAULT_SONNET_MODEL"],
        Value::String("GLM-4.7".to_owned())
    );
    assert_eq!(
        settings["env"]["MCP_TOOL_TIMEOUT"],
        Value::String("30000".to_owned())
    );

    let env_map = settings["env"].as_object().expect("env object");
    assert!(!env_map.contains_key("HTTP_PROXY"));

    let backup_count = fs::read_dir(&claude_home)
        .expect("read dir")
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("settings.json.bak.")
        })
        .count();
    assert!(backup_count >= 1, "expected at least one settings backup");
}

#[test]
fn remove_active_preset_fails() {
    let tmp = TempDir::new().expect("tempdir");
    let switcher_home = tmp.path().join("switcher-home");
    let claude_home = tmp.path().join("claude-home");
    fs::create_dir_all(&switcher_home).expect("switcher home");
    fs::create_dir_all(&claude_home).expect("claude home");

    command_with_env(&switcher_home, &claude_home)
        .args([
            "add",
            "--name",
            "team-glm",
            "--provider",
            "glm",
            "--base-url",
            "https://open.bigmodel.cn/api/anthropic",
            "--auth-token",
            "token-123",
            "--haiku",
            "GLM-4.7",
            "--sonnet",
            "GLM-4.7",
            "--opus",
            "GLM-4.7",
        ])
        .assert()
        .success();

    command_with_env(&switcher_home, &claude_home)
        .args(["use", "team-glm"])
        .assert()
        .success();

    command_with_env(&switcher_home, &claude_home)
        .args(["remove", "team-glm"])
        .assert()
        .failure()
        .stderr(contains("Cannot remove active preset 'team-glm'"));
}
