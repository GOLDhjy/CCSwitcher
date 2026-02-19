use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;

use crate::errors::{AppError, Result};

pub fn ensure_directory(path: &Path) -> Result<()> {
    fs::create_dir_all(path).map_err(|err| AppError::io(path, err))
}

pub fn backup_if_exists(path: &Path) -> Result<Option<PathBuf>> {
    if !path.exists() {
        return Ok(None);
    }

    let file_name = path
        .file_name()
        .and_then(|v| v.to_str())
        .unwrap_or("settings.json");
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_millis());
    let backup_path = path.with_file_name(format!("{file_name}.bak.{timestamp}"));

    fs::copy(path, &backup_path).map_err(|err| AppError::io(&backup_path, err))?;
    Ok(Some(backup_path))
}

pub fn write_json_atomic<T: Serialize>(path: &Path, payload: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(payload).map_err(|err| AppError::json(path, err))?;
    write_bytes_atomic(path, &bytes)
}

pub fn write_text_atomic(path: &Path, content: &str) -> Result<()> {
    write_bytes_atomic(path, content.as_bytes())
}

fn write_bytes_atomic(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    ensure_directory(parent)?;

    let file_name = path.file_name().and_then(|v| v.to_str()).unwrap_or("file");
    let tmp_path = parent.join(format!(
        ".{file_name}.tmp.{}.{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |d| d.as_nanos())
    ));

    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&tmp_path)
        .map_err(|err| AppError::io(&tmp_path, err))?;
    file.write_all(bytes)
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all())
        .map_err(|err| AppError::io(&tmp_path, err))?;

    fs::rename(&tmp_path, path).map_err(|err| AppError::io(path, err))
}
