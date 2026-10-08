//! `cargo docli` entry point — same argv surface as the `docli` binary.

use docli::cli;

fn main() {
    cli::run_with_name("cargo docli");
}
