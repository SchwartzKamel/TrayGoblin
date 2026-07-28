#!/usr/bin/env bash
set -euo pipefail

readonly TARGET="x86_64-pc-windows-gnu"
readonly LINKER="x86_64-w64-mingw32-gcc"
readonly WINDOWS_BINARY="target/${TARGET}/release/tray-goblin.exe"

fail() {
  printf 'error: %s\n' "$*" >&2
  exit 1
}

require_command() {
  local command_name="$1"
  local installation_hint="$2"

  command -v "${command_name}" >/dev/null 2>&1 ||
    fail "'${command_name}' was not found. ${installation_hint}"
}

require_command cargo "Install stable Rust from https://rustup.rs/."
require_command rustc "Install stable Rust from https://rustup.rs/."
require_command "${LINKER}" "Install the MinGW-w64 x86-64 GCC toolchain and add it to PATH."
require_command file "Install the 'file' utility so the Windows artifact can be verified."

rust_version="$(rustc --version)"
case "${rust_version}" in
  *nightly* | *beta* | *dev*)
    fail "stable Rust is required, but '${rust_version}' is active. Run: rustup override set stable"
    ;;
esac

target_libdir="$(rustc --print target-libdir --target "${TARGET}" 2>/dev/null || true)"
if [[ -z "${target_libdir}" || ! -d "${target_libdir}" ]]; then
  fail "Rust target '${TARGET}' is not installed. Run: rustup target add ${TARGET}"
fi

cargo test --workspace
cargo build --release --target "${TARGET}" --bin tray-goblin

[[ -f "${WINDOWS_BINARY}" ]] ||
  fail "cross-build completed without producing '${WINDOWS_BINARY}'."

binary_description="$(file -b "${WINDOWS_BINARY}")"
if [[ "${binary_description}" != *PE32+* || "${binary_description}" != *x86-64* ]]; then
  fail "'${WINDOWS_BINARY}' is not an x86-64 PE executable: ${binary_description}"
fi

printf 'Toolchain validation passed: %s\n' "${binary_description}"
