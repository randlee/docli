# docli

Language-agnostic CLI documentation generator. `docli` turns a neutral
command-tree description into a self-contained two-pane HTML reference and a
Markdown manual — mechanically, never hand-authored.

## Website

<https://randlee.github.io/docli/> — deployed via GitHub Pages from `main`
(`.github/workflows/pages.yml`); the self-generated CLI reference lives at
<https://randlee.github.io/docli/cli/>.

## Why

Every CLI framework knows its own command tree (clap in Rust, cobra in Go,
System.CommandLine in .NET). `docli` gives all of them one output: a
navigable website page and a distributable Markdown manual, regenerated on
every release from the same command definitions.

## CLI contract (mandatory)

The `docli` / `cargo docli` tools are **AI-first CLIs** and must follow the
**creating-ai-clis** skill you supplied for this repo (`.claude/skills/creating-ai-clis/`).
Hard requirements (`REQ-DOCLI-*`) and QA baselines:
[`docs/requirements.md`](docs/requirements.md). Architecture decisions and
`arch-qa` rules (`ADR-*`, `ARCH-RULE-*`):
[`docs/architecture.md`](docs/architecture.md).

Every `DOCLI.*` error code is integration-tested in `tests/error_contract.rs`
(`REQ-DOCLI-CLI-008`–`010`): one `--json` envelope on stdout, non-empty
`message` and `suggested_action`, and matching human stderr when not using
`--json`.

## Install

```sh
cargo install docli
```

## Usage

```sh
# Render both the HTML site page and the Markdown manual:
docli generate --input cli-model.json --html site/cli --markdown docs/manual/cli-reference.md

# Render HTML only (reads JSON from stdin):
docli generate --html site/cli < cli-model.json
```

With no `--html`, HTML is written to `site/cli/index.html` (default). With no `--markdown`, no Markdown file is written.

## Candidate repo verification

CI uses checked-in JSON under `fixtures/repos/` (see `tests/repo_fixtures.rs`).
To regenerate those models from live clap trees and run `generate` / `show` in
**docli**, **atm-core**, **sc-compose**, and **sc-observability** (contract
smoke only — no sc-observability CLI model):

```sh
./scripts/verify-candidate-repos.sh
```

The script runs `docli … --json` and requires `ok: true`. On failure it prints
the envelope `code`, `message`, and `suggested_action` (per **creating-ai-clis**
/ `REQ-DOCLI-CLI-003`), not a silent exit.

Set `DOCLI_ROOT` to override the docli checkout (default: repo root). Consumer
paths resolve in order: the env var, `../<repo>` next to docli, then
`~/Documents/github/<repo>`. Use `DOCLI_SKIP_GEN_FIXTURES=1` to skip
`gen-fixtures` and run `generate` / `show` against committed JSON only.

Override checkout paths with `ATM_CORE_ROOT`, `SC_COMPOSE_ROOT`, and
`SC_OBSERVABILITY_ROOT`. Optional Rust test:

```sh
DOCLI_LIVE_CANDIDATE_REPOS=1 \
  ATM_CORE_ROOT=~/Documents/github/atm-core \
  SC_COMPOSE_ROOT=~/Documents/github/sc-compose \
  SC_OBSERVABILITY_ROOT=~/Documents/github/sc-observability \
  cargo test --test live_candidate_repos -- --nocapture
```

## The neutral model

`docli` consumes one JSON document describing a CLI command tree. It carries
no framework-specific concepts, so any adapter can emit it:

```json
{
  "name": "demo",
  "version": "1.0.0",
  "description": "A demo CLI",
  "long_description": "Longer text...",
  "usage": "demo [OPTIONS] <COMMAND>",
  "epilogue": "Trailing notes...",
  "options": [
    {"name": "output", "long": "--output", "short": "-o", "value_name": "PATH",
     "help": "Write output to PATH", "required": false,
     "default_value": null, "choices": ["json", "yaml"]}
  ],
  "arguments": [
    {"name": "config", "help": "Config file", "required": false}
  ],
  "subcommands": [ "..." ]
}
```

Field reference:

- **command**: `name`, `version`, `description`, `long_description`,
  `epilogue`, `usage`, `options`, `arguments`, `subcommands`.
- **option**: `name`, `long`, `short`, `help`, `long_help`, `value_name`,
  `required`, `default_value`, `choices`, `min_values`, `max_values`.
- **argument**: `name`, `help`, `required`, `default_value`, `choices`.

## Language adapters

### Rust (clap) — built in

`docli::clap_model::from_clap` maps a live `clap::Command` onto the model:

```rust
use clap::{Parser, CommandFactory};

#[derive(Parser)]
#[command(name = "demo", version, about)]
struct Demo { /* ... */ }

fn main() {
    let model = docli::clap_model::from_clap(&Demo::command());
    println!("{}", serde_json::to_string(&model).unwrap());
}
```

Pipe that JSON into `docli generate` (or write a hidden dump subcommand) and
regenerate on every release. clap's `possible_values` maps to `choices`,
`value_names` to `value_name`, and `num_args` to `min_values`/`max_values`.

### Go (cobra) and .NET (System.CommandLine) — short term

Adapters that walk the framework's command tree and emit the same JSON. The
renderer does not care which language produced the input.

## Output

- `index.html` — a self-contained two-pane page: an indented, collapsible
  command tree (left) and a detail panel with description, usage, and an
  arguments/options table (right). Hash-routed per command, with search.
- Markdown — a flat reference for offline use, distributed with the installer.

## License

MIT
