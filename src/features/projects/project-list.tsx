import type { Project, RunState } from "../../types";

interface ProjectListProps {
  readonly projects: readonly Project[];
  readonly selectedId: string | null;
  readonly query: string;
  readonly onQueryChange: (query: string) => void;
  readonly onSelect: (id: string) => void;
}

function statusClass(status: RunState): string {
  return `status-dot status-${status}`;
}

export function ProjectList({
  projects,
  selectedId,
  query,
  onQueryChange,
  onSelect,
}: ProjectListProps) {
  const filtered = projects.filter((project) => {
    const needle = query.trim().toLowerCase();
    if (!needle) {
      return true;
    }
    return (
      project.name.toLowerCase().includes(needle) ||
      project.path.toLowerCase().includes(needle)
    );
  });

  return (
    <aside className="sidebar">
      <label className="search">
        <span className="sr-only">Search projects</span>
        <input
          value={query}
          onChange={(event) => onQueryChange(event.target.value)}
          placeholder="Search projects…"
        />
      </label>
      <ul className="project-list">
        {filtered.length === 0 && (
          <li className="empty-item">No projects match.</li>
        )}
        {filtered.map((project) => (
          <li key={project.id}>
            <button
              type="button"
              className={
                project.id === selectedId ? "project-item active" : "project-item"
              }
              onClick={() => onSelect(project.id)}
            >
              <span className={statusClass(project.run.status)} title={project.run.status} />
              <span className="project-item-text">
                <span className="project-item-name">{project.name}</span>
                <span className="project-item-status">{project.run.status}</span>
              </span>
            </button>
          </li>
        ))}
      </ul>
    </aside>
  );
}
