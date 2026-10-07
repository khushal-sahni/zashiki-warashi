import { useState } from "react";

import type { ForeignAgent, JobSummary } from "../../types";
import { describeSchedule, formatRelative, statusTone } from "./format";

interface JobListProps {
  readonly jobs: readonly JobSummary[];
  readonly loaded: boolean;
  readonly selectedId: string | null;
  readonly agents: readonly ForeignAgent[] | null;
  readonly onSelect: (id: string) => void;
}

export function JobList({ jobs, loaded, selectedId, agents, onSelect }: JobListProps) {
  const [query, setQuery] = useState("");
  const needle = query.trim().toLowerCase();
  const filtered = jobs.filter(
    (job) =>
      !needle ||
      job.name.toLowerCase().includes(needle) ||
      job.command.toLowerCase().includes(needle),
  );

  return (
    <aside className="sidebar">
      <label className="search">
        <span className="sr-only">Search jobs</span>
        <input
          value={query}
          onChange={(event) => setQuery(event.target.value)}
          placeholder="Search jobs…"
        />
      </label>
      <ul className="project-list job-list">
        {!loaded && <li className="empty-item">Loading jobs…</li>}
        {loaded && jobs.length === 0 && (
          <li className="empty-item">No scheduled jobs yet.</li>
        )}
        {loaded && jobs.length > 0 && filtered.length === 0 && (
          <li className="empty-item">No jobs match.</li>
        )}
        {filtered.map((job) => (
          <li key={job.id}>
            <JobListItem job={job} active={job.id === selectedId} onSelect={onSelect} />
          </li>
        ))}
      </ul>
      <ForeignAgents agents={agents} />
    </aside>
  );
}

interface JobListItemProps {
  readonly job: JobSummary;
  readonly active: boolean;
  readonly onSelect: (id: string) => void;
}

function JobListItem({ job, active, onSelect }: JobListItemProps) {
  const tone = statusTone(job.lastRun?.status ?? null);
  const next = job.enabled
    ? job.nextFireUnix !== null
      ? `next ${formatRelative(job.nextFireUnix)}`
      : "no upcoming run"
    : "paused";
  const timing = job.enabled && job.policy === "exact" ? " · wakes Mac" : "";
  return (
    <button
      type="button"
      className={active ? "project-item active" : "project-item"}
      onClick={() => onSelect(job.id)}
    >
      <span className={`status-dot status-${tone}`} title={job.lastRun?.status ?? "never run"} />
      <span className="project-item-text">
        <span className="project-item-name" title={job.name}>
          {job.name}
        </span>
        <span className="job-item-meta" title={`${describeSchedule(job.schedule)} · ${next}${timing}`}>
          {describeSchedule(job.schedule)} · {next}
          {timing}
        </span>
      </span>
    </button>
  );
}

function ForeignAgents({ agents }: { readonly agents: readonly ForeignAgent[] | null }) {
  return (
    <details className="foreign-agents">
      <summary>
        Other launch agents
        <span className="muted"> · {agents === null ? "…" : agents.length}</span>
      </summary>
      <p className="foreign-agents-note muted">
        Read-only. These are scheduled by other tools. Recreate one as a job to
        get wake, network wait, and run history.
      </p>
      <ul className="foreign-agent-list">
        {agents !== null && agents.length === 0 && (
          <li className="empty-item">None in ~/Library/LaunchAgents.</li>
        )}
        {agents?.map((agent) => (
          <li key={agent.label} className="foreign-agent" title={agent.plistPath}>
            <span className="foreign-agent-label">{agent.label}</span>
            <span className="foreign-agent-meta">
              {agent.scheduleHint ?? "on demand"} ·{" "}
              {agent.pid !== null
                ? `running (pid ${agent.pid})`
                : agent.loaded
                  ? `last exit ${agent.lastExit ?? "—"}`
                  : "not loaded"}
            </span>
            {agent.program && (
              <code className="foreign-agent-program" title={agent.program}>
                {agent.program}
              </code>
            )}
          </li>
        ))}
      </ul>
    </details>
  );
}
