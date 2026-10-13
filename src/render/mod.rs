//! Renderers: turn a [`CliModel`] into HTML and Markdown.
//!
//! HTML bytes come from the embedded `default` template pack. This module
//! does not write files or build envelopes.
pub mod html;
pub mod markdown;
