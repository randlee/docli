#!/usr/bin/env bash
# Verify docli against this repo and the three candidate consumer checkouts.
#
# Defaults assume sibling clones under ~/Documents/github (override with env).
#
# Usage:
#   ./scripts/verify-candidate-repos.sh
#   DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh
#
# Env:
#   DOCLI_ROOT          — docli checkout (default: repo root from script path)
#   ATM_CORE_ROOT       — atm-core checkout
#   SC_COMPOSE_ROOT     — sc-compose checkout
#   SC_OBSERVABILITY_ROOT — sc-observability checkout (no clap CLI; smoke only)
#   DOCLI_SKIP_GEN_FIXTURES — set to skip regen (default: run gen-fixtures)

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
cargo build --release -q

if [[ "${DOCLI_SKIP_GEN_FIXTURES:-}" != "1" ]]; then
  echo "== gen-fixtures (live clap → JSON) =="
  if ! cargo run --release --features gen-fixtures --bin gen-fixtures -- \
    --atm-core "$ATM_CORE_ROOT" \
    --sc-compose "$SC_COMPOSE_ROOT"; then
    echo "hint: ensure each consumer checkout is clean (no leftover docli_gen_fixtures injection)" >&2
    echo "hint: try RUSTUP_TOOLCHAIN=stable and cargo clean in the consumer crate if rustc artifacts look mixed" >&2
    exit 1
  fi
fi

DOCLI="$DOCLI_ROOT/target/release/docli"

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
  local tmp_html
  tmp_html="$(mktemp -d)"
  trap 'rm -rf "$tmp_html"' RETURN
  echo "== $label =="
  docli_require_envelope_ok "$label generate" \
    "$DOCLI" generate --input "$model_json" --html "$tmp_html" --json
  test -f "$tmp_html/index.html"
  docli_require_envelope_ok "$label show" \
    "$DOCLI" show --html "$tmp_html" --json
  rm -rf "$tmp_html"
  echo "  OK generate + show (temp html dir; committed site/cli untouched)"
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
