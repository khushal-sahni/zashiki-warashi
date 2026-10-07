import type {
  JobPolicy,
  JobRun,
  JobRunStatus,
  JobSchedule,
  NetworkPolicy,
  RunTrigger,
} from "../../types";

export const WEEKDAY_SHORT = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"] as const;

export const INTERVAL_OPTIONS: readonly number[] = [
  5, 10, 15, 20, 30, 60, 120, 180, 240, 360, 480, 720, 1440,
];

function pad(value: number): string {
  return value.toString().padStart(2, "0");
}

export function formatClock(hour: number, minute: number): string {
  return `${pad(hour)}:${pad(minute)}`;
}

export function formatInterval(minutes: number): string {
  if (minutes < 60) {
    return `${minutes} min`;
  }
  const hours = minutes / 60;
  return hours === 1 ? "hour" : `${hours} h`;
}

export function describeSchedule(schedule: JobSchedule): string {
  switch (schedule.kind) {
    case "interval":
      return `Every ${formatInterval(schedule.minutes)}`;
    case "daily":
      return `Daily at ${formatClock(schedule.hour, schedule.minute)}`;
    case "weekly": {
      const days = [...schedule.weekdays]
        .sort((a, b) => a - b)
        .map((day) => WEEKDAY_SHORT[day] ?? "?")
        .join(", ");
      return `${days} at ${formatClock(schedule.hour, schedule.minute)}`;
    }
  }
}

export function describePolicy(policy: JobPolicy): string {
  return policy === "exact" ? "Wake the Mac on time" : "Run when awake, catch up on wake";
}

export function describeNetwork(network: NetworkPolicy, graceSeconds: number): string {
  switch (network) {
    case "required":
      return `Required (wait ${graceSeconds}s, skip if offline)`;
    case "bestEffort":
      return `Preferred (wait ${graceSeconds}s, run anyway)`;
    case "none":
      return "Not needed";
  }
}

export function describeMaxRuntime(seconds: number | null): string {
  if (seconds === null) {
    return "No limit";
  }
  return formatDuration(seconds);
}

export function formatDuration(seconds: number): string {
  if (seconds < 60) {
    return `${seconds}s`;
  }
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) {
    return `${minutes} min`;
  }
  const hours = Math.floor(minutes / 60);
  const rest = minutes % 60;
  return rest === 0 ? `${hours} h` : `${hours} h ${rest} min`;
}

export function formatDateTime(unix: number): string {
  return new Date(unix * 1000).toLocaleString(undefined, {
    weekday: "short",
    month: "short",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function formatRelative(unix: number, nowMs: number = Date.now()): string {
  const deltaSeconds = Math.round(unix - nowMs / 1000);
  const abs = Math.abs(deltaSeconds);
  const span =
    abs < 60 ? "<1 min" : abs < 3600 ? `${Math.round(abs / 60)} min` : abs < 86_400 ? `${Math.round(abs / 3600)} h` : `${Math.round(abs / 86_400)} d`;
  return deltaSeconds >= 0 ? `in ${span}` : `${span} ago`;
}

const STATUS_LABEL: Record<JobRunStatus, string> = {
  running: "Running",
  succeeded: "Succeeded",
  failed: "Failed",
  timedOut: "Timed out",
  skippedOffline: "Skipped (offline)",
  interrupted: "Interrupted",
};

const TRIGGER_LABEL: Record<RunTrigger, string> = {
  calendar: "On time",
  catchUp: "Catch-up",
  manual: "Manual",
};

export function statusLabel(status: JobRunStatus): string {
  return STATUS_LABEL[status];
}

export function triggerLabel(trigger: RunTrigger): string {
  return TRIGGER_LABEL[trigger];
}

/** Maps a run status onto the shared status-dot / badge palette. */
export function statusTone(
  status: JobRunStatus | null,
): "running" | "ok" | "failed" | "starting" | "stopped" {
  switch (status) {
    case "running":
      return "running";
    case "succeeded":
      return "ok";
    case "failed":
    case "timedOut":
    case "interrupted":
      return "failed";
    case "skippedOffline":
      return "starting";
    default:
      return "stopped";
  }
}

export function runDuration(run: JobRun): string | null {
  if (run.finishedAtUnix === null) {
    return null;
  }
  return formatDuration(Math.max(0, run.finishedAtUnix - run.startedAtUnix));
}
