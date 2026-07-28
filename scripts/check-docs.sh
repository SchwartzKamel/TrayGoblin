#!/usr/bin/env bash
# Deterministic documentation contract check.
#
# Validates the operator- and agent-facing documentation set without touching
# the network: required files, audience labels, internal relative links and
# anchors, index completeness, required commands and gate statements, script
# and version references, stale implementation-status claims, and the syntax
# of the shell scripts this repository ships.
#
# Usage: bash scripts/check-docs.sh
set -uo pipefail

repository_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${repository_root}" || exit 1

readonly OPERATOR_LABEL="Operators"
readonly AGENT_LABEL="Agents"
readonly SHARED_LABEL="Operators and Agents"

# Scripts a document may reference before the task that creates them has run.
# Each entry is owned by a task in IMPLEMENTATION_PLAN.md. Empty: every
# documented script now exists.
readonly PLANNED_SCRIPTS=()

readonly REQUIRED_FILES=(
  "README.md"
  "docs/README.md"
  "docs/MVP.md"
  "docs/architecture.md"
  "docs/manual-release.md"
  "docs/operator/installation.md"
  "docs/operator/first-run.md"
  "docs/operator/status-reference.md"
  "docs/operator/configuration.md"
  "docs/operator/privacy.md"
  "docs/operator/troubleshooting.md"
  "docs/agent/development.md"
  "docs/agent/testing.md"
  "docs/agent/release-responsibilities.md"
  "docs/agent/documentation-standards.md"
  "scripts/check-docs.sh"
)

failures=0
checks=0
skips=0
current_section=""

section() {
  current_section="$1"
  printf '\n== %s\n' "$1"
}

pass() {
  checks=$((checks + 1))
}

report() {
  checks=$((checks + 1))
  failures=$((failures + 1))
  printf '   FAIL [%s] %s\n' "${current_section}" "$1" >&2
}

skip() {
  skips=$((skips + 1))
  printf '   skip %s\n' "$1"
}

info() {
  printf '   %s\n' "$1"
}

