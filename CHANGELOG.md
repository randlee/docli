# Changelog

`docli` is version `0.1.0`. Until `CHANGELOG.md` declares a stable API, versions
follow semver **0.x**: a minor bump (`0.2.0`) may change the CLI, the JSON
envelope, or the library surface. Pin an install with
`cargo install docli --version 0.1.0` when you need a fixed binary. `1.0.0` is
the stable-API release. It is not this version.

Uploading to crates.io is a maintainer step after `integrate/phase-b` merges
to `develop`. This file records what `0.1.0` contains. The commands to run
before and after that upload are in
[docs/release-first-crates-io.md](docs/release-first-crates-io.md).

## [0.1.0] — Unreleased

First crates.io release. The published crate embeds the HTML template packs.
`cargo install docli` is the primary install after that publish.

### Phase A

- Rust library plus the `docli` and `cargo docli` binaries.
- Neutral command-tree JSON, clap adapter (`docli::from_clap`), and
  deterministic HTML and Markdown renderers.
- `generate` / `show` with the version `"1"` JSON envelope.
- Contract fixtures under `fixtures/contract/`.

### Phase B

- Normative requirements and architecture, including the error contract.
- `tests/error_contract.rs` covers every `DOCLI.*` code (`REQ-DOCLI-CLI-008`–`010`).
- `scripts/verify-candidate-repos.sh` checks consumer repos. The default still
  builds `target/release/docli`. After publish, set `DOCLI_BIN` to the
  installed binary or to `docli` on `PATH`.
- Checked-in consumer fixtures and the fixture-policy modes
  (`DOCLI_SKIP_GEN_FIXTURES`, default drift restore, `DOCLI_REFRESH_FIXTURES`).
- Bundled HTML packs `default` and `cli-doc`, embedded with `include_dir` from
  `templates/html/`. `_skeleton` ships as an author starter and is not a
  listed pack id.
- `docli templates list|show|validate`.
- `docli generate --preview`, `--template`, and `--theme`.
- crates.io metadata and the package file list so a registry install compiles
  the bundled packs. See the release checklist for what ships and what stays
  in git only.
