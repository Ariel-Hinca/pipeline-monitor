import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Item = {
  id: string;
  label: string;
  check: string;
  done: boolean;
};

export type Stage = {
  id: string;
  title: string;
  next: string;
  items: Item[];
};

export type Note = {
  at: number;
  windowTitle: string;
  text: string;
  guidance: string;
};

export type Snapshot = {
  stages: Stage[];
  currentId: string;
  complete: boolean;
  percent: number;
  doneCount: number;
  total: number;
  projectPath: string;
  expanded: boolean;
  clickThrough: boolean;
  configPath: string;
  notes: Note[];
  message: string;
  hotkey: string;
};

export function loadSnapshot() {
  return invoke<Snapshot>("snapshot_state");
}

export function setItem(stageId: string, itemId: string, done: boolean) {
  return invoke<Snapshot>("set_item", { stageId, itemId, done });
}

export function markStage(stageId: string, done: boolean) {
  return invoke<Snapshot>("mark_stage_done", { stageId, done });
}

export function setProjectPath(path: string) {
  return invoke<Snapshot>("set_project_path", { path });
}

export function runCheck() {
  return invoke<Snapshot>("run_check");
}

export function saveNote(text: string) {
  return invoke<Snapshot>("save_note", { text });
}

export function setExpanded(expanded: boolean) {
  return invoke<Snapshot>("set_expanded", { expanded });
}

export function setClickThrough(enabled: boolean) {
  return invoke<Snapshot>("set_click_through", { enabled });
}

export function reloadPipeline() {
  return invoke<Snapshot>("reload_pipeline");
}

export function hideOverlay() {
  return invoke<void>("hide_overlay");
}

export function onSnapshot(handler: (snap: Snapshot) => void): Promise<UnlistenFn> {
  return listen<Snapshot>("snapshot", (event) => handler(event.payload));
}
