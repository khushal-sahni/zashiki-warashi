# STATUS.md
> Your weekend dashboard. Read this first. Update this last.
> Last updated: `2026-08-25` | Session: `#1`

---

## What Works Right Now

- [x] Product vision and architecture locked in docs (`AGENTS.md`, roadmap, decisions)
- [x] Tauri 2 + React + Vite + TypeScript scaffold
- [x] Rust layered backend (`commands` / `services` / `repositories` / `domain`)
- [x] SQLite app-data DB with schema v1 (`projects`, `project_runs`, `meta`)
- [x] Typed `AppError` + `tauri-plugin-log` / tracing
- [x] Empty shell UI that loads `get_app_status` (DB ready + app data path)
- [x] README with run instructions

---

## In Progress

- Nothing in progress — M0 complete. Next is M1 (catalog + lifecycle).

---

## Known Broken / Blocked

- None known. `tauri:dev` launched successfully (Vite + Rust binary). Manual click-through of the shell UI not exhaustively exercised.

---

## Where We Left Off

M0 foundation is in place. Next session should start **M1 — Catalog + lifecycle**: register/scan projects, infer start commands, start/stop with login shell + process groups, PID persistence and status rehydration.

---

## Current Architecture State

```
src/
├── components/          ✅ ShellHeader
├── features/projects/   ❌ placeholder
├── features/docker/     ❌ placeholder
├── lib/                 ✅ getAppStatus invoke wrapper
└── types/               ✅ AppStatus

src-tauri/src/
├── commands/            ✅ get_app_status
├── services/            ✅ AppService
├── repositories/        ✅ Database (rusqlite + migrate)
├── domain/              ✅ AppStatus
├── error.rs             ✅ AppError
└── lib.rs               ✅ setup + plugins
```

---

## Environment & Config

```env
# No app .env required for M0.
# Runtime needs (later): Docker CLI available on PATH via login shell.
```

---

## How to Run Locally

```bash
npm install
npm run tauri:dev

# Frontend only
npm run dev

# Rust checks
cd src-tauri && cargo test && cargo check
```

---

## Tech Debt

- Default Tauri icons still in place — replace with brand assets later
- `projects` / `project_runs` tables exist but have no repository/service API yet (intentional for M1)

---

## Open Questions

- Tray / menu-bar in M3 vs earlier if it helps daily use

---

## Quick Stats

| Metric | Value |
|---|---|
| Total sessions | 1 |
| Modules complete | M0 foundation |
| Test coverage | DB migrate unit test |
| Last deployed | Never |
