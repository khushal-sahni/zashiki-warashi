import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useMemo, useState } from "react";

import {
  KeepAwakeToggle,
  PaneControlsProvider,
  ShellHeader,
  SidebarRail,
  usePaneCollapse,
  usePaneControls,
  WorkspaceLayout,
} from "./components";
import { PortConflictDialog } from "./features/docker";
import {
  ProjectDetail,
  ProjectList,
  ScanResults,
  SettingsPanel,
} from "./features/projects";
import {
  addProject,
  formatInvokeError,
  getAppStatus,
  getKeepAwakeStatus,
  getSettings,
  listProjects,
  parsePortConflict,
  removeProject,
  resolvePortConflict,
  restartProject,
  scanProjects,
  setKeepAwakeEnabled,
  setScanRoots,
  startProject,
  stopProject,
  updateProjectCommands,
} from "./lib";
import type {
  AppStatus,
  KeepAwakeStatus,
  PortConflict,
  Project,
  ScanCandidate,
} from "./types";
import "./App.css";

function App() {
  const [projects, setProjects] = useState<Project[]>([]);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [query, setQuery] = useState("");
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [status, setStatus] = useState<AppStatus | null>(null);
  const [scanRoots, setScanRootsState] = useState<string[]>([]);
  const [candidates, setCandidates] = useState<ScanCandidate[] | null>(null);
  const [showSettings, setShowSettings] = useState(false);
  const [keepAwake, setKeepAwake] = useState<KeepAwakeStatus | null>(null);
  const [portConflict, setPortConflict] = useState<PortConflict | null>(null);
  const sidebar = usePaneCollapse();

  const selected = useMemo(
    () => projects.find((project) => project.id === selectedId) ?? null,
    [projects, selectedId],
  );

  const refresh = useCallback(async (): Promise<void> => {
    const [nextProjects, nextStatus, settings, nextKeepAwake] = await Promise.all([
      listProjects(),
      getAppStatus(),
      getSettings(),
      getKeepAwakeStatus(),
    ]);
    setProjects(nextProjects);
    setStatus(nextStatus);
    setScanRootsState(settings.scanRoots);
    setKeepAwake(nextKeepAwake);
    setSelectedId((current) => {
      if (current && nextProjects.some((project) => project.id === current)) {
        return current;
      }
      return nextProjects[0]?.id ?? null;
    });
  }, []);

  useEffect(() => {
    let cancelled = false;

    async function boot(): Promise<void> {
      try {
        await refresh();
      } catch (err) {
        if (!cancelled) {
          setError(formatInvokeError(err));
        }
      }
    }

    void boot();
    return () => {
      cancelled = true;
    };
  }, [refresh]);

  async function withBusy(action: () => Promise<void>): Promise<void> {
    setBusy(true);
    setError(null);
    try {
      await action();
    } catch (err) {
      setError(formatInvokeError(err));
    } finally {
      setBusy(false);
    }
  }

  async function handleAddFolder(): Promise<void> {
    await withBusy(async () => {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: "Add project folder",
      });
      if (typeof selectedPath !== "string" || selectedPath.length === 0) {
        return;
      }
      const project = await addProject(selectedPath);
      await refresh();
      setSelectedId(project.id);
    });
  }

  async function handleToggleKeepAwake(): Promise<void> {
    const nextEnabled = !(keepAwake?.enabled ?? false);
    setBusy(true);
    setError(null);
    try {
      const status = await setKeepAwakeEnabled(nextEnabled);
      setKeepAwake(status);
    } catch (err) {
      setError(formatInvokeError(err));
      try {
        setKeepAwake(await getKeepAwakeStatus());
      } catch {
        // Keep the last known status if refresh fails.
      }
    } finally {
      setBusy(false);
    }
  }

  async function handleStart(id: string): Promise<void> {
    setBusy(true);
    setError(null);
    setPortConflict(null);
    try {
      const project = await startProject(id);
      setProjects((current) =>
        current.map((item) => (item.id === id ? project : item)),
      );
    } catch (err) {
      const conflict = parsePortConflict(err);
      if (conflict) {
        setPortConflict(conflict);
      } else {
        setError(formatInvokeError(err));
      }
    } finally {
      setBusy(false);
    }
  }

  async function handleResolve(
    action: "stopOccupant" | "remap",
    writeToRepo: boolean,
    confirmNative: boolean,
  ): Promise<void> {
    if (!portConflict) {
      return;
    }
    const projectId = portConflict.projectId;
    const conflict = portConflict;
    setBusy(true);
    setError(null);
    try {
      await resolvePortConflict(
        projectId,
        action,
        writeToRepo,
        confirmNative,
        conflict,
      );
      setPortConflict(null);
      const project = await startProject(projectId);
      setProjects((current) =>
        current.map((item) => (item.id === projectId ? project : item)),
      );
    } catch (err) {
      const next = parsePortConflict(err);
      if (next) {
        setPortConflict(next);
      } else {
        setError(formatInvokeError(err));
        setPortConflict(null);
      }
    } finally {
      setBusy(false);
    }
  }

  async function handleScan(): Promise<void> {
    await withBusy(async () => {
      const results = await scanProjects();
      setCandidates(results);
      setShowSettings(false);
    });
  }

  return (
    <PaneControlsProvider sidebar={sidebar}>
      <AppShell
        projects={projects}
        selected={selected}
        selectedId={selectedId}
        query={query}
        busy={busy}
        error={error}
        status={status}
        scanRoots={scanRoots}
        candidates={candidates}
        showSettings={showSettings}
        keepAwake={keepAwake}
        portConflict={portConflict}
        onQueryChange={setQuery}
        onSelect={setSelectedId}
        onError={setError}
        onDismissPortConflict={() => setPortConflict(null)}
        onAddFolder={() => void handleAddFolder()}
        onScan={() => void handleScan()}
        onToggleKeepAwake={() => void handleToggleKeepAwake()}
        onToggleSettings={() => {
          setShowSettings((value) => !value);
          setCandidates(null);
        }}
        onCloseSettings={() => setShowSettings(false)}
        onCloseScan={() => setCandidates(null)}
        onRefresh={() => void withBusy(refresh)}
        onSaveRoots={async (roots) => {
          await withBusy(async () => {
            const settings = await setScanRoots(roots);
            setScanRootsState(settings.scanRoots);
            setShowSettings(false);
          });
        }}
        onAddCandidate={async (path) => {
          await withBusy(async () => {
            const project = await addProject(path);
            const results = await scanProjects();
            setCandidates(results);
            await refresh();
            setSelectedId(project.id);
          });
        }}
        onResolve={(action, writeToRepo, confirmNative) => {
          void handleResolve(action, writeToRepo, confirmNative);
        }}
        onStart={async (id) => {
          await handleStart(id);
        }}
        onStop={async (id) => {
          await withBusy(async () => {
            const project = await stopProject(id);
            setProjects((current) =>
              current.map((item) => (item.id === id ? project : item)),
            );
          });
        }}
        onRestart={async (id) => {
          setBusy(true);
          setError(null);
          setPortConflict(null);
          try {
            const project = await restartProject(id);
            setProjects((current) =>
              current.map((item) => (item.id === id ? project : item)),
            );
          } catch (err) {
            const conflict = parsePortConflict(err);
            if (conflict) {
              setPortConflict(conflict);
            } else {
              setError(formatInvokeError(err));
            }
          } finally {
            setBusy(false);
          }
        }}
        onRemove={async (id) => {
          await withBusy(async () => {
            await removeProject(id);
            await refresh();
          });
        }}
        onSaveCommands={async (id, startCommand, stopCommand) => {
          await withBusy(async () => {
            const project = await updateProjectCommands(
              id,
              startCommand,
              stopCommand,
            );
            setProjects((current) =>
              current.map((item) => (item.id === id ? project : item)),
            );
          });
        }}
      />
    </PaneControlsProvider>
  );
}

