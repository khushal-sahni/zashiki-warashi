import { useCallback, useEffect, useRef, useState } from "react";

import {
  formatInvokeError,
  getProjectStack,
  openCompass,
  peekProjectStack,
  startProjectStack,
  stopProjectStack,
} from "../../lib";
import type { DbEndpoint, ProjectStack } from "../../types";

interface StackPanelProps {
  readonly projectId: string;
  readonly busy: boolean;
  readonly onBusyError: (message: string | null) => void;
  readonly onComposePresence?: (hasCompose: boolean) => void;
}

const stackCache = new Map<string, ProjectStack>();

export function StackPanel({
  projectId,
  busy,
  onBusyError,
  onComposePresence,
}: StackPanelProps) {
  const [stack, setStack] = useState<ProjectStack | null>(
    () => stackCache.get(projectId) ?? null,
  );
  const [localBusy, setLocalBusy] = useState(false);
  const generation = useRef(0);

  const applyStack = useCallback(
    (next: ProjectStack): void => {
      stackCache.set(projectId, next);
      setStack(next);
      onComposePresence?.(next.composeFile !== null);
    },
    [onComposePresence, projectId],
  );

  useEffect(() => {
    const gen = ++generation.current;
    const cached = stackCache.get(projectId) ?? null;
    setStack(cached);
    onComposePresence?.(cached?.composeFile != null);

    void peekProjectStack(projectId)
      .then((peek) => {
        if (generation.current !== gen) {
          return;
        }
        applyStack(peek);
        if (peek.composeFile === null) {
          return;
        }
        void getProjectStack(projectId).then((full) => {
          if (generation.current !== gen) {
            return;
          }
          applyStack(full);
        });
      })
      .catch((err: unknown) => {
        if (generation.current === gen) {
          onBusyError(formatInvokeError(err));
        }
      });
  }, [applyStack, onBusyError, onComposePresence, projectId]);

  if (!stack || stack.composeFile === null) {
    return null;
  }

  if (stack.endpoints.length === 0) {
    return (
      <div className="stack-panel">
        <div className="stack-header">
          <h3>Stack</h3>
          <span className="muted">{stack.composeFile}</span>
        </div>
        <p className="muted stack-empty">No database services found in compose.</p>
      </div>
    );
  }

  const disabled = busy || localBusy;

  return (
    <div className="stack-panel">
      <div className="stack-header">
        <h3>Stack</h3>
        <span className="muted">{stack.composeFile}</span>
        <button
          type="button"
          className="ghost log-chip"
          disabled={disabled}
          onClick={() => {
            void runAction(async () => {
              applyStack(await startProjectStack(projectId));
            }, setLocalBusy, onBusyError);
          }}
        >
          Up
        </button>
        <button
          type="button"
          className="ghost log-chip"
          disabled={disabled}
          onClick={() => {
            void runAction(async () => {
              applyStack(await stopProjectStack(projectId));
            }, setLocalBusy, onBusyError);
          }}
        >
          Stop
        </button>
      </div>
      <ul className="stack-list">
        {stack.endpoints.map((endpoint) => (
          <StackRow
            key={endpoint.service}
            endpoint={endpoint}
            disabled={disabled}
            onCopy={() => {
              void navigator.clipboard.writeText(endpoint.uri);
            }}
            onCompass={() => {
              void openCompass(endpoint.uri).catch((err: unknown) => {
                onBusyError(formatInvokeError(err));
              });
            }}
          />
        ))}
      </ul>
    </div>
  );
}

interface StackRowProps {
  readonly endpoint: DbEndpoint;
  readonly disabled: boolean;
  readonly onCopy: () => void;
  readonly onCompass: () => void;
}

function StackRow({ endpoint, disabled, onCopy, onCompass }: StackRowProps) {
  return (
    <li className="stack-row">
      <span
        className={["log-pip", endpoint.running ? "live" : ""].filter(Boolean).join(" ")}
        aria-hidden
      />
      <div className="stack-row-main">
        <strong>
          {endpoint.service} · {endpoint.kind}
        </strong>
        <code>
          {endpoint.host}:{endpoint.port}
        </code>
        <code className="stack-uri">{endpoint.uriMasked}</code>
      </div>
      <button type="button" className="ghost log-chip" disabled={disabled} onClick={onCopy}>
        Copy
      </button>
      {endpoint.kind === "mongo" && (
        <button
          type="button"
          className="ghost log-chip"
          disabled={disabled}
          onClick={onCompass}
        >
          Compass
        </button>
      )}
    </li>
  );
}

async function runAction(
  action: () => Promise<void>,
  setLocalBusy: (value: boolean) => void,
  onBusyError: (message: string | null) => void,
): Promise<void> {
  setLocalBusy(true);
  onBusyError(null);
  try {
    await action();
  } catch (err) {
    onBusyError(formatInvokeError(err));
  } finally {
    setLocalBusy(false);
  }
}
