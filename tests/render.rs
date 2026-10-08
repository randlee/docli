use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "demo", version = "1.0.0", about = "A demo CLI")]
struct Demo {
    /// Increase verbosity
    #[arg(short, long)]
    verbose: bool,
    /// Output path
    #[arg(long, value_name = "PATH")]
    output: Option<String>,
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Run it
    Run {
        /// Dry run, no side effects
        #[arg(long)]
        dry_run: bool,
    },
}

#[test]
fn clap_to_model_maps_the_tree() {
    let model = docli::clap_model::from_clap(&Demo::command());
    assert_eq!(model.name, "demo");
    assert_eq!(model.version.as_deref(), Some("1.0.0"));
    assert_eq!(model.description, "A demo CLI");
    assert_eq!(model.subcommands.len(), 1);
    assert_eq!(model.subcommands[0].name, "run");
    assert!(model.subcommands[0].usage.contains("demo run"));

    // Options: --verbose (short -v) and --output <PATH>, plus the subcommand.
    assert!(model
        .options
        .iter()
        .any(|o| o.long.as_deref() == Some("--verbose")));
    assert!(model
        .options
        .iter()
        .any(|o| o.long.as_deref() == Some("--output") && o.value_name.as_deref() == Some("PATH")));
}

#[test]
fn renders_html_and_markdown() {
    let model = docli::clap_model::from_clap(&Demo::command());

    let html = docli::render::html::render(&model);
    assert!(html.contains("demo CLI Reference"));
    assert!(html.contains("run"));
    assert!(html.contains("--verbose"));

    let md = docli::render::markdown::render(&model);
    assert!(md.contains("## `demo`"));
    assert!(md.contains("### `demo run`"));
    assert!(md.contains("--verbose"));
}

#[test]
fn renders_from_neutral_json() {
    let json = r#"{
        "name": "x",
        "description": "hi",
        "usage": "x [OPTIONS]",
        "options": [{"name": "v", "long": "--verbose", "short": "-v", "help": "verbose"}],
        "arguments": [],
        "subcommands": []
    }"#;
    let model: docli::CliModel = serde_json::from_str(json).unwrap();
    let html = docli::render::html::render(&model);
    assert!(html.contains("--verbose"));
    let md = docli::render::markdown::render(&model);
    assert!(md.contains("--verbose"));
}
