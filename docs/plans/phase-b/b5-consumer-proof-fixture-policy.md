---
id: b.5
title: consumer proof and fixture policy
status: planned
branch: feature/phase-b-b5-consumers
worktree: ../docli-worktrees/feature/phase-b-b5-consumers
target: integrate/phase-b
---

# Sprint b.5 — consumer proof and fixture policy

## Goal

Prove `docli generate` and `docli show` on real atm-core, sc-compose, and sc-observability checkouts from committed JSON, and make live `gen-fixtures` fail closed when that JSON would change.

## Closes

- `REQ-DOCLI-GEN-002` fixture lock for this repo: CI reads committed `fixtures/repos/*.json` only
- Pre-publish evidence for the `REQ-DOCLI-PRODUCT-001` aggregate (the registry upload stays b.11)

No consumer repository is modified.

## Hard Dependencies

- b.3 merged to `integrate/phase-b` (`scripts/verify-candidate-repos.sh`, `scripts/docli-envelope.sh`, `tests/live_candidate_repos.rs`)
- Phase A `fixtures/repos/atm-core.json`, `fixtures/repos/sc-compose.json`, `gen-fixtures`, `tests/repo_fixtures.rs`
- Sibling checkouts available as `ATM_CORE_ROOT`, `SC_COMPOSE_ROOT`, and `SC_OBSERVABILITY_ROOT` (b.3 resolution order)

## Deliverables

- [`docs/requirements.md`](../../requirements.md) subsection `### Fixture policy` under section 8. That subsection is the only statement of the policy. Text:

