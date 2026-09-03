import { useState } from "react";

import type { PortConflict } from "../../types";

interface PortConflictDialogProps {
  readonly conflict: PortConflict;
  readonly busy: boolean;
  readonly onDismiss: () => void;
  readonly onStopOccupant: (confirmNative: boolean) => void;
  readonly onRemap: (writeToRepo: boolean) => void;
}

export function PortConflictDialog({
  conflict,
  busy,
  onDismiss,
  onStopOccupant,
  onRemap,
}: PortConflictDialogProps) {
  const [writeToRepo, setWriteToRepo] = useState(false);
  const [confirmNative, setConfirmNative] = useState(false);
  const isNative = conflict.occupant.kind === "nativeProcess";

  return (
    <div className="modal-backdrop" role="presentation">
      <div className="modal-card" role="dialog" aria-modal="true" aria-labelledby="port-conflict-title">
        <h3 id="port-conflict-title">Port {conflict.hostPort} is busy</h3>
        <p className="muted">
          Service <code>{conflict.service}</code> wants host port {conflict.hostPort}, but{" "}
          <strong>{occupantLabel(conflict)}</strong> is already listening there.
        </p>
        <div className="modal-actions">
          <button
            type="button"
            disabled={busy || (isNative && !confirmNative)}
            onClick={() => onStopOccupant(confirmNative)}
          >
            Stop {shortOccupant(conflict)}
          </button>
          <button
            type="button"
            disabled={busy}
            onClick={() => onRemap(writeToRepo)}
          >
            Use port {conflict.suggestedPort}
          </button>
          <button type="button" className="ghost" disabled={busy} onClick={onDismiss}>
            Cancel
          </button>
        </div>
        <label className="modal-check">
          <input
            type="checkbox"
            checked={writeToRepo}
            onChange={(event) => setWriteToRepo(event.target.checked)}
          />
          Also write the new port into the repo (.env / compose)
        </label>
        {isNative && (
          <label className="modal-check warn">
            <input
              type="checkbox"
              checked={confirmNative}
              onChange={(event) => setConfirmNative(event.target.checked)}
            />
            I understand this stops a native system process
          </label>
        )}
      </div>
    </div>
  );
}

function occupantLabel(conflict: PortConflict): string {
  const occupant = conflict.occupant;
  switch (occupant.kind) {
    case "catalogProject":
      return `${occupant.name} (${occupant.service})`;
    case "dockerOther":
      return occupant.service
        ? `${occupant.containerName} (${occupant.service})`
        : occupant.containerName;
    case "nativeProcess":
      return `pid ${occupant.pid} (${occupant.command})`;
    case "unknown":
      return "an unknown process";
  }
}

function shortOccupant(conflict: PortConflict): string {
  const occupant = conflict.occupant;
  switch (occupant.kind) {
    case "catalogProject":
      return occupant.name;
    case "dockerOther":
      return occupant.containerName;
    case "nativeProcess":
      return `pid ${occupant.pid}`;
    case "unknown":
      return "occupant";
  }
}
