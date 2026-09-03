import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

import type { LogChunk, LogSource } from "../types";

export async function getProjectLogs(
  projectId: string,
  source: LogSource,
  tail = 2000,
): Promise<LogChunk> {
  return invoke<LogChunk>("get_project_logs", { projectId, source, tail });
}

export async function clearProjectLogs(projectId: string): Promise<void> {
  return invoke<void>("clear_project_logs", { projectId });
}

export async function projectHasCompose(projectId: string): Promise<boolean> {
  return invoke<boolean>("project_has_compose", { projectId });
}

export function listenProjectLogs(
  projectId: string,
  onLines: (lines: readonly string[]) => void,
): Promise<UnlistenFn> {
  return listen<LogChunk>("project-log", (event) => {
    if (event.payload.projectId !== projectId) {
      return;
    }
    if (event.payload.source !== "process") {
      return;
    }
    if (event.payload.lines.length === 0) {
      return;
    }
    onLines(event.payload.lines);
  });
}
