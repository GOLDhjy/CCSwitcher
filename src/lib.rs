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

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    run_with_cli(cli, &mut std::io::stdout())
}

fn run_with_cli(cli: Cli, out: &mut dyn Write) -> Result<()> {
    let paths = paths::AppPaths::resolve()?;

    match cli.command {
        Commands::Install => install_slash_command(&paths, out),
        other => {
            let mut cfg = config::load_or_init(&paths)?;
            match other {
                Commands::List => list_presets(&cfg, out),
                Commands::Current => show_current(&cfg, out),
                Commands::Use { preset } => use_preset(&mut cfg, &preset, &paths, out),
                Commands::Add(args) => add_preset(&mut cfg, args, &paths, out),
                Commands::Remove { preset } => remove_preset(&mut cfg, &preset, &paths, out),
                Commands::ResetOfficial => reset_official(&mut cfg, &paths, out),
                Commands::Install => unreachable!("handled above"),
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
    writeln!(out, "Installed slash command: {}", command_path.display())
        .map_err(AppError::output)?;
    Ok(())
}
