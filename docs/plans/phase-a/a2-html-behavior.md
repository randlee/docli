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

- a.1 `generate` writes the page the tests inspect

## Deliverables

- `src/render/html.rs` — caret control separate from the command-name control; search uses a Rust-built index
- `src/render/markdown.rs` — unchanged contract: one section per command with description, usage, arguments, and options
- `fixtures/contract/model.json`, `fixtures/contract/index.html`, `fixtures/contract/manual.md`
- `tests/render_fixtures.rs` — byte compare of both renderers against those fixtures

## Explicit Code Samples

```rust
pub struct SearchEntry {
    pub anchor: String,
    pub ancestors: Vec<String>,
    pub terms: Vec<String>,
}

pub fn search_index(model: &CliModel) -> Vec<SearchEntry>;

pub fn matching_anchors(index: &[SearchEntry], query: &str) -> std::collections::BTreeSet<String>;
```

`terms` include the command name, each option `name`, `long`, and `short`, and each argument `name`. `matching_anchors` also returns every ancestor of a hit. The page embeds the index as `<script id="docli-search" type="application/json">` and filters from that index.

Initial tree state: nodes with children are expanded and the caret shows open. Activating the caret toggles only that node. Activating the command name selects it and does not change expanded state.

## Out of Scope

- A headed browser or default-app launch
- Rewriting the neutral model in `src/schema.rs`
- Test-repo command trees

## Acceptance Criteria

- `matching_anchors` for query `run` on a model whose root is `demo` and whose child is `run` contains both anchors
- `matching_anchors` for query `--verbose` contains the command anchor that owns that option and that command's ancestors
- A query that matches nothing returns an empty set
- Generated HTML has a caret element and a command-name element that are not the same node
- `fixtures/contract/index.html` and `fixtures/contract/manual.md` equal `render` output for `fixtures/contract/model.json`

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test render_fixtures`
- `cargo test search_index`
