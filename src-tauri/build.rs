fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "snapshot_state",
                "set_item",
                "mark_stage_done",
                "set_project_path",
                "run_check",
                "save_note",
                "set_expanded",
                "set_click_through",
                "reload_pipeline",
                "hide_overlay",
            ]),
        ),
    )
    .expect("failed to run tauri-build");
}
