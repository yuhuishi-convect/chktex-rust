#!/usr/bin/env bash
# Package a release binary with default chktexrc and docs.
#
# Usage:
#   ./tools/package-release.sh <binary-path> <archive-base-name>
#
# Example:
#   ./tools/package-release.sh target/release/chktex chktex-0.1.0-x86_64-unknown-linux-gnu

set -euo pipefail

if [[ $# -ne 2 ]]; then
    echo "usage: $(basename "$0") <binary-path> <archive-base-name>" >&2
    exit 1
fi

BINARY="$1"
NAME="$2"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
STAGING="$DIST/$NAME"

if [[ ! -f "$BINARY" ]]; then
    echo "error: binary not found: $BINARY" >&2
    exit 1
fi

rm -rf "$STAGING"
mkdir -p "$STAGING"
cp "$BINARY" "$STAGING/$(basename "$BINARY")"
cp "$ROOT/tests/fixtures/upstream/chktexrc" "$STAGING/chktexrc"
cp "$ROOT/LICENSE" "$ROOT/README.md" "$STAGING/"

mkdir -p "$DIST"
if [[ "$BINARY" == *.exe ]]; then
    archive="$DIST/${NAME}.zip"
    if command -v zip >/dev/null 2>&1; then
        (cd "$DIST" && zip -qr "${NAME}.zip" "$NAME")
    else
        python3 - "$DIST" "$NAME" "$archive" <<'PY'
import shutil, sys
dist, name, archive = sys.argv[1:4]
shutil.make_archive(f"{dist}/{name}", "zip", dist, name)
PY
    fi
    echo "$archive"
else
    tar -C "$DIST" -czf "$DIST/${NAME}.tar.gz" "$NAME"
    echo "$DIST/${NAME}.tar.gz"
fi
