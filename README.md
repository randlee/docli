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
