# ADR-002 — Search index is embedded JSON

> **Canonical copy**: [`docs/architecture.md`](../../architecture.md#adr-002--search-index-is-embedded-json).
> `arch-qa` and `req-qa` use that file, not this sprint snapshot.

## Decision

The page keeps `<script id="docli-data" type="application/json">` as the full `CliModel`. Tree building and the detail panel read only that tag.

Search is a second tag, `<script id="docli-search" type="application/json">`, holding `SearchEntry` values from `search_index`. The page does not rebuild the index from `docli-data`. Rust tests lock `matching_anchors` without a browser, and the embedded JSON is part of the HTML byte lock.

`SearchEntry`, `search_index`, and `matching_anchors` live in `src/search.rs` and are re-exported from `src/lib.rs`. `src/render/html.rs` calls `search_index` and embeds the result. It does not define `SearchEntry`.

Later language implementations that must match `fixtures/contract/index.html` emit the same `docli-search` JSON for the same model. They do not substitute a client-side index with a different schema.
