# STATUS.md
> Your weekend dashboard. Read this first. Update this last.
> Last updated: `2026-08-25` | Session: `#2`

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

---

## In Progress

- Nothing in progress — M1 complete. Next is M2 (Docker/DB peek).

---

## Known Broken / Blocked

- None known. Process stdout/stderr discarded (no logs pane until M4+).

---

## Where We Left Off

M1 catalog + lifecycle is implemented and tested. Next session: **M2 — Docker/DB peek** (compose detect, container list, up/down, URI + Compass deep-link).

---

## Current Architecture State

```
src/
├── components/              ✅ ShellHeader
├── features/projects/       ✅ list, detail, scan, settings
├── features/docker/         ❌ placeholder (M2)
├── lib/                     ✅ typed invoke wrappers
└── types/                   ✅ AppStatus, Project, …

src-tauri/src/
├── commands/                ✅ app + projects IPC
├── services/                ✅ App, Catalog, Process, infer
├── repositories/            ✅ Database + ProjectRepository
├── domain/                  ✅ AppStatus, Project, RunState, …
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

# Checks
npm run build
cd src-tauri && cargo test && cargo check
```

---

## Tech Debt

- Default Tauri icons still in place
- Start/stop discard process output (add log files when building logs pane)
- `now_iso` stores unix seconds as string — fine for identity, not pretty for UI

---

## Open Questions

- Tray / menu-bar in M3 vs earlier if it helps daily use

---

## Quick Stats

| Metric | Value |
|---|---|
| Total sessions | 2 |
| Modules complete | M0 + M1 |
| Test coverage | 9 Rust unit tests |
| Last deployed | Never |
