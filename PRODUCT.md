# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Static HTML/CSS deployed as Cloudflare Workers static assets (`site/`). Home is the Noren Threshold conversion page; evergreen SEO rooms are Markdown-built HTML with markdown twins and `llms.txt` for agents. The desktop product itself is Tauri 2 + React + Vite + TypeScript (UI) with a Rust backend; this PRODUCT.md covers the public landing surface at zashiki.anireco.app and the product it markets.

## Users

Primary visitor: a macOS developer with a pile of local projects — often AI-generated repos — who needs one place to find, start/stop, and peek at those projects without spelunking Docker or Terminal. Success on the landing page: they understand the offer within seconds and hit **Download for macOS**.

## Product Purpose

Zashiki Warashi is a macOS-first desktop control plane for localhost. It remembers projects in a machine-local catalog, one-button start/stop via the user's login shell, peeks Compose/DB endpoints, and deep-links into tools already on the machine (Cursor, Finder, MongoDB Compass). It glues tools you already have; it does not replace Docker Desktop, Portainer, Raycast, or a folder bookmark.

Success means: open the app on Monday, find a project, start/stop it, see compose services and DB connection info, and open Compass or copy a URI without hand-diving Docker Engine.

## Positioning

A house spirit for the localhost pile — especially AI-generated repos. Observation + machine-local catalog only; no required per-repo YAML; no cloud sync or accounts. Named after the 座敷童子 — a house spirit that looks after the place.

## Operating Context

macOS (primary). Docker Desktop or OrbStack for Compose stacks. Optional Cursor for Open in Cursor. GitHub Releases for download (unsigned builds; Gatekeeper may require right-click → Open or `xattr`). Menu-bar tray for show/quit. Developer workflow: catalog → select → Start/Stop → Stack peek → logs.

## Capabilities and Constraints

Confirmed: project catalog (add/scan/list/remove), start command inference and overrides, lifecycle (start/stop/restart) with process groups and PID rehydrate, live logs, Compose DB stack peek (Up/Stop, copy URI, Compass for mongo), port conflict reconciliation, Open in Finder/Cursor, coffee keep-awake toggle, resizable panes.

Not building: Docker GUI replacement, query editors, AI coding agent, cloud sync/auth, required per-repo config, DB provisioning in v1, writing into project repos by default, notarization yet (unsigned releases).

Landing stack constraint: stay static HTML/CSS on Cloudflare Workers; no inventing customers, stars, benchmarks, or fake social proof.

## Brand Commitments

- Product name: **Zashiki Warashi**
- Folklore meaning must remain: 座敷童子 — a house spirit that looks after the place
- Voice: plain, specific, anti-hype; honest about unsigned builds and Docker requirements when those facts are shown

## Evidence on Hand

- Real app screenshot: `site/assets/screenshot.png`
- GitHub repo and Releases: https://github.com/khushal-sahni/zashiki-warashi
- Sponsors: https://github.com/sponsors/khushal-sahni
- License: MIT
- No testimonials, customer logos, download counts, or benchmarks exist — do not fabricate them

## Product Principles

1. Prove the mechanism; do not claim category superiority with invented proof.
2. Observe existing repos; never tax them with required config.
3. Glue existing tools (Cursor, Finder, Compass, Docker) rather than replace them.
4. Stay honest about macOS-first, unsigned builds, and Docker needs.
5. The landing page's primary job is clear offer → Download for macOS; SEO rooms capture distinct search intent without inventing proof.
