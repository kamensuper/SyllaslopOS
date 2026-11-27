"""Extract printable-ASCII glyph outlines from DejaVu Sans Mono 2.37.

The kernel rasterizes these quadratic outlines at any pixel size, so text
stays smooth at every zoom level. This script needs only the Python standard
library. Normal kernel builds use the committed asset and do not run it.

Usage: python3 tools/bake-outlines.py [path/to/DejaVuSansMono.ttf]

Asset layout (little endian):
    0  b"SLOF"          magic
    4  u16 version      1
    6  u16 units_per_em
    8  u16 first        first code point (32)
   10  u16 count        number of glyphs (95)
   12  u16 advance      monospaced advance width in font units
   14  u16 reserved
   16  u32 offsets[count + 1], relative to the start of the asset
   then for each glyph:
       u16 contours
       per contour: u16 points, then points * (i16 x, i16 y, u8 on_curve)

Every contour starts with an on-curve point, and every off-curve point is
followed by an on-curve point (wrapping to the first point at the end).
Implied TrueType on-curve midpoints are made explicit here.
"""
from pathlib import Path
import struct
import sys

FIRST, LAST = 32, 126
DEFAULT_FONT = Path("/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf")
TARGET = Path(__file__).resolve().parents[1] / "kernel/graphics/assets/mono.outline"


class Font:
    def __init__(self, data: bytes):
        self.data = data
        count = struct.unpack_from(">H", data, 4)[0]
        self.tables = {}
        for index in range(count):
            tag, _, offset, length = struct.unpack_from(">4sIII", data, 12 + 16 * index)
            self.tables[tag.decode()] = (offset, length)
        head = self.tables["head"][0]
        self.units_per_em = struct.unpack_from(">H", data, head + 18)[0]
        self.long_loca = struct.unpack_from(">h", data, head + 50)[0] == 1
        self.glyph_count = struct.unpack_from(">H", data, self.tables["maxp"][0] + 4)[0]
        self.cmap = self._cmap()

    def _cmap(self) -> dict[int, int]:
        base = self.tables["cmap"][0]
        count = struct.unpack_from(">H", self.data, base + 2)[0]
        for index in range(count):
            platform, encoding, offset = struct.unpack_from(">HHI", self.data, base + 4 + 8 * index)
            if (platform, encoding) in {(3, 1), (0, 3), (0, 4)}:
                table = base + offset
                if struct.unpack_from(">H", self.data, table)[0] == 4:
                    return self._format4(table)
        raise ValueError("no format 4 Unicode cmap")

    def _format4(self, table: int) -> dict[int, int]:
        segments = struct.unpack_from(">H", self.data, table + 6)[0] // 2
        ends = table + 14
        starts = ends + 2 * segments + 2
        deltas = starts + 2 * segments
        ranges = deltas + 2 * segments
        result = {}
        for seg in range(segments):
            end = struct.unpack_from(">H", self.data, ends + 2 * seg)[0]
            start = struct.unpack_from(">H", self.data, starts + 2 * seg)[0]
            delta = struct.unpack_from(">h", self.data, deltas + 2 * seg)[0]
            range_offset = struct.unpack_from(">H", self.data, ranges + 2 * seg)[0]
            for code in range(start, min(end, 0xFFFE) + 1):
                if range_offset == 0:
                    glyph = (code + delta) & 0xFFFF
                else:
                    at = ranges + 2 * seg + range_offset + 2 * (code - start)
                    glyph = struct.unpack_from(">H", self.data, at)[0]
                    if glyph:
                        glyph = (glyph + delta) & 0xFFFF
                result[code] = glyph
        return result

    def advance(self, glyph: int) -> int:
        metrics = struct.unpack_from(">H", self.data, self.tables["hhea"][0] + 34)[0]
        index = min(glyph, metrics - 1)
        return struct.unpack_from(">H", self.data, self.tables["hmtx"][0] + 4 * index)[0]

    def _glyph_range(self, glyph: int) -> tuple[int, int]:
        loca = self.tables["loca"][0]
        if self.long_loca:
            start, end = struct.unpack_from(">II", self.data, loca + 4 * glyph)
        else:
            start, end = (2 * v for v in struct.unpack_from(">HH", self.data, loca + 2 * glyph))
        base = self.tables["glyf"][0]
        return base + start, base + end

    def contours(self, glyph: int, depth: int = 0) -> list[list[tuple[int, int, bool]]]:
        """Return raw TrueType contours as (x, y, on_curve) in font units."""
        if depth > 8:
            raise ValueError("composite glyph nesting is too deep")
        start, end = self._glyph_range(glyph)
        if start == end:
            return []
        count = struct.unpack_from(">h", self.data, start)[0]
        if count >= 0:
            return self._simple(start, count)
        return self._composite(start, depth)

    def _simple(self, at: int, count: int) -> list[list[tuple[int, int, bool]]]:
        data = self.data
        ends = list(struct.unpack_from(f">{count}H", data, at + 10))
        total = ends[-1] + 1 if ends else 0
        cursor = at + 10 + 2 * count
        instructions = struct.unpack_from(">H", data, cursor)[0]
        cursor += 2 + instructions
        flags = []
        while len(flags) < total:
            flag = data[cursor]
            cursor += 1
            repeat = 1
            if flag & 8:
                repeat += data[cursor]
                cursor += 1
            flags.extend([flag] * repeat)
        flags = flags[:total]

        def coordinates(short_bit: int, same_bit: int) -> list[int]:
            nonlocal cursor
            values, value = [], 0
            for flag in flags:
                if flag & short_bit:
                    step = data[cursor]
                    cursor += 1
                    value += step if flag & same_bit else -step
                elif not flag & same_bit:
                    value += struct.unpack_from(">h", data, cursor)[0]
                    cursor += 2
                values.append(value)
            return values

        xs = coordinates(2, 16)
        ys = coordinates(4, 32)
        result, first = [], 0
        for last in ends:
            result.append([(xs[i], ys[i], bool(flags[i] & 1)) for i in range(first, last + 1)])
            first = last + 1
        return result

    def _composite(self, at: int, depth: int) -> list[list[tuple[int, int, bool]]]:
        data = self.data
        cursor = at + 10
        result = []
        while True:
            flags, glyph = struct.unpack_from(">HH", data, cursor)
            cursor += 4
            if flags & 1:
                dx, dy = struct.unpack_from(">hh", data, cursor)
                cursor += 4
            else:
                dx, dy = struct.unpack_from(">bb", data, cursor)
                cursor += 2
            if not flags & 2:
                raise ValueError("point-matched composite components are unsupported")
            a, b, c, d = 1.0, 0.0, 0.0, 1.0
            if flags & 8:
                a = d = struct.unpack_from(">h", data, cursor)[0] / 16384
                cursor += 2
            elif flags & 0x40:
                a, d = (v / 16384 for v in struct.unpack_from(">hh", data, cursor))
                cursor += 4
            elif flags & 0x80:
                a, b, c, d = (v / 16384 for v in struct.unpack_from(">hhhh", data, cursor))
                cursor += 8
            for contour in self.contours(glyph, depth + 1):
                result.append([
                    (round(a * x + c * y + dx), round(b * x + d * y + dy), on)
                    for x, y, on in contour
                ])
            if not flags & 0x20:
                return result


