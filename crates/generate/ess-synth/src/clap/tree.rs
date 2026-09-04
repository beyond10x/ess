//! The emitted files: the command tree, the handler seam, and the binary that joins them.
//!
//! Everything here is derived from one declaration. A component's `cli:` block says which word each
//! command sits under; `naming.wire` says what the command's own word is, verbatim and un-cased,
//! exactly as `ess-gen`'s `OpenAPI` projection reads it — a generator inventing its own kebab-casing
//! would disagree with every other projection about what a command is called.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use ess_compiler::ir::{
    EssIr, ResolvedBody, ResolvedCommand, ResolvedCommandLineSurface, ResolvedComponent,
    ResolvedField, ResolvedTypeRef,
};
use ess_domain::name::QualifiedName;
use ess_gen::Provenance;

use super::layout::Layout;

/// One component that declared a command-line surface, with its tree resolved.
pub(crate) struct Surface<'a> {
    /// The component that declared it.
    pub component: &'a ResolvedComponent,
    /// What it declared.
    pub cli: &'a ResolvedCommandLineSurface,
    /// Every command the tree places, in the order the tree places them.
    pub commands: Vec<&'a ResolvedCommand>,
}

/// Every command-line surface the specification declares, in component order.
pub(crate) fn surfaces(ir: &EssIr) -> Vec<Surface<'_>> {
    ir.components()
        .values()
        .filter_map(|component| {
            let cli = component.cli.as_ref()?;
            let mut commands: Vec<&ResolvedCommand> = cli
                .commands
                .iter()
                .map(|handle| ir.command(handle))
                .collect();
            for group in &cli.groups {
                commands.extend(group.commands.iter().map(|handle| ir.command(handle)));
            }
            Some(Surface {
                component,
                cli,
                commands,
            })
        })
        .collect()
}

/// The word a command is typed as.
fn word(command: &ResolvedCommand) -> &str {
    command.naming.wire_or(&command.name)
}

/// The method a handler implements for a command, as a Rust identifier.
///
/// Derived from the qualified name rather than from the typed word, so two commands that share a
/// word under different groups do not share a method.
fn method(name: &QualifiedName) -> String {
    let mut out = String::new();
    let mut previous_lower = false;
    for segment in name.to_string().split('.') {
        if !out.is_empty() {
            out.push('_');
            // A segment boundary is already a separator, so the first capital after it must not
            // add a second one: `gatepass.visit.RegisterVisit` is one underscore between segments,
            // not two.
            previous_lower = false;
        }
        for character in segment.chars() {
            if character.is_ascii_uppercase() {
                if previous_lower {
                    out.push('_');
                }
                out.push(character.to_ascii_lowercase());
                previous_lower = false;
            } else if character == '-' {
                out.push('_');
                previous_lower = false;
            } else {
                out.push(character);
                previous_lower = character.is_ascii_lowercase() || character.is_ascii_digit();
            }
        }
    }
    out
}

/// The closed set of values a field accepts, where the model declares one.
///
/// An enum, or a newtype over one. Everything else completes as free text: a shell cannot enumerate
/// a `String`, and offering a guess would complete values the system refuses.
fn value_set(ir: &EssIr, type_ref: &ResolvedTypeRef) -> Option<Vec<String>> {
    match type_ref {
        ResolvedTypeRef::Optional { of } | ResolvedTypeRef::List { of } => value_set(ir, of),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Enum { variants } => Some(variants.clone()),
            ResolvedBody::Newtype { of, .. } => value_set(ir, of),
            _ => None,
        },
        _ => None,
    }
}

/// `true` where a field may be left out.
fn optional(type_ref: &ResolvedTypeRef) -> bool {
    matches!(type_ref, ResolvedTypeRef::Optional { .. })
}

/// `true` where a field is a list, so its flag repeats.
fn repeatable(type_ref: &ResolvedTypeRef) -> bool {
    matches!(type_ref, ResolvedTypeRef::List { .. })
}

