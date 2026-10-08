//! `docli` — a language-agnostic CLI documentation generator.
//!
//! Consumes a neutral command-tree JSON ([`CliModel`]) and renders a
//! self-contained two-pane HTML reference and/or a Markdown manual. Call
//! [`ops::generate`] and [`ops::show`] for the same envelope the CLI prints.

pub mod clap_model;
pub mod contract;
pub mod ops;
pub mod render;
pub mod schema;

#[doc(inline)]
pub use schema::{ArgumentSpec, CliModel, OptionSpec};
