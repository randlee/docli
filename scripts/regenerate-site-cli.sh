#!/usr/bin/env bash
# Capture docli's clap tree and render site/cli/index.html for GitHub Pages.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

cargo build --release --bin docli --bin dump-cli-model

mkdir -p fixtures/repos site/cli
target/release/dump-cli-model > fixtures/repos/docli.json
target/release/docli generate \
  --input fixtures/repos/docli.json \
  --html site/cli \
  --json >/dev/null

test -f site/cli/index.html
echo "Wrote fixtures/repos/docli.json and site/cli/index.html"
