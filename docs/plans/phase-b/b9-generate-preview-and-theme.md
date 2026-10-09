---
id: b.9
title: generate preview, template, and theme
status: complete
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
- **`--preview`**: temp dir `docli-preview-{pid}-{nanos}-{seq}/index.html`; envelope includes `preview_dir`
- Invalid theme JSON → `DOCLI.INPUT_INVALID`
- `REQ-DOCLI-CLI-012` + section 11 index update
- Section 9: **`DOCLI.TEMPLATE_NOT_FOUND`** — kind **`not_found`**, exit **3**, details `{ "template": "<id|path>" }`; **Covered by** named test(s)
- `ErrorBody` / `ErrorCode` in `src/contract.rs`
- b.9 updates b.8 `example_generate_argv` to include `--preview`, `--template`, `--theme` once flags exist
- `tests/template_contract.rs` — three `--preview` runs with **default** + different `--theme` only
- `tests/error_contract.rs` — named test for `DOCLI.TEMPLATE_NOT_FOUND`

## Explicit code samples

```rust
pub struct GenerateRequest {
    pub input: InputSource, // unchanged — stdin + file
    pub html_dir: Option<PathBuf>,
    pub preview: bool,
    pub template: Option<TemplateRef>,
    pub theme: Option<ThemeMap>, // CLI parses --theme JSON before ops
    // … existing markdown/json fields
}
```

When `--preview`: write only under temp dir; set **`data.html_dir`** and **`data.preview_dir`** to that dir; do not touch `site/cli`. When not preview: `preview_dir` is null; `html_dir` is resolved output (default `site/cli`).

## Acceptance Criteria

- Three `--preview` runs (default template, distinct themes) → three dirs, three success envelopes
- `--preview` + `--html` → USAGE
- Unknown template id → `DOCLI.TEMPLATE_NOT_FOUND`; section 9 **Covered by** matches test name
- Default template + no theme still matches b.7 contract bytes when writing to explicit `--html`

## Required Validation

- Phase B host gate — [README.md](README.md)
- `cargo test --test template_contract`
- `cargo test --test error_contract` (TEMPLATE_NOT_FOUND)
