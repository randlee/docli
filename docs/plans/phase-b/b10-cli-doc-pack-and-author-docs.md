---
id: b.10
title: cli-doc pack and author skeleton
status: planned
branch: feature/phase-b-b10-cli-doc
worktree: ../docli-worktrees/feature/phase-b-b10-cli-doc
target: integrate/phase-b
---

# Sprint b.10 — cli-doc pack and author skeleton

## Goal

- Ship a second bundled template **`cli-doc`** (layout reference: [spirali/cli_doc](https://github.com/spirali/cli_doc))
- Publish author docs and `_skeleton` pack for new templates

## Hard Dependencies

- b.7–b.9

## Deliverables

- `templates/html/cli-doc/` — card two-column layout, search widget behavior
- `templates/html/_skeleton/` — copy-paste starter with commented `theme_schema`
- [`docs/templates/AUTHOR.md`](../../templates/AUTHOR.md), [`AGENT-PREVIEW.md`](../../templates/AGENT-PREVIEW.md)
- README link under “CLI documentation templates”
- `REQ-DOCLI-HTML-008`–`010` (bundled packs, inline branding, author validate path)
- Golden test: render `fixtures/repos/docli.json` with `cli-doc` → snapshot or smoke asserts

## Acceptance Criteria

- `docli templates list --json` includes `default` and `cli-doc`
- Agent three-call flow in AGENT-PREVIEW.md succeeds (default + cli-doc + themed default)
- AUTHOR.md describes manifest, theme keys, validate, and install paths

## Required Validation

- Phase B host gate in [README.md](README.md)
- `cargo test --test template_contract`
- Optional: `./scripts/preview-all-templates.sh` or documented manual steps
