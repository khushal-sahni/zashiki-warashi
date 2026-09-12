#!/usr/bin/env node
/**
 * Build SEO rooms from content/*.md → slug/index.html + slug/index.md
 * Also writes sitemap.xml, llms.txt, llms-full.txt, _redirects.
 */
import { readFileSync, writeFileSync, mkdirSync, readdirSync, rmSync, existsSync } from "node:fs";
import { join, dirname, relative } from "node:path";
import { fileURLToPath } from "node:url";
import matter from "gray-matter";
import { marked } from "marked";

const __dirname = dirname(fileURLToPath(import.meta.url));
const SITE_ROOT = join(__dirname, "..");
const CONTENT_DIR = join(SITE_ROOT, "content");
const TEMPLATE = readFileSync(join(SITE_ROOT, "templates", "interior.html"), "utf8");
const ORIGIN = "https://zashiki.anireco.app";
const OG_IMAGE = `${ORIGIN}/assets/hero-comp.png`;

const PAGE_TITLES = {
  "/for-ai-projects/": "For AI-generated projects",
  "/localhost-control-plane/": "Localhost control plane",
  "/start-stop/": "Start and stop projects",
  "/compose-databases/": "Compose and databases",
  "/mongodb-compass/": "MongoDB Compass",
  "/port-already-in-use/": "Port already in use",
  "/download/": "Download for macOS",
  "/name/": "The name 座敷童子",
  "/compare/docker-desktop/": "vs Docker Desktop",
  "/compare/orbstack/": "vs OrbStack",
  "/compare/lpm/": "vs lpm",
  "/compare/portainer/": "vs Portainer",
};

marked.setOptions({ gfm: true });

function walkMd(dir, acc = []) {
  for (const name of readdirSync(dir, { withFileTypes: true })) {
    const p = join(dir, name.name);
    if (name.isDirectory()) walkMd(p, acc);
    else if (name.name.endsWith(".md")) acc.push(p);
  }
  return acc;
}

function escapeHtml(s) {
  return String(s)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;");
}

function assetPrefix(slugPath) {
  const depth = slugPath.split("/").filter(Boolean).length;
  return "../".repeat(depth);
}

function canonicalFor(slugPath) {
  return `${ORIGIN}/${slugPath}/`.replace(/([^:]\/)\/+/g, "$1");
}

