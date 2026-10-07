import { useEffect, useState } from "react";

import { DetailSplit, usePaneCollapse, usePaneControls } from "../../components";
import { formatInvokeError, listJobRuns } from "../../lib";
import type { JobRun, JobSummary, JobSystemStatus } from "../../types";
import {
  describeMaxRuntime,
  describeNetwork,
  describePolicy,
  describeSchedule,
  formatDateTime,
  formatRelative,
  runDuration,
  statusLabel,
  statusTone,
  triggerLabel,
} from "./format";
import { JobRunLog } from "./job-run-log";

const RUN_HISTORY = 30;

interface JobDetailProps {
  readonly job: JobSummary;
  readonly system: JobSystemStatus | null;
  readonly busy: boolean;
  readonly onRunNow: (id: string) => Promise<void>;
  readonly onEdit: (job: JobSummary) => void;
  readonly onToggle: (job: JobSummary) => Promise<void>;
  readonly onDelete: (id: string) => Promise<void>;
  readonly onOpenSettings: () => void;
  readonly onError: (message: string | null) => void;
}

/** Remount with `key={job.id}` so runs and log never leak across jobs. */
export function JobDetail(props: JobDetailProps) {
  const { job } = props;
  const logs = usePaneCollapse();
  useRegisterLogs(logs.collapsed, logs.toggle, logs.expand);
  const { runs, loaded, selectedRunId, setSelectedRunId } = useRuns(job, props.onError);

  return (
    <section className="panel detail-panel">
      <DetailSplit
        logs={logs}
        inspector={
          <>
            <JobHeader {...props} />
            <JobNotes job={job} system={props.system} onOpenSettings={props.onOpenSettings} />
            <JobFacts job={job} />
            <RunHistory
              runs={runs}
              loaded={loaded}
              selectedRunId={selectedRunId}
              onSelect={setSelectedRunId}
            />
          </>
        }
        logsPane={
          <JobRunLog
            runs={runs}
            selectedRunId={selectedRunId}
            collapsed={logs.collapsed}
            onSelectRun={setSelectedRunId}
            onExpand={logs.expand}
          />
        }
      />
    </section>
  );
}

function useRegisterLogs(collapsed: boolean, toggle: () => void, expand: () => void): void {
  const { registerLogs } = usePaneControls();
  useEffect(() => {
    registerLogs({ collapsed, toggle, expand });
  }, [collapsed, expand, registerLogs, toggle]);
  useEffect(() => () => registerLogs(null), [registerLogs]);
}

