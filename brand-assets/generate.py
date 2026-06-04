"""
Gearbox brand asset generator.
Produces exact pixel-geometry PNGs using Pillow.
No AI generation — deterministic, repeatable, correct.
"""
import math
import os
from PIL import Image, ImageDraw, ImageFont

OUT = os.path.join(os.path.dirname(__file__), "generated")

# Brand colors
DEEP_SPACE = (0x0A, 0x0A, 0x0F)
SIGNAL_AMBER = (0xFF, 0xB0, 0x00)
ENRICHMENT_VIOLET = (0x6B, 0x4E, 0xE6)
STRUCTURE_WHITE = (0xE8, 0xE6, 0xE3)

# Violet line alpha per spec: 85% opacity
VIOLET_ALPHA = 0.85

def dim(color, opacity):
    """Return color blended against DEEP_SPACE at given opacity (0.0-1.0)."""
    out = []
    for i in range(3):
        out.append(int(color[i] * opacity + DEEP_SPACE[i] * (1 - opacity)))
    return tuple(out)

def circle_center(cx, cy, r):
    """Return bbox (left, top, right, bottom) for a circle given center and radius."""
    return (cx - r, cy - r, cx + r, cy + r)


def line_endpoint(x1, y1, x2, y2, offset_r):
    """Compute a point along the line from (x1,y1) toward (x2,y2), offset by `offset_r` from the start."""
    dx = x2 - x1
    dy = y2 - y1
    length = math.hypot(dx, dy)
    if length < 0.001:
        return (x1, y1)
    ux = dx / length
    uy = dy / length
    return (x1 + ux * offset_r, y1 + uy * offset_r)


def draw_constellation(draw, cx, cy, scale, tilt_deg=6):
    """
    Draw the Gearbox constellation mark centered at (cx, cy).
    Returns the bounding box of the mark.

    Specs per SURFACE-ANALYSIS.md §3.2–3.3:
      - 3 amber core nodes in an asymmetrical triangle (angles ~35-45°, 40-50°, 95-105°)
      - 3 violet connecting lines at 85% opacity, 1.5-2px, stopping at node edges
      - 1-3 dim nodes at 10-20% opacity
      - 2-5 trimmed dim nodes at 5-8% opacity
      - 5-8° tilt
    """
    t = math.radians(tilt_deg)
    cos_t, sin_t = math.cos(t), math.sin(t)

    # Asymmetrical triangle: very wide top, narrow apex below
    # Target angles: ~100° (top), ~42° (left), ~38° (right)
    # In screen coords (y down): nodes at top form the wide base, apex below
    base_width = 0.60 * scale       # width of top edge
    top_y = -0.38 * scale           # two top nodes sit above center
    apex_x = 0.02 * scale           # slight asymmetry: apex offset right
    apex_y = 0.34 * scale           # apex below center

    raw = [
        (-base_width / 2, top_y),   # top-left
        ( base_width / 2, top_y),   # top-right
        ( apex_x, apex_y),          # bottom apex
    ]

    nodes = []
    node_r = int(scale * 0.04)
    for x, y in raw:
        rx = x * cos_t - y * sin_t + cx
        ry = x * sin_t + y * cos_t + cy
        nodes.append((int(rx), int(ry)))

    # Draw violet connecting lines — with alpha=0.85, stopping at node edges
    line_w = max(2, int(scale * 0.0035))  # 1.5-2px minimum at 1024
    violet_rgba = ENRICHMENT_VIOLET + (int(255 * VIOLET_ALPHA),)

    for a, b in [(0, 2), (1, 2), (0, 1)]:
        x1, y1 = nodes[a]
        x2, y2 = nodes[b]
        sx, sy = line_endpoint(x1, y1, x2, y2, node_r + 1)
        ex, ey = line_endpoint(x2, y2, x1, y1, node_r + 1)
        draw.line([(sx, sy), (ex, ey)], fill=violet_rgba, width=line_w)

    # Draw amber core nodes (opaque)
    for nx, ny in nodes:
        draw.ellipse(circle_center(nx, ny, node_r), fill=SIGNAL_AMBER)

    # 3 dim nodes at 10–20% (spec: 1-3)
    dim_positions = [
        (cx - int(0.30 * scale), cy - int(0.28 * scale)),
        (cx + int(0.36 * scale), cy + int(0.08 * scale)),
        (cx + int(0.08 * scale), cy - int(0.35 * scale)),
    ]
    dim_opacities = [0.16, 0.13, 0.18]
    for i, (dx, dy) in enumerate(dim_positions):
        dr = int(node_r * 1.15)
        draw.ellipse(circle_center(dx, dy, dr), fill=dim(STRUCTURE_WHITE, dim_opacities[i]))

    # 3 trimmed dim nodes at 5-8% (spec: 2-5)
    edge_positions = [
        (cx - int(0.44 * scale), cy + int(0.30 * scale)),
        (cx + int(0.44 * scale), cy - int(0.28 * scale)),
        (cx - int(0.10 * scale), cy + int(0.42 * scale)),
    ]
    for dx, dy in edge_positions:
        dr = int(node_r * 0.85)
        draw.ellipse(circle_center(dx, dy, dr), fill=dim(STRUCTURE_WHITE, 0.06))

    return (cx - int(scale * 0.55), cy - int(scale * 0.55),
            cx + int(scale * 0.55), cy + int(scale * 0.55))


