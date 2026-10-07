use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "docli",
    version,
    about = "Language-agnostic CLI documentation generator"
)]
struct Cli {
    #[command(subcommand)]
    command: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Render a neutral CLI model (JSON) to HTML and/or Markdown.
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
}

fn read_input(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    if input == "-" {
        let mut s = String::new();
        std::io::stdin().read_to_string(&mut s)?;
        Ok(s)
    } else {
        Ok(std::fs::read_to_string(input)?)
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cli { command } = Cli::parse();
    match command {
        Cmd::Generate {
            input,
            html,
            markdown,
        } => {
            let json = read_input(&input)?;
            let model: docli::CliModel = serde_json::from_str(&json)?;

            match (html, markdown) {
                (Some(dir), Some(path)) => {
                    std::fs::create_dir_all(&dir)?;
                    std::fs::write(dir.join("index.html"), docli::render::html::render(&model))?;
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, docli::render::markdown::render(&model))?;
                }
                (Some(dir), None) => {
                    std::fs::create_dir_all(&dir)?;
                    std::fs::write(dir.join("index.html"), docli::render::html::render(&model))?;
                }
                (None, Some(path)) => {
                    if let Some(parent) = path.parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                    std::fs::write(&path, docli::render::markdown::render(&model))?;
                }
                (None, None) => print!("{}", docli::render::html::render(&model)),
            }
            Ok(())
        }
    }
}
