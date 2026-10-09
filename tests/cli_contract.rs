#[path = "common/envelope.rs"]
mod envelope;

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use docli::ops::{generate, GenerateRequest, InputSource};
use docli::{ArgumentSpec, CliModel, OptionSpec};
use envelope::{assert_failure, parse_envelope};
use serde_json::Value;
use sha2::{Digest, Sha256};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

const MODEL_JSON: &str = r#"{
  "name": "demo",
  "version": "1.0.0",
  "description": "A demo CLI",
  "long_description": "A longer demo description",
  "epilogue": "Done.",
  "usage": "demo [OPTIONS]",
  "options": [
    {
      "name": "verbose",
      "long": "--verbose",
      "short": "-v",
      "help": "Increase verbosity",
      "long_help": "Print more detail",
      "value_name": null,
      "required": false,
      "default_value": null,
      "choices": [],
      "min_values": null,
      "max_values": null,
      "future": 1
    }
  ],
  "arguments": [
    {
      "name": "path",
      "help": "Input path",
      "required": false,
      "default_value": null,
      "choices": [],
      "future": 1
    }
  ],
  "subcommands": [],
  "future": 1
}"#;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docli"))
}

fn unique_dir() -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docli-a1-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos(),
        id
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn write_model(dir: &Path) -> PathBuf {
    let path = dir.join("model.json");
    fs::write(&path, MODEL_JSON).expect("write model");
    path
}

fn sha256_file(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    sha256_hex(&bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::with_capacity(64), |mut acc, byte| {
            use std::fmt::Write;
            let _ = write!(acc, "{byte:02x}");
            acc
        })
}

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).unwrap_or_else(|err| {
        panic!(
            "stdout is not one JSON envelope ({err}): {}",
            String::from_utf8_lossy(stdout)
        )
    })
}

fn assert_suggested_action_contains(error: &Value, needles: &[&str]) {
    let suggested_action = error["suggested_action"]
        .as_str()
        .expect("suggested_action must be a string");
    for needle in needles {
        assert!(
            suggested_action.contains(needle),
            "suggested_action {suggested_action:?} missing {needle:?}"
        );
    }
}

#[test]
fn generate_json_writes_artifacts_and_matching_hashes() {
    let dir = unique_dir();
    let model = write_model(&dir);
    let html_dir = dir.join("html");
    let markdown = dir.join("out.md");

    let output = bin()
        .args([
            "generate",
            "--input",
            model.to_str().expect("utf8"),
            "--html",
            html_dir.to_str().expect("utf8"),
            "--markdown",
            markdown.to_str().expect("utf8"),
            "--json",
        ])
        .output()
        .expect("run generate");

    assert_eq!(output.status.code(), Some(0));
    let envelope = parse_json(&output.stdout);
    assert_eq!(envelope["version"], "1");
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["error"], Value::Null);
    assert_eq!(envelope["data"]["operation"], "generate");
    assert_eq!(envelope["data"]["model_name"], "demo");
    assert_eq!(
        envelope["data"]["html_dir"],
        html_dir.to_string_lossy().as_ref()
    );

    let outputs = envelope["data"]["outputs"].as_array().expect("outputs");
    assert_eq!(outputs.len(), 2);
    let html_path = html_dir.join("index.html");
    assert_eq!(outputs[0]["kind"], "html");
    assert_eq!(outputs[0]["path"], html_path.to_string_lossy().as_ref());
    assert_eq!(outputs[0]["sha256"], sha256_file(&html_path));
    assert_eq!(outputs[0]["bytes"], fs::metadata(&html_path).unwrap().len());
    assert_eq!(outputs[1]["kind"], "markdown");
    assert_eq!(outputs[1]["path"], markdown.to_string_lossy().as_ref());
    assert_eq!(outputs[1]["sha256"], sha256_file(&markdown));
    assert_eq!(outputs[1]["bytes"], fs::metadata(&markdown).unwrap().len());
}

