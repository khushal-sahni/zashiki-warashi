import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";

import type { PaneCollapseControls } from "./use-pane-collapse";

export interface LogsPaneRegistration {
  readonly collapsed: boolean;
  readonly toggle: () => void;
  readonly expand: () => void;
}

interface PaneControlsValue {
  readonly sidebar: PaneCollapseControls;
  readonly logsCollapsed: boolean;
  readonly toggleLogs: () => void;
  readonly expandLogs: () => void;
  readonly registerLogs: (controls: LogsPaneRegistration | null) => void;
}

const PaneControlsContext = createContext<PaneControlsValue | null>(null);

interface PaneControlsProviderProps {
  readonly sidebar: PaneCollapseControls;
  readonly children: ReactNode;
}

export function PaneControlsProvider({
  sidebar,
  children,
}: PaneControlsProviderProps) {
  const logsRef = useRef<LogsPaneRegistration | null>(null);
  const [logsCollapsed, setLogsCollapsed] = useState(false);

  const registerLogs = useCallback((controls: LogsPaneRegistration | null) => {
    logsRef.current = controls;
    const next = controls?.collapsed ?? false;
    setLogsCollapsed((current) => (current === next ? current : next));
  }, []);

  const toggleLogs = useCallback(() => {
    logsRef.current?.toggle();
  }, []);

  const expandLogs = useCallback(() => {
    logsRef.current?.expand();
  }, []);

  const value = useMemo(
    (): PaneControlsValue => ({
      sidebar,
      logsCollapsed,
      toggleLogs,
      expandLogs,
      registerLogs,
    }),
    [expandLogs, logsCollapsed, registerLogs, sidebar, toggleLogs],
  );

  return (
    <PaneControlsContext.Provider value={value}>
      {children}
    </PaneControlsContext.Provider>
  );
}

export function usePaneControls(): PaneControlsValue {
  const value = useContext(PaneControlsContext);
  if (!value) {
    throw new Error("usePaneControls must be used within PaneControlsProvider");
  }
  return value;
}
