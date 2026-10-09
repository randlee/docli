//! `docli templates` list, show, and validate (`REQ-DOCLI-CLI-011`).

mod common;

use std::fs;
use std::path::{Path, PathBuf};

use common::envelope::{assert_failure, assert_success, parse_envelope};
use common::harness::{cargo_docli_bin, docli_bin, run, unique_dir};
use docli::ops::{templates_list, templates_show, TemplatesListRequest, TemplatesShowRequest};

fn assert_exit(output: &std::process::Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn default_pack_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("templates/html/default")
}

fn write_valid_pack(root: &Path, id: &str) -> PathBuf {
    let pack = root.join(id);
    fs::create_dir_all(&pack).unwrap();
    fs::write(
        pack.join("template.toml"),
        format!(
            "id = \"{id}\"\nname = \"Extra {id}\"\nversion = \"2\"\ndescription = \"extra\"\n\n[theme_schema]\naccent = {{ type = \"color\", default = \"#111111\", description = \"Accent\" }}\n"
        ),
    )
    .unwrap();
    fs::write(pack.join("page.html.j2"), "<p>{{ title }}</p>\n").unwrap();
    fs::write(
        pack.join("style.css.j2"),
        "body{color:{{ theme.accent }}}\n",
    )
    .unwrap();
    fs::write(pack.join("script.js"), "").unwrap();
    pack
}

#[test]
fn templates_list_json_includes_default() {
    let output = run(&mut docli_bin(), &["templates", "list", "--json"]);
    assert_exit(&output, 0);
    let envelope = parse_envelope(&output.stdout);
    assert_success(&envelope);
    let data = &envelope["data"];
    assert_eq!(data["operation"], "templates_list");
    let install_root = data["install_root"].as_str().expect("install_root");
    assert!(
        install_root.ends_with("share/docli/templates"),
        "{install_root}"
    );
    let default = data["templates"]
        .as_array()
        .expect("templates")
        .iter()
        .find(|template| template["id"] == "default")
        .expect("default pack");
    assert_eq!(default["name"], "docli default");
    assert_eq!(default["version"], "1");
    assert_eq!(default["path"], "embedded:default");
}

#[test]
fn templates_show_default_json_includes_theme_and_example_argv() {
    let output = run(
        &mut docli_bin(),
        &["templates", "show", "default", "--json"],
    );
    assert_exit(&output, 0);
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
}

#[test]
fn templates_validate_default_directory_json() {
    let pack = default_pack_dir();
    let pack_arg = pack.to_str().expect("utf8");
    let output = run(
        &mut docli_bin(),
        &["templates", "validate", pack_arg, "--json"],
    );
    assert_exit(&output, 0);
    let envelope = parse_envelope(&output.stdout);
    assert_success(&envelope);
    let data = &envelope["data"];
    assert_eq!(data["operation"], "templates_validate");
    assert_eq!(data["path"], pack_arg);
    assert_eq!(data["id"], "default");
    assert_eq!(data["version"], "1");
    assert_eq!(data["valid"], true);
}

#[test]
fn templates_validate_embedded_default_json() {
    let output = run(
        &mut docli_bin(),
        &["templates", "validate", "embedded:default", "--json"],
    );
    assert_exit(&output, 0);
    let envelope = parse_envelope(&output.stdout);
    assert_success(&envelope);
    assert_eq!(envelope["data"]["path"], "embedded:default");
    assert_eq!(envelope["data"]["id"], "default");
    assert_eq!(envelope["data"]["valid"], true);
}

#[test]
fn templates_list_includes_install_root_pack_and_skips_bundled_id() {
    let dir = unique_dir();
    let root = dir.join("share").join("docli").join("templates");
    write_valid_pack(&root, "extra");
    let shadowed = root.join("also-default");
    fs::create_dir_all(&shadowed).unwrap();
    fs::write(
        shadowed.join("template.toml"),
        "id = \"default\"\nname = \"shadow\"\nversion = \"9\"\ndescription = \"shadow\"\ntheme_schema = {}\n",
    )
    .unwrap();
    fs::write(shadowed.join("page.html.j2"), "ok\n").unwrap();
    fs::write(shadowed.join("style.css.j2"), "body{}\n").unwrap();
    fs::write(shadowed.join("script.js"), "").unwrap();

    let envelope = templates_list(TemplatesListRequest {
        install_root: root.clone(),
    });
    assert!(envelope.ok, "{:?}", envelope.error);
    let data = envelope.data.expect("data");
    assert_eq!(data.install_root, root);
    let ids: Vec<_> = data
        .templates
        .iter()
        .map(|template| template.id.as_str())
        .collect();
    assert_eq!(ids.iter().filter(|id| **id == "default").count(), 1);
    assert!(ids.contains(&"extra"));
    let default = data
        .templates
        .iter()
        .find(|template| template.id == "default")
        .unwrap();
    assert_eq!(default.path, "embedded:default");
    assert_eq!(default.version, "1");
    let extra = data
        .templates
        .iter()
        .find(|template| template.id == "extra")
        .unwrap();
    assert_eq!(extra.version, "2");
    assert_eq!(extra.path, root.join("extra").display().to_string());
}

#[test]
fn templates_show_unknown_id_is_template_invalid() {
    let output = run(
        &mut docli_bin(),
        &["templates", "show", "not-a-pack", "--json"],
    );
    assert_exit(&output, 2);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        2,
        "validation",
        "DOCLI.TEMPLATE_INVALID",
    );
    assert!(error["details"]["cause"]
        .as_str()
        .unwrap()
        .contains("not-a-pack"));
    assert!(error["suggested_action"]
        .as_str()
        .unwrap()
        .contains("templates list"));
}

#[test]
fn templates_show_directory_json() {
    let dir = unique_dir();
    let pack = write_valid_pack(&dir, "brand");
    let envelope = templates_show(TemplatesShowRequest {
        id_or_path: pack.display().to_string(),
    });
    assert!(envelope.ok, "{:?}", envelope.error);
    let data = envelope.data.expect("data");
    assert_eq!(data.id, "brand");
    assert_eq!(data.theme_schema["accent"].default, "#111111");
    assert_eq!(
        data.example_generate_argv,
        [
            "docli",
            "generate",
            "--input",
            "model.json",
            "--html",
            "site/cli"
        ]
    );
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
