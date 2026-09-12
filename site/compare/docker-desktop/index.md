# vs Docker Desktop

> Docker Desktop is a container runtime and GUI. Zashiki Warashi is a macOS project catalog with start/stop and Compose DB peek. Not a Docker Desktop alternative — complementary glue.

Canonical: https://zashiki.anireco.app/compare/docker-desktop/

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

## FAQ

### Can Zashiki replace Docker Desktop?

No. It does not run a container engine. You still need Docker Desktop, OrbStack, or another engine for Compose.

### Do I need Docker Desktop if I only run npm?

Not for plain Node projects. You need an engine when you use Compose stacks / DB peek.

### Why does this page exist then?

Because ‘Docker alternative’ searches often mean ‘I’m drowning in local projects and containers.’ We disambiguate so you pick the right layer.

## More rooms

- [vs OrbStack](https://zashiki.anireco.app/compare/orbstack/)
- [Compose and databases](https://zashiki.anireco.app/compose-databases/)
- [Localhost control plane](https://zashiki.anireco.app/localhost-control-plane/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