def make_mark(size=1024):
    """Pure constellation mark on Deep Space."""
    img = Image.new("RGBA", (size, size), DEEP_SPACE + (255,))
    draw = ImageDraw.Draw(img)
    scale = size * 0.55
    cx = size // 2
    cy = size // 2
    draw_constellation(draw, cx, cy, scale)
    return img


def make_favicon_amber_dot(size=1024):
    """Single amber dot on TRANSPARENT background. Per Brand.md §7.10."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    r = int(size * 0.025)
    cx, cy = size // 2, size // 2
    draw.ellipse(circle_center(cx, cy, r), fill=SIGNAL_AMBER)
    return img


def make_tray_icon(size=512):
    """Monochrome amber triangle glyph on transparent background."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    scale = size * 0.28
    cx, cy = size // 2, size // 2

    t = math.radians(6)
    cos_t, sin_t = math.cos(t), math.sin(t)
    base_width = 0.60 * scale
    top_y = -0.38 * scale
    apex_x = 0.02 * scale
    apex_y = 0.34 * scale

    raw = [
        (-base_width / 2, top_y),
        ( base_width / 2, top_y),
        ( apex_x, apex_y),
    ]

    nodes = []
    node_r = max(2, int(scale * 0.07))
    for x, y in raw:
        rx = x * cos_t - y * sin_t + cx
        ry = x * sin_t + y * cos_t + cy
        nodes.append((int(rx), int(ry)))

    line_w = max(1, int(scale * 0.035))
    amber_a = SIGNAL_AMBER + (255,)
    for a, b in [(0, 2), (1, 2), (0, 1)]:
        x1, y1 = nodes[a]
        x2, y2 = nodes[b]
        sx, sy = line_endpoint(x1, y1, x2, y2, node_r + 1)
        ex, ey = line_endpoint(x2, y2, x1, y1, node_r + 1)
        draw.line([(sx, sy), (ex, ey)], fill=amber_a, width=line_w)

    for nx, ny in nodes:
        draw.ellipse(circle_center(nx, ny, node_r), fill=amber_a)

    return img


