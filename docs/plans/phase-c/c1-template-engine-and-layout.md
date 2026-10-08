---
id: c.1
title: template engine and default pack
status: planned
branch: feature/phase-c-c1-engine
worktree: ../docli-worktrees/feature/phase-c-c1-engine
target: integrate/phase-c
---

# Sprint c.1 — template engine and default pack

## Goal

- Move Phase A HTML from `src/render/html.rs` string constants into an installable
  **`default`** template pack; rendering stays deterministic for `fixtures/contract/`.

## Hard Dependencies

- Phase B merged (error contract)
- ADR-002 embedded search JSON unchanged for `default`

## Deliverables

- `templates/html/default/` — `template.toml`, `page.html.j2`, `style.css.j2`, `script.js`
- `src/render/html.rs` — load pack, render via template engine (MiniJinja or Askama; pick one in impl)
- `src/templates/mod.rs` — resolve install root: dev (`CARGO_MANIFEST_DIR`), install (`share/docli/templates`)
- `build.rs` or `include_dir` — embed bundled packs for `cargo install`
- ADR-004 in `docs/architecture.md` — template pack boundary (render bytes API vs ops)
- `REQ-DOCLI-HTML-007` — HTML via template pack; default pack reproduces contract bytes

## Acceptance Criteria

- `docli generate --input fixtures/contract/model.json --html <tmp> --template default`
  produces bytes matching `fixtures/contract/index.html` (or explicit lock update with review)
- No external CSS/JS URLs in output
- `theme_schema` in default manifest documents `:root` variables

## Required Validation

- `cargo test --test render_fixtures`
- Phase C CI gate (partial until c.2)
