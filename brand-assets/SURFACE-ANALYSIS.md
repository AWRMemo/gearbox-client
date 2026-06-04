# Brand Surface Analysis — Gearbox

**Date:** June 2026
**Source:** Brand.md v3.1, Logo-Brainstorm.md
**Direction:** B+G Hybrid — "The Constellation with Violet Thread"

---

## 1. Surface Matrix

Every surface the Gearbox brand must appear on, with required format, dimensions, and usage.

### 1.1 Icon Surfaces (Level 2 — Mark Only)

| Surface | Dimensions | Format | Background | Notes |
|---------|-----------|--------|------------|-------|
| Favicon (browser tab) | 32×32 | PNG / ICO | Transparent dark | Amber dot per Brand.md §7.10. At 32×32 the 3-node constellation collapses to a single amber point. |
| Apple Touch Icon | 180×180 | PNG | Deep Space | Mark centered, optional small wordmark beneath. |
| Android Chrome icon | 192×192 | PNG | Deep Space | Same as Apple touch icon. |
| Windows ICO (multi-res) | 16, 24, 32, 48, 64, 128, 256 | ICO | Deep Space | Packaged into single .ico file for Windows app. |
| macOS ICNS (multi-res) | 16, 32, 64, 128, 256, 512, 1024 | ICNS | Deep Space | Packaged into single .icns file for macOS app. |
| Linux PNG set | 16, 24, 32, 48, 64, 128, 256, 512 | PNG | Deep Space | Named per freedesktop.org spec (e.g., `gearbox_256x256.png`). |
| System tray (Windows) | 16×16, 24×24 | PNG | Transparent | Must read at tiny size. Monochrome amber on transparent for light/dark taskbar compatibility. |
| System tray (macOS) | 18×18, 36×36 (2×) | PNG | Transparent | macOS menu bar extras. Template image (monochrome). |
| System tray (Linux) | 22×22, 48×48 | PNG | Transparent | StatusNotifierItem spec. |
| Mobile app icon (iOS) | 1024×1024 | PNG | Deep Space rounded square | iOS applies corner mask automatically. Mark centered, no text. |
| Mobile app icon (Android) | 1024×1024 (base) | PNG | Deep Space | Adaptive icon: 108dp foreground layer, Deep Space background. |
| Push notification icon (Android) | 24×24 dp | PNG | Transparent white-on-transparent | Single-color white silhouette of mark. |
| Notification icon (iOS) | 512×512 or 1024×1024 | PNG | Deep Space | OS crops to circle. Mark centered. |
| Stream favicon | 32×32 | PNG | Deep Space | Small mark for local HTTP stream pages. |
| Stream OG image | 1200×630 | PNG | Deep Space | Stream title + curator attribution + small mark in corner. |

### 1.2 Lockup Surfaces (Level 1 — Full)

| Surface | Dimensions | Format | Background | Notes |
|---------|-----------|--------|------------|-------|
| App loading screen | App window size (variable) | Centered layout | Deep Space `#0A0A0F` | Motion logo: constellation resolves + wordmark fades in (~1.4s). Mark centered above wordmark. |
| App About page | ~400px wide panel | Embedded in UI | Deep Space | Static full lockup: mark above "GEARBOX RELAY". |
| Installer splash (Windows) | 164×314 (banner) + 150×57 (dialog) | BMP | Deep Space | Windows installer branding. Mark + wordmark centered. |
| Installer DMG background (macOS) | ~658×498 | PNG | Deep Space | DMG background with mark at top, app icon near center, Applications folder alias. |
| Documentation header | ~120px height bar | Inline SVG/PNG | Deep Space | Mark (small, left) + wordmark inline. |
| og:image (gearbox.ai) | 1200×630 | PNG | Deep Space | Full lockup centered. Declaration text beneath: "Your personal intelligence engine. Zero cloud. Zero compromise." |
| Twitter/X card | 1200×675 | PNG | Deep Space | Same as og:image, different ratio. |
| GitHub org avatar | Any square (displays as circle) | PNG | Deep Space | Mark centered. Org: AWRMemo. |
| GitHub social preview | 1280×640 | PNG | Deep Space | For repo README display. |

### 1.3 Wordmark Surfaces (Level 3 — Text Only)

| Surface | Format | Notes |
|---------|--------|-------|
| Email signature | Unicode / styled HTML | "Gearbox" in clean type. No image needed. |
| Footer credits | Inline text | "Built with Gearbox" or "Powered by Gearbox Relay." |
| CLI / terminal output | ASCII / Unicode | Plain text. No styling required. |

### 1.4 Motion Logo Surfaces

