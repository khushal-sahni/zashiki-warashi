export type JobSchedule =
  | { readonly kind: "interval"; readonly minutes: number }
  | { readonly kind: "daily"; readonly hour: number; readonly minute: number }
  | {
      readonly kind: "weekly";
      readonly weekdays: readonly number[];
      readonly hour: number;
      readonly minute: number;
    };

export type JobPolicy = "optimistic" | "exact";
export type NetworkPolicy = "bestEffort" | "required" | "none";
export type RunTrigger = "calendar" | "catchUp" | "manual";
export type JobRunStatus =
  | "running"
  | "succeeded"
  | "failed"
  | "timedOut"
  | "skippedOffline"
  | "interrupted";
export type NetworkOutcome = "online" | "offline" | "notChecked";
export type PowerSource = "ac" | "battery" | "unknown";

export interface JobInput {
  readonly name: string;
  readonly command: string;
  readonly workingDir: string | null;
  readonly enabled: boolean;
  readonly schedule: JobSchedule;
  readonly policy: JobPolicy;
  readonly network: NetworkPolicy;
  readonly networkGraceSeconds: number;
  readonly maxRuntimeSeconds: number | null;
}

export interface JobRun {
  readonly id: string;
  readonly jobId: string;
  readonly trigger: RunTrigger;
  readonly status: JobRunStatus;
  readonly scheduledForUnix: number | null;
  readonly startedAtUnix: number;
  readonly finishedAtUnix: number | null;
  readonly exitCode: number | null;
  readonly network: NetworkOutcome | null;
  readonly powerSource: PowerSource | null;
  readonly runnerPid: number | null;
  readonly logPath: string;
  readonly message: string | null;
}

export interface JobSummary extends JobInput {
  readonly id: string;
  readonly createdAt: number;
  readonly updatedAt: number;
  readonly nextFireUnix: number | null;
  readonly lastRun: JobRun | null;
}

export interface ForeignAgent {
  readonly label: string;
  readonly program: string | null;
  readonly plistPath: string;
  readonly scheduleHint: string | null;
  readonly loaded: boolean;
  readonly pid: number | null;
  readonly lastExit: number | null;
}

export interface JobSystemStatus {
  readonly supervisorInstalled: boolean;
  readonly wakeHelperInstalled: boolean;
  readonly powerSource: PowerSource;
  readonly online: boolean;
  readonly socketPath: string;
  readonly mcpCommand: string;
}

export interface JobLogTail {
  readonly runId: string;
  readonly lines: readonly string[];
}
