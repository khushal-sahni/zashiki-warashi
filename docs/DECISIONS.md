# DECISIONS.md
> Architectural decisions and why they were made.
> The "why" is what you'll forget. Write it down.

---

## Template

Use this format for each decision:

```
### [Short title]
- **Date**: YYYY-MM-DD
- **Status**: Accepted | Superseded by [link] | Under discussion
- **Context**: What situation forced this decision?
- **Decision**: What did we decide?
- **Consequences**: What does this make easier or harder going forward?
```

---

## Decisions

### Product is a localhost control plane, not a framework
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: AI-assisted coding produces many local projects that are hard to remember and operate. Temptation is to invent a new "dev platform" YAML every repo must adopt.
- **Decision**: Observe existing projects; keep a machine-local catalog. Infer start commands and compose/DB info. Do not require per-repo config. Optional `zashiki.toml` export may come later as opt-in only.
- **Consequences**: Works with existing repos immediately. Less control over exotic setups — overrides live in the app, not in the repo.

### Tauri 2 over Electron
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: Need a native desktop control plane with start/stop buttons and OS integration. Electron would keep everything in TypeScript; Tauri is lighter and more native.
- **Decision**: Use Tauri 2 + React/Vite/TypeScript UI with a Rust backend for FS, processes, Docker, and SQLite.
- **Consequences**: Smaller/native binary and better OS fit. Dual-language codebase; AI sessions must respect Rust layer rules. Nest/Express template assumptions in the old AGENTS.md are obsolete.

### Layered architecture mapped to Tauri (not HTTP)
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: Previous AGENTS.md assumed class-based Nest/Express with controllers/services/repositories. That does not fit a desktop IPC app.
- **Decision**: Keep strict layers: React → typed invoke → `commands/` → `services/` → `repositories/`. No business logic in UI or commands. Domain types in `domain/`.
- **Consequences**: Predictable places for new logic. Slightly more boilerplate per feature.

### Compose CLI + bollard; do not reinvent Docker
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: Users already have Docker Desktop / OrbStack. Rebuilding Compose or a full container GUI is out of scope.
- **Decision**: Use bollard for Engine API list/inspect; shell out to `docker compose` for up/down/ps. Deep-link to Compass and friends for DB work.
- **Consequences**: Depends on Docker CLI being installed and on PATH via the login shell. No custom Compose parser beyond what's needed for DB peek.

### Login-shell spawn for project commands
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: GUI apps on macOS often lack the user's interactive PATH (`fnm`, `nvm`, Homebrew Docker).
- **Decision**: Run project start/stop commands via `$SHELL -lc` so the user's normal environment applies.
- **Consequences**: Slightly slower spawn; behavior matches Terminal. Edge cases if `$SHELL` is unusual.

### Processes outlive the app; PID + process-group stop
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: Closing the control plane should not kill running projects. `npm run dev` and similar spawn children that SIGTERM to the parent alone will miss.
- **Decision**: Start in a new process group; persist pid + start time; rehydrate status on launch; stop with SIGTERM to the group then SIGKILL.
- **Consequences**: Correct lifecycle for common Node tooling. Must carefully validate PID identity on rehydrate.

### macOS-first
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: Primary user is on Darwin; process groups, PATH, and deep-links differ on Linux/Windows.
- **Decision**: Ship and polish for macOS first. Linux/Windows are M4+.
- **Consequences**: Faster v1. Platform-specific code will need abstraction later.

### No secrets in SQLite
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: DB peek will surface connection URIs that may include passwords from compose/env.
- **Decision**: Derive URIs at runtime from project files/env; do not persist passwords; mask in UI.
- **Consequences**: Re-parse on each peek. Safer if app data is synced or inspected.

### V1 includes catalog + lifecycle + Docker/DB peek (no provisioning)
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: Provisioning local DBs is desirable but large; the painful daily loop is remembering projects and peeking into existing containers.
- **Decision**: V1 = catalog + start/stop/status + compose/DB peek + deep-links. Provisioning and port maps are later. Logs pane was later pulled forward (see 2026-09-03).
- **Consequences**: Clear scope cut. Users still need Docker/Compose already set up for DB projects.