#[test]
fn generate_missing_input_is_not_found() {
    let dir = unique_dir();
    let missing = dir.join("missing.json");

    let output = bin()
        .args([
            "generate",
            "--input",
            missing.to_str().expect("utf8"),
            "--json",
        ])
        .current_dir(&dir)
        .output()
        .expect("run generate");

    assert_eq!(output.status.code(), Some(3));
    let envelope = parse_envelope(&output.stdout);
    let error = assert_failure(
        output.status.code(),
        &envelope,
        3,
        "not_found",
        "DOCLI.INPUT_NOT_FOUND",
    );
    assert_eq!(
        error["details"],
        serde_json::json!({ "path": missing.to_string_lossy().as_ref() })
    );
    assert_suggested_action_contains(&error, &[missing.to_string_lossy().as_ref()]);
}

#[test]
fn generate_json_defaults_html_dir_to_site_cli() {
    let dir = unique_dir();
    let model = write_model(&dir);

    let output = bin()
        .args([
            "generate",
            "--input",
            model.to_str().expect("utf8"),
            "--json",
        ])
        .current_dir(&dir)
        .output()
        .expect("run generate");

    assert_eq!(output.status.code(), Some(0));
    let envelope = parse_json(&output.stdout);
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["error"], Value::Null);
    assert_eq!(envelope["data"]["html_dir"], "site/cli");
    assert_eq!(envelope["data"]["preview_dir"], Value::Null);
    assert!(dir.join("site/cli/index.html").is_file());
}

#[test]
fn generate_json_reads_stdin_when_input_omitted() {
    let dir = unique_dir();
    let mut child = bin()
        .args(["generate", "--json"])
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn generate");
    child
        .stdin
        .take()
        .expect("stdin")
        .write_all(MODEL_JSON.as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait generate");

    assert_eq!(output.status.code(), Some(0));
    let envelope = parse_json(&output.stdout);
    assert_eq!(envelope["ok"], true);
    assert_eq!(envelope["data"]["input"], "stdin");
}

#[test]
fn show_json_without_paths_is_usage() {
    let output = bin().args(["show", "--json"]).output().expect("run show");

    assert_eq!(output.status.code(), Some(2));
    let envelope = parse_envelope(&output.stdout);
    let error = assert_failure(
        output.status.code(),
        &envelope,
        2,
        "validation",
        "DOCLI.USAGE",
    );
    assert_eq!(error["details"], serde_json::json!({}));
    assert_suggested_action_contains(&error, &["--html", "--markdown"]);
}

#[test]
fn human_show_without_paths_is_usage() {
    let output = bin().args(["show"]).output().expect("run show");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("DOCLI.USAGE"), "stderr was {stderr}");
    assert!(stderr.contains("--html"), "stderr was {stderr}");
    assert!(stderr.contains("--markdown"), "stderr was {stderr}");
}

#[test]
fn human_generate_and_show_print_path_bytes_and_hash() {
    let dir = unique_dir();
    let model = write_model(&dir);
    let html_dir = dir.join("html");
    let markdown = dir.join("out.md");

    let generate = bin()
        .args([
            "generate",
            "--input",
            model.to_str().expect("utf8"),
            "--html",
            html_dir.to_str().expect("utf8"),
            "--markdown",
            markdown.to_str().expect("utf8"),
        ])
        .output()
        .expect("run generate");
    assert_eq!(generate.status.code(), Some(0));
    let html_path = html_dir.join("index.html");
    let html_bytes = fs::metadata(&html_path).unwrap().len();
    let html_hash = sha256_file(&html_path);
    let stdout = String::from_utf8_lossy(&generate.stdout);
    assert!(stdout.contains(&html_path.display().to_string()));
    assert!(stdout.contains(&html_bytes.to_string()));
    assert!(stdout.contains(&html_hash));

    let show = bin()
        .args([
            "show",
            "--html",
            html_dir.to_str().expect("utf8"),
            "--markdown",
            markdown.to_str().expect("utf8"),
        ])
        .output()
        .expect("run show");
    assert_eq!(show.status.code(), Some(0));
    let show_out = String::from_utf8_lossy(&show.stdout);
    assert!(show_out.contains(&html_path.display().to_string()));
    assert!(show_out.contains(&html_bytes.to_string()));
    assert!(show_out.contains(&html_hash));
}

