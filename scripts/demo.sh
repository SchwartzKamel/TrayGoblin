#!/usr/bin/env bash
# End-to-end demo: proves the deterministic contracts and assembles the
# release candidate in dist/.
#
# Five stages, in the order a release is judged:
#   1. Preflight        — required tools and the requested version
#   2. Host validation  — workspace tests plus the Windows PE cross-build
#   3. Fixture journeys — tier 1 (live session), then tier 2 (degraded session)
#   4. Packaging        — reproducible ZIP and SHA-256 file
#   5. Artifact checks  — archive entries, checksum, and the packaged executable
#
# Nothing here needs the network, a CI service, or Windows: the tray itself is
# only exercised by the manual checklist in docs/agent/testing.md.
set -euo pipefail

readonly TARGET="x86_64-pc-windows-gnu"
readonly PACKAGE_NAME="tray-goblin"
readonly EXECUTABLE_NAME="tray-goblin.exe"
readonly PLATFORM="windows-x86_64"
readonly WINDOWS_BINARY="target/${TARGET}/release/${EXECUTABLE_NAME}"
readonly DIST_DIR="dist"
readonly PROBE_BIN="tray-goblin-probe"
readonly LIVE_FIXTURE="tests/fixtures/live-session"
readonly DEGRADED_FIXTURE="tests/fixtures/degraded-session"

# The probe's entire JSON wire shape. A new key means the content-free
# allow-list changed and must be re-reviewed before a release.
readonly PROBE_FIELDS="active_directory attention_reason last_turn_duration_ms model repository state"

# Substrings that must never appear in a snapshot. `SENSITIVE_SENTINEL` is
# planted in the degraded fixture precisely so this demo can prove it.
readonly FORBIDDEN_SUBSTRINGS=(
  "SENSITIVE_SENTINEL"
  "prompt"
  "response"
  "toolArguments"
  "toolResult"
  "token"
  "credential"
  "futureField"
  "futureMetadata"
)

readonly EXPECTED_ARCHIVE_ENTRIES=(
  "LICENSE"
  "README.txt"
  "install.ps1"
  "tray-goblin.exe"
  "uninstall.ps1"
)