interface AppShellProps {
  readonly projects: readonly Project[];
  readonly selected: Project | null;
  readonly selectedId: string | null;
  readonly query: string;
  readonly busy: boolean;
  readonly error: string | null;
  readonly status: AppStatus | null;
  readonly scanRoots: readonly string[];
  readonly candidates: ScanCandidate[] | null;
  readonly showSettings: boolean;
  readonly keepAwake: KeepAwakeStatus | null;
  readonly portConflict: PortConflict | null;
  readonly onQueryChange: (query: string) => void;
  readonly onSelect: (id: string) => void;
  readonly onError: (message: string | null) => void;
  readonly onDismissPortConflict: () => void;
  readonly onAddFolder: () => void;
  readonly onScan: () => void;
  readonly onToggleKeepAwake: () => void;
  readonly onToggleSettings: () => void;
  readonly onCloseSettings: () => void;
  readonly onCloseScan: () => void;
  readonly onRefresh: () => void;
  readonly onSaveRoots: (roots: string[]) => Promise<void>;
  readonly onAddCandidate: (path: string) => Promise<void>;
  readonly onResolve: (
    action: "stopOccupant" | "remap",
    writeToRepo: boolean,
    confirmNative: boolean,
  ) => void;
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

function AppShell(props: AppShellProps) {
  const { sidebar, logsCollapsed, toggleLogs } = usePaneControls();

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent): void {
      if (!(event.metaKey || event.ctrlKey) || event.altKey || event.shiftKey) {
        return;
      }
      if (isEditableTarget(event.target)) {
        return;
      }
      const key = event.key.toLowerCase();
      if (key === "b") {
        event.preventDefault();
        sidebar.toggle();
        return;
      }
      if (key === "j") {
        event.preventDefault();
        toggleLogs();
      }
    }

    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [sidebar, toggleLogs]);

  return (
    <main className="shell app-shell">
      <div className="titlebar">
        <ShellHeader title="Zashiki Warashi" subtitle="LOCALHOST CONTROL PLANE" />
        <div className="toolbar">
          <button type="button" disabled={props.busy} onClick={props.onAddFolder}>
            Add folder
          </button>
          <button type="button" disabled={props.busy} onClick={props.onScan}>
            Scan
          </button>
          <KeepAwakeToggle
            status={props.keepAwake}
            busy={props.busy}
            onToggle={props.onToggleKeepAwake}
          />
          <button
            type="button"
            className={sidebar.collapsed ? "ghost" : "ghost active"}
            title="Toggle sidebar (⌘B)"
            onClick={sidebar.toggle}
          >
            Sidebar
          </button>
          <button
            type="button"
            className={logsCollapsed ? "ghost" : "ghost active"}
            title="Toggle logs (⌘J)"
            onClick={toggleLogs}
          >
            Logs
          </button>
          <button
            type="button"
            className={props.showSettings ? "ghost active" : "ghost"}
            disabled={props.busy}
            onClick={props.onToggleSettings}
          >
            Settings
          </button>
          <button
            type="button"
            className="ghost"
            disabled={props.busy}
            onClick={props.onRefresh}
          >
            Refresh
          </button>
        </div>
      </div>

      {props.portConflict && (
        <PortConflictDialog
          conflict={props.portConflict}
          busy={props.busy}
          onDismiss={props.onDismissPortConflict}
          onStopOccupant={(confirmNative) => {
            props.onResolve("stopOccupant", false, confirmNative);
          }}
          onRemap={(writeToRepo) => {
            props.onResolve("remap", writeToRepo, false);
          }}
        />
      )}

      {props.error && (
        <p className="error banner" role="alert">
          {props.error}
        </p>
      )}

      {props.showSettings && (
        <div className="modal-backdrop" role="presentation">
          <div
            className="modal-card overlay-panel"
            role="dialog"
            aria-label="Scan roots"
          >
            <SettingsPanel
              roots={props.scanRoots}
              busy={props.busy}
              onClose={props.onCloseSettings}
              onSave={props.onSaveRoots}
            />
          </div>
        </div>
      )}

      {props.candidates && (
        <div className="modal-backdrop" role="presentation">
          <div
            className="modal-card overlay-panel overlay-panel-wide"
            role="dialog"
            aria-label="Scan results"
          >
            <ScanResults
              candidates={props.candidates}
              busy={props.busy}
              onClose={props.onCloseScan}
              onAdd={props.onAddCandidate}
            />
          </div>
        </div>
      )}

      <WorkspaceLayout
        sidebar={sidebar}
        sidebarContent={
          <ProjectList
            projects={props.projects}
            selectedId={props.selectedId}
            query={props.query}
            onQueryChange={props.onQueryChange}
            onSelect={props.onSelect}
          />
        }
        sidebarRail={<SidebarRail onExpand={sidebar.expand} />}
        main={
          <ProjectDetail
            project={props.selected}
            busy={props.busy}
            onError={props.onError}
            onStart={props.onStart}
            onStop={props.onStop}
            onRestart={props.onRestart}
            onRemove={props.onRemove}
            onSaveCommands={props.onSaveCommands}
          />
        }
      />

      <footer className="footer muted">
        {props.status ? (
          <span>
            {props.status.name} v{props.status.version} · DB schema v
            {props.status.schemaVersion} · {props.projects.length} project
            {props.projects.length === 1 ? "" : "s"}
          </span>
        ) : (
          <span>Loading…</span>
        )}
      </footer>
    </main>
  );
}

function isEditableTarget(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) {
    return false;
  }
  const tag = target.tagName;
  return (
    tag === "INPUT" ||
    tag === "TEXTAREA" ||
    tag === "SELECT" ||
    target.isContentEditable
  );
}

export default App;
