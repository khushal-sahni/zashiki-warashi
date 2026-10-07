import { useCallback, useEffect, useMemo, useState } from "react";

import {
  formatInvokeError,
  getJobSystemStatus,
  listForeignAgents,
  listJobs,
} from "../../lib";
import type { ForeignAgent, JobSummary, JobSystemStatus } from "../../types";

const JOBS_POLL_MS = 4_000;
const SYSTEM_POLL_MS = 30_000;

export interface JobsState {
  readonly jobs: readonly JobSummary[];
  readonly loaded: boolean;
  readonly selected: JobSummary | null;
  readonly selectedId: string | null;
  readonly system: JobSystemStatus | null;
  readonly agents: readonly ForeignAgent[] | null;
  readonly select: (id: string) => void;
  readonly upsert: (job: JobSummary) => void;
  readonly refreshJobs: () => Promise<void>;
  readonly refreshSystem: () => Promise<void>;
}

/**
 * Job catalog for the Jobs mode. SQLite-backed list polls quickly; anything
 * that spawns pmset / launchctl polls slowly in the background.
 */
export function useJobs(active: boolean, onError: (message: string) => void): JobsState {
  const [jobs, setJobs] = useState<readonly JobSummary[]>([]);
  const [loaded, setLoaded] = useState(false);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [system, setSystem] = useState<JobSystemStatus | null>(null);
  const [agents, setAgents] = useState<readonly ForeignAgent[] | null>(null);

  const refreshJobs = useCallback(async (): Promise<void> => {
    const next = await listJobs();
    setJobs(next);
    setLoaded(true);
    setSelectedId((current) =>
      current && next.some((job) => job.id === current) ? current : (next[0]?.id ?? null),
    );
  }, []);

  const refreshSystem = useCallback(async (): Promise<void> => {
    const [nextSystem, nextAgents] = await Promise.all([
      getJobSystemStatus(),
      listForeignAgents(),
    ]);
    setSystem(nextSystem);
    setAgents(nextAgents);
  }, []);

  usePoll(active, JOBS_POLL_MS, refreshJobs, onError);
  usePoll(active, SYSTEM_POLL_MS, refreshSystem, onError);

  const upsert = useCallback((job: JobSummary) => {
    setJobs((current) => {
      const exists = current.some((item) => item.id === job.id);
      const next = exists
        ? current.map((item) => (item.id === job.id ? job : item))
        : [...current, job];
      return [...next].sort((a, b) => a.name.localeCompare(b.name));
    });
  }, []);

  const selected = useMemo(
    () => jobs.find((job) => job.id === selectedId) ?? null,
    [jobs, selectedId],
  );

  return {
    jobs,
    loaded,
    selected,
    selectedId,
    system,
    agents,
    select: setSelectedId,
    upsert,
    refreshJobs,
    refreshSystem,
  };
}

function usePoll(
  active: boolean,
  intervalMs: number,
  task: () => Promise<void>,
  onError: (message: string) => void,
): void {
  useEffect(() => {
    if (!active) {
      return;
    }
    let cancelled = false;
    const run = (): void => {
      task().catch((err: unknown) => {
        if (!cancelled) {
          onError(formatInvokeError(err));
        }
      });
    };
    run();
    const timer = window.setInterval(run, intervalMs);
    return () => {
      cancelled = true;
      window.clearInterval(timer);
    };
  }, [active, intervalMs, onError, task]);
}
