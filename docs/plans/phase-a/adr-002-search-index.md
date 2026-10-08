# ADR-002 — Search index is embedded JSON

## Decision

The page keeps `<script id="docli-data" type="application/json">` as the full `CliModel`. Tree building and the detail panel read only that tag.

Search is a second tag, `<script id="docli-search" type="application/json">`, holding `SearchEntry` values from `search_index`. The page does not rebuild the index from `docli-data`. Rust tests lock `matching_anchors` without a browser, and the embedded JSON is part of the HTML byte lock.

Later language implementations that must match `fixtures/contract/index.html` emit the same `docli-search` JSON for the same model. They do not substitute a client-side index with a different schema.
