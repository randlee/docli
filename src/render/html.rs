//! HTML renderer: a self-contained two-pane reference page.
//!
//! [`render`] fills the embedded `default` pack (`templates/html/default/`)
//! with the model's command tree, `#docli-data`, and `#docli-search`. The
//! stylesheet and script live in that pack. This function always uses the
//! pack's default theme. Caller template and theme selection belongs to
//! [`crate::ops::generate`].

use crate::schema::CliModel;
use crate::templates::{render_embedded_default, EmbeddedDefaultError};

/// Renders `model` with the embedded `default` pack and its default theme.
///
/// # Panics
///
/// Panics if the embedded pack cannot render. That is a broken binary
/// (`DOCLI.INTERNAL`). [`try_render`] returns the same failure so `ops` can
/// put it in an envelope.
pub fn render(model: &CliModel) -> String {
    try_render(model).unwrap_or_else(|error| panic!("DOCLI.INTERNAL: {error}"))
}

/// Render the embedded default pack, or return `DOCLI.INTERNAL`.
///
/// # Errors
///
/// Returns [`EmbeddedDefaultError`] when the compiled-in pack cannot render.
pub(crate) fn try_render(model: &CliModel) -> Result<String, EmbeddedDefaultError> {
    render_embedded_default(model)
}

#[cfg(test)]
mod html_anchors {
    use super::render;
    use crate::schema::CliModel;

    #[test]
    fn static_tree_suffixes_colliding_anchors() {
        let model: CliModel = serde_json::from_str(
            r#"{"name":"demo","subcommands":[{"name":"Run Once"},{"name":"run-once"}]}"#,
        )
        .expect("parse model");
        let html = render(&model);
        assert!(html.contains("data-anchor=\"demo-run-once\""));
        assert!(html.contains("data-anchor=\"demo-run-once-2\""));
        assert!(html.contains("\"anchor\":\"demo-run-once-2\""));
    }
}
