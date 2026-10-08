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
use std::process::Command;

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

fn generate_show(workdir: &Path, model: &Path) {
    let html = workdir.join("site/cli");
    let _ = std::fs::remove_dir_all(workdir.join("site"));
    let status = Command::new(docli_bin())
        .current_dir(workdir)
        .args([
            "generate",
            "--input",
            model.to_str().expect("utf8 model"),
            "--html",
            "site/cli",
            "--json",
        ])
        .status()
        .expect("spawn docli generate");
    assert!(status.success(), "generate failed in {}", workdir.display());
    assert!(
        html.join("index.html").is_file(),
        "missing {} after generate",
        html.join("index.html").display()
    );
    let show = Command::new(docli_bin())
        .current_dir(workdir)
        .args(["show", "--html", "site/cli", "--json"])
        .status()
        .expect("spawn docli show");
    assert!(show.success(), "show failed in {}", workdir.display());
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

    generate_show(&root, &contract);
    generate_show(&root, &atm_json);
    generate_show(&root, &compose_json);
    generate_show(&atm, &atm_json);
    generate_show(&compose, &compose_json);
    generate_show(&obs, &contract);
}