#[test]
fn show_json_hashes_match_generate() {
    let dir = unique_dir();
    let model = write_model(&dir);
    let html_dir = dir.join("html");
    let markdown = dir.join("out.md");

    let generate = bin()
        .args([
            "generate",
            "--input",
            model.to_str().expect("utf8"),
            "--html",
            html_dir.to_str().expect("utf8"),
            "--markdown",
            markdown.to_str().expect("utf8"),
            "--json",
        ])
        .output()
        .expect("run generate");
    assert_eq!(generate.status.code(), Some(0));
    let generated = parse_json(&generate.stdout);

    let show = bin()
        .args([
            "show",
            "--html",
            html_dir.to_str().expect("utf8"),
            "--markdown",
            markdown.to_str().expect("utf8"),
            "--json",
        ])
        .output()
        .expect("run show");
    assert_eq!(show.status.code(), Some(0));
    let shown = parse_json(&show.stdout);
    assert_eq!(shown["ok"], true);
    assert_eq!(shown["error"], Value::Null);
    assert_eq!(shown["data"]["operation"], "show");
    assert_eq!(
        shown["data"]["artifacts"][0]["sha256"],
        generated["data"]["outputs"][0]["sha256"]
    );
    assert_eq!(
        shown["data"]["artifacts"][1]["sha256"],
        generated["data"]["outputs"][1]["sha256"]
    );
}

#[test]
fn invalid_model_json_is_input_invalid() {
    let dir = unique_dir();
    let model = dir.join("bad.json");
    fs::write(&model, "{\"name\": 1}").expect("write bad model");

    let output = bin()
        .args([
            "generate",
            "--input",
            model.to_str().expect("utf8"),
            "--json",
        ])
        .current_dir(&dir)
        .output()
        .expect("run generate");

    assert_eq!(output.status.code(), Some(2));
    let envelope = parse_envelope(&output.stdout);
    let error = assert_failure(
        output.status.code(),
        &envelope,
        2,
        "validation",
        "DOCLI.INPUT_INVALID",
    );
    let cause = error["details"]["cause"]
        .as_str()
        .expect("details.cause must be a string");
    assert_eq!(
        error["details"].as_object().expect("details object").len(),
        1
    );
    assert_suggested_action_contains(&error, &[model.to_string_lossy().as_ref(), cause]);
}

#[test]
fn show_missing_artifact_is_output_not_found() {
    let dir = unique_dir();
    let missing = dir.join("missing-html");

    let output = bin()
        .args(["show", "--html", missing.to_str().expect("utf8"), "--json"])
        .output()
        .expect("run show");

    assert_eq!(output.status.code(), Some(3));
    let envelope = parse_envelope(&output.stdout);
    let error = assert_failure(
        output.status.code(),
        &envelope,
        3,
        "not_found",
        "DOCLI.OUTPUT_NOT_FOUND",
    );
    let missing_html = missing.join("index.html");
    assert_eq!(
        error["details"],
        serde_json::json!({
            "artifacts": [{
                "path": missing_html.to_string_lossy().as_ref(),
                "exists": false
            }]
        })
    );
    assert_suggested_action_contains(&error, &[missing_html.to_string_lossy().as_ref()]);
}

#[test]
fn ops_generate_writes_index_html_in_tmp() {
    let dir = unique_dir();
    let model = write_model(&dir);
    let tmp = dir.join("tmp");

    let envelope = generate(GenerateRequest {
        input: InputSource::File(model),
        html_dir: Some(tmp.clone()),
        preview: false,
        template: None,
        theme_json: None,
        markdown: None,
    });

    assert!(envelope.ok);
    assert!(envelope.error.is_none());
    let data = envelope.data.expect("success data");
    assert_eq!(data.html_dir, tmp);
    assert!(data.preview_dir.is_none());
    assert!(tmp.join("index.html").is_file());
}

