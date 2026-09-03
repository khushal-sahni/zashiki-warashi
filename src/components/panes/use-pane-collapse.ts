import { useCallback, useMemo, useState, type RefObject } from "react";
import {
  usePanelRef,
  type PanelImperativeHandle,
  type PanelSize,
} from "react-resizable-panels";

export interface PaneCollapseControls {
  readonly panelRef: RefObject<PanelImperativeHandle | null>;
  readonly collapsed: boolean;
  readonly onResize: (size: PanelSize) => void;
  readonly toggle: () => void;
  readonly collapse: () => void;
  readonly expand: () => void;
}

/**
 * Tracks collapse state for a collapsible Panel and exposes toggle helpers
 * for toolbar buttons, keyboard shortcuts, and collapse rails.
 */
export function usePaneCollapse(): PaneCollapseControls {
  const panelRef = usePanelRef();
  const [collapsed, setCollapsed] = useState(false);

  const onResize = useCallback(
    (size: PanelSize) => {
      const fromApi = panelRef.current?.isCollapsed();
      if (typeof fromApi === "boolean") {
        setCollapsed(fromApi);
        return;
      }
      setCollapsed(size.inPixels <= 1 || size.asPercentage <= 0.5);
    },
    [panelRef],
  );

  const collapse = useCallback(() => {
    panelRef.current?.collapse();
    setCollapsed(true);
  }, [panelRef]);

  const expand = useCallback(() => {
    panelRef.current?.expand();
    setCollapsed(false);
  }, [panelRef]);

  const toggle = useCallback(() => {
    if (panelRef.current?.isCollapsed()) {
      expand();
    } else {
      collapse();
    }
  }, [collapse, expand, panelRef]);

  return useMemo(
    () => ({ panelRef, collapsed, onResize, toggle, collapse, expand }),
    [panelRef, collapsed, onResize, toggle, collapse, expand],
  );
}
