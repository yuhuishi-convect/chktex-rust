#!/bin/bash
# Load oracle paths for chktex-rust compatibility tests.
#
# Priority:
#   1. Existing CHKTEX_ORACLE / CHKTEX_UPSTREAM_DIR environment variables
#   2. target/oracle.env written by tools/setup-oracle.sh
#   3. Legacy defaults under /tmp/chktex-upstream

_chktex_rust_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if [[ -z "${CHKTEX_ORACLE:-}" || -z "${CHKTEX_UPSTREAM_DIR:-}" ]]; then
    if [[ -f "$_chktex_rust_root/target/oracle.env" ]]; then
        # shellcheck disable=SC1091
        source "$_chktex_rust_root/target/oracle.env"
    fi
fi

: "${CHKTEX_ORACLE:=/tmp/chktex-upstream/chktex/chktex/chktex}"
: "${CHKTEX_UPSTREAM_DIR:=/tmp/chktex-upstream/chktex/chktex}"
export CHKTEX_ORACLE CHKTEX_UPSTREAM_DIR

unset _chktex_rust_root
