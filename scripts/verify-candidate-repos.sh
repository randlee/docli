#!/usr/bin/env bash
# Verify docli against this repo and the three candidate consumer checkouts.
#
# Defaults assume sibling clones under ~/Documents/github (override with env).
#
# Usage:
#   ./scripts/verify-candidate-repos.sh
#   DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh
#   DOCLI_REFRESH_FIXTURES=1 ./scripts/verify-candidate-repos.sh
#   DOCLI_BIN=docli DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh
#
# Env:
#   DOCLI_ROOT          — docli checkout (default: repo root from script path)
#   DOCLI_BIN           — installed docli for generate/show (post-publish).
#                         Unset: cargo build --release and use target/release/docli.
#                         Set to a path (DOCLI_BIN=/path/to/docli) or a PATH
#                         command (DOCLI_BIN=docli, or DOCLI_BIN="$(command -v docli)").
#                         A set value skips cargo build --release. gen-fixtures
#                         still builds from this checkout unless skip is set.
#   ATM_CORE_ROOT       — atm-core checkout
#   SC_COMPOSE_ROOT     — sc-compose checkout
#   SC_OBSERVABILITY_ROOT — sc-observability checkout (no clap CLI; smoke only)
#   DOCLI_SKIP_GEN_FIXTURES — skip gen-fixtures; wins over DOCLI_REFRESH_FIXTURES
#   DOCLI_REFRESH_FIXTURES — rewrite committed fixtures (reviewed PR; not the proof command)
#
# Mode rules: docs/requirements.md section "Fixture policy".

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=docli-envelope.sh
source "$SCRIPT_DIR/docli-envelope.sh"

DOCLI_ROOT="${DOCLI_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"

resolve_checkout() {
  local env_name=$1
  local sibling=$2
  local val="${!env_name:-}"
  if [[ -n "$val" ]]; then
    if [[ -d "$val" ]]; then
      printf '%s' "$val"
      return 0
    fi
    echo "missing checkout: $env_name ($val)" >&2
    echo "hint: clone the repo, set $env_name to an existing directory, or unset it to use sibling defaults" >&2
    exit 1
  fi
  local candidate="$DOCLI_ROOT/../$sibling"
  if [[ -d "$candidate" ]]; then
    printf '%s' "$candidate"
    return 0
  fi
  candidate="$HOME/Documents/github/$sibling"
  if [[ -d "$candidate" ]]; then
    printf '%s' "$candidate"
    return 0
  fi
  echo "missing checkout: $env_name ($sibling)" >&2
  echo "hint: clone the repo, set $env_name, or place a sibling checkout at $DOCLI_ROOT/../$sibling" >&2
  exit 1
}

DOCLI_ROOT="$(resolve_checkout DOCLI_ROOT docli)"
ATM_CORE_ROOT="$(resolve_checkout ATM_CORE_ROOT atm-core)"
SC_COMPOSE_ROOT="$(resolve_checkout SC_COMPOSE_ROOT sc-compose)"
SC_OBSERVABILITY_ROOT="$(resolve_checkout SC_OBSERVABILITY_ROOT sc-observability)"

echo "docli root:              $DOCLI_ROOT"
echo "atm-core:                $ATM_CORE_ROOT"
echo "sc-compose:              $SC_COMPOSE_ROOT"
echo "sc-observability:        $SC_OBSERVABILITY_ROOT"

cd "$DOCLI_ROOT"

# Pre-publish default builds this checkout. Post-publish sets DOCLI_BIN.
if [[ -z "${DOCLI_BIN:-}" ]]; then
  echo "docli binary: pre-publish (cargo build --release)"
  cargo build --release -q
  DOCLI="$DOCLI_ROOT/target/release/docli"
