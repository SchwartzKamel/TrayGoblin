#!/usr/bin/env bash
# Real offline pi host→shipped Rust journey, plus SEPARATE OCI CLI-only evidence.
set -euo pipefail
[[ $# -eq 2 && $1 == --container && $2 == podman ]] || {
  echo 'usage: bash scripts/demo-attention.sh --container podman' >&2; exit 2;
}
cd "$(dirname "${BASH_SOURCE[0]}")/.."
command -v podman >/dev/null || { echo 'Podman unavailable; OCI validation did NOT run.' >&2; exit 1; }
command -v pwsh >/dev/null || { echo 'PowerShell required for persistent installer controls.' >&2; exit 1; }
# Author/parent may select an existing verified engine store under isolated HOME.
# No image pull, user-config mount or host SDK mount is performed.
engine=(podman)
if [[ -n ${TRAY_GOBLIN_PODMAN_ROOT:-} ]]; then
  : "${TRAY_GOBLIN_PODMAN_RUNROOT:?set the matching cached engine runroot}"
  engine+=(--root "$TRAY_GOBLIN_PODMAN_ROOT" --runroot "$TRAY_GOBLIN_PODMAN_RUNROOT")
fi
"${engine[@]}" image exists docker.io/library/debian:bookworm-slim || {
  echo 'Cached Debian bookworm-slim unavailable; no pulling and no OCI claim.' >&2; exit 1;
}
evidence=${TRAY_GOBLIN_EVIDENCE_DIR:-$(mktemp -d /tmp/traygoblin-attention-evidence-XXXXXX)}
mkdir -p "$evidence"
evidence=$(cd "$evidence" && pwd)
case "$evidence/" in "$PWD/"*) echo 'Evidence must be outside the checkout.' >&2; exit 2;; esac
# Freeze the EXISTING engine cache before isolating all application state. This
# neither pulls an image nor exposes the engine store inside the CLI container.
TRAY_GOBLIN_PODMAN_ROOT=${TRAY_GOBLIN_PODMAN_ROOT:-$("${engine[@]}" info --format '{{.Store.GraphRoot}}')}
TRAY_GOBLIN_PODMAN_RUNROOT=${TRAY_GOBLIN_PODMAN_RUNROOT:-$("${engine[@]}" info --format '{{.Store.RunRoot}}')}
export TRAY_GOBLIN_PODMAN_ROOT TRAY_GOBLIN_PODMAN_RUNROOT
export CARGO_HOME=${CARGO_HOME:-$HOME/.cargo} RUSTUP_HOME=${RUSTUP_HOME:-$HOME/.rustup}
unset CARGO_TARGET_DIR
export CARGO_NET_OFFLINE=true
export HOME="$evidence/home" USERPROFILE="$evidence/home"
export XDG_CONFIG_HOME="$HOME/.config" XDG_DATA_HOME="$HOME/.local/share"
export XDG_CACHE_HOME="$HOME/.cache" XDG_STATE_HOME="$HOME/.local/state"
export PI_CODING_AGENT_DIR="$HOME/.pi/agent"
mkdir -p "$HOME" "$XDG_CONFIG_HOME" "$XDG_DATA_HOME" "$XDG_CACHE_HOME" "$XDG_STATE_HOME" "$PI_CODING_AGENT_DIR"
# The probe and driver must be built here; never silently reuse a stale binary.
cargo build --release --bin tray-goblin-probe --bin tray-goblin-attention-demo
bash scripts/package-release.sh 0.1.0
pwsh -NoProfile -File scripts/test-installer.ps1
node scripts/test-release-docs.mjs
payload="$evidence/extracted"
mkdir -p "$payload"
unzip -q -o dist/tray-goblin-0.1.0-windows-x86_64.zip -d "$payload"
# Install the actual archive into a synthetic per-user root, then remove the
# extraction. The journey must load the PERSISTED publisher/resources, not source.
pwsh -NoProfile -File install.ps1 \
  -SourcePath "$payload/tray-goblin-0.1.0-windows-x86_64" \
  -InstallRoot "$evidence/installed" -NoStartup -NoLaunch
rm -rf "$payload"
node scripts/demo-attention-host.mjs \
  "$evidence/installed" \
  "$PWD/target/release/tray-goblin-attention-demo" \
  "$PWD/target/release/tray-goblin-probe" "$evidence" podman
printf 'Attention implementation demo passed (not Windows delivery). Evidence: %s\n' "$evidence"
