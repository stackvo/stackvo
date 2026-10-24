#!/usr/bin/env python3
"""Build `src-tauri/icons/icon.icns` from `icon.png` in the shape macOS wants.

## Why the macOS icon is not the same file as the others

`icon.png` is the orange square edge to edge, which is what Windows, Linux
and the tray want. macOS wants something else: Apple's template is a
1024x1024 canvas with the icon drawn as an 824x824 rounded square in the
middle — a transparent margin of 100px on every side, corner radius about
185px. Every icon in the Dock is that shape (VS Code's and PhpStorm's
measure exactly 100..923 on both axes), and macOS 26 checks for it. An icon
that is not that shape is treated as a legacy one: scaled down and drawn on
a grey system backing, which is how StackVo sat in the Dock looking like a
sticker on a tile while its neighbours filled their slots. The full-bleed
PNG, handed to the bundler as the `.icns`, was that non-conforming icon.

So this draws the orange at Apple's geometry, resamples the white S from the
source into it, writes the 1024 PNG beside the others as `icon-macos.png`,
and turns it into the `.icns` with the system's own `iconutil`. No image
library: a PNG is a zlib stream with a filter byte per row, and 512 in,
1024 out is small enough to do by hand in a few seconds.

    python3 tools/mac-icon.py          # rewrites icon-macos.png and icon.icns

The `.icns` step is macOS only — `sips` and `iconutil` ship with it.
"""
import math
import os
import struct
import subprocess
import sys
import tempfile
import zlib

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICONS = os.path.join(ROOT, "src-tauri", "icons")
SRC = os.path.join(ICONS, "icon.png")
PNG_OUT = os.path.join(ICONS, "icon-macos.png")
ICNS_OUT = os.path.join(ICONS, "icon.icns")

CANVAS = 1024
MARGIN = 100
SHAPE = CANVAS - 2 * MARGIN  # 824
RADIUS = 185.0
ORANGE = (255, 136, 0)


def read_png(path):
    data = open(path, "rb").read()
    pos, idat = 8, b""
    while pos < len(data):
        (length,) = struct.unpack(">I", data[pos : pos + 4])
        kind = data[pos + 4 : pos + 8]
        body = data[pos + 8 : pos + 8 + length]
        if kind == b"IHDR":
            w, h, depth, colour = struct.unpack(">IIBB", body[:10])
        if kind == b"IDAT":
            idat += body
        pos += 12 + length
    if (depth, colour) != (8, 6):
        sys.exit(f"{path}: need 8-bit RGBA, got depth {depth} colour type {colour}")
    raw = zlib.decompress(idat)
    bpp, stride = 4, w * 4
    rows, prev, i = [], bytearray(stride), 0

    def paeth(a, b, c):
        pa, pb, pc = abs(b - c), abs(a - c), abs(a + b - 2 * c)
        return a if pa <= pb and pa <= pc else (b if pb <= pc else c)

    for _ in range(h):
        f = raw[i]
        i += 1
        line = bytearray(raw[i : i + stride])
        i += stride
        if f:
            for x in range(stride):
                a = line[x - bpp] if x >= bpp else 0
                b = prev[x]
                c = prev[x - bpp] if x >= bpp else 0
                if f == 1:
                    line[x] = (line[x] + a) & 255
                elif f == 2:
                    line[x] = (line[x] + b) & 255
                elif f == 3:
                    line[x] = (line[x] + (a + b) // 2) & 255
                else:
                    line[x] = (line[x] + paeth(a, b, c)) & 255
        rows.append(line)
        prev = line
    return w, h, rows


def write_png(path, size, rgba):
    def chunk(kind, body):
        crc = zlib.crc32(kind + body) & 0xFFFFFFFF
        return struct.pack(">I", len(body)) + kind + body + struct.pack(">I", crc)

    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    png = b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", header)
    png += chunk(b"IDAT", zlib.compress(bytes(rgba), 9)) + chunk(b"IEND", b"")
    open(path, "wb").write(png)


def main():
    sw, sh, src = read_png(SRC)
    # The S is white on orange (255,136,0): the blue channel alone says how
    # white a pixel is, scaled by its alpha.
    cover = [[src[y][x * 4 + 2] * src[y][x * 4 + 3] / 65025.0 for x in range(sw)] for y in range(sh)]

    def sample(u, v):
        x0, y0 = math.floor(u), math.floor(v)
        fx, fy = u - x0, v - y0

        def at(x, y):
            return cover[y][x] if 0 <= x < sw and 0 <= y < sh else 0.0

        top = at(x0, y0) * (1 - fx) + at(x0 + 1, y0) * fx
        bottom = at(x0, y0 + 1) * (1 - fx) + at(x0 + 1, y0 + 1) * fx
        return top * (1 - fy) + bottom * fy

    out = bytearray()
    half, inner = SHAPE / 2, SHAPE / 2 - RADIUS
    for y in range(CANVAS):
        out.append(0)  # filter: none
        py = y + 0.5 - MARGIN
        for x in range(CANVAS):
            px = x + 0.5 - MARGIN
            qx, qy = abs(px - half) - inner, abs(py - half) - inner
            dist = math.hypot(max(qx, 0.0), max(qy, 0.0)) + min(max(qx, qy), 0.0) - RADIUS
            alpha = min(1.0, max(0.0, 0.5 - dist))  # one-pixel anti-aliased edge
            if alpha <= 0.0:
                out += b"\0\0\0\0"
                continue
            inside = 0 <= px < SHAPE and 0 <= py < SHAPE
            c = sample(px / SHAPE * sw - 0.5, py / SHAPE * sh - 0.5) if inside else 0.0
            out += bytes(
                (
                    round(ORANGE[0] + (255 - ORANGE[0]) * c),
                    round(ORANGE[1] + (255 - ORANGE[1]) * c),
                    round(ORANGE[2] + (255 - ORANGE[2]) * c),
                    round(alpha * 255),
                )
            )
    write_png(PNG_OUT, CANVAS, out)
    print(f"wrote {os.path.relpath(PNG_OUT, ROOT)}")

    if sys.platform != "darwin":
        print("not macOS: icon.icns left alone (needs sips and iconutil)")
        return
    with tempfile.TemporaryDirectory() as tmp:
        iconset = os.path.join(tmp, "icon.iconset")
        os.mkdir(iconset)
        for size in (16, 32, 128, 256, 512):
            for scale in (1, 2):
                px = size * scale
                name = f"icon_{size}x{size}{'@2x' if scale == 2 else ''}.png"
                subprocess.run(
                    ["sips", "-z", str(px), str(px), PNG_OUT, "--out", os.path.join(iconset, name)],
                    check=True,
                    stdout=subprocess.DEVNULL,
                )
        subprocess.run(["iconutil", "-c", "icns", "-o", ICNS_OUT, iconset], check=True)
    print(f"wrote {os.path.relpath(ICNS_OUT, ROOT)}")


if __name__ == "__main__":
    main()
