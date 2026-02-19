mod cli;
mod config;
mod errors;
mod fsutil;
mod paths;
mod provider;
mod settings;

use std::io::Write;

use clap::Parser;
use cli::{Cli, Commands};
use config::{Preset, SwitcherConfig};
pub use errors::{AppError, Result};
use provider::ProviderKind;
use serde_json::{Value, json};

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    run_with_cli(cli, &mut std::io::stdout())
}

fn run_with_cli(cli: Cli, out: &mut dyn Write) -> Result<()> {
    let paths = paths::AppPaths::resolve()?;

    match cli.command {
        None => {
            let mut cfg = config::load(&paths)?;
            run_interactive_menu(&mut cfg, &paths, out)
        }
        Some(Commands::Install) => install_slash_command(&paths, out),
        Some(Commands::List) => {
            let cfg = config::load(&paths)?;
            list_presets(&cfg, out)
        }
        Some(Commands::Current) => {
            let cfg = config::load(&paths)?;
            show_current(&cfg, out)
        }
        Some(other) => {
            let mut cfg = config::load(&paths)?;
            match other {
                Commands::Use { preset } => use_preset(&mut cfg, &preset, &paths, out),
                Commands::Add(args) => add_preset(&mut cfg, args, &paths, out),
                Commands::Remove { preset } => remove_preset(&mut cfg, &preset, &paths, out),
                Commands::ResetOfficial => reset_official(&mut cfg, &paths, out),
                Commands::List | Commands::Current | Commands::Install => {
                    unreachable!("handled above")
                }
            }
        }
    }
}

fn list_presets(cfg: &SwitcherConfig, out: &mut dyn Write) -> Result<()> {
    if cfg.presets.is_empty() {
        writeln!(out, "No presets configured.").map_err(AppError::output)?;
        return Ok(());
    }

    writeln!(out, "Available presets:").map_err(AppError::output)?;
    for (name, preset) in &cfg.presets {
        let marker = if cfg.active_preset.as_deref() == Some(name.as_str()) {
            "*"
        } else {
            " "
        };
        writeln!(
            out,
            "{marker} {name} ({provider})",
            provider = preset.provider
        )
        .map_err(AppError::output)?;
    }
    Ok(())
}

fn show_current(cfg: &SwitcherConfig, out: &mut dyn Write) -> Result<()> {
    let Some(active_name) = cfg.active_preset.as_deref() else {
        writeln!(out, "No active preset.").map_err(AppError::output)?;
        return Ok(());
    };

    let preset = cfg
        .presets
        .get(active_name)
        .ok_or_else(|| AppError::PresetNotFound(active_name.to_owned()))?;

    write_preset_details(out, active_name, preset)
}

fn use_preset(
    cfg: &mut SwitcherConfig,
    preset_name: &str,
    paths: &paths::AppPaths,
    out: &mut dyn Write,
) -> Result<()> {
    let preset = cfg
        .presets
        .get(preset_name)
        .ok_or_else(|| AppError::PresetNotFound(preset_name.to_owned()))?;
    preset.validate_ready(preset_name)?;

    settings::apply_preset(paths, preset)?;
    cfg.active_preset = Some(preset_name.to_owned());
    config::save(paths, cfg)?;

    writeln!(
        out,
        "Switched to preset '{preset_name}'. New requests will use this model configuration."
    )
    .map_err(AppError::output)?;
    Ok(())
}

fn add_preset(
    cfg: &mut SwitcherConfig,
    args: cli::AddArgs,
    paths: &paths::AppPaths,
    out: &mut dyn Write,
) -> Result<()> {
    let (name, preset) = args.into_name_and_preset();
    if cfg.presets.contains_key(&name) {
        return Err(AppError::PresetAlreadyExists(name));
    }
    preset.validate_ready(&name)?;

    cfg.presets.insert(name.clone(), preset);
    config::save(paths, cfg)?;

    writeln!(out, "Saved preset '{name}'.").map_err(AppError::output)?;
    writeln!(out, "Warning: this preset stores auth_token in plain text.")
        .map_err(AppError::output)?;
    Ok(())
}

fn remove_preset(
    cfg: &mut SwitcherConfig,
    preset_name: &str,
    paths: &paths::AppPaths,
    out: &mut dyn Write,
) -> Result<()> {
    if cfg.active_preset.as_deref() == Some(preset_name) {
        return Err(AppError::CannotRemoveActivePreset(preset_name.to_owned()));
    }

    if cfg.presets.remove(preset_name).is_none() {
        return Err(AppError::PresetNotFound(preset_name.to_owned()));
    }

    config::save(paths, cfg)?;
    writeln!(out, "Removed preset '{preset_name}'.").map_err(AppError::output)?;
    Ok(())
}

