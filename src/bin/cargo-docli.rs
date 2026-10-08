//! `cargo docli` entry point — same argv surface as the `docli` binary.

fn main() {
    docli::cli::run_with_name("cargo docli");
}