#[test]
fn help_and_version_exit_zero() {
    let help = bin().arg("--help").output().expect("help");
    assert_eq!(help.status.code(), Some(0));
    assert!(!help.stdout.is_empty());
    let help_text = String::from_utf8_lossy(&help.stdout);
    assert!(!help_text.contains("DOCLI.USAGE"));
    assert!(help_text.contains("Usage") || help_text.contains("Usage:"));

    let version = bin().arg("--version").output().expect("version");
    assert_eq!(version.status.code(), Some(0));
    assert!(!version.stdout.is_empty());
    let version_text = String::from_utf8_lossy(&version.stdout);
    assert!(!version_text.contains("DOCLI.USAGE"));
}

#[test]
fn invalid_flag_is_usage_with_and_without_json() {
    let json = bin()
        .args(["generate", "--not-a-flag", "--json"])
        .output()
        .expect("run generate --not-a-flag --json");
    assert_eq!(json.status.code(), Some(2));
    let envelope = parse_envelope(&json.stdout);
    let error = assert_failure(
        json.status.code(),
        &envelope,
        2,
        "validation",
        "DOCLI.USAGE",
    );
    assert_eq!(error["details"], serde_json::json!({}));
    assert_suggested_action_contains(&error, &["--not-a-flag"]);

    let human = bin()
        .args(["generate", "--not-a-flag"])
        .output()
        .expect("run generate --not-a-flag");
    assert_eq!(human.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&human.stderr);
    assert!(stderr.contains("DOCLI.USAGE"), "stderr was {stderr}");
    assert!(stderr.contains("--not-a-flag"), "stderr was {stderr}");
}

#[test]
fn unknown_command_is_usage_with_and_without_json() {
    let json = bin()
        .args(["not-a-command", "--json"])
        .output()
        .expect("run not-a-command --json");
    assert_eq!(json.status.code(), Some(2));
    let envelope = parse_envelope(&json.stdout);
    let error = assert_failure(
        json.status.code(),
        &envelope,
        2,
        "validation",
        "DOCLI.USAGE",
    );
    assert_eq!(error["details"], serde_json::json!({}));
    assert_suggested_action_contains(&error, &["not-a-command"]);

    let human = bin()
        .args(["not-a-command"])
        .output()
        .expect("run not-a-command");
    assert_eq!(human.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&human.stderr);
    assert!(stderr.contains("DOCLI.USAGE"), "stderr was {stderr}");
    assert!(stderr.contains("not-a-command"), "stderr was {stderr}");
}

#[test]
fn human_input_invalid_includes_suggested_action_and_cause() {
    let dir = unique_dir();
    let model = dir.join("bad.json");
    fs::write(&model, "{\"name\": 1}").expect("write bad model");

    let output = bin()
        .args(["generate", "--input", model.to_str().expect("utf8")])
        .current_dir(&dir)
        .output()
        .expect("run generate");

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("DOCLI.INPUT_INVALID"),
        "stderr was {stderr}"
    );
    assert!(
        stderr.contains(model.to_string_lossy().as_ref()),
        "stderr was {stderr}"
    );
    assert!(stderr.contains("expected"), "stderr was {stderr}");
}

#[test]
fn unknown_fields_stay_in_extra_and_known_fields_render() {
    let model: CliModel = serde_json::from_str(MODEL_JSON).expect("parse model");
    assert_eq!(model.extra.get("future"), Some(&Value::from(1)));
    assert_eq!(model.options[0].extra.get("future"), Some(&Value::from(1)));
    assert_eq!(
        model.arguments[0].extra.get("future"),
        Some(&Value::from(1))
    );

    let html = docli::render::html::render(&model);
    assert!(html.contains("demo"));
    assert!(html.contains("--verbose"));
    assert!(html.contains("Input path"));
}