# Resolves "." and ".." segments without touching the filesystem, so link
# targets are compared as canonical repository-relative paths.
normalize_path() {
  local path="$1"
  local -a parts=()
  local -a segments=()
  local part

  IFS='/' read -r -a parts <<<"${path}"
  for part in ${parts[@]+"${parts[@]}"}; do
    case "${part}" in
      '' | '.')
        continue
        ;;
      '..')
        if [[ ${#segments[@]} -gt 0 ]]; then
          unset 'segments[-1]'
        fi
        ;;
      *)
        segments+=("${part}")
        ;;
    esac
  done

  (
    IFS='/'
    printf '%s\n' "${segments[*]-}"
  )
}

# GitHub heading-anchor slug: lowercase, drop punctuation other than hyphen
# and underscore, and replace runs of whitespace with a single hyphen.
slugify() {
  printf '%s' "${1,,}" |
    sed -E 's/`//g; s/[^a-z0-9 _-]//g; s/[[:space:]]+/-/g; s/^-+//; s/-+$//'
}

# Markdown outside fenced code blocks. Examples inside fences describe other
# systems and must not be validated as if they were repository links.
prose_of() {
  awk '/^[[:space:]]*```/ { fenced = !fenced; next } !fenced' "$1"
}

links_of() {
  prose_of "$1" |
    grep -oE '\]\([^()[:space:]]+\)' |
    sed -E 's/^\]\(//; s/\)$//' || true
}

headings_of() {
  prose_of "$1" | grep -E '^#{1,6} ' | sed -E 's/^#{1,6} +//' || true
}

# A required phrase is satisfied by a matching line, or by a match in the
# file's reflowed text, so that hard-wrapped sentences still count.
require_phrase() {
  local file="$1" pattern="$2" description="$3"

  if [[ ! -f "${file}" ]]; then
    report "${file} is missing, so '${description}' cannot be verified"
    return
  fi

  if grep -qE "${pattern}" "${file}" ||
    tr '\n' ' ' <"${file}" | tr -s ' ' | grep -qE "${pattern}"; then
    pass
  else
    report "${file} must document ${description} (expected pattern: ${pattern})"
  fi
}

forbid_pattern() {
  local pattern="$1" description="$2" file
  local -a offenders=()

  for file in "${DOC_FILES[@]}"; do
    if grep -qiE "${pattern}" "${file}"; then
      offenders+=("${file}")
    fi
  done

  if [[ ${#offenders[@]} -eq 0 ]]; then
    pass
  else
    report "${description}: ${offenders[*]}"
  fi
}

# ---------------------------------------------------------------------------
section "Required files"
# ---------------------------------------------------------------------------
for required in "${REQUIRED_FILES[@]}"; do
  if [[ -f "${required}" ]]; then
    pass
  else
    report "required file '${required}' is missing"
  fi
done
info "checked ${#REQUIRED_FILES[@]} required files"

mapfile -t DOC_FILES < <(
  {
    printf '%s\n' "README.md"
    find docs -type f -name '*.md' -not -path 'docs/audit/*'
  } | LC_ALL=C sort -u
)

if [[ ${#DOC_FILES[@]} -eq 0 ]]; then
  printf 'error: no documentation files were found\n' >&2
  exit 1
fi
info "documentation set: ${#DOC_FILES[@]} files"

# ---------------------------------------------------------------------------
section "Titles and audience labels"
# ---------------------------------------------------------------------------
for doc in "${DOC_FILES[@]}"; do
  if head -n 1 "${doc}" | grep -qE '^# .+'; then
    pass
  else
    report "${doc} must start with a level-1 title"
  fi

  label_count="$(prose_of "${doc}" | grep -cE '^\*\*Audience:\*\* ' || true)"
  if [[ "${label_count}" -ne 1 ]]; then
    report "${doc} must contain exactly one '**Audience:**' line (found ${label_count})"
    continue
  fi
  pass

  audience="$(prose_of "${doc}" | grep -m1 -E '^\*\*Audience:\*\* ' | sed -E 's/^\*\*Audience:\*\* +//')"
  case "${audience}" in
    "${OPERATOR_LABEL}" | "${AGENT_LABEL}" | "${SHARED_LABEL}")
      pass
      ;;
    *)
      report "${doc} declares an unsupported audience '${audience}'; use '${OPERATOR_LABEL}', '${AGENT_LABEL}', or '${SHARED_LABEL}'"
      continue
      ;;
  esac

  case "${doc}" in
    docs/operator/*)
      if [[ "${audience}" == "${OPERATOR_LABEL}" ]]; then
        pass
      else
        report "${doc} is an operator guide, so its audience must be '${OPERATOR_LABEL}'"
      fi
      ;;
    docs/agent/*)
      if [[ "${audience}" == "${AGENT_LABEL}" ]]; then
        pass
      else
        report "${doc} is an agent guide, so its audience must be '${AGENT_LABEL}'"
      fi
      ;;
  esac
done
info "every page declares one supported audience"

# ---------------------------------------------------------------------------
section "Forbidden wording"
# ---------------------------------------------------------------------------
forbid_pattern '\bhumans\b|audience[^.]{0,24}\bhuman\b|\*\*human' \
  "audiences are named Operators and Agents; no other audience label is used"
forbid_pattern 'request latency' \
  "non-canonical term 'request latency'; use 'last turn duration' (see specs/CONTEXT.md)"
forbid_pattern '/home/[a-z0-9_.-]+/' \
  "a contributor's absolute home path leaked into documentation"
forbid_pattern '\b(TODO|FIXME|TBD)\b' \
  "unfinished-documentation markers"
forbid_pattern 'not yet implemented|coming soon|under construction|work in progress|implementation is starting|no code exists' \
  "stale implementation-status wording"
forbid_pattern 'monitor::tests::turn_start_sets_generating|turn_end_sets_latency' \
  "stale monitor test identifiers that match zero tests"
forbid_pattern 'GitHub Actions (builds|publishes|validates|runs)|CI (builds|publishes|validates) (the|this)' \
  "a CI service described as building or publishing this project"

# ---------------------------------------------------------------------------
section "Internal links"
# ---------------------------------------------------------------------------
link_count=0
for doc in "${DOC_FILES[@]}"; do
  doc_dir="$(dirname "${doc}")"
  internal_links=0

  while IFS= read -r target; do
    [[ -n "${target}" ]] || continue

    case "${target}" in
      http://* | https://* | mailto:*)
        continue
        ;;
    esac

    link_count=$((link_count + 1))
    internal_links=$((internal_links + 1))

    anchor=""
    path="${target}"
    if [[ "${target}" == *"#"* ]]; then
      path="${target%%#*}"
      anchor="${target#*#}"
    fi

    if [[ -z "${path}" ]]; then
      resolved="${doc}"
    else
      resolved="$(normalize_path "${doc_dir}/${path}")"
    fi

    if [[ ! -e "${resolved}" ]]; then
      report "${doc} links to '${target}', which does not exist"
      continue
    fi
    pass

    [[ -n "${anchor}" && "${resolved}" == *.md ]] || continue

    anchor_found=0
    while IFS= read -r heading; do
      [[ -n "${heading}" ]] || continue
      if [[ "$(slugify "${heading}")" == "${anchor}" ]]; then
        anchor_found=1
        break
      fi
    done < <(headings_of "${resolved}")

    if [[ "${anchor_found}" -eq 1 ]]; then
      pass
    else
      report "${doc} links to '#${anchor}' in '${resolved}', which has no such heading"
    fi
  done < <(links_of "${doc}")

  if [[ "${internal_links}" -gt 0 ]]; then
    pass
  else
    report "${doc} has no link to another repository document"
  fi
done
info "resolved ${link_count} internal links and anchors"

# ---------------------------------------------------------------------------
section "Index completeness"
# ---------------------------------------------------------------------------
mapfile -t INDEXED < <(
  while IFS= read -r target; do
    [[ -n "${target}" ]] || continue
    case "${target}" in
      http://* | https://* | mailto:* | '#'*)
        continue
        ;;
    esac
    normalize_path "docs/${target%%#*}"
  done < <(links_of "docs/README.md") | LC_ALL=C sort -u
)

for doc in "${DOC_FILES[@]}"; do
  case "${doc}" in
    README.md | docs/README.md)
      continue
      ;;
  esac

  if printf '%s\n' ${INDEXED[@]+"${INDEXED[@]}"} | grep -Fxq "${doc}"; then
    pass
  else
    report "docs/README.md does not link '${doc}'"
  fi
done
info "docs/README.md indexes every page under docs/"

# ---------------------------------------------------------------------------
section "Required commands and gate statements"
# ---------------------------------------------------------------------------
readonly PROMOTION_GATE='must pass before promoting a preview build to stable|must not be promoted to stable until this measurement passes|must not be promoted from preview to stable|must pass before promoting a preview'

require_phrase "README.md" 'bash scripts/check-docs\.sh' "the documentation validation command"
require_phrase "README.md" 'cargo test --workspace' "the deterministic test command"
require_phrase "README.md" 'docs/README\.md' "a link to the documentation index"
require_phrase "README.md" 'Attention needed' "the Attention needed state"
require_phrase "README.md" "${PROMOTION_GATE}" "the Windows performance promotion gate"

require_phrase "docs/README.md" "\\*\\*${OPERATOR_LABEL}\\*\\*" "the Operators audience"
require_phrase "docs/README.md" "\\*\\*${AGENT_LABEL}\\*\\*" "the Agents audience"
require_phrase "docs/README.md" 'bash scripts/check-docs\.sh' "the documentation validation command"

require_phrase "docs/MVP.md" 'Attention needed' "the Attention needed state"
require_phrase "docs/MVP.md" "${PROMOTION_GATE}" "the Windows performance promotion gate"

require_phrase "docs/architecture.md" 'content-free' "the content-free boundary"
require_phrase "docs/architecture.md" 'inuse\.\*\.lock' "active-session selection"
require_phrase "docs/architecture.md" "${PROMOTION_GATE}" "the Windows performance promotion gate"

require_phrase "docs/manual-release.md" 'git status --porcelain' "the clean-tree proof"
require_phrase "docs/manual-release.md" 'bash scripts/package-release\.sh 0\.1\.0' "the packaging command"
require_phrase "docs/manual-release.md" 'sha256sum' "checksum generation"
require_phrase "docs/manual-release.md" 'Get-FileHash' "operator-side checksum verification"
require_phrase "docs/manual-release.md" 'git tag -a' "annotated tagging"
require_phrase "docs/manual-release.md" 'gh release create' "manual release publication"
require_phrase "docs/manual-release.md" 'cat > release-notes\.md' "release-notes creation before publication"
require_phrase "docs/manual-release.md" 'measure-performance\.ps1' "the Windows performance measurement"
require_phrase "docs/manual-release.md" "${PROMOTION_GATE}" "the Windows performance promotion gate"
require_phrase "docs/manual-release.md" '^## Abort conditions' "abort conditions"
require_phrase "docs/manual-release.md" '^## Rollback' "rollback procedure"
require_phrase "docs/manual-release.md" 'git push origin :refs/tags/' "remote tag rollback"

require_phrase "docs/operator/installation.md" 'Get-FileHash' "checksum verification"
require_phrase "docs/operator/installation.md" 'install\.ps1' "the installer"
require_phrase "docs/operator/installation.md" 'uninstall\.ps1' "the uninstaller"
require_phrase "docs/operator/installation.md" '-RemoveConfiguration' "configuration removal opt-in"
require_phrase "docs/operator/installation.md" 'LOCALAPPDATA' "the per-user installation location"

require_phrase "docs/operator/first-run.md" 'Refresh now' "the Refresh now command"
require_phrase "docs/operator/first-run.md" 'Open in VS Code' "the Open in VS Code command"
require_phrase "docs/operator/first-run.md" 'View Copilot logs' "the View Copilot logs command"
require_phrase "docs/operator/first-run.md" 'Open settings' "the Open settings command"
require_phrase "docs/operator/first-run.md" 'Quit' "the Quit command"

require_phrase "docs/operator/status-reference.md" '\*\*Idle\*\*' "the Idle state"
require_phrase "docs/operator/status-reference.md" '\*\*Working\*\*' "the Working state"
require_phrase "docs/operator/status-reference.md" '\*\*Attention needed\*\*' "the Attention needed state"
require_phrase "docs/operator/status-reference.md" 'tray-goblin-probe' "the diagnostic probe"

require_phrase "docs/operator/configuration.md" 'pollIntervalMs' "the polling interval key"
require_phrase "docs/operator/configuration.md" 'sessionRoot' "the session-root key"
require_phrase "docs/operator/configuration.md" '500' "the minimum accepted interval"
require_phrase "docs/operator/configuration.md" '10,000|10000' "the maximum accepted interval"
require_phrase "docs/operator/configuration.md" 'APPDATA' "the configuration file location"

require_phrase "docs/operator/privacy.md" 'read-only' "read-only access"
require_phrase "docs/operator/privacy.md" 'tokens' "the token exclusion"
require_phrase "docs/operator/privacy.md" 'does_not_model_sensitive_fields' "the enforcing test"

require_phrase "docs/operator/troubleshooting.md" 'tray-goblin-probe' "the diagnostic probe workflow"
require_phrase "docs/operator/troubleshooting.md" 'assistant\.turn_start' "the recognized event vocabulary"

require_phrase "docs/agent/development.md" 'bash scripts/validate-toolchain\.sh' "the toolchain validation command"
require_phrase "docs/agent/development.md" 'cargo test --workspace' "the deterministic test command"
require_phrase "docs/agent/development.md" 'x86_64-pc-windows-gnu' "the Windows cross-build target"

require_phrase "docs/agent/testing.md" 'cargo test --workspace' "the deterministic test command"
require_phrase "docs/agent/testing.md" 'cargo test --test monitor' "the monitor integration command"
require_phrase "docs/agent/testing.md" 'turn_end_sets_idle_and_duration' "the real completed-turn test identifier"
require_phrase "docs/agent/testing.md" 'measure-performance\.ps1' "the Windows performance measurement"
require_phrase "docs/agent/testing.md" "${PROMOTION_GATE}" "the Windows performance promotion gate"
require_phrase "docs/agent/testing.md" 'test-installer\.ps1' "the installer verification command"

require_phrase "docs/agent/release-responsibilities.md" "${PROMOTION_GATE}" "the Windows performance promotion gate"
require_phrase "docs/agent/release-responsibilities.md" '\| Operator \|' "the operator-owned duties"
require_phrase "docs/agent/release-responsibilities.md" '\| Agent \|' "the agent-owned duties"

require_phrase "docs/agent/documentation-standards.md" 'bash scripts/check-docs\.sh' "the documentation validation command"
require_phrase "docs/agent/documentation-standards.md" "\\*\\*Audience:\\*\\*" "the audience label convention"

# ---------------------------------------------------------------------------
section "Referenced scripts and versions"
# ---------------------------------------------------------------------------
mapfile -t REFERENCED_SCRIPTS < <(
  grep -ohE 'scripts/[A-Za-z0-9_.-]+\.(sh|ps1)' "${DOC_FILES[@]}" | LC_ALL=C sort -u || true
)

for script_path in ${REFERENCED_SCRIPTS[@]+"${REFERENCED_SCRIPTS[@]}"}; do
  if [[ -f "${script_path}" ]]; then
    pass
    continue
  fi

  planned=0
  for candidate in ${PLANNED_SCRIPTS[@]+"${PLANNED_SCRIPTS[@]}"}; do
    if [[ "${candidate}" == "${script_path}" ]]; then
      planned=1
      break
    fi
  done

  if [[ "${planned}" -eq 1 ]]; then
    skip "${script_path} is documented as planned and does not exist yet"
  else
    report "documentation references '${script_path}', which does not exist"
  fi
done
info "checked ${#REFERENCED_SCRIPTS[@]} referenced script paths"

cargo_version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
if [[ -z "${cargo_version}" ]]; then
  report "the package version could not be read from Cargo.toml"
else
  pass
  info "package version: ${cargo_version}"

  while IFS= read -r reference; do
    [[ -n "${reference}" ]] || continue
    documented_version="${reference#tray-goblin-}"
    documented_version="${documented_version%%-windows-x86_64}"
    if [[ "${documented_version}" == "${cargo_version}" ]]; then
      pass
    else
      report "documented archive '${reference}' does not match Cargo.toml version '${cargo_version}'"
    fi
  done < <(grep -ohE 'tray-goblin-[0-9]+\.[0-9]+\.[0-9]+[A-Za-z0-9.-]*-windows-x86_64' "${DOC_FILES[@]}" | LC_ALL=C sort -u || true)

  while IFS= read -r reference; do
    [[ -n "${reference}" ]] || continue
    documented_version="${reference##* }"
    if [[ "${documented_version}" == "${cargo_version}" ]]; then
      pass
    else
      report "documented command '${reference}' does not match Cargo.toml version '${cargo_version}'"
    fi
  done < <(grep -ohE '(package-release|demo)\.sh [0-9]+\.[0-9]+\.[0-9]+[A-Za-z0-9.-]*' "${DOC_FILES[@]}" | LC_ALL=C sort -u || true)
fi

# ---------------------------------------------------------------------------
section "Task status claims"
# ---------------------------------------------------------------------------
declare -A TASK_STATUS=()
while IFS=' ' read -r task_id task_status; do
  [[ -n "${task_id}" ]] || continue
  TASK_STATUS["${task_id}"]="${task_status}"
done < <(
  awk '/^### T[0-9A-Z]+ / { id = $2 }
       /^- \*\*status:\*\*/ { if (id != "") print id, $3 }' IMPLEMENTATION_PLAN.md
)

if [[ ${#TASK_STATUS[@]} -eq 0 ]]; then
  report "no task statuses could be read from IMPLEMENTATION_PLAN.md"
else
  pass
  info "read ${#TASK_STATUS[@]} task statuses from IMPLEMENTATION_PLAN.md"
fi

for doc in "${DOC_FILES[@]}"; do
  while IFS= read -r line; do
    [[ -n "${line}" ]] || continue
    while IFS= read -r task_id; do
      [[ -n "${task_id}" ]] || continue
      status="${TASK_STATUS[${task_id}]-}"
      [[ -n "${status}" ]] || continue

      if [[ "${status}" == "done" ]] &&
        printf '%s' "${line}" | grep -qiE "${task_id}[^.]*(is|are|remains|stays) (pending|planned|unstarted|not started)"; then
        report "${doc} calls ${task_id} pending, but IMPLEMENTATION_PLAN.md marks it done"
        continue
      fi

      if [[ "${status}" != "done" ]] &&
        printf '%s' "${line}" | grep -qiE "${task_id}[^.]*(is|are|was|were) (complete|completed|done|finished|shipped)"; then
        report "${doc} calls ${task_id} complete, but IMPLEMENTATION_PLAN.md marks it '${status}'"
        continue
      fi

      pass
    done < <(printf '%s' "${line}" | grep -oE '\bT[0-9]+\b|\bTZ\b' | LC_ALL=C sort -u)
  done < <(prose_of "${doc}" | grep -E '\bT[0-9]+\b|\bTZ\b' || true)
done
info "documented task states agree with IMPLEMENTATION_PLAN.md"

# ---------------------------------------------------------------------------
section "Shell script syntax"
# ---------------------------------------------------------------------------
mapfile -t SHELL_SCRIPTS < <(find scripts -maxdepth 1 -type f -name '*.sh' | LC_ALL=C sort)

for script_path in ${SHELL_SCRIPTS[@]+"${SHELL_SCRIPTS[@]}"}; do
  if bash -n "${script_path}" 2>/dev/null; then
    pass
  else
    report "'${script_path}' is not valid bash (bash -n failed)"
  fi
done
info "parsed ${#SHELL_SCRIPTS[@]} shell scripts with 'bash -n'"

if command -v shellcheck >/dev/null 2>&1; then
  for script_path in ${SHELL_SCRIPTS[@]+"${SHELL_SCRIPTS[@]}"}; do
    if shellcheck "${script_path}" >/dev/null 2>&1; then
      pass
    else
      report "shellcheck reported findings in '${script_path}'; run: shellcheck ${script_path}"
    fi
  done
else
  skip "shellcheck is not installed; shell linting was not performed"
fi

if command -v pwsh >/dev/null 2>&1; then
  info "PowerShell scripts are verified by scripts/test-installer.ps1, not by this check"
else
  skip "pwsh is not installed; PowerShell scripts are verified by scripts/test-installer.ps1"
fi

# ---------------------------------------------------------------------------
printf '\n'
if [[ "${failures}" -eq 0 ]]; then
  printf 'Documentation check passed: %d checks, %d skipped.\n' "${checks}" "${skips}"
  exit 0
fi

printf 'Documentation check failed: %d of %d checks failed, %d skipped.\n' \
  "${failures}" "${checks}" "${skips}" >&2
exit 1
