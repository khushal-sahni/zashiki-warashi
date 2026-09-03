import Anser from "anser";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import {
  clearProjectLogs,
  formatInvokeError,
  getProjectLogs,
  listenProjectLogs,
  projectHasCompose,
} from "../../lib";
import type { LogSource } from "../../types";

const MAX_LINES = 8_000;
const COMPOSE_POLL_MS = 1_000;

interface ProjectLogViewerProps {
  readonly projectId: string;
  readonly running: boolean;
  readonly collapsed?: boolean;
  readonly onExpand?: () => void;
}

export function ProjectLogViewer({
  projectId,
  running,
  collapsed = false,
  onExpand,
}: ProjectLogViewerProps) {
  const [source, setSource] = useState<LogSource>("process");
  const [hasCompose, setHasCompose] = useState(false);
  const [lines, setLines] = useState<string[]>([]);
  const [truncated, setTruncated] = useState(false);
  const [filter, setFilter] = useState("");
  const [follow, setFollow] = useState(true);
  const [wrap, setWrap] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const bodyRef = useRef<HTMLPreElement | null>(null);

  useEffect(() => {
    setSource("process");
    setFilter("");
    setFollow(true);
    setError(null);
  }, [projectId]);

  useEffect(() => {
    let cancelled = false;
    void projectHasCompose(projectId)
      .then((value) => {
        if (!cancelled) {
          setHasCompose(value);
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setHasCompose(false);
          setError(formatInvokeError(err));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [projectId]);

  const appendLines = useCallback((incoming: readonly string[]) => {
    setLines((current) => {
      const next = current.concat(incoming as string[]);
      if (next.length <= MAX_LINES) {
        return next;
      }
      return next.slice(next.length - MAX_LINES);
    });
  }, []);

  useProcessLogStream(
    projectId,
    source,
    running,
    setLines,
    setTruncated,
    setError,
    appendLines,
  );
  useComposeLogPoll(projectId, source, follow, setLines, setTruncated, setError);

  const filtered = useMemo(() => {
    const needle = filter.trim().toLowerCase();
    if (!needle) {
      return lines;
    }
    return lines.filter((line) => line.toLowerCase().includes(needle));
  }, [filter, lines]);

  useEffect(() => {
    if (!follow) {
      return;
    }
    const node = bodyRef.current;
    if (node) {
      node.scrollTop = node.scrollHeight;
    }
  }, [filtered, follow]);

  const html = useMemo(() => renderAnsiHtml(filtered), [filtered]);

  if (collapsed) {
    return (
      <button
        type="button"
        className="log-viewer log-collapse-strip"
        title="Show logs (⌘J)"
        onClick={onExpand}
      >
        <span className="log-sources" aria-hidden>
          <span className={source === "process" ? "ghost active" : "ghost"}>
            Process
          </span>
          {hasCompose ? (
            <span className={source === "compose" ? "ghost active" : "ghost"}>
              Compose
            </span>
          ) : null}
        </span>
        <span className="log-meta">
          <span
            className={["log-pip", running ? "live" : ""].filter(Boolean).join(" ")}
            aria-hidden
          />
          {filtered.length}
          {truncated ? "+" : ""} lines
        </span>
        <span className="muted">Expand logs</span>
      </button>
    );
  }

  return (
    <div className="log-viewer">
      <LogToolbar
        source={source}
        hasCompose={hasCompose}
        follow={follow}
        wrap={wrap}
        running={running}
        filter={filter}
        lineCount={filtered.length}
        truncated={truncated}
        canClear={source === "process"}
        onSource={setSource}
        onFilter={setFilter}
        onFollow={() => setFollow((value) => !value)}
        onWrap={() => setWrap((value) => !value)}
        onCopy={() => {
          void copyLines(filtered).catch((err: unknown) => {
            setError(formatInvokeError(err));
          });
        }}
        onClear={() => {
          void clearLogs(projectId, setLines, setTruncated, setError);
        }}
      />
      {error && (
        <p className="log-error" role="alert">
          {error}
        </p>
      )}
      <pre
        ref={bodyRef}
        className={["log-body", wrap ? "wrap" : ""].filter(Boolean).join(" ")}
        onScroll={(event) => {
          const node = event.currentTarget;
          const atBottom =
            node.scrollHeight - node.scrollTop - node.clientHeight < 28;
          if (!atBottom && follow) {
            setFollow(false);
          }
        }}
      >
        {filtered.length === 0 ? (
          <span className="log-empty">{emptyMessage(source, running)}</span>
        ) : (
          <code dangerouslySetInnerHTML={{ __html: html }} />
        )}
      </pre>
    </div>
  );
}

interface LogToolbarProps {
  readonly source: LogSource;
  readonly hasCompose: boolean;
  readonly follow: boolean;
  readonly wrap: boolean;
  readonly running: boolean;
  readonly filter: string;
  readonly lineCount: number;
  readonly truncated: boolean;
  readonly canClear: boolean;
  readonly onSource: (source: LogSource) => void;
  readonly onFilter: (value: string) => void;
  readonly onFollow: () => void;
  readonly onWrap: () => void;
  readonly onCopy: () => void;
  readonly onClear: () => void;
}

function LogToolbar(props: LogToolbarProps) {
  return (
    <div className="log-toolbar">
      <div className="log-sources" role="tablist" aria-label="Log source">
        <button
          type="button"
          role="tab"
          aria-selected={props.source === "process"}
          className={props.source === "process" ? "ghost active" : "ghost"}
          onClick={() => props.onSource("process")}
        >
          Process
        </button>
        {props.hasCompose && (
          <button
            type="button"
            role="tab"
            aria-selected={props.source === "compose"}
            className={props.source === "compose" ? "ghost active" : "ghost"}
            onClick={() => props.onSource("compose")}
          >
            Compose
          </button>
        )}
      </div>
      <label className="log-filter">
        <span className="sr-only">Filter logs</span>
        <input
          value={props.filter}
          onChange={(event) => props.onFilter(event.target.value)}
          placeholder="Filter"
          spellCheck={false}
        />
      </label>
      <span className="log-meta">
        <span
          className={["log-pip", props.running ? "live" : ""].filter(Boolean).join(" ")}
          aria-hidden
        />
        {props.lineCount}
        {props.truncated ? "+" : ""} lines
      </span>
      <button
        type="button"
        className={["ghost", "log-chip", props.follow ? "active" : ""].filter(Boolean).join(" ")}
        onClick={props.onFollow}
      >
        Follow
      </button>
      <button
        type="button"
        className={["ghost", "log-chip", props.wrap ? "active" : ""].filter(Boolean).join(" ")}
        onClick={props.onWrap}
      >
        Wrap
      </button>
      <button type="button" className="ghost log-chip" onClick={props.onCopy}>
        Copy
      </button>
      {props.canClear && (
        <button type="button" className="ghost log-chip" onClick={props.onClear}>
          Clear
        </button>
      )}
    </div>
  );
}

function useProcessLogStream(
  projectId: string,
  source: LogSource,
  running: boolean,
  setLines: (lines: string[]) => void,
  setTruncated: (value: boolean) => void,
  setError: (value: string | null) => void,
  appendLines: (incoming: readonly string[]) => void,
): void {
  useEffect(() => {
    if (source !== "process") {
      return;
    }
    let cancelled = false;
    void getProjectLogs(projectId, "process")
      .then((chunk) => {
        if (cancelled) {
          return;
        }
        setLines([...chunk.lines]);
        setTruncated(chunk.truncated);
        setError(null);
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(formatInvokeError(err));
        }
      });
    const unlisten = listenProjectLogs(projectId, (incoming) => {
      if (!cancelled) {
        appendLines(incoming);
      }
    });
    return () => {
      cancelled = true;
      void unlisten.then((stop) => stop());
    };
  }, [appendLines, projectId, running, setError, setLines, setTruncated, source]);
}

function useComposeLogPoll(
  projectId: string,
  source: LogSource,
  follow: boolean,
  setLines: (lines: string[]) => void,
  setTruncated: (value: boolean) => void,
  setError: (value: string | null) => void,
): void {
  useEffect(() => {
    if (source !== "compose") {
      return;
    }
    let cancelled = false;
    async function pull(): Promise<void> {
      try {
        const chunk = await getProjectLogs(projectId, "compose");
        if (cancelled) {
          return;
        }
        setLines([...chunk.lines]);
        setTruncated(chunk.truncated);
        setError(null);
      } catch (err) {
        if (!cancelled) {
          setError(formatInvokeError(err));
        }
      }
    }
    void pull();
    if (!follow) {
      return () => {
        cancelled = true;
      };
    }
    const timer = window.setInterval(() => {
      void pull();
    }, COMPOSE_POLL_MS);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [follow, projectId, setError, setLines, setTruncated, source]);
}

function renderAnsiHtml(lines: readonly string[]): string {
  return lines
    .map((line) => Anser.ansiToHtml(Anser.escapeForHtml(line)))
    .join("\n");
}

function emptyMessage(source: LogSource, running: boolean): string {
  if (source === "compose") {
    return "No compose output yet.";
  }
  if (running) {
    return "Waiting for process output…";
  }
  return "Start the project to capture output here.";
}

async function copyLines(lines: readonly string[]): Promise<void> {
  await navigator.clipboard.writeText(lines.join("\n"));
}

async function clearLogs(
  projectId: string,
  setLines: (lines: string[]) => void,
  setTruncated: (value: boolean) => void,
  setError: (value: string | null) => void,
): Promise<void> {
  try {
    await clearProjectLogs(projectId);
    setLines([]);
    setTruncated(false);
    setError(null);
  } catch (err) {
    setError(formatInvokeError(err));
  }
}
