//! The system-crate emitter: the bindings, and the one transport the specification requires.
//!
//! # The transport is derived, not chosen
//!
//! A binding declares how many times its command may run — `at_least_once` or `at_most_once` —
//! and the component surfaces declare who publishes what and who accepts what. What that
//! determines, and all it determines, is an **in-process dispatch, once per occurrence**: every
//! event a component publishes lands on an append-only log, and a pump delivers each logged event
//! to every binding that reacts to it, invoking the accepting component's port. The log doubles as
//! the system's observable record. No broker, no wire format, no second transport, and no
//! abstraction over transports that do not exist.
//!
//! One attempt per reacting binding per occurrence is what `at_most_once` requires and what
//! `at_least_once` permits, so the two words select the same dispatch. Delivery is tracked per
//! binding: the pump passes an occurrence once every reacting binding has had its attempt, and it
//! attempts the occurrence again only for the one binding that asked, out of that binding's own
//! held-back list — `on_failure: retry` after a declared refusal, or an `at_least_once` binding
//! whose attempt an unmet obligation stopped ([`crate::plan::attempts_again`]). The pump never
//! delivers an occurrence again to a binding that already ran, so an `at_most_once` binding runs
//! once per occurrence unless its own `retry` puts the attempt back. `redeliver`, which a caller
//! invokes, runs every `at_least_once` binding again and no `at_most_once` one.
//! [`crate::plan::declares_single_attempt`] decides the package doc's sentence about the model and
//! whether `redeliver` needs a dispatch that leaves `at_most_once` bindings out, not a second
//! transport.
//!
//! # Failure, per the binding's own words
//!
//! What a binding's `on_failure:` speaks about is the invoked command **refusing** — taking a
//! declared outcome that carries an error, which for billing is `SendEmail` answering `failed`
//! with `Undeliverable`. That is the failure the declared policy answers: `escalate` builds the
//! declared event through the escalation obligation and publishes it, `retry` holds the event for
//! the next pump (which is the failure policy's own redelivery, on the schedule the caller
//! provides, and not the delivery guarantee's),
//! `drop` gives up silently because that is what the author wrote. The pump therefore matches on
//! the outcome enum and takes the policy on exactly the error-carrying variants.
//!
//! An unmet obligation is deliberately **not** routed into the policy. A port refusing because
//! its behaviour is owed is a fact about the workspace being unfinished, not a fact about a
//! delivery, and escalating it would publish a domain event for a defect no provider caused —
//! manufactured evidence, in the vocabulary this repository uses for it. It propagates out of
//! `pump` instead, naming what is owed.
//!
//! # The transport records what its bindings invoke
//!
//! Beside the log, the pump keeps a second observable record: every command a binding invoked,
//! with the input it passed, as a typed `BindingInvocation`. A mapping's target is a command
//! input, and the invocation is the only observation attributed to the binding that filled it —
//! so without this record, "the binding filled `recipient` from the field the document names" is
//! a claim nothing can check. A conformance run reads it; nothing inside the system does.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use ess_compiler::ir::{EssIr, EventHandle, ResolvedBinding, ResolvedComponent, ResolvedFailure};
use ess_gen::{Artifact, Provenance};

use crate::plan::{
    accepting_components, Capability, CapabilityKind, DeterminedInput, SynthesisPlan, REGENERATE,
};

use super::layout::Layout;
use super::port::types_path;
use super::{event_variants, name, EDITION};

/// The system crate: manifest and module, emitted whenever the specification declares an
/// interaction layer at all — a component or a binding.
pub(super) fn system_crate(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    covered: &mut BTreeSet<Capability>,
    stubbed: &mut BTreeSet<Capability>,
) -> Vec<Artifact> {
    if ir.components().is_empty() && ir.bindings().is_empty() {
        return Vec::new();
    }
    vec![
        manifest(ir, layout, &plan.provenance),
        lib_module(ir, plan, layout, covered, stubbed),
    ]
}

/// The system crate's manifest: the types crate and every component crate, by path.
fn manifest(ir: &EssIr, layout: &Layout, provenance: &Provenance) -> Artifact {
    let package = layout.system_package();
    let mut out = provenance.commented_for("#", REGENERATE);
    let _ = write!(
        out,
        "\n[package]\nname = \"{package}\"\ndescription = \"The `{}` system, {}: its bindings \
         and its one transport, generated.\"\nversion = \"{}.0.0\"\nedition = \
         \"{EDITION}\"\n\n[dependencies]\n{} = {{ path = \"../{}\" }}\n",
        ir.system(),
        ir.version(),
        ir.version().get(),
        layout.package(),
        layout.package(),
    );
    for component in ir.components().keys() {
        let dependency = layout.component_package(component);
        let _ = writeln!(out, "{dependency} = {{ path = \"../{dependency}\" }}");
    }
    Artifact::new(format!("crates/{package}/Cargo.toml"), out)
}

/// One binding whose delivery the plan generates, with everything its arm needs decided.
struct Delivery<'a> {
    /// The binding.
    binding: &'a ResolvedBinding,
    /// The one component that accepts its command.
    acceptor: &'a ResolvedComponent,
    /// Whether the transformation is generated (a call to the emitted function) or owed (a call
    /// through the transformation obligation's trait).
    transformation_generated: bool,
}

