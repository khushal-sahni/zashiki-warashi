import { useEffect, useState } from "react";

import type { Project } from "../../types";

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
}

export function ProjectDetail({
  project,
  busy,
  onStart,
  onStop,
  onRestart,
  onRemove,
  onSaveCommands,
}: ProjectDetailProps) {
  const [startCommand, setStartCommand] = useState("");
  const [stopCommand, setStopCommand] = useState("");

  useEffect(() => {
    if (!project) {
      setStartCommand("");
      setStopCommand("");
      return;
    }
    setStartCommand(project.startCommand ?? "");
    setStopCommand(project.stopCommand ?? "");
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
            placeholder={project.inferredStartCommand ?? "e.g. npm run dev"}
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
    </section>
  );
}
