# vs Portainer

> Portainer is a Docker/container management UI (often remote). Zashiki Warashi is a macOS-local project catalog with process start/stop and Compose DB peek — not a Portainer replacement.

Canonical: https://zashiki.anireco.app/compare/portainer/

## Short answer

**Portainer** is a mature **Docker / container management** UI — stacks, environments, often remote endpoints, team-oriented Docker ops.

**Zashiki Warashi** is a **macOS localhost control plane** for developer projects: catalog folders, start/stop app processes via login shell, peek Compose DB endpoints, open Cursor / Compass.

Zashiki is **not** a Portainer alternative. If you need Portainer’s job, use Portainer (or Docker Desktop / OrbStack GUIs). If you need a project house, use Zashiki — optionally with those engines underneath.

## Side-by-side

| Concern | Portainer | Zashiki Warashi |
| --- | --- | --- |
| Primary object | Containers / stacks / environments | Local project folders |
| Remote Docker | Core strength | Out of scope (v1) |
| `npm run dev` / fnm PATH | Not the product | Login-shell lifecycle |
| DB URI copy + Compass | Manual / external | Built-in peek path |
| RBAC / multi-user | Portainer’s world | Single machine, no accounts |
| Required project config | Compose for stacks | No required per-repo YAML |

## Choose Portainer when

- You administer Docker environments (local or remote) as the main job
- You need broad container, volume, network, and stack operations in a UI
- Multiple hosts or team access patterns matter

## Choose Zashiki when

- The mess is **repos and processes** on one Mac — especially [AI-generated piles](/for-ai-projects/)
- Compose exists mainly to run databases beside an app you start with npm/cargo/go
- You want [port reconciliation](/port-already-in-use/) across catalog and docker occupants

Also read [vs Docker Desktop](/compare/docker-desktop/) and [Compose and databases](/compose-databases/).

Download: [GitHub Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest). MIT.

## FAQ

### Can Zashiki manage remote Docker hosts?

Not in v1. It is machine-local. Remote/SSH hosts are future ideas, not current scope.

### Does Zashiki edit Compose YAML visually?

No. Peek and up/stop for DB-like services; editing stays in your editor.

### I only need local compose up/down — is Portainer overkill?

Often yes. If your pain is multi-repo localhost plus occasional compose DBs, a project house may fit better than a full Docker UI.

## More rooms

- [Compose and databases](https://zashiki.anireco.app/compose-databases/)
- [vs Docker Desktop](https://zashiki.anireco.app/compare/docker-desktop/)
- [Localhost control plane](https://zashiki.anireco.app/localhost-control-plane/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
