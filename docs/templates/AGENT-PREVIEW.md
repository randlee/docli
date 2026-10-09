# Agent workflow: preview CLI documentation templates

Use this when the user wants to **see what their CLI docs would look like** before
choosing a template or brand colors. Requires Phase B **b.8–b.9** (`docli templates`,
`--preview`, `--template`, `--theme`).

## Preconditions

- `cli-model.json` exists (from `docli::from_clap`, `dump-cli-model`, or hand-written)
- `docli` on `PATH` (or `target/release/docli`)
- Phase B template sprints shipped (`default` and `cli-doc` embedded)

## Step 1 — Discover templates

```bash
docli templates list --json
```

Confirm `data.templates[].id` includes `default` and `cli-doc`. `_skeleton` is not listed.

## Step 2 — Render three previews

Use the **same** `--input` and vary `--template` and `--theme`. Prefer `--preview`
so output goes to a temp directory without touching `site/cli`.

```bash
MODEL=/absolute/path/to/cli-model.json

docli generate --json --input "$MODEL" --preview --template default \
  --theme '{"accent":"#007acc","font_body":"system-ui"}'

docli generate --json --input "$MODEL" --preview --template cli-doc \
  --theme '{"accent":"#d73a49","font_body":"Monaco"}'

docli generate --json --input "$MODEL" --preview --template default \
  --theme '{"accent":"#059669","font_body":"Inter"}'
```

Third call reuses `default` with a different theme so the user can compare **brand**
without changing layout.

## Step 3 — Report to the user

Label each preview from that call's `--template` and `--theme` arguments.
Success `data` follows `REQ-DOCLI-CLI-005`: `operation`, `input`, `model_name`,
`html_dir`, `outputs`, and `preview_dir` when `--preview` is set.

| Source | Use |
|--------|-----|
| this call's `--template` | label in UI |
| this call's `--theme` | what was applied |
| `data.outputs[0].path` | open this file in a browser |
| `data.preview_dir` | directory containing `index.html` |

Example message:

```text
Three previews are ready:
1. default / blue — file:///…/docli-preview-…/index.html
2. cli-doc / monospace — file:///…/docli-preview-…/index.html
3. default / green Inter — file:///…/docli-preview-…/index.html
```

## Step 4 — On failure

Parse stdout JSON only. Report `error.code`, `error.message`, and
`error.suggested_action`. Common codes:

| Code | Fix |
|------|-----|
| `DOCLI.INPUT_NOT_FOUND` | Correct `--input` path |
| `DOCLI.TEMPLATE_NOT_FOUND` | Run `templates list`; fix `--template` id |
| `DOCLI.TEMPLATE_INVALID` | Run `templates validate <path>` |
| `DOCLI.INPUT_INVALID` | Fix `--theme` JSON |

## Optional: single temp folder tree

```bash
BASE=$(mktemp -d)
docli generate --json --input "$MODEL" --html "$BASE/default" --template default --theme '{...}'
docli generate --json --input "$MODEL" --html "$BASE/cli-doc" --template cli-doc --theme '{...}'
docli generate --json --input "$MODEL" --html "$BASE/green" --template default --theme '{...}'
```

Then list `$BASE/*/index.html` for the user.
