# Authoring docli HTML template packs

Template packs turn a **neutral [`CliModel`](../schema.md)** JSON document into one
self-contained HTML file (`index.html`). Phase B ships **`default`** and
**`cli-doc`** as **`include_dir` embeds** compiled into the `docli` binary
(ADR-004). Optional extras may live under `<install-root>/share/docli/templates/`;
that directory is not required to render bundled packs after `cargo install`.

## Install layout

Bundled ids **`default`** and **`cli-doc`** are embedded from
`templates/html/default/` and `templates/html/cli-doc/`. `html::render` uses
`default` when the caller omits `--template`. The author starter
`templates/html/_skeleton/` is **on disk only** — copy it to your repo,
customize, and point `docli generate --template` at the copy. `templates list`
does not register `_skeleton` until you install a copy under
`share/docli/templates/`.

```text
templates/html/default/       # bundled default (embedded at build)
templates/html/cli-doc/       # bundled cli-doc (embedded at build)
templates/html/_skeleton/     # disk-only starter (not embedded, not listed)

templates/html/<id>/          # layout of any pack directory
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
bg = { type = "color", default = "#fff", description = "Page background" }
fg = { type = "color", default = "#1a1a2e", description = "Primary text" }
border = { type = "color", default = "#ddd", description = "Hairline borders" }
hover = { type = "color", default = "#f0f0f0", description = "Hover and table-header fill" }
accent = { type = "color", default = "#007acc", description = "Primary accent" }
muted = { type = "color", default = "#666", description = "Secondary text" }
font_body = { type = "string", default = "16px/1.5 system-ui,-apple-system,Segoe UI,sans-serif", description = "Body font shorthand, including size and line height" }
bg_dark = { type = "color", default = "#16181d", description = "Dark-scheme background" }
fg_dark = { type = "color", default = "#e6e6e6", description = "Dark-scheme text" }
border_dark = { type = "color", default = "#333", description = "Dark-scheme borders" }
hover_dark = { type = "color", default = "#23262e", description = "Dark-scheme hover fill" }
accent_dark = { type = "color", default = "#4da3ff", description = "Dark-scheme accent" }
muted_dark = { type = "color", default = "#999", description = "Dark-scheme secondary text" }
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
2. Edit `theme_schema` and CSS variables in `style.css.j2`.
3. Run `docli templates validate` before committing.
4. Point `docli generate --template /path/to/pack` at CI or local preview.

Custom packs are **not** part of the embedded `default` render. Optional extras
live under `<install-root>/share/docli/templates/` (ADR-004). `templates list`
registration is Phase B **b.8**.
