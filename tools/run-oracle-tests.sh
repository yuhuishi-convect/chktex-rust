#!/bin/bash
# Run differential oracle tests against the upstream C chktex binary.
#
# Build the oracle first with:
#   ./tools/setup-oracle.sh

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
# shellcheck disable=SC1091
source "$ROOT/tools/oracle-env.sh"

if [[ ! -f "$CHKTEX_ORACLE" ]]; then
    echo "error: upstream oracle not found at $CHKTEX_ORACLE" >&2
    echo "Run ./tools/setup-oracle.sh first, or set CHKTEX_ORACLE." >&2
    exit 1
fi

if [[ ! -d "$CHKTEX_UPSTREAM_DIR" ]]; then
    echo "error: upstream source dir not found at $CHKTEX_UPSTREAM_DIR" >&2
    echo "Run ./tools/setup-oracle.sh first, or set CHKTEX_UPSTREAM_DIR." >&2
    exit 1
fi

cd "$ROOT"
echo "Oracle:   $CHKTEX_ORACLE"
echo "Fixtures: $CHKTEX_UPSTREAM_DIR"
if [[ -n "${CHKTEX_UPSTREAM_COMMIT:-}" ]]; then
    echo "Commit:   $CHKTEX_UPSTREAM_COMMIT"
fi
echo

cargo test -p chktex-cli --test oracle -- --ignored --nocapture
