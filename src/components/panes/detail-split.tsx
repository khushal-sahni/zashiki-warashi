import type { ReactNode } from "react";
import {
  Group,
  Panel,
  Separator,
  useDefaultLayout,
} from "react-resizable-panels";

import type { PaneCollapseControls } from "./use-pane-collapse";

const INSPECTOR_DEFAULT = "42%";
const INSPECTOR_MIN_PX = 160;
const LOGS_DEFAULT = "58%";
const LOGS_MIN_PX = 120;
const LOGS_COLLAPSED_PX = 40;

interface DetailSplitProps {
  readonly logs: PaneCollapseControls;
  readonly inspector: ReactNode;
  readonly logsPane: ReactNode;
}

/**
 * Vertical detail split: project inspector (info + stack) over logs.
 * Layout persists in localStorage via useDefaultLayout.
 */
export function DetailSplit({ logs, inspector, logsPane }: DetailSplitProps) {
  const { defaultLayout, onLayoutChanged } = useDefaultLayout({
    id: "zw-detail",
    storage: localStorage,
  });

  return (
    <Group
      id="zw-detail"
      className="detail-split"
      orientation="vertical"
      defaultLayout={defaultLayout}
      onLayoutChanged={onLayoutChanged}
    >
      <Panel
        id="inspector"
        className="detail-inspector-panel"
        defaultSize={INSPECTOR_DEFAULT}
        minSize={INSPECTOR_MIN_PX}
      >
        <div className="pane-fill detail-scroll">{inspector}</div>
      </Panel>
      <Separator className="pane-separator pane-separator-horizontal" />
      <Panel
        id="logs"
        className="detail-logs-panel"
        panelRef={logs.panelRef}
        defaultSize={LOGS_DEFAULT}
        minSize={LOGS_MIN_PX}
        collapsedSize={LOGS_COLLAPSED_PX}
        collapsible
        onResize={logs.onResize}
      >
        <div className="pane-fill">{logsPane}</div>
      </Panel>
    </Group>
  );
}
