import { open } from "@tauri-apps/plugin-dialog";
import { useCallback, useEffect, useMemo, useState } from "react";

import { KeepAwakeToggle, ShellHeader } from "./components";
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
  removeProject,
  restartProject,
  scanProjects,
  setKeepAwakeEnabled,
  setScanRoots,
  startProject,
  stopProject,
  updateProjectCommands,
} from "./lib";
import type { AppStatus, KeepAwakeStatus, Project, ScanCandidate } from "./types";
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

  async function handleScan(): Promise<void> {
    await withBusy(async () => {
      const results = await scanProjects();
      setCandidates(results);
      setShowSettings(false);
    });
  }

  return (
    <main className="shell app-shell">
      <ShellHeader
        title="Zashiki Warashi"
        subtitle="Catalog and start/stop your local projects."
      />

      <div className="toolbar">
        <button type="button" disabled={busy} onClick={() => void handleAddFolder()}>
          Add folder
        </button>
        <button type="button" disabled={busy} onClick={() => void handleScan()}>
          Scan
        </button>
        <KeepAwakeToggle
          status={keepAwake}
          busy={busy}
          onToggle={() => void handleToggleKeepAwake()}
        />
        <button
          type="button"
          className="ghost"
          disabled={busy}
          onClick={() => {
            setShowSettings((value) => !value);
            setCandidates(null);
          }}
        >
          Settings
        </button>
        <button
          type="button"
          className="ghost"
          disabled={busy}
          onClick={() => void withBusy(refresh)}
        >
          Refresh
        </button>
      </div>

      {error && (
        <p className="error banner" role="alert">
          {error}
        </p>
      )}

      {showSettings && (
        <SettingsPanel
          roots={scanRoots}
          busy={busy}
          onClose={() => setShowSettings(false)}
          onSave={async (roots) => {
            await withBusy(async () => {
              const settings = await setScanRoots(roots);
              setScanRootsState(settings.scanRoots);
              setShowSettings(false);
            });
          }}
        />
      )}

      {candidates && (
        <ScanResults
          candidates={candidates}
          busy={busy}
          onClose={() => setCandidates(null)}
          onAdd={async (path) => {
            await withBusy(async () => {
              const project = await addProject(path);
              const results = await scanProjects();
              setCandidates(results);
              await refresh();
              setSelectedId(project.id);
            });
          }}
        />
      )}

      <div className="workspace">
        <ProjectList
          projects={projects}
          selectedId={selectedId}
          query={query}
          onQueryChange={setQuery}
          onSelect={setSelectedId}
        />
        <ProjectDetail
          project={selected}
          busy={busy}
          onStart={async (id) => {
            await withBusy(async () => {
              const project = await startProject(id);
              setProjects((current) =>
                current.map((item) => (item.id === id ? project : item)),
              );
            });
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
            await withBusy(async () => {
              const project = await restartProject(id);
              setProjects((current) =>
                current.map((item) => (item.id === id ? project : item)),
              );
            });
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
      </div>

      <footer className="footer muted">
        {status ? (
          <span>
            {status.name} v{status.version} · DB schema v{status.schemaVersion} ·{" "}
            {projects.length} project{projects.length === 1 ? "" : "s"}
          </span>
        ) : (
          <span>Loading…</span>
        )}
      </footer>
    </main>
  );
}

export default App;
