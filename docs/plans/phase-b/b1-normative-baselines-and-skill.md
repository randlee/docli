---
id: b.1
title: normative baselines and full creating-ai-clis skill
status: planned
branch: feature/phase-b-b1-baselines
worktree: ../docli-worktrees/feature/phase-b-b1-baselines
target: integrate/phase-b
---

# Sprint b.1 — normative baselines and full creating-ai-clis skill

## Goal

`docs/requirements.md`, `docs/architecture.md`, and a complete vendored creating-ai-clis tree are the baselines later sprints and QA agents cite.

## Closes

- `REQ-DOCLI-NORM-001`
- `ADR-001`, `ADR-002`, `ADR-003`
- `ARCH-RULE-001` through `ARCH-RULE-008`

`REQ-DOCLI-CLI-008`–`010` stay written in `requirements.md`. Test closure is b.2. Agent prompt closure is b.4.

## Hard Dependencies

- b.0 — `integrate/phase-b` exists. Branch this sprint from that tip.
- Upstream skill tree: `synaptic-canvas/packages/sc-ai-cli` path `skills/creating-ai-clis/` at skill version `0.12.0`.

## Deliverables

- [`docs/requirements.md`](../../requirements.md)
  - Header states this file is the hard `REQ-DOCLI-*` baseline, names `docs/architecture.md` as the ADR baseline, and points QA at `.claude/skills/creating-ai-clis/` for `REQ-DOCLI-NORM-001`.
  - `REQ-DOCLI-NORM-001` requires conformance with that vendored skill and its references.
  - Section 9 keeps `REQ-DOCLI-CLI-001`–`010` and the error inventory (code, exit, kind, details keys).
  - The requirement index heading is `## 11. Requirement index` and lists every `REQ-DOCLI-*` id in the file, with an ADR column for `ADR-001`, `ADR-002`, and `ADR-003` where those ADRs cite the id.
- [`docs/architecture.md`](../../architecture.md)
  - Full decision text for `ADR-001`, `ADR-002`, and `ADR-003` (this file is the canonical copy).
  - `ADR-003` requirements line includes `REQ-DOCLI-NORM-001` and `REQ-DOCLI-CLI-001`–`010`.
  - `ARCH-RULE-001`–`ARCH-RULE-008` table, each row tied to an ADR.
  - Error-contract section names `tests/error_contract.rs` as the merge gate b.2 implements for every `DOCLI.*` code in requirements section 9.
- [`.claude/skills/creating-ai-clis/`](../../../.claude/skills/creating-ai-clis/) — wholesale copy of the upstream tree at the recorded revision, plus docli-owned `PROVENANCE.md`. Minimum paths that must exist because `SKILL.md` cites them or the Rust templates are part of that tree:

  - `SKILL.md` (`version: 0.12.0`)
  - `references/core-contract.md`
  - `references/error-contracts.md`
  - `references/mcp-compatibility.md`
  - `references/simulation-and-auditability.md`
  - `references/template-generation.md`
  - `references/example-repos.md`
  - `references/rust.md`
  - `references/rust-examples.md`
  - `assets/templates/rust/Cargo.toml.j2`
  - `assets/templates/rust/src/backend.rs.j2`
  - `assets/templates/rust/src/contracts.rs.j2`
  - `assets/templates/rust/src/main.rs.j2`
  - `assets/templates/rust/src/operations.rs.j2`
  - `PROVENANCE.md`

  Any other file in the upstream tree at the recorded revision is also vendored. Vendored files other than `PROVENANCE.md` are unmodified upstream bytes.

- `PROVENANCE.md` contains these fields:

```text
upstream_package: synaptic-canvas/packages/sc-ai-cli
upstream_path: skills/creating-ai-clis
skill_version: 0.12.0
upstream_revision: <git sha of the imported tree>
policy: wholesale re-import. Replace the tree and update upstream_revision. Do not edit vendored files in place. PROVENANCE.md is the only docli-owned file in this directory.
```

- [`docs/plans/phase-a/adr-001-ops-boundary.md`](../phase-a/adr-001-ops-boundary.md) and [`docs/plans/phase-a/adr-002-search-index.md`](../phase-a/adr-002-search-index.md) keep a banner that canonical text is `docs/architecture.md`. `adr-001-ops-boundary.md` cites `docs/requirements.md` section 9 and `ADR-003` for the error contract.
- [`docs/plans/project-plan.md`](../project-plan.md) Phase B row links to [README.md](README.md).
- [`README.md`](../../../README.md) links to `docs/requirements.md`, `docs/architecture.md`, and `.claude/skills/creating-ai-clis/`.

## Out of Scope

