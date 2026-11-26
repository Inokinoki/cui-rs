#!/bin/sh
# Manage the upstream CUI sources this crate builds against. Run from the
# repository root:
#
#   ./vendor.sh --pin <tag>          Pin a release: download its amalgamated
#                                    assets, verify the version, compute the
#                                    SHA-256 digests, update the pin in
#                                    UPSTREAM.toml, and refresh the offline
#                                    cache in cui-sys/c/ and the license
#                                    texts in cui-sys/licenses/ and cui/licenses/.
#   ./vendor.sh --fetch              Re-download the pinned release into the
#                                    offline cache (cui-sys/c/), verifying the
#                                    recorded digests. For building offline.
#   ./vendor.sh <checkout-path>      Amalgamate a local upstream checkout into
#                                    the offline cache instead (for hacking on
#                                    upstream; the version must match the pin).
#
# The files it writes into cui-sys/c/ are committed for the default,
# network-less build; build.rs verifies them against the digests recorded in
# UPSTREAM.toml. Requires curl and python3.
set -e

repo_path=yappologistic/CUI
repo=https://github.com/$repo_path
raw=https://raw.githubusercontent.com/$repo_path
api=https://api.github.com/repos/$repo_path
upstream_dir=cui-sys/c

toml_field() {
    sed -n "s/^$1 *= *\"\(.*\)\"/\1/p" cui-sys/UPSTREAM.toml | head -1
}

write_pin() {
    # Rewrite the pin's keys in place, so the comments and `supported` — the
    # window build.rs refuses a version outside — stay as they are.
    version=$1 tag=$2 sha_h=$3 sha_c=$4
    sed \
        -e "s|^version *=.*|version = \"$version\"|" \
        -e "s|^tag *=.*|tag = \"$tag\"|" \
        -e "s|^base_url *=.*|base_url = \"$repo/releases/download\"|" \
        -e "s|^sha256_cui_h *=.*|sha256_cui_h = \"$sha_h\"|" \
        -e "s|^sha256_cui_c *=.*|sha256_cui_c = \"$sha_c\"|" \
        cui-sys/UPSTREAM.toml > "$out/UPSTREAM.toml"
    cp "$out/UPSTREAM.toml" cui-sys/UPSTREAM.toml
    grep -q "^supported *=.*\"$version\"" cui-sys/UPSTREAM.toml || echo \
        "note: \`supported\` in cui-sys/UPSTREAM.toml does not list $version; build.rs refuses to compile until it does" >&2
}

run_probe() {
    # Record the C ABI of what is in cui-sys/c/ as a TOML table; the layout
    # tests in cui-sys check the Rust FFI structs against it. gen_probe.c
    # (generated alongside gen.rs) covers the generated structs too.
    v=$(sed -n 's/^#define CUI_VERSION_STRING "\(.*\)"/\1/p' "$upstream_dir/cui.h" | head -1)
    mkdir -p cui-sys/layout
    cat cui-sys/probe/layout_probe.c cui-sys/probe/gen_probe.c > "$out/probe_all.c"
    echo 'int main(void) { base_probe(); gen_probe_all(); return 0; }' >> "$out/probe_all.c"
    cc -std=c99 -I "$upstream_dir" "$out/probe_all.c" -o "$out/probe"
    "$out/probe" > "cui-sys/layout/$v.toml"
    echo "layout probe table written: cui-sys/layout/$v.toml"
    # every Rust extern vs the header's prototype (scripts/gen_ffi.py -> sigcheck.c)
    scripts/sigcheck.sh
}

fetch_into() {
    # $1 = destination dir, uses the pin already in UPSTREAM.toml
    tag=$(toml_field tag)
    base=$(toml_field base_url)
    want_h=$(toml_field sha256_cui_h)
    want_c=$(toml_field sha256_cui_c)
    mkdir -p "$1"
    curl --fail --location --proto '=https' --silent --show-error \
        -o "$1/cui.h" "$base/$tag/cui.h"
    curl --fail --location --proto '=https' --silent --show-error \
        -o "$1/cui.c" "$base/$tag/cui.c"
    got_h=$(sha256sum "$1/cui.h" | cut -d' ' -f1)
    got_c=$(sha256sum "$1/cui.c" | cut -d' ' -f1)
    [ "$got_h" = "$want_h" ] || { echo "error: cui.h digest mismatch (expected $want_h, got $got_h)" >&2; exit 1; }
    [ "$got_c" = "$want_c" ] || { echo "error: cui.c digest mismatch (expected $want_c, got $got_c)" >&2; exit 1; }
}

