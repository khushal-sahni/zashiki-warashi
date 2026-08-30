import type { KeepAwakeStatus } from "../types";

interface KeepAwakeToggleProps {
  readonly status: KeepAwakeStatus | null;
  readonly busy: boolean;
  readonly onToggle: () => void;
}

export function KeepAwakeToggle({
  status,
  busy,
  onToggle,
}: KeepAwakeToggleProps) {
  const enabled = status?.enabled ?? false;
  const lidClosedArmed = status?.lidClosedArmed ?? false;
  const partial = enabled && !lidClosedArmed;

  const label = enabled
    ? partial
      ? "Coffee (idle only)"
      : "Coffee on"
    : "Coffee";

  const title = enabled
    ? partial
      ? "Idle sleep blocked. Approve the macOS prompt (Touch ID or password) to keep running with the lid closed."
      : "Mac stays awake with the lid closed. Turn off when done — closed lids trap heat."
    : "Keep the Mac awake so projects keep running with the lid closed. macOS will ask for Touch ID or your admin password.";

  return (
    <div className="keep-awake-control">
      <button
        type="button"
        className={[
          "ghost",
          enabled ? "active" : "",
          partial ? "partial" : "",
        ]
          .filter(Boolean)
          .join(" ")}
        disabled={busy}
        title={title}
        aria-pressed={enabled}
        onClick={onToggle}
      >
        {label}
      </button>
      {enabled && (
        <p
          className={["keep-awake-note", partial ? "warn" : ""]
            .filter(Boolean)
            .join(" ")}
        >
          {partial
            ? "Approve the macOS prompt (Touch ID or password) for lid-close sleep."
            : "Heat and battery drain quickly with the lid closed. Turn Coffee off when finished."}
        </p>
      )}
    </div>
  );
}
