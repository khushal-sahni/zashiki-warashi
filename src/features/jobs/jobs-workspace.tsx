import { JobDetail } from "./job-detail";
import { JobEditor } from "./job-editor";
import { JobList } from "./job-list";
import { JobsSettings } from "./jobs-settings";
import type { JobsController } from "./use-jobs-controller";

interface JobsPartProps {
  readonly controller: JobsController;
  readonly busy: boolean;
}

export function JobsSidebar({ controller }: JobsPartProps) {
  const { state } = controller;
  return (
    <JobList
      jobs={state.jobs}
      loaded={state.loaded}
      selectedId={state.selectedId}
      agents={state.agents}
      onSelect={state.select}
    />
  );
}

export function JobsMain({ controller, busy, onError }: JobsPartProps & { readonly onError: (message: string | null) => void }) {
  const { state } = controller;
  if (!state.selected) {
    return <JobsEmpty controller={controller} busy={busy} />;
  }
  return (
    <JobDetail
      key={state.selected.id}
      job={state.selected}
      system={state.system}
      busy={busy}
      onRunNow={controller.runNow}
      onEdit={controller.openEdit}
      onToggle={controller.toggle}
      onDelete={controller.remove}
      onOpenSettings={() => controller.setShowSettings(true)}
      onError={onError}
    />
  );
}

function JobsEmpty({ controller, busy }: JobsPartProps) {
  const loaded = controller.state.loaded;
  return (
    <section className="panel detail-panel jobs-empty">
      <h2>{loaded ? "Schedule a job" : "Loading jobs…"}</h2>
      {loaded && (
        <>
          <p className="muted">
            Run a script daily, hourly, or on chosen weekdays. Zashiki uses
            launchd, so jobs fire even when this window is closed. Pick
            “catch up on wake” for jobs that can run late, or “wake the Mac” for
            jobs that need to run on time with Wi-Fi.
          </p>
          <div className="action-row">
            <button type="button" className="primary" disabled={busy} onClick={controller.openNew}>
              New job
            </button>
            <button type="button" className="ghost" onClick={() => controller.setShowSettings(true)}>
              Wake &amp; agent setup
            </button>
          </div>
        </>
      )}
    </section>
  );
}

export function JobsOverlays({ controller, busy }: JobsPartProps) {
  const { editor, showSettings, state } = controller;
  return (
    <>
      {editor && (
        <div className="modal-backdrop" role="presentation">
          <div className="modal-card overlay-panel overlay-panel-wide" role="dialog" aria-label="Job editor">
            <JobEditor
              job={editor.kind === "edit" ? editor.job : null}
              busy={busy}
              wakeHelperInstalled={state.system?.wakeHelperInstalled ?? false}
              onSave={controller.save}
              onClose={controller.closeEditor}
            />
          </div>
        </div>
      )}
      {showSettings && (
        <div className="modal-backdrop" role="presentation">
          <div className="modal-card overlay-panel overlay-panel-wide" role="dialog" aria-label="Jobs settings">
            <JobsSettings
              system={state.system}
              busy={busy}
              onInstallHelper={controller.installHelper}
              onRemoveHelper={controller.removeHelper}
              onClose={() => controller.setShowSettings(false)}
            />
          </div>
        </div>
      )}
    </>
  );
}
