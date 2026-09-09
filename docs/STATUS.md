# STATUS.md
> Your weekend dashboard. Read this first. Update this last.
> Last updated: `2026-09-09` | Session: `#9`

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
- [x] **Snappy project switch** — select paints from memory; compose peek without Docker; running dots refresh in background; cached `docker` binary
- [x] **Open in Finder / Cursor** — inspector buttons via `OpenService`
- [x] **Actionable errors** — `AppError::user_message()` for IPC + `last_error`
- [x] **Menu-bar tray** — show window / quit
- [x] **Public OSS path** — MIT, FUNDING.yml, stranger README, unsigned Release workflow, `site/` for zashiki.anireco.app

---

## In Progress

- Nothing in progress. Manual follow-ups: `wrangler deploy` + DNS for `zashiki.anireco.app`, push a `v*` tag for the first Release draft.

---

## Known Broken / Blocked

- Coffee lid-close mode requires administrator approval per enable/disable.
- Compose/stack needs Docker CLI on the login-shell PATH (resolved once at startup).
- Occupant matching relies on compose `working_dir` labels; unnamed containers may show as `dockerOther`.
- macOS Releases are **unsigned** — Gatekeeper needs right-click Open or `xattr -dr com.apple.quarantine`.

---

## Where We Left Off

Shipped **M3 glue + OSS launch path**: Finder/Cursor open, user-facing errors, tray, MIT, GitHub Release workflow, Cloudflare Workers site folder. Landing screenshot is in `site/assets/screenshot.png`. Next: deploy site + DNS, tag a release.

---

## Current Architecture State

```
src/
├── components/              ✅ ShellHeader, KeepAwakeToggle, panes/
├── features/projects/       ✅ list, detail (+ Finder/Cursor), scan, settings, log viewer
├── features/docker/         ✅ StackPanel, PortConflictDialog
├── lib/                     ✅ api (+ open), logs, docker wrappers
└── types/

src-tauri/src/
├── commands/                ✅ app + projects (+ open) + logs + docker
├── services/                ✅ Catalog, Process, Log, Compose, docker_bin, Open, Occupancy, Stack
├── tray.rs                  ✅ menu-bar show / quit
├── repositories/            ✅ Database v3 + port_overrides
├── domain/                  ✅ Project, Log, Stack types
├── error.rs                 ✅ AppError + user_message mapper
└── lib.rs                   ✅ setup, rehydrate, warm docker bin, tray, manage stack

site/                        ✅ Workers static assets landing page
.github/workflows/release.yml ✅ unsigned macOS app+dmg on v* tags
```

---

## Environment & Config

```env
# No app .env required.
# Runtime: login shell PATH for npm/docker/etc.
# Docker Desktop / OrbStack required for stack Up and port reconcile via docker ps.
# docker binary is resolved once at startup via login shell, then invoked directly.
```

---

## How to Run Locally

```bash
npm install
npm run tauri:dev

npm run tauri:install

npm run build
cd src-tauri && cargo test && cargo check

# Landing page
cd site && npx wrangler deploy
```

---

## Tech Debt

- Default Tauri icons still in place
- Process log files grow until the next start
- Compose occupancy uses `docker ps` + `lsof` (not bollard) for login-shell PATH reliability
- Compose `--wait` depends on Compose v2 healthcheck support
- Native occupant stop requires an explicit confirm; still sharp-edged
- ANSI log paint caps at last 400 lines for first paint; full virtualizer not yet needed
- Apple notarization / Homebrew cask deferred

---

## Open Questions

- When to spend the Apple Developer $99 for notarization

---

## Quick Stats

| Metric | Value |
|---|---|
| Total sessions | 9 |
| Modules complete | M0 + M1 + M2 + M3 (+ Coffee + logs + panes + snappy + OSS path) |
| Test coverage | 44 Rust unit tests |
| Last deployed | Local `~/Applications` via `tauri:install` (rebuild to pick up M3) |
