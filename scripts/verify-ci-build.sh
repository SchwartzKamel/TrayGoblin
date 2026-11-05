#!/usr/bin/env bash
# CI2's live GitHub boundary; --self-test is offline evidence only.
set -euo pipefail
repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
export PYTHONDONTWRITEBYTECODE=1
exec python3 -B "${repository_root}/scripts/verify-ci-build.py" "$@"