function relatedSection(related, prefix) {
  if (!related?.length) return "";
  const items = related
    .map((href) => {
      const label = PAGE_TITLES[href] ?? href;
      const abs = href.startsWith("http") ? href : `${prefix}${href.replace(/^\//, "")}`;
      // href is like /start-stop/ — from /for-ai-projects/ need ../start-stop/
      const rel = href.replace(/^\//, "");
      return `<li><a href="${prefix}${rel}">${escapeHtml(label)}</a></li>`;
    })
    .join("\n          ");
  return `        <section class="fold related-fold" aria-labelledby="related-heading">
          <h2 id="related-heading">More rooms</h2>
          <ul class="related-list">
          ${items}
          </ul>
        </section>`;
}

function faqSection(faq) {
  if (!faq?.length) return "";
  const items = faq
    .map(
      (f) => `          <details class="faq-item">
            <summary>${escapeHtml(f.q)}</summary>
            <p>${escapeHtml(f.a)}</p>
          </details>`,
    )
    .join("\n");
  return `        <section class="fold faq-fold" aria-labelledby="faq-heading">
          <h2 id="faq-heading">FAQ</h2>
${items}
        </section>`;
}

function faqJsonLd(faq) {
  if (!faq?.length) return "";
  const data = {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    mainEntity: faq.map((f) => ({
      "@type": "Question",
      name: f.q,
      acceptedAnswer: { "@type": "Answer", text: f.a },
    })),
  };
  return `    <script type="application/ld+json">
${JSON.stringify(data, null, 2)}
    </script>`;
}

function softwareApplicationJson() {
  const data = {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: "Zashiki Warashi",
    applicationCategory: "DeveloperApplication",
    operatingSystem: "macOS",
    offers: { "@type": "Offer", price: "0", priceCurrency: "USD" },
    description:
      "macOS-first desktop control plane for localhost. Catalog local projects, one-button start/stop, Compose/DB peek, deep-links into Cursor, Finder, and Compass.",
    url: `${ORIGIN}/`,
    downloadUrl: "https://github.com/khushal-sahni/zashiki-warashi/releases/latest",
    license: "https://opensource.org/licenses/MIT",
  };
  return `    <script type="application/ld+json">
${JSON.stringify(data, null, 2)}
    </script>`;
}

function breadcrumbJson(slugPath, h1) {
  const parts = slugPath.split("/").filter(Boolean);
  const items = [
    { "@type": "ListItem", position: 1, name: "Home", item: `${ORIGIN}/` },
  ];
  let path = "";
  parts.forEach((part, i) => {
    path += `${part}/`;
    const isLast = i === parts.length - 1;
    const name = isLast
      ? h1
      : part === "compare"
        ? "Compare"
        : PAGE_TITLES[`/${path}`] ?? part;
    items.push({
      "@type": "ListItem",
      position: i + 2,
      name,
      item: `${ORIGIN}/${path}`,
    });
  });
  return JSON.stringify(
    { "@context": "https://schema.org", "@type": "BreadcrumbList", itemListElement: items },
    null,
    2,
  );
}

function breadcrumbTrail(slugPath, h1, prefix) {
  const parts = slugPath.split("/").filter(Boolean);
  let html = "";
  let accum = "";
  parts.forEach((part, i) => {
    accum += `${part}/`;
    const isLast = i === parts.length - 1;
    if (part === "compare" && !isLast) {
      html += ` <span aria-hidden="true">/</span> <a href="${prefix}compare/">Compare</a>`;
      return;
    }
    if (isLast) {
      html += ` <span aria-hidden="true">/</span> <span aria-current="page">${escapeHtml(h1)}</span>`;
    } else {
      html += ` <span aria-hidden="true">/</span> <a href="${prefix}${accum}">${escapeHtml(PAGE_TITLES[`/${accum}`] ?? part)}</a>`;
    }
  });
  return html;
}

function firstParagraph(md) {
  const lines = md.split(/\n+/);
  for (const line of lines) {
    const t = line.trim();
    if (!t || t.startsWith("#") || t.startsWith("!") || t.startsWith("|") || t.startsWith("-") || t.startsWith("*")) {
      continue;
    }
    return t.replace(/\*\*([^*]+)\*\*/g, "$1").replace(/\*([^*]+)\*/g, "$1").replace(/\[([^\]]+)\]\([^)]+\)/g, "$1");
  }
  return "";
}

