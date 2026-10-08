//! Acceptance tests for [`docli::from_clap`] (sprint a.4).

use clap::{Arg, Command};
use docli::from_clap;

fn option<'a>(model: &'a docli::CliModel, name: &str) -> &'a docli::OptionSpec {
    model
        .options
        .iter()
        .find(|o| o.name == name)
        .unwrap_or_else(|| panic!("option {name:?} not found"))
}

fn argument<'a>(model: &'a docli::CliModel, name: &str) -> &'a docli::ArgumentSpec {
    model
        .arguments
        .iter()
        .find(|a| a.name == name)
        .unwrap_or_else(|| panic!("argument {name:?} not found"))
}

#[test]
fn value_parser_yields_choices_on_option() {
    let cmd = Command::new("app").arg(
        Arg::new("format")
            .long("format")
            .value_parser(["json", "yaml"]),
    );
    let model = from_clap(&cmd);
    assert_eq!(option(&model, "format").choices, ["json", "yaml"]);
}

#[test]
fn num_args_yields_min_and_max_values() {
    let cmd = Command::new("app").arg(Arg::new("n").long("n").num_args(2..=3));
    let model = from_clap(&cmd);
    let n = option(&model, "n");
    assert_eq!(n.min_values, Some(2));
    assert_eq!(n.max_values, Some(3));
}

#[test]
fn value_name_maps_from_first_get_value_names_entry() {
    let cmd = Command::new("app").arg(Arg::new("output").long("output").value_name("PATH"));
    let model = from_clap(&cmd);
    assert_eq!(option(&model, "output").value_name.as_deref(), Some("PATH"));
}

#[test]
fn long_help_maps_to_option_long_help() {
    let cmd = Command::new("app").arg(Arg::new("verbose").long("verbose").long_help("more detail"));
    let model = from_clap(&cmd);
    assert_eq!(option(&model, "verbose").long_help, "more detail");
}

#[test]
fn value_parser_yields_choices_on_positional_argument() {
    let cmd = Command::new("app").arg(Arg::new("mode").value_parser(["a", "b"]));
    let model = from_clap(&cmd);
    assert_eq!(argument(&model, "mode").choices, ["a", "b"]);
}

#[test]
fn long_about_and_after_help_map_to_command_fields() {
    let cmd = Command::new("demo").long_about("long").after_help("bye");
    let model = from_clap(&cmd);
    assert_eq!(model.long_description, "long");
    assert_eq!(model.epilogue, "bye");
}

#[test]
fn excludes_auto_injected_help_and_version() {
    let cmd = Command::new("demo");
    let model = from_clap(&cmd);
    assert!(
        model
            .options
            .iter()
            .all(|o| o.name != "help" && o.name != "version"),
        "help/version must not appear as options: {:?}",
        model.options
    );
    assert!(
        model
            .arguments
            .iter()
            .all(|a| a.name != "help" && a.name != "version"),
        "help/version must not appear as arguments: {:?}",
        model.arguments
    );
}

#[test]
fn subcommand_usage_contains_parent_path() {
    let cmd = Command::new("demo").subcommand(Command::new("run"));
    let model = from_clap(&cmd);
    let run = model
        .subcommands
        .iter()
        .find(|s| s.name == "run")
        .expect("subcommand run");
    assert!(
        run.usage.contains("demo run"),
        "usage {:?} should contain demo run",
        run.usage
    );
}

#[test]
fn minimal_command_maps_name() {
    let cmd = Command::new("test");
    let model = from_clap(&cmd);
    assert_eq!(model.name, "test");
}
