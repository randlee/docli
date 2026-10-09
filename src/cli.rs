//! Shared argv parsing and envelope output for `docli` and `cargo-docli`.

use std::path::PathBuf;

use clap::error::ErrorKind as ClapErrorKind;
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand};
use serde::Serialize;

use crate::contract::{Envelope, ErrorBody};
use crate::ops::{
    default_template_install_root, generate, show, templates_list, templates_show,
    templates_validate, ArtifactReport, GenerateRequest, InputSource, ShowRequest,
    TemplatesListRequest, TemplatesListResponse, TemplatesShowRequest, TemplatesShowResponse,
    TemplatesValidateRequest, TemplatesValidateResponse,
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
        action: TemplatesCommand,
    },
}

/// `docli templates` subcommands.
#[derive(Subcommand)]
pub enum TemplatesCommand {
    /// List the embedded default pack and optional packs under the install root.
    List,
    /// Show one pack's manifest, theme schema, and example generate argv.
    Show {
        /// Bundled id (`default`), `embedded:<id>`, or a pack directory.
        #[arg(value_name = "ID|PATH")]
        id_or_path: String,
    },
    /// Validate a pack directory or `embedded:<id>`.
    Validate {
        /// Pack directory or `embedded:<id>`.
        #[arg(value_name = "PATH")]
        path: String,
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
            markdown,
        } => {
            let request = GenerateRequest {
                input: if input == "-" {
                    InputSource::Stdin
                } else {
                    InputSource::File(PathBuf::from(input))
                },
                html_dir: html,
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
            TemplatesCommand::List => {
                let envelope = templates_list(TemplatesListRequest {
                    install_root: default_template_install_root(),
                });
                finish(cli.json, envelope, print_template_list);
            }
            TemplatesCommand::Show { id_or_path } => {
                let envelope = templates_show(TemplatesShowRequest { id_or_path });
                finish(cli.json, envelope, print_template_show);
            }
            TemplatesCommand::Validate { path } => {
                let envelope = templates_validate(TemplatesValidateRequest { path });
                finish(cli.json, envelope, print_template_validate);
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

fn print_template_list(data: &TemplatesListResponse) {
    println!("{}", data.install_root.display());
    for template in &data.templates {
        println!(
            "{} {} {} {}",
            template.id, template.name, template.version, template.path
        );
    }
}

fn print_template_show(data: &TemplatesShowResponse) {
    println!("{}", data.id);
    for (key, spec) in &data.theme_schema {
        println!(
            "{key} {} {} {}",
            spec.value_type, spec.default, spec.description
        );
    }
    println!("{}", data.example_generate_argv.join(" "));
}

fn print_template_validate(data: &TemplatesValidateResponse) {
    println!("{} {} {} {}", data.path, data.id, data.version, data.valid);
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
