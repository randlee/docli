---
id: b.3
title: agent-safe verification tooling
status: planned
branch: feature/phase-b-b3-verify
worktree: ../docli-worktrees/feature/phase-b-b3-verify
target: integrate/phase-b
---

# Sprint b.3 — agent-safe verification tooling

## Goal

Maintainer scripts and the opt-in live test surface `DOCLI.*` envelope fields when `docli --json` fails.

## Closes

No new requirement id. This sprint implements the wrapper consequence of:

- `REQ-DOCLI-CLI-009` — `--json` failures are one stdout envelope
- `ADR-003` — callers surface `code`, `message`, and `suggested_action`

The pre-publish pass against real checkouts, and fail-closed fixture drift, are b.5.

## Hard Dependencies

- b.2 merged to `integrate/phase-b` (`tests/common/envelope.rs`, error inventory)
- b.1 requirements QA section

## Deliverables

- [`scripts/docli-envelope.sh`](../../../scripts/docli-envelope.sh) — `docli_require_envelope_ok` as in Explicit Code Samples. Failure paths print the envelope. They do not redirect `--json` stdout to `/dev/null`.
- [`scripts/verify-candidate-repos.sh`](../../../scripts/verify-candidate-repos.sh)
  - Sources `docli-envelope.sh`.
  - Resolves `DOCLI_ROOT`, `ATM_CORE_ROOT`, `SC_COMPOSE_ROOT`, and `SC_OBSERVABILITY_ROOT` in the order in Explicit Code Samples.
  - Builds `target/release/docli` with `cargo build --release` and uses that binary.
  - `DOCLI_SKIP_GEN_FIXTURES=1` skips `gen-fixtures`.
  - When the skip flag is unset, runs `cargo run --release --features gen-fixtures --bin gen-fixtures -- --atm-core <ATM_CORE_ROOT> --sc-compose <SC_COMPOSE_ROOT>`. Fail-closed drift restore is b.5; until then a failed `gen-fixtures` exits non-zero after the `hint:` line.
  - `generate` + `show` via `docli_require_envelope_ok` for:
    - docli repo + `fixtures/contract/model.json` (HTML dir is a temp directory; the run deletes that temp directory and does not delete committed `site/cli/`)
    - docli repo + `fixtures/repos/atm-core.json` (temp HTML dir)
    - docli repo + `fixtures/repos/sc-compose.json` (temp HTML dir)
    - atm-core checkout + `fixtures/repos/atm-core.json` with `--html site/cli` (remove that checkout's `site/cli` first)
    - sc-compose checkout + `fixtures/repos/sc-compose.json` with `--html site/cli`
    - sc-observability checkout + `fixtures/contract/model.json` with `--html site/cli` (smoke only)
  - If `fixtures/repos/sc-observability.json` exists, exit non-zero and print a `hint:` that the file must not exist.
  - Missing checkout: exit `1`, stderr contains `missing checkout:` and `hint:`.
- [`tests/live_candidate_repos.rs`](../../../tests/live_candidate_repos.rs) — without `DOCLI_LIVE_CANDIDATE_REPOS=1`, the test returns and passes. With `DOCLI_LIVE_CANDIDATE_REPOS=1`, it requires `ATM_CORE_ROOT`, `SC_COMPOSE_ROOT`, and `SC_OBSERVABILITY_ROOT` and runs the same six generate/show pairs as the script, asserting envelope `version` `"1"` and `ok: true`. On `ok: false` the panic text includes `code`, `message`, and `suggested_action`. It asserts `fixtures/repos/sc-observability.json` is absent. It does not run `gen-fixtures`.
- [`src/bin/gen-fixtures.rs`](../../../src/bin/gen-fixtures.rs) — every `Err` path prints `hint:` on stderr before exit `1`. The hint names a clean consumer checkout, `cargo clean` in the consumer crate, and `RUSTUP_TOOLCHAIN=stable`.
- [`README.md`](../../../README.md) section `## Candidate repo verification` documents the script, `DOCLI_SKIP_GEN_FIXTURES`, the four root variables, and the live-test env var. Fixture-policy rules are b.5.

## Explicit Code Samples

```bash
# docli_require_envelope_ok LABEL CMD...
# Run CMD with stdout and stderr captured.
# Exit 0 only when CMD exits 0 and stdout JSON has version "1" and ok true.
# Otherwise print to stderr:
#   FAIL: LABEL (exit N)
#   code: <error.code>
#   message: <error.message>
#   suggested_action: <error.suggested_action>
#   cause: <error.details.cause>    # only when details.cause is a string
# and return non-zero. Do not delete stdout before that print.
docli_require_envelope_ok() { :; }
```

```bash
# resolve_checkout ENV_NAME SIBLING_NAME
# 1. Use the environment variable when it is set and non-empty.
# 2. Else use "$DOCLI_ROOT/../$SIBLING_NAME" when that path is a directory.
# 3. Else use "$HOME/Documents/github/$SIBLING_NAME" when that path is a directory.
# 4. Else the caller exits 1 with "missing checkout:" and "hint:".
```

Live-test skip:

```rust
fn live_candidate_repos_generate_show() {
    if std::env::var("DOCLI_LIVE_CANDIDATE_REPOS").ok().as_deref() != Some("1") {
        eprintln!("skip live_candidate_repos (set DOCLI_LIVE_CANDIDATE_REPOS=1)");
        return;
    }
    // generate --json and show --json for the six pairs; panic with code,
    // message, and suggested_action when ok is not true.
}
```

## Out of Scope

- Default CI running live candidate repos or `verify-candidate-repos.sh`
- Fail-closed fixture drift and `DOCLI_REFRESH_FIXTURES` (b.5)
- Opening pull requests in consumer repos
- `DOCLI_BIN` / `cargo install docli` (b.11)
- Rewriting committed `fixtures/repos/*.json` as a success path

## Acceptance Criteria

- `docli_require_envelope_ok` on `docli generate --input /no/such.json --json` exits non-zero and stderr contains `code: DOCLI.INPUT_NOT_FOUND` and `suggested_action:`
- A missing `ATM_CORE_ROOT` makes `verify-candidate-repos.sh` exit `1` with `missing checkout:` and `hint:`
- `DOCLI_SKIP_GEN_FIXTURES=1` does not invoke `gen-fixtures`
- Docli-repo generate/show uses a temp HTML directory and leaves committed `site/cli/` in place
- `cargo test --test live_candidate_repos` passes when `DOCLI_LIVE_CANDIDATE_REPOS` is unset, and the test prints `skip live_candidate_repos`
- With `DOCLI_LIVE_CANDIDATE_REPOS=1`, the test runs the six generate/show pairs and does not return at the skip
- `gen-fixtures` failure stderr contains a `hint:` line
- Script failure paths do not send `docli --json` stdout to `/dev/null`
- The sibling-checkout success run is b.5, not a b.3 closure

## Required Validation

```text
cargo test
cargo test --test error_contract
cargo test --test cli_contract
cargo test --test cargo_docli
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```

```text
bash -n scripts/docli-envelope.sh
bash -n scripts/verify-candidate-repos.sh
cargo build --release
set +e
bash -c 'source scripts/docli-envelope.sh; docli_require_envelope_ok missing ./target/release/docli generate --input /no/such.json --json' >/tmp/docli-env.out 2>/tmp/docli-env.err
env_rc=$?
set -e
test "$env_rc" -ne 0
rg -n "code: DOCLI\\.INPUT_NOT_FOUND" /tmp/docli-env.err
rg -n "suggested_action:" /tmp/docli-env.err
set +e
ATM_CORE_ROOT=/no/such-atm-core DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh >/tmp/docli-verify.out 2>/tmp/docli-verify.err
verify_rc=$?
set -e
test "$verify_rc" -eq 1
rg -n "missing checkout:" /tmp/docli-verify.err
rg -n "hint:" /tmp/docli-verify.err
env -u DOCLI_LIVE_CANDIDATE_REPOS cargo test --test live_candidate_repos -- --nocapture
rg -n "skip live_candidate_repos" tests/live_candidate_repos.rs
set +e
cargo run --release --features gen-fixtures --bin gen-fixtures -- --atm-core /no/such-atm-core --sc-compose /no/such-sc-compose >/tmp/docli-gen.out 2>/tmp/docli-gen.err
gen_rc=$?
set -e
test "$gen_rc" -ne 0
rg -n "^hint:|hint:" /tmp/docli-gen.err
rg -n "^## Candidate repo verification$" README.md
rg -n "DOCLI_SKIP_GEN_FIXTURES" README.md scripts/verify-candidate-repos.sh
rg -n "DOCLI_LIVE_CANDIDATE_REPOS" README.md tests/live_candidate_repos.rs
if rg -n "/dev/null" scripts/docli-envelope.sh scripts/verify-candidate-repos.sh; then
  echo "json stdout must not be discarded" >&2
  exit 1
fi
```
