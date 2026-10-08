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
DOCLI_ROOT="${DOCLI_ROOT:-$(cd "$SCRIPT_DIR/.." && pwd)}"
ATM_CORE_ROOT="${ATM_CORE_ROOT:-$HOME/Documents/github/atm-core}"
SC_COMPOSE_ROOT="${SC_COMPOSE_ROOT:-$HOME/Documents/github/sc-compose}"
SC_OBSERVABILITY_ROOT="${SC_OBSERVABILITY_ROOT:-$HOME/Documents/github/sc-observability}"

for d in "$DOCLI_ROOT" "$ATM_CORE_ROOT" "$SC_COMPOSE_ROOT" "$SC_OBSERVABILITY_ROOT"; do
  if [[ ! -d "$d" ]]; then
    echo "missing checkout: $d" >&2
    exit 1
  fi
done

echo "docli root:              $DOCLI_ROOT"
echo "atm-core:                $ATM_CORE_ROOT"
echo "sc-compose:              $SC_COMPOSE_ROOT"
echo "sc-observability:        $SC_OBSERVABILITY_ROOT"

cd "$DOCLI_ROOT"
cargo build --release -q

if [[ "${DOCLI_SKIP_GEN_FIXTURES:-}" != "1" ]]; then
  echo "== gen-fixtures (live clap → JSON) =="
  cargo run --release --features gen-fixtures --bin gen-fixtures -- \
    --atm-core "$ATM_CORE_ROOT" \
    --sc-compose "$SC_COMPOSE_ROOT"
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
    "$DOCLI" generate --input "$model_json" --html site/cli --json >/dev/null
    test -f site/cli/index.html
    "$DOCLI" show --html site/cli --json >/dev/null
  )
  echo "  OK generate + show ($workdir/site/cli/index.html)"
}

generate_show "docli / contract fixture" "$DOCLI_ROOT" "$DOCLI_ROOT/fixtures/contract/model.json"
generate_show "docli / atm-core.json" "$DOCLI_ROOT" "$DOCLI_ROOT/fixtures/repos/atm-core.json"
generate_show "docli / sc-compose.json" "$DOCLI_ROOT" "$DOCLI_ROOT/fixtures/repos/sc-compose.json"
generate_show "atm-core checkout / atm-core.json" "$ATM_CORE_ROOT" "$DOCLI_ROOT/fixtures/repos/atm-core.json"
generate_show "sc-compose checkout / sc-compose.json" "$SC_COMPOSE_ROOT" "$DOCLI_ROOT/fixtures/repos/sc-compose.json"

echo "== sc-observability (no clap Command fixture) =="
if [[ -f "$DOCLI_ROOT/fixtures/repos/sc-observability.json" ]]; then
  echo "  FAIL: sc-observability.json must not exist" >&2
  exit 1
fi
generate_show "sc-observability tree / contract smoke" "$SC_OBSERVABILITY_ROOT" "$DOCLI_ROOT/fixtures/contract/model.json"

echo "== cargo test (committed fixtures) =="
cargo test --test repo_fixtures -q
cargo test -q

echo "All candidate-repo checks passed."
