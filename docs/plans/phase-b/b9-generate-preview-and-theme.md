---
id: b.9
title: generate preview, template, and theme
status: planned
branch: feature/phase-b-b9-preview
worktree: ../docli-worktrees/feature/phase-b-b9-preview
target: integrate/phase-b
---

# Sprint b.9 — `--template`, `--theme`, `--preview`

## Goal

Extend **`docli generate`** with template selection, theme JSON, and temp preview dirs (agent multi-preview with **default** pack only until b.10 adds `cli-doc`).

## Closes

- `REQ-DOCLI-CLI-012`
- `DOCLI.TEMPLATE_NOT_FOUND` (requirements + `error_contract`)

## Hard Dependencies

- b.7–b.8

## Deliverables

- CLI flags: `--template ID|PATH`, `--theme JSON`, `--preview`
- **`--preview` and `--html` are mutually exclusive** — both set → `DOCLI.USAGE`
- Neither `--preview` nor `--html` → Phase A default `site/cli`
- **`--preview`**: temp dir `docli-preview-{pid}-{nanos}/index.html`; envelope includes `preview_dir`
- Invalid theme JSON → `DOCLI.INPUT_INVALID`
- `REQ-DOCLI-CLI-012` + section 11 index update
- [`docs/requirements.md`](../../requirements.md) section 9 row for **`DOCLI.TEMPLATE_NOT_FOUND`** with **Covered by** test name in `tests/error_contract.rs`
- `tests/template_contract.rs` — three `--preview` runs with **default** + different `--theme` only
- `tests/error_contract.rs` — named test for `DOCLI.TEMPLATE_NOT_FOUND`

## Explicit code samples

```rust
pub struct GenerateRequest {
    pub input_path: Option<PathBuf>,
    pub html_dir: Option<PathBuf>,
    pub preview: bool,
    pub template_id: Option<String>,
    pub theme_json: Option<String>,
    // … existing markdown/json fields
}
```

Success `generate` `data` adds: `template`, `theme`, `preview_dir` (null when not preview), `html_dir`, `outputs`.

## Acceptance Criteria

- Three `--preview` runs (default template, distinct themes) → three dirs, three success envelopes
- `--preview` + `--html` → USAGE
- Unknown template id → `DOCLI.TEMPLATE_NOT_FOUND`; section 9 **Covered by** matches test name
- Default template + no theme still matches b.7 contract bytes when writing to explicit `--html`

## Required Validation

- Phase B host gate — [README.md](README.md)
- `cargo test --test template_contract`
- `cargo test --test error_contract` (TEMPLATE_NOT_FOUND)
