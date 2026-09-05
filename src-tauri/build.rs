fn main() {
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
        ]),
    ))
    .expect("tauri build failed");
}
