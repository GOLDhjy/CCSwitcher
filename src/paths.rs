use std::{env, path::PathBuf};

use crate::{
    errors::{AppError, Result},
    fsutil,
};

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub config_path: PathBuf,
    pub claude_home: PathBuf,
    pub settings_path: PathBuf,
    pub settings_local_path: PathBuf,
}

impl AppPaths {
    pub fn resolve() -> Result<Self> {
        let ccswitcher_home = resolve_home_path("CCSWITCHER_HOME", ".claudecode-switcher")?;
        let claude_home = resolve_home_path("CLAUDE_HOME", ".claude")?;
        let config_path = ccswitcher_home.join("config.json");
        let settings_path = claude_home.join("settings.json");
        let settings_local_path = claude_home.join("settings.local.json");

        fsutil::ensure_directory(&ccswitcher_home)?;
        fsutil::ensure_directory(&claude_home)?;

        Ok(Self {
            config_path,
            claude_home,
            settings_path,
            settings_local_path,
        })
    }
}

fn resolve_home_path(override_var: &str, default_suffix: &str) -> Result<PathBuf> {
    if let Some(path) = env::var_os(override_var) {
        return Ok(PathBuf::from(path));
    }

    let home = env::var_os("HOME").ok_or(AppError::MissingHomeDirectory)?;
    Ok(PathBuf::from(home).join(default_suffix))
}