def make_notification_icon(size=512):
    """White triangle glyph on transparent for push notifications."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    scale = size * 0.28
    cx, cy = size // 2, size // 2

    t = math.radians(6)
    cos_t, sin_t = math.cos(t), math.sin(t)
    base_width = 0.60 * scale
    top_y = -0.38 * scale
    apex_x = 0.02 * scale
    apex_y = 0.34 * scale

    raw = [
        (-base_width / 2, top_y),
        ( base_width / 2, top_y),
        ( apex_x, apex_y),
    ]

    nodes = []
    node_r = max(2, int(scale * 0.07))
    for x, y in raw:
        rx = x * cos_t - y * sin_t + cx
        ry = x * sin_t + y * cos_t + cy
        nodes.append((int(rx), int(ry)))

    line_w = max(1, int(scale * 0.035))
    white = (255, 255, 255, 255)
    for a, b in [(0, 2), (1, 2), (0, 1)]:
        x1, y1 = nodes[a]
        x2, y2 = nodes[b]
        sx, sy = line_endpoint(x1, y1, x2, y2, node_r + 1)
        ex, ey = line_endpoint(x2, y2, x1, y1, node_r + 1)
        draw.line([(sx, sy), (ex, ey)], fill=white, width=line_w)

    for nx, ny in nodes:
        draw.ellipse(circle_center(nx, ny, node_r), fill=white)

    return img


def make_app_icon(size=1024):
    """Constellation mark centered on Deep Space with padding, for mobile app icon."""
    img = Image.new("RGBA", (size, size), DEEP_SPACE + (255,))
    draw = ImageDraw.Draw(img)
    scale = size * 0.40
    cx, cy = size // 2, size // 2
    draw_constellation(draw, cx, cy, scale)
    return img


def try_load_font(name, size):
    """Try to load a font by name or family, fall back to default."""
    font_paths = [
        # Brand
        f"C:\\Windows\\Fonts\\{name}.ttf",
        f"C:\\Windows\\Fonts\\{name}.otf",
    ]
    # System fallbacks for serif (editorial) and mono
    family_fallbacks = {
        "Cinzel-Bold": ["cambriab.ttf", "cambriaz.ttf"],
        "IBMPlexMono-Regular": ["CascadiaMono.ttf", "consola.ttf"],
    }
    if name in family_fallbacks:
        for fb in family_fallbacks[name]:
            fp = f"C:\\Windows\\Fonts\\{fb}"
            if os.path.exists(fp):
                font_paths.append(fp)

    for p in font_paths:
        if os.path.exists(p):
            return ImageFont.truetype(p, size)
    return ImageFont.load_default()


def draw_text_tracked(draw, text, x, y, font, fill, tracking_pct=0.0):
    """Draw text with approximate letter-spacing (tracking). Pillow doesn't natively support it."""
    if tracking_pct == 0.0:
        draw.text((x, y), text, fill=fill, font=font)
        return
    cur_x = x
    for ch in text:
        draw.text((cur_x, y), ch, fill=fill, font=font)
        ch_bbox = draw.textbbox((0, 0), ch, font=font)
        ch_w = ch_bbox[2] - ch_bbox[0]
        cur_x += int(ch_w * (1 + tracking_pct / 100.0))


