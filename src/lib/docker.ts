import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";

import type { PortConflict, ProjectStack, ReconcileAction } from "../types";

export async function peekProjectStack(projectId: string): Promise<ProjectStack> {
  return invoke<ProjectStack>("peek_project_stack", { projectId });
}

export async function getProjectStack(projectId: string): Promise<ProjectStack> {
  return invoke<ProjectStack>("get_project_stack", { projectId });
}

export async function startProjectStack(projectId: string): Promise<ProjectStack> {
  return invoke<ProjectStack>("start_project_stack", { projectId });
}

export async function stopProjectStack(projectId: string): Promise<ProjectStack> {
  return invoke<ProjectStack>("stop_project_stack", { projectId });
}

export async function resolvePortConflict(
  projectId: string,
  action: ReconcileAction,
  writeToRepo: boolean,
  confirmNative: boolean,
  conflict: PortConflict | null = null,
): Promise<ProjectStack> {
  return invoke<ProjectStack>("resolve_port_conflict", {
    projectId,
    action,
    writeToRepo,
    confirmNative,
    conflict,
  });
}

export async function openCompass(uri: string): Promise<void> {
  await openUrl(uri);
}
