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

Move Phase A HTML from `src/render/html.rs` into an installable **`default`** pack;
default rendering stays byte-identical to `fixtures/contract/index.html` **without** new CLI flags.

## Closes

- `REQ-DOCLI-HTML-007`
- `ADR-004` (template pack boundary in `docs/architecture.md`)

## Hard Dependencies

- b.1–b.2 on `integrate/phase-b`
- ADR-002 embedded search JSON unchanged for `default`

## Deliverables

- `templates/html/default/` — `template.toml`, `page.html.j2`, `style.css.j2`, `script.js`
- **MiniJinja** (fixed choice) for pack render; no Askama in Phase B
- `src/templates/mod.rs` — install root resolution (dev tree vs `share/docli/templates`)
- `src/render/html.rs` — calls template loader; no `--template` flag (b.9)
- `build.rs` or `include_dir` — embed bundled `default` for `cargo install`
- `REQ-DOCLI-HTML-007` body + section 11 index row (same PR)

## Explicit code samples

```rust
// src/templates/mod.rs
pub fn install_root() -> std::path::PathBuf;
pub fn load_pack(id: &str) -> Result<Pack, TemplateLoadError>;
pub fn render_default(model: &CliModel, theme: &ThemeMap) -> Result<Vec<u8>, RenderError>;
```

```toml
# templates/html/default/template.toml (minimal)
id = "default"
name = "docli default"
version = "1"
theme_schema = { accent = "#007acc", font_body = "system-ui" }
```

b.7 maps load/render failures to existing `DOCLI.IO` / `DOCLI.INTERNAL` only; **no** `DOCLI.TEMPLATE_*` codes until b.8.

## Acceptance Criteria

- `cargo test --test render_fixtures` passes; default pack output matches `fixtures/contract/index.html`
- `docli generate --input fixtures/contract/model.json --html <tmp>` (no `--template`) matches contract bytes
- No external CSS/JS URLs in output
- `theme_schema` documented in `template.toml`

## Required Validation

- Phase B host gate — [README.md](README.md)
- `cargo test --test render_fixtures`
