---
id: a.2
title: HTML tree behavior and renderer byte lock
status: planned
branch: feature/phase-a-a2-html
worktree: ../docli-worktrees/feature/phase-a-a2-html
target: develop
---

# Sprint a.2 — HTML tree behavior and renderer byte lock

## Goal

- The generated page meets the HTML interaction rules, and `fixtures/contract/` freezes the HTML and Markdown bytes.

## Hard Dependencies

- a.1 `generate` writes the page
- [adr-002-search-index.md](adr-002-search-index.md)
- `pub fn render(model: &CliModel) -> String` stays the signature of both renderers. This sprint does not add parameters. The search index is built inside `html::render`.

## Deliverables

- `src/render/html.rs` — caret and command name are sibling controls; search uses the embedded index
- `src/render/markdown.rs` — one section per command with description, usage, arguments, and options
- `fixtures/contract/model.json`, `fixtures/contract/index.html`, `fixtures/contract/manual.md`
- `tests/render_fixtures.rs` — byte compare of both renderers against those fixtures

## Explicit Code Samples

Anchor for a command is the slash-free slug of its path: join `name` from the root with a single space, lowercase, replace every run of non-alphanumeric characters with one `-`, and trim `-` from both ends. Root `demo` is `demo`. Child `run` is `demo-run`.

```rust
pub struct SearchEntry {
    pub anchor: String,
    pub ancestors: Vec<String>,
    pub terms: Vec<String>,
}

pub fn search_index(model: &CliModel) -> Vec<SearchEntry>;

pub fn matching_anchors(index: &[SearchEntry], query: &str) -> std::collections::BTreeSet<String>;
```

`terms` include the command name, each option `name`, `long`, and `short`, and each argument `name`. `matching_anchors` returns the hit anchor and every ancestor anchor. Comparison is case-insensitive substring.

The page contains both script tags. `#docli-data` is the `CliModel` JSON and is what builds the tree and the detail panel. `#docli-search` is the `SearchEntry` array and is the only input to search filtering. Search does not walk `#docli-data`.

```html
<li class="node open">
  <button type="button" class="docli-caret" aria-expanded="true"></button>
  <button type="button" class="docli-cmd">demo</button>
  <ul class="children"></ul>
</li>
```

`.docli-caret` and `.docli-cmd` are siblings under `.node`. Nodes with children start with class `open` and `aria-expanded="true"`. Activating `.docli-caret` toggles only that node. Activating `.docli-cmd` selects the command and does not change `open`.

## Out of Scope

- A headed browser or default-app launch
- Rewriting `src/schema.rs`
- Test-repo command trees
- Changing `render`'s function signature

## Acceptance Criteria

- `matching_anchors` for query `run` on a model whose root name is `demo` and whose child name is `run` contains `demo` and `demo-run`
- `matching_anchors` for query `--verbose` contains the anchor of the command that owns that option and that command's ancestors
- A query that matches nothing returns an empty set
- Generated HTML for each command contains `.docli-caret` and `.docli-cmd` as siblings under the same `.node`
- Generated HTML contains `id="docli-data"` and `id="docli-search"`
- `fixtures/contract/index.html` and `fixtures/contract/manual.md` equal `render` output for `fixtures/contract/model.json`

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test render_fixtures`
- `cargo test search_index`
