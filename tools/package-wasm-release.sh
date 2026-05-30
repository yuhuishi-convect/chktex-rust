#!/usr/bin/env bash
# Package browser WASM artifacts for GitHub Releases.
#
# Usage:
#   ./tools/package-wasm-release.sh <archive-base-name>
#
# Example:
#   ./tools/build-wasm.sh
#   ./tools/package-wasm-release.sh chktex-wasm-0.1.1

set -euo pipefail

if [[ $# -ne 1 ]]; then
    echo "usage: $(basename "$0") <archive-base-name>" >&2
    exit 1
fi

NAME="$1"
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DIST="$ROOT/dist"
STAGING="$DIST/$NAME"

for path in pkg/chktex_wasm.js integrations/browser/chktex.js examples/wasm/example.release.html; do
    if [[ ! -f "$ROOT/$path" ]]; then
        echo "error: missing $path — run tools/build-wasm.sh first" >&2
        exit 1
    fi
done

rm -rf "$STAGING"
mkdir -p "$STAGING/integrations/browser"
cp -r "$ROOT/pkg" "$STAGING/"
cp "$ROOT/integrations/browser/chktex.js" \
   "$ROOT/integrations/browser/chktex.d.ts" \
   "$ROOT/integrations/browser/package.json" \
   "$STAGING/integrations/browser/"
cp "$ROOT/LICENSE" "$STAGING/"
cp "$ROOT/examples/wasm/example.release.html" "$STAGING/example.html"
cp "$ROOT/examples/wasm/WASM.release.md" "$STAGING/WASM.md"

mkdir -p "$DIST"
tar -C "$DIST" -czf "$DIST/${NAME}.tar.gz" "$NAME"
echo "$DIST/${NAME}.tar.gz"
