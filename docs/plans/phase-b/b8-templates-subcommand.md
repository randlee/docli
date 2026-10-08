---
id: b.8
title: templates subcommand
status: planned
branch: feature/phase-b-b8-templates-cmd
worktree: ../docli-worktrees/feature/phase-b-b8-templates-cmd
target: integrate/phase-b
---

# Sprint b.8 — `docli templates` subcommand

## Goal

Agents discover and validate template packs via **`docli templates`** with version `"1"` envelopes.

## Closes

- `REQ-DOCLI-CLI-011`
- `DOCLI.TEMPLATE_INVALID` (requirements section 9 + `tests/error_contract.rs` + `tests/template_contract.rs`)

## Hard Dependencies

- b.7 — manifest format and install root
- b.2 — envelope shape

## Deliverables

- `src/ops.rs` — `templates_list`, `templates_show`, `templates_validate`
- `src/cli.rs` — `Templates { List | Show | Validate }`
- `REQ-DOCLI-CLI-011` + section 11 index update
- [`docs/requirements.md`](../../requirements.md) section 9 row for **`DOCLI.TEMPLATE_INVALID`** (kind, exit **2**) with **Covered by** naming the validate test in `tests/error_contract.rs`
- `tests/template_contract.rs` — list/show/validate success envelopes + **`cargo_docli_templates_list_matches_docli`** (or equivalent) parity test
- `tests/error_contract.rs` — `templates validate` → `DOCLI.TEMPLATE_INVALID` (named test)

## Subcommand surface

```text
docli templates list [--json]
docli templates show <ID|PATH> [--json]
docli templates validate <PATH> [--json]
```

### `templates list` success `data`

```json
{
  "operation": "templates_list",
  "install_root": "/usr/local/share/docli/templates",
  "templates": [{ "id": "default", "name": "docli default", "version": "1", "path": "…" }]
}
```

### `templates show` success `data`

```json
{
  "operation": "templates_show",
  "id": "default",
  "manifest": { "id": "default", "version": "1" },
  "theme_schema": { "accent": "string", "font_body": "string" },
  "example_generate_argv": ["docli", "generate", "--input", "model.json", "--preview", "--template", "default", "--theme", "{}"]
}
```

### `templates validate` failure envelope (exit **2**)

```json
{
  "ok": false,
  "version": "1",
  "data": null,
  "error": {
    "code": "DOCLI.TEMPLATE_INVALID",
    "kind": "dependency",
    "message": "…",
    "details": {},
    "suggested_action": "…",
    "docs": "https://github.com/randlee/docli/blob/develop/docs/requirements.md"
  }
}
```

## Acceptance Criteria

- `docli templates list --json` lists at least `default`
- `docli templates show default --json` includes `theme_schema` and `example_generate_argv`
- Broken pack → validate exits 2, `DOCLI.TEMPLATE_INVALID`, actionable envelope
- Section 9 **Covered by** matches the named `error_contract` test
- `cargo docli templates list --json` matches `docli` stdout bytes (parity test name in sprint doc)

## Required Validation

- Phase B host gate — [README.md](README.md)
- `cargo test --test template_contract`
- `cargo test --test error_contract` — TEMPLATE_INVALID test name from deliverables
- `cargo test --test template_contract` — parity test for `cargo docli templates list`
