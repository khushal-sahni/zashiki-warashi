interface SidebarRailProps {
  readonly onExpand: () => void;
}

export function SidebarRail({ onExpand }: SidebarRailProps) {
  return (
    <div className="sidebar-rail">
      <button
        type="button"
        className="ghost sidebar-rail-toggle"
        title="Show sidebar (⌘B)"
        aria-label="Show sidebar"
        onClick={onExpand}
      >
        ▸
      </button>
    </div>
  );
}
