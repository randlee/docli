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

- One `generate` call: JSON in → self-contained HTML in **temp or explicit dir**,
  with chosen template and branding variables — optimized for **3-call agent previews**.

## Hard Dependencies

- b.7 render with template id + theme map
- b.8 — `templates list` for discovery

## Deliverables

- `GenerateRequest` extended: `template_id`, `theme_json`, `preview: bool`
- CLI flags:

```text
docli generate \
  --input cli-model.json \
  [--html DIR] \
  [--preview] \
  [--template ID|PATH] \
  [--theme JSON] \
  [--json]
```

- **`--preview`**: allocate `std::env::temp_dir()/docli-preview-{pid}-{nanos}/`,
  write `index.html`, return `preview_dir` in envelope (and `outputs` as today).
  Mutually exclusive with `--html` unless documented otherwise; if both absent and
  not preview, keep Phase A default `site/cli`.
- **`--theme`**: JSON object inlined into template (e.g. `accent`, `font_body`,
  `font_mono`, `bg`, `fg`). Invalid JSON → `DOCLI.INPUT_INVALID`.
- Error codes: `DOCLI.TEMPLATE_NOT_FOUND`, `DOCLI.TEMPLATE_INVALID` in requirements + tests
- `tests/error_contract.rs` extended for new codes
- `tests/template_contract.rs` — three-preview scenario (same input, three templates/themes)
- `REQ-DOCLI-CLI-012` in `docs/requirements.md`

## Explicit envelope addition

Success `generate` `data` adds optional fields:

```json
{
  "operation": "generate",
  "template": "default",
  "theme": { "accent": "#d73a49" },
  "preview_dir": "/tmp/docli-preview-…",
  "html_dir": "/tmp/docli-preview-…",
  "outputs": []
}
```

When not `--preview`, `preview_dir` is JSON `null`.

## Acceptance Criteria

- Three sequential `--preview` runs with different `--template` and `--theme`
  produce three distinct directories and three success envelopes
- Each `outputs[0].path` ends with `index.html` and file exists
- `--json` failures remain single-envelope on stdout
- Default `--template default` without theme matches b.7 byte lock for contract fixture

## Required Validation

- Phase B host gate in [README.md](README.md)
- `cargo test --test template_contract`
- Commands in [`docs/templates/AGENT-PREVIEW.md`](../../templates/AGENT-PREVIEW.md) succeed on maintainer machine