```markdown
### Fixture policy

CI (`cargo test`, including `tests/repo_fixtures.rs`) reads only the committed files `fixtures/repos/atm-core.json` and `fixtures/repos/sc-compose.json`.

`fixtures/repos/sc-observability.json` is not a file in this repository. sc-observability has no clap `Command` to capture.

`./scripts/verify-candidate-repos.sh` has three modes:

1. `DOCLI_SKIP_GEN_FIXTURES=1` — do not run `gen-fixtures`. `generate` and `show` use the committed JSON. This is the pre-publish proof command. When `DOCLI_REFRESH_FIXTURES` is also set, skip still wins and the script does not write fixtures.
2. Default (neither flag set) — snapshot `fixtures/repos/*.json`, run `gen-fixtures`, and diff against the snapshot. If the bytes differ, call `restore_committed_fixtures`, print the diff, and exit non-zero. The script does not leave a dirty fixture behind.
3. `DOCLI_REFRESH_FIXTURES=1` (and skip unset) — write the captured JSON over the committed files. That write is a reviewed pull request. It is not the proof command.

Live capture may differ from committed JSON when upstream help text is empty. That difference is a refresh pull request, not a silent verify success.
```

- [`scripts/verify-candidate-repos.sh`](../../../scripts/verify-candidate-repos.sh) implements those three modes, including a function named `restore_committed_fixtures`.
- [`README.md`](../../../README.md) `## Candidate repo verification` links to `docs/requirements.md` section `Fixture policy` and names `DOCLI_REFRESH_FIXTURES`. It does not restate the three modes.
- [`docs/plans/phase-b/evidence/b5-consumer-proof.md`](evidence/b5-consumer-proof.md) records the proof run: command, exit code `0`, and an OK line for each label below. It also records `gh pr list` JSON for the three consumer remotes.
  - `docli / contract fixture`
  - `docli / atm-core.json`
  - `docli / sc-compose.json`
  - `atm-core checkout / atm-core.json`
  - `sc-compose checkout / sc-compose.json`
  - `sc-observability tree / contract smoke`
- This sprint does not open a pull request in atm-core, sc-compose, or sc-observability, and it does not commit `site/cli/` into those repos. The docli-repo proof uses the b.3 temp HTML directory so committed `site/cli/index.html` is unchanged.

## Explicit Code Samples

```bash
restore_committed_fixtures() {
  : # copy the pre-run snapshot of fixtures/repos/*.json back into place
}

if [[ "${DOCLI_SKIP_GEN_FIXTURES:-}" == "1" ]]; then
  : # committed JSON only; ignore DOCLI_REFRESH_FIXTURES
elif [[ "${DOCLI_REFRESH_FIXTURES:-}" == "1" ]]; then
  cargo run --release --features gen-fixtures --bin gen-fixtures -- \
    --atm-core "$ATM_CORE_ROOT" --sc-compose "$SC_COMPOSE_ROOT"
else
  # snapshot, run gen-fixtures, diff, restore_committed_fixtures and exit 1 on drift
  :
fi
```

## Out of Scope

- Committing `site/cli/` in atm-core, sc-compose, or sc-observability
- A sc-observability clap model or `fixtures/repos/sc-observability.json`
- Pinning those repos to a docli git SHA or crates.io version
- Treating a live `gen-fixtures` diff as success
- `cargo install docli` (b.11)

## Acceptance Criteria

- `docs/requirements.md` contains the Fixture policy subsection above, including the three modes and the absence of `fixtures/repos/sc-observability.json`
- `DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh` exits `0` against the sibling checkouts and does not modify `fixtures/repos/*.json` or committed `site/cli/index.html`
- `DOCLI_LIVE_CANDIDATE_REPOS=1 cargo test --test live_candidate_repos` exits `0` with those roots set
- `cargo test --test repo_fixtures` passes, including `repo_fixtures_generate_and_show_match_hashes` and `sc_observability_repo_fixture_is_absent`
- Default mode (neither flag) calls `restore_committed_fixtures` when captured JSON differs from the snapshot
- `docs/plans/phase-b/evidence/b5-consumer-proof.md` records exit code `0` and the six OK labels
- Recorded `gh pr list` output shows this sprint opened no consumer pull request that adds `site/cli`

## Required Validation

- Phase B host gate — [README.md](README.md)

```text
rg -n "^### Fixture policy$" docs/requirements.md
rg -n "DOCLI_SKIP_GEN_FIXTURES=1" docs/requirements.md
rg -n "DOCLI_REFRESH_FIXTURES=1" docs/requirements.md
rg -n "restore_committed_fixtures" docs/requirements.md scripts/verify-candidate-repos.sh
rg -n "Fixture policy" README.md
test ! -e fixtures/repos/sc-observability.json
cargo test --test repo_fixtures
cargo test --test repo_fixtures repo_fixtures_generate_and_show_match_hashes
cargo test --test repo_fixtures sc_observability_repo_fixture_is_absent
: "${ATM_CORE_ROOT:?set ATM_CORE_ROOT}"
: "${SC_COMPOSE_ROOT:?set SC_COMPOSE_ROOT}"
: "${SC_OBSERVABILITY_ROOT:?set SC_OBSERVABILITY_ROOT}"
DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh
DOCLI_LIVE_CANDIDATE_REPOS=1 cargo test --test live_candidate_repos -- --nocapture
git diff -- fixtures/repos site/cli/index.html
for root in "$ATM_CORE_ROOT" "$SC_COMPOSE_ROOT" "$SC_OBSERVABILITY_ROOT"; do
  url="$(git -C "$root" remote get-url origin)"
  gh pr list --repo "$url" --state open --json number,title,body --limit 50
done
test -f docs/plans/phase-b/evidence/b5-consumer-proof.md
rg -n "exit code: 0" docs/plans/phase-b/evidence/b5-consumer-proof.md
rg -n "docli / contract fixture" docs/plans/phase-b/evidence/b5-consumer-proof.md
rg -n "docli / atm-core.json" docs/plans/phase-b/evidence/b5-consumer-proof.md
rg -n "docli / sc-compose.json" docs/plans/phase-b/evidence/b5-consumer-proof.md
rg -n "atm-core checkout / atm-core.json" docs/plans/phase-b/evidence/b5-consumer-proof.md
rg -n "sc-compose checkout / sc-compose.json" docs/plans/phase-b/evidence/b5-consumer-proof.md
rg -n "sc-observability tree / contract smoke" docs/plans/phase-b/evidence/b5-consumer-proof.md
```

`git diff -- fixtures/repos site/cli/index.html` must print nothing. Paste the `gh pr list` JSON into the evidence file before the evidence `rg` lines.
