#!/bin/sh
set -eu
ROOT="$(cd "$(dirname "$0")" && pwd)"
cat "$ROOT/vendor/jquery.min.js" \
  "$ROOT/vendor/fancytree-all.min.js" \
  "$ROOT/docli-app.js" > "$ROOT/script.js"
echo "Wrote $ROOT/script.js ($(wc -c < "$ROOT/script.js" | tr -d ' ') bytes)"