/// The system crate's one module.
fn lib_module(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    covered: &mut BTreeSet<Capability>,
    stubbed: &mut BTreeSet<Capability>,
) -> Artifact {
    let types = Layout::crate_ident(layout.package());

    let mut deliveries: Vec<Delivery<'_>> = Vec::new();
    for binding in ir.bindings().values() {
        let source = binding.name.to_string();
        if !plan.is_generated(CapabilityKind::BindingDelivery, &source) {
            continue;
        }
        covered.insert(Capability {
            kind: CapabilityKind::BindingDelivery,
            source: source.clone(),
        });
        let acceptors = accepting_components(ir, binding);
        assert_eq!(
            acceptors.len(),
            1,
            "the plan generated delivery for `{source}` without exactly one acceptor; that is a \
             defect in ess-synth"
        );
        deliveries.push(Delivery {
            binding,
            acceptor: acceptors[0],
            transformation_generated: plan
                .is_generated(CapabilityKind::BindingTransformation, &source),
        });
    }

    let variants = system_event_variants(ir, plan, layout);

    let mut out = plan.provenance.commented_for("//", REGENERATE);
    out.push('\n');
    let _ = writeln!(
        out,
        "//! The `{}` system, {}: its components assembled, its bindings wired, and its one \
         transport.",
        ir.system(),
        ir.version()
    );
    // Two spellings of one paragraph, chosen by what the model actually declares. A single
    // sentence claiming `at_least_once` is "the only delivery guarantee the model declares" is
    // false about a model that writes `at_most_once`, and a generated header that misdescribes its
    // own specification is the failure this whole word exists to remove.
    if crate::plan::declares_single_attempt(ir) {
        out.push_str(
            "//!\n//! The transport is derived from the specification, not chosen: this model \
             declares more\n//! than one delivery guarantee, and the pump gives each reacting \
             binding one attempt at each\n//! published event — which is what `at_most_once` \
             requires and what `at_least_once`\n//! permits — attempting it again only for the \
             one binding that asked. Published events land\n//! on an append-only log, which is \
             the system's observable record, and so is the\n//! record \
             of what each binding invoked. What no specification\n//! determines — how an \
             escalation event is filled, behaviour behind the ports — stays an\n//! obligation; \
             see the `PLAN.md` beside this workspace.\n\n#![forbid(unsafe_code)]\n#![deny(missing_docs)]\n",
        );
    } else {
        out.push_str(
            "//!\n//! The transport is derived from the specification, not chosen: `at_least_once` \
             is the only\n//! delivery guarantee the model declares, so published events land on an \
             append-only log and a\n//! pump delivers each to every binding that reacts to it. The \
             log is the system's observable\n//! record, and so is the record of what each binding \
             invoked. What no specification determines\n//! — how an escalation event is filled, \
             behaviour behind the ports — stays an obligation; see\n//! the `PLAN.md` beside this \
             workspace.\n\n#![forbid(unsafe_code)]\n#![deny(missing_docs)]\n",
        );
    }

    if ir
        .bindings()
        .values()
        .any(|binding| binding.selection.is_some())
    {
        out.push_str(&super::selection::support(&types));
    }
    system_event_enum(&mut out, layout, &types, &variants);
    from_impls(&mut out, ir, layout, &variants);
    binding_invocation_enum(&mut out, layout, &types, &deliveries);
    transformations(&mut out, ir, plan, layout, &types, covered);
    obligations_module(&mut out, ir, plan, layout, &types, stubbed);
    system_struct(&mut out, ir, layout, &types, &deliveries, &variants);

    Artifact::new(
        format!("crates/{}/src/lib.rs", layout.system_package()),
        out,
    )
}

/// Every variant of `SystemEvent`, by the event it carries.
///
/// The events the transport carries: everything any component publishes, plus what the generated
/// deliveries react to and escalate into — so an arm always has a variant to match and an
/// escalation always has one to publish. One function, because the server's
/// `wire::encode_system_event` matches on the same enum and a second derivation of its variants
/// would be a match that stops compiling the first time the two disagree.
pub(super) fn system_event_variants<'a>(
    ir: &'a EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
) -> std::collections::BTreeMap<&'a EventHandle, String> {
    let mut events: BTreeSet<&EventHandle> = ir
        .components()
        .values()
        .flat_map(|component| component.publishes.iter())
        .collect();
    for binding in ir.bindings().values() {
        if !plan.is_generated(CapabilityKind::BindingDelivery, &binding.name.to_string()) {
            continue;
        }
        events.insert(binding.cause.event().expect("generated event capability"));
        if let ResolvedFailure::Escalate { emits } = binding.on_failure() {
            events.insert(emits);
        }
    }
    event_variants(ir, layout, &events)
}

/// The transport's event type: one variant per event the system can carry.
fn system_event_enum(
    out: &mut String,
    layout: &Layout,
    types: &str,
    variants: &std::collections::BTreeMap<&EventHandle, String>,
) {
    out.push_str(
        "\n/// An event on the system's log: everything any component publishes, and everything \
         a binding\n/// escalates into.\n#[derive(Debug, Clone, PartialEq, Eq)]\npub enum \
         SystemEvent {\n",
    );
    for (event, variant) in variants {
        let _ = writeln!(
            out,
            "    /// `{event}`.\n    {variant}({}),",
            types_path(layout, types, event.name())
        );
    }
    out.push_str("}\n");

    // The qualified name beside the variant, so a runner names what the log carries without a
    // table of its own; `match *self {}` is the whole body when the log carries nothing.
    out.push_str(
        "\nimpl SystemEvent {\n    /// The qualified name the specification declares this \
         event under.\n    pub fn name(&self) -> &'static str {\n",
    );
    if variants.is_empty() {
        out.push_str("        match *self {}\n");
    } else {
        out.push_str("        match self {\n");
        for (event, variant) in variants {
            let _ = writeln!(
                out,
                "            Self::{variant}(_) => {:?},",
                event.name().to_string()
            );
        }
        out.push_str("        }\n");
    }
    out.push_str("    }\n}\n");
}

