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

  const hint = coffeeHint(enabled, partial);

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
        title={hint}
        aria-pressed={enabled}
        aria-describedby="keep-awake-hint"
        onClick={onToggle}
      >
        <span
          className={[
            "keep-awake-pip",
            enabled ? "on" : "",
            partial ? "partial" : "",
          ]
            .filter(Boolean)
            .join(" ")}
          aria-hidden
        />
        Coffee
      </button>
      <p id="keep-awake-hint" className="keep-awake-hint" role="tooltip">
        {hint}
      </p>
    </div>
  );
}

function coffeeHint(enabled: boolean, partial: boolean): string {
  if (!enabled) {
    return "Keep the Mac awake so projects keep running with the lid closed. macOS will ask for Touch ID or your admin password.";
  }
  if (partial) {
    return "Idle sleep blocked. Approve the macOS prompt (Touch ID or password) to keep running with the lid closed.";
  }
  return "Mac stays awake with the lid closed. Turn off when done — closed lids trap heat.";
}
