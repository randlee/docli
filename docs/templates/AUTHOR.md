# Authoring docli HTML template packs

Template packs turn a **neutral [`CliModel`](../schema.md)** JSON document into one
self-contained HTML file (`index.html`). Bundled packs are embedded in the
`docli` binary (ADR-004). Custom packs can be added locally.

## Install layout

Bundled packs are compiled into the `docli` binary with `include_dir`
(ADR-004). Phase B embeds `default` and `cli-doc` from `templates/html/<id>/`.
`html::render` uses `default` when the caller does not pass `--template`.
`share/docli/templates/` holds **optional extra** packs only. It is not
required to render `default` or `cli-doc`, including after `cargo install`.

`templates/html/_skeleton/` is a starter (`REQ-DOCLI-HTML-009`). `docli
templates validate` accepts that directory. `docli templates list` does not
include an id that starts with `_`.

```text
templates/html/<id>/          # source of a bundled pack (embedded at build)
  template.toml               # manifest (required)
  page.html.j2                # shell (required)
  style.css.j2                # inlined into <style> (required)
  script.js                   # inlined into <script> (required)
  partials/                   # optional includes

templates/html/_skeleton/     # starter; not a bundled id

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
font_body = { type = "string", default = "system-ui, sans-serif", description = "Body font family" }
font_mono = { type = "string", default = "ui-monospace, monospace", description = "Monospace font family" }
bg = { type = "color", default = "#ffffff", description = "Page background" }
fg = { type = "color", default = "#1a1a2e", description = "Primary text" }
```

`theme_schema` drives `docli templates show` and documents keys for `--theme` JSON.
Every key is an object with `type`, `default`, and `description` (sprint **b.7**).

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
  --theme '{"accent":"#d73a49","font_body":"Monaco"}'

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
2. Set `id` to a name that does not start with `_`, then edit `theme_schema` and CSS variables in `style.css.j2`.
3. Run `docli templates validate` before committing.
4. Point `docli generate --template /path/to/pack` at CI or local preview.

Custom packs are **not** part of the embedded `default` or `cli-doc` render.
Optional extras live under `<install-root>/share/docli/templates/` (ADR-004).
`docli templates list` (Phase B **b.8**) reports bundled ids and those extras.
