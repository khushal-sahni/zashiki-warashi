# ROADMAP.md
> High-level direction. Not a sprint board — just where we're going.
> Keep this coarse. Fine-grained tasks live in the session conversation.

---

## Goal

**Zashiki-warashi** is a macOS-first desktop control plane for localhost: remember local projects, one-button start/stop, peek at Docker/DB endpoints, and deep-link into Cursor / Docker / Compass — without replacing those tools.

V1 is "useful on Monday" when you can open the app, find a project, start/stop it, see its compose services and DB connection info, and open Compass (or copy a URI) without spelunking Docker Engine by hand.

---

## Milestones

### M0 — Foundation `[x]`
> App scaffolds and runs locally as an empty shell

- [x] Tauri 2 + React + Vite + TypeScript project scaffolded
- [x] App data directory + SQLite wired
- [x] Typed `AppError` + logging in place
- [x] Empty shell UI loads in a window
- [x] README with how to run the app

### M1 — Catalog + lifecycle `[x]`
> Remember projects and start/stop them

- [x] Register / scan / list / remove projects
- [x] Infer stack signals and default start command (overridable)
- [x] Start / stop / restart via user login shell
- [x] Process-group aware stop; PID persistence; reopen-safe status rehydration
- [x] Status: stopped / starting / running / failed
- [x] Logs pane (pulled forward from M4+) — process capture + compose snapshots

### M2 — Docker/DB peek `[x]`
> Still part of v1 — glue, not a Docker GUI

- [x] Detect compose files for a project (root + depth 2)
- [x] List DB services / running state (`docker compose ps`)
- [x] Compose up/stop for DB-like services
- [x] Detect common DB images; show host/port/user/db from compose/env
- [x] Copy URI; open Compass for mongo
- [x] Port conflict reconciliation (stop occupant or remap)
- [x] No query GUI; no DB provisioning

### M3 — Glue polish `[x]`
> Someone else (or you next month) could actually use this

- [x] Open in Cursor / Finder
- [x] Actionable failure messages
- [x] Optional menu-bar / tray entry
- [x] README with setup and usage
- [x] Basic smoke tests for catalog + lifecycle

### M4+ — Future ideas `[ ]`
> Nice-to-haves; not blocking v1

- [ ] Provision local databases from scratch
- [x] Logs pane / lightweight log tail (pulled into M1+ as of 2026-09-03)
- [x] Port map / conflict hints (pulled into M2 as of 2026-09-03)
- [ ] Optional per-project `zashiki.toml` export (opt-in; not required)
- [ ] Linux / Windows support
- [ ] Env-file awareness beyond peek
- [ ] Remote / SSH hosts
- [ ] Apple notarization + Homebrew cask
- [ ] Polar / paid notarized builds (optional later)
---

## What We're NOT Building (Scope Cuts)

- No replacement for Docker Desktop / OrbStack UI
- No MongoDB Compass / TablePlus / query editor inside the app
- No AI coding agent
- No cloud sync, accounts, or auth
- No required per-repo YAML / framework adoption tax
- No writing into project repos by default
- No DB provisioning in v1
- No full terminal emulator
- No mobile app
- No i18n (for now)
