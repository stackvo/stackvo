#!/usr/bin/env bash
# One-time, per machine: git does not track .git/hooks, so cloning this repo
# elsewhere (or a fresh checkout on a second machine) does not carry the
# pre-push hook with it. Run this once after cloning:
#
#   tools/git-hooks/install.sh
set -euo pipefail

repo="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ln -sf ../../tools/git-hooks/pre-push "$repo/.git/hooks/pre-push"
chmod +x "$repo/tools/git-hooks/pre-push"
echo "pre-push hook installed -> tools/before-push.sh will run before every push."
