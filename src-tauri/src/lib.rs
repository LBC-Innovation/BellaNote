mod artifacts;
mod audio_peaks;
mod capture;
mod chat;
mod commands;
mod db;
mod error;
mod import_transcript;
mod llm;
mod paths;
mod recording;
mod state;
mod transcribe;

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
            let state = Arc::new(AppState {
                db,
                transcriber: std::sync::Mutex::new(None),
                recording: crate::recording::RecordingRuntime::new(),
            });
            let _ = state.db.fail_interrupted_imports();
            let _ = state.db.fail_empty_ready_audio();
            app.manage(state);
            if let Some(window) = app.get_webview_window("main") {
                let focused = window.clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Enter { .. }) = event {
                        let _ = focused.set_focus();
                    }
                });
            }
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
            commands::list_artifacts,
            commands::get_artifact,
            commands::import_audio,
            commands::import_transcript,
            commands::rename_artifact,
            commands::delete_artifact,
            commands::retry_artifact,
            commands::get_artifact_audio_path,
            commands::get_artifact_audio_peaks,
            commands::list_artifact_comments,
            commands::create_artifact_comment,
            commands::update_artifact_comment,
            commands::delete_artifact_comment,
            commands::set_openai_api_key,
            commands::clear_openai_api_key,
            commands::openai_api_key_configured,
            commands::chat_scope_preview,
            commands::get_chat_thread,
            commands::new_chat_thread,
            commands::ask_chat,
            commands::start_recording,
            commands::stop_recording,
            commands::recording_status,
            commands::recording_capabilities,
            commands::get_rms,
            commands::get_spectrum,
        ])
        .run(tauri::generate_context!())
        .expect("error while running BellaNote");
}
