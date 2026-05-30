#!/usr/bin/env bash
# Build ChkTeX WASM packages for browser (web) and Node (nodejs) targets.
#
# Usage:
#   ./tools/build-wasm.sh              # release -> pkg/ and pkg-node/
#   ./tools/build-wasm.sh debug        # debug builds

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
PROFILE="${1:-release}"
CRATE="$ROOT/crates/chktex-wasm"
WEB_OUT="$ROOT/pkg"
NODE_OUT="$ROOT/pkg-node"

"$ROOT/tools/rustup.sh" target add wasm32-unknown-unknown >/dev/null

cd "$ROOT"

build_with_wasm_pack() {
    local target="$1"
    local out="$2"
    local flags=()
    if [[ "$PROFILE" == "debug" ]]; then
        flags+=(--dev)
    else
        flags+=(--release)
    fi
    wasm-pack build "$CRATE" "${flags[@]}" --target "$target" --out-dir "$out"
}

if command -v wasm-pack >/dev/null 2>&1; then
    echo ">>> Building web package -> $WEB_OUT"
    build_with_wasm_pack web "$WEB_OUT"
    echo ">>> Building nodejs package -> $NODE_OUT"
    build_with_wasm_pack nodejs "$NODE_OUT"
    echo
    echo "Browser glue: $WEB_OUT/chktex_wasm.js"
    echo "Node glue:    $NODE_OUT/chktex_wasm.js"
    cp "$ROOT/LICENSE" "$WEB_OUT/"
    cp "$ROOT/LICENSE" "$NODE_OUT/"
    echo "Integration:  integrations/browser/chktex.js"
    echo "Demo:         examples/wasm/index.html (serve repo root)"
else
    echo "wasm-pack not found; building .wasm only (no JS glue)" >&2
    if [[ "$PROFILE" == "debug" ]]; then
        cargo build -p chktex-wasm --target wasm32-unknown-unknown
        wasm="$ROOT/target/wasm32-unknown-unknown/debug/chktex_wasm.wasm"
    else
        cargo build -p chktex-wasm --release --target wasm32-unknown-unknown
        wasm="$ROOT/target/wasm32-unknown-unknown/release/chktex_wasm.wasm"
    fi
    echo "Built: $wasm"
    echo "Install wasm-pack for JS interop: cargo install wasm-pack" >&2
    exit 1
fi
