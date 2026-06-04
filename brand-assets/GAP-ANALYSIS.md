# Continuity Gap Analysis — v2 → v3

**Date:** June 2026
**Source vs. implementation:** `Brand.md` + `Logo-Brainstorm.md` + `SURFACE-ANALYSIS.md` vs. `generate.py`

---

## Fixed (v3)

| # | Gap | Spec | Was | Now |
|---|-----|------|-----|-----|
| 1 | **Triangle angles** | One angle 95–105°, one 40–50°, one 35–45° (wide, asymmetrical) | ~65° / 53° / 62° (too symmetrical) | Wide top base (0.60×scale), offset apex. Approx. 100° / 42° / 38° |
| 2 | **Lines stop at node edges** | "Lines do not extend to node centers — begin slightly outside the node edge" | Lines drew through amber dots | `line_endpoint()` offsets by `node_r + 1px` from each endpoint |
| 3 | **Violet line opacity** | 85% | 100% opaque | RGBA `#6B4EE6` with alpha = int(255 × 0.85) |
| 4 | **Violet line width** | 1.5–2px stroke | 1px at 1024×1024 | `max(2, int(scale * 0.0035))` — 2px at 1024 |
| 5 | **Favicon background** | "transparent dark background" | Opaque `#0A0A0F` fill | `(0, 0, 0, 0)` — fully transparent with amber dot only |
| 6 | **OG image text** | "Your signal. Your structure. Your machine." (Cinzel) | "Your personal intelligence engine. / Zero cloud. Zero compromise." | Correct declaration in Cinzel, plus `gearbox.ai` below |
| 7 | **Rule line thickness** | 1px | `max(1, int(size * 0.001))` = 2px at 2048 | Hardcoded `width=1` |
| 8 | **Rule line width** | "matching the wordmark" | Fixed fraction of canvas | Measures actual textbbox of "GEARBOX" + 5% tracking compensation |
| 9 | **Tracking** | +5% for GEARBOX, +10% for RELAY | No letter spacing | `draw_text_tracked()` renders per-character with spacing |
| 10 | **Dim node count** | 1–3 dim + 2–5 trimmed | 5 dim + 3 trimmed (too many) | 3 dim (16–18%) + 3 trimmed (6%) |
| 11 | **Mark-to-wordmark gap** | ~40% of mark height | ~77% | Recalculated layout: mark_cy at −24%, rule at +3%, text auto-positioned |

---

## Remaining Gaps (not fixable in Pillow renderer)

| # | Gap | Why not fixed | Mitigation |
|---|-----|---------------|------------|
| R1 | **Font matching** | Cinzel and IBM Plex Mono not installed on this system | Using Cambria Bold and Cascadia Mono as serif/mono equivalents. Final wordmarks should use actual Cinzel/IBM Plex via a vector tool (Figma, Illustrator, or SVG export). |
| R2 | **Wordmark-only assets** | `SURFACE-ANALYSIS.md` calls for wordmark-only SVGs ("GEARBOX" in styled text, no icon) | Not generated yet. Pending font installation or SVG workflow. |
| R3 | **Rounded square app icon corners** | Pillow can't easily apply platform-corner-masks | iOS/Android apply corner masks automatically. For the master, Deep Space fill is intentional. |
| R4 | **Windows .ico / macOS .icns packaging** | Requires platform-specific tooling | See `ASSET-MANIFEST.md` §4 for ImageMagick/iconutil commands. |
| R5 | **Motion logo animation** | Not a static asset | Described in `SURFACE-ANALYSIS.md` §6. Needs Lottie/WebM + after-Effects or Rive workflow. |
| R6 | **No "all-caps display type" rule** | Brand.md §5.3 says "No all-caps display type" but the wordmark IS all-caps | Tension in the documentation. Cinzel is designed for all-caps editorial use, which is distinct from artificially forcing sentence-case fonts into all-caps. Not a bug — a doc clarification needed. |
| R7 | **Violet line 85% on Deep Space** | The violet is alpha-blended against `#0A0A0F`, which dims it slightly | This is correct behavior — 85% violet on deep space = slightly subdued. Per spec. |

---

## Verified (already correct)

| Spec | Status |
|------|--------|
| Brand colors: `#0A0A0F`, `#FFB000`, `#6B4EE6`, `#E8E6E3` | ✅ Exact hex values |
| 3 core amber nodes forming a triangle | ✅ |
| 3 violet connecting lines (all 3 edges) | ✅ |
| 6° tilt on the constellation | ✅ |
| Favicon: single amber dot, 32×32 and 180×180 | ✅ |
| Monochrome tray icon (amber-only on transparent) | ✅ |
| White notification icon on transparent | ✅ |
| Deep Space background on all mark/lockup/OG assets | ✅ |
| No gradients, no cloud icons, no sparkles, no locks | ✅ |
| Structure White at 60% opacity for "RELAY" product line | ✅ |
| Mark centered above wordmark in stacked lockup | ✅ |
| Mark left of wordmark in horizontal lockup | ✅ |
| All mark sizes down to 16×16 | ✅ |
| No AI-generated assets used for final output | ✅ (pure Pillow geometry) |

---

## Visual Review Checklist

Open `brand-assets/generated/` and verify:

1. `gearbox-mark-1024.png` — Does the triangle have a wide top and a narrow apex below? Are the violet lines cleanly separated from the amber dots (no overlap)? Are the dim specks barely visible?
2. `gearbox-favicon-32.png` — Is it a single crisp amber dot on transparency? Does it look clean in a browser tab?
3. `gearbox-lockup-stacked-2048.png` — Is "GEARBOX" in a serif font with some letter spacing? Is the amber rule line exactly 1px thick? Is "RELAY" in monospace at lower opacity?
4. `gearbox-og-1200x630.png` — Does it say "Your signal. Your structure. Your machine." in a serif font?
5. `gearbox-tray-16.png` — Is it a simple amber triangle glyph on transparency, readable at 16×16?
6. `gearbox-mark-16.png` — Does it collapse to a readable single-point impression?

---

*This document closes the v2→v3 gap cycle. Regeneration command: `python brand-assets/generate.py`*
