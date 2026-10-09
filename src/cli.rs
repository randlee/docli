//! Shared argv parsing and envelope output for `docli` and `cargo-docli`.

use std::path::PathBuf;

use clap::error::ErrorKind as ClapErrorKind;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use serde::Serialize;

use crate::contract::{Envelope, ErrorBody};
use crate::ops::{
    generate, parse_template_selector, parse_theme_json, show, templates_list, templates_show,
    templates_validate, ArtifactReport, GenerateRequest, InputSource, ShowRequest,
    TemplatesListResponse, TemplatesShowResponse, TemplatesValidateResponse,
};

#[derive(Parser)]
#[command(version, about = "Language-agnostic CLI documentation generator")]
pub struct Cli {
    /// Print only the version `"1"` envelope on stdout.
    #[arg(long, global = true)]
    pub json: bool,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Render a neutral CLI model (JSON) to HTML and optional Markdown.
    Generate {
        /// Input CLI-model JSON. Pass `-` to read stdin.
        #[arg(long, value_name = "FILE", default_value = "-")]
        input: String,
        /// Output directory for the self-contained HTML reference (writes <dir>/index.html).
        #[arg(long, value_name = "DIR")]
        html: Option<PathBuf>,
        /// Write index.html under a temporary preview directory instead of --html.
        #[arg(long)]
        preview: bool,
        /// Bundled template id or pack directory.
        #[arg(long, value_name = "ID|PATH")]
        template: Option<String>,
        /// Theme overrides as a JSON object of strings.
        #[arg(long, value_name = "JSON")]
        theme: Option<String>,
        /// Output path for the Markdown reference.
        #[arg(long, value_name = "FILE")]
        markdown: Option<PathBuf>,
    },
    /// Report hashes of previously generated artifacts.
    Show {
        /// Directory that contains `index.html`.
        #[arg(long, value_name = "DIR")]
        html: Option<PathBuf>,
        /// Markdown file to inspect.
        #[arg(long, value_name = "FILE")]
        markdown: Option<PathBuf>,
    },
    /// Discover and validate HTML template packs.
    Templates {
        #[command(subcommand)]
        action: TemplatesAction,
    },
}

/// `templates` subcommand.
#[derive(Subcommand)]
pub enum TemplatesAction {
    /// List the embedded `default` pack and optional extra packs.
    List,
    /// Show one pack's manifest, theme schema, and example generate argv.
    Show {
        /// Bundled id, installed extra id, or pack directory.
        template: String,
    },
    /// Check that a pack directory loads and its templates compile.
    Validate {
        /// Pack directory containing `template.toml`.
        path: PathBuf,
    },
}

/// Run with the clap-visible application name (`docli` or `cargo docli`).
pub fn run_with_name(app_name: &'static str) {
    let cli = match Cli::command().name(app_name).try_get_matches() {
        Ok(matches) => Cli::from_arg_matches(&matches).expect("clap-validated"),
        Err(err) => match err.kind() {
            ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => err.exit(),
            _ => finish(
                json_flag_present(),
                Envelope::<()>::failure(ErrorBody::usage(usage_suggestion(&err))),
                |_| {},
            ),
        },
    };

    match cli.command {
        Command::Generate {
            input,
            html,
            preview,
            template,
            theme,
            markdown,
        } => {
            let template = match template.as_deref() {
                Some(selector) => match parse_template_selector(selector) {
                    Ok(template) => Some(template),
                    Err(error) => finish(cli.json, Envelope::<()>::failure(error), |_| {}),
                },
                None => None,
            };
            let theme = match theme.as_deref() {
                Some(json) => match parse_theme_json(json) {
                    Ok(theme) => Some(theme),
                    Err(error) => finish(cli.json, Envelope::<()>::failure(error), |_| {}),
                },
                None => None,
            };
            let request = GenerateRequest {
                input: if input == "-" {
                    InputSource::Stdin
                } else {
                    InputSource::File(PathBuf::from(input))
                },
                html_dir: html,
                preview,
                template,
                theme,
                markdown,
            };
            let envelope = generate(request);
            finish(cli.json, envelope, |data| print_artifacts(&data.outputs));
        }
        Command::Show { html, markdown } => {
            let envelope = show(ShowRequest {
                html_dir: html,
                markdown,
            });
            finish(cli.json, envelope, |data| print_artifacts(&data.artifacts));
        }
        Command::Templates { action } => match action {
            TemplatesAction::List => {
                finish(cli.json, templates_list(), print_templates_list);
            }
            TemplatesAction::Show { template } => {
                let template = match parse_template_selector(&template) {
                    Ok(template) => template,
                    Err(error) => finish(cli.json, Envelope::<()>::failure(error), |_| {}),
                };
                finish(cli.json, templates_show(template), print_templates_show);
            }
            TemplatesAction::Validate { path } => {
                finish(
                    cli.json,
                    templates_validate(&path),
                    print_templates_validate,
                );
            }
        },
    }
}

fn finish<T, F>(json: bool, envelope: Envelope<T>, human: F) -> !
where
    T: Serialize,
    F: FnOnce(&T),
{
    if json {
        emit_and_exit(&envelope);
    }
    if let Some(error) = &envelope.error {
        eprintln!("{error}");
        std::process::exit(envelope.exit_code());
    }
    if let Some(data) = &envelope.data {
        human(data);
    }
    std::process::exit(envelope.exit_code());
}

fn print_templates_list(data: &TemplatesListResponse) {
    for template in &data.templates {
        println!(
            "{} {} {} {}",
            template.id, template.name, template.version, template.path
        );
    }
}

fn print_templates_show(data: &TemplatesShowResponse) {
    println!("{}", data.id);
    println!("{}", data.example_generate_argv.join(" "));
    for (key, spec) in &data.theme_schema {
        println!("{key} {} {}", spec.value_type, spec.default);
    }
}

fn print_templates_validate(data: &TemplatesValidateResponse) {
    println!("{} {}", data.id, data.path.display());
}

fn print_artifacts(artifacts: &[ArtifactReport]) {
    for artifact in artifacts {
        println!(
            "{} ({} bytes) {}",
            artifact.path.display(),
            artifact.bytes,
            artifact.sha256
        );
    }
}

/// Last-resort JSON when even the internal-error envelope cannot serialize.
const CATASTROPHIC_ENVELOPE_JSON: &str = r#"{"version":"1","ok":false,"data":null,"error":{"kind":"internal","code":"DOCLI.INTERNAL","message":"could not serialize envelope","details":{},"suggested_action":"Retry or report a bug","docs":null}}"#;

fn emit_and_exit<T: Serialize>(envelope: &Envelope<T>) -> ! {
    match serde_json::to_string(envelope) {
        Ok(json) => {
            println!("{json}");
            std::process::exit(envelope.exit_code());
        }
        Err(err) => {
            let fallback = Envelope::<()>::failure(ErrorBody::internal(err.to_string()));
            match serde_json::to_string(&fallback) {
                Ok(json) => println!("{json}"),
                Err(_) => println!("{CATASTROPHIC_ENVELOPE_JSON}"),
            }
            std::process::exit(fallback.exit_code());
        }
    }
}

fn json_flag_present() -> bool {
    std::env::args().any(|arg| arg == "--json")
}

fn usage_suggestion(err: &clap::Error) -> String {
    err.to_string()
        .lines()
        .next()
        .unwrap_or("Use a valid command and flags")
        .trim()
        .trim_start_matches("error: ")
        .to_owned()
}
