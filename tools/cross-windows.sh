#!/bin/bash
# Cross-compile chktex for Windows from Linux/macOS.
#
# Default flavor uses cargo-xwin (MSVC target, no mingw install required):
#   ./tools/cross-windows.sh
#   ./tools/cross-windows.sh msvc
#
# GNU/mingw flavor (requires mingw-w64-gcc):
#   ./tools/cross-windows.sh gnu
#
# Output:
#   target/x86_64-pc-windows-msvc/release/chktex.exe
#   target/x86_64-pc-windows-gnu/release/chktex.exe

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
FLAVOR="${1:-msvc}"
CARGO="${CARGO:-cargo}"

TARGET_MSVC="x86_64-pc-windows-msvc"
TARGET_GNU="x86_64-pc-windows-gnu"

usage() {
    cat <<EOF
Usage: $(basename "$0") [msvc|gnu]

Cross-compile the chktex CLI for Windows.

Flavors:
  msvc   Use cargo-xwin and the MSVC ABI (default, recommended)
  gnu    Use mingw-w64 and the GNU ABI

Examples:
  $(basename "$0")
  $(basename "$0") gnu
  make release-windows
EOF
}

ensure_toolchain_target() {
    local target="$1"
    if "$ROOT/tools/rustup.sh" target list --installed | grep -qx "$target"; then
        return 0
    fi
    echo ">>> Installing Rust target $target"
    "$ROOT/tools/rustup.sh" target add "$target"
}

require_cargo_xwin() {
    if ! command -v cargo-xwin >/dev/null 2>&1; then
        echo "error: cargo-xwin not found" >&2
        echo "Install with: cargo install cargo-xwin --locked" >&2
        exit 1
    fi
}

require_mingw() {
    if ! command -v x86_64-w64-mingw32-gcc >/dev/null 2>&1; then
        echo "error: x86_64-w64-mingw32-gcc not found" >&2
        echo "On Arch/CachyOS: sudo pacman -S mingw-w64-gcc" >&2
        exit 1
    fi
}

build_msvc() {
    require_cargo_xwin
    ensure_toolchain_target "$TARGET_MSVC"
    echo ">>> Building $TARGET_MSVC release binary with cargo-xwin"
    cd "$ROOT"
    cargo xwin build --release -p chktex-cli --target "$TARGET_MSVC"
    local out="$ROOT/target/$TARGET_MSVC/release/chktex.exe"
    echo
    echo "Built: $out"
    file "$out"
}

build_gnu() {
    require_mingw
    ensure_toolchain_target "$TARGET_GNU"
    echo ">>> Building $TARGET_GNU release binary with mingw-w64"
    cd "$ROOT"
    "$CARGO" build --release -p chktex-cli --target "$TARGET_GNU"
    local out="$ROOT/target/$TARGET_GNU/release/chktex.exe"
    echo
    echo "Built: $out"
    file "$out"
}

case "$FLAVOR" in
    msvc | windows | win)
        build_msvc
        ;;
    gnu | mingw)
        build_gnu
        ;;
    -h | --help | help)
        usage
        ;;
    *)
        echo "error: unknown flavor: $FLAVOR" >&2
        usage >&2
        exit 1
        ;;
esac
