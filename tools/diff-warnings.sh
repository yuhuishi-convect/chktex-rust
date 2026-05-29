#!/bin/bash
# Compare warnings line by line between upstream and Rust chktex
# Usage: ./tools/diff-warnings.sh [testfile]

set -euo pipefail

TESTFILE="${1:-tests/fixtures/upstream/Test.tex}"
RCFILE="tests/fixtures/upstream/chktexrc"
ORACLE="${CHKTEX_ORACLE:-/tmp/chktex-upstream/chktex/chktex}"
UPSTREAM_DIR="${CHKTEX_UPSTREAM_DIR:-/tmp/chktex-upstream/chktex}"

echo "=== Extracting warnings per line from upstream ==="
"$ORACLE" -mall -r -g0 -l "$RCFILE" -v5 -q "$TESTFILE" 2>/dev/null \
  | grep "^Message\|^Warning\|^Error" \
  | sed 's/.*line \([0-9]*\):.*/\1/' \
  > /tmp/upstream-warnings.tmp

echo "=== Extracting warnings per line from Rust ==="
cargo run -- -mall -r -g0 -l "$RCFILE" -v5 -q "$TESTFILE" 2>/dev/null \
  | grep "^Message\|^Warning\|^Error" \
  | sed 's/.*line \([0-9]*\):.*/\1/' \
  > /tmp/rust-warnings.tmp

echo "=== Lines with different warning counts ==="
echo "Line | Upstream | Rust | Difference"
echo "-----|----------|------|-----------"

# Compare line by line
total_lines=$(wc -l < "$TESTFILE")
for line in $(seq 1 $total_lines); do
  up_count=$(grep -c "^$line$" /tmp/upstream-warnings.tmp 2>/dev/null || echo 0)
  rust_count=$(grep -c "^$line$" /tmp/rust-warnings.tmp 2>/dev/null || echo 0)
  if [ "$up_count" != "$rust_count" ]; then
    diff=$((rust_count - up_count))
    printf "%4d | %9d | %4d | %+d\n" "$line" "$up_count" "$rust_count" "$diff"
  fi
done

echo ""
echo "=== Total warnings ==="
echo "Upstream: $(wc -l < /tmp/upstream-warnings.tmp)"
echo "Rust:     $(wc -l < /tmp/rust-warnings.tmp)"
echo ""
echo "=== Full diff output ==="
cargo run -- -mall -r -g0 -l "$RCFILE" -v5 -q "$TESTFILE" 2>/dev/null > /tmp/rust-out.txt
"$ORACLE" -mall -r -g0 -l "$RCFILE" -v5 -q "$TESTFILE" 2>/dev/null > /tmp/oracle-out.txt
diff /tmp/oracle-out.txt /tmp/rust-out.txt 2>&1 | head -50
