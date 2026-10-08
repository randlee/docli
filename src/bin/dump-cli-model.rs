//! Write this crate's clap CLI as pretty-printed `CliModel` JSON on stdout.

use std::io::{self, Write};

use clap::CommandFactory;
use docli::cli::Cli;

fn main() {
    if let Err(err) = run() {
        eprintln!("dump-cli-model: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let model = docli::from_clap(&Cli::command());
    let json = serde_json::to_string_pretty(&model).map_err(|err| err.to_string())?;
    let mut out = io::stdout().lock();
    out.write_all(json.as_bytes())
        .and_then(|_| out.write_all(b"\n"))
        .map_err(|err| err.to_string())
}
