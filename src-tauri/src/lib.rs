mod commands;
mod db;
mod error;
mod paths;
mod state;

use std::sync::Arc;
use tauri::Manager;

use crate::state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();
            let db_path = paths::db_path(handle).map_err(std::io::Error::other)?;
            let db = db::Db::open(&db_path).map_err(|e| std::io::Error::other(e.to_string()))?;
            if let Ok(dir) = paths::library_dir(handle) {
                std::fs::create_dir_all(dir).ok();
            }
            app.manage(Arc::new(AppState { db }));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_library,
            commands::create_organization,
            commands::rename_organization,
            commands::delete_organization,
            commands::create_topic,
            commands::rename_topic,
            commands::delete_topic,
            commands::create_meeting_group,
            commands::rename_meeting_group,
            commands::delete_meeting_group,
        ])
        .run(tauri::generate_context!())
        .expect("error while running BellaNote");
}
