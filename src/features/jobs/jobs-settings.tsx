import { useState } from "react";

import { formatInvokeError } from "../../lib";
import type { JobSystemStatus } from "../../types";

interface JobsSettingsProps {
  readonly system: JobSystemStatus | null;
  readonly busy: boolean;
  readonly onInstallHelper: () => Promise<void>;
  readonly onRemoveHelper: () => Promise<void>;
  readonly onClose: () => void;
}

export function JobsSettings({ system, busy, onInstallHelper, onRemoveHelper, onClose }: JobsSettingsProps) {
  const [copied, setCopied] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  function copy(label: string, value: string): void {
    navigator.clipboard
      .writeText(value)
      .then(() => setCopied(label))
      .catch((err: unknown) => setError(formatInvokeError(err)));
  }

  return (
    <section className="panel settings-panel jobs-settings">
      <div className="detail-header">
        <h2>Scheduled jobs</h2>
        <button type="button" className="ghost" onClick={onClose}>
          Close
        </button>
      </div>

      <h3 className="settings-heading">Wake helper</h3>
      <p className="muted">
        Jobs set to “Wake the Mac on time” need a small background helper that
        asks macOS to wake ~90 seconds before they run. It only adds and removes
        Zashiki’s own wakes. Installing asks for your password once.
      </p>
      <div className="settings-row">
        <span className={system?.wakeHelperInstalled ? "badge badge-ok" : "badge badge-stopped"}>
          {system === null ? "Checking…" : system.wakeHelperInstalled ? "Installed" : "Not installed"}
        </span>
        {system?.wakeHelperInstalled ? (
          <button type="button" className="ghost" disabled={busy} onClick={() => void onRemoveHelper()}>
            Remove helper
          </button>
        ) : (
          <button type="button" className="primary" disabled={busy || system === null} onClick={() => void onInstallHelper()}>
            Install wake helper
          </button>
        )}
      </div>
      <p className="field-hint">
        A closed lid only stays awake on power. On battery, exact jobs fall back
        to running on the next wake.
      </p>

      <h3 className="settings-heading">For agents and scripts</h3>
      <p className="muted">
        Add this MCP server to Cursor or another agent so it can list, create,
        and run jobs. Scripts can talk JSON lines to the socket directly.
      </p>
      <CopyField label="MCP command" value={system?.mcpCommand ?? "…"} copied={copied} onCopy={copy} />
      <CopyField label="Socket" value={system?.socketPath ?? "…"} copied={copied} onCopy={copy} />
      <p className="field-hint">
        Background supervisor: {system === null ? "checking…" : system.supervisorInstalled ? "installed" : "not installed yet (reopen Zashiki)"}
      </p>
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}

interface CopyFieldProps {
  readonly label: string;
  readonly value: string;
  readonly copied: string | null;
  readonly onCopy: (label: string, value: string) => void;
}

function CopyField({ label, value, copied, onCopy }: CopyFieldProps) {
  return (
    <div className="copy-field">
      <span className="copy-field-label">{label}</span>
      <code className="copy-field-value" title={value}>
        {value}
      </code>
      <button type="button" className="ghost log-chip" onClick={() => onCopy(label, value)}>
        {copied === label ? "Copied" : "Copy"}
      </button>
    </div>
  );
}
