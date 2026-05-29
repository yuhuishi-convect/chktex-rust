#!/usr/bin/env bash
# Generate upstream config-lookup fixtures (see upstream tests/run-tests.sh).
#
# Upstream does not commit these files; run-tests.sh creates them at test time.

set -euo pipefail

UPSTREAM_DIR="${1:-${CHKTEX_UPSTREAM_DIR:-/tmp/chktex-upstream/chktex/chktex}}"
TESTS_DIR="$UPSTREAM_DIR/tests"

mkdir -p "$TESTS_DIR/sub"
cat >"$TESTS_DIR/sub/chktexrc" <<'EOF'
OutFormat
{
"loaded chktex/tests/sub %f!n"
}
EOF

mkdir -p "$TESTS_DIR/sub1/.config"
cat >"$TESTS_DIR/sub1/.config/chktexrc" <<'EOF'
OutFormat
{
"loaded chktex/tests/sub1/.config/chktexrc %f!n"
}
EOF

mkdir -p "$TESTS_DIR/sub2"
cat >"$TESTS_DIR/sub2/.chktexrc" <<'EOF'
OutFormat
{
"loaded chktex/tests/sub2/.chktexrc %f!n"
}
EOF

echo "Generated config lookup fixtures under $TESTS_DIR"