/// The transport's second record: one variant per generated delivery, holding what was passed.
///
/// Emitted only where a delivery is generated at all, because a system with no bindings invokes
/// nothing and an empty enum would be a type with no values pretending to be a record.
fn binding_invocation_enum(
    out: &mut String,
    layout: &Layout,
    types: &str,
    deliveries: &[Delivery<'_>],
) {
    if deliveries.is_empty() {
        return;
    }
    out.push_str(
        "\n/// One command a binding invoked, and the input it passed — the transport's own \
         record.\n///\n/// Recorded by the pump at the moment of invocation, so what a binding \
         actually passed is\n/// observable from outside — a conformance run holds a mapping to \
         its words with exactly this —\n/// without instrumenting the component underneath.\n\
         #[derive(Debug, Clone, PartialEq, Eq)]\npub enum BindingInvocation {\n",
    );
    for delivery in deliveries {
        let _ = writeln!(
            out,
            "    /// `{}` invoked `{}`.\n    {}({}),",
            delivery.binding.name,
            delivery.binding.command,
            name::pascal(&delivery.binding.name.to_string()),
            types_path(layout, types, delivery.binding.command.name()),
        );
    }
    out.push_str("}\n");
}

/// One `From` per component's outbox type, so collecting an outbox is a conversion rather than a
/// re-statement of which events exist.
fn from_impls(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    variants: &std::collections::BTreeMap<&EventHandle, String>,
) {
    for component in ir.components().values() {
        let package = Layout::crate_ident(layout.component_package(&component.name));
        let events: BTreeSet<&EventHandle> = component.publishes.iter().collect();
        let component_variants = event_variants(ir, layout, &events);
        let _ = writeln!(
            out,
            "\nimpl From<{package}::PublishedEvent> for SystemEvent {{\n    fn from(event: \
             {package}::PublishedEvent) -> Self {{\n        match event {{"
        );
        for (event, variant) in &component_variants {
            let _ = writeln!(
                out,
                "            {package}::PublishedEvent::{variant}(event) => Self::{}(event),",
                variants[*event]
            );
        }
        out.push_str("        }\n    }\n}\n");
    }
}

/// The generated transformations: one function per binding whose mapping the specification fully
/// determines.
fn transformations(
    out: &mut String,
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    types: &str,
    covered: &mut BTreeSet<Capability>,
) {
    for binding in ir.bindings().values() {
        let source = binding.name.to_string();
        let generated = plan.is_generated(CapabilityKind::BindingTransformation, &source);
        if !generated && !crate::selection::prepared_helper(ir, binding) {
            continue;
        }
        if generated {
            covered.insert(Capability {
                kind: CapabilityKind::BindingTransformation,
                source: source.clone(),
            });
        }

        let event = types_path(
            layout,
            types,
            binding
                .cause
                .event()
                .expect("generated event capability")
                .name(),
        );
        let input = types_path(layout, types, binding.command.name());
        let mut function = name::value_ident(&source);
        if !generated {
            function.push_str("_from_prepared");
        }
        let inputs: Vec<_> = ir
            .command(&binding.command)
            .input
            .iter()
            .map(|field| {
                let determined = crate::plan::determined_prepared_input(ir, binding, field)
                    .unwrap_or_else(|| {
                        panic!(
                        "the plan generated the transformation of `{source}` with an undetermined \
                     mapping for `{}`; that is a defect in ess-synth",
                        field.name
                    )
                    });
                (field, determined)
            })
            .collect();
        if let Some(selection) = &binding.selection {
            let mut parameters = String::new();
            for (index, input) in selection
                .plan
                .inputs
                .iter()
                .enumerate()
                .filter(|(_, input)| input.conversion.is_some())
            {
                let ty = layout
                    .absolute_type(&crate::plan::accessor_type(
                        &input.list_type,
                        &selection.types,
                    ))
                    .replace("crate::", &format!("{types}::"));
                let _ = write!(parameters, ", prepared_{index}: &{ty}");
            }
            let _ = writeln!(out, "\n/// Select declared occurrences and project command input after complete bounded preflight.\npub fn {function}(event: &{event}{parameters}) -> Result<{input}, SelectionFailure> {{");
            out.push_str("let _ = event;\n");
            out.push_str(
                &super::selection::prelude(ir, binding, selection, layout, types)
                    .expect("selection emission preflight"),
            );
            let _ = writeln!(out, "Ok({input} {{");
        } else {
            let reads_event = inputs.iter().any(|(_, input)| input_reads_event(input));
            let parameter = if reads_event { "event" } else { "_event" };
            let _ = writeln!(
            out,
            "\n/// The binding `{source}`: `{}`, read as `{}` input.\n///\n/// Fully determined \
             by the specification: every input is filled from an event field — through the\n/// \
             declared crossing where one is named — from a literal the target admits, or left \
             absent\n/// where the input is optional and the binding says nothing.\npub fn \
             {function}({parameter}: &{event}) -> {input} {{\n    {input} {{",
            binding.cause.event().expect("generated event capability"),
            binding.command
        );
        }
        for (field, determined) in inputs {
            let expression = mapping_expression(determined, ir, layout, types);
            let _ = writeln!(
                out,
                "        {}: {expression},",
                name::value_ident(&field.name)
            );
        }
        out.push_str(if binding.selection.is_some() {
            "    })\n}\n"
        } else {
            "    }\n}\n"
        });
    }
}

/// Constant and absent assignments do not read the event that caused this binding.
fn input_reads_event(input: &DeterminedInput<'_>) -> bool {
    match input {
        DeterminedInput::Copy { .. }
        | DeterminedInput::Convert { .. }
        | DeterminedInput::Accessor { .. }
        | DeterminedInput::Selection { .. } => true,
        DeterminedInput::Literal { .. }
        | DeterminedInput::Variant { .. }
        | DeterminedInput::Omitted => false,
    }
}

fn mapping_expression(
    determined: DeterminedInput<'_>,
    ir: &EssIr,
    layout: &Layout,
    types: &str,
) -> String {
    match determined {
        DeterminedInput::Selection {
            selection,
            selector,
            projection,
            target,
        } => super::selection::mapping(ir, selection, selector, projection, target, layout, types)
            .expect("selection projection preflight"),
        DeterminedInput::Accessor {
            plan,
            types: handles,
            target,
            conversion,
        } => super::accessor::expression(plan, handles, target, conversion, layout, types)
            .expect("accessor emission preflight"),
        DeterminedInput::Copy { field } => {
            format!("event.{}.clone()", name::value_ident(field))
        }
        DeterminedInput::Convert { field, to } => format!(
            "{}::from(event.{}.clone())",
            types_path(layout, types, to.name()),
            name::value_ident(field)
        ),
        DeterminedInput::Literal { value, wraps } => {
            let mut expression = format!("{value:?}.to_owned()");
            for wrap in wraps.iter().rev() {
                expression = format!("{}({expression})", types_path(layout, types, wrap.name()));
            }
            expression
        }
        DeterminedInput::Variant { of, value } => format!(
            "{}::{}",
            types_path(layout, types, of.name()),
            name::pascal(value)
        ),
        DeterminedInput::Omitted => "None".to_owned(),
    }
}

/// One owed binding capability, as its trait and stub need it.
struct SystemObligation {
    /// The plan capability the stub stands in for.
    kind: CapabilityKind,
    /// The binding.
    source: String,
    /// The trait's name.
    trait_name: String,
    /// The method's name.
    method: String,
    /// The method's one-line doc.
    method_doc: String,
    /// The borrowed argument's identifier and type.
    argument: (String, String),
    /// The `Ok` type.
    answer: String,
    /// The plan's reason and contract, quoted on the trait.
    reason: String,
    /// The contract sentence.
    contract: String,
    /// The trait's one-line heading.
    heading: String,
}

/// What the system owes, in binding order: owed transformations, then escalations.
fn system_obligations(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    types: &str,
) -> Vec<SystemObligation> {
    let mut owed = Vec::new();
    for binding in ir.bindings().values() {
        let source = binding.name.to_string();
        let pascal = name::pascal(&source);
        let ident = name::value_ident(&source);
        if let Some(obligation) = plan.obligation_of(CapabilityKind::BindingTransformation, &source)
        {
            owed.push(SystemObligation {
                kind: CapabilityKind::BindingTransformation,
                source: source.clone(),
                trait_name: format!("{pascal}Transformation"),
                method: format!("{ident}_input"),
                method_doc: format!(
                    "Reads a `{}` as `{}` input, where the specification does not say how.",
                    binding.cause.event().expect("generated event capability"),
                    binding.command
                ),
                argument: (
                    "event".to_owned(),
                    format!(
                        "&{}",
                        types_path(
                            layout,
                            types,
                            binding
                                .cause
                                .event()
                                .expect("generated event capability")
                                .name()
                        )
                    ),
                ),
                answer: types_path(layout, types, binding.command.name()),
                reason: obligation.reason.describes(),
                contract: obligation.contract.clone(),
                heading: format!(
                    "The transformation of `{source}` — an implementation obligation."
                ),
            });
        }
        if let Some(obligation) = plan.obligation_of(CapabilityKind::BindingEscalation, &source) {
            let ResolvedFailure::Escalate { emits } = binding.on_failure() else {
                panic!(
                    "the plan owes an escalation for `{source}`, whose failure policy does not \
                     escalate; that is a defect in ess-synth"
                );
            };
            owed.push(SystemObligation {
                kind: CapabilityKind::BindingEscalation,
                source: source.clone(),
                trait_name: format!("{pascal}Escalation"),
                method: format!("{ident}_escalation"),
                method_doc: format!(
                    "Builds the declared `{emits}` from the invocation that was given up on."
                ),
                argument: (
                    "failed".to_owned(),
                    format!("&{}", types_path(layout, types, binding.command.name())),
                ),
                answer: types_path(layout, types, emits.name()),
                reason: obligation.reason.describes(),
                contract: obligation.contract.clone(),
                heading: format!("The escalation of `{source}` — an implementation obligation."),
            });
        }
    }
    owed
}

/// The system's `obligations` module: a trait per owed binding capability, and the
/// `Unimplemented` stub refusing them all.
fn obligations_module(
    out: &mut String,
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    types: &str,
    stubbed: &mut BTreeSet<Capability>,
) {
    let owed = system_obligations(ir, plan, layout, types);
    if owed.is_empty() {
        return;
    }
    out.push_str(
        "\n/// What the system itself owes its implementor, as typed seams.\n///\n/// One trait \
         per owed binding capability in the synthesis plan, each carrying the plan's own\n/// \
         contract. [`Unimplemented`](obligations::Unimplemented) satisfies every trait by \
         refusing in the type system.\npub mod obligations {\n",
    );
    for spec in &owed {
        let extra = if spec.kind == CapabilityKind::BindingEscalation {
            ir.bindings().values().find(|binding| binding.name.to_string() == spec.source && binding.selection.is_some()).map(|binding| format!("        /// Builds the escalation from a selection refusal before an Input exists.\n        fn {}_selection_escalation(&self, _event: &{}, _failure: &super::SelectionFailure) -> Result<{}, {types}::obligation::UnmetObligation> {{ Err({types}::obligation::UnmetObligation {{ capability: \"binding-escalation\", source: {:?} }}) }}\n", name::value_ident(&spec.source), types_path(layout, types, binding.cause.event().expect("selection event").name()), spec.answer, spec.source)).unwrap_or_default()
        } else {
            String::new()
        };
        let _ = writeln!(
            out,
            "    /// {}\n    ///\n    /// Why it is not generated: {}.\n    ///\n    /// \
             Contract: {}.\n    pub trait {} {{\n        /// {}\n        ///\n        /// `Err` \
             is the typed refusal of an obligation nothing has satisfied; a satisfying\n        \
             /// implementation never returns it.\n        fn {}(&self, {}: {}) -> Result<{}, \
             {types}::obligation::UnmetObligation>;\n{extra}    }}\n",
            spec.heading,
            spec.reason,
            spec.contract,
            spec.trait_name,
            spec.method_doc,
            spec.method,
            spec.argument.0,
            spec.argument.1,
            spec.answer,
        );
    }
    out.push_str(
        "    /// Every obligation of the system, refused in the type system.\n    ///\n    /// \
         Each method returns the typed refusal naming what is owed — never a panic, never a \
         guessed\n    /// value — so a system built on this stub compiles and reports its own \
         gaps.\n    pub struct Unimplemented;\n",
    );
    for spec in &owed {
        assert!(
            stubbed.insert(Capability {
                kind: spec.kind,
                source: spec.source.clone(),
            }),
            "two stubs claimed the obligation `{}` ({}); that is a defect in ess-synth",
            spec.source,
            spec.kind.describes()
        );
        let _ = writeln!(
            out,
            "\n    impl {} for Unimplemented {{\n        fn {}(&self, _{}: {}) -> Result<{}, \
             {types}::obligation::UnmetObligation> {{\n            \
             Err({types}::obligation::UnmetObligation {{ capability: \"{}\", source: \"{}\" \
             }})\n        }}\n    }}",
            spec.trait_name,
            spec.method,
            spec.argument.0,
            spec.argument.1,
            spec.answer,
            spec.kind.describes(),
            spec.source,
        );
    }
    out.push_str("}\n");
}

/// The assembled system: its components, the log, and the pump.
fn system_struct(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    types: &str,
    deliveries: &[Delivery<'_>],
    variants: &std::collections::BTreeMap<&EventHandle, String>,
) {
    // Which system obligations the pump actually calls: those of the bindings it delivers.
    let mut used_traits: Vec<String> = Vec::new();
    for delivery in deliveries {
        let pascal = name::pascal(&delivery.binding.name.to_string());
        if !delivery.transformation_generated {
            used_traits.push(format!("obligations::{pascal}Transformation"));
        }
        match delivery.binding.on_failure() {
            ResolvedFailure::Escalate { .. } => {
                used_traits.push(format!("obligations::{pascal}Escalation"));
            }
            ResolvedFailure::BoundedRetry { .. } => {
                unreachable!("refused by `failure::retry_bound`")
            }
            ResolvedFailure::ByRefusal { .. } => {
                unreachable!("refused by `failure::refusal_policy`")
            }
            ResolvedFailure::Retry | ResolvedFailure::Drop => {}
        }
    }
    // One held-back list per binding that can be attempted again, typed as the event it reacts to.
    let held: Vec<(String, String)> = deliveries
        .iter()
        .filter(|delivery| crate::plan::holds_back(delivery.binding))
        .map(|delivery| {
            (
                held_field(delivery),
                types_path(layout, types, trigger(delivery).name()),
            )
        })
        .collect();
    let with_obligations = !used_traits.is_empty();

    let mut generics: Vec<String> = ir
        .components()
        .keys()
        .map(|component| format!("{}Behaviors", name::pascal(&component.to_string())))
        .collect();
    if with_obligations {
        generics.push("Obligations".to_owned());
    }
    let generic_list = generics.join(", ");
    let angled = if generics.is_empty() {
        String::new()
    } else {
        format!("<{generic_list}>")
    };

    let _ = writeln!(
        out,
        "\n/// The `{}` system: every component behind its port, and the transport between \
         them.\n///\n/// The component fields are public because commands enter the system \
         through a component's own\n/// port; the log and its delivery cursor are not, because \
         publishing happens by pumping, not by\n/// writing history directly.\npub struct \
         System{angled} {{",
        ir.system()
    );
    for (component_name, generic) in ir.components().keys().zip(components_generics(ir)) {
        let package = Layout::crate_ident(layout.component_package(component_name));
        let port = name::pascal(&component_name.to_string());
        let _ = writeln!(
            out,
            "    /// The `{component_name}` component.\n    pub {}: {package}::{port}<{generic}>,",
            name::value_ident(&component_name.to_string())
        );
    }
    if with_obligations {
        out.push_str("    obligations: Obligations,\n");
    }
    let with_invocations = !deliveries.is_empty();
    if with_invocations {
        out.push_str("    invocations: Vec<BindingInvocation>,\n");
    }
    out.push_str("    published: Vec<SystemEvent>,\n    cursor: usize,\n");
    for (field, event) in &held {
        let _ = writeln!(out, "    {field}: Vec<{event}>,");
    }
    out.push_str("}\n");

    constructor(
        out,
        ir,
        layout,
        &angled,
        with_obligations,
        with_invocations,
        &held,
    );

    pump_impl(
        out,
        ir,
        layout,
        types,
        deliveries,
        variants,
        &generic_list,
        &angled,
        with_obligations,
        &used_traits,
    );
}

/// The event a binding reacts to.
fn trigger<'a>(delivery: &Delivery<'a>) -> &'a EventHandle {
    delivery
        .binding
        .cause
        .event()
        .expect("generated event capability")
}

