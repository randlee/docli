# Authoring docli HTML template packs

Template packs turn a **neutral [`CliModel`](../schema.md)** JSON document into one
self-contained HTML file (`index.html`). Packs ship **with docli** under the install
root and can be added locally for custom branding.

## Install layout

Bundled packs are compiled into the `docli` binary with `include_dir`
(ADR-004). `default` is embedded from `templates/html/default/` and is the
pack `html::render` uses when the caller does not pass `--template`.
`share/docli/templates/` holds **optional extra** packs only. It is not
required to render `default`, including after `cargo install`.

```text
templates/html/<id>/          # source of a bundled pack (embedded at build)
  template.toml               # manifest (required)
  page.html.j2                # shell (required)
  style.css.j2                # inlined into <style> (required)
  script.js                   # inlined into <script> (required)
  partials/                   # optional includes

<install-root>/share/docli/templates/<id>/   # optional extra packs only
```

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

Custom packs are **not** part of the embedded `default` render. Optional extras
live under `<install-root>/share/docli/templates/` (ADR-004). `templates list`
registration is Phase B **b.8**.