function stripH1(md) {
  return md.replace(/^#\s+.+\n+/, "");
}

function cleanGeneratedDirs(pages) {
  for (const page of pages) {
    const outDir = join(SITE_ROOT, ...page.slugPath.split("/").filter(Boolean));
    if (existsSync(join(outDir, "index.html"))) {
      // keep; rebuild overwrites
    }
  }
  // Remove known generated room dirs that might be stale
  const known = Object.keys(PAGE_TITLES).map((p) => p.replace(/^\/|\/$/g, ""));
  for (const rel of known) {
    const dir = join(SITE_ROOT, ...rel.split("/"));
    if (existsSync(dir)) {
      try {
        rmSync(join(dir, "index.html"), { force: true });
        rmSync(join(dir, "index.md"), { force: true });
      } catch {
        /* ignore */
      }
    }
  }
}

function renderPage(filePath) {
  const raw = readFileSync(filePath, "utf8");
  const { data, content } = matter(raw);
  const slugPath = String(data.slug || relative(CONTENT_DIR, filePath).replace(/\.md$/, ""));
  const h1 = data.h1 || data.title;
  const title = data.title;
  const description = data.description;
  const related = data.related || [];
  const faq = data.faq || [];
  const bodyMd = stripH1(content.trim());
  const lede = data.lede || firstParagraph(bodyMd);
  const prefix = assetPrefix(slugPath);
  const canonical = canonicalFor(slugPath);
  const markdownHref = `${canonical}index.md`;
  const bodyHtml = marked.parse(bodyMd);

  let html = TEMPLATE;
  const replacements = {
    "{{title}}": escapeHtml(title),
    "{{description}}": escapeHtml(description),
    "{{canonical}}": canonical,
    "{{markdown_href}}": markdownHref,
    "{{og_image}}": OG_IMAGE,
    "{{asset_prefix}}": prefix,
    "{{breadcrumb_json}}": breadcrumbJson(slugPath, h1),
    "{{faq_json_block}}":
      [faqJsonLd(faq), slugPath === "download" ? softwareApplicationJson() : ""]
        .filter(Boolean)
        .join("\n"),
    "{{h1}}": escapeHtml(h1),
    "{{lede}}": escapeHtml(lede),
    "{{body}}": bodyHtml,
    "{{faq_section}}": faqSection(faq),
    "{{related_section}}": relatedSection(related, prefix),
    "{{breadcrumb_trail}}": breadcrumbTrail(slugPath, h1, prefix),
  };
  for (const [k, v] of Object.entries(replacements)) {
    html = html.split(k).join(v);
  }

  const outDir = join(SITE_ROOT, ...slugPath.split("/").filter(Boolean));
  mkdirSync(outDir, { recursive: true });
  writeFileSync(join(outDir, "index.html"), html);

  // Markdown twin: frontmatter-free readable doc for agents
  const mdTwin = [
    `# ${h1}`,
    "",
    `> ${description}`,
    "",
    `Canonical: ${canonical}`,
    "",
    bodyMd,
    "",
  ];
  if (faq.length) {
    mdTwin.push("## FAQ", "");
    for (const f of faq) {
      mdTwin.push(`### ${f.q}`, "", f.a, "");
    }
  }
  if (related.length) {
    mdTwin.push("## More rooms", "");
    for (const href of related) {
      mdTwin.push(`- [${PAGE_TITLES[href] ?? href}](${ORIGIN}${href})`);
    }
    mdTwin.push("");
  }
  mdTwin.push(
    "---",
    "",
    `[Download for macOS](https://github.com/khushal-sahni/zashiki-warashi/releases/latest) · [GitHub](https://github.com/khushal-sahni/zashiki-warashi)`,
    "",
  );
  writeFileSync(join(outDir, "index.md"), mdTwin.join("\n"));

  return {
    slugPath,
    title,
    description,
    h1,
    canonical,
    faq,
    related,
    bodyMd,
    lede,
  };
}

function writeCompareIndex(comparePages) {
  const prefix = "../";
  const links = comparePages
    .map((p) => {
      const leaf = p.slugPath.split("/").pop();
      return `<li><a href="./${leaf}/">${escapeHtml(p.h1)}</a> — ${escapeHtml(p.description)}</li>`;
    })
    .join("\n          ");
  const html = `<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <title>Honest comparisons — Zashiki Warashi</title>
    <meta name="description" content="How Zashiki Warashi differs from Docker Desktop, OrbStack, Portainer, and lpm — disambiguation, not scorecards." />
    <link rel="canonical" href="${ORIGIN}/compare/" />
    <link rel="stylesheet" href="${prefix}fonts.css" />
    <link rel="stylesheet" href="${prefix}styles.css" />
    <link rel="icon" href="${prefix}favicon.svg" type="image/svg+xml" />
  </head>
  <body class="page-interior">
    <header class="site-chrome">
      <a class="chrome-brand" href="${prefix}">Zashiki Warashi</a>
      <nav class="chrome-actions" aria-label="Primary">
        <a class="ghost-inline chrome-github" href="https://github.com/khushal-sahni/zashiki-warashi">GitHub</a>
        <a class="download-inline" href="https://github.com/khushal-sahni/zashiki-warashi/releases/latest">Download for macOS</a>
      </nav>
    </header>
    <main class="below interior-main">
      <nav class="breadcrumb" aria-label="Breadcrumb">
        <a href="${prefix}">Home</a>
        <span aria-hidden="true">/</span>
        <span aria-current="page">Compare</span>
      </nav>
      <article class="room">
        <header class="room-head">
          <h1>Honest comparisons</h1>
          <p class="lede">Disambiguation pages — what job each tool owns. Not invented scorecards.</p>
        </header>
        <section class="fold">
          <h2>Rooms</h2>
          <ul class="related-list">
          ${links}
          </ul>
        </section>
      </article>
      <p class="closing">
        <a class="download-inline" href="https://github.com/khushal-sahni/zashiki-warashi/releases/latest">Download for macOS</a>
        <a class="ghost-inline" href="${prefix}">Home</a>
      </p>
      <footer class="site-footer">
        <span>座敷童子 — a house spirit that looks after the place.</span>
        <span>MIT</span>
      </footer>
    </main>
  </body>
</html>
`;
  const outDir = join(SITE_ROOT, "compare");
  mkdirSync(outDir, { recursive: true });
  writeFileSync(join(outDir, "index.html"), html);
  writeFileSync(
    join(outDir, "index.md"),
    [
      "# Honest comparisons",
      "",
      "Disambiguation pages — what job each tool owns.",
      "",
      ...comparePages.map((p) => `- [${p.h1}](${p.canonical}): ${p.description}`),
      "",
    ].join("\n"),
  );
}

function writeSitemap(pages) {
  const urls = [
    `  <url>\n    <loc>${ORIGIN}/</loc>\n    <changefreq>weekly</changefreq>\n    <priority>1.0</priority>\n  </url>`,
    `  <url>\n    <loc>${ORIGIN}/compare/</loc>\n    <changefreq>monthly</changefreq>\n    <priority>0.6</priority>\n  </url>`,
    ...pages.map(
      (p) =>
        `  <url>\n    <loc>${p.canonical}</loc>\n    <changefreq>monthly</changefreq>\n    <priority>0.8</priority>\n  </url>`,
    ),
  ];
  writeFileSync(
    join(SITE_ROOT, "sitemap.xml"),
    `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls.join("\n")}\n</urlset>\n`,
  );
}

function writeLlms(pages) {
  const lines = [
    "# Zashiki Warashi",
    "",
    "> macOS-first desktop control plane for localhost. Catalog local projects (especially AI-generated repos), one-button start/stop, Compose/DB peek, deep-links into Cursor, Finder, and Compass. Observe existing repos — no required per-repo YAML.",
    "",
    `Home: ${ORIGIN}/`,
    `Download: https://github.com/khushal-sahni/zashiki-warashi/releases/latest`,
    `GitHub: https://github.com/khushal-sahni/zashiki-warashi`,
    "",
    "## Rooms",
    "",
  ];
  for (const p of pages) {
    lines.push(`- [${p.h1}](${p.canonical}): ${p.description}`);
    lines.push(`  - Markdown: ${p.canonical}index.md`);
  }
  lines.push("", "## Notes", "");
  lines.push(
    "- Not a Docker GUI, Portainer, OrbStack, or Docker Desktop replacement — glue for tools you already have.",
    "- Not the Live2D AI companion also nicknamed Warashi; see /name/ for folklore + disambiguation.",
    "- Builds are unsigned; Gatekeeper may need right-click Open or xattr quarantine clear.",
    "",
  );
  writeFileSync(join(SITE_ROOT, "llms.txt"), lines.join("\n"));

  const full = [
    "# Zashiki Warashi — full site for agents",
    "",
    lines.slice(2, 8).join("\n"),
    "",
  ];
  for (const p of pages) {
    full.push("---", "", `# ${p.h1}`, "", p.bodyMd, "");
    if (p.faq?.length) {
      full.push("## FAQ", "");
      for (const f of p.faq) full.push(`### ${f.q}`, "", f.a, "");
    }
  }
  writeFileSync(join(SITE_ROOT, "llms-full.txt"), full.join("\n"));
}

function writeRedirects(pages) {
  const lines = pages.map((p) => {
    const withSlash = `/${p.slugPath}/`;
    const noSlash = `/${p.slugPath}`;
    return `${noSlash} ${withSlash} 301`;
  });
  lines.push("/compare /compare/ 301");
  writeFileSync(join(SITE_ROOT, "_redirects"), lines.join("\n") + "\n");
}

function main() {
  const files = walkMd(CONTENT_DIR).sort();
  if (!files.length) {
    console.error("No content/*.md files found");
    process.exit(1);
  }
  cleanGeneratedDirs([]);
  const pages = files.map(renderPage);
  pages.sort((a, b) => a.slugPath.localeCompare(b.slugPath));
  const comparePages = pages.filter((p) => p.slugPath.startsWith("compare/"));
  writeCompareIndex(comparePages);
  writeSitemap(pages);
  writeLlms(pages);
  writeRedirects(pages);
  console.log(`Built ${pages.length} rooms → HTML + markdown twins, sitemap, llms.txt`);
}

main();
