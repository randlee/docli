# Phase B — Normative contract, templates, agent preview, and pre-publish proof (`integrate/phase-b`)

Phase B implementation PRs target **`integrate/phase-b`**, then merge to **`develop`**. This directory is the sole authority for Phase B sprint deliverables, acceptance criteria, and validation. [`docs/requirements.md`](../../requirements.md) and [`docs/architecture.md`](../../architecture.md) are the product and design baselines.

Sprint docs are the QA source. This README is the phase index. It does not restate sprint acceptance criteria.

**HTML templates:** There is no separate “Phase C.” Templating is **b.7–b.10** in this
same phase (after b.1–b.5 contract/verify work, before **b.11** publish). Agent preview
docs live under [`docs/templates/`](../../templates/).

## What Phase B closes

- `REQ-DOCLI-NORM-001`: full `.claude/skills/creating-ai-clis/` tree and provenance
- Canonical `ADR-001`–`ADR-004` and `ARCH-RULE-001`–`008` in `docs/architecture.md`, with `req-qa` and `arch-qa` reading those baselines
- `REQ-DOCLI-CLI-008`–`012`: every `DOCLI.*` code tested, actionable `--json` envelopes, `cargo docli` parity
- `scripts/docli-envelope.sh`, `scripts/verify-candidate-repos.sh`, and `tests/live_candidate_repos.rs`
- Committed-fixture versus live `gen-fixtures` policy, and a pre-crates.io consumer proof
- **HTML template packs** installed with the application; **`docli templates list|show|validate`**
- **`docli generate`** with **`--template`**, **`--theme`**, **`--preview`** (temp HTML for multi-layout agent previews)
- Author and agent docs: [`docs/templates/AUTHOR.md`](../../templates/AUTHOR.md), [`docs/templates/AGENT-PREVIEW.md`](../../templates/AGENT-PREVIEW.md)
- `REQ-DOCLI-PRODUCT-001` registry-install readiness (`cargo publish --dry-run`, install docs)

## What Phase B does not close

- Go, .NET, and Python generators (`REQ-DOCLI-GEN-003`, `REQ-DOCLI-GO-001`, `REQ-DOCLI-NET-001`, `REQ-DOCLI-PY-001`)
- An MCP server or wrapper
- Committing `site/cli/` into atm-core, sc-compose, or sc-observability, or opening pull requests in those repos
- A sc-observability clap fixture (that repo has no clap `Command`)
- Cross-language byte parity beyond Rust
- Opening a browser from the CLI (agents/users open `index.html` paths locally)
- Markdown template packs or remote template marketplace
- The crates.io upload itself (b.11 leaves the repo publish-ready; a human publishes after merge to `develop`)

## Commands

```text
docli generate [--input <FILE|->] [--html DIR] [--preview] [--template ID|PATH] [--theme JSON]
               [--markdown FILE] [--json]
docli show (--html DIR | --markdown FILE) [--json]
docli templates list [--json]
docli templates show <ID|PATH> [--json]
docli templates validate <PATH> [--json]
cargo docli generate …
cargo docli show …
cargo docli templates …
```

`--input` defaults to `-`. `--html` defaults to `site/cli` when neither `--preview` nor `--html` is set. `--markdown` has no default. Machine contract: `REQ-DOCLI-CLI-001`–`012`, `REQ-DOCLI-HTML-007`–`010`, and `REQ-DOCLI-NORM-001`.

## Agent preview workflow (normative example)

Given `cli-model.json` and bundled templates `default`, `cli-doc`:

```bash
MODEL=./cli-model.json

docli generate --json --input "$MODEL" --preview --template default \
  --theme '{"accent":"#007acc","font_body":"system-ui"}'

docli generate --json --input "$MODEL" --preview --template cli-doc \
  --theme '{"accent":"#d73a49","font_body":"Monaco"}'

docli generate --json --input "$MODEL" --preview --template default \
  --theme '{"accent":"#059669","font_body":"Inter"}'
```

