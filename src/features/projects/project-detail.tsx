import { useCallback, useEffect, useState } from "react";

import {
  DetailSplit,
  usePaneCollapse,
  usePaneControls,
} from "../../components";
import { StackPanel } from "../docker";
import type { Project } from "../../types";
import { ProjectLogViewer } from "./project-log-viewer";

interface ProjectDetailProps {
  readonly project: Project | null;
  readonly busy: boolean;
  readonly onStart: (id: string) => Promise<void>;
  readonly onStop: (id: string) => Promise<void>;
  readonly onRestart: (id: string) => Promise<void>;
  readonly onRemove: (id: string) => Promise<void>;
  readonly onSaveCommands: (
    id: string,
    startCommand: string,
    stopCommand: string,
  ) => Promise<void>;
  readonly onError: (message: string | null) => void;
}

export function ProjectDetail({
  project,
  busy,
  onStart,
  onStop,
  onRestart,
  onRemove,
  onSaveCommands,
  onError,
}: ProjectDetailProps) {
  const [startCommand, setStartCommand] = useState("");
  const [stopCommand, setStopCommand] = useState("");
  const [hasCompose, setHasCompose] = useState(false);
  const logs = usePaneCollapse();
  const { registerLogs } = usePaneControls();

  const onComposePresence = useCallback((value: boolean) => {
    setHasCompose(value);
  }, []);

  useEffect(() => {
    if (!project) {
      registerLogs(null);
      return;
    }
    registerLogs({
      collapsed: logs.collapsed,
      toggle: logs.toggle,
      expand: logs.expand,
    });
  }, [logs.collapsed, logs.expand, logs.toggle, project, registerLogs]);

  useEffect(() => {
    return () => {
      registerLogs(null);
    };
  }, [registerLogs]);

  useEffect(() => {
    if (!project) {
      setStartCommand("");
      setStopCommand("");
      setHasCompose(false);
      return;
    }
    setStartCommand(project.startCommand ?? "");
    setStopCommand(project.stopCommand ?? "");
    setHasCompose(false);
  }, [project]);

  if (!project) {
    return (
      <section className="panel detail-panel">
        <h2>Select a project</h2>
        <p className="muted">
          Add a folder or scan your roots to start managing local projects.
        </p>
      </section>
    );
  }

  const effectiveStart =
    project.startCommand?.trim() ||
    project.inferredStartCommand ||
    "(none — set an override)";

  const canStart =
    Boolean(project.startCommand?.trim() || project.inferredStartCommand) &&
    !busy;

  return (
    <section className="panel detail-panel">
      <DetailSplit
        logs={logs}
        inspector={
          <>
            <div className="detail-header">
              <div>
                <h2>{project.name}</h2>
                <p className="path">{project.path}</p>
              </div>
              <span className={`badge badge-${project.run.status}`}>
                {project.run.status}
              </span>
            </div>

            <div className="action-row">
              <button
                type="button"
                disabled={!canStart || project.run.status === "running"}
                onClick={() => void onStart(project.id)}
              >
                Start
              </button>
              <button
                type="button"
                disabled={busy || project.run.status === "stopped"}
                onClick={() => void onStop(project.id)}
              >
                Stop
              </button>
              <button
                type="button"
                disabled={!canStart}
                onClick={() => void onRestart(project.id)}
              >
                Restart
              </button>
              <button
                type="button"
                className="danger"
                disabled={busy}
                onClick={() => void onRemove(project.id)}
              >
                Remove
              </button>
            </div>

            {project.run.lastError && (
              <p className="error" role="alert">
                {project.run.lastError}
              </p>
            )}

            <dl className="status-grid">
              <div>
                <dt>Effective start</dt>
                <dd>
                  <code>{effectiveStart}</code>
                </dd>
              </div>
              <div>
                <dt>Inferred</dt>
                <dd>
                  <code>{project.inferredStartCommand ?? "—"}</code>
                </dd>
              </div>
              <div>
                <dt>PID</dt>
                <dd>{project.run.pid ?? "—"}</dd>
              </div>
            </dl>

            <details className="command-details">
              <summary>Commands</summary>
              <form
                className="command-form"
                onSubmit={(event) => {
                  event.preventDefault();
                  void onSaveCommands(project.id, startCommand, stopCommand);
                }}
              >
                <label>
                  Start command override
                  <input
                    value={startCommand}
                    onChange={(event) => setStartCommand(event.target.value)}
                    placeholder={
                      project.inferredStartCommand ?? "e.g. npm run dev"
                    }
                  />
                </label>
                <label>
                  Stop command override
                  <input
                    value={stopCommand}
                    onChange={(event) => setStopCommand(event.target.value)}
                    placeholder="optional — default is SIGTERM to process group"
                  />
                </label>
                <button type="submit" disabled={busy}>
                  Save commands
                </button>
              </form>
            </details>

            <StackPanel
              key={project.id}
              projectId={project.id}
              busy={busy}
              onBusyError={onError}
              onComposePresence={onComposePresence}
            />
          </>
        }
        logsPane={
          <ProjectLogViewer
            key={project.id}
            projectId={project.id}
            running={project.run.status === "running"}
            hasCompose={hasCompose}
            collapsed={logs.collapsed}
            onExpand={logs.expand}
          />
        }
      />
    </section>
  );
}