def make_lockup_stacked(size=2048):
    """Full stacked lockup: mark + GEARBOX + 1px amber rule + RELAY."""
    img = Image.new("RGBA", (size, size), DEEP_SPACE + (255,))
    draw = ImageDraw.Draw(img)

    cx, cy = size // 2, size // 2

    # Mark: fit in upper portion
    mark_scale = size * 0.20
    mark_cy = int(cy - size * 0.24)
    draw_constellation(draw, cx, mark_cy, mark_scale)

    # Fonts
    font_brand = try_load_font("Cinzel-Bold", int(size * 0.07))
    font_product = try_load_font("IBMPlexMono-Regular", int(size * 0.028))

    text_brand = "GEARBOX"
    text_product = "RELAY"

    # Measure actual wordmark width
    brand_bbox = draw.textbbox((0, 0), text_brand, font=font_brand)
    brand_w = brand_bbox[2] - brand_bbox[0]
    brand_h = brand_bbox[3] - brand_bbox[1]
    relay_bbox = draw.textbbox((0, 0), text_product, font=font_product)
    relay_w = relay_bbox[2] - relay_bbox[0]

    # Amber rule line: 1px thick, matching wordmark width
    rule_y = int(cy + size * 0.03)
    rule_w = brand_w * 1.05  # approximate tracking compensation
    rule_x1 = int(cx - rule_w / 2)
    rule_x2 = int(cx + rule_w / 2)
    draw.line([(rule_x1, rule_y), (rule_x2, rule_y)], fill=SIGNAL_AMBER, width=1)

    # Gap from mark bottom to wordmark: ~40% of mark height
    # mark bottom = mark_cy + mark_scale/2, wordmark top = rule_y - brand_h - gap
    # This naturally works out from layout.

    # GEARBOX above rule (tracking +5%)
    brand_x = cx - brand_w // 2
    brand_y = rule_y - brand_h - int(size * 0.018)
    draw_text_tracked(draw, text_brand, brand_x, brand_y, font_brand, STRUCTURE_WHITE, tracking_pct=5.0)

    # RELAY below rule (tracking +10%)
    relay_x = cx - relay_w // 2
    relay_y = rule_y + int(size * 0.018)
    draw_text_tracked(draw, text_product, relay_x, relay_y, font_product,
                      dim(STRUCTURE_WHITE, 0.6), tracking_pct=10.0)

    return img


def make_lockup_horizontal(size_w=2048, size_h=1152):
    """Horizontal lockup: mark left of GEARBOX RELAY."""
    img = Image.new("RGBA", (size_w, size_h), DEEP_SPACE + (255,))
    draw = ImageDraw.Draw(img)

    cx, cy = size_w // 2, size_h // 2
    mark_scale = size_h * 0.42

    # Mark on the left portion
    mark_cx = int(cx - size_w * 0.19)
    draw_constellation(draw, mark_cx, cy, mark_scale)

    font_brand = try_load_font("Cinzel-Bold", int(size_h * 0.11))
    font_product = try_load_font("IBMPlexMono-Regular", int(size_h * 0.045))

    text_brand = "GEARBOX"
    text_product = "RELAY"

    brand_bbox = draw.textbbox((0, 0), text_brand, font=font_brand)
    brand_h = brand_bbox[3] - brand_bbox[1]
    brand_w = brand_bbox[2] - brand_bbox[0]
    relay_bbox = draw.textbbox((0, 0), text_product, font=font_product)
    relay_h = relay_bbox[3] - relay_bbox[1]
    relay_w = relay_bbox[2] - relay_bbox[0]
    total_text_h = brand_h + relay_h + int(size_h * 0.025)

    text_base_y = cy - total_text_h // 2
    text_x = int(cx + size_w * 0.03)

    draw_text_tracked(draw, text_brand, text_x, text_base_y, font_brand,
                      STRUCTURE_WHITE, tracking_pct=5.0)

    relay_y = text_base_y + brand_h + int(size_h * 0.025)
    draw_text_tracked(draw, text_product, text_x + int(size_h * 0.02), relay_y,
                      font_product, dim(STRUCTURE_WHITE, 0.6), tracking_pct=10.0)

    return img


def make_og_image(size_w=1200, size_h=630):
    """Social sharing card per Brand.md §7.10: mark + declaration text + amber signal line."""
    img = Image.new("RGBA", (size_w, size_h), DEEP_SPACE + (255,))
    draw = ImageDraw.Draw(img)

    cx, cy = size_w // 2, size_h // 2

    # Mark: small, left-of-center
    mark_scale = size_h * 0.24
    mark_cx = int(cx - size_w * 0.23)
    draw_constellation(draw, mark_cx, cy, mark_scale)

    # Amber signal line (vertical rule separating mark from text)
    rule_h = int(size_h * 0.45)
    rule_x = int(cx - size_w * 0.08)
    rule_y1 = int(cy - rule_h / 2)
    rule_y2 = int(cy + rule_h / 2)
    draw.line([(rule_x, rule_y1), (rule_x, rule_y2)], fill=SIGNAL_AMBER, width=1)

    # Fonts
    font_declaration = try_load_font("Cinzel-Bold", int(size_h * 0.085))
    font_sub = try_load_font("IBMPlexMono-Regular", int(size_h * 0.04))

    # Declaration per Brand.md §7.10
    declaration = "Your signal. Your structure. Your machine."
    sub_text = "gearbox.ai"

    text_x = int(rule_x + size_w * 0.03)
    text_y = int(cy - size_h * 0.12)

    draw.text((text_x, text_y), declaration, fill=STRUCTURE_WHITE, font=font_declaration)

    sub_y = text_y + int(size_h * 0.12)
    draw.text((text_x, sub_y), sub_text, fill=SIGNAL_AMBER, font=font_sub)

    return img


