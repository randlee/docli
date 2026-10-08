# Phase B — Normative contract, agent errors, and pre-publish proof (`integrate/phase-b`)

Phase B closes the gaps between Phase A delivery and **safe crates.io publish**:
mandatory **creating-ai-clis** alignment, **hard requirements + ADRs** that
`req-qa` / `arch-qa` enforce, **full error-code test coverage** with actionable
`--json` envelopes, and **local consumer verification** without opening PRs in
atm-core, sc-compose, or sc-observability.

Implementation PRs target **`integrate/phase-b`**, then merge to **`develop`**.
This directory is the sole authority for Phase B sprint deliverables, acceptance
criteria, and validation. [`docs/requirements.md`](../../requirements.md) and
[`docs/architecture.md`](../../architecture.md) are the product and design
baselines; Phase B **lands and locks** them in the repo.

Planning follows [codex-orchestration](../../../.claude/skills/codex-orchestration/SKILL.md)
and [sprint-planning-guidelines](../../../.claude/skills/plan-hardening/sprint-planning-guidelines.md).
Run `/plan-hardening` on this README and sprint docs before implementation dispatch.

## What Phase B closes

- **Normative baselines**: `docs/requirements.md` (hard `REQ-DOCLI-*`, error
  inventory, `REQ-DOCLI-CLI-008`–`010`) and `docs/architecture.md` (canonical
  `ADR-001`–`ADR-003`, `ARCH-RULE-001`–`008`)
- **Full skill vendoring**: complete `.claude/skills/creating-ai-clis/` tree from
  sc-ai-cli with `PROVENANCE.md` (upstream sync policy)
- **Error contract gate**: `tests/error_contract.rs` covers every `DOCLI.*` code;
  `--json` failures emit one envelope on stdout; human stderr is actionable
- **Agent-safe tooling**: `scripts/docli-envelope.sh`, `scripts/verify-candidate-repos.sh`,
  `tests/live_candidate_repos.rs`; `gen-fixtures` recovery hints
- **QA agent alignment**: `req-qa` and `arch-qa` read baselines first; docli
  `ARCH-RULE-*` before generic ATM-style rules
- **Pre-publish consumer proof**: `./scripts/verify-candidate-repos.sh` passes against
  sibling checkouts; fixture refresh policy documented (no silent drift)
- **Publish readiness** (final sprint): crates.io metadata, install path for scripts,
  version and changelog suitable for first publish

## What Phase B does not close

- Go / .NET / Python generators (`REQ-DOCLI-GEN-003`, language adapter reqs)
- MCP server wrapper
- Committing `site/cli/` into consumer repos or opening consumer PRs
- sc-observability clap fixture (still no `Command`)
- Cross-language byte parity beyond Rust

## Commands (unchanged from Phase A)

```text
docli generate [--input <FILE|->] [--html DIR] [--markdown FILE] [--json]
docli show (--html DIR | --markdown FILE) [--json]
cargo docli …
```

Mandatory CLI design: `.claude/skills/creating-ai-clis/` and
`REQ-DOCLI-NORM-001`. Machine contract: `REQ-DOCLI-CLI-001`–`010`.

## Sprint index (6 sprints: b.1–b.6)

| Sprint | Doc | Branch | Closes |
|--------|-----|--------|--------|
| b.1 | [b1-normative-baselines-and-skill.md](b1-normative-baselines-and-skill.md) | `feature/phase-b-b1-baselines` | `REQ-DOCLI-NORM-001`, canonical ADRs in `architecture.md`, requirement index, full skill import |
| b.2 | [b2-error-contract-gate.md](b2-error-contract-gate.md) | `feature/phase-b-b2-errors` | `REQ-DOCLI-CLI-008`–`010`, error inventory tests, expanded `cargo docli` parity |
| b.3 | [b3-verification-tooling.md](b3-verification-tooling.md) | `feature/phase-b-b3-verify` | Pre-publish verify script, envelope helpers, live opt-in test, README |
| b.4 | [b4-qa-agent-alignment.md](b4-qa-agent-alignment.md) | `feature/phase-b-b4-qa-agents` | `req-qa` / `arch-qa` baseline + error-gate instructions |
| b.5 | [b5-consumer-proof-fixture-policy.md](b5-consumer-proof-fixture-policy.md) | `feature/phase-b-b5-consumers` | Fixture regen policy, verify gate evidence, no consumer PRs |
| b.6 | [b6-publish-readiness.md](b6-publish-readiness.md) | `feature/phase-b-b6-publish` | `REQ-DOCLI-PRODUCT-001` registry install path, release docs |

## Phase B CI gate

Every sprint runs from the repo root. Do not replace with a narrower check.

```text
cargo test
cargo test --test error_contract
cargo test --test cli_contract
cargo test --test cargo_docli
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```

Optional maintainer gate (not CI): `DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh`.

## Phase-end QA (codex-orchestration)

After all sprints land on `integrate/phase-b`, `quality-mgr` runs mandatory
reviewers on the integration branch:

- `req-qa` — 100% deliverable completion from this README + sprint docs;
  every `REQ-DOCLI-*` cited in findings must trace to `requirements.md`
- `arch-qa` — `ARCH-RULE-*` / `ADR-*` from `architecture.md`
- `rust-qa-agent`, `rust-best-practices-agent`, `rust-service-hardening-agent`,
  `flaky-test-qa`

Merge to `develop` only when QA PASS, CI green, and triage TTLs closed.