/// The field holding a binding's held-back occurrences.
fn held_field(delivery: &Delivery<'_>) -> String {
    format!(
        "held_{}",
        name::value_ident(&delivery.binding.name.to_string())
    )
}

/// The method that makes one binding's attempt at one occurrence: `attempt_…` where a stopped
/// attempt is held back for the next pump, the bare delivery where it is not.
fn attempt_call(delivery: &Delivery<'_>) -> String {
    let ident = name::value_ident(&delivery.binding.name.to_string());
    if crate::plan::attempts_again(delivery.binding) {
        format!("attempt_{ident}")
    } else {
        format!("deliver_{ident}")
    }
}

/// The unbounded half of the system: construction and observation, possible whatever the
/// obligations are.
fn constructor(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    angled: &str,
    with_obligations: bool,
    with_invocations: bool,
    held: &[(String, String)],
) {
    let _ = writeln!(out, "\nimpl{angled} System{angled} {{");
    let mut parameters: Vec<String> = ir
        .components()
        .keys()
        .zip(components_generics(ir))
        .map(|(component_name, generic)| {
            let package = Layout::crate_ident(layout.component_package(component_name));
            let port = name::pascal(&component_name.to_string());
            format!(
                "{}: {package}::{port}<{generic}>",
                name::value_ident(&component_name.to_string())
            )
        })
        .collect();
    if with_obligations {
        parameters.push("obligations: Obligations".to_owned());
    }
    let _ = writeln!(
        out,
        "    /// Assembles the system from its components{}.\n    pub fn new({}) -> Self {{\n        Self {{",
        if with_obligations {
            " and the owed obligations"
        } else {
            ""
        },
        parameters.join(", ")
    );
    for component_name in ir.components().keys() {
        let _ = writeln!(
            out,
            "            {},",
            name::value_ident(&component_name.to_string())
        );
    }
    if with_obligations {
        out.push_str("            obligations,\n");
    }
    if with_invocations {
        out.push_str("            invocations: Vec::new(),\n");
    }
    out.push_str("            published: Vec::new(),\n            cursor: 0,\n");
    for (field, _) in held {
        let _ = writeln!(out, "            {field}: Vec::new(),");
    }
    out.push_str("        }\n    }\n");
    out.push_str(
        "\n    /// Everything published so far, in publication order — the system's observable \
         record.\n    pub fn published(&self) -> &[SystemEvent] {\n        &self.published\n    \
         }\n",
    );
    out.push_str(
        "\n    /// Takes every event the pump has already delivered off the log, in publication \
         order.\n    ///\n    /// A long-running shell calls this after each `pump`, or the log \
         holds every event the\n    /// process ever published. A `pump` returns with every \
         logged event delivered: each\n    /// reacting binding has had its attempt, and a \
         binding whose attempt stopped holds the event in\n    /// its own held-back list, not \
         on the log. Events published since the last `pump` stay on the\n    /// log, so the \
         next `pump` still delivers them; taking never skips a binding.\n    pub fn \
         take_published(&mut self) -> Vec<SystemEvent> {\n        \
         let delivered: Vec<SystemEvent> = self.published.drain(..self.cursor).collect();\n        \
         self.cursor = 0;\n        delivered\n    }\n",
    );
    if with_invocations {
        out.push_str(
            "\n    /// Every command a binding invoked so far, in invocation order, with what it \
             passed.\n    pub fn invocations(&self) -> &[BindingInvocation] {\n        \
             &self.invocations\n    }\n\n    /// Takes the record of every command a binding \
             invoked so far, in invocation order.\n    ///\n    /// The record's counterpart to \
             [`System::take_published`], for the same long-running shell.\n    pub fn \
             take_invocations(&mut self) -> Vec<BindingInvocation> {\n        \
             core::mem::take(&mut self.invocations)\n    }\n",
        );
    }
    out.push_str("}\n");
}

