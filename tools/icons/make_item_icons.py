"""Item icons for the inventory and the rack's Relevant strip.

One PNG per item at assets/ui/icons/<item_id>.png (the game finds an item's icon by
its ID). Drawn in the rack's panel language: amber line glyphs on a dark tile, the
same square-wave glyph as the VCO slot label. Reproduce: python3 tools/icons/make_item_icons.py
"""
import math
from pathlib import Path

from PIL import Image, ImageDraw

OUT = Path(__file__).resolve().parents[2] / "assets/ui/icons"
SIZE, SS = 128, 4  # output size, supersampling
AMBER = (255, 178, 79, 255)
BONE = (230, 222, 200, 255)
GREEN = (120, 160, 120, 255)
TILE = (28, 26, 23, 255)
EDGE = (90, 70, 45, 255)


def canvas():
    s = SIZE * SS
    img = Image.new("RGBA", (s, s), (0, 0, 0, 0))
    d = ImageDraw.Draw(img)
    d.rounded_rectangle((4 * SS, 4 * SS, s - 4 * SS, s - 4 * SS), radius=18 * SS, fill=TILE,
                        outline=EDGE, width=3 * SS)
    return img, d


def px(*pts):
    return [(x * SS, y * SS) for x, y in pts]


def square_wave(d):
    d.line(px((22, 82), (40, 82), (40, 46), (64, 46), (64, 82), (88, 82), (88, 46), (106, 46)),
           fill=AMBER, width=8 * SS, joint="curve")


def patch_cable(d):
    pts = [(30 + 68 * t, 40 + 52 * math.sin(math.pi * t)) for t in (i / 40 for i in range(41))]
    d.line(px(*pts), fill=BONE, width=7 * SS, joint="curve")
    for x in (30, 98):
        d.rounded_rectangle(px((x - 8, 22), (x + 8, 44)), radius=4 * SS, fill=AMBER)
        d.line(px((x, 14), (x, 22)), fill=BONE, width=3 * SS)


def reed(d):
    for x0, lean in ((46, -10), (62, 2), (78, 12)):
        d.line(px((x0, 104), (x0 + lean, 28)), fill=GREEN, width=5 * SS)
        d.ellipse(px((x0 + lean - 5, 26), (x0 + lean + 5, 46)), fill=AMBER)


def clock(d):
    d.ellipse(px((28, 28), (100, 100)), outline=AMBER, width=7 * SS)
    d.line(px((64, 64), (64, 40)), fill=AMBER, width=6 * SS)
    d.line(px((64, 64), (82, 72)), fill=AMBER, width=6 * SS)


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    for name, draw in {"square_wave": square_wave, "patch_cable": patch_cable,
                       "reed": reed, "clock": clock}.items():
        img, d = canvas()
        draw(d)
        img.resize((SIZE, SIZE), Image.LANCZOS).save(OUT / f"{name}.png")
        print("wrote", OUT / f"{name}.png")


if __name__ == "__main__":
    main()
