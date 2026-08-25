import { useEffect, useState } from "react";

interface SettingsPanelProps {
  readonly roots: readonly string[];
  readonly busy: boolean;
  readonly onSave: (roots: string[]) => Promise<void>;
  readonly onClose: () => void;
}

export function SettingsPanel({
  roots,
  busy,
  onSave,
  onClose,
}: SettingsPanelProps) {
  const [text, setText] = useState(roots.join("\n"));

  useEffect(() => {
    setText(roots.join("\n"));
  }, [roots]);

  return (
    <section className="panel settings-panel">
      <div className="detail-header">
        <h2>Scan roots</h2>
        <button type="button" className="ghost" onClick={onClose}>
          Close
        </button>
      </div>
      <p className="muted">
        One directory per line. Scan looks at immediate children only.
      </p>
      <textarea
        value={text}
        onChange={(event) => setText(event.target.value)}
        rows={5}
        placeholder="~/Documents/Projects"
      />
      <button
        type="button"
        disabled={busy}
        onClick={() =>
          void onSave(
            text
              .split("\n")
              .map((line) => line.trim())
              .filter(Boolean),
          )
        }
      >
        Save roots
      </button>
    </section>
  );
}
