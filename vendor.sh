#!/bin/sh
# Regenerates the vendored C sources and the icon constants from an upstream
# CUI checkout. Run from the repository root:
#
#     ./vendor.sh [path-to-CUI-checkout]
#
# The default checkout path is ../CUI-clone. Requires python3. After running,
# commit the updated cui-sys/c/ files (and cui/src/icons.rs) together.
set -e

upstream="${1:-../CUI-clone}"
out="$(mktemp -d)"
trap 'rm -rf "$out"' EXIT

python3 "$upstream/tools/amalgamate.py" "$out" >/dev/null
cp "$out/cui.h" "$out/cui.c" cui-sys/c/

python3 - "$upstream" <<'EOF'
import re, sys, pathlib

header = pathlib.Path(sys.argv[1], "include/cui/cui_icons.h").read_text()
pat = re.compile(r'#define CUI_ICON_([A-Z0-9_]+)\s+"((?:\\x[0-9a-f]{2})+)"')

lines = [
    "//! Lucide icon constants, generated from `cui_icons.h` (see `vendor.sh`).",
    "//! Each icon is a UTF-8 string usable anywhere text goes, e.g. `ICON_PLUS`.",
    "#![allow(missing_docs)] // generated constants, named after their C macros",
    "",
]
for name, hexes in pat.findall(header):
    ch = bytes(int(h, 16) for h in hexes.split("\\x")[1:]).decode("utf-8")
    lines.append(f'pub const ICON_{name}: &str = "{ch}";')
lines.append("")
pathlib.Path("cui/src/icons.rs").write_text("\n".join(lines))
print(f"{len(pat.findall(header))} icons generated")
EOF

echo "vendored cui-sys/c/{cui.h,cui.c} and cui/src/icons.rs from $upstream"
