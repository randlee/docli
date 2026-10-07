//! Adapter that builds a [`CliModel`] from a live [`clap::Command`] tree.
//!
//! This is how a Rust CLI produces the neutral input: walk
//! `clap::CommandFactory::command()` and map it onto the framework-agnostic
//! model. Auto-injected `--help`/`--version` plumbing is excluded.

use crate::schema::{ArgumentSpec, CliModel, OptionSpec};
use clap::{Arg, Command};

/// Build a [`CliModel`] from a [`clap::Command`] (typically
/// `Cli::command()` from a `#[derive(Parser)]` type).
pub fn from_clap(command: &Command) -> CliModel {
    from_clap_path(command, command.get_name())
}

fn from_clap_path(command: &Command, full_name: &str) -> CliModel {
    let mut options = Vec::new();
    let mut arguments = Vec::new();

    for arg in command.get_arguments().filter(|a| !is_auto_injected(a)) {
        let long = arg.get_long().map(|l| format!("--{l}"));
        let short = arg.get_short().map(|c| format!("-{c}"));
        let help = arg.get_help().map(ToString::to_string).unwrap_or_default();
        let long_help = arg
            .get_long_help()
            .map(ToString::to_string)
            .unwrap_or_default();
        let value_name = arg
            .get_value_names()
            .and_then(|names| names.first())
            .map(ToString::to_string);
        let default_value = arg
            .get_default_values()
            .first()
            .map(|v| v.to_string_lossy().to_string());
        let choices: Vec<String> = arg
            .get_possible_values()
            .into_iter()
            .map(|v| v.get_name().to_string())
            .collect();
        let required = arg.is_required_set();
        let num_args = arg.get_num_args();

        // A clap arg with neither long nor short form is positional.
        if long.is_none() && short.is_none() {
            arguments.push(ArgumentSpec {
                name: arg.get_id().as_str().to_string(),
                help,
                required,
                default_value,
                choices,
            });
        } else {
            options.push(OptionSpec {
                name: arg.get_id().as_str().to_string(),
                long,
                short,
                help,
                long_help,
                value_name,
                required,
                default_value,
                choices,
                min_values: num_args.map(|r| r.min_values()),
                max_values: num_args.map(|r| r.max_values()),
            });
        }
    }

    options.sort_by(|a, b| a.name.cmp(&b.name));
    arguments.sort_by(|a, b| a.name.cmp(&b.name));

    let mut subcommands: Vec<CliModel> = command
        .get_subcommands()
        .filter(|c| !c.is_hide_set())
        .map(|c| from_clap_path(c, &format!("{full_name} {}", c.get_name())))
        .collect();
    subcommands.sort_by(|a, b| a.name.cmp(&b.name));

    CliModel {
        name: command.get_name().to_string(),
        version: command.get_version().map(ToString::to_string),
        description: command
            .get_about()
            .map(ToString::to_string)
            .unwrap_or_default(),
        long_description: command
            .get_long_about()
            .map(ToString::to_string)
            .unwrap_or_default(),
        epilogue: command
            .get_after_help()
            .map(ToString::to_string)
            .unwrap_or_default(),
        // Force the full command path into the usage line (a subcommand's own
        // `render_usage` would otherwise drop the parent, e.g. `run` vs
        // `demo run`).
        usage: command
            .clone()
            .bin_name(full_name)
            .render_usage()
            .to_string(),
        options,
        arguments,
        subcommands,
    }
}

fn is_auto_injected(arg: &Arg) -> bool {
    matches!(arg.get_id().as_str(), "help" | "version")
}
