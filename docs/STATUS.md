# STATUS.md
> Your weekend dashboard. Read this first. Update this last.
> Last updated: `2026-10-07` | Session: `#16`

---

## What Works Right Now

- [x] Product vision and architecture locked in docs
- [x] Tauri 2 + React + Vite + TypeScript scaffold
- [x] SQLite app-data DB (schema v4 — jobs + job runs; WAL so the app and job runner share it)
- [x] Typed `AppError` + logging (+ `port_conflict` payload)
- [x] **Project catalog** — add folder, scan roots, list/search, remove (catalog only)
- [x] **Start command inference** — package.json / compose / Makefile / cargo / go
- [x] **Command overrides** stored in app DB (not written into repos)
- [x] **Lifecycle** — start / stop / restart via login shell + process groups
- [x] **PID + pgid persistence** and rehydrate on launch
- [x] Catalog UI (sidebar + detail + scan results + scan-root settings)
- [x] **Local install** — `npm run tauri:install` → `/Applications/Zashiki Warashi.app` (auth sheet when needed)
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
- [x] **Landing redesign** — Noren Threshold world (PRODUCT.md + DESIGN.md); static `site/` ready to deploy
- [x] **SEO rooms** — 12 intent pages + compare hub; markdown twins; `llms.txt` / `llms-full.txt`; sitemap; real 404; cache headers
- [x] **Scheduled jobs** — Jobs mode; per-job launchd agents + supervisor; network wait, keep-awake hold, max runtime, run history + logs; optional wake helper for on-time wakes; socket API + MCP server (`<app> job mcp`)

---

## In Progress

- Scheduled jobs: code complete, uncommitted. Needs a real-app check: launch the installed app, create a job, install the wake helper (admin prompt), and confirm a lid-closed wake on AC power.
- Manual follow-ups: `cd site && npm run build && npx wrangler deploy` + DNS for `zashiki.anireco.app`, push a `v*` tag for the first Release draft.

---

## Known Broken / Blocked

- Coffee lid-close mode requires administrator approval per enable/disable.
- Compose/stack needs Docker CLI on the login-shell PATH (resolved once at startup).
- Occupant matching relies on compose `working_dir` labels; unnamed containers may show as `dockerOther`.
- macOS Releases are **unsigned** — Gatekeeper needs right-click Open or `xattr -dr com.apple.quarantine`.
- Exact-time jobs only wake a closed-lid Mac on AC power (macOS limit); on battery they run on the next wake.

---

## Where We Left Off

Scheduled jobs (M5) built end to end and verified in the browser against a mocked backend, plus a CLI smoke test (`job run`, wake request file, socket + MCP). Not yet exercised: wake helper install and real launchd/pmset wakes. Still need `wrangler deploy` + DNS when ready for the site.

---

## Current Architecture State

```
src/                         ✅ app UI
src-tauri/                   ✅ backend
site/                        ✅ noren home + SEO rooms (build: npm run site:build)
PRODUCT.md                   ✅ product truth
DESIGN.md                    ✅ visual system from shipped site
```

---

## Environment & Config

```env
# No app .env required.
# Landing: cd site && npm run build && npx wrangler deploy
```

---

## How to Run Locally

```bash
npm install
npm run tauri:dev

# Landing + SEO rooms
cd site && npm install && npm run build
npm run site:preview   # http://127.0.0.1:8765
# Deploy
cd site && npm run build && npx wrangler deploy
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
- Landing hero is approved-comp raster + hotspots (not a fully semantic CSS reconstruction of every region)
- Unused intermediate plates under `site/assets/plates/` from the comp-led path
- SEO room body links use root-absolute `/slug/` paths (fine in prod; python preview matches)
- Generated HTML under `site/*/index.html` must be rebuilt after editing `site/content/`
- `login_shell.rs` duplicates process-group helpers from `process_service`
- Supervisor + job plists point at the current executable; `tauri:dev` repoints them at the dev binary until the installed app launches again
- Job run logs are never pruned

---

## Open Questions

- When to spend the Apple Developer $99 for notarization

---

## Quick Stats

| Metric | Value |
|---|---|
| Total sessions | 16 |
| Modules complete | M0 + M1 + M2 + M3 + M5 scheduled jobs (+ Coffee + logs + panes + snappy + OSS path + landing redesign + SEO rooms) |
| Test coverage | 76 Rust unit tests |
| Last deployed | Local `/Applications` via `tauri:install`; site SEO not yet wrangler-deployed |
| SEO rooms | 12 + compare hub |
