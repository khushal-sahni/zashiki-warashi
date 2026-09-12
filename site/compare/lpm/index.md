# vs lpm

> lpm is a macOS local project manager with services YAML and agent terminals. Zashiki Warashi observes existing repos without required per-project config — catalog, start/stop, Compose DB peek.

Canonical: https://zashiki.anireco.app/compare/lpm/

## Short answer

**[lpm](https://lpm.cx/)** (and similar local project managers) shine when you want a defined multi-service workspace — often with explicit config, live terminals, and AI-agent-oriented flows.

**Zashiki Warashi** shines when you want a **house spirit for folders you already have**: scan roots, infer start commands, stop process groups, peek Compose DBs, open Compass — **without** taxing every AI-generated repo with a new schema.

## Side-by-side

| Concern | lpm (typical) | Zashiki Warashi |
| --- | --- | --- |
| Project model | Services you define | Observe existing repo signals |
| Per-project config | Central to the workflow | Optional later; none required in v1 |
| AI coding agents in-app | Strong focus in current lpm | Out of scope |
| Compose DB URI / Compass | Not the core story | First-class peek |
| Port reconciliation | Varies | Catalog / docker / native occupants |
| Cloud accounts | No (local) | No (local) |

Exact lpm features move quickly — treat their site as source of truth. This page is about **category fit**, not a scorecard of version N.

## Who should pick Zashiki

- You have a pile of [Cursor / vibe-coded folders](/for-ai-projects/) and refuse another YAML per folder
- You care about Compose DB peek and Compass more than embedded agent terminals
- You want login-shell start/stop with process-group semantics — [Start and stop](/start-stop/)

## Who should pick lpm (or similar)

- You want one workspace that runs services **and** AI agent terminals side by side
- You are happy maintaining explicit service definitions and profiles
- Duplicating checkouts for parallel agents is a primary workflow

Honest product principle: prove the mechanism; do not invent superiority. Both can be right for different Mondays.

Download Zashiki: [Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest).

## FAQ

### Which one needs YAML in the project?

lpm is built around defining services (and profiles) in config. Zashiki infers start commands and keeps overrides in app data — no required per-repo YAML.

### Does Zashiki run Claude Code or Codex terminals?

No. It is not an agent workspace. It operates the projects agents leave behind and can open Cursor.

### Can I use both?

Yes in principle — they optimize different workflows. Pick one as the daily house to avoid duplicate lifecycle ownership.

## More rooms

- [For AI-generated projects](https://zashiki.anireco.app/for-ai-projects/)
- [Start and stop projects](https://zashiki.anireco.app/start-stop/)
- [Localhost control plane](https://zashiki.anireco.app/localhost-control-plane/)

---

[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)