### Coffee keep-awake wraps macOS native tools (no third-party app)
- **Date**: 2026-08-30
- **Status**: Accepted
- **Context**: Local projects must keep running when the MacBook lid closes; network drops on sleep. Amphetamine/StayAwake/etc. all use the same public levers (`caffeinate`, `pmset disablesleep`).
- **Decision**: Add a toolbar Coffee toggle backed by `KeepAwakeService`: `caffeinate -ims` for idle sleep + `osascript`/`pmset -a disablesleep` for lid-close. Persist preference in SQLite meta; rehydrate caffeinate on launch. Coffee outlives the app (like project processes).
- **Consequences**: No external dependency. Lid-close requires admin approval per toggle (unless user adds their own sudoers rule). Heat/battery risk when lid closed — UI warns user. Does not fix Wi-Fi antenna-in-lid hardware limits.

### SQLite via rusqlite (bundled)
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: Need local persistence for catalog and run state without a server database. sqlx vs rusqlite were both viable.
- **Decision**: Use `rusqlite` with the `bundled` feature for sync access from Tauri managed state.
- **Consequences**: Simple setup, no separate system SQLite dependency. If we later need heavy async DB work, revisit sqlx.

### Process identity via pid + pgid
- **Date**: 2026-08-25
- **Status**: Accepted
- **Context**: PID reuse after reboot/exit can make “pid is alive” alone unsafe for rehydrate.
- **Decision**: Persist pid and pgid; on rehydrate require `kill(pid,0)` success and `getpgid(pid) == stored_pgid`. Stop signals the process group.
- **Consequences**: Safer reopen behavior on macOS. Windows will need a different identity strategy later.

### Daily driver via local `~/Applications` install (not auto on tauri:dev)
- **Date**: 2026-08-26
- **Status**: Accepted
- **Context**: Want Zashiki available from Spotlight/Dock like a normal app, while keeping `tauri:dev` for coding. Hooking install into every `tauri:dev` would be slow and still would not reflect mid-session edits.
- **Decision**: Separate `npm run tauri:install` builds a release `.app` and copies it to `~/Applications`. Dev and installed app share `com.zashiki.warashi` app data; do not run both at once.
- **Consequences**: Fast coding loop unchanged. Daily driver is refreshed on demand. No Apple signing/notarization for personal local use yet.

### Project logs via app-data files + Tauri events
- **Date**: 2026-09-03
- **Status**: Accepted
- **Context**: Started processes discarded stdout/stderr. Pipes die when the app closes, but processes must outlive the app.
- **Decision**: Redirect child stdio to `{app_data_dir}/logs/{project_id}/current.log`. Rotate that file on each start. An in-app tailer emits `project-log` events. Never write logs into project repos.
- **Consequences**: Output survives app restart for still-running processes. Failed boots are visible in the last session file. Files can grow until the next start.

### Compose logs are CLI snapshots, not Docker peek
- **Date**: 2026-09-03
- **Status**: Accepted
- **Context**: Want compose output in the log pane without building M2 (bollard, service list, up/down).
- **Decision**: If compose files exist, a Compose tab runs `docker compose logs --no-color --tail N` via the login shell and the UI may poll while Follow is on. Process capture still covers `docker compose up` stdout.
- **Consequences**: No extra follow process to leak. Depends on Docker CLI. Not a substitute for M2 inspect/peek.

### Nested compose + DB lifecycle on Start
- **Date**: 2026-09-03
- **Status**: Accepted
- **Context**: Repos like aurum keep compose under `local/`; starting Nest without Postgres yields confusing auth errors against whoever owns 5432.
- **Decision**: Discover compose at root and depth 2. On Start, `docker compose -f … up -d --wait` only DB-like services. Stop app does not stop DBs. Stack panel offers explicit Up/Stop.
- **Consequences**: Monday start path includes databases. Non-DB compose services are not auto-started.

### Port remaps live in Zashiki by default
- **Date**: 2026-09-03
- **Status**: Accepted
- **Context**: AI-generated compose files collide on 5432/27017/6379 across catalog projects.
- **Decision**: On conflict, UI offers stop-occupant or remap. Remap writes `{app_data}/compose-overrides/{id}.yml` + SQLite `port_overrides`, and injects rewritten `DATABASE_URL` (etc.) at spawn. Optional checkbox writes into the repo `.env`/compose.
- **Consequences**: Terminal may still see stale ports unless the user opts into repo writes. Occupancy uses `lsof` + `docker ps` (login-shell PATH) rather than bollard for v1.
