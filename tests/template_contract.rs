//! `docli templates` success envelopes and `generate` preview (`REQ-DOCLI-CLI-011`, `REQ-DOCLI-CLI-012`).

mod common;

use std::fs;
use std::path::Path;

use common::envelope::{assert_json_mode_stdout_only_envelope, assert_success, parse_envelope};
use common::harness::{
    cargo_docli_bin, docli_bin, run, unique_dir, workspace_fixture, write_demo_model,
};

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
    let cli_doc = templates
        .iter()
        .find(|template| template["id"] == "cli-doc")
        .expect("cli-doc pack");
    assert_eq!(cli_doc["name"], "docli cli-doc");
    assert_eq!(cli_doc["version"], "1");
    assert_eq!(cli_doc["path"], "embedded:cli-doc");
    assert!(
        templates
            .iter()
            .all(|template| template["id"] != "_skeleton"),
        "list included _skeleton: {templates:?}"
    );
    assert!(templates.iter().all(|template| {
        template["id"]
            .as_str()
            .is_some_and(|id| !id.starts_with('_'))
    }));
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
            "--preview",
            "--template",
            "default",
            "--theme",
            r##"{"accent":"#007acc"}"##
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
fn generate_preview_default_template_three_distinct_themes() {
    let dir = unique_dir();
    let model = write_demo_model(&dir);
    let themes = [
        r##"{"accent":"#007acc"}"##,
        r##"{"accent":"#005a9e"}"##,
        r##"{"accent":"#059669"}"##,
    ];
    let mut preview_dirs = Vec::new();
    let mut pages = Vec::new();
    for theme in themes {
        let mut cmd = docli_bin();
        cmd.current_dir(&dir);
        let output = run(
            &mut cmd,
            &[
                "generate",
                "--input",
                model.to_str().expect("utf8"),
                "--preview",
                "--template",
                "default",
                "--theme",
                theme,
                "--json",
            ],
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let envelope = parse_envelope(&output.stdout);
        assert_success(&envelope);
        assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
        let data = &envelope["data"];
        assert_eq!(data["operation"], "generate");
        let html_dir = data["html_dir"].as_str().expect("html_dir");
        let preview_dir = data["preview_dir"].as_str().expect("preview_dir");
        assert_eq!(html_dir, preview_dir);
        assert_preview_dir_name(preview_dir);
        let index = Path::new(preview_dir).join("index.html");
        assert!(index.is_file(), "missing {}", index.display());
        pages.push(fs::read_to_string(&index).expect("read preview"));
        preview_dirs.push(preview_dir.to_owned());
    }
    assert_eq!(preview_dirs.len(), 3);
    assert_ne!(preview_dirs[0], preview_dirs[1]);
    assert_ne!(preview_dirs[1], preview_dirs[2]);
    assert_ne!(preview_dirs[0], preview_dirs[2]);
    assert!(pages[0].contains("--accent:#007acc"));
    assert!(pages[1].contains("--accent:#005a9e"));
    assert!(pages[2].contains("--accent:#059669"));
    assert_ne!(pages[0], pages[1]);
    assert_ne!(pages[1], pages[2]);
    assert!(!dir.join("site/cli").exists());
}

#[test]
fn generate_default_template_explicit_html_matches_contract_bytes() {
    let dir = unique_dir();
    let model = workspace_fixture("fixtures/contract/model.json");
    let expected = fs::read_to_string(workspace_fixture("fixtures/contract/index.html"))
        .expect("contract html");
    for template in [None, Some("default")] {
        let html_dir = dir.join(template.unwrap_or("omitted"));
        let mut args = vec![
            "generate",
            "--input",
            model.to_str().expect("utf8"),
            "--html",
            html_dir.to_str().expect("utf8"),
            "--json",
        ];
        if let Some(id) = template {
            args.extend(["--template", id]);
        }
        let mut cmd = docli_bin();
        cmd.current_dir(&dir);
        let output = run(&mut cmd, &args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "template={template:?} stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let envelope = parse_envelope(&output.stdout);
        assert_success(&envelope);
        assert_eq!(envelope["data"]["preview_dir"], serde_json::Value::Null);
        assert_eq!(
            envelope["data"]["html_dir"],
            html_dir.to_string_lossy().as_ref()
        );
        let written = fs::read_to_string(html_dir.join("index.html")).expect("written html");
        assert_eq!(written, expected, "template={template:?}");
    }
    assert!(!dir.join("site/cli").exists());
}

#[test]
fn templates_validate_skeleton_succeeds() {
    let pack = workspace_fixture("templates/html/_skeleton");
    let output = run(
        &mut docli_bin(),
        &[
            "templates",
            "validate",
            pack.to_str().expect("utf8"),
            "--json",
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope = parse_envelope(&output.stdout);
    assert_success(&envelope);
    let data = &envelope["data"];
    assert_eq!(data["operation"], "templates_validate");
    assert_eq!(data["id"], "_skeleton");
    assert_eq!(data["valid"], true);
    assert_eq!(data["path"], pack.to_string_lossy().as_ref());
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn generate_cli_doc_docli_repo_fixture_has_layout_markers() {
    let dir = unique_dir();
    let model = workspace_fixture("fixtures/repos/docli.json");
    let cli_dir = dir.join("cli-doc");
    let default_dir = dir.join("default");
    let cli_html = render_template(&dir, &model, "cli-doc", &cli_dir);
    let default_html = render_template(&dir, &model, "default", &default_dir);
    assert!(cli_html.contains("data-layout=\"two-column\""));
    assert!(cli_html.contains("class=\"option-card\""));
    assert!(cli_html.contains("id=\"docli-data\""));
    assert!(cli_html.contains("id=\"docli-search\""));
    assert!(cli_html.contains("id=\"docli-cli-doc-pack\""));
    assert!(!cli_html.contains("id=\"docli-default-pack\""));
    assert!(!cli_html.contains("<link"));
    assert!(!cli_html.contains("src=\"http"));
    assert!(!default_html.contains("data-layout=\"two-column\""));
    assert!(!default_html.contains("option-card"));
    assert!(default_html.contains("id=\"docli-data\""));
    assert!(default_html.contains("id=\"docli-search\""));
    assert!(default_html.contains("id=\"docli-default-pack\""));
}

#[test]
fn agent_preview_three_calls_succeed_on_docli_fixture() {
    let model = workspace_fixture("fixtures/repos/docli.json");
    let calls = [
        (
            "default",
            r##"{"accent":"#007acc","font_body":"system-ui"}"##,
            "#007acc",
        ),
        (
            "cli-doc",
            r##"{"accent":"#d73a49","font_body":"Monaco"}"##,
            "#d73a49",
        ),
        (
            "default",
            r##"{"accent":"#059669","font_body":"Inter"}"##,
            "#059669",
        ),
    ];
    let mut preview_dirs = Vec::new();
    for (template, theme, accent) in calls {
        let output = run(
            &mut docli_bin(),
            &[
                "generate",
                "--json",
                "--input",
                model.to_str().expect("utf8"),
                "--preview",
                "--template",
                template,
                "--theme",
                theme,
            ],
        );
        assert_eq!(
            output.status.code(),
            Some(0),
            "template={template} stderr={}",
            String::from_utf8_lossy(&output.stderr)
        );
        let envelope = parse_envelope(&output.stdout);
        assert_success(&envelope);
        assert_eq!(envelope["ok"], true);
        assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
        let preview_dir = envelope["data"]["preview_dir"]
            .as_str()
            .expect("preview_dir");
        assert_preview_dir_name(preview_dir);
        let html = fs::read_to_string(Path::new(preview_dir).join("index.html")).expect("html");
        assert!(
            html.contains(&format!("--accent:{accent}")),
            "template={template} missing {accent}"
        );
        if template == "cli-doc" {
            assert!(html.contains("data-layout=\"two-column\""));
            assert!(html.contains("class=\"option-card\""));
            assert!(html.contains("font-family:Monaco"));
        }
        preview_dirs.push(preview_dir.to_owned());
    }
    assert_ne!(preview_dirs[0], preview_dirs[1]);
    assert_ne!(preview_dirs[1], preview_dirs[2]);
}

fn render_template(cwd: &Path, model: &Path, template: &str, html_dir: &Path) -> String {
    let mut cmd = docli_bin();
    cmd.current_dir(cwd);
    let output = run(
        &mut cmd,
        &[
            "generate",
            "--input",
            model.to_str().expect("utf8"),
            "--html",
            html_dir.to_str().expect("utf8"),
            "--template",
            template,
            "--json",
        ],
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "template={template} stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let envelope = parse_envelope(&output.stdout);
    assert_success(&envelope);
    assert_eq!(envelope["data"]["preview_dir"], serde_json::Value::Null);
    fs::read_to_string(html_dir.join("index.html")).expect("written html")
}

fn assert_preview_dir_name(preview_dir: &str) {
    let name = Path::new(preview_dir)
        .file_name()
        .and_then(|name| name.to_str())
        .expect("preview dir name");
    let rest = name
        .strip_prefix("docli-preview-")
        .unwrap_or_else(|| panic!("preview dir {name}"));
    let mut parts = rest.split('-');
    let pid = parts.next().unwrap_or_else(|| panic!("preview dir {name}"));
    let nanos = parts.next().unwrap_or_else(|| panic!("preview dir {name}"));
    let seq = parts.next().unwrap_or_else(|| panic!("preview dir {name}"));
    assert!(parts.next().is_none(), "preview dir {name}");
    for part in [pid, nanos, seq] {
        assert!(
            part.chars().all(|ch| ch.is_ascii_digit()) && !part.is_empty(),
            "preview dir {name}"
        );
    }
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
