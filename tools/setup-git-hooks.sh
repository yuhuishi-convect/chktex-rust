#!/bin/bash
# Enable repository git hooks from .githooks/

set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

chmod +x .githooks/pre-commit
git config --local core.hooksPath .githooks

echo "Git hooks enabled: core.hooksPath=.githooks"
echo "Pre-commit will run: cargo fmt --all"
