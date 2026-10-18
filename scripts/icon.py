#!/usr/bin/env python3
"""Render the app icon set from the wordmark's "c".

    scripts/icon.py

Reads the mark in assets/c.txt and renders it with JetBrains
Mono at line-height 0.9 (the wordmark's own spacing, so rows overlap into
solid strokes the way zwipe's Z icon does) onto gruvbox dark, cream on
#282828. Writes every size iOS, the web, and Android need into
assets/favicon/, plus an inset variant for Android's adaptive mask.

Also writes one 1024 icon per orangui theme and mode into assets/app-icons/,
each theme's text on its background in that mode, read from the themes.css of
the orangui build Cargo.lock pins. scripts/ios/icons.sh compiles them into
the alternate icons the App icon sheet switches between.

Needs Pillow and fonttools: pip3 install pillow fonttools brotli
"""

import json
import pathlib
import re
import subprocess
import tempfile

from fontTools.ttLib import TTFont
from PIL import Image, ImageDraw, ImageFont

ROOT = pathlib.Path(__file__).resolve().parent.parent
BG, FG = "#282828", "#ebdbb2"
PRIMARY = ("gruvbox", "dark")
SIZES = [16, 32, 40, 60, 80, 87, 120, 180, 192, 512]


def mark_from_wordmark() -> list[str]:
    """The icon mark is the wordmark's first letter, kept in assets/c.txt."""
    return (ROOT / "assets/c.txt").read_text().rstrip("\n").split("\n")


def ttf_path() -> str:
    font = TTFont(ROOT / "assets/fonts/jetbrains-mono-400.woff2")
    font.flavor = None
    out = pathlib.Path(tempfile.gettempdir()) / "cairn-jbm-400.ttf"
    font.save(out)
    return str(out)


def palettes() -> dict[tuple[str, str], tuple[str, str]]:
    """Each (theme, mode)'s (background, text) from the pinned orangui themes.css."""
    meta = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1"],
            cwd=ROOT, check=True, capture_output=True, text=True,
        ).stdout
    )
    manifest = next(p["manifest_path"] for p in meta["packages"] if p["name"] == "orangui")
    css = (pathlib.Path(manifest).parent / "assets/themes.css").read_text()
    out = {}
    for name, mode, body in re.findall(
        r"^\.theme-([a-z0-9-]+)-(dark|light)\s*\{(.*?)^\}", css, re.M | re.S
    ):
        bg = re.search(r"--bg-primary:\s*(#[0-9a-fA-F]{6})", body)
        fg = re.search(r"--text-primary:\s*(#[0-9a-fA-F]{6})", body)
        if bg and fg:
            out[(name, mode)] = (bg.group(1), fg.group(1))
    return out


def render(
    lines: list[str], canvas: int, fill: float, ttf: str,
    line_h: float = 0.9, bg: str = BG, fg: str = FG,
) -> Image.Image:
    cols = max(len(l) for l in lines)
    n = len(lines)
    # Cells are 0.6em wide; the block glyphs span 1.32em, rows step 0.9em.
    em = min(canvas * fill / (cols * 0.6), canvas * fill / ((n - 1) * line_h + 1.32))
    font = ImageFont.truetype(ttf, int(em))
    em = font.size
    w = cols * 0.6 * em
    h = (n - 1) * line_h * em + 1.32 * em
    img = Image.new("RGB", (canvas, canvas), bg)
    draw = ImageDraw.Draw(img)
    x0 = (canvas - w) / 2
    y0 = (canvas - h) / 2 + 1.02 * em  # baseline sits 1.02em below the ink top
    for i, line in enumerate(lines):
        draw.text((x0, y0 + i * line_h * em), line, font=font, fill=fg, anchor="ls")
    return img


def main() -> None:
    mark = mark_from_wordmark()
    ttf = ttf_path()
    out = ROOT / "assets/favicon"
    out.mkdir(exist_ok=True)
    icon = render(mark, 1024, 0.74, ttf)
    icon.save(out / "icon-1024.png")
    render(mark, 1024, 0.52, ttf).save(out / "icon-1024-android.png")
    for px in SIZES:
        icon.resize((px, px), Image.LANCZOS).save(out / f"icon-{px}.png")
    icon.resize((64, 64), Image.LANCZOS).save(
        out / "favicon.ico", sizes=[(16, 16), (32, 32), (48, 48), (64, 64)]
    )
    print("wrote", sorted(p.name for p in out.iterdir()))

    alternates = ROOT / "assets/app-icons"
    alternates.mkdir(exist_ok=True)
    for stale in alternates.glob("*.png"):
        stale.unlink()
    for (name, mode), (bg, fg) in sorted(palettes().items()):
        if (name, mode) != PRIMARY:
            render(mark, 1024, 0.74, ttf, bg=bg, fg=fg).save(alternates / f"{name}-{mode}.png")
    print("wrote", len(list(alternates.glob("*.png"))), "alternate icons")


if __name__ == "__main__":
    main()