| Surface | Trigger | Duration | Description |
|---------|---------|----------|-------------|
| App launch | Cold start | ~1.4s | Scattered dim amber dots → gravitate into 3-node constellation → violet lines appear → dim dots fade → "GEARBOX" wordmark fades in. |
| App resume | Wake from tray / sleep | ~0.6s | Constellation already formed. Brief violet pulse + wordmark fade. |
| Enrichment complete | In-app toast/indicator | ~0.2s | Icon mark pulses violet briefly, returns to amber. |
| Stream publish | Publish confirmation | ~0.6s | Amber transmission pulse ring radiates from mark. |
| Logo hover (web) | Mouse enter | ~0.5s | Nodes brighten slightly (10–15%). Violet lines illuminate. |

---

## 2. Color Specifications (Per Surface)

| Surface | Background | Mark Color | Wordmark | Violet Lines? |
|---------|-----------|------------|----------|---------------|
| Favicon (32×32) | Transparent dark | `#FFB000` (amber dot) | None | No (too small) |
| App icons (desktop) | `#0A0A0F` | `#FFB000` | None | Yes (512px+) |
| App icons (mobile) | `#0A0A0F` | `#FFB000` | None | Yes (512px+) |
| System tray | Transparent | `#FFB000` | None | No (too small) |
| Notification icon | Transparent | White (`#FFFFFF`) | None | No |
| Full lockup (loading) | `#0A0A0F` | `#FFB000` | `#E8E6E3` (Cinzel) | Yes |
| Full lockup (og:image) | `#0A0A0F` | `#FFB000` | `#E8E6E3` (Cinzel) | Yes |
| GitHub avatar | `#0A0A0F` | `#FFB000` | None | Yes |
| Print / merchandise | Black or material | White (engraved/embossed) | N/A | No (monochrome) |

---

## 3. Mark Anatomy — The Constellation

### 3.1 Component Inventory

```
┌─────────────────────────────────────────┐
│                                         │
│   ○ (dim)                               │
│                ○ (core, amber)          │
│      ╲                                   │
│        ╲  (violet line)                  │
│          ╲                               │
│           ○ (core, amber)                │
│          ╱                               │
│        ╱  (violet line)                  │
│      ╱                                   │
│    ○ (core, amber)                       │
│                     ○ (dim)              │
│               ○ (dim)                    │
│                                         │
└─────────────────────────────────────────┘
```

### 3.2 Specifications

| Element | Count | Color | Size (relative) | Opacity | Notes |
|---------|-------|-------|----------------|---------|-------|
| Core nodes | 3 | `#FFB000` (Signal Amber) | 100% | 100% | Primary constellation. Arranged in loose triangle, slightly asymmetrical. |
| Connecting lines | 2–3 | `#6B4EE6` (Enrichment Violet) | 1.5–2px stroke | 85% | Connecting core nodes. Must not form perfect triangle. One edge slightly longer. |
| Dim nodes | 1–3 | `#E8E6E3` (Structure White) | 100% | 10–20% | Scattered around core nodes. "Unconnected captures." |
| Trimmed dim nodes | 2–5 | `#E8E6E3` | 100% | 5–8% | Faintest. At edges. Barely visible. |

### 3.3 Geometry Rules

- No node is equidistant from its neighbors. Slight asymmetry throughout.
- The overall composition is tilted ~5–8° from vertical.
- Core nodes form a triangle with one angle ~95–105°, one ~40–50°, one ~35–45°.
- Violet lines do not extend to node centers — they begin slightly outside the node edge (clean separation).
- The composition fits within a 1:1 bounding box. Mark is always rendered square, centered in its container.
- Minimum legible size: 32×32 (favicon — collapses to a single amber point; constellation detail not visible).

### 3.4 Grid & Alignment

- Mark is always centered in its container.
- For the full lockup, the mark sits above the wordmark, centered on the same vertical axis.
- The mark-to-wordmark gap is equal to ~40% of the mark's height.
- The amber rule line (1px) between "GEARBOX" and "RELAY" is centered, matching the wordmark width.

---

## 4. Asset Dimension Audit (What Needs Generating)

### 4.1 Primary Generations (via Magica text-to-image)

| # | Asset | Model | Size | Description |
|---|-------|-------|------|-------------|
| 1 | Constellation Mark (Master) | flux-2-max-text | 1024×1024 | Pure mark on Deep Space. 3 amber nodes, 2–3 violet lines, 3–5 dim nodes. No text. |
| 2 | Full Lockup (Master) | gpt-image-2-text | 1024×1024 | Mark above "GEARBOX" in serif editorial type, amber rule, "RELAY" below. Deep Space bg. |
| 3 | og:image (Landing) | gpt-image-2-text | 1200×630 | Full lockup + declaration text. Deep Space bg. |
| 4 | Constellation Mark (Alt) | flux-2-max-text | 1024×1024 | Variation — different node arrangement for A/B testing. |

