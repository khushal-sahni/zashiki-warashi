---
name: Zashiki Warashi
description: Indigo noren threshold into a warm house that keeps localhost tidy
colors:
  ink: "#101822"
  indigo: "#1a2744"
  indigo-deep: "#0d172a"
  paper: "#e8e0d4"
  muted: "#cdb69f"
  wood: "#3a200c"
  persimmon: "#d9671c"
  cta-ink: "#1a120c"
typography:
  display:
    fontFamily: "Fresca, \"Protest Revolution\", sans-serif"
    fontSize: "clamp(2.4rem, 11vw, 3.25rem)"
    fontWeight: 400
    lineHeight: 0.95
    letterSpacing: "normal"
  headline:
    fontFamily: "Fresca, \"Protest Revolution\", sans-serif"
    fontSize: "1.75rem"
    fontWeight: 400
    lineHeight: 1.2
    letterSpacing: "0.03em"
  body:
    fontFamily: "\"Reddit Sans Condensed\", \"Segoe UI\", system-ui, sans-serif"
    fontSize: "1.05rem"
    fontWeight: 400
    lineHeight: 1.65
    letterSpacing: "normal"
  label:
    fontFamily: "Kosugi, \"Reddit Sans Condensed\", sans-serif"
    fontSize: "0.95rem"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "normal"
  footer:
    fontFamily: "\"Reddit Sans Condensed\", \"Segoe UI\", system-ui, sans-serif"
    fontSize: "0.9rem"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "normal"
rounded:
  window: "0.55rem"
  fold: "0.75rem"
  pill: "999px"
spacing:
  xs: "0.45rem"
  sm: "0.75rem"
  md: "1.25rem"
  lg: "1.5rem"
  xl: "2.75rem"
  section: "3.5rem"
components:
  button-primary:
    backgroundColor: "{colors.persimmon}"
    textColor: "{colors.cta-ink}"
    rounded: "{rounded.pill}"
    padding: "0.7rem 1.25rem"
    height: "2.75rem"
  button-primary-hover:
    backgroundColor: "{colors.persimmon}"
    textColor: "{colors.cta-ink}"
  button-ghost:
    backgroundColor: "transparent"
    textColor: "{colors.paper}"
    rounded: "{rounded.pill}"
    padding: "0.7rem 1.25rem"
    height: "2.75rem"
  fold-panel:
    backgroundColor: "{colors.indigo}"
    textColor: "{colors.muted}"
    rounded: "{rounded.fold}"
    padding: "1.25rem 1.35rem"
  hotspot-download:
    backgroundColor: "transparent"
    textColor: "transparent"
    rounded: "{rounded.pill}"
---

# Design System: Zashiki Warashi

## Overview

**Creative North Star: "Noren Threshold"**

The public face of Zashiki Warashi is a machiya shop curtain, not a SaaS dashboard. Desktop first viewport is an approved composition raster (`site/assets/hero-comp.png`): indigo-dyed noren parted left, cedar lintel across the top, warm paper interior glow on the right, and a real app screenshot inset in the revealed room. Download sits on the parted edge; GitHub stays quiet on the lintel. The visitor parts the curtain into a house that already keeps localhost tidy.

Below the fold, content is numbered sequential rooms — offer, how it works, what you need, spirit — in indigo-washed panels on sumi-ink ground. Persimmon is reserved for the primary action. Voice stays plain and anti-hype; folklore meaning (座敷童子) is brand, not decoration. Rejected: dark purple SaaS heroes, bolted-on screenshot cards, fake social proof, eyebrow kickers, and system display faces.

**Key Characteristics:**
- Asymmetric noren reveal as the hero grammar (comp-b)
- Indigo cloth / cedar wood / warm paper / sumi ink / persimmon accent
- Fresca display + Reddit Sans Condensed body; Kosugi for Japanese/folklore line
- Composition raster + hotspot links on desktop; labeled CTA lockup on mobile
- Real screenshot as interior proof, never a floating marketing card

## Colors

Indigo-dyed cloth and sumi night against warm paper and cedar — persimmon only on the parting action.

### Primary
- **Persimmon** (`persimmon`): Primary CTA fill (Download for macOS). Rarity is the point — one voice for the slit through the curtain.

### Secondary
- **Indigo Dye** (`indigo`): Noren cloth and fold-panel washes; the dyed textile of the threshold.
- **Indigo Deep** (`indigo-deep`): Mobile hero ground when the composition raster is hidden.

### Tertiary
- **Cedar Wood** (`wood`): Lintel / timber tone carried in the hero composition; anchors the threshold as architecture, not flat UI chrome.

### Neutral
- **Sumi Ink** (`ink`): Page ground and hero frame.
- **Warm Paper** (`paper`): Primary light text and interior glow reading.
- **Muted Paper** (`muted`): Body and footer secondary text on dark panels.
- **CTA Ink** (`cta-ink`): Dark brown text on persimmon pills for contrast.

### Named Rules
**The One Slit Rule.** Persimmon appears on primary Download actions only. Do not sprinkle it across decorative accents, icons, or secondary links beyond the existing footer Sponsors treatment.

**The Cloth-Not-Chrome Rule.** Large surfaces read as dyed cloth, timber, or paper — not glassmorphism, purple gradients, or neon glow.

## Typography

**Display Font:** Fresca (fallback Protest Revolution)
**Body Font:** Reddit Sans Condensed (Segoe UI / system-ui fallback)
**Label/Mono Font:** Kosugi for the 座敷童子 / folklore line

**Character:** Fresca carries brushy shop-curtain energy for the product name and fold titles. Reddit Sans Condensed keeps body copy condensed and workmanlike. Kosugi marks the Japanese spirit line without turning the whole page into a display carnival.

