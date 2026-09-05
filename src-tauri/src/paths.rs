use std::path::PathBuf;
use tauri::{AppHandle, Manager};

pub fn app_data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("app data dir: {e}"))
}

pub fn db_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("bella.db"))
}

pub fn library_dir(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app_data_dir(app)?.join("library"))
}

pub fn artifact_dir(app: &AppHandle, artifact_id: &str) -> Result<PathBuf, String> {
    Ok(library_dir(app)?.join(artifact_id))
}
