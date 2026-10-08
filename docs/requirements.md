# docli Requirements

**Status**: Approved baseline (skeleton built and committed 2026-10-07)
**Applies to**: `docli`
**Source of truth**: this document and `src/schema.rs`

## 1. Purpose And Scope

`docli` is a language-agnostic CLI documentation generator. It converts a
neutral command-tree description into a self-contained two-pane HTML reference
and a Markdown manual, mechanically and deterministically — never
hand-authored.

`docli` is explicitly not:

- a parser of rendered `--help` text (extraction is the caller's job)
- a site framework or theme library
- tied to any single CLI framework

## 2. Product Definition

- `REQ-DOCLI-PRODUCT-001`: `docli` is a standalone Rust command-line tool
  distributed on crates.io and installable via `cargo install docli`.
- `REQ-DOCLI-PRODUCT-002`: `docli` integrates with cargo and runs as a build
  tool in CI/release pipelines (invoked from `just`/workflows, not a service).

## 3. Neutral Input Model

The input is one JSON document describing a CLI command tree, carrying no
framework-specific concepts.

- `REQ-DOCLI-INPUT-001`: a command carries `name`, `version`, `description`,
  `long_description`, `epilogue`, `usage`, `options`, `arguments`,
  `subcommands`.
- `REQ-DOCLI-INPUT-002`: an option carries `name`, `long`, `short`, `help`,
  `long_help`, `value_name`, `required`, `default_value`, `choices`,
  `min_values`, `max_values`.
- `REQ-DOCLI-INPUT-003`: an argument carries `name`, `help`, `required`,
  `default_value`, `choices`.
- `REQ-DOCLI-INPUT-004`: input is read from a file path or stdin (`-`).

## 4. Language Adapters

- `REQ-DOCLI-RUST-001`: Rust is supported out of the box via
  `docli::from_clap(&clap::Command)`. clap's `possible_values` maps to
  `choices`, `value_names` to `value_name`, and `num_args` to
  `min_values`/`max_values`.
- `REQ-DOCLI-RUST-002`: the adapter excludes auto-injected `--help`/`--version`
  and preserves the full command path in `usage`.
- `REQ-DOCLI-GO-001` (short term): a Go adapter emits the same JSON from a
  cobra command tree.
- `REQ-DOCLI-NET-001` (short term): a .NET adapter emits the same JSON from
  System.CommandLine.

## 5. HTML Output

- `REQ-DOCLI-HTML-001`: output is a self-contained page (single file, inline
  CSS and JS, embedded data), hash-routed per command with readable slug
  anchors.
- `REQ-DOCLI-HTML-002`: layout is two-pane — an indented, collapsible command
  tree on the left and a detail panel on the right.
- `REQ-DOCLI-HTML-003`: the tree has indentation by depth, a selected
  highlight, a hover state, and expand/collapse carets (cli_doc-style).
- `REQ-DOCLI-HTML-004`: the detail panel shows description, usage, and an
  arguments/options table (name/short/value/required/default/choices/
  description).
- `REQ-DOCLI-HTML-005`: a search box filters commands and options.

## 6. Markdown Output

- `REQ-DOCLI-MD-001`: a flat Markdown reference with one section per command,
  including description, usage, arguments, and options.
- `REQ-DOCLI-MD-002`: the Markdown manual is distributed with the installer
  for offline use.

## 7. Generation Guarantees

- `REQ-DOCLI-GEN-001`: output is deterministic and mechanically generated from
  the command model — no hand-authored reference content.
- `REQ-DOCLI-GEN-002`: consumers regenerate both outputs on every release; CI
  rejects missing or stale outputs.

## 8. Distribution

- `REQ-DOCLI-DIST-001`: licensed MIT.
