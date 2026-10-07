import Anser from "anser";
import { useEffect, useMemo, useRef, useState } from "react";

import { formatInvokeError, getJobRunLog } from "../../lib";
import type { JobRun } from "../../types";
import { formatDateTime, statusLabel, triggerLabel } from "./format";

const LOG_POLL_MS = 1_200;
const LOG_LINES = 2_000;
const ANSI_PAINT_LINES = 600;

interface JobRunLogProps {
  readonly runs: readonly JobRun[];
  readonly selectedRunId: string | null;
  readonly collapsed: boolean;
  readonly onSelectRun: (id: string) => void;
  readonly onExpand: () => void;
}

export function JobRunLog({ runs, selectedRunId, collapsed, onSelectRun, onExpand }: JobRunLogProps) {
  const run = runs.find((item) => item.id === selectedRunId) ?? null;
  const live = run?.status === "running";
  const { lines, error, setError } = useRunLog(run?.id ?? null, live);
  const [wrap, setWrap] = useState(true);
  const [follow, setFollow] = useState(true);
  const bodyRef = useRef<HTMLPreElement | null>(null);

  useEffect(() => {
    if (follow && bodyRef.current) {
      bodyRef.current.scrollTop = bodyRef.current.scrollHeight;
    }
  }, [follow, lines]);

  const html = useMemo(
    () =>
      lines
        .slice(-ANSI_PAINT_LINES)
        .map((line) => Anser.ansiToHtml(Anser.escapeForHtml(line)))
        .join("\n"),
    [lines],
  );

  if (collapsed) {
    return (
      <button type="button" className="log-viewer log-collapse-strip" title="Show run log (⌘J)" onClick={onExpand}>
        <span className="log-meta">
          <span className={live ? "log-pip live" : "log-pip"} aria-hidden />
          {run ? `${triggerLabel(run.trigger)} · ${statusLabel(run.status)}` : "No runs yet"}
        </span>
        <span className="muted">Expand run log</span>
      </button>
    );
  }

  return (
    <div className="log-viewer">
      <div className="log-toolbar">
        <label className="log-filter run-picker">
          <span className="sr-only">Run</span>
          <select
            value={run?.id ?? ""}
            disabled={runs.length === 0}
            onChange={(event) => onSelectRun(event.target.value)}
          >
            {runs.length === 0 && <option value="">No runs yet</option>}
            {runs.map((item) => (
              <option key={item.id} value={item.id}>
                {formatDateTime(item.startedAtUnix)} · {triggerLabel(item.trigger)} · {statusLabel(item.status)}
              </option>
            ))}
          </select>
        </label>
        <span className="log-meta">
          <span className={live ? "log-pip live" : "log-pip"} aria-hidden />
          {lines.length} lines
        </span>
        <button type="button" className={follow ? "ghost log-chip active" : "ghost log-chip"} onClick={() => setFollow((v) => !v)}>
          Follow
        </button>
        <button type="button" className={wrap ? "ghost log-chip active" : "ghost log-chip"} onClick={() => setWrap((v) => !v)}>
          Wrap
        </button>
        <button
          type="button"
          className="ghost log-chip"
          disabled={lines.length === 0}
          onClick={() => {
            navigator.clipboard.writeText(lines.join("\n")).catch((err: unknown) => {
              setError(formatInvokeError(err));
            });
          }}
        >
          Copy
        </button>
      </div>
      {error && (
        <p className="log-error" role="alert">
          {error}
        </p>
      )}
      <pre
        ref={bodyRef}
        className={wrap ? "log-body wrap" : "log-body"}
        onScroll={(event) => {
          const node = event.currentTarget;
          if (node.scrollHeight - node.scrollTop - node.clientHeight >= 28 && follow) {
            setFollow(false);
          }
        }}
      >
        {lines.length === 0 ? (
          <span className="log-empty">{emptyMessage(run)}</span>
        ) : (
          <code dangerouslySetInnerHTML={{ __html: html }} />
        )}
      </pre>
    </div>
  );
}

function emptyMessage(run: JobRun | null): string {
  if (!run) {
    return "Runs appear here with their output. Use Run now to try the job.";
  }
  return run.status === "running" ? "Waiting for output…" : "This run produced no output.";
}

function useRunLog(runId: string | null, live: boolean) {
  const [lines, setLines] = useState<readonly string[]>([]);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setLines([]);
    setError(null);
  }, [runId]);

  useEffect(() => {
    if (!runId) {
      return;
    }
    let cancelled = false;
    const pull = (): void => {
      getJobRunLog(runId, LOG_LINES)
        .then((tail) => {
          if (!cancelled) {
            setLines(tail.lines);
            setError(null);
          }
        })
        .catch((err: unknown) => {
          if (!cancelled) {
            setError(formatInvokeError(err));
          }
        });
    };
    pull();
    if (!live) {
      return () => {
        cancelled = true;
      };
    }
    const timer = window.setInterval(pull, LOG_POLL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [live, runId]);

  return { lines, error, setError };
}