fn write_preset_details(out: &mut dyn Write, name: &str, preset: &Preset) -> Result<()> {
    writeln!(out, "Active preset: {name}").map_err(AppError::output)?;
    writeln!(out, "Provider: {}", preset.provider).map_err(AppError::output)?;
    writeln!(out, "Base URL: {}", preset.base_url).map_err(AppError::output)?;
    writeln!(out, "Haiku model: {}", preset.models.haiku_model).map_err(AppError::output)?;
    writeln!(out, "Sonnet model: {}", preset.models.sonnet_model).map_err(AppError::output)?;
    writeln!(out, "Opus model: {}", preset.models.opus_model).map_err(AppError::output)?;
    Ok(())
}

fn reset_official(
    cfg: &mut SwitcherConfig,
    paths: &paths::AppPaths,
    out: &mut dyn Write,
) -> Result<()> {
    settings::reset_to_official(paths)?;
    cfg.active_preset = None;
    config::save(paths, cfg)?;
    writeln!(
        out,
        "Reset complete. Official Claude model/provider defaults will be used for new requests."
    )
    .map_err(AppError::output)?;
    Ok(())
}

fn install_slash_command(paths: &paths::AppPaths, out: &mut dyn Write) -> Result<()> {
    let command_dir = paths.claude_home.join("commands");
    fsutil::ensure_directory(&command_dir)?;
    let command_path = command_dir.join("switchmodel.md");
    let template = include_str!("../templates/switchmodel.md");
    fsutil::write_text_atomic(&command_path, template)?;
    ensure_bash_permission_rule(paths)?;
    writeln!(out, "Installed slash command: {}", command_path.display())
        .map_err(AppError::output)?;
    writeln!(
        out,
        "Ensured local permission rule: Bash(ccswitcher:*) at {}",
        paths.settings_local_path.display()
    )
    .map_err(AppError::output)?;
    Ok(())
}

fn ensure_bash_permission_rule(paths: &paths::AppPaths) -> Result<()> {
    const RULE: &str = "Bash(ccswitcher:*)";
    let path = &paths.settings_local_path;

    let mut root = if path.exists() {
        let raw = std::fs::read_to_string(path).map_err(|err| AppError::io(path, err))?;
        serde_json::from_str::<Value>(&raw).map_err(|err| AppError::json(path, err))?
    } else {
        json!({})
    };

    let root_obj = root
        .as_object_mut()
        .ok_or_else(|| AppError::invalid_json_root(path))?;
    let permissions = root_obj
        .entry("permissions".to_owned())
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| AppError::invalid_json_root(path))?;

    let allow = permissions
        .entry("allow".to_owned())
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| AppError::invalid_json_root(path))?;

    if !allow.iter().any(|v| v.as_str() == Some(RULE)) {
        allow.push(Value::String(RULE.to_owned()));
    }
    permissions
        .entry("deny".to_owned())
        .or_insert_with(|| json!([]));
    permissions
        .entry("ask".to_owned())
        .or_insert_with(|| json!([]));

    fsutil::write_json_atomic(path, &root)
}

