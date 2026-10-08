# Architecture

`ops` is the library boundary. In-process callers use `docli::ops::generate`
and `docli::ops::show` and receive the same version `"1"` envelope the CLI
prints. `src/main.rs` only converts argv into those requests. A later MCP
wrapper calls the same functions and does not re-parse flags or reshape the
JSON.

`render` stays a public byte API. `html::render` and `markdown::render` are
`pub fn render(model: &CliModel) -> String`. They return document bytes and do
not write files, hash them, or build an envelope. Callers who need paths,
hashes, and errors use `ops`. `ops` calls those functions.

Search will live in `src/search.rs` in sprint a.2. That module will own
`SearchEntry`, `search_index`, and `matching_anchors`. The HTML renderer will
embed the index; it will not define those types.

These decisions are recorded in:

- [ADR-001 — Operations are the library boundary](plans/phase-a/adr-001-ops-boundary.md)
- [ADR-002 — Search index is embedded JSON](plans/phase-a/adr-002-search-index.md)
