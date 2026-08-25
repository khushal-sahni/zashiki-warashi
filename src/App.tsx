import { useEffect, useState } from "react";

import { ShellHeader } from "./components";
import { getAppStatus } from "./lib";
import type { AppStatus } from "./types";
import "./App.css";

type LoadState =
  | { readonly kind: "loading" }
  | { readonly kind: "ready"; readonly status: AppStatus }
  | { readonly kind: "error"; readonly message: string };

function App() {
  const [state, setState] = useState<LoadState>({ kind: "loading" });

  useEffect(() => {
    let cancelled = false;

    async function loadStatus(): Promise<void> {
      try {
        const status = await getAppStatus();
        if (!cancelled) {
          setState({ kind: "ready", status });
        }
      } catch (error) {
        const message =
          error instanceof Error
            ? error.message
            : typeof error === "object" &&
                error !== null &&
                "message" in error &&
                typeof error.message === "string"
              ? error.message
              : "Failed to load app status";
        if (!cancelled) {
          setState({ kind: "error", message });
        }
      }
    }

    void loadStatus();

    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <main className="shell">
      <ShellHeader
        title="Zashiki Warashi"
        subtitle="A house spirit for your local projects — catalog, start/stop, and Docker peek."
      />

      <section className="panel">
        <h2>Foundation</h2>
        {state.kind === "loading" && <p className="muted">Checking backend…</p>}
        {state.kind === "error" && (
          <p className="error" role="alert">
            {state.message}
          </p>
        )}
        {state.kind === "ready" && (
          <dl className="status-grid">
            <div>
              <dt>App</dt>
              <dd>
                {state.status.name} v{state.status.version}
              </dd>
            </div>
            <div>
              <dt>Database</dt>
              <dd>{state.status.databaseReady ? "Ready" : "Not ready"}</dd>
            </div>
            <div>
              <dt>Schema</dt>
              <dd>v{state.status.schemaVersion}</dd>
            </div>
            <div>
              <dt>App data</dt>
              <dd className="path">{state.status.appDataDir}</dd>
            </div>
          </dl>
        )}
      </section>

      <section className="panel muted-panel">
        <h2>Coming next</h2>
        <ul>
          <li>Project catalog — register, scan, and find local repos</li>
          <li>Lifecycle — one-button start / stop / status</li>
          <li>Docker / DB peek — compose services and Compass deep-links</li>
        </ul>
      </section>
    </main>
  );
}

export default App;
