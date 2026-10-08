---
id: b.7
title: template engine and default pack
status: planned
branch: feature/phase-b-b7-engine
worktree: ../docli-worktrees/feature/phase-b-b7-engine
target: integrate/phase-b
---

# Sprint b.7 — template engine and default pack

## Goal

- Move Phase A HTML from `src/render/html.rs` string constants into an installable
  **`default`** template pack; rendering stays deterministic for `fixtures/contract/`.

## Hard Dependencies

- b.1–b.2 on `integrate/phase-b` (normative baselines, `REQ-DOCLI-CLI-008`–`010`)
- ADR-002 embedded search JSON unchanged for `default`

## Deliverables

- `templates/html/default/` — `template.toml`, `page.html.j2`, `style.css.j2`, `script.js`
- `src/render/html.rs` — load pack, render via template engine (MiniJinja or Askama; pick one in impl)
- `src/templates/mod.rs` — resolve install root: dev (`CARGO_MANIFEST_DIR`), install (`share/docli/templates`)
- `build.rs` or `include_dir` — embed bundled packs for `cargo install`
- ADR-004 in `docs/architecture.md` — template pack boundary (render bytes API vs ops)
- `REQ-DOCLI-HTML-007` in `docs/requirements.md` — HTML via template pack; default reproduces contract bytes

## Acceptance Criteria

- `docli generate --input fixtures/contract/model.json --html <tmp> --template default`
  produces bytes matching `fixtures/contract/index.html` (or explicit lock update with review)
- No external CSS/JS URLs in output
- `theme_schema` in default manifest documents `:root` variables

## Required Validation

- Phase B host gate in [README.md](README.md)
- `cargo test --test render_fixtures`
