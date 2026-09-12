fn ensure_sidecar_placeholders() {
    let triples = [
        ("aarch64-apple-darwin", false),
        ("x86_64-pc-windows-msvc", true),
        ("aarch64-pc-windows-msvc", true),
    ];
    let dir = std::path::Path::new("binaries");
    let _ = std::fs::create_dir_all(dir);
    for (triple, windows) in triples {
        let name = if windows {
            format!("transcribe-worker-{triple}.exe")
        } else {
            format!("transcribe-worker-{triple}")
        };
        let path = dir.join(name);
        if path.exists() {
            continue;
        }
        let _ = std::fs::write(&path, b"#!/bin/sh\necho placeholder-sidecar\n");
        #[cfg(unix)]
        if !windows {
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
    if std::env::var("TARGET")
        .unwrap_or_default()
        .contains("apple-darwin")
    {
        for dir in swift_concurrency_lib_dirs() {
            println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
        }
    }
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
            "import_video",
            "import_transcript",
            "rename_artifact",
            "delete_artifact",
            "retry_artifact",
            "export_artifact_audio",
            "export_artifact_transcript",
            "get_artifact_audio_path",
            "get_artifact_video_path",
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
            "start_recording",
            "stop_recording",
            "recording_status",
            "recording_capabilities",
            "get_rms",
            "get_spectrum",
        ]),
    ))
    .expect("tauri build failed");
}

fn swift_concurrency_lib_dirs() -> Vec<std::path::PathBuf> {
    let sys = std::path::Path::new("/usr/lib/swift");
    if sys.is_dir() {
        return vec![sys.to_path_buf()];
    }
    let candidates = [
        "/Library/Developer/CommandLineTools/usr/lib/swift-5.5/macosx",
        "/Library/Developer/CommandLineTools/usr/lib/swift/macosx",
        "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift/macosx",
        "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/swift-5.5/macosx",
    ];
    for c in candidates {
        let p = std::path::Path::new(c);
        if p.join("libswift_Concurrency.dylib").is_file() {
            return vec![p.to_path_buf()];
        }
    }
    Vec::new()
}
