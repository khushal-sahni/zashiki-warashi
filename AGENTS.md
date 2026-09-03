# AGENTS.md
> AI agent instructions for this project. Read this before writing any code.

---

## Project Overview

- **Name**: zashiki-warashi
- **Purpose**: A macOS-first desktop control plane for local (often AI-generated) projects — catalog, one-button start/stop, Docker/DB peek, and deep-links into tools you already use (Cursor, Docker, Compass).
- **Stack**: Tauri 2 + React + Vite + TypeScript (UI); Rust (backend)
- **Database**: SQLite in the app data directory (catalog, command overrides, last-known PIDs) — not a server DB
- **Key External Services**: Docker Engine (bollard + `docker compose` CLI), OS shell, deep-links to Cursor / Finder / MongoDB Compass

---

## Architecture

### Pattern
This project uses a **strict layered architecture** across two runtimes:

- **Frontend (TypeScript/React)**: rendering and typed Tauri `invoke` only. No Docker, no process spawn, no SQLite.
- **Backend (Rust)**: all OS, Docker, process, and persistence logic. Logic lives in services; commands are a thin IPC boundary.

Keep the spirit of class-based layered backends: clear ownership, no logic in the UI, no swallowed errors — mapped onto Tauri commands / Rust services rather than HTTP controllers.

### Layer Structure
```
src/                         # React UI — rendering and Tauri invoke only
├── features/projects/
├── features/docker/
├── components/
├── lib/                     # typed invoke wrappers
└── types/

src-tauri/src/
├── commands/                # IPC boundary — parse, call service, return
├── services/                # Business logic — catalog, process, compose, db-peek
├── repositories/            # Data access — SQLite only
├── domain/                  # Project, RunState, DbEndpoint, ...
├── error.rs                 # Typed error hierarchy
└── lib.rs
```

### Strict Layer Rules
- React **never** talks to Docker, never spawns processes, never reads SQLite
- `commands/` **never** contain business logic — parse args, call a service, map errors
- `services/` hold all domain operations; they may call other services shallowly
- `repositories/` **never** contain business logic (no inferring start commands, no parsing compose)
- Utils / helpers are **pure** where possible — no side effects unless named as I/O adapters
- Do **not** write into project repos by default — observation + machine-local catalog only

### Secrets
- Derive connection URIs at runtime from compose/env
- **Do not** persist passwords in SQLite
- Mask secrets in the UI

---

## Coding Standards

### Rust (backend)
- Prefer small modules and explicit return types (`Result<T, AppError>`)
- All domain errors go through `AppError` (or equivalent) — never leak raw `anyhow`/`std::io::Error` across the IPC boundary without mapping
- Prefer `&str` / owned `String` consciously; avoid unnecessary clones in hot paths
- Async where I/O warrants it (Docker, process wait); keep CPU-bound parsing sync
- No `unwrap()` / `expect()` in production paths unless genuinely unreachable and documented

### TypeScript / React (frontend)
- **Strict mode is on** — no `any`, no `as unknown as`, no `!` non-null assertions unless truly unavoidable
- Feature folders own their UI; shared pieces live in `components/` and `lib/`
- All Tauri calls go through typed wrappers in `lib/` — components do not call `invoke` with raw string command names ad hoc
- Prefer `interface` for shapes you don't instantiate; use discriminated unions for status/state
- Prefer `readonly` on props/data that should not mutate

### Naming
| Thing | Convention | Example |
|---|---|---|
| Rust modules / files | snake_case | `project_service.rs` |
| Rust types / enums | PascalCase | `RunState`, `Project` |
| TS components / types | PascalCase | `ProjectList`, `IProject` |
| TS files | kebab-case | `project-list.tsx` |
| Methods / functions | camelCase (TS) / snake_case (Rust) | `getProjectById` / `get_project_by_id` |
| Constants | SCREAMING_SNAKE | `MAX_RETRY_COUNT` |
| DB columns | snake_case | `created_at` |

### Error Handling
- Use a **custom error hierarchy** in Rust (`AppError`) — never throw raw errors across IPC
- Commands map `AppError` to serializable frontend errors
- Frontend surfaces actionable messages; do not swallow failures silently
- Async errors must always be handled — no unhandled promise rejections

