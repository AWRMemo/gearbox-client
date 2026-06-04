# Asset Manifest — Gearbox Brand

**Generated:** June 2026
**Direction:** B+G Hybrid — "The Constellation with Violet Thread"
**Source files:** `brand-assets/generated/`

---

## 1. Master Assets (AI-Generated Originals)

These are the raw Magica generations. Review these first, select the best variants, then proceed to downscale/crop/edit.

| # | Filename | Size | Dims | Type | Model | Run ID | Status |
|---|----------|------|------|------|-------|--------|--------|
| 1 | `constellation-mark-v1.png` | 463 KB | 1024×1024 | Icon mark | flux-2-max-text | 93f53e02-1 | ✅ |
| 2 | `constellation-mark-v2.png` | 534 KB | 1024×1024 | Icon mark | flux-2-max-text | 93f53e02-2 | ✅ |
| 3 | `constellation-mark-v3.png` | 507 KB | 1024×1024 | Icon mark | flux-2-max-text | 93f53e02-3 | ✅ |
| 4 | `constellation-mark-v4.png` | 590 KB | 1024×1024 | Icon mark | flux-2-max-text | 93f53e02-4 | ✅ |
| 5 | `favicon-master-1024.png` | 574 KB | 1024×1024 | Favicon base | flux-2-max-text | 94374072 | ✅ |
| 6 | `app-icon-master-1024.png` | 792 KB | 1024×1024 | Mobile app icon | flux-2-max-text | dbb3cae7 | ✅ |
| 7 | `tray-icon-master-512.png` | 208 KB | 512×512 | System tray (amber) | flux-2-max-text | 2dacf863 | ✅ |
| 8 | `lockup-full-v1.png` | 2.3 MB | 2048×2048 | Full stacked lockup | gpt-image-2-text | baac9c22 | ✅ |
| 9 | `lockup-horizontal-v1.png` | 1.4 MB | 2048×1152 | Horizontal lockup | gpt-image-2-text | fec716f3 | ✅ |
| 10 | `og-image-v1.png` | 1.5 MB | 2048×1152 | Social OG image | gpt-image-2-text | 8e732036 | ✅ |
| 11 | `notification-icon-master-512.png` | 183 KB | 512×512 | Notification icon | flux-2-max-text | 71aace99 | ✅ |

### Selection Guidance

1. **Constellation mark (v1–v4):** Pick the variant with the clearest 3-node triangular constellation with violet connecting lines and dim scattered dots. Avoid variants that are too complex, too many nodes, or look like a generic network graph. The selected variant becomes `gearbox-mark-master.png`.

2. **Favicon master:** Should be a single clean amber dot on Deep Space. If the dot isn't perfectly centered or has artifacts, regenerate or edit.

3. **Lockup variants:** AI text rendering is unreliable. Check that "GEARBOX" is spelled correctly in a serif typeface. If text is garbled, the lockup serves as a visual direction reference only — the final wordmark must be produced with actual Cinzel and IBM Plex Mono font files.

---

## 2. Derivative Assets (Downscaled from Master)

After selecting the master constellation mark and favicon, generate these derivatives:

### 2.1 Icon Sizes (from selected constellation mark)

| Filename | Size | Usage |
|----------|------|-------|
| `gearbox-mark-16.png` | 16×16 | Windows tray, small icon |
| `gearbox-mark-24.png` | 24×24 | Tray fallback |
| `gearbox-mark-32.png` | 32×32 | Favicon, small icon |
| `gearbox-mark-48.png` | 48×48 | Windows app icon |
| `gearbox-mark-64.png` | 64×64 | App icon |
| `gearbox-mark-128.png` | 128×128 | App icon |
| `gearbox-mark-256.png` | 256×256 | App icon |
| `gearbox-mark-512.png` | 512×512 | App icon, doc header |
| `gearbox-mark-1024.png` | 1024×1024 | Master copy |

### 2.2 Favicon Sizes (from selected favicon master)

| Filename | Size | Usage |
|----------|------|-------|
| `gearbox-favicon-32.png` | 32×32 | Browser tab |
| `gearbox-favicon-180.png` | 180×180 | Apple Touch Icon |
| `gearbox-favicon-192.png` | 192×192 | Android Chrome |
| `gearbox-favicon.ico` | 16+32+48 | Windows ICO (multi-res) |

### 2.3 Tray Icons (from tray icon master)

| Filename | Size | Usage |
|----------|------|-------|
| `gearbox-tray-16.png` | 16×16 | Windows taskbar |
| `gearbox-tray-24.png` | 24×24 | Windows taskbar (large DPI) |
| `gearbox-tray-18.png` | 18×18 | macOS menubar |
| `gearbox-tray-36.png` | 36×36 | macOS menubar (2×) |
| `gearbox-tray-22.png` | 22×22 | Linux StatusNotifierItem |
| `gearbox-tray-48.png` | 48×48 | Linux (large) |

### 2.4 Notification Icons (from notification master)

| Filename | Size | Usage |
|----------|------|-------|
| `gearbox-notif-24.png` | 24×24 dp | Android push notification |
| `gearbox-notif-48.png` | 48×48 dp | Android push (large) |
| `gearbox-notif-512.png` | 512×512 | iOS notification |

