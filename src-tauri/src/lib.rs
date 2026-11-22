mod pipeline;

use pipeline::{
    evaluate, guidance_for, is_checked, mark_stage, parse_pipeline, parse_progress, push_note,
    set_checked, snapshot, Progress, Snapshot, HOTKEY,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, LogicalSize, Manager, WindowEvent};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

const COMPACT_HEIGHT: f64 = 400.0;
const EXPANDED_HEIGHT: f64 = 640.0;
const WINDOW_WIDTH: f64 = 400.0;

fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir)
}

fn pipeline_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join("pipeline.json"))
}

fn progress_path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(data_dir(app)?.join("progress.json"))
}

fn write_atomic(path: &Path, text: &str) -> Result<(), String> {
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())?;
    Ok(())
}

fn load_pipeline(app: &AppHandle) -> Result<(pipeline::Pipeline, PathBuf), String> {
    let path = pipeline_path(app)?;
    if !path.exists() {
        write_atomic(&path, include_str!("../../pipeline.json"))?;
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let parsed = parse_pipeline(&text).map_err(|e| format!("{e}. Edit {} and reload.", path.display()))?;
    Ok((parsed, path))
}

fn load_progress(app: &AppHandle) -> Result<Progress, String> {
    let path = progress_path(app)?;
    if !path.exists() {
        return Ok(Progress::default());
    }
    let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    match parse_progress(&text) {
        Ok(progress) => Ok(progress),
        Err(_) => {
            let bak = path.with_extension("json.bak");
            let _ = fs::rename(&path, &bak);
            Err(format!(
                "Saved progress could not be read. The old file was kept as {}.",
                bak.display()
            ))
        }
    }
}

fn save_progress(app: &AppHandle, progress: &Progress) -> Result<(), String> {
    let text = serde_json::to_string_pretty(progress).map_err(|e| e.to_string())?;
    write_atomic(&progress_path(app)?, &text)
}

fn view(app: &AppHandle, progress: &Progress, message: &str) -> Result<Snapshot, String> {
    let (pipeline, path) = load_pipeline(app)?;
    Ok(snapshot(&pipeline, progress, &path.display().to_string(), message))
}

fn publish(app: &AppHandle, snap: &Snapshot) {
    let _ = app.emit("snapshot", snap);
}

fn window(app: &AppHandle) -> Result<tauri::WebviewWindow, String> {
    app.get_webview_window("main")
        .ok_or_else(|| "The overlay window is missing.".into())
}

fn apply_click_through(app: &AppHandle, enabled: bool) -> Result<(), String> {
    window(app)?
        .set_ignore_cursor_events(enabled)
        .map_err(|e| e.to_string())
}

fn apply_size(app: &AppHandle, expanded: bool) -> Result<(), String> {
    let height = if expanded { EXPANDED_HEIGHT } else { COMPACT_HEIGHT };
    window(app)?
        .set_size(LogicalSize::new(WINDOW_WIDTH, height))
        .map_err(|e| e.to_string())
}

fn show_window(app: &AppHandle) {
    if let Ok(win) = window(app) {
        let _ = win.show();
    }
}

fn hide_window(app: &AppHandle) {
    if let Ok(win) = window(app) {
        let _ = win.hide();
    }
}

fn toggle_from_hotkey(app: &AppHandle) {
    let Ok(win) = window(app) else { return };
    let visible = win.is_visible().unwrap_or(false);
    if !visible {
        let _ = win.show();
        return;
    }
    if let Ok(progress) = load_progress(app) {
        if progress.click_through {
            let mut progress = progress;
            progress.click_through = false;
            let _ = save_progress(app, &progress);
            let _ = apply_click_through(app, false);
            if let Ok(snap) = view(app, &progress, "Click-through is off.") {
                publish(app, &snap);
            }
            return;
        }
    }
    let _ = win.hide();
}

fn toggle_click_through(app: &AppHandle) -> Result<Snapshot, String> {
    let mut progress = load_progress(app)?;
    progress.click_through = !progress.click_through;
    save_progress(app, &progress)?;
    apply_click_through(app, progress.click_through)?;
    let message = if progress.click_through {
        "Click-through is on. Alt+Shift+P or the tray menu turns it off."
    } else {
        "Click-through is off."
    };
    let snap = view(app, &progress, message)?;
    publish(app, &snap);
    Ok(snap)
}

fn active_window_title() -> String {
    // shortcut: Windows has no title lookup yet; the note still saves. Upgrade with GetForegroundWindow.
    #[cfg(target_os = "linux")]
    {
        let Ok(out) = Command::new("xdotool")
            .args(["getactivewindow", "getwindowname"])
            .output()
        else {
            return String::new();
        };
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
        String::new()
    }
    #[cfg(target_os = "macos")]
    {
        let Ok(out) = Command::new("osascript")
            .args([
                "-e",
                r#"tell application "System Events" to get name of first application process whose frontmost is true"#,
            ])
            .output()
        else {
            return String::new();
        };
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
        String::new()
    }
    #[cfg(target_os = "windows")]
    {
        String::new()
    }
}

#[tauri::command]
fn snapshot_state(app: AppHandle) -> Result<Snapshot, String> {
    let progress = load_progress(&app)?;
    view(&app, &progress, "")
}

#[tauri::command]
fn set_item(app: AppHandle, stage_id: String, item_id: String, done: bool) -> Result<Snapshot, String> {
    let (pipeline, _) = load_pipeline(&app)?;
    if !pipeline
        .stages
        .iter()
        .any(|s| s.id == stage_id && s.items.iter().any(|i| i.id == item_id))
    {
        return Err("That check is not in the pipeline.".into());
    }
    let mut progress = load_progress(&app)?;
    set_checked(&mut progress, &stage_id, &item_id, done);
    save_progress(&app, &progress)?;
    view(&app, &progress, "")
}

#[tauri::command]
fn mark_stage_done(app: AppHandle, stage_id: String, done: bool) -> Result<Snapshot, String> {
    let (pipeline, _) = load_pipeline(&app)?;
    let mut progress = load_progress(&app)?;
    if !mark_stage(&pipeline, &mut progress, &stage_id, done) {
        return Err("That stage is not in the pipeline.".into());
    }
    save_progress(&app, &progress)?;
    let message = if done {
        "Stage marked done."
    } else {
        "Stage marked open."
    };
    view(&app, &progress, message)
}

#[tauri::command]
fn set_project_path(app: AppHandle, path: String) -> Result<Snapshot, String> {
    let mut progress = load_progress(&app)?;
    progress.project_path = path.trim().to_string();
    save_progress(&app, &progress)?;
    view(&app, &progress, "Project folder saved.")
}

#[tauri::command]
fn run_check(app: AppHandle) -> Result<Snapshot, String> {
    let (pipeline, _) = load_pipeline(&app)?;
    let mut progress = load_progress(&app)?;
    let folder = PathBuf::from(progress.project_path.trim());
    if progress.project_path.trim().is_empty() {
        return view(&app, &progress, "Set a project folder, then run the check again.");
    }
    if !folder.is_dir() {
        return view(&app, &progress, "That folder is not on this machine.");
    }
    let index = pipeline::current_index(&pipeline, &progress);
    let Some(stage) = pipeline.stages.get(index) else {
        return view(&app, &progress, "The pipeline has no stages.");
    };
    let mut open = Vec::new();
    for item in &stage.items {
        if item.check == "manual" {
            if !is_checked(&progress, &stage.id, &item.id) {
                open.push(item.label.clone());
            }
            continue;
        }
        let outcome = evaluate(&item.check, &folder);
        set_checked(&mut progress, &stage.id, &item.id, outcome.ok);
        if !outcome.ok {
            open.push(format!("{} ({})", item.label, outcome.detail));
        }
    }
    save_progress(&app, &progress)?;
    let message = if open.is_empty() {
        "This stage checks out.".to_string()
    } else {
        format!("Still open: {}", open.join("; "))
    };
    view(&app, &progress, &message)
}

#[tauri::command]
fn save_note(app: AppHandle, text: String) -> Result<Snapshot, String> {
    let mut progress = load_progress(&app)?;
    let title = active_window_title();
    let guidance = guidance_for(&title);
    push_note(&mut progress, &title, &text, &guidance);
    save_progress(&app, &progress)?;
    view(&app, &progress, &guidance)
}

#[tauri::command]
fn set_expanded(app: AppHandle, expanded: bool) -> Result<Snapshot, String> {
    let mut progress = load_progress(&app)?;
    progress.expanded = expanded;
    save_progress(&app, &progress)?;
    apply_size(&app, expanded)?;
    view(&app, &progress, "")
}

#[tauri::command]
fn set_click_through(app: AppHandle, enabled: bool) -> Result<Snapshot, String> {
    let mut progress = load_progress(&app)?;
    progress.click_through = enabled;
    save_progress(&app, &progress)?;
    apply_click_through(&app, enabled)?;
    let message = if enabled {
        "Click-through is on. Alt+Shift+P or the tray menu turns it off."
    } else {
        "Click-through is off."
    };
    let snap = view(&app, &progress, message)?;
    publish(&app, &snap);
    Ok(snap)
}

#[tauri::command]
fn reload_pipeline(app: AppHandle) -> Result<Snapshot, String> {
    let progress = load_progress(&app)?;
    let _ = load_pipeline(&app)?;
    view(&app, &progress, "Stages reloaded.")
}

#[tauri::command]
fn hide_overlay(app: AppHandle) -> Result<(), String> {
    hide_window(&app);
    Ok(())
}

fn install_tray(app: &tauri::App) -> tauri::Result<()> {
    let show = MenuItem::with_id(app, "show", "Show", true, None::<&str>)?;
    let hide = MenuItem::with_id(app, "hide", "Hide", true, None::<&str>)?;
    let click = MenuItem::with_id(app, "click", "Toggle click-through", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&show, &hide, &click, &quit])?;
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::Io(std::io::Error::other("The tray icon is missing.")))?;
    TrayIconBuilder::with_id("supervisor")
        .icon(icon)
        .tooltip("Pipeline Supervisor")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => show_window(app),
            "hide" => hide_window(app),
            "click" => {
                let _ = toggle_click_through(app);
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                let Ok(win) = window(app) else { return };
                if win.is_visible().unwrap_or(false) {
                    hide_window(app);
                } else {
                    show_window(app);
                }
            }
        })
        .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        toggle_from_hotkey(app);
                    }
                })
                .build(),
        )
        .setup(|app| {
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            install_tray(app)?;
            let shortcut = Shortcut::new(Some(Modifiers::ALT | Modifiers::SHIFT), Code::KeyP);
            if let Err(err) = app.global_shortcut().register(shortcut) {
                eprintln!("hotkey {HOTKEY} was not registered: {err}");
            }

            let handle = app.handle().clone();
            if let Some(win) = app.get_webview_window("main") {
                win.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        hide_window(&handle);
                    }
                });
            }

            if let Ok(progress) = load_progress(app.handle()) {
                let _ = apply_click_through(app.handle(), progress.click_through);
                let _ = apply_size(app.handle(), progress.expanded);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            snapshot_state,
            set_item,
            mark_stage_done,
            set_project_path,
            run_check,
            save_note,
            set_expanded,
            set_click_through,
            reload_pipeline,
            hide_overlay
        ])
        .run(tauri::generate_context!())
        .expect("error while running pipeline supervisor");
}
