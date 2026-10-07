import { open } from "@tauri-apps/plugin-dialog";
import { useState } from "react";

import { formatInvokeError } from "../../lib";
import type { JobInput, JobPolicy, JobSchedule, JobSummary, NetworkPolicy } from "../../types";
import { formatClock, formatInterval, INTERVAL_OPTIONS, WEEKDAY_SHORT } from "./format";

type ScheduleKind = JobSchedule["kind"];

interface Draft {
  readonly name: string;
  readonly command: string;
  readonly workingDir: string;
  readonly enabled: boolean;
  readonly kind: ScheduleKind;
  readonly intervalMinutes: number;
  readonly time: string;
  readonly weekdays: readonly number[];
  readonly policy: JobPolicy;
  readonly network: NetworkPolicy;
  readonly graceSeconds: string;
  readonly maxRuntimeMinutes: string;
}

interface JobEditorProps {
  readonly job: JobSummary | null;
  readonly busy: boolean;
  readonly wakeHelperInstalled: boolean;
  readonly onSave: (input: JobInput) => Promise<void>;
  readonly onClose: () => void;
}

export function JobEditor({ job, busy, wakeHelperInstalled, onSave, onClose }: JobEditorProps) {
  const [draft, setDraft] = useState<Draft>(() => draftFrom(job));
  const [error, setError] = useState<string | null>(null);
  const set = <K extends keyof Draft>(key: K, value: Draft[K]): void => {
    setDraft((current) => ({ ...current, [key]: value }));
  };

  async function submit(): Promise<void> {
    const result = toInput(draft);
    if (typeof result === "string") {
      setError(result);
      return;
    }
    try {
      setError(null);
      await onSave(result);
    } catch (err) {
      setError(formatInvokeError(err));
    }
  }

  return (
    <form
      className="panel job-editor"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <div className="detail-header">
        <h2>{job ? "Edit job" : "New scheduled job"}</h2>
        <button type="button" className="ghost" onClick={onClose}>
          Close
        </button>
      </div>
      <p className="muted job-editor-lede">
        Zashiki starts the command on schedule, waits for the network, keeps the
        Mac awake while it runs, and keeps the log. The command does the work.
      </p>
      <BasicsFields draft={draft} set={set} />
      <ScheduleFields draft={draft} set={set} />
      <TimingFields draft={draft} set={set} wakeHelperInstalled={wakeHelperInstalled} />
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
      <div className="modal-actions job-editor-actions">
        <button type="submit" className="primary" disabled={busy}>
          {job ? "Save job" : "Create job"}
        </button>
        <button type="button" className="ghost" onClick={onClose}>
          Cancel
        </button>
      </div>
    </form>
  );
}

interface FieldsProps {
  readonly draft: Draft;
  readonly set: <K extends keyof Draft>(key: K, value: Draft[K]) => void;
}

function BasicsFields({ draft, set }: FieldsProps) {
  return (
    <fieldset className="job-fieldset">
      <label>
        Name
        <input value={draft.name} onChange={(e) => set("name", e.target.value)} placeholder="Daily ingest" required />
      </label>
      <label>
        Command
        <textarea
          className="mono-input"
          value={draft.command}
          onChange={(e) => set("command", e.target.value)}
          rows={2}
          placeholder="npm run ingest"
          spellCheck={false}
          required
        />
        <span className="field-hint">Runs in your login shell, so fnm / nvm / Homebrew paths work.</span>
      </label>
      <label>
        Working directory
        <span className="field-row">
          <input
            className="mono-input"
            value={draft.workingDir}
            onChange={(e) => set("workingDir", e.target.value)}
            placeholder="Home folder"
            spellCheck={false}
          />
          <button
            type="button"
            onClick={() => {
              void open({ directory: true, multiple: false, title: "Job working directory" }).then((picked) => {
                if (typeof picked === "string") {
                  set("workingDir", picked);
                }
              });
            }}
          >
            Choose…
          </button>
        </span>
      </label>
    </fieldset>
  );
}

function ScheduleFields({ draft, set }: FieldsProps) {
  return (
    <fieldset className="job-fieldset">
      <legend>Schedule</legend>
      <div className="segmented" role="radiogroup" aria-label="Schedule type">
        {(["interval", "daily", "weekly"] as const).map((kind) => (
          <button
            key={kind}
            type="button"
            role="radio"
            aria-checked={draft.kind === kind}
            className={draft.kind === kind ? "ghost active" : "ghost"}
            onClick={() => set("kind", kind)}
          >
            {kind === "interval" ? "Every…" : kind === "daily" ? "Daily" : "Weekly"}
          </button>
        ))}
      </div>
      {draft.kind === "interval" ? (
        <label>
          Every
          <select value={draft.intervalMinutes} onChange={(e) => set("intervalMinutes", Number(e.target.value))}>
            {INTERVAL_OPTIONS.map((minutes) => (
              <option key={minutes} value={minutes}>
                {formatInterval(minutes)}
              </option>
            ))}
          </select>
          <span className="field-hint">Aligned to the clock (e.g. every 15 min fires at :00, :15, :30, :45).</span>
        </label>
      ) : (
        <label>
          At (local time)
          <input type="time" value={draft.time} onChange={(e) => set("time", e.target.value)} required />
        </label>
      )}
      {draft.kind === "weekly" && <WeekdayPicker draft={draft} set={set} />}
    </fieldset>
  );
}

