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

### M1 — Catalog + lifecycle `[ ]`
> Remember projects and start/stop them

- [ ] Register / scan / list / remove projects
- [ ] Infer stack signals and default start command (overridable)
- [ ] Start / stop / restart via user login shell
- [ ] Process-group aware stop; PID persistence; reopen-safe status rehydration
- [ ] Status: stopped / starting / running / failed

### M2 — Docker/DB peek `[ ]`
> Still part of v1 — glue, not a Docker GUI

- [ ] Detect compose files for a project
- [ ] List services/containers (bollard + compose CLI)
- [ ] Compose up/down (or per-service start/stop)
- [ ] Detect common DB images; show host/port/user/db from compose/env
- [ ] Copy URI; open Compass / connection deep-link
- [ ] No query GUI; no DB provisioning

### M3 — Glue polish `[ ]`
> Someone else (or you next month) could actually use this

- [ ] Open in Cursor / Finder
- [ ] Actionable failure messages
- [ ] Optional menu-bar / tray entry
- [ ] README with setup and usage
- [ ] Basic smoke tests for catalog + lifecycle

### M4+ — Future ideas `[ ]`
> Nice-to-haves; not blocking v1

- [ ] Provision local databases from scratch
- [ ] Logs pane / lightweight log tail
- [ ] Port map / conflict hints
- [ ] Optional per-project `zashiki.toml` export (opt-in; not required)
- [ ] Linux / Windows support
- [ ] Env-file awareness beyond peek
- [ ] Remote / SSH hosts

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