### 4.2 Downscaled Derivatives (from Master)

| # | Asset | Source | Sizes |
|---|-------|--------|-------|
| 5 | Favicon 32×32 | #1 (Master) | 32×32 — crop/blur to single amber point |
| 6 | Favicon 180×180 | #1 (Master) | 180×180 |
| 7 | App icon 512×512 | #1 (Master) | 512×512 |
| 8 | App icon 256×256 | #1 (Master) | 256×256 |
| 9 | App icon 128×128 | #1 (Master) | 128×128 |
| 10 | App icon 64×64 | #1 (Master) | 64×64 |
| 11 | App icon 48×48 | #1 (Master) | 48×48 |
| 12 | App icon 32×32 | #1 (Master) | 32×32 |
| 13 | App icon 24×24 | #1 (Master) | 24×24 |
| 14 | App icon 16×16 | #1 (Master) | 16×16 |
| 15 | Tray icon 16×16 | #1 (Master) | 16×16, monochrome amber on transparent |
| 16 | Tray icon 24×24 | #1 (Master) | 24×24, monochrome amber on transparent |
| 17 | Mobile icon 1024×1024 | #1 (Master) | 1024×1024, Deep Space rounded square |
| 18 | Notification icon | #1 (Master) | White silhouette on transparent |

### 4.3 Text-Dependent Assets

| # | Asset | Source | Notes |
|---|-------|--------|-------|
| 19 | Full lockup (horizontal) | #2 (Master) | Mark left of wordmark — for title bars, docs |
| 20 | Wordmark only (white text) | Typography | "GEARBOX" in styled text, no icon. For inline/email use. |
| 21 | Wordmark only (amber text) | Typography | "GEARBOX" in amber, no icon. |

---

## 5. File Naming Convention

```
gearbox-logo-{variant}-{size}.{ext}

Variants:
  mark             — Icon-only, constellation, no text
  mark-alt         — Alternative constellation arrangement
  lockup-stacked   — Mark above "GEARBOX RELAY" (centered)
  lockup-inline    — Mark left of "GEARBOX" (horizontal)
  wordmark         — Text only, no icon
  og-image         — Social sharing card
  favicon          — Browser favicon
  tray             — System tray icon
  notification     — Push notification icon

Sizes:
  Named: favicon-32, touch-180, app-1024, app-512, app-256, etc.

Examples:
  gearbox-mark-1024.png
  gearbox-lockup-stacked-1024.png
  gearbox-favicon-32.png
  gearbox-tray-16.png
```

Directory: `brand-assets/`

---

## 6. Motion Logo Asset Requirements

### 6.1 Animation Frames (if sprite sheet or sequential PNGs)

- 60fps × 1.4s = ~84 frames for app launch animation
- Alternatively: single keyframe setup with CSS/JS animation parameters (preferred for web)
- For app: Lottie JSON or WebM with alpha (if engine supports it)

### 6.2 Required Motion States

| State | Format | Description |
|-------|--------|-------------|
| app-launch | Lottie / WebM | Full chaos→structure animation |
| app-resume | Lottie / WebM | Short violet pulse |
| enrichment-pulse | CSS / Lottie micro | 200ms violet flash |
| stream-publish | CSS / Lottie | 600ms amber ring pulse |

---

## 7. Typography Specs (for Generated Lockups)

Since AI image generators may not render Cinzel or IBM Plex Mono accurately, the generated lockup serves as a **visual direction** asset. Final wordmarks must be produced with actual font files.

| Element | Font | Size (relative) | Color | Tracking |
|---------|------|----------------|-------|----------|
| "GEARBOX" | Cinzel (serif, editorial) | 100% base | `#E8E6E3` | +5% |
| Amber rule | N/A | 1px height, wordmark width | `#FFB000` | N/A |
| "RELAY" | IBM Plex Mono | ~40% of "GEARBOX" cap height | `#E8E6E3` at 60% | +10% |

---

## 8. Immediate Next Steps

1. Generate Master Mark (#1) — constellation on Deep Space
2. Generate Full Lockup (#2) — mark + wordmark
3. Generate OG Image (#3) — social sharing card
4. Downscale Master to all icon sizes (#5–18)
5. Create tray icons in monochrome amber on transparent
6. Create notification icon (white silhouette on transparent)
7. Package into ICO / ICNS / multi-res PNG sets

---

*This document defines the complete surface list. No surface omitted.*