fn run_interactive_menu(
    cfg: &mut SwitcherConfig,
    paths: &paths::AppPaths,
    out: &mut dyn Write,
) -> Result<()> {
    writeln!(out, "CCSwitcher interactive mode.").map_err(AppError::output)?;
    writeln!(out, "Choose an action by number, or type exit to quit.").map_err(AppError::output)?;

    loop {
        writeln!(out).map_err(AppError::output)?;
        writeln!(out, "1. list presets").map_err(AppError::output)?;
        writeln!(out, "2. current preset").map_err(AppError::output)?;
        writeln!(out, "3. use preset").map_err(AppError::output)?;
        writeln!(out, "4. add preset").map_err(AppError::output)?;
        writeln!(out, "5. remove preset").map_err(AppError::output)?;
        writeln!(out, "6. reset official").map_err(AppError::output)?;
        writeln!(out, "7. install /switchmodel command").map_err(AppError::output)?;
        writeln!(out, "0. exit").map_err(AppError::output)?;

        let action = prompt_line(out, "Select [0-7]").map(|v| v.to_ascii_lowercase())?;
        match action.as_str() {
            "1" | "list" => list_presets(cfg, out)?,
            "2" | "current" => show_current(cfg, out)?,
            "3" | "use" => {
                if let Some(preset) = prompt_select_preset(
                    cfg,
                    out,
                    "Select a preset to use",
                    "No presets configured. Use action 4 to add one first.",
                )? {
                    use_preset(cfg, &preset, paths, out)?;
                }
            }
            "4" | "add" => {
                let maybe_args = prompt_add_args(out)?;
                if let Some(args) = maybe_args {
                    add_preset(cfg, args, paths, out)?;
                } else {
                    writeln!(out, "Add preset cancelled.").map_err(AppError::output)?;
                }
            }
            "5" | "remove" => {
                if let Some(preset) = prompt_select_preset(
                    cfg,
                    out,
                    "Select a preset to remove",
                    "No presets configured. Nothing to remove.",
                )? {
                    remove_preset(cfg, &preset, paths, out)?;
                }
            }
            "6" | "reset" | "reset-official" => {
                let confirm = prompt_line(
                    out,
                    "Type RESET to confirm official reset (or Enter to cancel)",
                )?;
                if confirm == "RESET" {
                    reset_official(cfg, paths, out)?;
                } else {
                    writeln!(out, "Reset cancelled.").map_err(AppError::output)?;
                }
            }
            "7" | "install" => install_slash_command(paths, out)?,
            "0" | "exit" | "quit" => {
                writeln!(out, "Bye.").map_err(AppError::output)?;
                break;
            }
            _ => writeln!(out, "Invalid selection.").map_err(AppError::output)?,
        }
    }

    Ok(())
}

fn prompt_add_args(out: &mut dyn Write) -> Result<Option<cli::AddArgs>> {
    writeln!(out, "Add preset wizard (terminal interactive mode).").map_err(AppError::output)?;
    writeln!(out, "For optional fields, press Enter to use default.").map_err(AppError::output)?;

    let name = prompt_required(out, "Preset name")?;
    let provider_input = prompt_line(out, "Provider tag (optional, default: custom)")?;
    let provider = if provider_input.eq_ignore_ascii_case("glm") {
        ProviderKind::Glm
    } else {
        ProviderKind::Custom
    };

    let base_url = prompt_required(out, "Base URL")?;

    let auth_token = prompt_required(out, "Auth token")?;
    let default_model = prompt_required(
        out,
        "Default model (applies to haiku/sonnet/opus unless overridden)",
    )?;
    let separate_models =
        prompt_yes_no(out, "Set separate haiku/sonnet/opus models? [y/N]", false)?;
    let (haiku, sonnet, opus) = if separate_models {
        (
            prompt_with_default(out, "Haiku model", &default_model)?,
            prompt_with_default(out, "Sonnet model", &default_model)?,
            prompt_with_default(out, "Opus model", &default_model)?,
        )
    } else {
        (default_model.clone(), default_model.clone(), default_model)
    };

    let http_proxy = prompt_optional(out, "HTTP proxy", "not set")?;
    let api_timeout_ms = prompt_optional(out, "API timeout (ms)", "not set")?;
    let mcp_tool_timeout = prompt_optional(out, "MCP tool timeout (ms)", "not set")?;
    let disable_nonessential_traffic =
        prompt_optional_bool(out, "Disable nonessential traffic? (true/false)", "not set")?;

    let args = cli::AddArgs {
        name,
        provider,
        base_url,
        auth_token,
        haiku,
        sonnet,
        opus,
        http_proxy,
        api_timeout_ms,
        mcp_tool_timeout,
        disable_nonessential_traffic,
    };

    print_add_summary(out, &args)?;
    let confirm = prompt_yes_no(out, "Save this preset? [Y/n]", true)?;
    if confirm { Ok(Some(args)) } else { Ok(None) }
}

fn prompt_line(out: &mut dyn Write, prompt: &str) -> Result<String> {
    write!(out, "{prompt}: ").map_err(AppError::output)?;
    out.flush().map_err(AppError::output)?;
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|err| AppError::io("stdin", err))?;
    Ok(line.trim().to_owned())
}

fn prompt_required(out: &mut dyn Write, prompt: &str) -> Result<String> {
    let labeled = format!("{prompt} (required)");
    loop {
        let value = prompt_line(out, &labeled)?;
        if !value.is_empty() {
            return Ok(value);
        }
        writeln!(out, "This field is required.").map_err(AppError::output)?;
    }
}

