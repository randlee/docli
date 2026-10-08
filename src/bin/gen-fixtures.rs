//! Regenerate `fixtures/repos/*.json` from live clap trees in external checkouts.
//!
//! See `docs/plans/phase-a/a5-repo-fixtures.md`. This binary is behind the
//! `gen-fixtures` feature and is not built during `cargo test`.
//!
//! Each target CLI crate is binary-only (no library), so a sibling `[[bin]]`
//! cannot import its internal `Cli` type. Instead we inject a short-lived
//! `docli-gen-fixtures` module plus unit test, run `cargo test`, read the JSON
//! from a temp file, then restore the checkout.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use clap::Parser;

const FEATURE: &str = "docli-gen-fixtures";

#[derive(Debug, Parser)]
#[command(
    name = "gen-fixtures",
    about = "Capture CliModel JSON from atm-core and sc-compose checkouts"
)]
struct Args {
    /// Path to an atm-core repository checkout.
    #[arg(long)]
    atm_core: PathBuf,
    /// Path to an sc-compose repository checkout.
    #[arg(long)]
    sc_compose: PathBuf,
}

struct Target {
    fixture_file: &'static str,
    package_dir: &'static str,
    package_name: &'static str,
    test_filter: &'static str,
    inject_path: &'static str,
    inject_snippet: &'static str,
}

const TARGETS: &[(&str, Target)] = &[
    (
        "atm-core",
        Target {
            fixture_file: "atm-core.json",
            package_dir: "crates/atm",
            package_name: "agent-team-mail",
            test_filter: "commands::docli_gen_fixtures::dump_model",
            inject_path: "src/commands/mod.rs",
            inject_snippet: r#"

#[cfg(feature = "docli-gen-fixtures")]
mod docli_gen_fixtures {
    use clap::CommandFactory;

    #[test]
    fn dump_model() {
        let model = docli::from_clap(&super::Cli::command());
        let json = serde_json::to_string_pretty(&model).expect("serialize CliModel");
        let out = std::env::var("DOCLI_GEN_OUT").expect("DOCLI_GEN_OUT");
        std::fs::write(out, format!("{json}\n")).expect("write fixture JSON");
    }
}
"#,
        },
    ),
    (
        "sc-compose",
        Target {
            fixture_file: "sc-compose.json",
            package_dir: "crates/sc-compose",
            package_name: "sc-compose",
            test_filter: "cli::docli_gen_fixtures::dump_model",
            inject_path: "src/cli/mod.rs",
            inject_snippet: r#"

#[cfg(feature = "docli-gen-fixtures")]
mod docli_gen_fixtures {
    use clap::CommandFactory;

    #[test]
    fn dump_model() {
        let model = docli::from_clap(&super::Cli::command());
        let json = serde_json::to_string_pretty(&model).expect("serialize CliModel");
        let out = std::env::var("DOCLI_GEN_OUT").expect("DOCLI_GEN_OUT");
        std::fs::write(out, format!("{json}\n")).expect("write fixture JSON");
    }
}
"#,
        },
    ),
];

fn main() {
    if let Err(err) = run() {
        eprintln!("gen-fixtures: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args = Args::parse();
    let docli_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let fixtures_dir = docli_root.join("fixtures/repos");
    fs::create_dir_all(&fixtures_dir).map_err(|err| format!("create fixtures/repos: {err}"))?;

    let checkouts = [("atm-core", args.atm_core), ("sc-compose", args.sc_compose)];

    for (key, checkout) in checkouts {
        let target = TARGETS
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, target)| target)
            .expect("target");
        let json = capture_fixture(&checkout, target, &docli_root)?;
        let out_path = fixtures_dir.join(target.fixture_file);
        fs::write(&out_path, format!("{json}\n"))
            .map_err(|err| format!("write {}: {err}", out_path.display()))?;
        eprintln!("wrote {}", out_path.display());
    }

    Ok(())
}