Each success envelope returns **`preview_dir`** and **`outputs`**. The agent presents three absolute or `file://` paths; the user compares layouts side by side. Full copy: [`docs/templates/AGENT-PREVIEW.md`](../../templates/AGENT-PREVIEW.md).

## Sprint index (12 sprints: b.0–b.5, b.7–b.11)

| Sprint | Doc | Branch | Closes |
|--------|-----|--------|--------|
| b.0 | [b0-integrate-branch.md](b0-integrate-branch.md) | `integrate/phase-b` | Integration branch and worktree. No requirement id. |
| b.1 | [b1-normative-baselines-and-skill.md](b1-normative-baselines-and-skill.md) | `feature/phase-b-b1-baselines` | `REQ-DOCLI-NORM-001`, `ADR-001`–`ADR-003`, `ARCH-RULE-001`–`008` |
| b.2 | [b2-error-contract-gate.md](b2-error-contract-gate.md) | `feature/phase-b-b2-errors` | `REQ-DOCLI-CLI-008`–`010` |
| b.3 | [b3-verification-tooling.md](b3-verification-tooling.md) | `feature/phase-b-b3-verify` | Envelope helper, verify script, live test, `gen-fixtures` hint |
| b.4 | [b4-qa-agent-alignment.md](b4-qa-agent-alignment.md) | `feature/phase-b-b4-qa-agents` | `req-qa` / `arch-qa` baseline instructions |
| b.5 | [b5-consumer-proof-fixture-policy.md](b5-consumer-proof-fixture-policy.md) | `feature/phase-b-b5-consumers` | Fixture policy, pre-publish proof |
| b.7 | [b7-template-engine-and-layout.md](b7-template-engine-and-layout.md) | `feature/phase-b-b7-engine` | `REQ-DOCLI-HTML-007`, ADR-004, bundled `default` pack |
| b.8 | [b8-templates-subcommand.md](b8-templates-subcommand.md) | `feature/phase-b-b8-templates-cmd` | `REQ-DOCLI-CLI-011`, `templates list|show|validate` |
| b.9 | [b9-generate-preview-and-theme.md](b9-generate-preview-and-theme.md) | `feature/phase-b-b9-preview` | `REQ-DOCLI-CLI-012`, `--preview` / `--theme`, template error codes |
| b.10 | [b10-cli-doc-pack-and-author-docs.md](b10-cli-doc-pack-and-author-docs.md) | `feature/phase-b-b10-cli-doc` | `cli-doc` pack, `REQ-DOCLI-HTML-008`–`010`, author docs |
| b.11 | [b11-publish-readiness.md](b11-publish-readiness.md) | `feature/phase-b-b11-publish` | `REQ-DOCLI-PRODUCT-001` registry-install slice |

Sprint **b.6** is unused (reserved). Template work runs **b.7–b.10** before **b.11** publish so the first crates.io release ships template packs and preview flags.

## Host gate

Every sprint from b.0 onward runs this from the repo root. Sprint **Required Validation** sections include this block verbatim plus that sprint's extra commands. Do not replace it with a narrower check.

```text
cargo test
cargo test --test error_contract
cargo test --test cli_contract
cargo test --test cargo_docli
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```

From **b.8** onward, add:

```text
cargo test --test template_contract
```

`DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh` is a b.5 proof command. It is not part of the host gate and it is not default CI.

## Phase-end QA

After b.0–b.5 and b.7–b.11 are on `integrate/phase-b`, `quality-mgr` runs, using each sprint doc as the deliverable source:

- `req-qa` — `docs/requirements.md` ids, including `REQ-DOCLI-CLI-008`–`012` and `REQ-DOCLI-HTML-007`–`010`
- `arch-qa` — `ADR-*` and `ARCH-RULE-*` in `docs/architecture.md`
- `rust-qa-agent`, `rust-best-practices-agent`, `rust-service-hardening-agent`, `flaky-test-qa`

Merge to `develop` when that QA passes, the host gate is green, and triage items are closed.