/// The generic parameter names of the component fields, in component order.
pub(crate) fn components_generics(ir: &EssIr) -> Vec<String> {
    ir.components()
        .keys()
        .map(|component| format!("{}Behaviors", name::pascal(&component.to_string())))
        .collect()
}

/// `true` when the system crate carries obligations of its own — a transformation nobody
/// determined, or an escalation to build.
///
/// The question that decides whether `System` takes a third type parameter, asked by every
/// artifact that has to spell the type: the system crate itself, the browser bridge and the server
/// crate. One answer, because three would drift.
pub(crate) fn has_obligations(ir: &EssIr, plan: &SynthesisPlan) -> bool {
    ir.bindings().values().any(|binding| {
        let source = binding.name.to_string();
        if !plan.is_generated(CapabilityKind::BindingDelivery, &source) {
            return false;
        }
        !plan.is_generated(CapabilityKind::BindingTransformation, &source)
            || matches!(binding.on_failure(), ResolvedFailure::Escalate { .. })
    })
}

/// Every `where` line `System::pump` carries, spelled from outside the system crate: each
/// component's behaviours bound by its own port, and the system's obligations by the traits the
/// pump calls. `system` is the system crate's path (`gatepass_system`).
///
/// The server's handlers call `pump` after every command, so they carry exactly these bounds; the
/// pump's own `impl` block derives them from the same plan answer ([`has_obligations`]'s), and a
/// generated server that disagreed with it would not compile.
pub(crate) fn pump_bounds(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    types: &str,
    system: &str,
) -> Vec<String> {
    let mut bounds = Vec::new();
    for (component_name, generic) in ir.components().keys().zip(components_generics(ir)) {
        let component = &ir.components()[component_name];
        let list = super::port::bound_list(ir, layout, component, types);
        if !list.is_empty() {
            bounds.push(format!("    {generic}: {},", list.join(" + ")));
        }
    }
    let mut owed = Vec::new();
    for binding in ir.bindings().values() {
        let source = binding.name.to_string();
        if !plan.is_generated(CapabilityKind::BindingDelivery, &source) {
            continue;
        }
        let pascal = name::pascal(&source);
        if !plan.is_generated(CapabilityKind::BindingTransformation, &source) {
            owed.push(format!("{system}::obligations::{pascal}Transformation"));
        }
        if matches!(binding.on_failure(), ResolvedFailure::Escalate { .. }) {
            owed.push(format!("{system}::obligations::{pascal}Escalation"));
        }
    }
    if !owed.is_empty() {
        bounds.push(format!("    Obligations: {},", owed.join(" + ")));
    }
    bounds
}