fetch_licenses() {
    # $1 = tag, $2 = directory to fill: the license texts of the fonts and
    # icons the amalgamation embeds, as the upstream tree keeps them next to
    # the font files (assets/fonts)
    mkdir -p "$2"
    listing=$(curl --fail --location --proto '=https' --silent --show-error \
            "$api/contents/assets/fonts?ref=$1") || {
        echo "error: could not list the license texts at $1 (assets/fonts); the pin is unchanged" >&2
        exit 1
    }
    for name in $(printf '%s\n' "$listing" | python3 -c 'import json, sys
items = json.load(sys.stdin)
if not isinstance(items, list):
    sys.exit("assets/fonts is not a directory listing at this tag")
for item in items:
    if item["name"].startswith("LICENSE-") and item["name"].endswith(".txt"):
        print(item["name"])'); do
        curl --fail --location --proto '=https' --silent --show-error \
            -o "$2/$name" "$raw/$1/assets/fonts/$name"
    done
    [ -e "$2/LICENSE-Lucide.txt" ] || {
        echo "error: no Lucide license text at $1 (assets/fonts); the pin is unchanged" >&2
        exit 1
    }
}

copy_licenses() {
    # $1 = directory of LICENSE-*.txt: the fonts sit in cui-sys, the icon
    # strings in cui come from Lucide
    mkdir -p cui-sys/licenses cui/licenses
    cp "$1"/LICENSE-*.txt cui-sys/licenses/
    cp "$1/LICENSE-Lucide.txt" cui/licenses/
}

case "${1:-}" in
--pin)
    tag=${2:?usage: ./vendor.sh --pin <tag>}
    case "$tag" in v*) ;; *) echo "error: tag should look like v1.4.0" >&2; exit 1 ;; esac
    version=${tag#v}
    out=$(mktemp -d)
    trap 'rm -rf "$out"' EXIT
    curl --fail --location --proto '=https' --silent --show-error \
        -o "$out/cui.h"  "$repo/releases/download/$tag/cui.h"
    curl --fail --location --proto '=https' --silent --show-error \
        -o "$out/cui.c"  "$repo/releases/download/$tag/cui.c"
    header_version=$(sed -n 's/^#define CUI_VERSION_STRING "\(.*\)"/\1/p' "$out/cui.h" | head -1)
    [ "$header_version" = "$version" ] || {
        echo "error: the $tag assets carry CUI $header_version, not $version" >&2; exit 1;
    }
    fetch_licenses "$tag" "$out/licenses"
    write_pin "$version" "$tag" \
        "$(sha256sum "$out/cui.h" | cut -d' ' -f1)" \
        "$(sha256sum "$out/cui.c" | cut -d' ' -f1)"
    mkdir -p "$upstream_dir"
    cp "$out/cui.h" "$out/cui.c" "$upstream_dir/"
    copy_licenses "$out/licenses"
    run_probe
    echo "pinned $tag (cui $version) in cui-sys/UPSTREAM.toml; offline cache refreshed"
    ;;
--fetch)
    out=$(mktemp -d)
    trap 'rm -rf "$out"' EXIT
    fetch_into "$upstream_dir"
    run_probe
    echo "offline cache refreshed from the pinned release ($(toml_field version))"
    ;;
"")
    sed -n '2,12p' "$0"
    exit 1
    ;;
*)
    upstream=$1
    out=$(mktemp -d)
    trap 'rm -rf "$out"' EXIT
    python3 "$upstream/tools/amalgamate.py" "$out" >/dev/null
    header_version=$(sed -n 's/^#define CUI_VERSION_STRING "\(.*\)"/\1/p' "$out/cui.h" | head -1)
    pinned=$(toml_field version)
    if [ "$header_version" != "$pinned" ]; then
        echo "warning: the checkout amalgamates to CUI $header_version but the pin is $pinned;" \
             "build.rs will refuse this cache until the pin matches" >&2
    fi
    mkdir -p "$upstream_dir"
    cp "$out/cui.h" "$out/cui.c" "$upstream_dir/"
    [ -d "$upstream/assets/fonts" ] && copy_licenses "$upstream/assets/fonts"
    run_probe
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
count = 0
for name, hexes in pat.findall(header):
    ch = bytes(int(h, 16) for h in hexes.split("\\x")[1:]).decode("utf-8")
    lines.append(f'pub const ICON_{name}: &str = "{ch}";')
    count += 1
lines.append("")
pathlib.Path("cui/src/icons.rs").write_text("\n".join(lines))
print(f"{count} icons generated")
EOF
    echo "cui-sys/c/ rebuilt from the local checkout ($upstream); build.rs compiles it with a digest-mismatch warning"
    ;;
esac
