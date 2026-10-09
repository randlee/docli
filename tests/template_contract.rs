//! `docli templates` success envelopes and `cargo docli` list parity (`REQ-DOCLI-CLI-011`).

mod common;

use common::envelope::{assert_json_mode_stdout_only_envelope, assert_success, parse_envelope};
use common::harness::{cargo_docli_bin, docli_bin, run, workspace_fixture};

#[test]
fn templates_list_json_includes_default() {
    let output = run(&mut docli_bin(), &["templates", "list", "--json"]);
    assert_eq!(output.status.code(), Some(0));
    let envelope = parse_envelope(&output.stdout);
    assert_success(&envelope);
    let data = &envelope["data"];
    assert_eq!(data["operation"], "templates_list");
    let install_root = data["install_root"].as_str().expect("install_root");
    assert!(
        install_root
            .replace('\\', "/")
            .ends_with("share/docli/templates"),
        "install_root was {install_root}"
    );
    let templates = data["templates"].as_array().expect("templates");
    let default = templates
        .iter()
        .find(|template| template["id"] == "default")
        .expect("default pack");
    assert_eq!(default["name"], "docli default");
    assert_eq!(default["version"], "1");
    assert_eq!(default["path"], "embedded:default");
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn templates_show_default_json_includes_theme_schema_and_example_argv() {
    let output = run(
        &mut docli_bin(),
        &["templates", "show", "default", "--json"],
    );
    assert_eq!(output.status.code(), Some(0));
    let envelope = parse_envelope(&output.stdout);
    assert_success(&envelope);
    let data = &envelope["data"];
    assert_eq!(data["operation"], "templates_show");
    assert_eq!(data["id"], "default");
    assert_eq!(data["manifest"]["id"], "default");
    assert_eq!(data["manifest"]["version"], "1");
    assert_eq!(data["theme_schema"]["accent"]["type"], "color");
    assert_eq!(data["theme_schema"]["accent"]["default"], "#007acc");
    assert_eq!(
        data["theme_schema"]["accent"]["description"],
        "Primary accent"
    );
    assert_eq!(
        data["example_generate_argv"],
        serde_json::json!([
            "docli",
            "generate",
            "--input",
            "model.json",
            "--html",
            "site/cli"
        ])
    );
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn templates_validate_default_pack_json_succeeds() {
    let pack = workspace_fixture("templates/html/default");
    let output = run(
        &mut docli_bin(),
        &[
            "templates",
            "validate",
            pack.to_str().expect("utf8"),
            "--json",
        ],
    );
    assert_eq!(output.status.code(), Some(0));
    let envelope = parse_envelope(&output.stdout);
    assert_success(&envelope);
    let data = &envelope["data"];
    assert_eq!(data["operation"], "templates_validate");
    assert_eq!(data["id"], "default");
    assert_eq!(data["valid"], true);
    assert_eq!(data["path"], pack.to_string_lossy().as_ref());
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn cargo_docli_templates_list_matches_docli() {
    let args = ["templates", "list", "--json"];
    let docli = run(&mut docli_bin(), &args);
    let cargo = run(&mut cargo_docli_bin(), &args);
    assert_eq!(docli.status.code(), Some(0), "docli list failed");
    assert_eq!(cargo.status.code(), Some(0), "cargo docli list failed");
    assert_eq!(
        docli.stdout,
        cargo.stdout,
        "stdout mismatch\ndocli: {}\ncargo-docli: {}",
        String::from_utf8_lossy(&docli.stdout),
        String::from_utf8_lossy(&cargo.stdout)
    );
}