### 2.5 OG Image Derivatives

| Filename | Size | Usage |
|----------|------|-------|
| `gearbox-og-1200x630.png` | 1200×630 | Open Graph / Twitter card |
| `gearbox-og-1280x640.png` | 1280×640 | GitHub social preview |

---

## 3. Wordmark Assets (Typography-Based — NOT AI Generated)

These must be produced with actual font files due to unreliable AI text rendering.

| Filename | Description | Font | Color |
|----------|-------------|------|-------|
| `gearbox-wordmark-white.svg` | "GEARBOX" only, no icon | Cinzel Bold | `#E8E6E3` |
| `gearbox-wordmark-white.png` | "GEARBOX" only, raster | Cinzel Bold | `#E8E6E3` |
| `gearbox-wordmark-amber.svg` | "GEARBOX" only, amber | Cinzel Bold | `#FFB000` |
| `gearbox-lockup-stacked.svg` | Mark + "GEARBOX RELAY" | Cinzel + IBM Plex Mono | Per Brand.md |
| `gearbox-lockup-horizontal.svg` | Mark left of "GEARBOX" | Cinzel | Per Brand.md |
| `gearbox-wordmark-relay.svg` | "RELAY" product line | IBM Plex Mono | `#E8E6E3` at 60% |

---

## 4. Platform-Specific Packages

### 4.1 Windows ICO

```powershell
# From gearbox-mark-256.png, use ImageMagick or similar to create multi-res ICO:
magick gearbox-mark-256.png -define icon:auto-resize=256,128,64,48,32,24,16 gearbox.ico
```

### 4.2 macOS ICNS

```bash
# From gearbox-mark-1024.png, generate ICNS with iconutil
mkdir gearbox.iconset
sips -z 16 16   gearbox-mark-1024.png --out gearbox.iconset/icon_16x16.png
sips -z 32 32   gearbox-mark-1024.png --out gearbox.iconset/icon_16x16@2x.png
sips -z 32 32   gearbox-mark-1024.png --out gearbox.iconset/icon_32x32.png
sips -z 64 64   gearbox-mark-1024.png --out gearbox.iconset/icon_32x32@2x.png
sips -z 128 128 gearbox-mark-1024.png --out gearbox.iconset/icon_128x128.png
sips -z 256 256 gearbox-mark-1024.png --out gearbox.iconset/icon_128x128@2x.png
sips -z 256 256 gearbox-mark-1024.png --out gearbox.iconset/icon_256x256.png
sips -z 512 512 gearbox-mark-1024.png --out gearbox.iconset/icon_256x256@2x.png
sips -z 512 512 gearbox-mark-1024.png --out gearbox.iconset/icon_512x512.png
sips -z 1024 1024 gearbox-mark-1024.png --out gearbox.iconset/icon_512x512@2x.png
iconutil -c icns gearbox.iconset
```

### 4.3 Linux Freedesktop

Place scaled PNGs in:
```
/usr/share/icons/hicolor/{size}x{size}/apps/gearbox.png
/usr/share/icons/hicolor/scalable/apps/gearbox.svg
```

---

## 5. File Size Budget

| Category | Target | Current |
|----------|--------|---------|
| Favicon (32×32) | < 3 KB | — |
| Tray icon (16×16) | < 1 KB | — |
| App icon (256×256) | < 50 KB | — |
| Full lockup (2048×2048) | < 3 MB | 2.3 MB ✅ |
| OG image (1200×630) | < 500 KB | — |

---

## 6. Known AI Generation Issues

| Issue | Affects | Mitigation |
|-------|---------|------------|
| Colors may not match `#FFB000` / `#6B4EE6` / `#0A0A0F` exactly | All flux/gpt-image assets | Color-correct in post with image editor or Magica image-to-image edit |
| Constellation may have too many nodes (looks like network graph, not simple 3-node constellation) | `constellation-mark-v1–4` | Select best variant; use image-to-image edit to simplify if needed |
| AI text in lockups may be garbled or wrong typeface | `lockup-full-v1`, `lockup-horizontal-v1` | These are visual direction only. Final wordmarks use actual Cinzel font. |
| Rounded square app icon may not have clean rounded corners | `app-icon-master-1024.png` | Apply iOS/Android corner mask in post |
| Background may not be exact `#0A0A0F` | All | Color-pick and fill in post |

---

## 7. Next Steps (Sequential)

1. **Review** all `brand-assets/generated/` images via Magica widget
2. **Select** best constellation mark variant (v1–v4)
3. **Edit** selected mark if needed (simplify nodes, color-correct)
4. **Downscale** mark to all icon sizes (16–1024)
5. **Crop/center** favicon to amber dot at 32×32 and 180×180
6. **Generate** wordmark SVGs with actual Cinzel + IBM Plex Mono fonts (not AI)
7. **Package** platform-specific icon bundles (.ico, .icns, freedesktop PNGs)
8. **Test** in context: website, app, tray, mobile simulator

---

*This manifest tracks all generated and pending brand assets. Update as assets are reviewed, edited, and finalized.*
