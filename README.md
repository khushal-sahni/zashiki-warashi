# Zashiki Warashi

A macOS-first desktop **control plane for localhost** — remember local (often AI-generated) projects, one-button start/stop, peek at Docker/DB endpoints, and deep-link into Cursor / Docker / Compass.

Named after the 座敷童子 — a house spirit that looks after the place.

## Stack

- **UI**: Tauri 2 + React + Vite + TypeScript
- **Backend**: Rust (FS, processes, Docker, SQLite)
- **Persistence**: SQLite in the app data directory

## Prerequisites

- Node.js 20+
- Rust (stable) via [rustup](https://rustup.rs/)
- macOS (primary target for v1)
- Xcode Command Line Tools

## Develop

```bash
npm install
npm run tauri:dev
```

This starts Vite and opens the Tauri window (hot reload). Use this while coding. You can add project folders, scan configured roots, override start/stop commands, and start/stop processes.

## Install (daily driver)

To use Zashiki like any other Mac app (Spotlight / Dock / Launchpad):

```bash
npm run tauri:install
```

This builds a release `.app` and installs it to `~/Applications/Zashiki Warashi.app`. Re-run when you want the installed app to catch up with your latest changes. Do **not** run the installed app and `tauri:dev` at the same time — they share the same SQLite catalog.

## Build

```bash
npm run tauri:build
```

Produces the release bundle under `src-tauri/target/release/bundle/` without installing it.

## Checks

```bash
npm run build
cd src-tauri && cargo test
```

## Docs

- [AGENTS.md](./AGENTS.md) — architecture and coding rules for humans and AI agents
- [docs/ROADMAP.md](./docs/ROADMAP.md) — milestones and non-goals
- [docs/STATUS.md](./docs/STATUS.md) — current state
- [docs/DECISIONS.md](./docs/DECISIONS.md) — architectural decisions

## What this is not

Not a Docker GUI, not a database client, not an AI coding agent. It glues tools you already have.