function useRuns(job: JobSummary, onError: (message: string | null) => void) {
  const [runs, setRuns] = useState<readonly JobRun[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const lastRunKey = `${job.lastRun?.id ?? ""}:${job.lastRun?.status ?? ""}`;

  useEffect(() => {
    let cancelled = false;
    listJobRuns(job.id, RUN_HISTORY)
      .then((next) => {
        if (cancelled) {
          return;
        }
        setRuns(next);
        setLoaded(true);
        setSelectedRunId((current) =>
          current && next.some((run) => run.id === current) && next[0]?.status !== "running"
            ? current
            : (next[0]?.id ?? null),
        );
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          onError(formatInvokeError(err));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [job.id, lastRunKey, onError]);

  return { runs, loaded, selectedRunId, setSelectedRunId };
}

function JobHeader({ job, busy, onRunNow, onEdit, onToggle, onDelete }: JobDetailProps) {
  const [confirmDelete, setConfirmDelete] = useState(false);
  const running = job.lastRun?.status === "running";
  const tone = job.enabled ? statusTone(job.lastRun?.status ?? null) : "stopped";
  const badge = !job.enabled
    ? "Paused"
    : job.lastRun
      ? statusLabel(job.lastRun.status)
      : "Never run";

  return (
    <>
      <div className="detail-header">
        <div className="detail-header-text">
          <h2>{job.name}</h2>
          <p className="path" title={job.command}>
            {job.command}
          </p>
        </div>
        <span className={`badge badge-${tone}`}>{badge}</span>
      </div>
      <div className="action-row">
        <button type="button" className="primary" disabled={busy || running} onClick={() => void onRunNow(job.id)}>
          {running ? "Running…" : "Run now"}
        </button>
        <button type="button" disabled={busy} onClick={() => onEdit(job)}>
          Edit
        </button>
        <button type="button" disabled={busy} onClick={() => void onToggle(job)}>
          {job.enabled ? "Pause" : "Resume"}
        </button>
        {confirmDelete ? (
          <>
            <button type="button" className="danger" disabled={busy} onClick={() => void onDelete(job.id)}>
              Delete job and history
            </button>
            <button type="button" className="ghost" onClick={() => setConfirmDelete(false)}>
              Keep
            </button>
          </>
        ) : (
          <button type="button" className="danger" disabled={busy} onClick={() => setConfirmDelete(true)}>
            Delete
          </button>
        )}
      </div>
    </>
  );
}

interface JobNotesProps {
  readonly job: JobSummary;
  readonly system: JobSystemStatus | null;
  readonly onOpenSettings: () => void;
}

function JobNotes({ job, system, onOpenSettings }: JobNotesProps) {
  if (job.policy !== "exact" || !system) {
    return null;
  }
  if (!system.wakeHelperInstalled) {
    return (
      <div className="job-note warn" role="note">
        <span>
          The wake helper is not installed, so this job runs on the next wake
          instead of waking the Mac.
        </span>
        <button type="button" className="ghost" onClick={onOpenSettings}>
          Set up wake
        </button>
      </div>
    );
  }
  if (system.powerSource === "battery") {
    return (
      <p className="job-note" role="note">
        On battery with the lid closed, macOS can sleep through a scheduled
        wake. Plug in to run with the lid closed; otherwise it runs on the next
        wake.
      </p>
    );
  }
  return null;
}

function JobFacts({ job }: { readonly job: JobSummary }) {
  const next =
    job.enabled && job.nextFireUnix !== null
      ? `${formatDateTime(job.nextFireUnix)} (${formatRelative(job.nextFireUnix)})`
      : job.enabled
        ? "—"
        : "Paused";
  return (
    <dl className="status-grid">
      <Fact label="Schedule" value={describeSchedule(job.schedule)} />
      <Fact label="Next run" value={next} />
      <Fact label="Timing" value={describePolicy(job.policy)} />
      <Fact label="Network" value={describeNetwork(job.network, job.networkGraceSeconds)} />
      <Fact label="Max runtime" value={describeMaxRuntime(job.maxRuntimeSeconds)} />
      <Fact label="Working dir" value={job.workingDir ?? "Home folder"} mono />
    </dl>
  );
}

function Fact({ label, value, mono = false }: { readonly label: string; readonly value: string; readonly mono?: boolean }) {
  return (
    <div>
      <dt>{label}</dt>
      <dd className={mono ? "fact-mono" : undefined} title={value}>
        {value}
      </dd>
    </div>
  );
}

interface RunHistoryProps {
  readonly runs: readonly JobRun[];
  readonly loaded: boolean;
  readonly selectedRunId: string | null;
  readonly onSelect: (id: string) => void;
}

function RunHistory({ runs, loaded, selectedRunId, onSelect }: RunHistoryProps) {
  const count = !loaded ? "loading…" : runs.length === 0 ? "none yet" : `last ${runs.length}`;
  return (
    <section className="stack-panel run-history" aria-label="Run history">
      <div className="stack-header">
        <h3>Runs</h3>
        <span className="muted">{count}</span>
      </div>
      {!loaded ? (
        <p className="stack-empty muted">Loading run history…</p>
      ) : runs.length === 0 ? (
        <p className="stack-empty muted">No runs recorded. Scheduled and manual runs show up here.</p>
      ) : (
        <ul className="stack-list">
          {runs.map((run) => (
            <li key={run.id}>
              <button
                type="button"
                className={run.id === selectedRunId ? "run-row active" : "run-row"}
                onClick={() => onSelect(run.id)}
                title={run.message ?? undefined}
              >
                <span className={`status-dot status-${statusTone(run.status)}`} aria-hidden />
                <span className="run-row-when">{formatDateTime(run.startedAtUnix)}</span>
                <span className="run-row-meta">
                  {statusLabel(run.status)} · {triggerLabel(run.trigger)}
                  {runDuration(run) ? ` · ${runDuration(run)}` : ""}
                  {run.exitCode !== null && run.exitCode !== 0 ? ` · exit ${run.exitCode}` : ""}
                  {run.network === "offline" ? " · offline" : ""}
                </span>
              </button>
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
