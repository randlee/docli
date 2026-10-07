//! `docli` — a language-agnostic CLI documentation generator.
//!
//! Consumes a neutral command-tree JSON ([`CliModel`]) and renders a
//! self-contained two-pane HTML reference and/or a Markdown manual. Rust CLIs
//! can produce the input directly from clap via [`from_clap`].

pub mod clap_model;
pub mod render;
pub mod schema;

pub use clap_model::from_clap;
pub use schema::{ArgumentSpec, CliModel, OptionSpec};
