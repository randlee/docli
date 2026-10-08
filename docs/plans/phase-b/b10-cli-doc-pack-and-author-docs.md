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

Second bundled pack **`cli-doc`**, `_skeleton` starter, aligned author/agent docs, HTML requirement text.

## Closes

- `REQ-DOCLI-HTML-008`, `REQ-DOCLI-HTML-009`, `REQ-DOCLI-HTML-010` (+ section 11 rows)

## Hard Dependencies

- b.7–b.9

## Deliverables

1. **Packs:** `templates/html/cli-doc/`, `templates/html/_skeleton/`
2. **Docs:** [`AUTHOR.md`](../../templates/AUTHOR.md), [`AGENT-PREVIEW.md`](../../templates/AGENT-PREVIEW.md) — Phase B ids only (b.7–b.11); no `c.*` or fictional `compact` template
3. **README** link to template docs
4. **Test:** render `fixtures/repos/docli.json` with `--template cli-doc` (smoke or snapshot)

## Acceptance Criteria

- `docli templates list --json` includes `default` and `cli-doc`
- Full three-call flow in AGENT-PREVIEW.md (default + cli-doc + themed default) succeeds
- AUTHOR.md matches install layout and `theme_schema` rules
- No `c.1` / `c.4` / `compact` template references remain in `docs/templates/`

## Required Validation

- Phase B host gate — [README.md](README.md)
- `cargo test --test template_contract`
- AGENT-PREVIEW.md commands (maintainer machine)
- `! rg -n 'c\\.(1|4)|compact' docs/templates/`
