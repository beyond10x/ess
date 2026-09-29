//! Renders the `ess` command reference from a clap command tree.
//!
//! This file is compiled twice: into `ess-xtask`'s tests, which hold it to a fixture tree, and
//! through a `#[path]` module into the `ess` binary's own tests, which is the only place its real
//! command tree can be built. It therefore depends on `clap` and the standard library alone.

use std::fmt::Write as _;

/// The Markdown between the reference page's markers, for `command` and everything below it.
///
/// Every command that is not hidden gets a section when it is a leaf or takes arguments of its
/// own: its full invocation as clap prints it, its help text and one row per visible argument.
/// Subcommands and arguments keep the order clap lists them in. Hidden commands, hidden aliases
/// and hidden arguments are left out, as `--help` leaves them out.
pub(crate) fn render(command: &clap::Command) -> String {
    let mut root = command.clone();
    root.build();
    let name = root.get_name().to_owned();
    let mut out = String::new();
    let _ = writeln!(
        out,
        "This section is generated from the `{name}` command definition by \
         `cargo xtask cli-reference`. Change the command's help text and regenerate rather than \
         editing it here.\n"
    );
    let globals: Vec<&clap::Arg> = visible_arguments(&root)
        .into_iter()
        .filter(|argument| argument.is_global_set())
        .collect();
    if !globals.is_empty() {
        let _ = writeln!(
            out,
            "### Global options\n\nAccepted by every `{name}` command, in any position.\n"
        );
        table(&mut out, &globals);
    }
    for child in visible_subcommands(&root) {
        section(&mut out, child, 3);
    }
    out
}

/// One command's section, then its subcommands' sections one heading level down (at most `####`).
fn section(out: &mut String, command: &clap::Command, level: usize) {
    let arguments: Vec<&clap::Arg> = visible_arguments(command)
        .into_iter()
        .filter(|argument| !argument.is_global_set())
        .collect();
    let children = visible_subcommands(command);
    if level == 3 || children.is_empty() || !arguments.is_empty() {
        let path = command.get_bin_name().unwrap_or_else(|| command.get_name());
        let _ = writeln!(out, "{} `{path}`\n", "#".repeat(level));
        if let Some(text) = command.get_long_about().or_else(|| command.get_about()) {
            let _ = writeln!(out, "{}\n", prose(&text.to_string()));
        }
        let aliases: Vec<&str> = command.get_visible_aliases().collect();
        if !aliases.is_empty() {
            let _ = writeln!(out, "Also spelled {}.\n", code_list(&aliases));
        }
        if children.is_empty() || !arguments.is_empty() {
            let usage = command.clone().render_usage().to_string();
            let usage = usage.strip_prefix("Usage: ").unwrap_or(&usage);
            let lines: Vec<&str> = usage.lines().map(str::trim).collect();
            let _ = writeln!(out, "```text\n{}\n```\n", lines.join("\n"));
            if arguments.is_empty() {
                let _ = writeln!(out, "No arguments beyond the global options.\n");
            } else {
                table(out, &arguments);
            }
        }
    }
    for child in children {
        section(out, child, 4);
    }
}

fn visible_subcommands(command: &clap::Command) -> Vec<&clap::Command> {
    // The `help` subcommand clap adds when building is how `--help` is spelled, not a command.
    let generated_help = !command.is_disable_help_subcommand_set();
    let mut children: Vec<&clap::Command> = command
        .get_subcommands()
        .filter(|child| !child.is_hide_set())
        .filter(|child| !(generated_help && child.get_name() == "help"))
        .collect();
    // Stable, so declaration order breaks ties exactly as `--help` does.
    children.sort_by_key(|child| child.get_display_order());
    children
}

/// Positional arguments in index order, then options in the order `--help` prints them.
fn visible_arguments(command: &clap::Command) -> Vec<&clap::Arg> {
    let mut arguments: Vec<&clap::Arg> = command
        .get_arguments()
        .filter(|argument| !argument.is_hide_set())
        .filter(|argument| {
            !matches!(
                argument.get_action(),
                clap::ArgAction::Help
                    | clap::ArgAction::HelpShort
                    | clap::ArgAction::HelpLong
                    | clap::ArgAction::Version
            )
        })
        .collect();
    arguments.sort_by_key(|argument| {
        (
            !argument.is_positional(),
            argument.get_index().unwrap_or(usize::MAX),
            argument.get_display_order(),
        )
    });
    arguments
}

