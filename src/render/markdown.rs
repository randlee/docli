//! Markdown renderer: a flat, human-readable CLI reference shipped with the
//! installer and readable offline.

use crate::schema::CliModel;

pub fn render(model: &CliModel) -> String {
    let mut out = format!("# {} CLI Reference\n\n", model.name);
    if let Some(version) = &model.version {
        out.push_str(&format!("Version {version}\n\n"));
    }
    out.push_str("Generated from the CLI command tree. Do not hand-edit.\n\n");
    render_command(model, 0, &model.name, &mut out);
    out
}

fn render_command(command: &CliModel, depth: usize, full_name: &str, out: &mut String) {
    let level = (depth + 2).min(6);
    out.push_str(&format!("{} `{full_name}`\n\n", "#".repeat(level)));

    let description = if command.long_description.is_empty() {
        command.description.as_str()
    } else {
        command.long_description.as_str()
    };
    if !description.is_empty() {
        out.push_str(description);
        out.push_str("\n\n");
    }

    if !command.usage.is_empty() {
        out.push_str("```text\n");
        out.push_str(&command.usage);
        out.push_str("\n```\n\n");
    }

    if !command.arguments.is_empty() {
        out.push_str("**Arguments:**\n\n");
        for arg in &command.arguments {
            out.push_str(&format!("- `{}`", arg.name));
            if arg.required {
                out.push_str(" *(required)*");
            }
            if let Some(default) = &arg.default_value {
                out.push_str(&format!(" — default: `{default}`"));
            }
            if !arg.choices.is_empty() {
                out.push_str(&format!(" — choices: `{}`", arg.choices.join(", ")));
            }
            if !arg.help.is_empty() {
                out.push_str(&format!(" — {help}", help = arg.help));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    if !command.options.is_empty() {
        out.push_str("**Options:**\n\n");
        for option in &command.options {
            let mut flag = String::new();
            if let Some(long) = &option.long {
                flag.push_str(long);
            }
            if let Some(short) = &option.short {
                if !flag.is_empty() {
                    flag.push_str(", ");
                }
                flag.push_str(short);
            }
            if let Some(value_name) = &option.value_name {
                flag.push_str(&format!(" <{value_name}>"));
            }
            out.push_str(&format!("- `{flag}`"));
            if option.required {
                out.push_str(" *(required)*");
            }
            if let Some(default) = &option.default_value {
                out.push_str(&format!(" — default: `{default}`"));
            }
            if !option.choices.is_empty() {
                out.push_str(&format!(" — choices: `{}`", option.choices.join(", ")));
            }
            let text = if option.long_help.is_empty() {
                &option.help
            } else {
                &option.long_help
            };
            if !text.is_empty() {
                out.push_str(&format!(" — {text}"));
            }
            out.push('\n');
        }
        out.push('\n');
    }

    if !command.epilogue.is_empty() {
        out.push_str(&command.epilogue);
        out.push_str("\n\n");
    }

    for sub in &command.subcommands {
        render_command(sub, depth + 1, &format!("{full_name} {}", sub.name), out);
    }
}
