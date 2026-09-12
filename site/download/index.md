# Download for macOS

> Download Zashiki Warashi from GitHub Releases. Unsigned builds: Gatekeeper right-click Open or xattr quarantine. Needs Docker Desktop or OrbStack for Compose.

Canonical: https://zashiki.anireco.app/download/

## Get the app

**[Download the latest release](https://github.com/khushal-sahni/zashiki-warashi/releases/latest)** from GitHub.

Source, issues, and MIT license live in the [repository](https://github.com/khushal-sahni/zashiki-warashi). Optional support: [GitHub Sponsors](https://github.com/sponsors/khushal-sahni).

## First open on macOS (unsigned builds)

Releases are **not Apple-notarized** yet. Gatekeeper may block a double-click.

**Option A — right-click Open**

1. Open Finder → Applications (or wherever you placed the app)
2. Right-click **Zashiki Warashi** → **Open**
3. Confirm Open in the dialog

**Option B — clear quarantine**

```bash
xattr -dr com.apple.quarantine "/path/to/Zashiki Warashi.app"
```

Then open normally.

This friction is honest product debt, not a mystery. Notarization and Homebrew cask are deferred until the right time — not pretended away on the marketing site.

## What you need after install

| Need | Why |
| --- | --- |
| macOS | Primary supported platform |
| Docker Desktop or OrbStack | Compose stacks / DB peek |
| Optional Cursor | Open in Cursor |
| Optional MongoDB Compass | Open mongo URIs |

Project start/stop uses your **login shell**, so language runtimes installed for Terminal (`fnm`, `nvm`, etc.) apply. See [Start and stop](/start-stop/).

## What you get

- Catalog local projects (great for [AI-generated piles](/for-ai-projects/))
- Start / stop / restart with logs
- Compose DB peek, copy URI, Compass for mongo
- Port conflict reconciliation
- Machine-local only — no cloud account

## Verify you have the right “Warashi”

This product is a **localhost control plane**, named after the folklore house spirit 座敷童子. It is **not** the Live2D AI companion also nicknamed Warashi. Disambiguation: [The name](/name/).

## FAQ

### Is it notarized by Apple?

Not yet. Use right-click → Open, or clear quarantine with xattr. Notarization is on the roadmap when Developer Program timing allows.

### Is there Homebrew?

Not yet. Install from GitHub Releases. A cask may come after notarization.

### Linux or Windows?

macOS-first. Other platforms are future work.

## More rooms

- [Start and stop projects](https://zashiki.anireco.app/start-stop/)
- [For AI-generated projects](https://zashiki.anireco.app/for-ai-projects/)
- [The name 座敷童子](https://zashiki.anireco.app/name/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