### Hierarchy
- **Display** (400, `clamp(2.4rem, 11vw, 3.25rem)`, line-height 0.95): Mobile lockup product name; desktop name lives in the baked hero raster.
- **Headline** (400, 1.75rem, letter-spacing 0.03em): Fold section titles (What it keeps, How it works, …).
- **Body** (400, 1.05rem, line-height 1.65): Fold paragraphs and list items; max measure ~42rem via `.below`.
- **Label** (400, 0.95rem, Kosugi): Folklore subtitle / 座敷童子 line on mobile.
- **Footer** (400, 0.9rem): Footer meta and muted legal line.

### Named Rules
**The Sumi Type Rule.** Light type on ink/indigo; never invert to dark-on-cream marketing blocks for marketing sections. Display stays Fresca/Protest — no Inter, Roboto, or system UI as hero face.

## Layout

Desktop hero is a full-bleed 1280×720 composition (`aspect-ratio: 1280 / 720`) with hotspot regions from the scaffold (`--r-download-*`, `--r-github-btn-*`, `--r-brand-mark-*`). The real screenshot overlays the inset window (~left 52%, top 18%, width 44%, height 72%) clear of the Download zone. Below-fold content centers at `max-width: 42rem` with section padding `3.5rem 1.5rem 3rem` and fold spacing `2.75rem`.

At `max-width: 820px`, hide `.hero-art`, `.app-proof`, and lintel hotspots. Show `.mobile-lockup` (name + 座敷童子) and a full-width labeled persimmon Download so the baked CTA is never cropped. Recompose — do not scale the desktop noren down.

### Named Rules
**The Parted Edge Rule.** Download lives on the curtain slit (desktop hotspot) or as an explicit labeled pill (mobile) — never as a floating badge on top of the app screenshot.

## Elevation & Depth

Depth comes from materials and reveal motion, not floating card stacks. The hero settles from slight scale/blur; the app proof slides in from the right. Fold panels use a soft indigo wash plus one ambient shadow. Hotspots signal hover with an inset persimmon ring, not lift.

### Shadow Vocabulary
- **App interior** (`filter: drop-shadow(0 18px 40px rgba(0, 0, 0, 0.4))`): Real screenshot sitting in the revealed room.
- **Fold panel** (`box-shadow: 0 12px 36px rgba(0, 0, 0, 0.22)`): Quiet depth under below-fold rooms.
- **Mobile CTA** (`box-shadow: 0 10px 28px rgba(0, 0, 0, 0.35)`): Grounds the labeled Download on indigo.
- **Hotspot hover** (`box-shadow: inset 0 0 0 2px` persimmon mix): Focus/hover affordance on transparent desktop targets.

### Named Rules
**The Reveal-Not-Lift Rule.** Prefer reveal animations and material shadows over multi-layer drop shadows or glow. Honor `prefers-reduced-motion: reduce` by disabling hero animations.

## Shapes

Pills for actions and hotspots (`999px`). Soft window rounding on the inset screenshot (`0.55rem`). Fold panels use a gentle room corner (`0.75rem`) with a hairline paper-mixed border — panels are sequential rooms, not marketing cards stacked for decoration.

### Named Rules
**The Room-Not-Card Rule.** If removing a fold’s border/shadow would still leave a clear section with one job, keep the treatment restrained. Never put cards in the hero.

## Components

### Buttons
- **Shape:** Full pill (`999px`), min-height `2.75rem`, padding `0.7rem 1.25rem`, weight 600.
- **Primary:** Persimmon fill, CTA ink text (`.download-inline`; mobile `.hotspot.download`). Hover: slight brightness bump.
- **Ghost:** Transparent with paper-mixed 1px border, paper text (`.ghost-inline` for GitHub).
- **Desktop hotspot Download:** Transparent text over the baked CTA in the composition; hover shows inset persimmon ring. Focus-visible: 2px persimmon outline, offset 3px.

### Cards / Containers
- **Fold panels:** Indigo-to-ink vertical wash, paper-mixed border, fold radius, ambient shadow. One headline + one supporting block per panel.
- **Corner Style:** `0.75rem` — room corners, not sharp editorial rules.

### Navigation
- **Desktop:** Brand mark, GitHub, and Download are absolute hotspots over the composition raster (scaffold region %). No separate text nav chrome.
- **Mobile:** Lockup + labeled Download only; lintel GitHub/brand hotspots hidden.

### Signature: Noren Hero
- Full-bleed approved composition (`hero-comp.png`) with transparent hotspot links.
- Real product screenshot (`screenshot.png`) inset in the reveal window — proof of the house interior, not a collage tile.
- Mobile abandons the raster for indigo lockup + explicit CTA to avoid cropping the baked button.

## Do's and Don'ts

### Do:
- **Do** treat Download as parting the noren — primary persimmon action on the slit or as a labeled mobile pill.
- **Do** keep the real app screenshot inside the revealed interior; update `screenshot.png` when the product UI changes.
- **Do** use Fresca for display/fold titles, Reddit Sans Condensed for body, Kosugi for the folklore line.
- **Do** recompose below 820px: hide hero-art and show labeled Download.

### Don't:
- **Don't** ship another dark SaaS hero with a bolted-on screenshot card, purple gradients, or glow stacks.
- **Don't** invent testimonials, star counts, or benchmarks — evidence is the real window and GitHub Releases only.
- **Don't** crop or scale the desktop composition on mobile in a way that hides the baked CTA; use the lockup path instead.
- **Don't** promote unused loaded faces (e.g. Finger Paint) or system UI fonts into the display role.
