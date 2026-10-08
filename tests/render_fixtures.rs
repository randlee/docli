//! Byte-lock both renderers against `fixtures/contract/`.

use std::fs;
use std::path::PathBuf;

use docli::CliModel;

fn fixture_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("fixtures/contract");
    path.push(name);
    path
}

fn read_fixture(name: &str) -> String {
    fs::read_to_string(fixture_path(name)).unwrap_or_else(|err| panic!("read {name}: {err}"))
}

fn command_count(model: &CliModel) -> usize {
    1 + model.subcommands.iter().map(command_count).sum::<usize>()
}

fn assert_inline_assets(html: &str) {
    let lower = html.to_ascii_lowercase();
    for key in ["href=", "src="] {
        let mut rest = lower.as_str();
        while let Some(index) = rest.find(key) {
            let after = &rest[index + key.len()..];
            let value = after.trim_start_matches([' ', '"', '\'']);
            let end = value.find(['"', '\'', ' ', '>']).unwrap_or(value.len());
            let url = value[..end].split('?').next().unwrap_or("");
            assert!(
                !url.ends_with(".css") && !url.ends_with(".js"),
                "external asset in {key}{url}"
            );
            rest = &rest[index + key.len()..];
        }
    }
}

fn assert_markdown_sections(model: &CliModel, path: &str, markdown: &str) {
    assert!(
        markdown.contains(&format!("`{path}`")),
        "missing section for {path}"
    );
    for sub in &model.subcommands {
        assert_markdown_sections(sub, &format!("{path} {}", sub.name), markdown);
    }
}

#[test]
fn contract_fixtures_match_renderer_bytes() {
    let model: CliModel = serde_json::from_str(&read_fixture("model.json")).expect("model");
    let html = docli::render::html::render(&model);
    let markdown = docli::render::markdown::render(&model);

    let commands = command_count(&model);
    assert_eq!(
        html.matches("class=\"docli-caret\"").count(),
        commands,
        "each command needs a caret button"
    );
    assert_eq!(
        html.matches("class=\"docli-cmd\"").count(),
        commands,
        "each command needs a command button"
    );
    assert_eq!(
        html.matches("</button><button type=\"button\" class=\"docli-cmd\"")
            .count(),
        commands,
        "caret and command buttons must be siblings"
    );
    assert!(html.contains("id=\"docli-data\""));
    assert!(html.contains("id=\"docli-search\""));
    assert!(html.contains("<style"));
    assert!(html.contains("<script"));
    assert_eq!(html.matches("class=\"pane\"").count(), 2);
    assert!(html.contains(
        "[\"Name\", \"Short\", \"Value\", \"Required\", \"Default\", \"Choices\", \"Description\"]"
    ));
    assert_inline_assets(&html);
    assert_markdown_sections(&model, &model.name, &markdown);

    let data = serde_json::to_string(&model)
        .expect("model json")
        .replace("</", "<\\/");
    let search = serde_json::to_string(&docli::search_index(&model))
        .expect("search json")
        .replace("</", "<\\/");
    assert!(html.contains(&format!(
        "<script id=\"docli-data\" type=\"application/json\">{data}</script>"
    )));
    assert!(html.contains(&format!(
        "<script id=\"docli-search\" type=\"application/json\">{search}</script>"
    )));

    assert_eq!(html, read_fixture("index.html"));
    assert_eq!(markdown, read_fixture("manual.md"));
}
