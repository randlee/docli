# First crates.io release

Sprint **b.11** makes `docli` `0.1.0` publish-ready. It does not upload the
crate. Run `cargo publish` only after `integrate/phase-b` has merged to
`develop`, from a clean `develop` checkout. Version policy is in
[CHANGELOG.md](../CHANGELOG.md): semver **0.x** until a stable API is declared.

## What the published crate ships

`Cargo.toml` `include` is the package file list. `cargo package --list` must
show these:

| Path | Why it ships |
|------|----------------|
| `README.md`, `LICENSE`, `CHANGELOG.md`, `Cargo.toml`, `Cargo.lock` | crates.io metadata and the locked build |
| `src/**` | library and binaries (`docli`, `cargo-docli`, `dump-cli-model`) |
| `templates/html/default/**` | `include_dir!("$CARGO_MANIFEST_DIR/templates/html/default")` |
| `templates/html/cli-doc/**` | `include_dir!("$CARGO_MANIFEST_DIR/templates/html/cli-doc")` |
| `templates/html/_skeleton/**` | author starter; validated from disk by tests; not embedded and not listed by `templates list` |
| `tests/**` | `cargo test` on the packaged sources, including `tests/error_contract.rs` |
| `fixtures/contract/**`, `fixtures/repos/**` | render, template, and repo fixture tests |
| `examples/sample.json` | sample neutral model |

`cargo install docli` builds the default binaries from those sources. The
embedded packs are inside the binary. The install does not read `default` or
`cli-doc` from disk. `gen-fixtures` is behind the `gen-fixtures` feature, so
a default install does not ship that binary.

## What stays dev-only

These stay in the git repo and are omitted from the `.crate`:

- `docs/` (requirements, architecture, plans, author and agent docs)
- `scripts/` (`verify-candidate-repos.sh` and the envelope helper)
- `site/` (generated HTML pages)
- `.github/`, `.claude/`, `.cursor/`, `.plan-hardening/`, `.triage/`

README links to those docs resolve on GitHub. They are not required to
compile or to run `cargo test --test error_contract` from the packaged crate.

## Pre-publish checks (this repo)

From the repo root:

```sh
cargo publish --dry-run
cargo package --list
```

`cargo package --list` must include `README.md`, `LICENSE`,
`templates/html/default/template.toml`, `templates/html/cli-doc/template.toml`,
`templates/html/_skeleton/template.toml`, and
`fixtures/contract/model.json`.

Install smoke uses a private prefix so it does not replace a `docli` already
on `PATH`. `cargo test` runs from this checkout, not from the installed
binary.

```sh
rm -rf /tmp/docli-install-smoke
cargo install --path . --locked --root /tmp/docli-install-smoke
/tmp/docli-install-smoke/bin/docli templates list --json
/tmp/docli-install-smoke/bin/docli generate \
  --input fixtures/contract/model.json --preview --json
cargo test --test error_contract
```

`templates list --json` must report `default` and `cli-doc`. The preview
command must exit 0 with `"ok": true` and a `preview_dir`.

Phase B host gate (also required on this branch):

```text
cargo test
cargo test --test error_contract
cargo test --test cli_contract
cargo test --test cargo_docli
cargo test --test template_contract
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
git diff --check
```

## After merge to `develop` (human)

```sh
cargo publish
cargo install docli --version 0.1.0
docli templates list --json
docli generate --input fixtures/contract/model.json --preview --json
cargo test --test error_contract
```

The last command still runs in a source checkout (git or an unpacked `.crate`).
`cargo install` only puts the binaries on `PATH`.

Point the candidate-repo script at that install:

```sh
DOCLI_BIN=docli DOCLI_SKIP_GEN_FIXTURES=1 ./scripts/verify-candidate-repos.sh
```

`DOCLI_BIN="$(command -v docli)"` is the same PATH lookup. Leave `DOCLI_BIN`
unset to keep the pre-publish default (`cargo build --release`).
