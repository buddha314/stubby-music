"""Rebuild the transparent sunflower HUD reticle (requires Pillow)."""
from pathlib import Path
import math
from PIL import Image, ImageDraw

ROOT = Path(__file__).resolve().parents[2]
SIZE, SCALE = 96, 4
# Native drawing coordinates are the intended 28 logical-pixel HUD size.
paths = []
for petal in range(12):
    angle = petal * math.tau / 12
    points = []
    for i in range(65):
        t = i * math.tau / 64
        radial = 7.1 + 3.0 * math.cos(t)
        tangent = 1.35 * math.sin(t)
        points.append((14 + radial * math.cos(angle) - tangent * math.sin(angle),
                       14 + radial * math.sin(angle) + tangent * math.cos(angle)))
    paths.append(points)
image = Image.new('RGBA', (SIZE * SCALE, SIZE * SCALE))
draw = ImageDraw.Draw(image)
factor = SIZE * SCALE / 28
for color, width in [((20, 22, 20, 190), 1.5), ((242, 239, 221, 240), .65)]:
    for points in paths:
        draw.line([(round(x * factor), round(y * factor)) for x, y in points],
                  fill=color, width=round(width * factor), joint='curve')
    bounds = [(14 - 3.0) * factor, (14 - 3.0) * factor,
              (14 + 3.0) * factor, (14 + 3.0) * factor]
    draw.ellipse(bounds, outline=color, width=round(width * factor))
image.resize((SIZE, SIZE), Image.Resampling.LANCZOS).save(ROOT / 'assets/ui/sunflower_reticle.png')
svg_paths = '\n'.join('<path d="M ' + ' L '.join(f'{x:.3f},{y:.3f}' for x,y in points) + ' Z"/>' for points in paths)
shapes = svg_paths + '\n<circle cx="14" cy="14" r="3"/>'
(ROOT / 'assets/ui/sunflower_reticle.svg').write_text(
    '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 28 28" fill="none">\n'
    '<g stroke="#141614" stroke-opacity=".75" stroke-width="1.5">' + shapes + '</g>\n'
    '<g stroke="#f2efdd" stroke-opacity=".94" stroke-width=".65">' + shapes + '</g></svg>\n')
