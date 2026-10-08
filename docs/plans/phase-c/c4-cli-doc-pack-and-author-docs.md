---
id: c.4
title: cli-doc pack and author skeleton
status: planned
branch: feature/phase-c-c4-cli-doc
worktree: ../docli-worktrees/feature/phase-c-c4-cli-doc
target: integrate/phase-c
---

# Sprint c.4 — cli-doc pack and author skeleton

## Goal

- Ship a second bundled template **`cli-doc`** (layout reference: [spirali/cli_doc](https://github.com/spirali/cli_doc))
- Publish author docs and `_skeleton` pack for new templates

## Hard Dependencies

- c.1–c.3

## Deliverables

- `templates/html/cli-doc/` — card two-column layout, search widget behavior
- `templates/html/_skeleton/` — copy-paste starter with commented `theme_schema`
- [`docs/templates/AUTHOR.md`](../../templates/AUTHOR.md), [`AGENT-PREVIEW.md`](../../templates/AGENT-PREVIEW.md)
- README link under “CLI documentation templates”
- Golden test: render `fixtures/repos/docli.json` with `cli-doc` → snapshot or smoke asserts (no byte lock with contract)

## Acceptance Criteria

- `docli templates list --json` includes `default` and `cli-doc`
- Agent three-call flow in AGENT-PREVIEW.md succeeds on maintainer machine
- AUTHOR.md describes manifest, theme keys, validate, and install paths

## Required Validation

- Full Phase C CI gate
- `./scripts/preview-all-templates.sh` (optional script added this sprint) or documented manual steps