- New CLI commands, error codes, or envelope fields
- `tests/error_contract.rs` coverage (b.2)
- `.claude/agents/req-qa.md` and `.claude/agents/arch-qa.md` instruction edits (b.4)
- crates.io metadata (b.11)
- Renderer HTML byte changes

## Acceptance Criteria

- `docs/requirements.md` section 11 is titled `## 11. Requirement index` and includes `REQ-DOCLI-NORM-001` and `REQ-DOCLI-CLI-001`–`010`
- `docs/architecture.md` contains `ADR-001`, `ADR-002`, `ADR-003`, and `ARCH-RULE-001` through `ARCH-RULE-008`, and `ADR-003` cites `REQ-DOCLI-CLI-008`, `REQ-DOCLI-CLI-009`, and `REQ-DOCLI-CLI-010`
- The vendored skill matches the upstream tree at `upstream_revision` except `PROVENANCE.md`, and every minimum path above is present
- `PROVENANCE.md` records `synaptic-canvas/packages/sc-ai-cli`, `skills/creating-ai-clis`, `0.12.0`, a git sha, and the wholesale re-import policy
- `docs/plans/phase-a/adr-001-ops-boundary.md` does not cite `a1-cli-contract.md` as the error-contract specification
- `README.md` and `docs/plans/project-plan.md` link the baselines named in Deliverables

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
test -f .claude/skills/creating-ai-clis/PROVENANCE.md
test -f .claude/skills/creating-ai-clis/SKILL.md
for f in \
  references/core-contract.md \
  references/error-contracts.md \
  references/mcp-compatibility.md \
  references/simulation-and-auditability.md \
  references/template-generation.md \
  references/example-repos.md \
  references/rust.md \
  references/rust-examples.md \
  assets/templates/rust/Cargo.toml.j2 \
  assets/templates/rust/src/backend.rs.j2 \
  assets/templates/rust/src/contracts.rs.j2 \
  assets/templates/rust/src/main.rs.j2 \
  assets/templates/rust/src/operations.rs.j2
do
  test -f ".claude/skills/creating-ai-clis/$f"
done
rg -n "^version: 0\\.12\\.0$" .claude/skills/creating-ai-clis/SKILL.md
rg -n "upstream_package: synaptic-canvas/packages/sc-ai-cli" .claude/skills/creating-ai-clis/PROVENANCE.md
rg -n "upstream_path: skills/creating-ai-clis" .claude/skills/creating-ai-clis/PROVENANCE.md
rg -n "skill_version: 0\\.12\\.0" .claude/skills/creating-ai-clis/PROVENANCE.md
rg -n "^upstream_revision: [0-9a-f]{40}$" .claude/skills/creating-ai-clis/PROVENANCE.md
rg -n "wholesale re-import" .claude/skills/creating-ai-clis/PROVENANCE.md
: "${SC_AI_CLI_ROOT:?set SC_AI_CLI_ROOT to the sc-ai-cli checkout at upstream_revision}"
diff -rq --exclude PROVENANCE.md "${SC_AI_CLI_ROOT}/skills/creating-ai-clis" .claude/skills/creating-ai-clis
rg -n "^## 11\\. Requirement index$" docs/requirements.md
rg -n "REQ-DOCLI-NORM-001" docs/requirements.md
rg -n "REQ-DOCLI-CLI-008" docs/architecture.md
rg -n "REQ-DOCLI-CLI-009" docs/architecture.md
rg -n "REQ-DOCLI-CLI-010" docs/architecture.md
rg -n "ARCH-RULE-001" docs/architecture.md
rg -n "ARCH-RULE-002" docs/architecture.md
rg -n "ARCH-RULE-003" docs/architecture.md
rg -n "ARCH-RULE-004" docs/architecture.md
rg -n "ARCH-RULE-005" docs/architecture.md
rg -n "ARCH-RULE-006" docs/architecture.md
rg -n "ARCH-RULE-007" docs/architecture.md
rg -n "ARCH-RULE-008" docs/architecture.md
rg -n "docs/architecture.md" docs/plans/phase-a/adr-001-ops-boundary.md docs/plans/phase-a/adr-002-search-index.md
rg -n "requirements.md" docs/plans/phase-a/adr-001-ops-boundary.md
if rg -n "a1-cli-contract.md" docs/plans/phase-a/adr-001-ops-boundary.md; then
  echo "adr-001 still points the error contract at the phase-a sprint doc" >&2
  exit 1
fi
rg -n "phase-b/README.md" docs/plans/project-plan.md
rg -n "docs/requirements.md" README.md
rg -n "docs/architecture.md" README.md
rg -n "creating-ai-clis" README.md
```