fn capture_fixture(checkout: &Path, target: &Target, docli_root: &Path) -> Result<String, String> {
    if !checkout.is_dir() {
        return Err(format!(
            "checkout {} is not a directory",
            checkout.display()
        ));
    }

    let package_root = checkout.join(target.package_dir);
    let manifest_path = package_root.join("Cargo.toml");
    let inject_path = package_root.join(target.inject_path);
    if !manifest_path.is_file() {
        return Err(format!(
            "missing package manifest {}",
            manifest_path.display()
        ));
    }
    if !inject_path.is_file() {
        return Err(format!("missing inject path {}", inject_path.display()));
    }

    let docli_path = fs::canonicalize(docli_root)
        .map_err(|err| format!("resolve docli root {}: {err}", docli_root.display()))?;
    let docli_dep = docli_path.to_string_lossy().into_owned();

    let manifest_backup = fs::read_to_string(&manifest_path)
        .map_err(|err| format!("read {}: {err}", manifest_path.display()))?;
    let inject_backup = fs::read_to_string(&inject_path)
        .map_err(|err| format!("read {}: {err}", inject_path.display()))?;

    if inject_backup.contains("mod docli_gen_fixtures") {
        return Err(format!(
            "{} already contains docli_gen_fixtures injection; clean the checkout first",
            inject_path.display()
        ));
    }

    let patched_manifest = patch_manifest(&manifest_backup, &docli_dep)?;
    fs::write(&manifest_path, &patched_manifest)
        .map_err(|err| format!("patch {}: {err}", manifest_path.display()))?;
    if let Err(err) = fs::write(
        &inject_path,
        format!("{inject_backup}{}", target.inject_snippet),
    )
    .map_err(|err| format!("patch {}: {err}", inject_path.display()))
    {
        if let Err(restore_err) = fs::write(&manifest_path, &manifest_backup) {
            return Err(format!(
                "{err}\nfailed to restore {}: {restore_err}",
                manifest_path.display()
            ));
        }
        return Err(err);
    }

    let temp_out = env::temp_dir().join(format!(
        "docli-gen-out-{}-{}.json",
        std::process::id(),
        target.fixture_file.replace('.', "-")
    ));
    let _ = fs::remove_file(&temp_out);

    let output = run_dump_test(checkout, target.package_name, target.test_filter, &temp_out);

    let restore = || {
        fs::write(&manifest_path, &manifest_backup)
            .map_err(|err| format!("restore {}: {err}", manifest_path.display()))?;
        fs::write(&inject_path, &inject_backup)
            .map_err(|err| format!("restore {}: {err}", inject_path.display()))?;
        let _ = fs::remove_file(&temp_out);
        Ok::<(), String>(())
    };

    if let Err(err) = &output {
        return fail_after_restore(restore, err.clone());
    }
    let output = output?;

    if !output.status.success() {
        return fail_after_restore(
            restore,
            format!(
                "cargo test failed for {} in {} (exit {:?}); filter {}; DOCLI_GEN_OUT={}; verify docli path dep and feature docli-gen-fixtures\nstdout:\n{}\nstderr:\n{}",
                target.package_name,
                checkout.display(),
                output.status.code(),
                target.test_filter,
                temp_out.display(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        );
    }

    if !temp_out.is_file() {
        return fail_after_restore(
            restore,
            format!(
                "dump test for {} did not write {}; stdout:\n{}\nstderr:\n{}",
                target.package_name,
                temp_out.display(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            ),
        );
    }

    let json = match fs::read_to_string(&temp_out) {
        Ok(json) => json,
        Err(err) => {
            return fail_after_restore(
                restore,
                format!("read generated JSON at {}: {err}", temp_out.display()),
            );
        }
    };
    restore()?;
    serde_json::from_str::<serde_json::Value>(&json)
        .map_err(|err| format!("generated JSON is invalid: {err}"))?;
    Ok(json.trim_end().to_string())
}

fn fail_after_restore(
    restore: impl FnOnce() -> Result<(), String>,
    err: String,
) -> Result<String, String> {
    match restore() {
        Ok(()) => Err(err),
        Err(restore_err) => Err(format!("{err}\nfailed to restore checkout: {restore_err}")),
    }
}

fn patch_manifest(original: &str, docli_dep: &str) -> Result<String, String> {
    if original.contains("docli-gen-fixtures") || original.contains("dependencies.docli") {
        return Err(
            "refusing to patch manifest that already mentions docli-gen-fixtures or docli".into(),
        );
    }

    let mut patched = original.to_string();
    if !patched.ends_with('\n') {
        patched.push('\n');
    }

    if patched.contains("[features]") {
        patched = patched.replacen("[features]", &format!("[features]\n{FEATURE} = []"), 1);
    } else if !patched.ends_with('\n') {
        patched.push('\n');
        patched.push_str(&format!("\n[features]\n{FEATURE} = []\n"));
    } else {
        patched.push_str(&format!("\n[features]\n{FEATURE} = []\n"));
    }

    patched.push_str(&format!("\n[dependencies.docli]\npath = \"{docli_dep}\"\n"));
    Ok(patched)
}

fn run_dump_test(
    checkout: &Path,
    package_name: &str,
    test_filter: &str,
    out_path: &Path,
) -> Result<Output, String> {
    Command::new(env!("CARGO"))
        .args([
            "test",
            "--quiet",
            "-p",
            package_name,
            "--features",
            FEATURE,
            test_filter,
            "--",
            "--exact",
        ])
        .current_dir(checkout)
        .env("DOCLI_GEN_OUT", out_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|err| format!("spawn cargo test -p {package_name}: {err}"))
}
