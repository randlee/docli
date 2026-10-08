---
id: b.2
title: error contract integration gate
status: planned
branch: feature/phase-b-b2-errors
worktree: ../docli-worktrees/feature/phase-b-b2-errors
target: integrate/phase-b
---

# Sprint b.2 — error contract integration gate

## Goal

- Every stable `DOCLI.*` error code is **integration-tested** with exit code,
  envelope shape, non-empty `message` and `suggested_action`, and normative
  `details` schema — so agents running `docli --json` always get an actionable
  result, never silence or prose-only stderr.

## Hard Dependencies

- b.1 — `REQ-DOCLI-CLI-008`–`010` and error inventory in `requirements.md`
- [ADR-001](../../architecture.md#adr-001--operations-are-the-library-boundary)
- [ADR-003](../../architecture.md#adr-003--cli-contract-is-ai-first-normative)
- creating-ai-clis [`error-contracts.md`](../../../.claude/skills/creating-ai-clis/references/error-contracts.md)

## Deliverables

- [`docs/requirements.md`](../../requirements.md) — §9 error inventory table (code → exit → details → test name)
- [`tests/common/mod.rs`](../../../tests/common/mod.rs), [`envelope.rs`](../../../tests/common/envelope.rs), [`harness.rs`](../../../tests/common/harness.rs)
- [`tests/error_contract.rs`](../../../tests/error_contract.rs) — one or more tests per row in inventory:

| Code | Minimum tests |
|------|----------------|
| `DOCLI.USAGE` | unknown command, invalid flag, `show` without paths (`--json` + human where applicable) |
| `DOCLI.INPUT_INVALID` | bad JSON, empty file, empty stdin |
| `DOCLI.INPUT_NOT_FOUND` | missing `--input` path (`--json` + human) |
| `DOCLI.OUTPUT_NOT_FOUND` | missing html and html+markdown (`--json` + human) |
| `DOCLI.IO` | unwritable html dir, partial write + `outputs_written`, unreadable index on `show` |
| `DOCLI.INTERNAL` | `ErrorBody::internal` contract / envelope serialization |

- Shared helpers enforce:
  - `REQ-DOCLI-CLI-009`: `--json` failure → single envelope on stdout; contract not stderr-only
  - non-empty `message` and `suggested_action` on every failure envelope
- [`tests/cargo_docli.rs`](../../../tests/cargo_docli.rs) — parity with `docli` for **all** error scenarios in `error_contract.rs` (exit + stdout bytes)
- [`tests/cli_contract.rs`](../../../tests/cli_contract.rs) — deduplicate or delegate to `tests/common/envelope.rs` where overlap exists (no conflicting assertions)

## Explicit Code Samples

Failure envelope (normative):

```json
{
  "version": "1",
  "ok": false,
  "data": null,
  "error": {
    "kind": "validation",
    "code": "DOCLI.INPUT_INVALID",
    "message": "model JSON is missing, empty, or does not match known field types",
    "details": { "cause": "..." },
    "suggested_action": "Fix the model JSON at /path/to/file: ...",
    "docs": null
  }
}
```

Human stderr (non-`--json`) must include code, message, and suggested_action via
`ErrorBody` `Display` (see `src/contract.rs`).

## Out of Scope

- New error codes beyond the Phase A inventory
- MCP error wrapping
- Localized / translated messages

## Acceptance Criteria

- `cargo test --test error_contract` passes
- Every code in requirements §9 inventory has at least one passing test named in that table
- `cargo_docli_matches_docli_error_envelopes` (or successor) covers **every** `--json` error scenario in `error_contract.rs`
- No test passes by discarding stdout (`>/dev/null`) without asserting envelope
- Adding a new `DOCLI.*` code without updating inventory + `error_contract.rs` is a merge blocker (`req-qa` Blocking)

## Required Validation

- Phase B CI gate in [README.md](README.md)
- `cargo test --test error_contract --test cli_contract --test cargo_docli`
