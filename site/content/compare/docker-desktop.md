---
slug: compare/docker-desktop
title: "Zashiki Warashi vs Docker Desktop — honest comparison"
h1: "vs Docker Desktop"
description: "Docker Desktop is a container runtime and GUI. Zashiki Warashi is a macOS project catalog with start/stop and Compose DB peek. Not a Docker Desktop alternative — complementary glue."
lede: "If you searched for a Docker Desktop alternative, OrbStack or Colima may be what you want. Zashiki Warashi is a different tool: a localhost project house that sits on top of whatever engine you already run."
related:
  - /compare/orbstack/
  - /compose-databases/
  - /localhost-control-plane/
faq:
  - q: "Can Zashiki replace Docker Desktop?"
    a: "No. It does not run a container engine. You still need Docker Desktop, OrbStack, or another engine for Compose."
  - q: "Do I need Docker Desktop if I only run npm?"
    a: "Not for plain Node projects. You need an engine when you use Compose stacks / DB peek."
  - q: "Why does this page exist then?"
    a: "Because ‘Docker alternative’ searches often mean ‘I’m drowning in local projects and containers.’ We disambiguate so you pick the right layer."
---

## Short answer

**Docker Desktop** = container engine + Docker GUI on your Mac.

**Zashiki Warashi** = catalog of local *projects*, one-button process start/stop, Compose/DB peek, deep-links to Cursor / Finder / Compass.

Zashiki is **not** a Docker Desktop alternative. Calling it one would be false advertising. It **uses** `docker compose` when your projects have stacks.

## Side-by-side

| Concern | Docker Desktop | Zashiki Warashi |
| --- | --- | --- |
| Runs containers | Yes | No (needs an engine) |
| Project catalog across repos | Not the job | Yes |
| `npm run dev` lifecycle | Indirect | First-class via login shell |
| Compose DB URI → Compass | Manual | Peek + deep-link |
| Port conflict vs other *projects* | Container-focused | Catalog / docker / native occupants |
| Required per-repo YAML | Compose when you use it | None for the catalog |

## When to use which

- **Choose Docker Desktop** (or [OrbStack](/compare/orbstack/)) when you need the engine, Kubernetes bits, or a full container GUI.
- **Choose Zashiki** when the pain is “too many local repos / AI folders / leftover processes,” and Compose is only part of the story.
- **Use both** when Monday looks like: start the API from the catalog, up Postgres from Stack, open Compass, stop when done.

## Related honest pages

- [vs OrbStack](/compare/orbstack/)
- [vs Portainer](/compare/portainer/) — remote/full Docker UI vs machine-local project house
- [Compose and databases](/compose-databases/)

Download Zashiki: [Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest).
