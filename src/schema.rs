//! The language-neutral CLI model that `docli` consumes.
//!
//! This is the stable contract between any CLI framework adapter and the
//! `docli` renderer. It deliberately carries no framework-specific concepts:
//! clap's `possible_values` maps to [`OptionSpec::choices`], clap's
//! `value_names` to [`OptionSpec::value_name`], and clap's `num_args` to
//! [`OptionSpec::min_values`]/[`OptionSpec::max_values`].

use serde::{Deserialize, Serialize};

/// A command (or subcommand) in a CLI's command tree.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct CliModel {
    /// Command name (leaf, not the full path).
    pub name: String,
    /// Version string, when the CLI reports one.
    #[serde(default)]
    pub version: Option<String>,
    /// Short one-line description.
    #[serde(default)]
    pub description: String,
    /// Long description; falls back to `description` when empty.
    #[serde(default)]
    pub long_description: String,
    /// Trailing help / epilogue notes.
    #[serde(default)]
    pub epilogue: String,
    /// Full usage line(s), e.g. `demo [OPTIONS] <COMMAND>`.
    #[serde(default)]
    pub usage: String,
    /// Flags and options (things with a `--long` and/or `-s` short form).
    #[serde(default)]
    pub options: Vec<OptionSpec>,
    /// Positional arguments.
    #[serde(default)]
    pub arguments: Vec<ArgumentSpec>,
    /// Nested subcommands.
    #[serde(default)]
    pub subcommands: Vec<CliModel>,
    /// Unknown fields preserved for forward compatibility.
    #[serde(flatten, default)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A flag or option (e.g. `--output <PATH>`, `-v`).
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct OptionSpec {
    /// Canonical identifier (e.g. `output`, `verbose`).
    pub name: String,
    /// Long form, with dashes (e.g. `--output`).
    #[serde(default)]
    pub long: Option<String>,
    /// Short form, with dash (e.g. `-v`).
    #[serde(default)]
    pub short: Option<String>,
    /// Short help text.
    #[serde(default)]
    pub help: String,
    /// Long help text; falls back to `help` when empty.
    #[serde(default)]
    pub long_help: String,
    /// Value placeholder (e.g. `PATH`), when the option takes a value.
    #[serde(default)]
    pub value_name: Option<String>,
    #[serde(default)]
    pub required: bool,
    /// Default value as a string, when present.
    #[serde(default)]
    pub default_value: Option<String>,
    /// Allowed values, when the option is constrained.
    #[serde(default)]
    pub choices: Vec<String>,
    /// Minimum number of values this option consumes.
    #[serde(default)]
    pub min_values: Option<usize>,
    /// Maximum number of values this option consumes.
    #[serde(default)]
    pub max_values: Option<usize>,
    /// Unknown fields preserved for forward compatibility.
    #[serde(flatten, default)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// A positional argument.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct ArgumentSpec {
    pub name: String,
    #[serde(default)]
    pub help: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<String>,
    #[serde(default)]
    pub choices: Vec<String>,
    /// Unknown fields preserved for forward compatibility.
    #[serde(flatten, default)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}
