#!/bin/bash
# Run rustup with a clean environment.
#
# Cursor's AppImage rustup proxy breaks `rustup target add` in some shells.
# Use this wrapper from build scripts instead of calling rustup directly.

set -euo pipefail

exec env -i \
    HOME="${HOME:?HOME must be set}" \
    USER="${USER:-$(id -un)}" \
    PATH="/usr/bin:/bin:/usr/lib/rustup/bin:${HOME}/.cargo/bin" \
    RUSTUP_HOME="${RUSTUP_HOME:-${HOME}/.rustup}" \
    CARGO_HOME="${CARGO_HOME:-${HOME}/.cargo}" \
    /usr/bin/rustup "$@"
