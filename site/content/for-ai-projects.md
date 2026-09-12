---
slug: for-ai-projects
title: "Local control plane for AI-generated projects — Zashiki Warashi"
h1: "For the AI-generated project pile"
description: "macOS app that catalogs Cursor/vibe-coded repos, starts and stops them via your login shell, and peeks Compose DBs — without taxing every folder with YAML."
lede: "When AI coding leaves you with a drawer of half-finished repos, you need a house that remembers them — not another config file in every project."
related:
  - /start-stop/
  - /localhost-control-plane/
  - /port-already-in-use/
faq:
  - q: "Does Zashiki Warashi require a config file in each AI project?"
    a: "No. It observes existing repos and keeps overrides in app data. There is no required per-repo YAML."
  - q: "Is this an AI coding agent?"
    a: "No. It does not write code or run agents. It catalogs, starts, stops, and peeks the projects those agents leave behind."
  - q: "Will it work with Cursor-generated Node apps?"
    a: "Yes when the project has a normal start signal (package.json scripts, compose, Makefile, Cargo, Go). You can override the start command in the app."
---

AI assistants and vibe-coding tools are excellent at spinning up new folders. They are less excellent at helping you find last Tuesday’s API, remember which port it claimed, or stop the Node process that is still holding `3000` after you closed the chat.

**Zashiki Warashi** is a macOS-first desktop control plane for that pile. It keeps a machine-local catalog of projects, starts and stops them through your login shell (so `fnm` / `nvm` PATH works), peeks Docker Compose database endpoints, and deep-links into tools you already use — Cursor, Finder, MongoDB Compass.

It is glue, not another framework. It does not ask every generated repo to adopt a new YAML format.

## The pain it is built for

A typical week with Cursor, Claude Code, or similar tools looks like this:

1. Scaffold a Next app. It runs. You close the window.
2. Scaffold a FastAPI sidecar. Different port. Still running somewhere.
3. Clone a compose stack for Postgres and Mongo. You copy a URI once, then lose the note.
4. Monday: “Which folder was the invoice prototype?” Finder search. Terminal archaeology. `lsof -i :3000`.

Folder bookmarks and Raycast snippets help a little. A Docker GUI helps when the problem is containers. Neither is a **project house** that knows your AI-spawned repos as first-class citizens.

## Observe, don’t tax

Zashiki Warashi’s product rule is deliberate: **observe existing repos**. Catalog lives in the app’s SQLite data — not in your git trees. Start-command inference looks at `package.json`, Compose files, Makefiles, Cargo, and Go layouts. Overrides stay in app data.

That matters for AI-generated code because those folders are disposable, half-owned, or not yours to litter with tooling config. You should not have to teach every throwaway prototype a new schema before you can start it again.

Optional export formats may appear later. They will stay opt-in. V1 does not write into managed project repos by default.

## What you do in the app

1. **Add a folder** or **scan roots** you already keep projects under.
2. **Select** a project — sidebar and inspector paint from memory in the same frame.
3. **Start / Stop / Restart** via your login shell in a process group (children die with the group).
4. **Peek the Stack** for Compose DB-like services — copy a URI, open Compass for mongo.
5. When ports collide, **reconcile**: stop the occupant or remap — see [Port already in use](/port-already-in-use/).

Logs for the process (and Compose) live in a pane. The app can quit while projects keep running; status rehydrates from PIDs on launch.

## What it is not

- Not an AI coding agent or chat UI.
- Not Docker Desktop, OrbStack, Portainer, or a full Compose editor — see [vs Docker Desktop](/compare/docker-desktop/) and [vs Portainer](/compare/portainer/).
- Not a replacement for Cursor — it opens Cursor for you when you want the editor.

If your problem is “I have twelve vibe-coded folders and I cannot remember which ones are running,” this house is for that. If your problem is “I need a better container runtime,” use OrbStack or Docker Desktop and put Zashiki on top.

## Honest requirements

- **macOS** first.
- **Docker Desktop or OrbStack** when you use Compose stacks.
- Releases are **unsigned** for now — first open may need right-click → Open, or `xattr -dr com.apple.quarantine`. Details on [Download](/download/).

MIT open source. Download from [GitHub Releases](https://github.com/khushal-sahni/zashiki-warashi/releases/latest).
