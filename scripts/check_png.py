#!/usr/bin/env python3
"""Checks PNG files decode cleanly (zlib IDAT stream, chunk CRCs).

Used by CI over the gallery's screenshot-mode output, so a broken PNG
writer (wrong zlib checksum, truncated stream) fails the build instead of
shipping unusable artifacts.

    python3 scripts/check_png.py <file.png> [more.png ...]
"""
import struct
import sys
import zlib

PNG_SIG = b"\x89PNG\r\n\x1a\n"


def crc32(data: bytes) -> int:
    crc = 0xFFFF_FFFF
    for b in data:
        crc ^= b
        for _ in range(8):
            crc = (crc >> 1) ^ (0xEDB8_8320 & -(crc & 1))
    return crc ^ 0xFFFF_FFFF


def check(path: str) -> None:
    d = open(path, "rb").read()
    if d[:8] != PNG_SIG:
        raise SystemExit(f"{path}: not a PNG (bad signature)")
    i, idat, tags, ihdr, ended = 8, b"", [], None, False
    while i < len(d):
        if ended:
            raise SystemExit(f"{path}: data after IEND")
        if i + 12 > len(d):
            raise SystemExit(f"{path}: truncated chunk header")
        ln = struct.unpack(">I", d[i:i + 4])[0]
        tag, data = d[i + 4:i + 8], d[i + 8:i + 8 + ln]
        if len(data) != ln:
            raise SystemExit(f"{path}: truncated {tag} chunk")
        if crc32(tag + data) != struct.unpack(">I", d[i + 8 + ln:i + 12 + ln])[0]:
            raise SystemExit(f"{path}: bad chunk CRC for {tag}")
        if not tags and tag != b"IHDR":
            raise SystemExit(f"{path}: first chunk is {tag}, not IHDR")
        tags.append(tag)
        if tag == b"IHDR":
            if ln != 13:
                raise SystemExit(f"{path}: IHDR is {ln} bytes, not 13")
            ihdr = struct.unpack(">IIBBBBB", data)  # w, h, depth, color, comp, filter, interlace
        if tag == b"IDAT":
            idat += data
        if tag == b"IEND":
            ended = True
        i += 12 + ln
    if not ended:
        raise SystemExit(f"{path}: no IEND chunk (truncated file?)")
    if ihdr is None or not idat:
        raise SystemExit(f"{path}: missing IHDR or IDAT")
    raw = zlib.decompress(idat)  # raises on a bad Adler-32 / corrupt stream
    w, h, depth, color, _, _, interlace = ihdr
    channels = {0: 1, 2: 3, 4: 2, 6: 4}.get(color)
    if depth == 8 and channels and interlace == 0:
        want = h * (1 + w * channels)
        if len(raw) != want:
            raise SystemExit(f"{path}: {w}x{h} image needs {want} bytes of pixel data, found {len(raw)}")


if __name__ == "__main__":
    if len(sys.argv) < 2:
        raise SystemExit(__doc__)
    for p in sys.argv[1:]:
        check(p)
    print(f"{len(sys.argv) - 1} PNG(s) decode cleanly")
