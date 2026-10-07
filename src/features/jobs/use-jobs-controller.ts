import { useCallback, useState } from "react";

import {
  createJob,
  deleteJob,
  installWakeHelper,
  runJobNow,
  setJobEnabled,
  uninstallWakeHelper,
  updateJob,
} from "../../lib";
import type { JobInput, JobSummary } from "../../types";
import { useJobs, type JobsState } from "./use-jobs";

const AFTER_RUN_REFRESH_MS = 900;

export type EditorTarget = { readonly kind: "new" } | { readonly kind: "edit"; readonly job: JobSummary };

export interface JobsController {
  readonly state: JobsState;
  readonly editor: EditorTarget | null;
  readonly showSettings: boolean;
  readonly openNew: () => void;
  readonly openEdit: (job: JobSummary) => void;
  readonly closeEditor: () => void;
  readonly setShowSettings: (value: boolean) => void;
  readonly save: (input: JobInput) => Promise<void>;
  readonly runNow: (id: string) => Promise<void>;
  readonly toggle: (job: JobSummary) => Promise<void>;
  readonly remove: (id: string) => Promise<void>;
  readonly installHelper: () => Promise<void>;
  readonly removeHelper: () => Promise<void>;
}

type WithBusy = (action: () => Promise<void>) => Promise<void>;

export function useJobsController(
  active: boolean,
  withBusy: WithBusy,
  setBusy: (value: boolean) => void,
  onError: (message: string) => void,
): JobsController {
  const state = useJobs(active, onError);
  const [editor, setEditor] = useState<EditorTarget | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const { upsert, select, refreshJobs, refreshSystem } = state;

  /** Errors surface inside the editor, so this rethrows instead of banners. */
  const save = useCallback(
    async (input: JobInput): Promise<void> => {
      setBusy(true);
      try {
        const saved = editor?.kind === "edit" ? await updateJob(editor.job.id, input) : await createJob(input);
        upsert(saved);
        select(saved.id);
        setEditor(null);
      } finally {
        setBusy(false);
      }
    },
    [editor, select, setBusy, upsert],
  );

  const runNow = (id: string): Promise<void> =>
    withBusy(async () => {
      await runJobNow(id);
      window.setTimeout(() => void refreshJobs().catch(() => undefined), AFTER_RUN_REFRESH_MS);
    });

  const toggle = (job: JobSummary): Promise<void> =>
    withBusy(async () => upsert(await setJobEnabled(job.id, !job.enabled)));

  const remove = (id: string): Promise<void> =>
    withBusy(async () => {
      await deleteJob(id);
      await refreshJobs();
    });

  const installHelper = (): Promise<void> =>
    withBusy(async () => {
      try {
        await installWakeHelper();
      } finally {
        await refreshSystem();
      }
    });

  const removeHelper = (): Promise<void> =>
    withBusy(async () => {
      try {
        await uninstallWakeHelper();
      } finally {
        await refreshSystem();
      }
    });

  return {
    state,
    editor,
    showSettings,
    openNew: () => setEditor({ kind: "new" }),
    openEdit: (job) => setEditor({ kind: "edit", job }),
    closeEditor: () => setEditor(null),
    setShowSettings,
    save,
    runNow,
    toggle,
    remove,
    installHelper,
    removeHelper,
  };
}
