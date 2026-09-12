# Landing + SEO rooms (`zashiki.anireco.app`)

Static site for Cloudflare Workers static assets. Home is the Noren Threshold hero; interior SEO rooms are built from Markdown.

## Develop

```bash
# Edit rooms
site/content/*.md
site/content/compare/*.md

# Build HTML + markdown twins + sitemap + llms.txt
cd site && npm install && npm run build

# Preview
npm run preview   # http://127.0.0.1:8765
```

From repo root: `npm run site:build` / `npm run site:preview`.

## Deploy

```bash
cd site
npm run build
npx wrangler deploy
```

## DNS

On the **anireco.app** Cloudflare zone:

1. Deploy the Worker once.
2. Attach custom domain `zashiki.anireco.app` in the Workers dashboard, **or** CNAME `zashiki` to the `*.workers.dev` hostname.

## Screenshot

Real app window capture lives at [`assets/screenshot.png`](./assets/screenshot.png).

## Agent / LLM

- [`llms.txt`](./llms.txt) — curated room index
- [`llms-full.txt`](./llms-full.txt) — concatenated room bodies
- Each room also ships `index.md` next to `index.html`
