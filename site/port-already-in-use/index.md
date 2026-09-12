# Port already in use

> EADDRINUSE and Docker ‘port is already allocated’ on macOS — how Zashiki Warashi detects catalog, Docker, or native occupants and lets you stop or remap.

Canonical: https://zashiki.anireco.app/port-already-in-use/

`Error: listen EADDRINUSE: address already in use :::3000` is not one problem. It is three:

1. **You** left last night’s `npm run dev` running (maybe from another AI-generated folder).
2. **Docker** still publishes that host port from a compose stack.
3. Something **native** (another tool, AirPlay on 5000, a rogue process) holds the socket.

Blind `kill -9 $(lsof -ti :3000)` works until the owner is `docker-proxy` — then you stopped nothing useful — or until you kill the wrong database.

## What Zashiki Warashi does

When a start or stack action hits a conflict, the app tries to **name the occupant**:

| Occupant kind | Typical meaning | Reconciliation |
| --- | --- | --- |
| Catalog project | Another (or same) project Zashiki knows | Stop that project, or remap |
| Docker | Container / compose publish | Stop stack service / container path, or remap |
| Native | Host process outside the catalog | Confirm before stop, or remap |

Remaps are stored in **app data** and applied via spawn environment — so you can keep two projects alive without immediately editing every compose file. Compose file edits remain your choice when you want them permanent.

## Why AI project piles make this worse

Vibe-coded apps love default ports: `3000`, `5173`, `8000`, `5432`, `27017`. Ten Cursor sessions later, Monday is a port lottery. A [project catalog with lifecycle](/for-ai-projects/) plus occupant-aware reconciliation beats a shell alias you forget under stress.

## Manual fallback (still useful)

When you are outside the app:

```bash
lsof -i :3000 -sTCP:LISTEN
docker ps --filter publish=3000
```

If `COMMAND` is `com.docke` / `docker-proxy`, stop the container or `docker compose down` in the right project — [Compose peek](/compose-databases/) helps you find which project that was.

Prefer SIGTERM before SIGKILL for anything that looks like a database.

## Process groups help the “my own leftover” case

Zashiki starts user commands in a **process group** and stops the group on Stop — so orphaned Node children are less likely to own the port after you thought you stopped. Details: [Start and stop](/start-stop/).

## Honest limits

Occupancy matching for Docker uses compose labels / working_dir signals where available. Unnamed or odd containers may show as a generic docker occupant. Native stop remains sharp-edged on purpose — confirm before killing.

Download the macOS app: [Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest).

## FAQ

### Is this only for Node EADDRINUSE?

The pain shows up as Node EADDRINUSE, Vite bind errors, or Docker ‘port is already allocated’. Reconciliation covers catalog, docker, and native occupants.

### Does remapping edit my compose file?

Remaps live in app data and spawn env for the project — observation first; it does not casually rewrite your repo.

### Will it kill system services?

Native stop requires an explicit confirm. Treat database and system listeners carefully; prefer remap when unsure.

## More rooms

- [Start and stop projects](https://zashiki.anireco.app/start-stop/)
- [Compose and databases](https://zashiki.anireco.app/compose-databases/)
- [For AI-generated projects](https://zashiki.anireco.app/for-ai-projects/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
