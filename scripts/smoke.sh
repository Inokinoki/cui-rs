#!/bin/sh
# Smoke test: run each example for a few seconds under the display in $DISPLAY
# (wrap the call in xvfb-run on a headless machine) and fail if any of them
# crashes, exits early or stops drawing. When xwd is installed, a screenshot
# of every example lands in $SMOKE_OUT (default /tmp/cui-smoke) for eyeballing
# and CI artifacts.
#
#     xvfb-run -a scripts/smoke.sh target/release
#
# Arguments override the default example list.
set -u

dir=${1:?usage: smoke.sh <directory-with-example-binaries>}
shift 2>/dev/null || true
examples=${*:-"assistant caffeine dashboard finance gallery hello mail music notes settings todo tracker"}

: "${SMOKE_OUT:=/tmp/cui-smoke}"
mkdir -p "$SMOKE_OUT"
: "${SMOKE_SECONDS:=3}"

# Xvfb's GLX has no depth buffer without a compositor; ask for the software
# renderer explicitly so CI measures the app, not the driver stack.
CUI_RENDERER=software
export CUI_RENDERER

status=0
for name in $examples; do
    exe="$dir/$name"
    if [ ! -x "$exe" ]; then
        echo "FAIL $name: no executable at $exe"
        status=1
        continue
    fi
    if [ -f "$dir/../todo.txt" ]; then rm -f "$dir/../todo.txt"; fi
    "$exe" &
    pid=$!
    sleep "$SMOKE_SECONDS"
    if kill -0 "$pid" 2>/dev/null; then
        echo "ok   $name"
        if command -v xwd >/dev/null 2>&1 && [ -n "${DISPLAY:-}" ]; then
            xwd -root -silent -out "$SMOKE_OUT/$name.xwd" 2>/dev/null && \
                [ -s "$SMOKE_OUT/$name.xwd" ] || true
        fi
    else
        wait "$pid"
        code=$?
        echo "FAIL $name: exited early with code $code"
        status=1
    fi
    kill "$pid" 2>/dev/null
    wait "$pid" 2>/dev/null
done
rm -f "$dir/../todo.txt"
exit $status