elif [[ "$DOCLI_BIN" == */* ]]; then
  if [[ ! -x "$DOCLI_BIN" ]]; then
    echo "DOCLI_BIN is not an executable file: $DOCLI_BIN" >&2
    echo "hint: set DOCLI_BIN to the installed docli path, or to the command name docli so PATH is searched; unset DOCLI_BIN to build target/release/docli" >&2
    exit 1
  fi
  DOCLI="$DOCLI_BIN"
  echo "docli binary: $DOCLI (DOCLI_BIN path; skipped cargo build --release)"
else
  resolved_bin="$(command -v "$DOCLI_BIN" || true)"
  if [[ -z "$resolved_bin" || ! -x "$resolved_bin" ]]; then
    echo "DOCLI_BIN is not executable on PATH: $DOCLI_BIN" >&2
    echo "hint: cargo install docli, then DOCLI_BIN=docli or DOCLI_BIN=\"\$(command -v docli)\"; unset DOCLI_BIN to build target/release/docli" >&2
    exit 1
  fi
  DOCLI="$resolved_bin"
  echo "docli binary: $DOCLI (DOCLI_BIN on PATH; skipped cargo build --release)"
fi

fixture_snapshot_dir=""
default_fixture_guard=0

snapshot_committed_fixtures() {
  fixture_snapshot_dir="$(mktemp -d)"
  local f
  for f in "$DOCLI_ROOT/fixtures/repos"/*.json; do
    [[ -f "$f" ]] || continue
    cp -p "$f" "$fixture_snapshot_dir/"
  done
}

# Copy the pre-run snapshot of fixtures/repos/*.json back into place.
restore_committed_fixtures() {
  local f base
  if [[ -z "${fixture_snapshot_dir:-}" || ! -d "$fixture_snapshot_dir" ]]; then
    echo "restore_committed_fixtures: no snapshot directory" >&2
    return 1
  fi
  for f in "$DOCLI_ROOT/fixtures/repos"/*.json; do
    [[ -f "$f" ]] || continue
    base="$(basename "$f")"
    if [[ ! -f "$fixture_snapshot_dir/$base" ]]; then
      rm -f "$f"
    fi
  done
  for f in "$fixture_snapshot_dir"/*.json; do
    [[ -f "$f" ]] || continue
    cp -p "$f" "$DOCLI_ROOT/fixtures/repos/$(basename "$f")"
  done
}

fixture_basenames() {
  local f
  for f in "$fixture_snapshot_dir"/*.json "$DOCLI_ROOT/fixtures/repos"/*.json; do
    [[ -f "$f" ]] || continue
    basename "$f"
  done | sort -u
}

committed_fixtures_differ() {
  local base snap live
  while IFS= read -r base; do
    [[ -n "$base" ]] || continue
    snap="$fixture_snapshot_dir/$base"
    live="$DOCLI_ROOT/fixtures/repos/$base"
    if [[ ! -f "$snap" || ! -f "$live" ]] || ! cmp -s "$snap" "$live"; then
      return 0
    fi
  done < <(fixture_basenames)
  return 1
}

capture_fixture_diff() {
  local base snap live
  while IFS= read -r base; do
    [[ -n "$base" ]] || continue
    snap="$fixture_snapshot_dir/$base"
    live="$DOCLI_ROOT/fixtures/repos/$base"
    if [[ -f "$snap" && -f "$live" ]]; then
      if ! cmp -s "$snap" "$live"; then
        diff -u "$snap" "$live" || true
      fi
    elif [[ -f "$snap" ]]; then
      echo "diff: missing live fixture $base"
    else
      echo "diff: untracked live fixture $base"
    fi
  done < <(fixture_basenames)
}

release_fixture_guard() {
  default_fixture_guard=0
  trap - EXIT
  if [[ -n "${fixture_snapshot_dir:-}" && -d "$fixture_snapshot_dir" ]]; then
    rm -rf "$fixture_snapshot_dir"
    fixture_snapshot_dir=""
  fi
}

guard_restore_fixtures() {
  if [[ "$default_fixture_guard" == "1" ]]; then
    restore_committed_fixtures || true
  fi
  if [[ -n "${fixture_snapshot_dir:-}" && -d "$fixture_snapshot_dir" ]]; then
    rm -rf "$fixture_snapshot_dir"
    fixture_snapshot_dir=""
  fi
}

run_gen_fixtures() {
  echo "== gen-fixtures (live clap → JSON) =="
  if ! cargo run --release --features gen-fixtures --bin gen-fixtures -- \
    --atm-core "$ATM_CORE_ROOT" \
    --sc-compose "$SC_COMPOSE_ROOT"; then
    echo "hint: ensure each consumer checkout is clean (no leftover docli_gen_fixtures injection)" >&2
    echo "hint: try RUSTUP_TOOLCHAIN=stable and cargo clean in the consumer crate if rustc artifacts look mixed" >&2
    return 1
  fi
}

# Three-mode matrix. Skip wins over refresh. Default fail-closes on byte drift.
if [[ "${DOCLI_SKIP_GEN_FIXTURES:-}" == "1" ]]; then
  if [[ "${DOCLI_REFRESH_FIXTURES:-}" == "1" ]]; then
    echo "== gen-fixtures skipped (DOCLI_SKIP_GEN_FIXTURES=1 wins over DOCLI_REFRESH_FIXTURES) =="
  else
    echo "== gen-fixtures skipped (DOCLI_SKIP_GEN_FIXTURES=1; committed JSON) =="
  fi
elif [[ "${DOCLI_REFRESH_FIXTURES:-}" == "1" ]]; then
  echo "== gen-fixtures refresh (DOCLI_REFRESH_FIXTURES=1) =="
  run_gen_fixtures
else
  echo "== gen-fixtures (default: snapshot, diff, restore on drift) =="
  snapshot_committed_fixtures
  default_fixture_guard=1
  trap guard_restore_fixtures EXIT
  if ! run_gen_fixtures; then
    exit 1
  fi
  if committed_fixtures_differ; then
    drift_text="$(capture_fixture_diff)"
    restore_committed_fixtures
    echo "fixture drift: live gen-fixtures output differs from committed fixtures/repos/*.json; restored snapshot" >&2
    printf '%s\n' "$drift_text" >&2
    default_fixture_guard=0
    release_fixture_guard
    exit 1
  fi
  release_fixture_guard
fi

generate_show() {
  local label=$1
  local workdir=$2
  local model_json=$3
  echo "== $label =="
  rm -rf "$workdir/site/cli"
  (
    cd "$workdir"
    docli_require_envelope_ok "$label generate" \
      "$DOCLI" generate --input "$model_json" --html site/cli --json
    test -f site/cli/index.html
    docli_require_envelope_ok "$label show" \
      "$DOCLI" show --html site/cli --json
  )
  echo "  OK generate + show ($workdir/site/cli/index.html)"
}

generate_show_temp_html() {
  local label=$1
  local model_json=$2
  (
    set -euo pipefail
    local tmp_html
    tmp_html="$(mktemp -d)"
    trap 'rm -rf "$tmp_html"' EXIT
    echo "== $label =="
    docli_require_envelope_ok "$label generate" \
      "$DOCLI" generate --input "$model_json" --html "$tmp_html" --json
    test -f "$tmp_html/index.html"
    docli_require_envelope_ok "$label show" \
      "$DOCLI" show --html "$tmp_html" --json
    echo "  OK generate + show (temp html dir; committed site/cli untouched)"
  )
}

generate_show_temp_html "docli / contract fixture" "$DOCLI_ROOT/fixtures/contract/model.json"
generate_show_temp_html "docli / atm-core.json" "$DOCLI_ROOT/fixtures/repos/atm-core.json"
generate_show_temp_html "docli / sc-compose.json" "$DOCLI_ROOT/fixtures/repos/sc-compose.json"
generate_show "atm-core checkout / atm-core.json" "$ATM_CORE_ROOT" "$DOCLI_ROOT/fixtures/repos/atm-core.json"
generate_show "sc-compose checkout / sc-compose.json" "$SC_COMPOSE_ROOT" "$DOCLI_ROOT/fixtures/repos/sc-compose.json"

echo "== sc-observability (no clap Command fixture) =="
if [[ -f "$DOCLI_ROOT/fixtures/repos/sc-observability.json" ]]; then
  echo "  FAIL: sc-observability.json must not exist" >&2
  echo "hint: remove sc-observability.json; that repo has no clap Command to capture" >&2
  exit 1
fi
generate_show "sc-observability tree / contract smoke" "$SC_OBSERVABILITY_ROOT" "$DOCLI_ROOT/fixtures/contract/model.json"

echo "== cargo test (committed fixtures) =="
cargo test --test repo_fixtures -q
cargo test -q

echo "All candidate-repo checks passed."