/// `true` when the plan generates any binding's delivery, which is when `System` records what the
/// bindings invoked (`invocations`, `take_invocations`).
pub(crate) fn has_deliveries(ir: &EssIr, plan: &SynthesisPlan) -> bool {
    ir.bindings().values().any(|binding| {
        plan.is_generated(CapabilityKind::BindingDelivery, &binding.name.to_string())
    })
}

/// `true` when `System::pump` fails with the selection transport's `TransportFailure`, which has
/// no `Display`, rather than with an `UnmetObligation`.
pub(crate) fn pump_fails_with_transport(ir: &EssIr) -> bool {
    ir.bindings()
        .values()
        .any(|binding| binding.selection.is_some())
}

/// The pump: collection, delivery per binding, and the held-back attempts, one arm per event.
#[allow(clippy::too_many_arguments)]
fn pump_impl(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    types: &str,
    deliveries: &[Delivery<'_>],
    variants: &std::collections::BTreeMap<&EventHandle, String>,
    generic_list: &str,
    angled: &str,
    with_obligations: bool,
    used_traits: &[String],
) {
    let mut bounds: Vec<String> = Vec::new();
    for (component_name, generic) in ir.components().keys().zip(components_generics(ir)) {
        let component = &ir.components()[component_name];
        let list = super::port::bound_list(ir, layout, component, types);
        if !list.is_empty() {
            bounds.push(format!("    {generic}: {},", list.join(" + ")));
        }
    }
    if with_obligations {
        bounds.push(format!("    Obligations: {},", used_traits.join(" + ")));
    }

    let _ = writeln!(out, "\nimpl<{generic_list}> System{angled}");
    if !bounds.is_empty() {
        out.push_str("where\n");
        for bound in &bounds {
            let _ = writeln!(out, "{bound}");
        }
    }
    out.push_str("{\n");

    let unmet = if pump_fails_with_transport(ir) {
        "TransportFailure".to_owned()
    } else {
        format!("{types}::obligation::UnmetObligation")
    };
    if deliveries.is_empty() {
        // Nothing in this specification reacts to an occurrence, so the pump advances the cursor
        // and stops. Binding the occurrence would be binding a value nothing reads, which the
        // generated tree's own build reports as a warning — and a generated tree that warns
        // teaches its reader that warnings here are normal.
        let _ = writeln!(
            out,
            "    /// Delivers until quiescent: collects every component's outbox onto the log. No \
             binding\n    /// reacts to anything this specification publishes, so collecting is \
             the whole delivery.\n    pub fn pump(&mut self) -> Result<(), {unmet}> {{\n        \
             loop {{\n            self.collect();\n            if self.cursor == \
             self.published.len() {{\n                return Ok(());\n            }}\n            \
             self.cursor += 1;\n        }}\n    }}"
        );
    } else {
        pump_fn(out, deliveries, &unmet);
        redeliver_fn(out, ir, &unmet);
        let _ = writeln!(
            out,
            "\n    /// Delivers every logged event the cursor has not passed yet, collecting as it \
             goes: each\n    /// reacting binding gets one attempt, and the cursor passes the \
             event once all of them have\n    /// had it. `Err` is the first attempt that \
             failed.\n    fn drain(&mut self) -> Result<(), {unmet}> {{\n        let mut failed \
             = None;\n        loop {{\n            self.collect();\n            if self.cursor == \
             self.published.len() {{\n                return failed.map_or(Ok(()), Err);\n            \
             }}\n            let event = self.published[self.cursor].clone();\n            if let \
             Err(failure) = self.deliver(&event) {{\n                failed = \
             failed.or(Some(failure));\n            }}\n            self.cursor += 1;\n        \
             }}\n    }}"
        );
    }

    out.push_str(
        "\n    /// Moves every component's outbox onto the log, in component order.\n    fn \
         collect(&mut self) {\n",
    );
    for component_name in ir.components().keys() {
        let _ = writeln!(
            out,
            "        for event in self.{}.drain_outbox() {{\n            \
             self.published.push(SystemEvent::from(event));\n        }}",
            name::value_ident(&component_name.to_string())
        );
    }
    out.push_str("    }\n");

    if !deliveries.is_empty() {
        let every: Vec<&Delivery<'_>> = deliveries.iter().collect();
        dispatch_fn(
            out,
            "deliver",
            "    /// Gives every binding that reacts to one logged event its attempt at it.\n    \
             ///\n    /// Each binding's attempt is its own: one that stops holds the event in \
             that binding's\n    /// held-back list where it may run again, and the bindings \
             beside it still run. `Err` is the\n    /// first attempt that failed.\n",
            &every,
            variants,
            &unmet,
        );
        if crate::plan::declares_single_attempt(ir) {
            let again: Vec<&Delivery<'_>> = deliveries
                .iter()
                .filter(|delivery| crate::plan::attempts_again(delivery.binding))
                .collect();
            dispatch_fn(
                out,
                "redeliver_occurrence",
                "    /// Gives one occurrence again to every binding that reacts to it and \
                 declares\n    /// `at_least_once`. An `at_most_once` binding declared one attempt \
                 and no redelivery, so it\n    /// is left out.\n",
                &again,
                variants,
                &unmet,
            );
        }
        let selects = pump_fails_with_transport(ir);
        for delivery in deliveries {
            binding_fns(out, ir, layout, types, delivery, variants, &unmet, selects);
        }
    }
    out.push_str("}\n");
}

/// `pump` where a binding reacts: this pump's own events first, then every held-back attempt.
fn pump_fn(out: &mut String, deliveries: &[Delivery<'_>], unmet: &str) {
    let _ = writeln!(
        out,
        "    /// Delivers until quiescent: collects every component's outbox onto the log, then \
         gives each\n    /// binding that reacts to a logged event one attempt at it, and passes \
         the event once every one\n    /// of them has had its attempt.\n    ///\n    /// \
         Delivery is tracked per binding. A binding whose attempt an unmet obligation stopped, \
         and\n    /// which permits a second run, holds the event in its own held-back list; so \
         does a declared\n    /// refusal its `on_failure: retry` answers. The next pump attempts \
         each held event again for\n    /// that binding alone, after the events published \
         since, and no other binding receives it\n    /// twice.\n    ///\n    /// `Err` carries \
         the first failure among the events this pump delivered for the first time:\n    /// what \
         was published since the last pump, and what delivering it published in turn. A\n    /// \
         held-back attempt that fails again stays held and is not this pump's answer, so an \
         event\n    /// that keeps failing never fails the pump for what is published after it. A \
         specification\n    /// whose bindings feed each other without end will not quiesce, and \
         this pump will not\n    /// pretend otherwise.\n    pub fn pump(&mut self) -> \
         Result<(), {unmet}> {{"
    );
    let holding: Vec<&Delivery<'_>> = deliveries
        .iter()
        .filter(|delivery| crate::plan::holds_back(delivery.binding))
        .collect();
    if holding.is_empty() {
        out.push_str("        self.drain()\n    }\n");
        return;
    }
    for delivery in &holding {
        let field = held_field(delivery);
        let _ = writeln!(
            out,
            "        let {field} = core::mem::take(&mut self.{field});"
        );
    }
    out.push_str(
        "        let delivered = self.drain();\n        // Each held-back event, attempted again \
         for its own binding alone. What that publishes is\n        // delivered like anything \
         else; an attempt that stops again is held again.\n",
    );
    for delivery in &holding {
        let _ = writeln!(
            out,
            "        for event in {} {{\n            let _ = self.{}(&event);\n        }}",
            held_field(delivery),
            attempt_call(delivery)
        );
    }
    out.push_str("        let _ = self.drain();\n        delivered\n    }\n");
}

/// The caller's redelivery: every binding that may run again, without a second occurrence.
fn redeliver_fn(out: &mut String, ir: &EssIr, unmet: &str) {
    let (who, dispatch) = if crate::plan::declares_single_attempt(ir) {
        (
            "every binding that reacts to it and declares\n    /// `at_least_once`, again, then \
             pumps until quiescent. An `at_most_once` binding declared one\n    /// attempt and \
             no redelivery, so it does not run again.",
            "redeliver_occurrence",
        )
    } else {
        (
            "every binding that reacts to it, again,\n    /// then pumps until quiescent.",
            "deliver",
        )
    };
    let _ = writeln!(
        out,
        "\n    /// Delivers one already-published occurrence to {who}\n    ///\n    /// The \
         duplicate a delivery guarantee of at least once explicitly permits: the occurrence \
         is\n    /// not published a second time — a second occurrence would be a different claim \
         — but\n    /// every binding that permits it runs again, and what that causes lands on \
         the log as usual.\n    pub fn redeliver(&mut self, event: &SystemEvent) -> Result<(), \
         {unmet}> {{\n        self.{dispatch}(event)?;\n        self.pump()\n    }}"
    );
}

/// One dispatch over the log's events: each reacting binding among `deliveries` gets its attempt,
/// and the first failure is the answer once they all have.
fn dispatch_fn(
    out: &mut String,
    function: &str,
    doc: &str,
    deliveries: &[&Delivery<'_>],
    variants: &std::collections::BTreeMap<&EventHandle, String>,
    unmet: &str,
) {
    let _ = writeln!(
        out,
        "\n{doc}    fn {function}(&mut self, event: &SystemEvent) -> Result<(), {unmet}> {{"
    );
    if deliveries.is_empty() {
        out.push_str("        let _ = event;\n        Ok(())\n    }\n");
        return;
    }
    out.push_str("        let mut failed = None;\n        match event {\n");
    for (event, variant) in variants {
        let reacting: Vec<&&Delivery<'_>> = deliveries
            .iter()
            .filter(|delivery| trigger(delivery).name() == event.name())
            .collect();
        if reacting.is_empty() {
            let _ = writeln!(out, "            SystemEvent::{variant}(_) => {{}}");
            continue;
        }
        let _ = writeln!(out, "            SystemEvent::{variant}(event) => {{");
        for delivery in reacting {
            let _ = writeln!(
                out,
                "                if let Err(failure) = self.{}(event) {{\n                    \
                 failed = failed.or(Some(failure));\n                }}",
                attempt_call(delivery)
            );
        }
        out.push_str("            }\n");
    }
    out.push_str("        }\n        failed.map_or(Ok(()), Err)\n    }\n");
}

/// One binding's own delivery method, and where its stopped attempt may run again, the attempt
/// that holds it back.
#[allow(clippy::too_many_arguments)]
fn binding_fns(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    types: &str,
    delivery: &Delivery<'_>,
    variants: &std::collections::BTreeMap<&EventHandle, String>,
    unmet: &str,
    selects: bool,
) {
    let source = delivery.binding.name.to_string();
    let ident = name::value_ident(&source);
    let event = trigger(delivery);
    let event_type = types_path(layout, types, event.name());
    let mut body = String::new();
    delivery_arm(&mut body, ir, layout, types, delivery, variants);
    let _ = writeln!(
        out,
        "\n    /// `{source}`'s attempt at one `{event}`: transform, record the invocation, \
         invoke the\n    /// acceptor's port, and answer a declared refusal with the declared \
         policy.\n    fn deliver_{ident}(&mut self, event: &{event_type}) -> Result<(), {unmet}> \
         {{"
    );
    for line in body.lines() {
        out.push_str(line.strip_prefix("        ").unwrap_or(line));
        out.push('\n');
    }
    out.push_str("        Ok(())\n    }\n");
    if !crate::plan::attempts_again(delivery.binding) {
        return;
    }
    let field = held_field(delivery);
    let hold = if selects {
        format!(
            "                // A failed selection is a delivery that ran to its end, its policy \
             applied.\n                if !matches!(failure, TransportFailure::Selection(_)) \
             {{\n                    self.{field}.push(event.clone());\n                }}\n"
        )
    } else {
        format!("                self.{field}.push(event.clone());\n")
    };
    let _ = writeln!(
        out,
        "\n    /// One attempt of `{source}` at one `{event}`, held back for the next pump when \
         an unmet\n    /// obligation stopped it: `at_least_once` permits running it again, for \
         this binding alone.\n    fn attempt_{ident}(&mut self, event: &{event_type}) -> \
         Result<(), {unmet}> {{\n        match self.deliver_{ident}(event) {{\n            Ok(()) \
         => Ok(()),\n            Err(failure) => {{\n{hold}                Err(failure)\n            \
         }}\n        }}\n    }}"
    );
}

/// One binding's delivery: transform, record the invocation, invoke the acceptor's port, and
/// answer a declared refusal with the declared policy.
///
/// The failure a policy speaks about is the invoked command taking an error-carrying outcome —
/// see the [module documentation](self) — so the arm matches on the outcome enum. An unmet
/// obligation propagates with `?` instead: it is the workspace being unfinished, not a delivery
/// failing, and routing it into the policy would publish a domain event no domain fact caused.
fn delivery_arm(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    types: &str,
    delivery: &Delivery<'_>,
    variants: &std::collections::BTreeMap<&EventHandle, String>,
) {
    let binding = delivery.binding;
    let held = held_field(delivery);
    let source = binding.name.to_string();
    let ident = name::value_ident(&source);
    let pascal = name::pascal(&source);
    let acceptor = name::value_ident(&delivery.acceptor.name.to_string());
    let method = name::value_ident(&layout.type_name(binding.command.name()));
    let outcome_type = format!(
        "{}Outcome",
        types_path(layout, types, binding.command.name())
    );
    let command = ir.command(&binding.command);
    let successes: Vec<String> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.error.is_none())
        .map(|outcome| name::pascal(outcome.name.as_str()))
        .collect();
    let mut refusals: Vec<String> = command
        .outcomes
        .iter()
        .filter(|outcome| outcome.error.is_some())
        .map(|outcome| name::pascal(outcome.name.as_str()))
        .collect();
    // An unknown instance is refused with the same declared error, so the policy runs for it too.
    refusals.extend(
        ess_gen::unknown_instance::unknown_instance_answer(ir, command)
            .map(super::items::unknown_instance_variant),
    );

    let _ = writeln!(
        out,
        "                // `{source}`: {}, on failure {}.",
        ess_gen::graph::delivery_word(binding.delivery),
        binding.failure.as_str()
    );
    if delivery.transformation_generated && binding.selection.is_some() {
        selection_failure_policy(out, binding, &ident, &held, variants);
    } else if delivery.transformation_generated {
        let _ = writeln!(out, "                let input = {ident}(event);");
    } else {
        let _ = writeln!(
            out,
            "                let input = self.obligations.{ident}_input(event)?;"
        );
    }
    let _ = writeln!(
        out,
        "                self.invocations.push(BindingInvocation::{pascal}(input.clone()));"
    );

    // A command with no error-carrying outcome cannot refuse, so no policy can ever run; the
    // invocation still happens, and an unmet obligation still propagates.
    if refusals.is_empty() {
        let _ = writeln!(
            out,
            "                // No declared refusal exists, so this invocation cannot fail.\n                \
             let _ = self.{acceptor}.{method}(input)?;"
        );
        return;
    }
    match binding.on_failure() {
        ResolvedFailure::Escalate { emits } => {
            let body = format!(
                "                        // The declared refusal is the failure the policy names: \
                 escalate.\n                        let escalation = \
                 self.obligations.{ident}_escalation(&input)?;\n                        \
                 self.published.push(SystemEvent::{}(escalation));\n",
                variants[emits]
            );
            refusal_match(
                out,
                &format!("self.{acceptor}.{method}(input.clone())"),
                &outcome_type,
                &successes,
                &refusals,
                &body,
            );
        }
        ResolvedFailure::Retry => {
            let body = format!(
                "                        // The declared refusal is the failure the policy names: \
                 hold the event for this\n                        // binding alone, and the next \
                 pump makes one more attempt.\n                        \
                 self.{held}.push(event.clone());\n"
            );
            refusal_match(
                out,
                &format!("self.{acceptor}.{method}(input)"),
                &outcome_type,
                &successes,
                &refusals,
                &body,
            );
        }
        ResolvedFailure::BoundedRetry { .. } | ResolvedFailure::ByRefusal { .. } => refused(),
        ResolvedFailure::Drop => {
            let _ = writeln!(
                out,
                "                // `drop`: a declared refusal is given up silently, because that \
                 is what the author\n                // wrote; an unmet obligation still \
                 propagates.\n                let _ = self.{acceptor}.{method}(input)?;"
            );
        }
    }
}

/// The outcome match a failure policy needs: `?` on the call so an unmet obligation propagates,
/// every success variant inert, and every error-carrying variant running the policy's body.
fn refusal_match(
    out: &mut String,
    call: &str,
    outcome_type: &str,
    successes: &[String],
    refusals: &[String],
    body: &str,
) {
    let _ = writeln!(out, "                match {call}? {{");
    for variant in successes {
        let _ = writeln!(
            out,
            "                    {outcome_type}::{variant} {{ .. }} => {{}}"
        );
    }
    let refused: String = refusals
        .iter()
        .map(|variant| format!("{outcome_type}::{variant} {{ .. }}"))
        .collect::<Vec<_>>()
        .join(" | ");
    let _ = writeln!(
        out,
        "                    {refused} => {{\n{body}                    }}\n                }}"
    );
}

fn selection_failure_policy(
    out: &mut String,
    binding: &ResolvedBinding,
    ident: &str,
    held: &str,
    variants: &std::collections::BTreeMap<&EventHandle, String>,
) {
    let _ = writeln!(out, "                let input = match {ident}(event) {{ Ok(input) => input, Err(failure) => {{");
    match binding.on_failure() {
        ResolvedFailure::Retry => {
            let _ = writeln!(out, "self.{held}.push(event.clone());");
        }
        ResolvedFailure::BoundedRetry { .. } | ResolvedFailure::ByRefusal { .. } => refused(),
        ResolvedFailure::Drop => {}
        ResolvedFailure::Escalate { emits } => {
            let _ = writeln!(out, "let escalation = self.obligations.{ident}_selection_escalation(event, &failure)?; self.published.push(SystemEvent::{}(escalation));", variants[emits]);
        }
    }
    // The policy has run, so this binding's attempt ends here; the other bindings that react to
    // this event still run (`deliver`), and a failed selection is not held back (`attempt_…`).
    out.push_str("return Err(failure.into()); } };\n");
}

/// A bounded retry or a policy selected per refusal reached emission: both are refused before it,
/// by `failure::retry_bound` and `failure::refusal_policy`.
fn refused() -> ! {
    unreachable!("refused by `failure::retry_bound` and `failure::refusal_policy`")
}
