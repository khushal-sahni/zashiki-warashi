# STATUS.md
> Your weekend dashboard. Read this first. Update this last.
> Last updated: `2026-09-03` | Session: `#7`

---

## What Works Right Now

- [x] Product vision and architecture locked in docs
- [x] Tauri 2 + React + Vite + TypeScript scaffold
- [x] SQLite app-data DB (schema v3 — port overrides)
- [x] Typed `AppError` + logging (+ `port_conflict` payload)
- [x] **Project catalog** — add folder, scan roots, list/search, remove (catalog only)
- [x] **Start command inference** — package.json / compose / Makefile / cargo / go
- [x] **Command overrides** stored in app DB (not written into repos)
- [x] **Lifecycle** — start / stop / restart via login shell + process groups
- [x] **PID + pgid persistence** and rehydrate on launch
- [x] Catalog UI (sidebar + detail + scan results + scan-root settings)
- [x] **Local install** — `npm run tauri:install` → `~/Applications/Zashiki Warashi.app`
- [x] **Coffee toggle** — compact toolbar keep-awake
- [x] **Live project logs** + Compose log tab (nested compose `-f`)
- [x] **Compose DB stack** — discover nested compose, Up/Stop DB services, peek/copy URI, Compass for mongo
- [x] **Port reconciliation** — detect occupant (catalog / docker / native), stop or remap; remaps in app-data + spawn env
- [x] **Resizable pane layout** — Cursor-style sidebar / inspector / logs splits; collapse + drag resize; layout in localStorage

---

## In Progress

- Nothing in progress. Next product work: **M3 — Glue polish**.

---

## Known Broken / Blocked

- Coffee lid-close mode requires administrator approval per enable/disable.
- Compose/stack needs Docker CLI on the login-shell PATH.
- Occupant matching relies on compose `working_dir` labels; unnamed containers may show as `dockerOther`.

---

## Where We Left Off

Shipped **resizable pane layout**: sidebar, inspector, and logs are drag-resizable and collapsible (⌘B / ⌘J); settings and scan are overlays; layout persists in localStorage. Next: M3 polish.

---

## Current Architecture State

```
src/
├── components/              ✅ ShellHeader, KeepAwakeToggle, panes/
├── features/projects/       ✅ list, detail, scan, settings, log viewer
├── features/docker/         ✅ StackPanel, PortConflictDialog
├── lib/                     ✅ api, logs, docker wrappers
└── types/                   ✅ Project, LogChunk, PortConflict, ProjectStack, …

src-tauri/src/
├── commands/                ✅ app + projects + logs + docker IPC
├── services/                ✅ Catalog, Process, Log, Compose, Occupancy, Stack, …
├── repositories/            ✅ Database v3 + port_overrides
├── domain/                  ✅ Project, Log, Stack types
├── error.rs                 ✅ AppError (+ port_conflict)
└── lib.rs                   ✅ setup, rehydrate, manage stack
```

---

## Environment & Config

```env
# No app .env required.
# Runtime: login shell PATH for npm/docker/etc.
# Docker Desktop / OrbStack required for stack Up and port reconcile via docker ps.
```

---

## How to Run Locally

```bash
npm install
npm run tauri:dev

npm run tauri:install

npm run build
cd src-tauri && cargo test && cargo check
```

---

## Tech Debt

- Default Tauri icons still in place
- Process log files grow until the next start
- Compose occupancy uses `docker ps` + `lsof` (not bollard) for login-shell PATH reliability
- Compose `--wait` depends on Compose v2 healthcheck support
- Native occupant stop requires an explicit confirm; still sharp-edged

---

## Open Questions

- Tray / menu-bar in M3 vs earlier if it helps daily use

---

## Quick Stats

| Metric | Value |
|---|---|
| Total sessions | 7 |
| Modules complete | M0 + M1 + M2 (+ Coffee + logs + pane layout) |
| Test coverage | 34 Rust unit tests |
| Last deployed | Local `~/Applications` via `tauri:install` |
