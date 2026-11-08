#!/usr/bin/env bash
# REL2 uses real gh Release API/downloads; this never tags/dispatches/publishes.
set -euo pipefail
script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
export PYTHONDONTWRITEBYTECODE=1
exec python3 "$script_dir/verify-release.py" "$@"
