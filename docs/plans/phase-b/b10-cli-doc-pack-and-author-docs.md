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
2. **Requirements:** [`docs/requirements.md`](../../requirements.md) bodies for **`REQ-DOCLI-HTML-008`–`010`** + section 11 rows (same PR)
3. **Docs:** [`AUTHOR.md`](../../templates/AUTHOR.md), [`AGENT-PREVIEW.md`](../../templates/AGENT-PREVIEW.md) — Phase B ids only; no `c.*` or `compact`
4. **README** link to template docs
5. **Test:** render `fixtures/repos/docli.json` with `--template cli-doc` (smoke or snapshot)

## Acceptance Criteria

- `docli templates list --json` includes `default` and `cli-doc`
- Full three-call flow in AGENT-PREVIEW.md (default + cli-doc + themed default) succeeds
- AUTHOR.md matches install layout and `theme_schema` rules
- No `c.1` / `c.4` / `compact` template references remain in `docs/templates/`

## Required Validation

- Phase B host gate — [README.md](README.md)
- `cargo test --test template_contract`
- `! rg -n 'c\\.(1|4)|compact' docs/templates/`

Three-call preview (from AGENT-PREVIEW.md; `MODEL=fixtures/contract/model.json` or equivalent):

```bash
docli generate --json --input "$MODEL" --preview --template default --theme '{"accent":"#007acc","font_body":"system-ui"}'
docli generate --json --input "$MODEL" --preview --template cli-doc --theme '{"accent":"#d73a49","font_body":"Monaco"}'
docli generate --json --input "$MODEL" --preview --template default --theme '{"accent":"#059669","font_body":"Inter"}'
```

Each command exits 0; parse `--json` stdout for `"ok":true` and `preview_dir`.
