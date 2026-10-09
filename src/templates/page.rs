//! Static command tree and HTML/JSON escaping for pack render.
//!
//! The tree markup must stay byte-identical to the Phase A renderer: caret
//! and command buttons are siblings, and anchors follow [`unique_anchor`].

use std::collections::BTreeSet;
use std::fmt::Write;

use minijinja::Value;

use crate::schema::CliModel;
use crate::search::{anchor_for_path, unique_anchor};

pub(crate) fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

/// Phase A HTML escape as a MiniJinja filter.
///
/// MiniJinja's HTML escape also rewrites `'` and `/`. This filter keeps the
/// Phase A replacements so command names stay byte-compatible.
pub(crate) fn docli_escape(value: String) -> Value {
    Value::from_safe_string(escape_html(&value))
}

/// Serializes `value` for an inline JSON script tag.
///
/// This cannot fail for a valid [`CliModel`]: serde writes plain JSON values,
/// and the `</` escape is a [`String`] replacement.
pub(crate) fn embed_json(value: &impl serde::Serialize) -> String {
    serde_json::to_string(value)
        .expect("serialize embedded JSON")
        .replace("</", "<\\/")
}

pub(crate) fn render_tree(model: &CliModel) -> String {
    let mut names = Vec::new();
    let mut used = BTreeSet::new();
    let mut out = String::from("<ul>");
    render_node(model, &mut names, &mut used, &mut out);
    out.push_str("</ul>");
    out
}

/// Appends the HTML for `command` and its subcommands.
///
/// Writing into `out` cannot fail for a valid [`CliModel`]: `out` is a
/// [`String`], and [`std::fmt::Write`] for [`String`] does not return an error.
fn render_node(
    command: &CliModel,
    names: &mut Vec<String>,
    used: &mut BTreeSet<String>,
    out: &mut String,
) {
    names.push(command.name.clone());
    let anchor = unique_anchor(&anchor_for_path(names), used);
    let has_children = !command.subcommands.is_empty();
    let class = if has_children {
        "node open"
    } else {
        "node leaf"
    };
    let expanded = if has_children { "true" } else { "false" };
    let name = escape_html(&command.name);
    write!(
        out,
        "<li class=\"{class}\" data-anchor=\"{anchor}\"><button type=\"button\" class=\"docli-caret\" aria-expanded=\"{expanded}\"></button><button type=\"button\" class=\"docli-cmd\" data-anchor=\"{anchor}\">{name}</button>"
    )
    .expect("write html");
    if has_children {
        out.push_str("<ul class=\"children\">");
        for sub in &command.subcommands {
            render_node(sub, names, used, out);
        }
        out.push_str("</ul>");
    }
    out.push_str("</li>");
    names.pop();
}
