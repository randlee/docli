use std::path::PathBuf;

use clap::error::ErrorKind as ClapErrorKind;
use clap::{Parser, Subcommand};
use docli::contract::{Envelope, ErrorBody};
use docli::ops::{generate, show, ArtifactReport, GenerateRequest, InputSource, ShowRequest};
use serde::Serialize;

#[derive(Parser)]
#[command(
    name = "docli",
    version,
    about = "Language-agnostic CLI documentation generator"
)]
struct Cli {
    /// Print only the version `"1"` envelope on stdout.
    #[arg(long, global = true)]
    json: bool,
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
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
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => match err.kind() {
            ClapErrorKind::DisplayHelp | ClapErrorKind::DisplayVersion => err.exit(),
            _ => {
                if json_flag_present() {
                    emit_and_exit(&Envelope::<()>::failure(ErrorBody::usage(
                        usage_suggestion(&err),
                    )));
                }
                err.exit();
            }
        },
    };

    match cli.command {
        Cmd::Generate {
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
            finish(cli.json, envelope, |data| &data.outputs);
        }
        Cmd::Show { html, markdown } => {
            let envelope = show(ShowRequest {
                html_dir: html,
                markdown,
            });
            finish(cli.json, envelope, |data| &data.artifacts);
        }
    }
}

fn finish<T, F>(json: bool, envelope: Envelope<T>, artifacts: F)
where
    T: Serialize,
    F: FnOnce(&T) -> &[ArtifactReport],
{
    if json {
        emit_and_exit(&envelope);
    }
    if let Some(error) = &envelope.error {
        eprintln!("{error}");
        std::process::exit(envelope.exit_code());
    }
    if let Some(data) = &envelope.data {
        print_human(artifacts(data));
    }
    std::process::exit(envelope.exit_code());
}

fn print_human(artifacts: &[ArtifactReport]) {
    for artifact in artifacts {
        println!(
            "{} ({} bytes) {}",
            artifact.path.display(),
            artifact.bytes,
            artifact.sha256
        );
    }
}

fn emit_and_exit<T: Serialize>(envelope: &Envelope<T>) -> ! {
    match serde_json::to_string(envelope) {
        Ok(json) => {
            println!("{json}");
            std::process::exit(envelope.exit_code());
        }
        Err(err) => {
            let fallback = Envelope::<()>::failure(ErrorBody::internal(err.to_string()));
            println!(
                "{}",
                serde_json::to_string(&fallback).unwrap_or_else(|_| {
                    "{\"version\":\"1\",\"ok\":false,\"data\":null,\"error\":null}".to_owned()
                })
            );
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
