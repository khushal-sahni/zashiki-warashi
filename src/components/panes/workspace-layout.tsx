import type { ReactNode } from "react";
import {
  Group,
  Panel,
  Separator,
  useDefaultLayout,
} from "react-resizable-panels";

import type { PaneCollapseControls } from "./use-pane-collapse";

const SIDEBAR_COLLAPSED_PX = 36;
const SIDEBAR_MIN_PX = 180;
const SIDEBAR_DEFAULT = "24%";

interface WorkspaceLayoutProps {
  readonly sidebar: PaneCollapseControls;
  readonly sidebarContent: ReactNode;
  readonly sidebarRail: ReactNode;
  readonly main: ReactNode;
}

/**
 * Horizontal workspace: collapsible project sidebar + main detail area.
 * Layout persists in localStorage via useDefaultLayout.
 */
export function WorkspaceLayout({
  sidebar,
  sidebarContent,
  sidebarRail,
  main,
}: WorkspaceLayoutProps) {
  const { defaultLayout, onLayoutChanged } = useDefaultLayout({
    id: "zw-workspace",
    storage: localStorage,
  });

  return (
    <Group
      id="zw-workspace"
      className="workspace"
      orientation="horizontal"
      defaultLayout={defaultLayout}
      onLayoutChanged={onLayoutChanged}
    >
      <Panel
        id="sidebar"
        className="workspace-sidebar-panel"
        panelRef={sidebar.panelRef}
        defaultSize={SIDEBAR_DEFAULT}
        minSize={SIDEBAR_MIN_PX}
        collapsedSize={SIDEBAR_COLLAPSED_PX}
        collapsible
        onResize={sidebar.onResize}
      >
        <div className="pane-fill">
          {sidebar.collapsed ? sidebarRail : sidebarContent}
        </div>
      </Panel>
      <Separator className="pane-separator pane-separator-vertical" />
      <Panel id="main" className="workspace-main-panel" minSize="40%">
        <div className="pane-fill">{main}</div>
      </Panel>
    </Group>
  );
}
