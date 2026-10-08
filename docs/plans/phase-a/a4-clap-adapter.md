---
id: a.4
title: clap adapter mapping
status: planned
branch: feature/phase-a-a4-clap
worktree: ../docli-worktrees/feature/phase-a-a4-clap
target: develop
---

# Sprint a.4 — clap adapter mapping

## Goal

- `docli::from_clap` fills the neutral model the way `REQ-DOCLI-RUST-001` and `REQ-DOCLI-RUST-002` require.

## Hard Dependencies

- Existing `src/clap_model.rs` and `src/schema.rs`
- a.2 renderer fixtures stay green after any mapping change that flows into render tests

## Deliverables

- `src/clap_model.rs` — mapping below, including fixes where the prototype drops required fields
- `tests/clap_adapter.rs` — the acceptance checks in this sprint

## Explicit Code Samples

```rust
pub fn from_clap(command: &clap::Command) -> CliModel;
```

Mapping:

- `possible_values` → `choices`
- first `value_names` entry → `value_name`
- `num_args` → `min_values` and `max_values`
- arg id `help` or `version` is excluded
- subcommand `usage` contains the parent command path (`demo run`, not `run`)

## Out of Scope

- A second CLI framework adapter
- Generating HTML inside `from_clap`
- Reading clap types out of atm-core, sc-compose, or sc-observability in this sprint

## Acceptance Criteria

- An option with `possible_values(["json", "yaml"])` yields `choices` of those two strings
- An option with `num_args(2..=3)` yields `min_values == Some(2)` and `max_values == Some(3)`
- An option with `value_name = "PATH"` yields `value_name == Some("PATH")`
- The model contains no option or argument whose name is `help` or `version` when those args are clap's built-in flags
- A subcommand named `run` under command `demo` has `usage` containing `demo run`

## Required Validation

- Phase A CI gate in [README.md](README.md)
- `cargo test --test clap_adapter`