function WeekdayPicker({ draft, set }: FieldsProps) {
  return (
    <div className="weekday-picker" role="group" aria-label="Weekdays">
      {WEEKDAY_SHORT.map((label, day) => {
        const on = draft.weekdays.includes(day);
        return (
          <button
            key={label}
            type="button"
            aria-pressed={on}
            className={on ? "ghost active" : "ghost"}
            onClick={() =>
              set("weekdays", on ? draft.weekdays.filter((d) => d !== day) : [...draft.weekdays, day])
            }
          >
            {label}
          </button>
        );
      })}
    </div>
  );
}

function TimingFields({ draft, set, wakeHelperInstalled }: FieldsProps & { readonly wakeHelperInstalled: boolean }) {
  return (
    <fieldset className="job-fieldset">
      <legend>If the Mac is asleep</legend>
      <label className="radio-row">
        <input type="radio" checked={draft.policy === "optimistic"} onChange={() => set("policy", "optimistic")} />
        <span>
          <strong>Catch up on wake</strong>
          <span className="field-hint">Runs once as soon as the Mac wakes. Missed runs are merged into one.</span>
        </span>
      </label>
      <label className="radio-row">
        <input type="radio" checked={draft.policy === "exact"} onChange={() => set("policy", "exact")} />
        <span>
          <strong>Wake the Mac on time</strong>
          <span className="field-hint">
            Wakes ~90s early so Wi-Fi can rejoin. Closed-lid runs need power.
            {wakeHelperInstalled ? "" : " Needs the one-time wake helper (Settings)."}
          </span>
        </span>
      </label>
      <div className="field-grid">
        <label>
          Network
          <select value={draft.network} onChange={(e) => set("network", e.target.value as NetworkPolicy)}>
            <option value="bestEffort">Preferred: wait, then run anyway</option>
            <option value="required">Required: skip if still offline</option>
            <option value="none">Not needed</option>
          </select>
        </label>
        <label>
          Wait for network (s)
          <input
            type="number"
            min={0}
            max={600}
            value={draft.graceSeconds}
            disabled={draft.network === "none"}
            onChange={(e) => set("graceSeconds", e.target.value)}
          />
        </label>
        <label>
          Max runtime (min)
          <input
            type="number"
            min={1}
            value={draft.maxRuntimeMinutes}
            placeholder="No limit"
            onChange={(e) => set("maxRuntimeMinutes", e.target.value)}
          />
        </label>
      </div>
      <label className="check-row">
        <input type="checkbox" checked={draft.enabled} onChange={(e) => set("enabled", e.target.checked)} />
        Enabled
      </label>
    </fieldset>
  );
}

function draftFrom(job: JobSummary | null): Draft {
  const schedule = job?.schedule;
  const time =
    schedule && schedule.kind !== "interval" ? formatClock(schedule.hour, schedule.minute) : "09:00";
  return {
    name: job?.name ?? "",
    command: job?.command ?? "",
    workingDir: job?.workingDir ?? "",
    enabled: job?.enabled ?? true,
    kind: schedule?.kind ?? "daily",
    intervalMinutes: schedule?.kind === "interval" ? schedule.minutes : 60,
    time,
    weekdays: schedule?.kind === "weekly" ? schedule.weekdays : [1, 2, 3, 4, 5],
    policy: job?.policy ?? "optimistic",
    network: job?.network ?? "bestEffort",
    graceSeconds: String(job?.networkGraceSeconds ?? 90),
    maxRuntimeMinutes: job?.maxRuntimeSeconds ? String(Math.round(job.maxRuntimeSeconds / 60)) : "",
  };
}

function toSchedule(draft: Draft): JobSchedule | string {
  if (draft.kind === "interval") {
    return { kind: "interval", minutes: draft.intervalMinutes };
  }
  const [hour, minute] = draft.time.split(":").map(Number);
  if (hour === undefined || minute === undefined || Number.isNaN(hour) || Number.isNaN(minute)) {
    return "Pick a time.";
  }
  if (draft.kind === "daily") {
    return { kind: "daily", hour, minute };
  }
  if (draft.weekdays.length === 0) {
    return "Pick at least one weekday.";
  }
  return { kind: "weekly", weekdays: [...draft.weekdays].sort((a, b) => a - b), hour, minute };
}

function toInput(draft: Draft): JobInput | string {
  const schedule = toSchedule(draft);
  if (typeof schedule === "string") {
    return schedule;
  }
  const grace = Number(draft.graceSeconds || "0");
  const maxMinutes = draft.maxRuntimeMinutes.trim() === "" ? null : Number(draft.maxRuntimeMinutes);
  if (!Number.isInteger(grace) || grace < 0 || grace > 600) {
    return "Network wait must be 0–600 seconds.";
  }
  if (maxMinutes !== null && (!Number.isInteger(maxMinutes) || maxMinutes < 1)) {
    return "Max runtime must be a whole number of minutes, or empty for no limit.";
  }
  return {
    name: draft.name,
    command: draft.command,
    workingDir: draft.workingDir.trim() || null,
    enabled: draft.enabled,
    schedule,
    policy: draft.policy,
    network: draft.network,
    networkGraceSeconds: grace,
    maxRuntimeSeconds: maxMinutes === null ? null : maxMinutes * 60,
  };
}
