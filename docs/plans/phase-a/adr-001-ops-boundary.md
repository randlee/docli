# ADR-001 — Operations are the library boundary

## Decision

`ops::generate` and `ops::show` are the stable Rust API. Their request and response structs live in `src/ops.rs`. `src/main.rs` and `src/bin/cargo-docli.rs` only convert argv into those requests and print the envelope.

A later MCP wrapper calls `ops::generate` and `ops::show` directly. It does not re-parse CLI flags and it does not reshape the response JSON. The envelope serialized by the CLI is `serde_json` of the same `Envelope<T>` the library returns.

`html::render` and `markdown::render` stay `pub fn render(model: &CliModel) -> String`. `ops` calls those functions. a.2 may change the HTML body those functions return. It does not change the signature.
