# Zashiki Warashi

**A macOS control plane for the localhost pile — especially AI-generated repos.** It remembers projects, one-button start/stop, peeks Compose/DB endpoints, and deep-links into Cursor / Finder / Compass.

It is not a Docker GUI, not Portainer/Dockge, not a Raycast launcher, and not a folder bookmark. It glues tools you already have.

Named after the 座敷童子 — a house spirit that looks after the place.

Site: [zashiki.anireco.app](https://zashiki.anireco.app)

## Install (macOS)

1. Download the latest **`.dmg`** from [Releases](https://github.com/khushal-sahni/zashiki-warashi/releases).
2. Open the disk image and drag **Zashiki Warashi** to Applications (or copy the `.app` wherever you like).
3. **Gatekeeper (unsigned builds):** first launch may be blocked.
   - Right-click the app → **Open** → confirm, **or**
   - `xattr -dr com.apple.quarantine "/Applications/Zashiki Warashi.app"`
4. Open from Spotlight, Dock, or Launchpad.

Builds are **not notarized** yet. That is intentional for this release path.

### What you need

- macOS (primary target)
- [Docker Desktop](https://www.docker.com/products/docker-desktop/) or [OrbStack](https://orbstack.dev/) if you use Compose / DB stacks
- [Cursor](https://cursor.com/) optional (for **Open in Cursor**)

### First use

1. **Add** a project folder, or set scan roots in Settings and **Scan**.
2. Select a project → **Start** / **Stop** / **Restart**.
3. Use **Finder** / **Cursor** to open the folder.
4. If Compose is present, the **Stack** panel peeks DB services, Up/Stop, copy URI, and open Compass for Mongo.
5. Port conflicts offer stop-occupant or remap (stored in app data by default).

Menu bar tray: click the icon (or **Show Zashiki Warashi**) to focus the window; **Quit** exits.

## Develop from source

For contributors and local hacking:

```bash
npm install
npm run tauri:dev
```

Prerequisites: Node.js 20+, Rust (stable) via [rustup](https://rustup.rs/), Xcode Command Line Tools.

Daily-driver install from a local checkout:

```bash
npm run tauri:install
```

Builds a release `.app` into `~/Applications/Zashiki Warashi.app`. Do **not** run the installed app and `tauri:dev` at the same time — they share the same SQLite catalog.

```bash
npm run tauri:build   # bundle under src-tauri/target/release/bundle/
npm run build && cd src-tauri && cargo test
```

## Stack

- **UI**: Tauri 2 + React + Vite + TypeScript
- **Backend**: Rust (FS, processes, Docker, SQLite)
- **Persistence**: SQLite in the app data directory (machine-local catalog; no cloud sync)

## Docs

- [AGENTS.md](./AGENTS.md) — architecture and coding rules
- [docs/ROADMAP.md](./docs/ROADMAP.md) — milestones and non-goals
- [docs/STATUS.md](./docs/STATUS.md) — current state
- [docs/DECISIONS.md](./docs/DECISIONS.md) — architectural decisions

## License & support

MIT — see [LICENSE](./LICENSE).

Optional: [GitHub Sponsors](https://github.com/sponsors/khushal-sahni). Stars and issues help more than anything.
