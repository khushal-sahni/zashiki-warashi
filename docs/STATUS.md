# STATUS.md
> Your weekend dashboard. Read this first. Update this last.
> Last updated: `2026-08-30` | Session: `#4`

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
- [x] **Coffee toggle** — toolbar keep-awake via `caffeinate` + `pmset disablesleep` (lid-close; admin prompt)

---

## In Progress

- Nothing in progress. Next product work: **M2 — Docker/DB peek**.

---

## Known Broken / Blocked

- None known. Process stdout/stderr discarded (no logs pane until M4+).
- Coffee lid-close mode requires one-time administrator approval per enable/disable (osascript + `pmset`).

---

## Where We Left Off

Added **Coffee** toolbar toggle: wraps macOS `caffeinate -ims` (idle sleep) and `pmset disablesleep` (lid close). Preference + caffeinate PID persist in SQLite; rehydrates on launch. Next: **M2 — Docker/DB peek**.

---

## Current Architecture State

```
src/
├── components/              ✅ ShellHeader, KeepAwakeToggle
├── features/projects/       ✅ list, detail, scan, settings
├── features/docker/         ❌ placeholder (M2)
├── lib/                     ✅ typed invoke wrappers (+ keep-awake)
└── types/                   ✅ AppStatus, KeepAwakeStatus, Project, …

src-tauri/src/
├── commands/                ✅ app (+ keep-awake) + projects IPC
├── services/                ✅ App, Catalog, Process, KeepAwake, infer
├── repositories/            ✅ Database + ProjectRepository (+ meta keys)
├── domain/                  ✅ AppStatus, KeepAwakeStatus, Project, …
├── error.rs                 ✅ AppError (+ conflict/invalid)
└── lib.rs                   ✅ setup, rehydrate, plugins
```

---

## Environment & Config

```env
# No app .env required.
# Runtime: login shell PATH for npm/docker/etc when starting projects.
# Optional later: Docker CLI for M2.
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
- Start/stop discard process output (add log files when building logs pane)
- `now_iso` stores unix seconds as string — fine for identity, not pretty for UI
- Coffee: no battery-floor auto-off or timed sessions yet; `pmset disablesleep` is sticky until toggled off

---

## Open Questions

- Tray / menu-bar in M3 vs earlier if it helps daily use

---

## Quick Stats

| Metric | Value |
|---|---|
| Total sessions | 4 |
| Modules complete | M0 + M1 (+ Coffee) |
| Test coverage | 14 Rust unit tests |
| Last deployed | Local `~/Applications` via `tauri:install` |