def explicit(contour: list[tuple[int, int, bool]]) -> list[tuple[int, int, bool]]:
    """Insert implied on-curve midpoints and rotate to start on-curve."""
    if not contour:
        return []
    count = len(contour)
    points = []
    for index, point in enumerate(contour):
        nxt = contour[(index + 1) % count]
        points.append(point)
        if not point[2] and not nxt[2]:
            points.append(((point[0] + nxt[0]) // 2, (point[1] + nxt[1]) // 2, True))
    start = next(i for i, p in enumerate(points) if p[2])
    points = points[start:] + points[:start]
    for index, point in enumerate(points):
        assert point[2] or points[(index + 1) % len(points)][2], "unpaired control point"
    return points


def bake(font_path: Path) -> bytes:
    font = Font(font_path.read_bytes())
    advance = font.advance(font.cmap[ord("M")])
    glyphs = []
    for code in range(FIRST, LAST + 1):
        glyph = font.cmap.get(code, 0)
        assert font.advance(glyph) == advance, f"{chr(code)!r} is not monospaced"
        record = bytearray()
        contours = [explicit(c) for c in font.contours(glyph)]
        contours = [c for c in contours if len(c) >= 2]
        record += struct.pack("<H", len(contours))
        for contour in contours:
            record += struct.pack("<H", len(contour))
            for x, y, on in contour:
                record += struct.pack("<hhB", x, y, 1 if on else 0)
        glyphs.append(bytes(record))

    count = len(glyphs)
    header = struct.pack("<4sHHHHHH", b"SLOF", 1, font.units_per_em, FIRST, count, advance, 0)
    offset = len(header) + 4 * (count + 1)
    offsets = []
    for record in glyphs:
        offsets.append(offset)
        offset += len(record)
    offsets.append(offset)
    return header + struct.pack(f"<{count + 1}I", *offsets) + b"".join(glyphs)


def main() -> int:
    source = Path(sys.argv[1]) if len(sys.argv) > 1 else DEFAULT_FONT
    TARGET.write_bytes(bake(source))
    print(f"wrote {TARGET} ({TARGET.stat().st_size} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
