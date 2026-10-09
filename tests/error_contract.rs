//! Integration tests for every `DOCLI.*` error code (`REQ-DOCLI-CLI-003`, `REQ-DOCLI-CLI-008`).
//!
//! Agents and automation must receive one actionable version `"1"` envelope on `--json`.

mod common;

use std::fs;

use common::envelope::{
    assert_failure, assert_human_actionable, assert_json_mode_stdout_only_envelope, assert_success,
    parse_envelope,
};
use common::harness::{docli_bin, run, unique_dir, write_demo_model};

fn assert_exit(output: &std::process::Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn docli_usage_unknown_command_json() {
    let output = run(&mut docli_bin(), &["not-a-command", "--json"]);
    assert_exit(&output, 2);
    let envelope = parse_envelope(&output.stdout);
    let error = assert_failure(
        output.status.code(),
        &envelope,
        2,
        "validation",
        "DOCLI.USAGE",
    );
    assert_eq!(error["details"], serde_json::json!({}));
    assert!(error["suggested_action"]
        .as_str()
        .unwrap()
        .contains("not-a-command"));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_usage_invalid_flag_json() {
    let output = run(&mut docli_bin(), &["generate", "--not-a-flag", "--json"]);
    assert_exit(&output, 2);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        2,
        "validation",
        "DOCLI.USAGE",
    );
    assert_eq!(error["details"], serde_json::json!({}));
    assert!(error["suggested_action"]
        .as_str()
        .unwrap()
        .contains("--not-a-flag"));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_usage_show_without_paths_json() {
    let output = run(&mut docli_bin(), &["show", "--json"]);
    assert_exit(&output, 2);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        2,
        "validation",
        "DOCLI.USAGE",
    );
    assert_eq!(error["details"], serde_json::json!({}));
    let action = error["suggested_action"].as_str().unwrap();
    assert!(action.contains("--html"));
    assert!(action.contains("--markdown"));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_usage_show_without_paths_human() {
    let output = run(&mut docli_bin(), &["show"]);
    assert_exit(&output, 2);
    assert_human_actionable(&output.stderr, "DOCLI.USAGE", &["--html", "--markdown"]);
}

#[test]
fn docli_input_not_found_json() {
    let dir = unique_dir();
    let missing = dir.join("missing.json");
    let output = run(
        &mut docli_bin(),
        &["generate", "--input", missing.to_str().unwrap(), "--json"],
    );
    assert_exit(&output, 3);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        3,
        "not_found",
        "DOCLI.INPUT_NOT_FOUND",
    );
    assert_eq!(error["details"]["path"], missing.to_string_lossy().as_ref());
    assert!(error["suggested_action"]
        .as_str()
        .unwrap()
        .contains(missing.to_string_lossy().as_ref()));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_input_not_found_human() {
    let dir = unique_dir();
    let missing = dir.join("missing.json");
    let output = run(
        &mut docli_bin(),
        &["generate", "--input", missing.to_str().unwrap()],
    );
    assert_exit(&output, 3);
    assert_human_actionable(
        &output.stderr,
        "DOCLI.INPUT_NOT_FOUND",
        &[missing.to_string_lossy().as_ref()],
    );
}

#[test]
fn docli_input_invalid_parse_error_json() {
    let dir = unique_dir();
    let bad = dir.join("bad.json");
    fs::write(&bad, "{\"name\": 1}").unwrap();
    let output = run(
        &mut docli_bin(),
        &["generate", "--input", bad.to_str().unwrap(), "--json"],
    );
    assert_exit(&output, 2);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        2,
        "validation",
        "DOCLI.INPUT_INVALID",
    );
    let cause = error["details"]["cause"].as_str().expect("serde cause");
    assert!(!cause.is_empty());
    let action = error["suggested_action"].as_str().unwrap();
    assert!(action.contains(bad.to_string_lossy().as_ref()));
    assert!(action.contains(cause));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_input_invalid_empty_file_json() {
    let dir = unique_dir();
    let empty = dir.join("empty.json");
    fs::write(&empty, "").unwrap();
    let output = run(
        &mut docli_bin(),
        &["generate", "--input", empty.to_str().unwrap(), "--json"],
    );
    assert_exit(&output, 2);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        2,
        "validation",
        "DOCLI.INPUT_INVALID",
    );
    assert_eq!(error["details"], serde_json::json!({}));
    assert!(error["message"].as_str().unwrap().contains("model JSON"));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_input_invalid_human() {
    let dir = unique_dir();
    let bad = dir.join("bad.json");
    fs::write(&bad, "{\"name\": 1}").unwrap();
    let output = run(
        &mut docli_bin(),
        &["generate", "--input", bad.to_str().unwrap()],
    );
    assert_exit(&output, 2);
    assert_human_actionable(&output.stderr, "DOCLI.INPUT_INVALID", &["expected"]);
}

#[test]
fn docli_output_not_found_single_html_json() {
    let dir = unique_dir();
    let missing = dir.join("no-html");
    let output = run(
        &mut docli_bin(),
        &["show", "--html", missing.to_str().unwrap(), "--json"],
    );
    assert_exit(&output, 3);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        3,
        "not_found",
        "DOCLI.OUTPUT_NOT_FOUND",
    );
    let index = missing.join("index.html");
    assert_eq!(
        error["details"]["artifacts"],
        serde_json::json!([{
            "path": index.to_string_lossy().as_ref(),
            "exists": false
        }])
    );
    assert!(error["suggested_action"]
        .as_str()
        .unwrap()
        .contains(index.to_string_lossy().as_ref()));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_output_not_found_html_and_markdown_json() {
    let dir = unique_dir();
    let html_dir = dir.join("html-missing");
    let md = dir.join("missing.md");
    let output = run(
        &mut docli_bin(),
        &[
            "show",
            "--html",
            html_dir.to_str().unwrap(),
            "--markdown",
            md.to_str().unwrap(),
            "--json",
        ],
    );
    assert_exit(&output, 3);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        3,
        "not_found",
        "DOCLI.OUTPUT_NOT_FOUND",
    );
    let artifacts = error["details"]["artifacts"].as_array().unwrap();
    assert_eq!(artifacts.len(), 2);
    assert!(artifacts.iter().all(|a| a["exists"] == false));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_output_not_found_human() {
    let dir = unique_dir();
    let missing = dir.join("no-html");
    let output = run(
        &mut docli_bin(),
        &["show", "--html", missing.to_str().unwrap()],
    );
    assert_exit(&output, 3);
    let index = missing.join("index.html");
    assert_human_actionable(
        &output.stderr,
        "DOCLI.OUTPUT_NOT_FOUND",
        &[index.to_string_lossy().as_ref()],
    );
}

#[test]
fn docli_io_generate_html_dir_not_writable_json() {
    let dir = unique_dir();
    let model = write_demo_model(&dir);
    let html_blocker = dir.join("blocker-file");
    fs::write(&html_blocker, "x").unwrap();
    let output = run(
        &mut docli_bin(),
        &[
            "generate",
            "--input",
            model.to_str().unwrap(),
            "--html",
            html_blocker.to_str().unwrap(),
            "--json",
        ],
    );
    assert_exit(&output, 4);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        4,
        "dependency",
        "DOCLI.IO",
    );
    assert!(error["details"]["cause"].as_str().is_some());
    assert!(error["details"].get("outputs_written").is_none());
    assert!(error["suggested_action"]
        .as_str()
        .unwrap()
        .contains("blocker-file"));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_io_generate_partial_write_lists_outputs_written_json() {
    let dir = unique_dir();
    let model = write_demo_model(&dir);
    let html_dir = dir.join("html");
    let blocker = dir.join("blocker-file");
    fs::write(&blocker, "x").unwrap();
    let markdown = blocker.join("out.md");
    let output = run(
        &mut docli_bin(),
        &[
            "generate",
            "--input",
            model.to_str().unwrap(),
            "--html",
            html_dir.to_str().unwrap(),
            "--markdown",
            markdown.to_str().unwrap(),
            "--json",
        ],
    );
    assert_exit(&output, 4);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        4,
        "dependency",
        "DOCLI.IO",
    );
    assert!(error["details"]["cause"]
        .as_str()
        .is_some_and(|cause| !cause.is_empty()));
    let written = error["details"]["outputs_written"]
        .as_array()
        .expect("partial write must list outputs_written");
    assert_eq!(written.len(), 1);
    assert_eq!(written[0]["kind"], "html");
    assert!(html_dir.join("index.html").is_file());
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_io_show_unreadable_index_json() {
    let dir = unique_dir();
    let model = write_demo_model(&dir);
    let html_dir = dir.join("html");
    let gen = run(
        &mut docli_bin(),
        &[
            "generate",
            "--input",
            model.to_str().unwrap(),
            "--html",
            html_dir.to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(gen.status.code(), Some(0));
    let index = html_dir.join("index.html");
    fs::remove_file(&index).unwrap();
    fs::create_dir(&index).unwrap();
    let output = run(
        &mut docli_bin(),
        &["show", "--html", html_dir.to_str().unwrap(), "--json"],
    );
    assert_exit(&output, 4);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        4,
        "dependency",
        "DOCLI.IO",
    );
    assert!(error["details"]["cause"].as_str().is_some());
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_io_show_unreadable_index_human() {
    let dir = unique_dir();
    let model = write_demo_model(&dir);
    let html_dir = dir.join("html");
    run(
        &mut docli_bin(),
        &[
            "generate",
            "--input",
            model.to_str().unwrap(),
            "--html",
            html_dir.to_str().unwrap(),
            "--json",
        ],
    );
    let index = html_dir.join("index.html");
    fs::remove_file(&index).unwrap();
    fs::create_dir(&index).unwrap();
    let output = run(
        &mut docli_bin(),
        &["show", "--html", html_dir.to_str().unwrap()],
    );
    assert_exit(&output, 4);
    assert_human_actionable(&output.stderr, "DOCLI.IO", &["index.html"]);
}

#[test]
fn docli_internal_error_body_contract() {
    let body = docli::contract::ErrorBody::internal("serialization failed");
    assert_eq!(body.code, docli::contract::ErrorCode::Internal);
    assert!(!body.message.is_empty());
    assert!(!body.suggested_action.is_empty());
    assert_eq!(body.details["cause"], "serialization failed");
    let envelope = docli::contract::Envelope::<()>::failure(body);
    assert_eq!(envelope.exit_code(), docli::contract::EXIT_INTERNAL);
    let json = serde_json::to_value(&envelope).unwrap();
    assert_failure(
        Some(envelope.exit_code()),
        &json,
        1,
        "internal",
        "DOCLI.INTERNAL",
    );
}

#[test]
fn docli_success_json_still_single_envelope() {
    let dir = unique_dir();
    let model = write_demo_model(&dir);
    let output = run(
        &mut docli_bin(),
        &[
            "generate",
            "--input",
            model.to_str().unwrap(),
            "--html",
            dir.join("out").to_str().unwrap(),
            "--json",
        ],
    );
    assert_eq!(output.status.code(), Some(0));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
    assert_success(&parse_envelope(&output.stdout));
}

#[test]
fn docli_stdin_empty_is_input_invalid() {
    use std::io::Write;
    use std::process::Stdio;

    let mut child = docli_bin()
        .args(["generate", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");
    child.stdin.take().unwrap().write_all(b"").unwrap();
    let output = child.wait_with_output().unwrap();
    assert_exit(&output, 2);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        2,
        "validation",
        "DOCLI.INPUT_INVALID",
    );
    assert_eq!(error["details"], serde_json::json!({}));
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

fn write_broken_pack(dir: &std::path::Path) -> std::path::PathBuf {
    let pack = dir.join("broken-pack");
    fs::create_dir_all(&pack).unwrap();
    fs::write(pack.join("template.toml"), "id = [\n").unwrap();
    pack
}

#[test]
fn docli_template_invalid_validate_json() {
    let dir = unique_dir();
    let pack = write_broken_pack(&dir);
    let output = run(
        &mut docli_bin(),
        &["templates", "validate", pack.to_str().unwrap(), "--json"],
    );
    assert_exit(&output, 2);
    let error = assert_failure(
        output.status.code(),
        &parse_envelope(&output.stdout),
        2,
        "validation",
        "DOCLI.TEMPLATE_INVALID",
    );
    let cause = error["details"]["cause"].as_str().expect("cause");
    assert!(cause.contains("template.toml"), "{cause}");
    assert_eq!(error["details"].as_object().unwrap().len(), 1);
    assert!(error["message"]
        .as_str()
        .unwrap()
        .contains("template pack is invalid"));
    let action = error["suggested_action"].as_str().unwrap();
    assert!(action.contains(pack.to_str().unwrap()), "{action}");
    assert!(action.contains("templates validate"), "{action}");
    assert_eq!(
        error["docs"],
        "https://github.com/randlee/docli/blob/develop/docs/requirements.md"
    );
    assert_json_mode_stdout_only_envelope(&output.stdout, &output.stderr);
}

#[test]
fn docli_template_invalid_validate_human() {
    let dir = unique_dir();
    let pack = write_broken_pack(&dir);
    let output = run(
        &mut docli_bin(),
        &["templates", "validate", pack.to_str().unwrap()],
    );
    assert_exit(&output, 2);
    assert_human_actionable(
        &output.stderr,
        "DOCLI.TEMPLATE_INVALID",
        &[
            "template.toml",
            "templates validate",
            pack.to_str().unwrap(),
        ],
    );
}
