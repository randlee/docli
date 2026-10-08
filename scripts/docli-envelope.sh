#!/usr/bin/env bash
# Helpers for scripts that invoke `docli --json` (sc-ai-cli actionable errors).
#
# On failure, prints code, message, suggested_action, and details.cause from the
# version "1" envelope on stdout; falls back to stderr/exit code when stdout is not JSON.

set -euo pipefail

_docli_print_envelope_error() {
  local label=$1
  local stdout_file=$2
  local stderr_file=$3
  local exit_code=$4

  echo "FAIL: $label (exit $exit_code)" >&2
  if [[ -s "$stdout_file" ]]; then
    python3 - "$stdout_file" <<'PY' >&2 || true
import json, sys
path = sys.argv[1]
try:
    with open(path) as f:
        e = json.load(f)
except json.JSONDecodeError as err:
    print(f"stdout is not JSON: {err}", file=sys.stderr)
    with open(path) as f:
        print(f.read(), file=sys.stderr)
    raise SystemExit(0)
if e.get("ok") is True:
    print("envelope ok:true but command exited non-zero", file=sys.stderr)
err = e.get("error") or {}
print(f"code: {err.get('code')}", file=sys.stderr)
print(f"message: {err.get('message')}", file=sys.stderr)
print(f"suggested_action: {err.get('suggested_action')}", file=sys.stderr)
details = err.get("details") or {}
if isinstance(details, dict) and details.get("cause"):
    print(f"cause: {details['cause']}", file=sys.stderr)
PY
  fi
  if [[ -s "$stderr_file" ]]; then
    echo "--- stderr ---" >&2
    cat "$stderr_file" >&2
  fi
}

# Run docli (or any command) that must print a success envelope on stdout.
# Usage: docli_require_envelope_ok "label" docli generate ...
docli_require_envelope_ok() {
  local label=$1
  shift
  local stdout stderr
  stdout="$(mktemp)"
  stderr="$(mktemp)"

  set +e
  "$@" >"$stdout" 2>"$stderr"
  local rc=$?
  set -e

  if [[ $rc -ne 0 ]]; then
    _docli_print_envelope_error "$label" "$stdout" "$stderr" "$rc"
    rm -f "$stdout" "$stderr"
    return 1
  fi

  if ! python3 - "$stdout" <<'PY'
import json, sys
with open(sys.argv[1]) as f:
    e = json.load(f)
if e.get("version") != "1":
    raise SystemExit("envelope version is not \"1\"")
if e.get("ok") is not True:
    err = e.get("error") or {}
    print(err.get("code"), err.get("message"), err.get("suggested_action"), sep="\n", file=sys.stderr)
    raise SystemExit("envelope ok is not true")
PY
  then
    _docli_print_envelope_error "$label" "$stdout" "$stderr" 0
    rm -f "$stdout" "$stderr"
    return 1
  fi

  rm -f "$stdout" "$stderr"
}
