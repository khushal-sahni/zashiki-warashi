# STATUS.md
> Your weekend dashboard. Read this first. Update this last.
> Last updated: `2026-09-03` | Session: `#5`

---

## What Works Right Now

- [x] Product vision and architecture locked in docs
- [x] Tauri 2 + React + Vite + TypeScript scaffold
- [x] SQLite app-data DB (schema v2)
- [x] Typed `AppError` + logging
- [x] **Project catalog** — add folder, scan roots, list/search, remove (catalog only)
- [x] **Start command inference** — package.json / compose / Makefile / cargo / go
- [x] **Command overrides** stored in app DB (not written into repos)
- [x] **Lifecycle** — start / stop / restart via login shell + process groups
- [x] **PID + pgid persistence** and rehydrate on launch
- [x] Catalog UI (sidebar + detail + scan results + scan-root settings)
- [x] **Local install** — `npm run tauri:install` → `~/Applications/Zashiki Warashi.app`
- [x] **Coffee toggle** — compact toolbar keep-awake (hover hint, no toolbar reflow)
- [x] **Live project logs** — stdout/stderr captured to app-data files, log pane with follow/filter/wrap/copy/clear
- [x] **Compose logs** — snapshot via `docker compose logs` when compose files exist (not M2 peek)

---

## In Progress

- Nothing in progress. Next product work: **M2 — Docker/DB peek**.

---

## Known Broken / Blocked

- None known.
- Coffee lid-close mode requires one-time administrator approval per enable/disable (osascript + `pmset`).
- Compose log tab needs Docker CLI on the login-shell PATH.

---

## Where We Left Off

Added a **logs pane** on project detail: process output is redirected to `{app_data}/logs/{id}/current.log` (survives app restart), streamed over Tauri events, with a thin Compose source. Coffee toggle no longer expands the toolbar. Next: **M2 — Docker/DB peek**.

---

## Current Architecture State

```
src/
├── components/              ✅ ShellHeader, KeepAwakeToggle
├── features/projects/       ✅ list, detail, scan, settings, log viewer
├── features/docker/         ❌ placeholder (M2)
├── lib/                     ✅ typed invoke wrappers (+ keep-awake, logs)
└── types/                   ✅ AppStatus, KeepAwakeStatus, Project, LogChunk, …

src-tauri/src/
├── commands/                ✅ app + projects + logs IPC
├── services/                ✅ App, Catalog, Process, KeepAwake, Log, infer
├── repositories/            ✅ Database + ProjectRepository (+ meta keys)
├── domain/                  ✅ AppStatus, KeepAwakeStatus, Project, LogChunk, …
├── error.rs                 ✅ AppError (+ conflict/invalid)
└── lib.rs                   ✅ setup, rehydrate, log tailer, plugins
```

---

## Environment & Config

```env
# No app .env required.
# Runtime: login shell PATH for npm/docker/etc when starting projects.
# Optional later: Docker CLI for M2 (also used now for Compose log snapshots).
```

---

## How to Run Locally

```bash
npm install
npm run tauri:dev

# Daily driver (Spotlight / Dock) — re-run when you want the installed app updated
npm run tauri:install

# Checks
npm run build
cd src-tauri && cargo test && cargo check
```

---

## Tech Debt

- Default Tauri icons still in place
- Process log files grow until the next start (rotate on start only); no size cap / vacuum
- Compose logs are polled snapshots, not a live `docker compose logs -f` sidecar
- `now_iso` stores unix seconds as string — fine for identity, not pretty for UI
- Coffee: no battery-floor auto-off or timed sessions yet; `pmset disablesleep` is sticky until toggled off

---

## Open Questions

- Tray / menu-bar in M3 vs earlier if it helps daily use

---

## Quick Stats

| Metric | Value |
|---|---|
| Total sessions | 5 |
| Modules complete | M0 + M1 (+ Coffee + logs pane) |
| Test coverage | 22 Rust unit tests |
| Last deployed | Local `~/Applications` via `tauri:install` |
