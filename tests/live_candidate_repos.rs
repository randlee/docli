//! Live integration against sibling checkouts (not run in default CI).
//!
//! ```bash
//! export DOCLI_LIVE_CANDIDATE_REPOS=1
//! export ATM_CORE_ROOT=$HOME/Documents/github/atm-core
//! export SC_COMPOSE_ROOT=$HOME/Documents/github/sc-compose
//! export SC_OBSERVABILITY_ROOT=$HOME/Documents/github/sc-observability
//! cargo test --test live_candidate_repos -- --nocapture
//! ```

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn require_env(name: &str) -> PathBuf {
    std::env::var(name)
        .unwrap_or_else(|_| panic!("set {name} when DOCLI_LIVE_CANDIDATE_REPOS=1"))
        .into()
}

fn docli_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_docli"))
}

fn assert_envelope_ok(label: &str, output: &Output) {
    let stdout = String::from_utf8_lossy(&output.stdout);
    let envelope: Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|err| {
        panic!(
            "{label}: stdout is not a JSON envelope ({err}); exit={:?}; stderr={}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
        );
    });
    assert_eq!(
        envelope.get("version"),
        Some(&Value::String("1".into())),
        "{label}: unexpected envelope version: {envelope}"
    );
    if envelope.get("ok") == Some(&Value::Bool(true)) {
        return;
    }
    let err = envelope.get("error").cloned().unwrap_or(Value::Null);
    let cause = err
        .get("details")
        .and_then(|d| d.get("cause"))
        .filter(|c| c.is_string())
        .cloned()
        .unwrap_or(Value::Null);
    panic!(
        "{label}: docli envelope ok:false — code={} message={} suggested_action={} cause={} (exit={:?})",
        err.get("code").unwrap_or(&Value::Null),
        err.get("message").unwrap_or(&Value::Null),
        err.get("suggested_action").unwrap_or(&Value::Null),
        cause,
        output.status.code()
    );
}

fn generate_show(workdir: &Path, model: &Path, html_dir: &Path, label: &str) {
    let _ = std::fs::remove_dir_all(html_dir);
    let html_arg = html_dir.to_str().expect("utf8 html dir");
    let generate = Command::new(docli_bin())
        .current_dir(workdir)
        .args([
            "generate",
            "--input",
            model.to_str().expect("utf8 model"),
            "--html",
            html_arg,
            "--json",
        ])
        .output()
        .expect("spawn docli generate");
    assert_envelope_ok(&format!("{label} generate"), &generate);
    assert!(
        html_dir.join("index.html").is_file(),
        "missing {} after generate",
        html_dir.join("index.html").display()
    );
    let show = Command::new(docli_bin())
        .current_dir(workdir)
        .args(["show", "--html", html_arg, "--json"])
        .output()
        .expect("spawn docli show");
    assert_envelope_ok(&format!("{label} show"), &show);
}

fn generate_show_site_cli(workdir: &Path, model: &Path, label: &str) {
    let html = workdir.join("site/cli");
    let _ = std::fs::remove_dir_all(workdir.join("site"));
    generate_show(workdir, model, &html, label);
}

fn generate_show_temp_html(model: &Path, label: &str) {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let html = std::env::temp_dir().join(format!("docli-live-{nanos}"));
    generate_show(&workspace_root(), model, &html, label);
    let _ = std::fs::remove_dir_all(&html);
}

#[test]
fn live_candidate_repos_generate_show() {
    if std::env::var("DOCLI_LIVE_CANDIDATE_REPOS").ok().as_deref() != Some("1") {
        eprintln!("skip live_candidate_repos (set DOCLI_LIVE_CANDIDATE_REPOS=1)");
        return;
    }

    let root = workspace_root();
    let atm = require_env("ATM_CORE_ROOT");
    let compose = require_env("SC_COMPOSE_ROOT");
    let obs = require_env("SC_OBSERVABILITY_ROOT");

    let contract = root.join("fixtures/contract/model.json");
    let atm_json = root.join("fixtures/repos/atm-core.json");
    let compose_json = root.join("fixtures/repos/sc-compose.json");

    assert!(
        !root.join("fixtures/repos/sc-observability.json").exists(),
        "sc-observability has no clap Command; fixture must stay absent"
    );

    generate_show_temp_html(&contract, "docli contract");
    generate_show_temp_html(&atm_json, "docli atm-core.json");
    generate_show_temp_html(&compose_json, "docli sc-compose.json");
    generate_show_site_cli(&atm, &atm_json, "atm-core checkout");
    generate_show_site_cli(&compose, &compose_json, "sc-compose checkout");
    generate_show_site_cli(&obs, &contract, "sc-observability contract smoke");
}
