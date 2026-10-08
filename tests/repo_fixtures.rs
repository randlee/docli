use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use docli::CliModel;
use serde_json::Value;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

const FIXTURES: &[&str] = &[
    "fixtures/repos/atm-core.json",
    "fixtures/repos/sc-compose.json",
];

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docli"))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn unique_site_cli_dir() -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docli-a5-{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos(),
        id
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir.join("site/cli")
}

fn run(args: &[&str]) -> Output {
    bin()
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run docli")
}

fn parse_json(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).unwrap_or_else(|err| {
        panic!(
            "stdout is not JSON ({err}): {}",
            String::from_utf8_lossy(stdout)
        )
    })
}

fn load_model(path: &Path) -> CliModel {
    let json = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read fixture {}: {err}", path.display()));
    serde_json::from_str(&json)
        .unwrap_or_else(|err| panic!("parse fixture {} as CliModel: {err}", path.display()))
}

#[test]
fn repo_fixtures_parse_as_cli_model() {
    for rel in FIXTURES {
        let path = workspace_root().join(rel);
        let model = load_model(&path);
        assert!(
            !model.name.is_empty(),
            "fixture {} must have a non-empty name",
            path.display()
        );
    }
}

#[test]
fn repo_fixtures_generate_and_show_match_hashes() {
    for rel in FIXTURES {
        let input = workspace_root().join(rel);
        let html_dir = unique_site_cli_dir();
        let html_dir_str = html_dir.to_str().expect("utf8 html dir");

        let generate = run(&[
            "generate",
            "--input",
            input.to_str().expect("utf8 input"),
            "--html",
            html_dir_str,
            "--json",
        ]);
        assert_eq!(
            generate.status.code(),
            Some(0),
            "generate failed for {}:\nstderr: {}",
            rel,
            String::from_utf8_lossy(&generate.stderr)
        );

        let index_html = html_dir.join("index.html");
        assert!(
            index_html.is_file(),
            "expected {} after generate --html {}",
            index_html.display(),
            html_dir_str
        );

        let generated = parse_json(&generate.stdout);
        assert_eq!(generated["ok"], true);
        assert_eq!(generated["error"], Value::Null);
        let outputs = generated["data"]["outputs"]
            .as_array()
            .expect("generate outputs array");
        assert_eq!(outputs.len(), 1, "html-only generate has one output");
        assert_eq!(outputs[0]["kind"], "html");

        let show = run(&["show", "--html", html_dir_str, "--json"]);
        assert_eq!(
            show.status.code(),
            Some(0),
            "show failed for {}:\nstderr: {}",
            rel,
            String::from_utf8_lossy(&show.stderr)
        );

        let shown = parse_json(&show.stdout);
        assert_eq!(shown["ok"], true);
        assert_eq!(shown["error"], Value::Null);
        let artifacts = shown["data"]["artifacts"]
            .as_array()
            .expect("show artifacts array");
        assert_eq!(artifacts.len(), 1);
        assert_eq!(
            artifacts[0]["sha256"], outputs[0]["sha256"],
            "show hash must match generate for {rel}"
        );
    }
}

#[test]
fn sc_observability_repo_fixture_is_absent() {
    let path = workspace_root().join("fixtures/repos/sc-observability.json");
    assert!(
        !path.exists(),
        "sc-observability has no clap Command; fixture must not exist"
    );
}
