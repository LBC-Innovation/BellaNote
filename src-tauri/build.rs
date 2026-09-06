fn ensure_sidecar_placeholders() {
    let triples = ["aarch64-apple-darwin", "x86_64-apple-darwin"];
    let dir = std::path::Path::new("binaries");
    let _ = std::fs::create_dir_all(dir);
    for triple in triples {
        let path = dir.join(format!("transcribe-worker-{triple}"));
        if path.exists() {
            continue;
        }
        let _ = std::fs::write(&path, b"#!/bin/sh\necho placeholder-sidecar\n");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Ok(meta) = std::fs::metadata(&path) {
                let mut perms = meta.permissions();
                perms.set_mode(0o755);
                let _ = std::fs::set_permissions(&path, perms);
            }
        }
    }
}

fn main() {
    ensure_sidecar_placeholders();
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_library",
            "create_organization",
            "rename_organization",
            "delete_organization",
            "create_topic",
            "rename_topic",
            "delete_topic",
            "create_meeting_group",
            "rename_meeting_group",
            "delete_meeting_group",
            "list_artifacts",
            "get_artifact",
            "import_audio",
            "import_transcript",
            "rename_artifact",
            "delete_artifact",
            "retry_artifact",
            "get_artifact_audio_path",
            "get_artifact_audio_peaks",
            "list_artifact_comments",
            "create_artifact_comment",
            "update_artifact_comment",
            "delete_artifact_comment",
            "set_openai_api_key",
            "clear_openai_api_key",
            "openai_api_key_configured",
            "chat_scope_preview",
            "get_chat_thread",
            "new_chat_thread",
            "ask_chat",
        ]),
    ))
    .expect("tauri build failed");
}
