---
id: b.7
title: template engine and default pack
status: complete
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
- Delivers **ADR-004** (template pack boundary in `docs/architecture.md`)

## Hard Dependencies

- b.1–b.2 on `integrate/phase-b`
- ADR-002 embedded search JSON unchanged for `default`

## Deliverables

- `templates/html/default/` — `template.toml`, `page.html.j2`, `style.css.j2`, `script.js`
- **MiniJinja** (fixed choice) for pack render; no Askama in Phase B
- **ADR-004** in `docs/architecture.md` (amends ADR-001): bundled packs **embedded in the binary** via `include_dir`; `share/docli/templates` holds **optional extra** packs only. `html::render(&CliModel) -> String` uses embedded **default** + default theme (no `--template`). Only `ops::generate` applies caller template/theme and returns envelopes.
- `src/templates/mod.rs` — `resolve_pack` for bundled id or filesystem directory
- `src/render/html.rs` — pack render only; delete Phase A inline CSS/JS body
- `build.rs` or `include_dir` — embed bundled `default` for `cargo install`
- `REQ-DOCLI-HTML-007` body + section 11 index row (same PR)

## Explicit code samples

```rust
// src/templates/mod.rs — resolve bundled id OR filesystem pack directory
pub fn resolve_pack(template: &TemplateRef) -> Result<Pack, PackResolveError>;
pub fn render_pack(pack: &Pack, model: &CliModel, theme: &ThemeMap) -> Result<String, RenderError>;
```

`theme_schema` in every `template.toml` uses the AUTHOR.md object form (`type`, `default`, `description` per key).

b.7 embedded-default path: I/O failures → `DOCLI.IO` / `DOCLI.INTERNAL` only. Caller-selected packs use `DOCLI.TEMPLATE_*` from b.8+.

## Acceptance Criteria

- `rg` does not find Phase A inline `const CSS:` / `const JS:` in `src/render/html.rs` (deleted)
- Output includes `id="docli-default-pack"` (or equivalent single marker documented in the sprint PR) present only in `templates/html/default/`
- `cargo test --test render_fixtures` and generate without `--template` match `fixtures/contract/index.html`
- No external CSS/JS URLs in output
- `theme_schema` documented in `template.toml`

## Required Validation

- Phase B host gate — [README.md](README.md)
- `cargo test --test render_fixtures`
