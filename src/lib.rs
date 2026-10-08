//! `docli` — a language-agnostic CLI documentation generator.
//!
//! Consumes a neutral command-tree JSON ([`CliModel`]) and renders a
//! self-contained two-pane HTML reference and/or a Markdown manual.
//! [`search_index`] supplies the embedded search JSON. Call
//! [`ops::generate`] and [`ops::show`] for the same envelope the CLI prints.

pub mod clap_model;
pub mod cli;
pub mod contract;
pub mod ops;
pub mod render;
pub mod schema;
pub mod search;

#[doc(inline)]
pub use schema::{ArgumentSpec, CliModel, OptionSpec};
#[doc(inline)]
pub use search::{matching_anchors, search_index, SearchEntry};