def save_all():
    os.makedirs(OUT, exist_ok=True)

    print("Generating assets with exact pixel geometry...")

    # Constellation mark — 1024×1024
    make_mark(1024).save(os.path.join(OUT, "gearbox-mark-1024.png"))
    print("  gearbox-mark-1024.png")

    # Favicon master — amber dot
    make_favicon_amber_dot(1024).save(os.path.join(OUT, "gearbox-favicon-master-1024.png"))
    print("  gearbox-favicon-master-1024.png")

    # Favicon at 32×32 and 180×180
    make_favicon_amber_dot(1024).resize((32, 32), Image.LANCZOS).save(os.path.join(OUT, "gearbox-favicon-32.png"))
    print("  gearbox-favicon-32.png")
    make_favicon_amber_dot(1024).resize((180, 180), Image.LANCZOS).save(os.path.join(OUT, "gearbox-favicon-180.png"))
    print("  gearbox-favicon-180.png")

    # App icon
    make_app_icon(1024).save(os.path.join(OUT, "gearbox-app-icon-1024.png"))
    print("  gearbox-app-icon-1024.png")

    # Tray icon at multiple sizes
    for sz in [16, 24, 32]:
        make_tray_icon(512).resize((sz, sz), Image.LANCZOS).save(os.path.join(OUT, f"gearbox-tray-{sz}.png"))
        print(f"  gearbox-tray-{sz}.png")

    # Notification icon
    make_notification_icon(512).save(os.path.join(OUT, "gearbox-notif-master-512.png"))
    print("  gearbox-notif-master-512.png")
    make_notification_icon(512).resize((24, 24), Image.LANCZOS).save(os.path.join(OUT, "gearbox-notif-24.png"))
    print("  gearbox-notif-24.png")

    # Stacked lockup
    make_lockup_stacked(2048).save(os.path.join(OUT, "gearbox-lockup-stacked-2048.png"))
    print("  gearbox-lockup-stacked-2048.png")

    # Horizontal lockup
    make_lockup_horizontal(2048, 1152).save(os.path.join(OUT, "gearbox-lockup-horizontal-2048x1152.png"))
    print("  gearbox-lockup-horizontal-2048x1152.png")

    # OG image
    make_og_image(1200, 630).save(os.path.join(OUT, "gearbox-og-1200x630.png"))
    print("  gearbox-og-1200x630.png")

    # Downscale mark to all icon sizes
    mark_master = make_mark(1024)
    for sz in [512, 256, 128, 64, 48, 32, 24, 16]:
        mark_master.resize((sz, sz), Image.LANCZOS).save(os.path.join(OUT, f"gearbox-mark-{sz}.png"))
        print(f"  gearbox-mark-{sz}.png")

    # Downscale app icon
    app_master = make_app_icon(1024)
    for sz in [512, 256, 128]:
        app_master.resize((sz, sz), Image.LANCZOS).save(os.path.join(OUT, f"gearbox-app-icon-{sz}.png"))
        print(f"  gearbox-app-icon-{sz}.png")

    print("\nDone. All assets in brand-assets/generated/")


if __name__ == "__main__":
    save_all()