#[test]
fn structs_expose_input_model_fields() {
    let option = OptionSpec {
        name: "verbose".into(),
        long: Some("--verbose".into()),
        short: Some("-v".into()),
        help: "help".into(),
        long_help: "long".into(),
        value_name: Some("VAL".into()),
        required: true,
        default_value: Some("off".into()),
        choices: vec!["on".into(), "off".into()],
        min_values: Some(1),
        max_values: Some(1),
        extra: serde_json::Map::new(),
    };
    let argument = ArgumentSpec {
        name: "path".into(),
        help: "file".into(),
        required: false,
        default_value: Some(".".into()),
        choices: vec![".".into()],
        extra: serde_json::Map::new(),
    };
    let model = CliModel {
        name: "demo".into(),
        version: Some("1".into()),
        description: "short".into(),
        long_description: "long".into(),
        epilogue: "end".into(),
        usage: "demo".into(),
        options: vec![option.clone()],
        arguments: vec![argument.clone()],
        subcommands: vec![],
        extra: serde_json::Map::new(),
    };

    assert_eq!(model.name, "demo");
    assert_eq!(model.version.as_deref(), Some("1"));
    assert_eq!(model.description, "short");
    assert_eq!(model.long_description, "long");
    assert_eq!(model.epilogue, "end");
    assert_eq!(model.usage, "demo");
    assert_eq!(option.name, "verbose");
    assert_eq!(option.long.as_deref(), Some("--verbose"));
    assert_eq!(option.short.as_deref(), Some("-v"));
    assert_eq!(option.help, "help");
    assert_eq!(option.long_help, "long");
    assert_eq!(option.value_name.as_deref(), Some("VAL"));
    assert!(option.required);
    assert_eq!(option.default_value.as_deref(), Some("off"));
    assert_eq!(option.choices, ["on", "off"]);
    assert_eq!(option.min_values, Some(1));
    assert_eq!(option.max_values, Some(1));
    assert_eq!(argument.name, "path");
    assert_eq!(argument.help, "file");
    assert!(!argument.required);
    assert_eq!(argument.default_value.as_deref(), Some("."));
    assert_eq!(argument.choices, ["."]);
}

#[test]
fn license_is_mit() {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .expect("cargo metadata");
    assert!(output.status.success());
    let metadata: Value = serde_json::from_slice(&output.stdout).expect("metadata json");
    let license = metadata["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .find(|pkg| pkg["name"] == "docli")
        .expect("docli package")["license"]
        .as_str()
        .expect("license string");
    assert_eq!(license, "MIT");

    let license_text = fs::read_to_string("LICENSE").expect("LICENSE");
    assert!(license_text.starts_with("MIT License"));
}

#[test]
fn error_kind_and_code_serialize_to_contract_strings() {
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorKind::Validation).unwrap(),
        "\"validation\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorKind::NotFound).unwrap(),
        "\"not_found\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorKind::Dependency).unwrap(),
        "\"dependency\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorKind::Internal).unwrap(),
        "\"internal\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorCode::Usage).unwrap(),
        "\"DOCLI.USAGE\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorCode::InputInvalid).unwrap(),
        "\"DOCLI.INPUT_INVALID\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorCode::InputNotFound).unwrap(),
        "\"DOCLI.INPUT_NOT_FOUND\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorCode::OutputNotFound).unwrap(),
        "\"DOCLI.OUTPUT_NOT_FOUND\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorCode::Io).unwrap(),
        "\"DOCLI.IO\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorCode::TemplateInvalid).unwrap(),
        "\"DOCLI.TEMPLATE_INVALID\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorCode::TemplateNotFound).unwrap(),
        "\"DOCLI.TEMPLATE_NOT_FOUND\""
    );
    assert_eq!(
        serde_json::to_string(&docli::contract::ErrorCode::Internal).unwrap(),
        "\"DOCLI.INTERNAL\""
    );
}

#[test]
fn envelope_always_serializes_data_error_and_docs() {
    let src = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/contract.rs"));
    assert!(
        !src.contains("skip_serializing_if"),
        "skip_serializing_if is forbidden on data, error, and docs"
    );

    let success = serde_json::to_value(docli::contract::Envelope::success("ok")).unwrap();
    assert_eq!(success["data"], "ok");
    assert_eq!(success["error"], Value::Null);

    let failure = serde_json::to_value(docli::contract::Envelope::<()>::failure(
        docli::contract::ErrorBody::usage("Pass --html DIR and/or --markdown FILE"),
    ))
    .unwrap();
    assert_eq!(failure["data"], Value::Null);
    assert_eq!(failure["error"]["docs"], Value::Null);
}
