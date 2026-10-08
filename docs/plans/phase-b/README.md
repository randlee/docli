# Phase B — normative contract, templates, preview, publish (`integrate/phase-b`)

Sprint docs here are the **authority** for deliverables, acceptance, and validation.
[`docs/requirements.md`](../../requirements.md) and [`docs/architecture.md`](../../architecture.md)
are product baselines. Git workflow (planning branch, hardening, **go**, stack): see
[`docs/plans/project-plan.md`](../project-plan.md) and
[`.plan-hardening/README.md`](../../../.plan-hardening/README.md).

**Templates** are **b.7–b.10** in this phase (not a separate Phase C). Agent preview:
[`docs/templates/AGENT-PREVIEW.md`](../../templates/AGENT-PREVIEW.md).

## As-built on `integrate/phase-b`

PR **#13** may have already landed **b.2** / **b.3**-class work (error contract tests,
verify scripts). Treat those sprints as **close when acceptance passes**, not greenfield
re-implementation.

## What Phase B closes

- Normative req/arch + QA agents (**b.1**); skill tree on **`develop`** via planning branch
- `REQ-DOCLI-CLI-008`–`012`, verify tooling, consumer proof (**b.2–b.3**, **b.5**)
- HTML template packs, `docli templates`, `--preview` / `--theme` (**b.7–b.10**)
- `REQ-DOCLI-PRODUCT-001` publish readiness (**b.11**)

## What Phase B does not close

Go/.NET/Python generators, MCP, consumer repo PRs, sc-observability clap fixture,
crates.io upload itself, opening browsers from the CLI.

## Commands (target state)

```text
docli generate [--input FILE] [--html DIR] [--preview] [--template ID] [--theme JSON] [--json]
docli show (--html DIR | --markdown FILE) [--json]
docli templates list|show|validate [--json]
cargo docli …
```

## Sprint index

| Sprint | Doc | Closes |
|--------|-----|--------|
| b.0 | [b0-integrate-branch.md](b0-integrate-branch.md) | Sync `develop` → `integrate/phase-b` |
| b.1 | [b1-normative-baselines-and-skill.md](b1-normative-baselines-and-skill.md) | Req/arch, req-qa, arch-qa |
| b.2 | [b2-error-contract-gate.md](b2-error-contract-gate.md) | `REQ-DOCLI-CLI-008`–`010` |
| b.3 | [b3-verification-tooling.md](b3-verification-tooling.md) | Verify scripts, live test |
| b.5 | [b5-consumer-proof-fixture-policy.md](b5-consumer-proof-fixture-policy.md) | Fixture policy, consumer proof |
| b.7 | [b7-template-engine-and-layout.md](b7-template-engine-and-layout.md) | `REQ-DOCLI-HTML-007`, default pack |
| b.8 | [b8-templates-subcommand.md](b8-templates-subcommand.md) | `REQ-DOCLI-CLI-011` |
| b.9 | [b9-generate-preview-and-theme.md](b9-generate-preview-and-theme.md) | `REQ-DOCLI-CLI-012` |
| b.10 | [b10-cli-doc-pack-and-author-docs.md](b10-cli-doc-pack-and-author-docs.md) | cli-doc pack, author docs |
| b.11 | [b11-publish-readiness.md](b11-publish-readiness.md) | Registry publish readiness |

Execution stacks use **`/sc-gh-stack`** on trunk **`integrate/phase-b`** after operator **go**.

## Host gate (implementation sprints only)

Run from repo root for **b.1** onward (not **b.0**). Each sprint’s **Required Validation**
references this block plus sprint-specific commands.

```text
cargo test
cargo test --test error_contract
cargo test --test cli_contract
cargo test --test cargo_docli
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```

From **b.8** onward also: `cargo test --test template_contract`.

`DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh` — **b.5** proof only; not default CI.

## Requirement index rule

Section 11 lists every Phase B `REQ-DOCLI-*` at **b.1** with sprint ownership notes. Sprints **b.7–b.11** add requirement bodies and touch the index row in the same PR.

## Phase-end QA

After **b.11** merges to `integrate/phase-b`, run **quality-mgr** once, then merge **`integrate/phase-b` → `develop`**. b.11 does not depend on QA completing first.
