#![allow(dead_code)] // shared across integration test binaries; each crate uses a subset.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

pub fn unique_dir() -> PathBuf {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "docli-test-{}-{}-{}",
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

pub fn docli_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_docli"))
}

pub fn cargo_docli_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_cargo-docli"))
}

pub fn run(bin: &mut Command, args: &[&str]) -> Output {
    bin.args(args)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("run binary")
}

pub fn workspace_fixture(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(rel)
}

pub const DEMO_MODEL_JSON: &str = r#"{
  "name": "demo",
  "version": "1.0.0",
  "description": "A demo CLI",
  "long_description": "",
  "epilogue": "",
  "usage": "demo [OPTIONS]",
  "options": [],
  "arguments": [],
  "subcommands": []
}"#;

pub fn write_demo_model(dir: &Path) -> PathBuf {
    let path = dir.join("model.json");
    fs::write(&path, DEMO_MODEL_JSON).expect("write model");
    path
}
