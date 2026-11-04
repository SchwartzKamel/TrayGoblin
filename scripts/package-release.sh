#!/usr/bin/env bash
# Builds and packages the manual Windows x86-64 portable release.
#
# The archive is byte-for-byte reproducible for a given executable: file
# permissions and timestamps are normalized before zipping, and the resulting
# archive is packed twice and compared before it is published.
set -euo pipefail

readonly TARGET="x86_64-pc-windows-gnu"
readonly PACKAGE_NAME="tray-goblin"
readonly EXECUTABLE_NAME="tray-goblin.exe"
readonly PLATFORM="windows-x86_64"
readonly WINDOWS_BINARY="target/${TARGET}/release/${EXECUTABLE_NAME}"
readonly DIST_DIR="dist"
# Fixed archive timestamp keeps the ZIP reproducible across machines and days.
readonly DEFAULT_SOURCE_DATE_EPOCH=1451606400

fail() {
  printf 'error: %s\n' "$1" >&2
  if [[ $# -gt 1 ]]; then
    printf '       %s\n' "$2" >&2
  fi
  exit 1
}

require_command() {
  local command_name="$1"
  local installation_hint="$2"

  command -v "${command_name}" >/dev/null 2>&1 ||
    fail "'${command_name}' was not found." "${installation_hint}"
}

usage() {
  cat >&2 <<'USAGE'
usage: scripts/package-release.sh <version>

  <version>  Release version, for example 0.1.0. It must match the version in
             Cargo.toml so the archive name cannot drift from the build.
USAGE
}

if [[ $# -ne 1 ]]; then
  usage
  fail "exactly one version argument is required." "Run: bash scripts/package-release.sh 0.1.0"
fi

version="$1"

case "${version}" in
  -h | --help)
    usage
    exit 0
    ;;
esac

if [[ ! "${version}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?$ ]]; then
  fail "'${version}' is not a semantic version." "Use a version such as 0.1.0."
fi

repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repository_root}"

require_command cargo "Install stable Rust from https://rustup.rs/."
require_command zip "Install the 'zip' utility, for example: apt-get install zip"
require_command unzip "Install the 'unzip' utility, for example: apt-get install unzip"
require_command sha256sum "Install GNU coreutils so SHA-256 checksums can be generated."
require_command file "Install the 'file' utility so the Windows artifact can be verified."
require_command git "Install Git so the build commit can be recorded without guessing."

cargo_version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
[[ -n "${cargo_version}" ]] ||
  fail "the package version could not be read from Cargo.toml." "Confirm Cargo.toml has a 'version = \"x.y.z\"' line."

if [[ "${cargo_version}" != "${version}" ]]; then
  fail "requested version '${version}' does not match Cargo.toml version '${cargo_version}'." \
    "Update Cargo.toml, or package '${cargo_version}' instead."
fi

for required_file in install.ps1 uninstall.ps1 LICENSE integrations/pi/tray-goblin.js integrations/pi/package.json integrations/pi/README.md assets/attention/SETUP.md assets/attention/config.example.json scripts/test-attention-windows.ps1 scripts/measure-performance.ps1; do
  [[ -f "${required_file}" ]] ||
    fail "release payload file '${required_file}' is missing." "Restore it before packaging a release."
done

printf 'Building %s for %s...\n' "${EXECUTABLE_NAME}" "${TARGET}"
cargo build --release --target "${TARGET}" --bin "${PACKAGE_NAME}"

[[ -f "${WINDOWS_BINARY}" ]] ||
  fail "the build finished without producing '${WINDOWS_BINARY}'." "Run: bash scripts/validate-toolchain.sh"

binary_description="$(file -b "${WINDOWS_BINARY}")"
if [[ "${binary_description}" != *PE32+* || "${binary_description}" != *x86-64* ]]; then
  fail "'${WINDOWS_BINARY}' is not an x86-64 PE executable: ${binary_description}" \
    "Confirm the MinGW toolchain and the ${TARGET} Rust target are installed."
fi

archive_stem="${PACKAGE_NAME}-${version}-${PLATFORM}"
staging_root="${DIST_DIR}/staging"
payload_dir="${staging_root}/${archive_stem}"
archive_path="${DIST_DIR}/${archive_stem}.zip"
checksum_path="${archive_path}.sha256"
verify_archive_path="${staging_root}/${archive_stem}.verify.zip"

rm -rf "${staging_root}"
rm -f "${archive_path}" "${checksum_path}"
mkdir -p "${payload_dir}"

cp "${WINDOWS_BINARY}" "${payload_dir}/${EXECUTABLE_NAME}"
cp install.ps1 "${payload_dir}/install.ps1"
cp uninstall.ps1 "${payload_dir}/uninstall.ps1"
cp LICENSE "${payload_dir}/LICENSE"
mkdir -p "${payload_dir}/integrations/pi" "${payload_dir}/resources"
cp integrations/pi/{tray-goblin.js,package.json,README.md} "${payload_dir}/integrations/pi/"
cp assets/attention/{SETUP.md,config.example.json} "${payload_dir}/resources/"
cp scripts/{test-attention-windows.ps1,measure-performance.ps1} "${payload_dir}/resources/"
# Bind collectors to the actual packaged PE; dirty author builds are explicitly
# NOT misrepresented as clean tagged release candidates.
build_commit="$(git rev-parse --verify HEAD)"
worktree_dirty=false
# Metadata/pathspec-scoped inspection avoids opening either holdout directory.
# Excluded holdouts are outside author evidence; parent freezes the full tree.
git diff --quiet HEAD -- . ':!scenarios' ':!.wgm/scenarios' || worktree_dirty=true
[[ -z "$(git ls-files --others --exclude-standard -- . ':!scenarios' ':!.wgm/scenarios')" ]] || worktree_dirty=true
executable_hash="$(sha256sum "${WINDOWS_BINARY}" | cut -d ' ' -f 1)"
printf '{"buildCommit":"%s","worktreeDirty":%s,"executableSha256":"%s","published":false}\n' \
  "${build_commit}" "${worktree_dirty}" "${executable_hash}" >"${payload_dir}/resources/build.json"
# The spec's checkout-origin field command names this exact built PE. Preserve
# the SAME digest-bound identity beside it as well as inside the portable ZIP.
mkdir -p "$(dirname "${WINDOWS_BINARY}")/resources"
cp "${payload_dir}/resources/build.json" "$(dirname "${WINDOWS_BINARY}")/resources/build.json"

cat >"${payload_dir}/README.txt" <<EOF
TrayGoblin ${version} (Windows x86-64, portable)

Install (no administrator rights required):
  1. Extract this folder anywhere, for example to your Downloads folder.
  2. Open PowerShell and run:
       powershell -NoProfile -ExecutionPolicy Bypass -File .\\install.ps1
     TrayGoblin is copied below %LOCALAPPDATA%\\Programs\\TrayGoblin and a
     shortcut is added to your per-user Startup folder.

Uninstall:
       powershell -NoProfile -ExecutionPolicy Bypass -File .\\uninstall.ps1
  Your configuration in %APPDATA%\\TrayGoblin is kept. Add
  -RemoveConfiguration to delete it as well.

Verify this download:
  Compare the SHA-256 of ${archive_stem}.zip with ${archive_stem}.zip.sha256:
       Get-FileHash .\\${archive_stem}.zip -Algorithm SHA256

Attention setup and persistent opt-in pi integration:
  Read resources/SETUP.md. The installer copies integrations/pi and resources
  below the install root; pi settings are NEVER edited automatically.
  Use the INSTALLED path for pi install, not this extracted folder.
  Remove the pi declaration and reload pi BEFORE uninstalling TrayGoblin.

Status:
  UNPUBLISHED implementation artifact; no Windows/WSL delivery or resource
  evidence is implied. resources/build.json identifies the actual executable.

Privacy:
  Local state is read-only. Only allow-listed metadata is modeled; temporary
  raw-byte scanning is not prompt/result/argument/token/credential modeling.
  No conversation content is stored, logged or displayed.
EOF

source_date_epoch="${SOURCE_DATE_EPOCH:-${DEFAULT_SOURCE_DATE_EPOCH}}"
normalized_timestamp="$(date -u -d "@${source_date_epoch}" +'%Y%m%d%H%M.%S' 2>/dev/null || true)"
[[ -n "${normalized_timestamp}" ]] ||
  fail "SOURCE_DATE_EPOCH='${source_date_epoch}' could not be converted to a timestamp." \
    "Unset SOURCE_DATE_EPOCH or set it to a Unix timestamp after 1980-01-01."

# Normalize the payload so the archive depends only on file contents.
find "${payload_dir}" -type f -name '*.exe' -exec chmod 0755 {} +
find "${payload_dir}" -type f ! -name '*.exe' -exec chmod 0644 {} +
find "${payload_dir}" -type d -exec chmod 0755 {} +
find "${payload_dir}" -exec touch -h -t "${normalized_timestamp}" {} +

pack_archive() {
  local output_path="$1"
  local absolute_output
  absolute_output="$(cd "$(dirname "${output_path}")" && pwd)/$(basename "${output_path}")"

  rm -f "${absolute_output}"
  (
    cd "${staging_root}"
    # -X drops uid/gid and extra timestamp fields; the sorted file list and
    # the normalized mtimes make the remaining bytes deterministic.
    find "${archive_stem}" -print | LC_ALL=C sort |
      zip -q -X -9 "${absolute_output}" -@
  ) || fail "the archive '${output_path}' could not be created." "Confirm '${DIST_DIR}' is writable."
}

pack_archive "${archive_path}"
pack_archive "${verify_archive_path}"

archive_hash="$(sha256sum "${archive_path}" | cut -d ' ' -f 1)"
verify_hash="$(sha256sum "${verify_archive_path}" | cut -d ' ' -f 1)"

if [[ "${archive_hash}" != "${verify_hash}" ]]; then
  fail "packaging is not reproducible: two runs produced different archives." \
    "Report this; the release must not be published from a non-deterministic package."
fi

rm -f "${verify_archive_path}"

(
  cd "${DIST_DIR}"
  sha256sum "${archive_stem}.zip" >"${archive_stem}.zip.sha256"
  sha256sum --check --status "${archive_stem}.zip.sha256"
) || fail "the SHA-256 checksum could not be generated or verified." "Confirm '${DIST_DIR}' is writable."

unzip -tqq "${archive_path}" >/dev/null ||
  fail "the packaged archive failed its integrity test." "Delete '${DIST_DIR}' and package again."

# Exact file AND directory allowlist; adding accidental fixtures/user data fails.
expected_entries=(
  "" LICENSE README.txt install.ps1 "${EXECUTABLE_NAME}" uninstall.ps1
  integrations/ integrations/pi/ integrations/pi/README.md
  integrations/pi/package.json integrations/pi/tray-goblin.js
  resources/ resources/SETUP.md resources/config.example.json resources/build.json
  resources/test-attention-windows.ps1 resources/measure-performance.ps1
)
expected_listing="$(for entry in "${expected_entries[@]}"; do printf '%s/%s\n' "${archive_stem}" "${entry}"; done | LC_ALL=C sort)"
actual_listing="$(unzip -Z1 "${archive_path}" | LC_ALL=C sort)"
[[ "${actual_listing}" == "${expected_listing}" ]] ||
  fail "the archive does not match the exact payload allowlist." "Inspect the package before proceeding."
packaged_executable_hash="$(unzip -p "${archive_path}" "${archive_stem}/${EXECUTABLE_NAME}" | sha256sum | cut -d ' ' -f 1)"
[[ "${packaged_executable_hash}" == "${executable_hash}" ]] || fail "packaged PE differs from the built PE."

rm -rf "${staging_root}"

printf 'Packaged %s\n' "${archive_path}"
printf 'Checksum %s\n' "${checksum_path}"
printf 'SHA-256  %s\n' "${archive_hash}"
printf 'Reproducible: two independent packing runs matched.\n'
