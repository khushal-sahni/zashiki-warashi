# Compose and database peek

> Peek Docker Compose DB services from a macOS project catalog: host, port, user, database, copy URI, open MongoDB Compass — without becoming a Docker GUI.

Canonical: https://zashiki.anireco.app/compose-databases/

Local apps often ship with a `docker-compose.yml` (or `compose.yaml`) that only exists to run Postgres, MySQL, Redis, or Mongo. You do not want a second career as a Docker Desktop power user just to copy a connection string.

**Zashiki Warashi** peeks that stack from the **project** you already cataloged.

## What “peek” means

For a selected project, the app:

1. **Discovers** Compose files (project root and common nested locations)
2. **Lists** DB-like services and whether they look running (`docker compose ps` via a cached docker binary)
3. Surfaces **host / port / user / database** derived from compose and env
4. Lets you **copy a URI** or **open MongoDB Compass** when the service is mongo
5. Offers **Up / Stop** for those DB-oriented services — not a full substitute for `docker compose` day-to-day ops

This is **observation + glue**, not a Compose YAML editor, not volume browsers, not remote Docker fleets. For that class of tool, see [vs Portainer](/compare/portainer/).

## Engine underneath

You still need **Docker Desktop** or **OrbStack** (or another engine that speaks `docker compose`). Zashiki does not ship a container runtime. Comparisons that stay honest:

- [vs Docker Desktop](/compare/docker-desktop/) — they are the engine; we are the project catalog
- [vs OrbStack](/compare/orbstack/) — we sit on top of OrbStack the same way

## Secrets stay out of the catalog DB

Connection URIs may include passwords from env files. Product rule: **derive at runtime, do not persist passwords in SQLite, mask in the UI**. Peek again when files change.

## How this fits the Monday workflow

1. Catalog the repo ([start/stop](/start-stop/))
2. Start the app process if needed
3. Up the DB services from the Stack pane
4. Copy URI into your `.env` consumer — or open [Compass](/mongodb-compass/) for mongo
5. If host ports collide with another project, use [port reconciliation](/port-already-in-use/)

## Nested compose

Real repos bury compose under `docker/` or `infra/`. Discovery walks a shallow depth and nested compose `-f` works for the log and stack paths the app supports. Exotic multi-file matrices may still need Terminal — the goal is the common path, not every Compose feature.

## What we are not building (on purpose)

- No Mongo / SQL query editor inside Zashiki
- No DB provisioning from scratch in v1
- No inventing a Docker GUI

Download: [Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest). MIT.

## FAQ

### Does this replace Portainer or Docker Desktop?

No. It peeks and drives compose for DB-like services as part of a project house. Full container management stays with your engine UI.

### Are database passwords stored in SQLite?

No. URIs are derived at runtime from compose/env. Passwords are not persisted; the UI masks secrets.

### How deep does compose discovery go?

Root plus nested compose within a shallow depth (including common nested layouts). Nested compose -f is supported for logs and stack actions.

## More rooms

- [MongoDB Compass](https://zashiki.anireco.app/mongodb-compass/)
- [Start and stop projects](https://zashiki.anireco.app/start-stop/)
- [vs Portainer](https://zashiki.anireco.app/compare/portainer/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
