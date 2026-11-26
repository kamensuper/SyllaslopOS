"""Regenerate the bitmap assets from DejaVu 2.37 with Pillow."""
from pathlib import Path
import sys

from PIL import Image, ImageDraw, ImageFont

target = Path(__file__).resolve().parents[1] / "kernel/graphics/assets"
fonts = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("/usr/share/fonts/truetype/dejavu")

for name, size, width, height in [("body", 15, 9, 18), ("small", 13, 8, 16)]:
    font = ImageFont.truetype(str(fonts / "DejaVuSansMono.ttf"), size)
    result = bytearray()
    for code in range(32, 127):
        glyph = Image.new("L", (width, height))
        ImageDraw.Draw(glyph).text((width // 2, height - 4), chr(code), anchor="ms", font=font, fill=255)
        result.extend(glyph.tobytes())
    (target / f"{name}.mask").write_bytes(result)
