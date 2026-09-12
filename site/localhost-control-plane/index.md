# A control plane for localhost

> Zashiki Warashi is a macOS control plane for local projects: catalog, start/stop, Compose/DB peek, and deep-links — not a Docker GUI, not Raycast, not a folder bookmark.

Canonical: https://zashiki.anireco.app/localhost-control-plane/

“Control plane” usually shows up in Kubernetes blogs. On a single developer Mac, the idea is simpler: **one surface that knows what is local, what is running, and how to start or stop it** — without replacing the tools that actually edit code, run containers, or query databases.

**Zashiki Warashi** is that surface for macOS localhost.

## The jobs a control plane should own

| Job | What Zashiki does |
| --- | --- |
| Remember projects | Machine-local catalog: add folder, scan roots, search, remove |
| Operate processes | Start / stop / restart via login shell; process groups; PID rehydrate |
| Peek stacks | Compose DB services, host/port/user/db, copy URI |
| Hand off | Open in Finder, Cursor, Compass for mongo |
| Reduce collisions | Port occupant detection — catalog, docker, or native |

It deliberately does **not** own: editing code, full Docker Engine UI, query GUIs, cloud sync, or required per-repo config.

## Why not “just a folder bookmark”?

Bookmarks answer *where*. They do not answer *is it running*, *what start command*, *which compose file is nested two levels down*, or *who owns port 5432*.

Raycast, Alfred, and Spotlight are excellent launchers. They are not process-group-aware stop buttons with Compose peek and Compass deep-links. Zashiki is the house that keeps those operations together.

## Why not a Docker GUI?

Docker Desktop, OrbStack, Dockev, Portainer, and Compose Launcher solve container-centric problems. Many local projects are mostly `npm run dev` with an optional compose file for Postgres. Treating every repo as a Docker project forces the wrong abstraction.

Zashiki sits **beside** the engine: use Docker Desktop or OrbStack for containers; use Zashiki for the project catalog and lifecycle. Honest comparisons: [vs Docker Desktop](/compare/docker-desktop/), [vs OrbStack](/compare/orbstack/), [vs Portainer](/compare/portainer/).

## Category language we use on purpose

- **Localhost control plane** — operate local projects as a fleet of one machine.
- **House spirit** — named after 座敷童子; look after the place without owning every room. See [The name](/name/).
- **Observe, don’t tax** — no mandatory `zashiki.toml` in every repo.

If you searched for “manage multiple local projects macOS,” “localhost project manager,” or “start stop all my apps,” this is the category we mean — especially when those apps came from [AI-assisted coding](/for-ai-projects/).

## What success looks like on Monday

Open the app. Find the project. Start it. See Compose services and a DB connection string. Copy the URI or open Compass. Stop when you are done. No spelunking Docker Engine by hand for the common path.

Download: [GitHub Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest). MIT.

## FAQ

### How is this different from Raycast or Spotlight?

Launchers open things. A control plane remembers project state, starts and stops processes, and peeks stack endpoints. Zashiki keeps a catalog and lifecycle, not only shortcuts.

### Do I need cloud accounts?

No. Catalog and overrides stay on your Mac. No sync, no login.

### Does it replace Docker Desktop?

No. Docker Desktop or OrbStack remain the engine for containers. Zashiki peeks and drives compose for DB-like services as glue.

## More rooms

- [For AI-generated projects](https://zashiki.anireco.app/for-ai-projects/)
- [Start and stop projects](https://zashiki.anireco.app/start-stop/)
- [Compose and databases](https://zashiki.anireco.app/compose-databases/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
