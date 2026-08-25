export type RunState = "stopped" | "starting" | "running" | "failed";

export interface ProjectRun {
  readonly projectId: string;
  readonly pid: number | null;
  readonly pgid: number | null;
  readonly startedAt: string | null;
  readonly startedAtUnix: number | null;
  readonly status: RunState;
  readonly lastError: string | null;
}

export interface Project {
  readonly id: string;
  readonly name: string;
  readonly path: string;
  readonly startCommand: string | null;
  readonly stopCommand: string | null;
  readonly createdAt: string;
  readonly updatedAt: string;
  readonly inferredStartCommand: string | null;
  readonly run: ProjectRun;
}

export interface ScanCandidate {
  readonly name: string;
  readonly path: string;
  readonly inferredStartCommand: string | null;
  readonly alreadyRegistered: boolean;
}

export interface AppSettings {
  readonly scanRoots: string[];
}
