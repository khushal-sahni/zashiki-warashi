# STATUS.md
> Your weekend dashboard. Read this first. Update this last.
> Last updated: `2026-09-12` | Session: `#14`

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
- [x] **SEO rooms** — 12 intent pages + compare hub; markdown twins; `llms.txt` / `llms-full.txt`; sitemap; real 404; cache headers

---

## In Progress

- Nothing in progress. Manual follow-ups: `cd site && npm run build && npx wrangler deploy` + DNS for `zashiki.anireco.app`, push a `v*` tag for the first Release draft.

---

## Known Broken / Blocked

- Coffee lid-close mode requires administrator approval per enable/disable.
- Compose/stack needs Docker CLI on the login-shell PATH (resolved once at startup).
- Occupant matching relies on compose `working_dir` labels; unnamed containers may show as `dockerOther`.
- macOS Releases are **unsigned** — Gatekeeper needs right-click Open or `xattr -dr com.apple.quarantine`.

---

## Where We Left Off

SEO multi-room static site built under `site/` (markdown source → HTML). Previewed locally at `:8765`. Still need `wrangler deploy` + DNS when ready.

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

---

## Open Questions

- When to spend the Apple Developer $99 for notarization

---

## Quick Stats

| Metric | Value |
|---|---|
| Total sessions | 14 |
| Modules complete | M0 + M1 + M2 + M3 (+ Coffee + logs + panes + snappy + OSS path + landing redesign + SEO rooms) |
| Test coverage | 44 Rust unit tests |
| Last deployed | Local `~/Applications` via `tauri:install`; site SEO not yet wrangler-deployed |
| SEO rooms | 12 + compare hub |
