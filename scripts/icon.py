#!/usr/bin/env python3
"""Render the app icon set from the wordmark's "n".

    scripts/icon.py

Reads assets/notch.txt, cuts the first letter out (8 cells wide, leaning one
column per row), writes it to assets/n.txt, and renders it with JetBrains
Mono at line-height 0.9 (the wordmark's own spacing, so rows overlap into
solid strokes the way zwipe's Z icon does) onto gruvbox dark, cream on
#282828. Writes every size iOS, the web, and Android need into
assets/favicon/, plus an inset variant for Android's adaptive mask.

Needs Pillow and fonttools: pip3 install pillow fonttools brotli
"""

import pathlib
import tempfile

from fontTools.ttLib import TTFont
from PIL import Image, ImageDraw, ImageFont

ROOT = pathlib.Path(__file__).resolve().parent.parent
BG, FG = "#282828", "#ebdbb2"
SIZES = [16, 32, 40, 60, 80, 87, 120, 180, 192, 512]


def mark_from_wordmark() -> list[str]:
    """The icon mark is the wordmark's first letter, kept in assets/n.txt."""
    return (ROOT / "assets/n.txt").read_text().rstrip("\n").split("\n")


def ttf_path() -> str:
    font = TTFont(ROOT / "assets/fonts/jetbrains-mono-400.woff2")
    font.flavor = None
    out = pathlib.Path(tempfile.gettempdir()) / "notch-jbm-400.ttf"
    font.save(out)
    return str(out)


def render(lines: list[str], canvas: int, fill: float, ttf: str, line_h: float = 0.9) -> Image.Image:
    cols = max(len(l) for l in lines)
    n = len(lines)
    # Cells are 0.6em wide; the block glyphs span 1.32em, rows step 0.9em.
    em = min(canvas * fill / (cols * 0.6), canvas * fill / ((n - 1) * line_h + 1.32))
    font = ImageFont.truetype(ttf, int(em))
    em = font.size
    w = cols * 0.6 * em
    h = (n - 1) * line_h * em + 1.32 * em
    img = Image.new("RGB", (canvas, canvas), BG)
    draw = ImageDraw.Draw(img)
    x0 = (canvas - w) / 2
    y0 = (canvas - h) / 2 + 1.02 * em  # baseline sits 1.02em below the ink top
    for i, line in enumerate(lines):
        draw.text((x0, y0 + i * line_h * em), line, font=font, fill=FG, anchor="ls")
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


if __name__ == "__main__":
    main()