/// One `clap::Arg`, as source.
///
/// The flag's word is `naming.wire` where the field declares one, and the field's own name
/// otherwise — verbatim, the way `ess-gen`'s `OpenAPI` projection reads it. A generator inventing
/// its own hyphenation would disagree with every other projection about what this field is called,
/// so a specification wanting `--expected-stay` writes that as the field's `naming.wire`.
fn argument(ir: &EssIr, field: &ResolvedField, indent: &str) -> String {
    let name = field.naming.wire.as_deref().unwrap_or(&field.name);
    let mut out = format!(
        "{indent}.arg(\n{indent}    ::clap::Arg::new({name:?})\n{indent}        .long({name:?})",
    );
    if let Some(summary) = &field.naming.summary {
        let _ = write!(out, "\n{indent}        .help({summary:?})");
    }
    if repeatable(&field.type_ref) {
        let _ = write!(out, "\n{indent}        .action(::clap::ArgAction::Append)");
    }
    if !optional(&field.type_ref) && !repeatable(&field.type_ref) {
        let _ = write!(out, "\n{indent}        .required(true)");
    }
    if let Some(values) = value_set(ir, &field.type_ref) {
        // The whole point of putting the tree in the model: a shell completes the *values* a field
        // accepts, not merely the word in front of them.
        let _ = write!(
            out,
            "\n{indent}        .value_parser(\n{indent}            \
             ::clap::builder::PossibleValuesParser::new({values:?}),\n{indent}        )",
        );
    }
    let _ = write!(out, ",\n{indent})");
    out
}

/// One command, as source.
fn command_source(ir: &EssIr, command: &ResolvedCommand, indent: &str) -> String {
    let mut out = format!(
        "{indent}.subcommand(\n{indent}    ::clap::Command::new({:?})",
        word(command)
    );
    if let Some(summary) = &command.naming.summary {
        let _ = write!(out, "\n{indent}        .about({summary:?})");
    }
    for field in &command.input {
        let _ = write!(
            out,
            "\n{}",
            argument(ir, field, &format!("{indent}        "))
        );
    }
    let _ = write!(out, ",\n{indent})");
    out
}

/// One view, as a verb that reads it.
///
/// A view declares no input, so the verb takes no flags: what it carries is the row the view
/// projects, and reading it is the whole act. Emitted at all because a projection served with no
/// way to read it is one nobody can reach — the defect `validate_command_line_views` refuses.
fn view_source(ir: &EssIr, view: &ess_compiler::ir::ViewHandle, indent: &str) -> String {
    let view = ir.view(view);
    let mut out = format!(
        "{indent}.subcommand(\n{indent}    ::clap::Command::new({:?})",
        view.naming.wire_or(&view.name)
    );
    if let Some(summary) = &view.naming.summary {
        let _ = write!(out, "\n{indent}        .about({summary:?})");
    }
    let _ = write!(out, ",\n{indent})");
    out
}

