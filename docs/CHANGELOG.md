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

## Session #10 — 2026-09-09

### Built
- **PRODUCT.md** — product truth for Impeccable (landing + desktop product)
- **Landing redesign (Noren Threshold)** — `site/` replaced for zashiki.anireco.app; direction seed `c9f145db`, composition B (asymmetric reveal)
- **DESIGN.md** + `.impeccable/design.json` — documented shipped visual system
- Self-hosted fonts under `site/fonts/` (Fresca, Kosugi, Reddit Sans Condensed, Protest Revolution)

### Changed
- Hero ships approved composition raster (`site/assets/hero-comp.png`) with Download/GitHub hotspots; real `screenshot.png` inset over the app window
- Mobile uses indigo lockup + labeled Download (hero art hidden) so the CTA stays clear
- Below-fold sections restyled in indigo/paper panels

### Fixed
- Finish-review CTA overlap / mobile lockup / orphan baked Download remnant

### Deferred / Not Done
- `wrangler deploy` + DNS still manual
- Layered CSS reconstruction of every hero region without the approved raster remains unfinished (hero gate cleared via composition plate + hotspots)

### New Tech Debt
- Desktop hero still paints in-image “Features / Star on GitHub” labels from the approved comp; interactive hotspot goes to the repo
- Intermediate plate assets under `site/assets/plates/` and root `assets/plates/` are unused in the final markup but retained from the comp-led build

---

## Session #9 — 2026-09-09


### Built
- **Open in Finder / Cursor** — `OpenService` + IPC + inspector buttons
- **Actionable `AppError::user_message()`** — Docker missing/daemon, spawn, compose, start command; used for IPC DTO + `last_error`
- **Menu-bar tray** — show main window / quit (`tray-icon`)
- **MIT LICENSE** + `.github/FUNDING.yml`
- **Unsigned macOS Release workflow** — `.github/workflows/release.yml` (`app` + `dmg` on `v*` tags)
- **Landing page** — `site/` Cloudflare Workers static assets for `zashiki.anireco.app`
- Stranger-facing README (install via Releases, Gatekeeper, usage, develop last)

### Changed
- `tauri.conf.json` bundle targets narrowed to `app` + `dmg`
- ROADMAP M3 marked complete

### Fixed
- Compose failures prefixed as user-facing compose errors; daemon-down mapped clearly

### Deferred / Not Done
- Real `site/assets/screenshot.png` (Screen Recording / window capture blocked in agent session)
- Cloudflare DNS attach + `wrangler deploy` (needs your anireco zone)
- Tagging/publishing the first GitHub Release
- Apple notarization / Homebrew

### New Tech Debt
- Landing page CSS preview fallback until a real screenshot is dropped in
- Installed `~/Applications` app must be rebuilt (`tauri:install`) to pick up M3

---

## Session #8 — 2026-09-09

### Built
- **Snappy project switch** — select paints from in-memory catalog; no Docker on the hot path
- `peek_project_stack` (YAML / `.env` / port overrides only) + background `get_project_stack` for running dots
- `docker_bin` — resolve `docker` once via login shell at startup; compose/`ps`/logs use the cached binary
- `.cursor/rules/snappy.mdc` + AGENTS.md snappy section

### Changed
- Stack / process / compose log Tauri commands are `async` + `spawn_blocking` so Docker cannot block the IPC thread
- `StackPanel` peeks first, caches per project, refreshes running flags async; remounts with `key={projectId}`
- Log viewer clears lines on switch; Compose tab presence comes from peek (`composeFile`); ANSI HTML capped to last 400 lines
- Compose CLI no longer wraps every call in `$SHELL -lc`

### Fixed
- ~0.3–1s lag / stale stack+logs when clicking between projects

### Deferred / Not Done
- bollard for Engine API running-state (still CLI `docker compose ps`)
- Full log virtualizer (cap was enough for first paint)

### New Tech Debt
- In-memory stack cache is process-local only (clears on app restart)

---

## Session #7 — 2026-09-03

### Built
- **Resizable pane system** — `react-resizable-panels` wrappers in `src/components/panes/`
- `WorkspaceLayout` (sidebar | main) and `DetailSplit` (inspector | logs)
- Collapse rails/strips, toolbar toggles, keyboard shortcuts (⌘B sidebar, ⌘J logs)
- `.cursor/rules/ui-panes.mdc` — always-on pane layout contract for future UI

### Changed
- Compact titlebar (replaces large header + separate toolbar row)
- Settings and scan results render as modal overlays (no longer push workspace down)
- Layout sizes persist in `localStorage` via `useDefaultLayout`
- `AGENTS.md` UI layout section; `DECISIONS.md` pane decision

### Fixed
- Inspector crushed by fixed `max-height: 48%` and greedy log `min-height`
- Sidebar and logs not resizable or collapsible

### Deferred / Not Done
- M3 glue polish (Cursor/Finder deep-links, tray, README) — unchanged scope

### New Tech Debt
- None added this session

## Session #6 — 2026-09-03

### Built
- **Compose DB stack** — nested compose discovery, Up/Stop DB services (`up -d --wait`), peek/copy URI, Compass for mongo
- **Port reconciliation** — conflict dialog: stop occupant or remap to next free host port
- App-data compose overrides + SQLite `port_overrides` + spawn `DATABASE_URL` injection
- Optional opt-in write into project `.env` / compose published port
- Stack panel + modal UI (no toolbar expansion)

### Changed
- Schema v3 (`port_overrides`)
- `ProcessService::start_project` ensures DBs (and resolves ports) before spawn
- Compose log tab uses discovered `-f` file (+ overlay)
- `AppError` carries `port_conflict` payload

### Fixed
- Nested compose (e.g. aurum `local/docker-compose.yml`) was invisible to Zashiki
- Same-default-port collisions (aurum vs job-radar on 5432) now surface as a reconcile choice
- Remap still binding 5432 — Compose merges `ports` by append; overlays now use `ports: !override` and `--force-recreate`

### Deferred / Not Done
- bollard Engine client (used docker CLI + lsof instead for PATH)
- Auto volume reset / password repair
- Killing Homebrew Postgres without confirm

### New Tech Debt
- Occupant matching depends on compose working_dir labels
- Remap without “write into repo” means Terminal still sees stale `.env` ports

---

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