fn prompt_select_preset(
    cfg: &SwitcherConfig,
    out: &mut dyn Write,
    title: &str,
    empty_message: &str,
) -> Result<Option<String>> {
    if cfg.presets.is_empty() {
        writeln!(out, "{empty_message}").map_err(AppError::output)?;
        return Ok(None);
    }

    let names: Vec<String> = cfg.presets.keys().cloned().collect();
    writeln!(out, "{title}:").map_err(AppError::output)?;
    for (idx, name) in names.iter().enumerate() {
        writeln!(out, "{}. {}", idx + 1, name).map_err(AppError::output)?;
    }

    loop {
        let answer = prompt_line(
            out,
            &format!("Preset number [1-{}] (Enter to cancel)", names.len()),
        )?;
        if answer.is_empty() {
            writeln!(out, "Selection cancelled.").map_err(AppError::output)?;
            return Ok(None);
        }

        match answer.parse::<usize>() {
            Ok(index) if (1..=names.len()).contains(&index) => {
                return Ok(Some(names[index - 1].clone()));
            }
            _ => {
                writeln!(out, "Invalid selection. Enter one of the listed numbers.")
                    .map_err(AppError::output)?;
            }
        }
    }
}

fn prompt_with_default(out: &mut dyn Write, prompt: &str, default_value: &str) -> Result<String> {
    let value = prompt_line(out, &format!("{prompt} (default: {default_value})"))?;
    if value.is_empty() {
        Ok(default_value.to_owned())
    } else {
        Ok(value)
    }
}

fn prompt_optional(
    out: &mut dyn Write,
    prompt: &str,
    default_value: &str,
) -> Result<Option<String>> {
    let value = prompt_line(
        out,
        &format!("{prompt} (optional, default: {default_value}, Enter to skip)"),
    )?;
    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

fn prompt_optional_bool(
    out: &mut dyn Write,
    prompt: &str,
    default_value: &str,
) -> Result<Option<bool>> {
    let labeled = format!("{prompt} (optional, default: {default_value}, Enter to skip)");
    loop {
        let value = prompt_line(out, &labeled)?;
        if value.is_empty() {
            return Ok(None);
        }
        match value.to_ascii_lowercase().as_str() {
            "true" | "t" | "1" | "yes" | "y" => return Ok(Some(true)),
            "false" | "f" | "0" | "no" | "n" => return Ok(Some(false)),
            _ => writeln!(out, "Please enter true or false.").map_err(AppError::output)?,
        }
    }
}

fn print_add_summary(out: &mut dyn Write, args: &cli::AddArgs) -> Result<()> {
    writeln!(out).map_err(AppError::output)?;
    writeln!(out, "Preset summary:").map_err(AppError::output)?;
    writeln!(out, "- name: {}", args.name).map_err(AppError::output)?;
    writeln!(out, "- provider: {}", args.provider).map_err(AppError::output)?;
    writeln!(out, "- base_url: {}", args.base_url).map_err(AppError::output)?;
    writeln!(out, "- auth_token: ****").map_err(AppError::output)?;
    writeln!(out, "- haiku_model: {}", args.haiku).map_err(AppError::output)?;
    writeln!(out, "- sonnet_model: {}", args.sonnet).map_err(AppError::output)?;
    writeln!(out, "- opus_model: {}", args.opus).map_err(AppError::output)?;
    writeln!(
        out,
        "- http_proxy: {}",
        args.http_proxy.as_deref().unwrap_or("not set")
    )
    .map_err(AppError::output)?;
    writeln!(
        out,
        "- api_timeout_ms: {}",
        args.api_timeout_ms.as_deref().unwrap_or("not set")
    )
    .map_err(AppError::output)?;
    writeln!(
        out,
        "- mcp_tool_timeout: {}",
        args.mcp_tool_timeout.as_deref().unwrap_or("not set")
    )
    .map_err(AppError::output)?;
    let traffic = match args.disable_nonessential_traffic {
        Some(true) => "true",
        Some(false) => "false",
        None => "not set",
    };
    writeln!(out, "- disable_nonessential_traffic: {traffic}").map_err(AppError::output)?;
    Ok(())
}

fn prompt_yes_no(out: &mut dyn Write, prompt: &str, default_value: bool) -> Result<bool> {
    loop {
        let value = prompt_line(out, prompt)?;
        if value.is_empty() {
            return Ok(default_value);
        }
        match value.to_ascii_lowercase().as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => writeln!(out, "Please answer y or n.").map_err(AppError::output)?,
        }
    }
}
