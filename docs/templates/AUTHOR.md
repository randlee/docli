# Authoring docli HTML template packs

Template packs turn a **neutral [`CliModel`](../schema.md)** JSON document into one
self-contained HTML file (`index.html`). Packs ship **with docli** under the install
root and can be added locally for custom branding.

## Install layout

After `cargo install docli` (Phase B **b.11** publish; packs land in **b.7–b.10**), bundled packs live at:

```text
<install-root>/share/docli/templates/<id>/
  template.toml          # manifest (required)
  page.html.j2           # shell (required)
  style.css.j2           # inlined into <style> (required)
  script.js              # inlined into <script> (required)
  partials/              # optional includes
```

Development builds resolve templates from `<repo>/templates/html/<id>/` via
`CARGO_MANIFEST_DIR`.

## Manifest (`template.toml`)

```toml
id = "my-brand"
name = "My product CLI reference"
version = "1"
description = "Acme Corp colors and Inter typography"
min_docli = "0.2.0"

[theme_schema]
accent = { type = "color", default = "#007acc", description = "Primary accent" }
font_body = { type = "string", default = "system-ui, sans-serif" }
font_mono = { type = "string", default = "ui-monospace, monospace" }
bg = { type = "color", default = "#ffffff" }
fg = { type = "color", default = "#1a1a2e" }
```

`theme_schema` drives `docli templates show` and documents keys for `--theme` JSON.

## Render context

Templates receive (MiniJinja — Phase B **b.7**):

| Variable | Content |
|----------|---------|
| `model` | `CliModel` as JSON-serializable struct |
| `model_json` | embedded `#docli-data` string |
| `search_json` | embedded `#docli-search` string (ADR-002) |
| `theme` | merged defaults + `--theme` overrides |
| `title` | `model.name` |
| `generator` | `docli {version}` |

**Contract:** output must remain **one HTML file**, inline CSS and JS only (no
`href=` / `src=` to external `.css` or `.js`). Hash-routed command anchors must
stay compatible with `search_index` / `matching_anchors` tests unless the pack
declares a different search mode in manifest (advanced; default packs must not).

## Validate a pack

```bash
docli templates validate ./templates/html/my-brand --json
docli generate --input fixtures/contract/model.json --html /tmp/out \
  --template ./templates/html/my-brand --theme '{"accent":"#ff0"}' --json
```

## Agent: compare all bundled templates

Discover ids, then preview each (user opens paths):

```bash
docli templates list --json
MODEL=path/to/cli-model.json

docli generate --json --input "$MODEL" --preview --template default \
  --theme '{"accent":"#007acc","font_body":"system-ui"}'

docli generate --json --input "$MODEL" --preview --template cli-doc \
  --theme '{"accent":"#007acc","font_body":"Monaco"}'

docli generate --json --input "$MODEL" --preview --template default \
  --theme '{"accent":"#059669","font_body":"Inter"}'
```

Read `data.preview_dir` and `outputs[0].path` from each envelope; return those paths
to the user. Do not discard `--json` stdout on failure — parse `error.code` and
`suggested_action`.

## cli-doc style reference

The **`cli-doc`** pack reimplements layout cues from
[spirali/cli_doc](https://github.com/spirali/cli_doc) (two columns, card options,
search navigation). docli still consumes **CliModel JSON**, not `--help` scraping.

## Publishing a custom pack

1. Copy `templates/html/_skeleton/` (Phase B **b.10**) to your repo or site repo.
2. Edit `theme_schema` and CSS variables in `style.css.j2`.
3. Run `docli templates validate` before committing.
4. Point `docli generate --template /path/to/pack` at CI or local preview.

Custom packs are **not** registered in `templates list` unless installed under
`<install-root>/share/docli/templates/` (packaging decision for Phase B **b.7** / **b.11**).