fail() {
  printf 'error: %s\n' "$1" >&2
  if [[ $# -gt 1 ]]; then
    printf '       %s\n' "$2" >&2
  fi
  exit 1
}

stage() {
  printf '\n== %s\n' "$1"
}

ok() {
  printf '   ok  %s\n' "$1"
}

require_command() {
  command -v "$1" >/dev/null 2>&1 || fail "'$1' was not found." "$2"
}

usage() {
  cat >&2 <<'USAGE'
usage: bash scripts/demo.sh <version>

  <version>  Release version, for example 0.1.0. It must match the version in
             Cargo.toml so the demo cannot certify an archive that does not
             match the build.
USAGE
}

if [[ $# -eq 1 ]]; then
  case "$1" in
    -h | --help)
      usage
      exit 0
      ;;
  esac
fi

if [[ $# -ne 1 ]]; then
  usage
  fail "exactly one version argument is required." "Run: bash scripts/demo.sh 0.1.0"
fi

version="$1"

repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repository_root}"

# ---------------------------------------------------------------------------
stage "1/5 Preflight"
# ---------------------------------------------------------------------------
require_command cargo "Install stable Rust from https://rustup.rs/."
require_command unzip "Install the 'unzip' utility, for example: apt-get install unzip"
require_command sha256sum "Install GNU coreutils so SHA-256 checksums can be verified."
require_command file "Install the 'file' utility so the Windows artifact can be verified."
ok "required tools are available"

for helper in scripts/validate-toolchain.sh scripts/package-release.sh; do
  [[ -f "${helper}" ]] ||
    fail "helper script '${helper}' is missing." "Restore it; this demo delegates to it instead of duplicating it."
done
ok "helper scripts are present"

if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  fail "'${version}' is not a semantic version." "Use a version such as 0.1.0."
fi

cargo_version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
[[ -n "${cargo_version}" ]] ||
  fail "the package version could not be read from Cargo.toml." "Confirm Cargo.toml has a 'version = \"x.y.z\"' line."

if [[ "${cargo_version}" != "${version}" ]]; then
  fail "requested version '${version}' does not match Cargo.toml version '${cargo_version}'." \
    "Run: bash scripts/demo.sh ${cargo_version}"
fi
ok "version ${version} matches Cargo.toml"

for fixture_root in "${LIVE_FIXTURE}" "${DEGRADED_FIXTURE}"; do
  [[ -d "${fixture_root}" ]] ||
    fail "fixture session root '${fixture_root}' is missing." "Restore the content-free fixtures committed with the monitor tests."
done

# A leak check over a fixture without a sentinel would silently prove nothing.
grep -Rqs "SENSITIVE_SENTINEL" "${DEGRADED_FIXTURE}" ||
  fail "the degraded fixture no longer contains a SENSITIVE_SENTINEL value." \
    "Restore the sentinel; without it the content-free assertion is vacuous."
ok "fixtures present, degraded fixture still carries the sentinel"

# ---------------------------------------------------------------------------
stage "2/5 Host validation and Windows cross-build"
# ---------------------------------------------------------------------------
bash scripts/validate-toolchain.sh ||
  fail "host validation failed." "Fix the reported test or cross-build failure, then re-run: bash scripts/demo.sh ${version}"

[[ -f "${WINDOWS_BINARY}" ]] ||
  fail "'${WINDOWS_BINARY}' was not produced." "Run: bash scripts/validate-toolchain.sh"
ok "workspace tests passed and ${WINDOWS_BINARY} is a Windows x86-64 PE executable"

# ---------------------------------------------------------------------------
stage "3/5 Content-free fixture journeys"
# ---------------------------------------------------------------------------
probe_snapshot=""

run_probe() {
  local fixture_root="$1"

  if ! probe_snapshot="$(cargo run --quiet --bin "${PROBE_BIN}" -- --session-root "${fixture_root}")"; then
    fail "the probe failed for '${fixture_root}'." \
      "Reproduce with: cargo run --bin ${PROBE_BIN} -- --session-root ${fixture_root}"
  fi
}

# The probe emits one pretty-printed field per line, so the snapshot can be
# read without adding a JSON tool to the demo's dependencies.
snapshot_keys() {
  printf '%s\n' "${probe_snapshot}" |
    sed -n 's/^[[:space:]]*"\([^"]*\)":.*/\1/p' | LC_ALL=C sort | tr '\n' ' ' | sed 's/ $//'
}

snapshot_value() {
  local field="$1" raw

  raw="$(printf '%s\n' "${probe_snapshot}" | sed -n "s/^[[:space:]]*\"${field}\": \\(.*\\)/\\1/p")"
  raw="${raw%,}"
  raw="${raw#\"}"
  raw="${raw%\"}"
  printf '%s' "${raw}"
}

assert_field() {
  local field="$1" expected="$2" actual

  actual="$(snapshot_value "${field}")"
  if [[ "${actual}" != "${expected}" ]]; then
    fail "snapshot field '${field}' was '${actual}', expected '${expected}'." \
      "Compare the journey expectations in docs/agent/testing.md with the current monitor behaviour."
  fi
}

assert_allow_listed_fields() {
  local actual_fields
  actual_fields="$(snapshot_keys)"

  if [[ "${actual_fields}" != "${PROBE_FIELDS}" ]]; then
    fail "the probe emitted fields '${actual_fields}' instead of the allow-list '${PROBE_FIELDS}'." \
      "Any new field must be reviewed as content-free before it is released; see src/bin/${PROBE_BIN}.rs."
  fi
}

assert_content_free() {
  local fixture_root="$1" forbidden

  for forbidden in "${FORBIDDEN_SUBSTRINGS[@]}"; do
    if printf '%s' "${probe_snapshot}" | grep -qiF -- "${forbidden}"; then
      fail "the snapshot for '${fixture_root}' contains '${forbidden}'." \
        "TrayGoblin must never surface event content; see events::tests::does_not_model_sensitive_fields."
    fi
  done
}

# Tier 1 — active-turn magic moment: a live session in mid-turn.
run_probe "${LIVE_FIXTURE}"
assert_allow_listed_fields
assert_content_free "${LIVE_FIXTURE}"
assert_field state "working"
assert_field model "gpt-5.6-sol"
assert_field repository "octo-org/content-free-demo"
assert_field active_directory "C:/fixture/content-free-demo"
assert_field last_turn_duration_ms "null"
assert_field attention_reason "null"
ok "tier 1 ${LIVE_FIXTURE}: working, newest active session, no leaked content"

# Tier 2 — degraded session: unknown event types and unknown fields must
# still yield an actionable, content-free Attention needed snapshot.
run_probe "${DEGRADED_FIXTURE}"
assert_allow_listed_fields
assert_content_free "${DEGRADED_FIXTURE}"
assert_field state "attention_needed"
assert_field attention_reason "tool_failed"
assert_field model "future-model-x"
assert_field repository "octo-org/nested-demo"
assert_field active_directory "C:/fixture/nested-demo"
assert_field last_turn_duration_ms "3000"
ok "tier 2 ${DEGRADED_FIXTURE}: attention_needed via tool_failed, future formats tolerated"

# ---------------------------------------------------------------------------
stage "4/5 Deterministic release packaging"
# ---------------------------------------------------------------------------
bash scripts/package-release.sh "${version}" ||
  fail "release packaging failed for version ${version}." \
    "Fix the reported packaging error, then re-run: bash scripts/demo.sh ${version}"

# ---------------------------------------------------------------------------
stage "5/5 Release candidate verification"
# ---------------------------------------------------------------------------
archive_stem="${PACKAGE_NAME}-${version}-${PLATFORM}"
archive_path="${DIST_DIR}/${archive_stem}.zip"
checksum_path="${archive_path}.sha256"

[[ -f "${archive_path}" ]] ||
  fail "packaging did not produce '${archive_path}'." "Run: bash scripts/package-release.sh ${version}"
[[ -f "${checksum_path}" ]] ||
  fail "packaging did not produce '${checksum_path}'." "Run: bash scripts/package-release.sh ${version}"

unzip -tqq "${archive_path}" >/dev/null ||
  fail "'${archive_path}' failed its integrity test." "Delete '${DIST_DIR}' and run: bash scripts/demo.sh ${version}"

expected_listing="$(
  printf '%s/\n' "${archive_stem}"
  for expected_entry in "${EXPECTED_ARCHIVE_ENTRIES[@]}"; do
    printf '%s/%s\n' "${archive_stem}" "${expected_entry}"
  done
)"
actual_listing="$(unzip -Z1 "${archive_path}" | LC_ALL=C sort)"
expected_listing="$(printf '%s\n' "${expected_listing}" | LC_ALL=C sort)"

if [[ "${actual_listing}" != "${expected_listing}" ]]; then
  fail "'${archive_path}' does not contain exactly the expected entries." \
    "Expected: $(printf '%s' "${expected_listing}" | tr '\n' ' '); got: $(printf '%s' "${actual_listing}" | tr '\n' ' ')"
fi
ok "archive contains exactly ${#EXPECTED_ARCHIVE_ENTRIES[@]} payload entries"

packaged_description="$(unzip -p "${archive_path}" "${archive_stem}/${EXECUTABLE_NAME}" | file -b -)"
if [[ "${packaged_description}" != *PE32+* || "${packaged_description}" != *x86-64* ]]; then
  fail "the packaged executable is not an x86-64 PE executable: ${packaged_description}" \
    "Confirm the MinGW toolchain and the ${TARGET} Rust target are installed, then package again."
fi
ok "packaged ${EXECUTABLE_NAME} is ${packaged_description}"

built_hash="$(sha256sum "${WINDOWS_BINARY}" | cut -d ' ' -f 1)"
packaged_hash="$(unzip -p "${archive_path}" "${archive_stem}/${EXECUTABLE_NAME}" | sha256sum | cut -d ' ' -f 1)"
if [[ "${built_hash}" != "${packaged_hash}" ]]; then
  fail "the packaged executable does not match the validated build." \
    "Delete '${DIST_DIR}' and run: bash scripts/demo.sh ${version}"
fi
ok "packaged executable is byte-identical to the validated build"

(
  cd "${DIST_DIR}"
  sha256sum --check --status "${archive_stem}.zip.sha256"
) || fail "'${checksum_path}' does not match '${archive_path}'." \
  "Delete '${DIST_DIR}' and run: bash scripts/demo.sh ${version}"

recorded_hash="$(cut -d ' ' -f 1 "${checksum_path}")"
ok "SHA-256 file verified"

printf '\nDemo passed for %s.\n' "${version}"
printf 'Release candidate:\n'
printf '  %s\n' "${archive_path}"
printf '  %s\n' "${checksum_path}"
printf 'SHA-256 %s\n' "${recorded_hash}"
printf 'Windows interaction and the resource budgets are still manual: see docs/agent/testing.md.\n'
