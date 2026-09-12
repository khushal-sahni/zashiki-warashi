# vs OrbStack

> OrbStack replaces Docker Desktop as a fast macOS container runtime. Zashiki Warashi catalogs local projects and peeks Compose — it runs on top of OrbStack, not instead of it.

Canonical: https://zashiki.anireco.app/compare/orbstack/

## Short answer

**OrbStack** is a macOS-native container runtime — often chosen as a lighter Docker Desktop replacement. It speaks the Docker API and Compose.

**Zashiki Warashi** is a localhost **project control plane**: catalog, start/stop native project commands, peek DB-like Compose services, deep-link to tools you already have.

If your search was “OrbStack vs Docker Desktop,” that comparison is elsewhere. This page answers “where does Zashiki fit?” — **on top of OrbStack**, not against it.

## Side-by-side

| Concern | OrbStack | Zashiki Warashi |
| --- | --- | --- |
| Container runtime | Yes | No |
| Fast Docker-compatible API | Yes | Consumes via CLI |
| Multi-project sidebar for *repos* | Container/project grouping ≠ app catalog | Machine-local project catalog |
| Login-shell `npm` / `fnm` start | Not the product | Yes |
| Compass deep-link from compose mongo | Manual | Built-in peek path |
| Resource usage of the engine | OrbStack’s win | N/A |

## Recommended pairing

1. Install **OrbStack** (or Docker Desktop) for containers.
2. Install **Zashiki** from [Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest).
3. Catalog repos; use Stack peek for DBs; use Start/Stop for app processes.

See also [vs Docker Desktop](/compare/docker-desktop/) and [Compose and databases](/compose-databases/).

## FAQ

### Does Zashiki embed OrbStack?

No. Install OrbStack (or Docker Desktop) separately. Zashiki shells out to docker compose using a cached binary path.

### Is OrbStack required?

Any engine that provides a working docker compose on your PATH is fine. OrbStack is a common choice on macOS.

### Will Zashiki manage OrbStack VMs?

No. VM and engine settings stay in OrbStack.

## More rooms

- [vs Docker Desktop](https://zashiki.anireco.app/compare/docker-desktop/)
- [Compose and databases](https://zashiki.anireco.app/compose-databases/)
- [Start and stop projects](https://zashiki.anireco.app/start-stop/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