fn table(out: &mut String, arguments: &[&clap::Arg]) {
    out.push_str("| Argument | Value | Required | Default | Description |\n");
    out.push_str("|---|---|---|---|---|\n");
    for argument in arguments {
        let takes_values = argument.get_action().takes_values();
        let value_name = argument
            .get_value_names()
            .and_then(|names| names.first())
            .map_or_else(
                || argument.get_id().as_str().to_owned(),
                ToString::to_string,
            );
        let repeated = matches!(argument.get_action(), clap::ArgAction::Append)
            || argument
                .get_num_args()
                .is_some_and(|range| range.max_values() > 1);
        let ellipsis = if repeated { "…" } else { "" };
        let name = if argument.is_positional() {
            format!("`<{value_name}>`{ellipsis}")
        } else {
            let mut spellings = Vec::new();
            if let Some(short) = argument.get_short() {
                spellings.push(format!("`-{short}`"));
            }
            if let Some(long) = argument.get_long() {
                spellings.push(format!("`--{long}`"));
            }
            for alias in argument.get_visible_aliases().unwrap_or_default() {
                spellings.push(format!("`--{alias}`"));
            }
            spellings.join(", ")
        };
        let value = if takes_values && !argument.is_positional() {
            format!("`<{value_name}>`{ellipsis}")
        } else {
            String::new()
        };
        let required = if argument.is_required_set() {
            "yes"
        } else {
            "no"
        };
        let default = if takes_values {
            let defaults: Vec<String> = argument
                .get_default_values()
                .iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect();
            code_list(&defaults)
        } else {
            String::new()
        };
        let mut description = argument
            .get_long_help()
            .or_else(|| argument.get_help())
            .map(|help| cell(&help.to_string()))
            .unwrap_or_default();
        if takes_values {
            let choices: Vec<String> = argument
                .get_possible_values()
                .into_iter()
                .filter(|choice| !choice.is_hide_set())
                .map(|choice| choice.get_name().to_owned())
                .collect();
            if !choices.is_empty() {
                if !description.is_empty() {
                    if !description.ends_with(['.', ':', '?', '!']) {
                        description.push('.');
                    }
                    description.push(' ');
                }
                let _ = write!(description, "One of {}.", code_list(&choices));
            }
        }
        let _ = writeln!(
            out,
            "| {name} | {value} | {required} | {default} | {description} |"
        );
    }
    out.push('\n');
}

/// `a`, `b` and `c` as code spans separated by commas.
fn code_list<S: AsRef<str>>(items: &[S]) -> String {
    items
        .iter()
        .map(|item| format!("`{}`", item.as_ref().replace('|', "\\|")))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Help text as a Markdown table cell: paragraphs joined by line breaks, `|` escaped.
fn cell(text: &str) -> String {
    let paragraphs: Vec<String> = text
        .trim()
        .split("\n\n")
        .map(|paragraph| paragraph.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();
    prose(&paragraphs.join("<br /><br />")).replace('|', "\\|")
}

/// Help text made safe for the site's MDX: outside code spans, `{`, `}`, `<` and `>` are literal.
///
/// The `<br />` line breaks [`cell`] inserts are the only markup let through.
fn prose(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut in_code = false;
    let mut rest = text;
    while let Some(character) = rest.chars().next() {
        if !in_code && rest.starts_with("<br />") {
            out.push_str("<br />");
            rest = &rest["<br />".len()..];
            continue;
        }
        match character {
            '`' => {
                in_code = !in_code;
                out.push('`');
            }
            '{' if !in_code => out.push_str("\\{"),
            '}' if !in_code => out.push_str("\\}"),
            '<' if !in_code => out.push_str("&lt;"),
            '>' if !in_code => out.push_str("&gt;"),
            other => out.push(other),
        }
        rest = &rest[character.len_utf8()..];
    }
    out.trim_end().to_owned()
}
