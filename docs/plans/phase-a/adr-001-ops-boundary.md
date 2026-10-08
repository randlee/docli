# ADR-001 — Operations are the library boundary

## Decision

`ops::generate` and `ops::show` are the stable Rust API. Their request and response structs live in `src/ops.rs`. `src/main.rs` and `src/bin/cargo-docli.rs` only convert argv into those requests and print the envelope.

A later MCP wrapper calls `ops::generate` and `ops::show` directly. It does not re-parse CLI flags and it does not reshape the response JSON. The envelope serialized by the CLI is `serde_json` of the same `Envelope<T>` the library returns. The error contract — codes, `details` shape, and `suggested_action` — is specified in [a1-cli-contract.md](a1-cli-contract.md). An MCP caller surfaces an `ok: false` envelope to its own caller without flattening or dropping fields.

`pub mod render` stays public. `html::render` and `markdown::render` stay `pub fn render(model: &CliModel) -> String`. That is a second stable API: it returns document bytes and does not write files, hash them, or build an envelope. Callers who need paths, hashes, and errors use `ops`. `ops` calls those functions. a.2 may change the HTML body those functions return. It does not change the signature.
