# Landing page (`zashiki.anireco.app`)

Static site for Cloudflare Workers static assets.

## Deploy

```bash
cd site
npx wrangler deploy
```

## DNS

On the **anireco.app** Cloudflare zone:

1. Deploy the Worker once.
2. Attach custom domain `zashiki.anireco.app` in the Workers dashboard, **or** CNAME `zashiki` to the `*.workers.dev` hostname.

## Screenshot

Real app window capture lives at [`assets/screenshot.png`](./assets/screenshot.png). If that file is missing, `index.html` falls back to the on-brand CSS preview.
