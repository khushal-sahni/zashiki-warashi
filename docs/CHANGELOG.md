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

## Session #5 — 2026-09-03

### Built
- **Logs pane** on project detail — follow, filter, wrap, copy, clear, ANSI color
- `LogService` — app-data log files, rotate on start, tail, clear, compose CLI snapshot
- Background file tailer emitting `project-log` events
- Process/Compose log sources (Compose only when compose files exist)

### Changed
- `ProcessService` redirects child stdout/stderr to `{app_data}/logs/{id}/current.log`
- Window default 1180×800; shell fills the viewport so the log pane can grow
- Coffee toggle: fixed "Coffee" label, status pip, hover popover (no toolbar reflow)
- CSS variables for sage/amber/ink used by logs + Coffee

### Fixed
- Coffee control expanding the toolbar with inline warning copy

### Deferred / Not Done
- M2 Docker/DB peek (bollard, compose up/down, service list)
- Log archive browser, download, stdin
- Live `docker compose logs -f` sidecar (snapshots + poll instead)

### New Tech Debt
- Log files are unbounded between starts
- Compose polling is 1s, not event-streamed

---

## Session #4 — 2026-08-30


### Built
- **Coffee toggle** — toolbar keep-awake control for macOS
- `KeepAwakeService` — spawns `caffeinate -ims`, runs `pmset disablesleep` via osascript admin prompt
- `KeepAwakeToggle` component with full/partial armed states and heat warning
- Meta persistence: `keep_awake_enabled`, `keep_awake_caffeinate_pid`
- 5 unit tests for keep-awake service (fake runner)

### Changed
- `App.tsx` toolbar — Coffee button between Scan and Settings
- IPC: `get_keep_awake_status`, `set_keep_awake_enabled`
- Rehydrate keep-awake on app launch (same pattern as project PIDs)

### Fixed
- N/A

### Deferred / Not Done
- Battery-floor auto-off, timed Coffee sessions
- Amphetamine / third-party keep-awake integration (using native tools instead)
- Changing `tcpkeepalive` / `networkoversleep` pmset knobs

### New Tech Debt
- Admin password prompt on every Coffee enable/disable unless user adds a scoped sudoers rule manually
- Wi-Fi in closed lid is hardware-limited; Ethernet/dock is the fallback for flaky RF

## Session #3 — 2026-08-26

### Built
- `scripts/install-macos.sh` — release build (`--bundles app`), safe replace into `~/Applications`, clear quarantine
- `npm run tauri:install` for daily-driver install (Spotlight / Dock / Launchpad)

### Changed
- README: Develop vs Install (daily driver) vs Build sections
- STATUS: how to run + session notes

### Fixed
- (none)

### Deferred / Not Done
- Auto-update installed app on `tauri:dev` (intentionally out of scope)
- Apple signing / notarization (local personal use only)

### New Tech Debt
- Default Tauri icons still in place for the installed app

---

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
