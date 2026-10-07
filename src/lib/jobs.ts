import { invoke } from "@tauri-apps/api/core";

import type {
  ForeignAgent,
  JobInput,
  JobLogTail,
  JobRun,
  JobSummary,
  JobSystemStatus,
} from "../types";

export async function listJobs(): Promise<JobSummary[]> {
  return invoke<JobSummary[]>("list_jobs");
}

export async function listJobRuns(id: string, limit?: number): Promise<JobRun[]> {
  return invoke<JobRun[]>("list_job_runs", { id, limit: limit ?? null });
}

export async function getJobRunLog(runId: string, lines?: number): Promise<JobLogTail> {
  return invoke<JobLogTail>("get_job_run_log", { runId, lines: lines ?? null });
}

export async function createJob(input: JobInput): Promise<JobSummary> {
  return invoke<JobSummary>("create_job", { input });
}

export async function updateJob(id: string, input: JobInput): Promise<JobSummary> {
  return invoke<JobSummary>("update_job", { id, input });
}

export async function setJobEnabled(id: string, enabled: boolean): Promise<JobSummary> {
  return invoke<JobSummary>("set_job_enabled", { id, enabled });
}

export async function deleteJob(id: string): Promise<void> {
  return invoke<void>("delete_job", { id });
}

export async function runJobNow(id: string): Promise<void> {
  return invoke<void>("run_job_now", { id });
}

export async function getJobSystemStatus(): Promise<JobSystemStatus> {
  return invoke<JobSystemStatus>("get_job_system_status");
}

export async function listForeignAgents(): Promise<ForeignAgent[]> {
  return invoke<ForeignAgent[]>("list_foreign_agents");
}

export async function installWakeHelper(): Promise<void> {
  return invoke<void>("install_wake_helper");
}

export async function uninstallWakeHelper(): Promise<void> {
  return invoke<void>("uninstall_wake_helper");
}
