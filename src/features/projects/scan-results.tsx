import type { ScanCandidate } from "../../types";

interface ScanResultsProps {
  readonly candidates: readonly ScanCandidate[];
  readonly busy: boolean;
  readonly onAdd: (path: string) => Promise<void>;
  readonly onClose: () => void;
}

export function ScanResults({
  candidates,
  busy,
  onAdd,
  onClose,
}: ScanResultsProps) {
  return (
    <section className="panel scan-panel">
      <div className="detail-header">
        <h2>Scan results</h2>
        <button type="button" className="ghost" onClick={onClose}>
          Close
        </button>
      </div>
      {candidates.length === 0 ? (
        <p className="muted">No project candidates found in scan roots.</p>
      ) : (
        <ul className="scan-list">
          {candidates.map((candidate) => (
            <li key={candidate.path} className="scan-item">
              <div>
                <strong>{candidate.name}</strong>
                <p className="path">{candidate.path}</p>
                <p className="muted">
                  {candidate.inferredStartCommand ?? "no inferred start command"}
                </p>
              </div>
              {candidate.alreadyRegistered ? (
                <span className="badge badge-stopped">Registered</span>
              ) : (
                <button
                  type="button"
                  disabled={busy}
                  onClick={() => void onAdd(candidate.path)}
                >
                  Add
                </button>
              )}
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
