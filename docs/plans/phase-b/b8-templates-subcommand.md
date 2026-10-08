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

- Agents discover installed template packs and validate custom packs **without**
  guessing filesystem paths.
- Every operation uses the version `"1"` envelope when `--json` is set.

## Hard Dependencies

- b.7 — template manifest format and install root resolution
- b.2 — actionable `DOCLI.*` envelopes

## Deliverables

- `src/ops.rs` — `templates_list`, `templates_show`, `templates_validate`
- `src/cli.rs` — subcommand `Templates { List | Show | Validate }`
- `REQ-DOCLI-CLI-011` in `docs/requirements.md`
- `tests/template_contract.rs` — list/show/validate envelopes
- Bundled templates discoverable under install root (see b.7)

## Subcommand surface

```text
docli templates list [--json]
docli templates show <ID|PATH> [--json]
docli templates validate <PATH> [--json]
```

### `templates list`

Response `data`:

```json
{
  "operation": "templates_list",
  "install_root": "/usr/local/share/docli/templates",
  "templates": [
    {
      "id": "default",
      "name": "docli default",
      "version": "1",
      "description": "Two-pane tree + search (Phase A layout)",
      "path": "/usr/local/share/docli/templates/default"
    }
  ]
}
```

### `templates show`

`<ID|PATH>`: bundled id (`default`) or absolute path to a pack directory.
Response includes manifest fields, supported `theme_schema` keys, and example
`docli generate` argv snippet for agents.

### `templates validate`

Checks pack directory: required files, manifest parse, dry-run render with
`fixtures/contract/model.json`. Failure → `DOCLI.TEMPLATE_INVALID`.

## Acceptance Criteria

- `docli templates list --json` exits 0; lists at least `default`
- `docli templates show default --json` includes `theme_schema` and example command
- `docli templates validate` on a broken pack exits 2 with `suggested_action`
- `cargo docli templates list --json` matches `docli` stdout bytes

## Required Validation

- Phase B host gate in [README.md](README.md)
- `cargo test --test template_contract`
