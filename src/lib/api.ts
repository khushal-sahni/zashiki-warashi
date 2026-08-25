import { invoke } from "@tauri-apps/api/core";

import type {
  AppSettings,
  AppStatus,
  Project,
  ScanCandidate,
} from "../types";

export async function getAppStatus(): Promise<AppStatus> {
  return invoke<AppStatus>("get_app_status");
}

export async function listProjects(): Promise<Project[]> {
  return invoke<Project[]>("list_projects");
}

export async function addProject(path: string): Promise<Project> {
  return invoke<Project>("add_project", { path });
}

export async function removeProject(id: string): Promise<void> {
  return invoke<void>("remove_project", { id });
}

export async function updateProjectCommands(
  id: string,
  startCommand: string,
  stopCommand: string,
): Promise<Project> {
  return invoke<Project>("update_project_commands", {
    id,
    startCommand,
    stopCommand,
  });
}

export async function scanProjects(roots?: string[]): Promise<ScanCandidate[]> {
  return invoke<ScanCandidate[]>("scan_projects", { roots: roots ?? null });
}

export async function getSettings(): Promise<AppSettings> {
  return invoke<AppSettings>("get_settings");
}

export async function setScanRoots(roots: string[]): Promise<AppSettings> {
  return invoke<AppSettings>("set_scan_roots", { roots });
}

export async function startProject(id: string): Promise<Project> {
  return invoke<Project>("start_project", { id });
}

export async function stopProject(id: string): Promise<Project> {
  return invoke<Project>("stop_project", { id });
}

export async function restartProject(id: string): Promise<Project> {
  return invoke<Project>("restart_project", { id });
}