### Functions & Methods
- Max **~20 lines** per function where practical — extract when longer
- Max **3 parameters** — if more, use an options/struct
- Single responsibility — if you need "and" to describe a function, split it
- No nested callbacks — use async/await throughout

### Process lifecycle (critical)
- Start project commands via the **user login shell** (`$SHELL -lc`) so `fnm`/`nvm`/`docker` PATH works from a GUI app
- Start in a **new process group**; stop with SIGTERM to the group, then SIGKILL
- Persist pid + start time; treat as running only if the pid is alive and matches
- Processes **outlive the app**; on launch, rehydrate status from PIDs + `docker ps`

### Docker
- Use **bollard** for Engine API listing/inspect
- Use **`docker compose` CLI** for up/down/ps — do not reimplement Compose

### UI layout (panes)
- Major regions are **resizable, collapsible panes** — not fixed CSS grids or competing `max-height` / `min-height` fights
- Use shared primitives in `src/components/panes/` (`WorkspaceLayout`, `DetailSplit`, `usePaneCollapse`, `PaneControlsProvider`)
- Built on `react-resizable-panels` (`Group` / `Panel` / `Separator`) — do not hand-roll drag math
- Each pane needs: **default size**, **min size**, and **collapse/restore** when it is a first-class region
- Persist layout in **`localStorage`** via `useDefaultLayout` with a stable group `id` (e.g. `zw-workspace`, `zw-detail`) — not SQLite
- New surfaces (terminal, env editor, etc.) join an existing `PanelGroup` as a new `Panel`, not a fixed block wedged into flex
- Transient UI (settings, scan results, dialogs) uses **overlays** so it does not steal pane height
- Pane content wrappers use `height: 100%; min-height: 0; overflow: auto` inside the panel fill div

---

## What NOT to Do

- ❌ No business logic in React components or `commands/`
- ❌ No SQLite access outside `repositories/`
- ❌ No `console.log` / `println!` in production paths — use the logging crate / structured logger
- ❌ No hardcoded secrets or machine-specific paths in source — use config / app data dir
- ❌ No circular dependencies between modules
- ❌ No `any` type in TypeScript
- ❌ No mutations of input parameters for "convenience"
- ❌ Never catch an error and swallow it silently
- ❌ Do not invent a Docker GUI, Mongo query editor, AI coding agent, or required per-repo YAML
- ❌ Do not provision databases in v1 (peek + deep-link only)
- ❌ Do not write into managed project repos unless the user explicitly opts into an export format later
- ❌ No hard-coded pane heights (`max-height: 48%`, fixed log `min-height`, etc.) — use the pane system in `src/components/panes/`

---

## Testing Expectations

- Unit tests for all **Rust services** — mock or fake repositories / Docker where needed
- Integration tests for critical IPC flows as they stabilize
- Frontend tests can wait until UI is non-trivial; prefer testing typed wrappers and state logic
- Test files live next to the source: `project_service.rs` → tests in the same module or `project_service_test.rs` pattern used by the crate
- Use descriptive test names: `returns_not_found_when_project_missing`
- Coverage target: **focus on services and repositories** first

---

## Session Instructions for AI Agent

At the **start** of each session:
1. Read `docs/STATUS.md` to understand current project state
2. Read `docs/ROADMAP.md` to understand what's next
3. Ask the engineer to confirm scope before writing any code

During the session:
- Follow all rules in this file without exception
- If a pattern you need isn't covered here, ask before inventing one
- When creating a new backend capability, always create: domain type → repository (if persisted) → service → command → frontend invoke wrapper → UI
- Keep functions small — refactor proactively; avoid files ballooning past ~250–300 lines without a reason

At the **end** of every session, before closing:
1. Update `docs/STATUS.md` — current state, what works, what's pending
2. Append to `docs/CHANGELOG.md` — what was built/changed this session
3. Update `docs/DECISIONS.md` if any architectural choices were made
4. Note any new tech debt under `## Tech Debt` in STATUS.md
