# STATUS.md
> Your weekend dashboard. Read this first. Update this last.
> Last updated: `2026-09-09` | Session: `#10`

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
- [x] **Landing redesign** — Noren Threshold world (PRODUCT.md + DESIGN.md); static `site/` ready to deploy

---

## In Progress

- Nothing in progress. Manual follow-ups: `cd site && npx wrangler deploy` + DNS for `zashiki.anireco.app`, push a `v*` tag for the first Release draft.

---

## Known Broken / Blocked

- Coffee lid-close mode requires administrator approval per enable/disable.
- Compose/stack needs Docker CLI on the login-shell PATH (resolved once at startup).
- Occupant matching relies on compose `working_dir` labels; unnamed containers may show as `dockerOther`.
- macOS Releases are **unsigned** — Gatekeeper needs right-click Open or `xattr -dr com.apple.quarantine`.

---

## Where We Left Off

Redesigned the public landing (`site/`) under Impeccable: Noren Threshold / asymmetric reveal. Finish review disposition **ship**. Deploy with `wrangler deploy` when ready.

---

## Current Architecture State

```
src/                         ✅ app UI (unchanged this session)
src-tauri/                   ✅ backend (unchanged this session)
site/                        ✅ redesigned landing (Noren Threshold)
PRODUCT.md                   ✅ product truth
DESIGN.md                    ✅ visual system from shipped site
.impeccable/                 ✅ surfaces, mocks, build, design.json
```

---

## Environment & Config

```env
# No app .env required.
# Landing: cd site && npx wrangler deploy
```

---

## How to Run Locally

```bash
npm install
npm run tauri:dev

# Landing page preview
cd site && python3 -m http.server 8765
# Deploy
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
- Landing hero is approved-comp raster + hotspots (not a fully semantic CSS reconstruction of every region)
- Unused intermediate plates under `site/assets/plates/` from the comp-led path

---

## Open Questions

- When to spend the Apple Developer $99 for notarization

---

## Quick Stats

| Metric | Value |
|---|---|
| Total sessions | 10 |
| Modules complete | M0 + M1 + M2 + M3 (+ Coffee + logs + panes + snappy + OSS path + landing redesign) |
| Test coverage | 44 Rust unit tests |
| Last deployed | Local `~/Applications` via `tauri:install`; site redesign not yet wrangler-deployed |
