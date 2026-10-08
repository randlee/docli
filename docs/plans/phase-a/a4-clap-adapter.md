---
id: a.4
title: clap adapter mapping
status: complete
branch: feature/phase-a-a4-clap
worktree: ../docli-worktrees/feature/phase-a-a4-clap
target: develop
---

# Sprint a.4 — clap adapter mapping

## Goal

- `docli::from_clap` fills the neutral model from a clap 4.5 `Command`.

## Hard Dependencies

- `src/clap_model.rs` and `src/schema.rs`
- a.4 tests do not read `fixtures/contract/`. If a mapping change alters rendered bytes, re-run a.2's fixture test before merge. That re-run is not an a.4 acceptance criterion.

## Deliverables

- `src/clap_model.rs` — clap 4 mapping below
- `src/lib.rs` — `pub use clap_model::from_clap;`
- `tests/clap_adapter.rs` — the acceptance checks in this sprint

## Explicit Code Samples

```rust
pub fn from_clap(command: &clap::Command) -> CliModel;
```

clap 4.5 only. Tests build args with `Arg::new` and `value_parser`, then read them back through `from_clap`. The adapter reads `Arg::get_possible_values()`, which returns `Vec<PossibleValue>`. It does not call the clap 3 method `possible_values()`.

| clap 4 read | model field |
|---|---|
| `get_possible_values()` | `choices` on the option or argument |
| first `get_value_names()` entry | `value_name` |
| `get_num_args()` | `min_values`, `max_values` |
| `get_long_help()` | `long_help` |
| `get_long_about()` | `long_description` |
| `get_after_help()` | `epilogue` |
| arg id `help` or `version` | excluded |
| subcommand usage | contains the parent path (`demo run`) |

## Out of Scope

- A second CLI framework
- Rendering HTML inside `from_clap`
- Reading command types from atm-core, sc-compose, or sc-observability

## Acceptance Criteria

- `Arg::new("format").value_parser(["json", "yaml"])` yields `choices` `["json", "yaml"]`
- `Arg::new("n").num_args(2..=3)` yields `min_values == Some(2)` and `max_values == Some(3)`
- `Arg::new("output").value_name("PATH")` yields `value_name == Some("PATH")`
- `Arg::new("verbose").long_help("more detail")` yields `long_help == "more detail"`
- A positional `Arg` with `value_parser(["a", "b"])` yields argument `choices` of those two strings
- `Command::new("demo").long_about("long").after_help("bye")` yields `long_description == "long"` and `epilogue == "bye"`
- The model has no option or argument named `help` or `version` when those ids are clap's built-in flags
- Subcommand `run` under `demo` has `usage` containing `demo run`
- `docli::from_clap(&clap::Command::new("test"))` compiles and returns `name == "test"`

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test clap_adapter`