/// The emitted `tree` module: one function returning the whole grammar.
pub(crate) fn tree_module(ir: &EssIr, surfaces: &[Surface<'_>], provenance: &Provenance) -> String {
    let mut out = provenance.commented_for("//", "cargo xtask synth --target clap");
    out.push_str(
        "\n\n//! The command tree, as the specification declares it.\n\n\
         /// The whole grammar: every group, every command, and every flag a command's input \
         declares.\n\
         #[must_use]\n\
         pub fn command() -> ::clap::Command {\n",
    );
    let Some(surface) = surfaces.first() else {
        out.push_str(
            "    // No component declares `reached_by: command_line`, so there is no tree.\n\
             \x20   ::clap::Command::new(\"unnamed\")\n}\n",
        );
        return out;
    };
    let _ = write!(
        out,
        "    ::clap::Command::new({:?})\n        .subcommand_required(true)\n        \
         .arg_required_else_help(true)",
        surface.cli.binary.as_str()
    );
    if let Some(summary) = &surface.component.naming.summary {
        let _ = write!(out, "\n        .about({summary:?})");
    }
    for handle in &surface.cli.commands {
        let _ = write!(
            out,
            "\n{}",
            command_source(ir, ir.command(handle), "        ")
        );
    }
    for handle in &surface.cli.views {
        let _ = write!(out, "\n{}", view_source(ir, handle, "        "));
    }
    for group in &surface.cli.groups {
        let _ = write!(
            out,
            "\n        .subcommand(\n            ::clap::Command::new({:?})\n                \
             .subcommand_required(true)\n                .arg_required_else_help(true)",
            group.name.as_str()
        );
        if let Some(summary) = &group.summary {
            let _ = write!(out, "\n                .about({summary:?})");
        }
        for handle in &group.commands {
            let _ = write!(
                out,
                "\n{}",
                command_source(ir, ir.command(handle), "                ")
            );
        }
        for handle in &group.views {
            let _ = write!(out, "\n{}", view_source(ir, handle, "                "));
        }
        out.push_str(",\n        )");
    }
    out.push_str(
        "\n        .subcommand(\n            ::clap::Command::new(\"completions\")\n                \
         .about(\"Print a completion script for one shell, from this same command tree\")\n         \
         \x20      .arg(\n                    ::clap::Arg::new(\"shell\")\n                        \
         .required(true)\n                        .value_parser(\n                            \
         ::clap::builder::EnumValueParser::<::clap_complete::Shell>::new(),\n                     \
         \x20  ),\n                ),\n        )\n}\n",
    );
    out
}

/// Everything the tree places, as the identity and one-line summary a handler method carries.
///
/// Commands and views together: both are words somebody types, and both are behaviour nothing here
/// decides. Ordered by the tree rather than by name, so the emitted trait reads in the order the
/// `--help` output does.
fn owed<'a>(ir: &'a EssIr, surfaces: &[Surface<'a>]) -> Vec<(QualifiedName, Option<String>)> {
    let mut owed = Vec::new();
    for surface in surfaces {
        let mut take_command = |handle: &_| {
            let command = ir.command(handle);
            owed.push((command.name.clone(), command.naming.summary.clone()));
        };
        for handle in &surface.cli.commands {
            take_command(handle);
        }
        for group in &surface.cli.groups {
            for handle in &group.commands {
                take_command(handle);
            }
        }
        let mut take_view = |handle: &_| {
            let view = ir.view(handle);
            owed.push((view.name.clone(), view.naming.summary.clone()));
        };
        for handle in &surface.cli.views {
            take_view(handle);
        }
        for group in &surface.cli.groups {
            for handle in &group.views {
                take_view(handle);
            }
        }
    }
    owed
}

/// The emitted `handler` module: the seam, and nothing that decides anything.
pub(crate) fn handler_module(
    ir: &EssIr,
    surfaces: &[Surface<'_>],
    provenance: &Provenance,
) -> String {
    let mut out = provenance.commented_for("//", "cargo xtask synth --target clap");
    out.push_str(
        "\n\n//! What is owed: one method per command the tree places.\n//!\n\
         //! A method receives `clap::ArgMatches` rather than the command's declared input type. \
         The\n//! Rust target already emits every input as a type, and a fourth rendering of the \
         type layer\n//! would be a fourth thing to keep in step — so this target emits the \
         grammar and leaves the\n//! types where they are. `TARGET.md` states that as a weakening \
         rather than leaving it to be\n//! discovered.\n\n\
         /// What a command does, once somebody decides.\n\
         pub trait Handler {\n",
    );
    let mut seen = BTreeSet::new();
    for (qualified, summary) in owed(ir, surfaces) {
        let name = method(&qualified);
        if !seen.insert(name.clone()) {
            continue;
        }
        let _ = write!(out, "    /// `{qualified}`");
        if let Some(summary) = &summary {
            let _ = write!(out, " — {summary}");
        }
        let _ = writeln!(
            out,
            "\n    fn {name}(&self, matches: &::clap::ArgMatches) -> ::std::process::ExitCode;"
        );
    }
    out.push_str(
        "}\n\n/// A handler that decides nothing, and says which obligation is owed.\n///\n\
         /// The honest empty state: the emitted binary parses, completes and refuses. A refusal \
         that\n/// names the command is the one a reader learns the plan from.\n\
         pub struct Unimplemented;\n\n\
         impl Handler for Unimplemented {\n",
    );
    let mut seen = BTreeSet::new();
    for (qualified, _) in owed(ir, surfaces) {
        let name = method(&qualified);
        if !seen.insert(name.clone()) {
            continue;
        }
        let _ = writeln!(
            out,
            "    fn {name}(&self, _: &::clap::ArgMatches) -> ::std::process::ExitCode {{\n        \
             eprintln!(\"`{qualified}` is an obligation nothing has implemented\");\n        \
             ::std::process::ExitCode::FAILURE\n    }}",
        );
    }
    out.push_str("}\n");
    out
}

/// The emitted `main`: parse, complete, dispatch.
pub(crate) fn main_module(ir: &EssIr, surfaces: &[Surface<'_>], provenance: &Provenance) -> String {
    let mut out = provenance.commented_for("//", "cargo xtask synth --target clap");
    out.push_str(
        "\n\n//! The binary: parse the tree, answer `completions` from it, dispatch the rest.\n\n\
         mod handler;\nmod tree;\n\n\
         pub use self::handler::{Handler, Unimplemented};\n\n\
         fn main() -> ::std::process::ExitCode {\n    \
         let matches = self::tree::command().get_matches();\n    \
         if let Some(completions) = matches.subcommand_matches(\"completions\") {\n        \
         let shell = *completions\n            \
         .get_one::<::clap_complete::Shell>(\"shell\")\n            \
         .expect(\"the shell is required\");\n        \
         let mut command = self::tree::command();\n        \
         let name = command.get_name().to_owned();\n        \
         ::clap_complete::generate(shell, &mut command, name, &mut ::std::io::stdout());\n        \
         return ::std::process::ExitCode::SUCCESS;\n    }\n    \
         dispatch(&Unimplemented, &matches)\n}\n\n\
         /// Routes one parsed invocation to the handler that owes it.\n\
         ///\n\
         /// Exhaustive over the tree by construction: every arm is a command the `cli:` block \
         places,\n/// and a command it places nowhere is a specification `ess validate` refuses.\n\
         fn dispatch<H: Handler>(handler: &H, matches: &::clap::ArgMatches) \
         -> ::std::process::ExitCode {\n    \
         match matches.subcommand() {\n",
    );
    for surface in surfaces {
        for handle in &surface.cli.commands {
            let command = ir.command(handle);
            let _ = writeln!(
                out,
                "        Some(({:?}, arguments)) => handler.{}(arguments),",
                word(command),
                method(&command.name)
            );
        }
        for handle in &surface.cli.views {
            let view = ir.view(handle);
            let _ = writeln!(
                out,
                "        Some(({:?}, arguments)) => handler.{}(arguments),",
                view.naming.wire_or(&view.name),
                method(&view.name)
            );
        }
        for group in &surface.cli.groups {
            let _ = writeln!(
                out,
                "        Some(({:?}, sub)) => match sub.subcommand() {{",
                group.name.as_str()
            );
            for handle in &group.commands {
                let command = ir.command(handle);
                let _ = writeln!(
                    out,
                    "            Some(({:?}, arguments)) => handler.{}(arguments),",
                    word(command),
                    method(&command.name)
                );
            }
            for handle in &group.views {
                let view = ir.view(handle);
                let _ = writeln!(
                    out,
                    "            Some(({:?}, arguments)) => handler.{}(arguments),",
                    view.naming.wire_or(&view.name),
                    method(&view.name)
                );
            }
            out.push_str("            _ => ::std::process::ExitCode::FAILURE,\n        },\n");
        }
    }
    out.push_str(
        "        Some((\"completions\", _)) => ::std::process::ExitCode::SUCCESS,\n        \
         _ => ::std::process::ExitCode::FAILURE,\n    }\n}\n",
    );
    out
}

/// The emitted crate's manifest.
///
/// The `[[bin]]` name is the declared `binary:`, not the package name. Without it Cargo names the
/// executable after the crate, and the word an operator actually types would be `gatepass-cli`
/// where the specification says `gatepass` — a surface that disagrees with the document it was
/// generated from, in the one place a person can see.
pub(crate) fn manifest(
    layout: &Layout,
    surfaces: &[Surface<'_>],
    provenance: &Provenance,
) -> String {
    let mut out = format!(
        "{}\n\n[package]\nname = \"{}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = \
         false\n",
        provenance.commented_for("#", "cargo xtask synth --target clap"),
        layout.package(),
    );
    if let Some(surface) = surfaces.first() {
        let _ = write!(
            out,
            "\n[[bin]]\nname = \"{}\"\npath = \"src/main.rs\"\n",
            surface.cli.binary.as_str()
        );
    }
    out.push_str("\n[dependencies]\nclap = \"4\"\nclap_complete = \"4\"\n");
    out
}
