---
slug: start-stop
title: "Start and stop local projects on macOS — Zashiki Warashi"
h1: "One-button start and stop"
description: "How Zashiki Warashi starts and stops local projects via your login shell, process groups, and PID rehydration — so fnm/nvm PATH works from a GUI app."
lede: "GUI apps on macOS often miss your interactive PATH. Zashiki starts project commands through your login shell, in a new process group, and remembers enough to stop them later."
related:
  - /for-ai-projects/
  - /port-already-in-use/
  - /download/
faq:
  - q: "Why use a login shell instead of spawning node directly?"
    a: "So fnm, nvm, Homebrew, and docker on your interactive PATH work the same as in Terminal. GUI apps otherwise see a thinner environment."
  - q: "Do projects die when I quit Zashiki?"
    a: "No. Processes outlive the app. On launch, status rehydrates from stored PIDs when they are still alive and match."
  - q: "Can I override the inferred start command?"
    a: "Yes. Overrides live in app SQLite data, not in the project repo."
---

Starting a local app from a desktop GUI sounds trivial until PATH, child processes, and leftover PIDs enter the chat. **Zashiki Warashi** treats start/stop as a first-class lifecycle — not a fire-and-forget Terminal paste.

## Inference, then override

When you add a project, the app looks for common start signals:

- `package.json` scripts (`dev`, `start`, and cousins)
- Docker Compose layouts
- Makefile targets
- Cargo / Go project cues

If inference is wrong, you set an override in the app. That override is stored in **app data**, not written into the repo. AI-generated folders stay untaxed; see [For AI-generated projects](/for-ai-projects/).

## Login shell spawn

macOS GUI processes often lack the environment you built in zsh or fish: `fnm`, `nvm`, Volta, Homebrew’s `docker`. Zashiki runs project start/stop commands via:

```text
$SHELL -lc '<your start command>'
```

That matches Terminal behavior for the common case. Edge cases exist if `$SHELL` is unusual — but the default path is the one developers already trust.

Docker status and `compose ps` use a **cached docker binary path** resolved once at startup, not a login shell on every click. Snappy selection matters: chrome paints from memory; running dots may lag a beat.

## Process groups and stop

Many `npm run dev` trees spawn children. Signaling only the parent leaves orphans holding ports.

Zashiki:

1. Starts the command in a **new process group**
2. Persists pid (and related identity) in app data
3. On **Stop**, sends SIGTERM to the group, then SIGKILL if needed

That is how you free the port without playing `lsof` roulette every time — though when something else already owns the port, see [Port already in use](/port-already-in-use/).

## Rehydrate on launch

Closing the control plane must not kill your stack. Processes **outlive the app**. Next launch, Zashiki rehydrates running status from stored PIDs when they are still alive and match expectations, and can refresh Docker occupancy in the background.

## Logs

Live process logs (and Compose log views) sit in a resizable pane so start/stop is not a black box. You still have Terminal; you should not *need* it for the happy path.

## What you need

- macOS
- Whatever runtime your projects need on the login-shell PATH
- Docker Desktop or OrbStack if Compose is part of the project

Install notes (unsigned builds, Gatekeeper): [Download](/download/).

Source and releases: [GitHub](https://github.com/khushal-sahni/zashiki-warashi).
