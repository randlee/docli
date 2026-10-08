use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

use serde_json::Value;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

const MINIMAL_MODEL: &str = "tests/fixtures/minimal-model.json";
const MISSING_INPUT: &str = "tests/fixtures/does-not-exist.json";

fn docli_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docli"))
}

fn cargo_docli_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cargo-docli"))
}

fn unique_dir() -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docli-a3-{}-{}-{}",
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

fn run(bin: &mut Command, args: &[&str]) -> Output {
    bin.args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run binary")
}

fn assert_parity(args: &[&str]) {
    let docli = run(&mut docli_bin(), args);
    let cargo = run(&mut cargo_docli_bin(), args);

    assert_eq!(
        docli.status.code(),
        cargo.status.code(),
        "exit code mismatch for args {args:?}\ndocli stderr: {}\ncargo-docli stderr: {}",
        String::from_utf8_lossy(&docli.stderr),
        String::from_utf8_lossy(&cargo.stderr)
    );
    assert_eq!(
        docli.stdout,
        cargo.stdout,
        "stdout mismatch for args {args:?}\ndocli: {}\ncargo-docli: {}",
        String::from_utf8_lossy(&docli.stdout),
        String::from_utf8_lossy(&cargo.stdout)
    );
}

fn parse_envelope(stdout: &[u8]) -> Value {
    serde_json::from_slice(stdout).unwrap_or_else(|err| {
        panic!(
            "stdout is not JSON ({err}): {}",
            String::from_utf8_lossy(stdout)
        )
    })
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

#[test]
fn generate_json_stdout_matches_docli() {
    let html_dir = unique_dir();
    let args = [
        "generate",
        "--input",
        MINIMAL_MODEL,
        "--html",
        html_dir.to_str().expect("utf8"),
        "--json",
    ];
    assert_parity(&args);
}

#[test]
fn show_json_stdout_matches_docli() {
    let html_dir = unique_dir();
    let generate_args = [
        "generate",
        "--input",
        MINIMAL_MODEL,
        "--html",
        html_dir.to_str().expect("utf8"),
        "--json",
    ];
    let gen = run(&mut docli_bin(), &generate_args);
    assert_eq!(gen.status.code(), Some(0), "setup generate failed");

    let show_args = ["show", "--html", html_dir.to_str().expect("utf8"), "--json"];
    assert_parity(&show_args);
}

#[test]
fn missing_input_json_matches_docli() {
    let missing = workspace_root().join(MISSING_INPUT);
    let missing = missing.to_str().expect("utf8");
    let args = ["generate", "--input", missing, "--json"];

    let docli = run(&mut docli_bin(), &args);
    let cargo = run(&mut cargo_docli_bin(), &args);

    assert_eq!(docli.status.code(), Some(3));
    assert_eq!(cargo.status.code(), Some(3));
    assert_eq!(docli.stdout, cargo.stdout);

    for stdout in [&docli.stdout, &cargo.stdout] {
        let envelope = parse_envelope(stdout);
        assert_eq!(envelope["ok"], false);
        assert_eq!(envelope["error"]["code"], "DOCLI.INPUT_NOT_FOUND");
    }
}

#[test]
fn help_lists_generate_and_show() {
    let output = run(&mut cargo_docli_bin(), &["--help"]);
    assert_eq!(output.status.code(), Some(0));
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("generate"), "help missing generate:\n{help}");
    assert!(help.contains("show"), "help missing show:\n{help}");
}
