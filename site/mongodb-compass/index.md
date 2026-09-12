# Local MongoDB → Compass

> How Zashiki Warashi peeks a Compose Mongo service on macOS and opens MongoDB Compass with a derived connection URI — without storing passwords in the app database.

Canonical: https://zashiki.anireco.app/mongodb-compass/

MongoDB Compass is a fine place to inspect documents. It is a poor place to *discover* which compose service on which host port belongs to which throwaway project from last week.

**Zashiki Warashi** bridges that gap on macOS: catalog the project, peek the Compose stack, open Compass (or copy the URI).

## The happy path

1. Add the project folder (or land on it after a root scan).
2. Ensure Docker Desktop or OrbStack is running.
3. Open the project’s **Stack** pane — Mongo-like services show with connection fields derived from compose/env.
4. **Up** the DB service if it is not running.
5. Click **Open in Compass**, or **Copy URI** for another client.

No query GUI ships inside Zashiki. That is intentional: glue into Compass rather than reinvent it. Broader stack peek: [Compose and databases](/compose-databases/).

## How the URI is built

Fields come from what compose and env already declare — image cues for mongo, published ports, user, database name, password when present. The URI is **computed when you peek**, not loaded from a password vault in SQLite.

Product constraints:

- Do not persist passwords in the app database
- Mask secrets in the UI
- Prefer deep-link over reimplementing Compass

## When it fails

| Symptom | What to check |
| --- | --- |
| No mongo service listed | Compose file missing, nested deeper than discovery, or image not recognized as DB-like |
| Port busy | Another container or native process — [port already in use](/port-already-in-use/) |
| Compass does not open | Install Compass; or copy URI manually |
| Wrong credentials | Fix compose/env in the project; Zashiki does not invent auth |

## Related lifecycle

App processes (API, Next, etc.) still use [login-shell start/stop](/start-stop/). Mongo in Compose is stack lifecycle. Both live under one project selection so Monday morning is one house, not three Terminal tabs and a sticky note.

Download: [GitHub Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest). MIT · [repo](https://github.com/khushal-sahni/zashiki-warashi).

## FAQ

### Do I need Compass installed?

Yes for the open action. Without Compass you can still copy the URI into any Mongo client.

### Does Zashiki include a query editor?

No. Compass (or another client) remains the query tool. Zashiki only peeks and deep-links.

### Will it work with OrbStack?

Yes if `docker compose` works on your login-shell PATH the same as with Docker Desktop.

## More rooms

- [Compose and databases](https://zashiki.anireco.app/compose-databases/)
- [Start and stop projects](https://zashiki.anireco.app/start-stop/)
- [Port already in use](https://zashiki.anireco.app/port-already-in-use/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
