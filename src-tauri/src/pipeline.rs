use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path};
use std::process::Command;

pub const HOTKEY: &str = "Alt+Shift+P";
const MAX_NOTES: usize = 8;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    pub label: String,
    pub check: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Stage {
    pub id: String,
    pub title: String,
    pub next: String,
    pub items: Vec<Item>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub stages: Vec<Stage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Note {
    pub at: u64,
    pub window_title: String,
    pub text: String,
    pub guidance: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Progress {
    #[serde(default)]
    pub checked: BTreeMap<String, bool>,
    #[serde(default)]
    pub project_path: String,
    #[serde(default)]
    pub expanded: bool,
    #[serde(default)]
    pub click_through: bool,
    #[serde(default)]
    pub notes: Vec<Note>,
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            checked: BTreeMap::new(),
            project_path: String::new(),
            expanded: false,
            click_through: false,
            notes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemView {
    pub id: String,
    pub label: String,
    pub check: String,
    pub done: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StageView {
    pub id: String,
    pub title: String,
    pub next: String,
    pub items: Vec<ItemView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub stages: Vec<StageView>,
    pub current_id: String,
    pub complete: bool,
    pub percent: u8,
    pub done_count: usize,
    pub total: usize,
    pub project_path: String,
    pub expanded: bool,
    pub click_through: bool,
    pub config_path: String,
    pub notes: Vec<Note>,
    pub message: String,
    pub hotkey: String,
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub ok: bool,
    pub detail: String,
}

#[cfg_attr(not(test), allow(dead_code))]
pub fn bundled() -> Pipeline {
    parse_pipeline(include_str!("../../pipeline.json")).expect("bundled pipeline.json")
}

pub fn parse_pipeline(text: &str) -> Result<Pipeline, String> {
    let pipeline: Pipeline = serde_json::from_str(text).map_err(|e| e.to_string())?;
    if pipeline.stages.is_empty() {
        return Err("pipeline.json has no stages".into());
    }
    for stage in &pipeline.stages {
        if stage.id.is_empty() || stage.items.is_empty() {
            return Err(format!("stage '{}' needs an id and at least one check", stage.title));
        }
    }
    Ok(pipeline)
}

pub fn parse_progress(text: &str) -> Result<Progress, String> {
    serde_json::from_str(text).map_err(|e| e.to_string())
}

pub fn key(stage: &str, item: &str) -> String {
    format!("{stage}.{item}")
}

pub fn is_checked(progress: &Progress, stage: &str, item: &str) -> bool {
    progress.checked.get(&key(stage, item)).copied().unwrap_or(false)
}

pub fn counts(pipeline: &Pipeline, progress: &Progress) -> (usize, usize) {
    let mut done = 0;
    let mut total = 0;
    for stage in &pipeline.stages {
        for item in &stage.items {
            total += 1;
            if is_checked(progress, &stage.id, &item.id) {
                done += 1;
            }
        }
    }
    (done, total)
}

pub fn percent(done: usize, total: usize) -> u8 {
    if total == 0 {
        0
    } else {
        ((done * 100) / total) as u8
    }
}

/// First stage that still has an open check. The last stage once everything is checked.
pub fn current_index(pipeline: &Pipeline, progress: &Progress) -> usize {
    pipeline
        .stages
        .iter()
        .position(|stage| {
            stage
                .items
                .iter()
                .any(|item| !is_checked(progress, &stage.id, &item.id))
        })
        .unwrap_or_else(|| pipeline.stages.len().saturating_sub(1))
}

pub fn snapshot(
    pipeline: &Pipeline,
    progress: &Progress,
    config_path: &str,
    message: &str,
) -> Snapshot {
    let (done_count, total) = counts(pipeline, progress);
    let complete = total > 0 && done_count == total;
    let index = current_index(pipeline, progress);
    let current_id = pipeline
        .stages
        .get(index)
        .map(|s| s.id.clone())
        .unwrap_or_default();
    let stages = pipeline
        .stages
        .iter()
        .map(|stage| StageView {
            id: stage.id.clone(),
            title: stage.title.clone(),
            next: stage.next.clone(),
            items: stage
                .items
                .iter()
                .map(|item| ItemView {
                    id: item.id.clone(),
                    label: item.label.clone(),
                    check: item.check.clone(),
                    done: is_checked(progress, &stage.id, &item.id),
                })
                .collect(),
        })
        .collect();
    Snapshot {
        stages,
        current_id,
        complete,
        percent: percent(done_count, total),
        done_count,
        total,
        project_path: progress.project_path.clone(),
        expanded: progress.expanded,
        click_through: progress.click_through,
        config_path: config_path.to_string(),
        notes: progress.notes.clone(),
        message: message.to_string(),
        hotkey: HOTKEY.to_string(),
    }
}

pub fn set_checked(progress: &mut Progress, stage: &str, item: &str, done: bool) {
    progress.checked.insert(key(stage, item), done);
}

pub fn mark_stage(pipeline: &Pipeline, progress: &mut Progress, stage_id: &str, done: bool) -> bool {
    let Some(stage) = pipeline.stages.iter().find(|s| s.id == stage_id) else {
        return false;
    };
    for item in &stage.items {
        set_checked(progress, &stage.id, &item.id, done);
    }
    true
}

pub fn push_note(progress: &mut Progress, window_title: &str, text: &str, guidance: &str) {
    progress.notes.insert(
        0,
        Note {
            at: now_secs(),
            window_title: window_title.to_string(),
            text: text.trim().to_string(),
            guidance: guidance.to_string(),
        },
    );
    progress.notes.truncate(MAX_NOTES);
}

pub fn guidance_for(title: &str) -> String {
    let t = title.to_lowercase();
    if t.is_empty() {
        return "No window title. Write what you see so the note is still useful.".into();
    }
    if t.contains("github") {
        return "The front window looks like GitHub. That lines up with version control.".into();
    }
    if t.contains("supabase") {
        return "The front window looks like Supabase. That lines up with the backend stage.".into();
    }
    if t.contains("vercel") || t.contains("netlify") {
        return "The front window looks like a host dashboard. That lines up with deployment.".into();
    }
    if t.contains("cursor") || t.contains("code") || t.contains("vscode") {
        return "The front window looks like an editor. That lines up with local development.".into();
    }
    format!("Front window: {title}. Compare it with the stage you are on.")
}

pub fn evaluate(check: &str, root: &Path) -> Outcome {
    if check == "manual" {
        return Outcome {
            ok: false,
            detail: "this one is yours to confirm".into(),
        };
    }
    if let Some(rel) = check.strip_prefix("file:") {
        return path_check(rel, root, true);
    }
    if let Some(rel) = check.strip_prefix("dir:") {
        return path_check(rel, root, false);
    }
    match check {
        "git" => git_repo(root),
        "remote" => git_remote(root),
        "env" => {
            let ok = root.join(".env").is_file() || root.join(".env.local").is_file();
            Outcome {
                ok,
                detail: if ok {
                    "secrets file is present".into()
                } else {
                    "no .env or .env.local in this folder".into()
                },
            }
        }
        "tests" => tests(root),
        "backend" => any_path(
            root,
            &[
                ("supabase/config.toml", true),
                ("prisma/schema.prisma", true),
                ("drizzle.config.ts", true),
                ("docker-compose.yml", true),
                ("compose.yaml", true),
            ],
            "no supabase, prisma, drizzle, or compose file",
        ),
        "deploy" => any_path(
            root,
            &[
                ("vercel.json", true),
                ("netlify.toml", true),
                ("fly.toml", true),
                ("Dockerfile", true),
            ],
            "no vercel.json, netlify.toml, fly.toml, or Dockerfile",
        ),
        other => Outcome {
            ok: false,
            detail: format!("unknown check '{other}'"),
        },
    }
}

fn path_check(rel: &str, root: &Path, want_file: bool) -> Outcome {
    let rel = rel.trim();
    if !relative_ok(rel) {
        return Outcome {
            ok: false,
            detail: format!("check path is not allowed: {rel}"),
        };
    }
    let path = root.join(rel);
    let ok = if want_file { path.is_file() } else { path.is_dir() };
    Outcome {
        ok,
        detail: if ok {
            format!("{rel} is present")
        } else {
            format!("{rel} is missing")
        },
    }
}

fn any_path(root: &Path, candidates: &[(&str, bool)], miss: &str) -> Outcome {
    for (rel, want_file) in candidates {
        let path = root.join(rel);
        let ok = if *want_file { path.is_file() } else { path.is_dir() };
        if ok {
            return Outcome {
                ok: true,
                detail: format!("{rel} is present"),
            };
        }
    }
    Outcome {
        ok: false,
        detail: miss.into(),
    }
}

fn relative_ok(rel: &str) -> bool {
    let path = Path::new(rel);
    !rel.is_empty()
        && !path.is_absolute()
        && path.components().all(|c| matches!(c, Component::Normal(_)))
}

fn git_repo(root: &Path) -> Outcome {
    match git(root, &["rev-parse", "--is-inside-work-tree"]) {
        Err(detail) => Outcome { ok: false, detail },
        Ok(text) if text == "true" => Outcome {
            ok: true,
            detail: "git repository".into(),
        },
        Ok(_) => Outcome {
            ok: false,
            detail: "not a git repository".into(),
        },
    }
}

fn git_remote(root: &Path) -> Outcome {
    match git(root, &["remote"]) {
        Err(detail) => Outcome { ok: false, detail },
        Ok(text) if !text.is_empty() => Outcome {
            ok: true,
            detail: "remote is set".into(),
        },
        Ok(_) => Outcome {
            ok: false,
            detail: "no git remote".into(),
        },
    }
}

fn tests(root: &Path) -> Outcome {
    let markers = [
        "playwright.config.ts",
        "playwright.config.js",
        "vitest.config.ts",
        "vitest.config.js",
        "jest.config.js",
        "jest.config.ts",
        "cypress.config.js",
        "cypress.config.ts",
        "pytest.ini",
    ];
    for name in markers {
        if root.join(name).is_file() {
            return Outcome {
                ok: true,
                detail: format!("{name} is present"),
            };
        }
    }
    if root.join("tests").is_dir() || root.join("__tests__").is_dir() {
        return Outcome {
            ok: true,
            detail: "a tests directory is present".into(),
        };
    }
    if package_has_test_script(root) {
        return Outcome {
            ok: true,
            detail: "package.json has a test script".into(),
        };
    }
    Outcome {
        ok: false,
        detail: "no test config, tests directory, or test script".into(),
    }
}

fn package_has_test_script(root: &Path) -> bool {
    let Ok(text) = fs::read_to_string(root.join("package.json")) else {
        return false;
    };
    let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) else {
        return false;
    };
    json.get("scripts")
        .and_then(|s| s.get("test"))
        .and_then(|v| v.as_str())
        .is_some_and(|script| !script.is_empty() && !script.starts_with("echo "))
}

fn git(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .map_err(|_| "git is not installed".to_string())?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Ok(String::new())
    }
}

fn now_secs() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn empty() -> Progress {
        Progress::default()
    }

    #[test]
    fn bundled_pipeline_has_the_default_stages() {
        let pipeline = bundled();
        let ids: Vec<_> = pipeline.stages.iter().map(|s| s.id.as_str()).collect();
        assert_eq!(
            ids,
            vec![
                "planning", "local", "git", "backend", "auth", "testing", "deploy", "domain",
                "monitoring", "compliance", "launch"
            ]
        );
    }

    #[test]
    fn current_stage_is_the_first_open_one() {
        let pipeline = bundled();
        let mut progress = empty();
        assert_eq!(current_index(&pipeline, &progress), 0);
        mark_stage(&pipeline, &mut progress, "planning", true);
        assert_eq!(current_index(&pipeline, &progress), 1);
        let (done, total) = counts(&pipeline, &progress);
        assert_eq!(done, 3);
        assert!(total > done);
        assert_eq!(percent(0, 0), 0);
        assert_eq!(percent(1, 4), 25);
    }

    #[test]
    fn file_checks_stay_inside_the_folder() {
        let dir = std::env::temp_dir().join(format!(
            "pipeline-supervisor-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(dir.join("supabase")).unwrap();
        fs::write(dir.join("supabase/config.toml"), "").unwrap();
        fs::write(dir.join("vercel.json"), "{}").unwrap();
        fs::write(dir.join(".env"), "SECRET=1\n").unwrap();
        assert!(evaluate("file:vercel.json", &dir).ok);
        assert!(evaluate("dir:supabase", &dir).ok);
        assert!(evaluate("env", &dir).ok);
        assert!(evaluate("deploy", &dir).ok);
        assert!(evaluate("backend", &dir).ok);
        assert!(!evaluate("file:../secret", &dir).ok);
        assert!(!evaluate("file:/etc/passwd", &dir).ok);
        let env = fs::read_to_string(dir.join(".env")).unwrap();
        assert!(env.contains("SECRET"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn guidance_uses_the_window_title_only() {
        assert!(guidance_for("").contains("No window title"));
        assert!(guidance_for("GitHub").contains("version control"));
        assert!(guidance_for("Cursor").contains("editor"));
    }
}
