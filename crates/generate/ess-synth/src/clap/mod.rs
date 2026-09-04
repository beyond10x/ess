//! The clap emitter: the fourth target behind the plan seam, and the first one a person types at.
//!
//! # Why a fourth target
//!
//! Rust proved the plan could be emitted; Go proved it was language-neutral; the web target proved
//! a person could drive it. All three answer *what does this system do*. None answers the question
//! an operator asks at a shell — *what can I run, and what may I type after it* — because that
//! answer is a **grammar**, and a grammar is the one surface none of the three emits.
//!
//! A command line is not a fourth language. It is the surface a component declares when it says
//! [`Reach::CommandLine`](ess_domain::component::Reach::CommandLine), and its shape comes from the
//! `cli:` block the same component writes: which word each command sits under, and which words are
//! typed at the top level. Everything else here is derived — a command's own word from its
//! `naming.wire`, its flags from its `input:`, its exit conditions from its `outcomes:`.
//!
//! # What it emits, and what it deliberately does not
//!
//! It emits the **grammar and the completion of it**: a `clap` command tree, one flag per declared
//! input field, a closed value set for every enum-typed one so a shell completes the *values* and
//! not merely the verbs, and a `completions` verb that writes a script for each shell `clap`
//! supports.
//!
//! It does not emit the type layer. The Rust target already emits every input, outcome, event and
//! error as a type, and emitting them again in a fourth shape would be a fourth thing to keep in
//! step. So a handler here receives `clap::ArgMatches` rather than a generated input struct, and
//! that is a weakening this target states in `TARGET.md` rather than one a reader discovers.
//!
//! # It never chooses a realization
//!
//! Every command's behaviour stays an obligation. What this target emits is the seam: a `Handler`
//! trait with one method per command, and a dispatcher that parses, routes and returns the exit
//! code. With nothing implemented the binary parses, completes and refuses, naming what is owed.

mod layout;
mod tree;

use std::collections::BTreeSet;

use ess_compiler::ir::EssIr;
use ess_gen::Artifact;

use crate::plan::{Capability, CapabilityKind, SynthesisPlan};
use crate::{TargetRefusal, TargetReport, TargetWeakening};

use self::layout::Layout;

/// The name this target reports itself under.
pub const TARGET: &str = "clap";

/// Everything one emission produced.
pub struct Emission {
    /// Every file, in path order.
    pub artifacts: Vec<Artifact>,
    /// What this target could not carry across from the plan.
    pub report: TargetReport,
}

/// Plans nothing and emits the command tree every command-line component declares.
///
/// A specification with no such component emits a crate with no verbs and says so in its report,
/// rather than inventing a surface nobody declared.
pub fn workspace(ir: &EssIr, plan: &SynthesisPlan) -> Emission {
    let layout = Layout::of(ir);
    let surfaces = tree::surfaces(ir);
    let provenance = &plan.provenance;

    let mut artifacts = vec![
        Artifact::new(
            layout.manifest(),
            tree::manifest(&layout, &surfaces, provenance),
        ),
        Artifact::new(
            layout.source("main"),
            tree::main_module(ir, &surfaces, provenance),
        ),
        Artifact::new(
            layout.source("tree"),
            tree::tree_module(ir, &surfaces, provenance),
        ),
        Artifact::new(
            layout.source("handler"),
            tree::handler_module(ir, &surfaces, provenance),
        ),
    ];
    artifacts.sort_by(|left, right| left.path.cmp(&right.path));

    Emission {
        report: report(ir, &surfaces, plan),
        artifacts,
    }
}

/// What the grammar cannot carry, said once per rule rather than once per command.
fn report(ir: &EssIr, surfaces: &[tree::Surface<'_>], plan: &SynthesisPlan) -> TargetReport {
    let mut weakenings = Vec::new();
    let mut refusals = Vec::new();

    if !surfaces.is_empty() {
        weakenings.push(TargetWeakening {
            guarantee: "a command's input arrives as its declared type".to_owned(),
            instead: "a handler receives `clap::ArgMatches`. The Rust target already emits every \
                      input as a type, and a fourth rendering of the type layer would be a fourth \
                      thing to keep in step — so this target emits the grammar and leaves the \
                      types where they are."
                .to_owned(),
            affects: vec![CapabilityKind::CommandContract],
        });
        weakenings.push(TargetWeakening {
            guarantee: "a shell completes every value a flag accepts".to_owned(),
            instead: "an enum-typed field completes its whole closed set; every other field \
                      completes as free text. A shell cannot enumerate a `String`, and offering a \
                      guess would complete values the system refuses."
                .to_owned(),
            affects: vec![CapabilityKind::CommandContract],
        });
    }

    // A command a component accepts and no tree places cannot happen — `validate_command_line`
    // refuses that specification — so anything missing here is a command no command-line component
    // accepts at all, which is a fact about the system rather than a gap in this target.
    let placed: BTreeSet<&ess_domain::name::QualifiedName> = surfaces
        .iter()
        .flat_map(|surface| surface.commands.iter().map(|command| &command.name))
        .collect();
    for command in ir.commands().values() {
        if !placed.contains(&command.name) {
            refusals.push(TargetRefusal {
                capability: Capability {
                    kind: CapabilityKind::CommandContract,
                    source: command.name.to_string(),
                },
                detail: "no component declaring `reached_by: command_line` accepts this command, \
                         so no tree places it and there is no word to type"
                    .to_owned(),
            });
        }
    }

    TargetReport {
        provenance: plan.provenance.clone(),
        target: TARGET,
        weakenings,
        refusals,
    }
}
