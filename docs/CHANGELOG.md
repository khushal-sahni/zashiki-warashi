# CHANGELOG.md
> Written by the AI agent at the end of every session.
> One entry per session. Most recent at the top.

---

## Template

```
## Session #N — YYYY-MM-DD (~Xhr)

### Built
- [What was created from scratch]

### Changed
- [What was modified or refactored]

### Fixed
- [What was broken and is now working]

### Deferred / Not Done
- [What was planned for this session but not completed, and why]

### New Tech Debt
- [Any shortcuts taken, with a note on what the proper fix would be]
```

---

## Sessions

## Session #2 — 2026-08-25

### Built
- Project catalog: add folder, scan roots, list/search, remove from catalog
- Start-command inference (npm/compose/make/cargo/go) with unit tests
- Command overrides in SQLite (not written into project repos)
- Process lifecycle: login-shell spawn, process-group stop, PID/pgid persist + rehydrate
- Schema v2: `pgid`, `last_error`, `started_at_unix` on `project_runs`
- Catalog UI (sidebar, detail, scan results, scan-root settings)
- `tauri-plugin-dialog` for native folder picker

### Changed
- Empty foundation shell replaced by catalog as the main screen
- App footer shows lightweight foundation status

### Fixed
- Nothing notable beyond greenfield M1 work

### Deferred / Not Done
- Docker/DB peek (M2)
- Open in Cursor / Finder / tray (M3)
- Process log capture

### New Tech Debt
- Discarded stdout/stderr for started processes until a logs pane exists
- Timestamp display is unix-seconds string (OK for rehydrate, not pretty)

## Session #1 — 2026-08-25

### Built
- Product definition: localhost control plane for AI-generated / local projects
- Rewrote `AGENTS.md` for Tauri 2 + React/TS + Rust layered architecture
- Filled `docs/ROADMAP.md`, `docs/DECISIONS.md`, `docs/STATUS.md`
- Scaffolded Tauri 2 + React + TypeScript app (`zashiki-warashi`)
- Rust layers: `commands`, `services`, `repositories`, `domain`, `AppError`
- SQLite via rusqlite (bundled) with schema v1 in app data dir
- Logging via `tauri-plugin-log` + tracing
- Empty shell UI calling `get_app_status`
- README with develop/build instructions
- Unit test for database open/migrate

### Changed
- Replaced Nest/Express + Postgres template assumptions across project docs
- Renamed package/product from template `tauri-app` to Zashiki Warashi

### Fixed
- Restored `AGENTS.md` / `docs/` after `create-tauri-app --force` wiped the non-empty tree

### Deferred / Not Done
- Exhaustive GUI click-through (`tauri:dev` did launch Vite + binary)
- M1 catalog + lifecycle (intentionally next)

### New Tech Debt
- Stock Tauri icons; brand later
- Project tables present without CRUD API until M1
