//! Generated behaviour: what the specification fully determines, written against generated ports.
//!
//! A command the plan marks generated (`crate::determined`) gets an implementation of its existing
//! `…Behavior` trait on [`Generated<P>`](module), so the component ports that already take a
//! behaviour bundle take this one unchanged, and an implementor who wants a different behaviour
//! still writes their own bundle. Everything the specification leaves to the implementor is a
//! port `P` supplies: one storage trait per entity (get, put and delete a snapshot by identity —
//! ess generates the trait and never a store) and one context trait (the caller's attributes, the
//! identities and values the model says the implementation assigns, and the answer to an
//! `external:` branch, forced or decided). Every behaviour and query the plan still owes is
//! forwarded to `P`, so `Generated<P>` is a complete bundle.
//!
//! # The order of evaluation
//!
//! The conformance interpreter's (`ess_conformance::interpret::execute`), and the precedence order
//! of `docs/design/cross-record-and-stored-field-guards.md` where the interpreter declines:
//!
//! 1. the input-guarded refusals, the first declared whose guard holds;
//! 2. on a command whose branches read the addressed row (`when_subject`, `when_subject_state`),
//!    the row, answering an identity no record carries with the unknown-instance answer, then the
//!    subject-guarded branches in declaration order;
//! 3. the accepting and external branches in declaration order — an external branch is taken where
//!    the context port answers that it is and its input guard holds;
//! 4. the default.
//!
//! The branch selected then reads its own subject: an identity no record carries is answered by
//! `unknown_instance:` (else `wrong_state:`), and a move from a state it does not start in by
//! `wrong_state:`. A forced external branch naming no subject therefore answers before any row is
//! read. A guard that is Unknown, or a request no declared branch answers, is the typed refusal
//! naming the command: the model declares no outcome for it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_compiler::ir::{
    EssIr, ResolvedBody, ResolvedCommand, ResolvedCondition, ResolvedEffect, ResolvedEntity,
    ResolvedInstance, ResolvedOutcome, ResolvedPayloadField, ResolvedPayloadValue, ResolvedTypeRef,
};
use ess_domain::name::QualifiedName;
use ess_domain::types::Primitive;
use ess_gen::{Artifact, Provenance};
use ess_primitives::facts::FactValue;
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

use super::layout::Layout;
use super::name;
use crate::determined::{self, Env, HeldField, Kind, Resolved, Root, Step};
use crate::plan::{Capability, CapabilityKind, SynthesisPlan, REGENERATE};

/// `true` where the workspace carries a `behaviour` module: some command is generated.
pub(crate) fn used(ir: &EssIr) -> bool {
    determined::any_generated(ir)
}

/// The types crate's `behaviour` module, or `None` where no command's behaviour is generated.
pub(super) fn module(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    provenance: &Provenance,
    covered: &mut BTreeSet<Capability>,
) -> Option<Artifact> {
    if !used(ir) {
        return None;
    }
    let storages = storage_names(ir, layout);
    let mut uses = Uses::default();
    let mut impls = String::new();
    for command in ir.commands().values() {
        let source = command.name.to_string();
        if plan.is_generated(CapabilityKind::CommandBehavior, &source) {
            covered.insert(Capability {
                kind: CapabilityKind::CommandBehavior,
                source,
            });
            let mut writer = Writer {
                ir,
                layout,
                command,
                storages: &storages,
                uses: &mut uses,
                bounds: Bounds::default(),
            };
            impls.push_str(&writer.implementation());
        } else if plan
            .obligation_of(CapabilityKind::CommandBehavior, &source)
            .is_some()
        {
            forward_behaviour(&mut impls, layout, command);
        }
    }
    for view in ir.views().values() {
        if plan
            .obligation_of(CapabilityKind::ViewQuery, &view.name.to_string())
            .is_some()
        {
            forward_query(&mut impls, layout, &view.name);
        }
    }

    let mut out = provenance.commented_for("//", REGENERATE);
    out.push_str(HEADER);
    for entity in &uses.storages {
        storage_trait(&mut out, ir, layout, &storages, entity);
    }
    context_trait(&mut out, &uses);
    out.push_str(GENERATED);
    out.push_str(&impls);
    helpers(&mut out, &uses);
    Some(Artifact::new(
        format!("crates/{}/src/behaviour.rs", layout.package()),
        out,
    ))
}

/// The module's opening: what it is, and the one import every impl names.
const HEADER: &str = "
//! What the specification fully determines, generated: the behaviour of every command the plan
//! lists as generated, written against ports the implementor supplies.
//!
//! Storage is a port: one trait per entity, get, put and delete of a snapshot by identity. ess
//! generates the trait and never a store. `Context` is the other port: the caller's attributes,
//! every identity and value the model says the implementation assigns, and the answer to each
//! `external:` branch. [`Generated`] implements every generated `…Behavior` trait over those ports
//! and forwards every behaviour and query the plan still owes to them, so it is a complete bundle
//! for every component port. To replace one generated behaviour, write a bundle of your own that
//! implements that trait and delegates the rest to a `Generated`.
//!
//! An `Err` from a generated behaviour is the typed refusal naming the command: the model declares
//! no outcome for the request (a guard is undecidable over it, or no declared branch answers it),
//! or — as `entity invariant` — the declared outcome would leave an entity breaking an invariant.

use crate::obligation::UnmetObligation;
";

/// The bundle every generated behaviour is implemented on.
const GENERATED: &str = "
/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes,
/// `Context` where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
/// owes; `Generated<P>` forwards those to it.
pub struct Generated<P> {
    /// The storage and context ports, and every behaviour or query still owed.
    pub ports: P,
}

impl<P> Generated<P> {
    /// The generated behaviours, over `ports`.
    pub fn new(ports: P) -> Self {
        Self { ports }
    }
}
";

/// What the generated impls asked of the ports and of the helpers, collected while rendering.
#[derive(Default)]
struct Uses {
    /// Entities whose storage trait some behaviour uses.
    storages: BTreeSet<QualifiedName>,
    /// Caller attribute methods: name → returned type.
    callers: BTreeMap<String, (String, String)>,
    /// Assigned-value methods: name → returned type.
    generates: BTreeMap<String, (String, String)>,
    /// Some behaviour asks the context about an `external:` branch.
    external: bool,
    /// Helper functions used, by name.
    helpers: BTreeSet<&'static str>,
}

/// The ports one impl needs.
#[derive(Default)]
struct Bounds {
    context: bool,
    storages: BTreeSet<QualifiedName>,
}

/// The storage trait name of each entity: `<Type>Storage`, or — where two entities of different
/// domains share a type name — every one spelled from its full name, so adding an entity never
/// renames another's trait.
fn storage_names(ir: &EssIr, layout: &Layout) -> BTreeMap<QualifiedName, String> {
    let short: BTreeMap<QualifiedName, String> = ir
        .entities()
        .keys()
        .map(|entity| {
            (
                entity.clone(),
                format!("{}Storage", layout.type_name(entity)),
            )
        })
        .collect();
    let distinct: BTreeSet<&String> = short.values().collect();
    if distinct.len() == short.len() {
        return short;
    }
    ir.entities()
        .keys()
        .map(|entity| {
            (
                entity.clone(),
                format!("{}Storage", name::type_fragment(&entity.to_string())),
            )
        })
        .collect()
}

/// One entity's storage port.
fn storage_trait(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    storages: &BTreeMap<QualifiedName, String>,
    entity: &QualifiedName,
) {
    let declared = &ir.entities()[entity];
    let snapshot = snapshot_path(layout, entity);
    let identity = layout.absolute_type(&declared.identity.type_ref);
    let _ = writeln!(
        out,
        "\n/// Where `{entity}` is stored — a port the implementor provides.\n///\n/// Keyed by the \
         identity `{}`. ess generates this trait and never an implementation of it.\npub trait {} \
         {{\n    /// The instance with this identity, or `None` where none is stored.\n    \
         fn get(&self, identity: &{identity}) -> Option<{snapshot}>;\n\n    /// Stores this \
         instance under its identity, replacing what was held.\n    fn put(&mut self, snapshot: \
         {snapshot});\n\n    /// Removes the instance with this identity.\n    fn delete(&mut \
         self, identity: &{identity});\n}}",
        declared.identity.name, storages[entity]
    );
}

/// The context port: only the methods some generated behaviour asks.
fn context_trait(out: &mut String, uses: &Uses) {
    if uses.callers.is_empty() && uses.generates.is_empty() && !uses.external {
        return;
    }
    out.push_str(
        "\n/// What the specification leaves to the implementor's context — a port the implementor \
         provides.\n///\n/// The caller's attributes, the values the model says the \
         implementation assigns, and the answer\n/// to each `external:` branch.\npub trait \
         Context {\n",
    );
    let mut first = true;
    let mut separate = |out: &mut String| {
        if !first {
            out.push('\n');
        }
        first = false;
    };
    for (method, (ty, attribute)) in &uses.callers {
        separate(out);
        let _ = writeln!(
            out,
            "    /// The authenticated caller's `{attribute}`, or `None` where the caller carries \
             none.\n    fn {method}(&self) -> Option<{ty}>;"
        );
    }
    for (method, (ty, of)) in &uses.generates {
        separate(out);
        let _ = writeln!(
            out,
            "    /// A new `{of}`, which the model says the implementation assigns — a created \
             identity, a\n    /// `{{generated: true}}` value, or an event field the model leaves \
             undetermined.\n    fn {method}(&mut self) -> {ty};"
        );
    }
    if uses.external {
        separate(out);
        out.push_str(
            "    /// Whether the external branch `outcome` of `command` is taken on this \
             invocation.\n    ///\n    /// Asked in declaration order, before the branch's input \
             guard is read; the first branch\n    /// answered `true` whose guard holds is taken. A \
             test forces a branch by answering `true`\n    /// for it alone; a deployment asks \
             whatever decides it.\n    fn external(&mut self, command: &'static str, outcome: \
             &'static str) -> bool;\n",
        );
    }
    out.push_str("}\n");
}

/// Forwards one owed behaviour to the ports.
fn forward_behaviour(out: &mut String, layout: &Layout, command: &ResolvedCommand) {
    let type_name = layout.type_name(&command.name);
    let module = layout.module(layout.owner(&command.name));
    let method = name::value_ident(&type_name);
    let _ = writeln!(
        out,
        "\nimpl<P: crate::{module}::obligations::{type_name}Behavior> \
         crate::{module}::obligations::{type_name}Behavior for Generated<P> {{\n    fn \
         {method}(&mut self, input: crate::{module}::{type_name}) -> \
         Result<crate::{module}::{type_name}Outcome, UnmetObligation> {{\n        \
         crate::{module}::obligations::{type_name}Behavior::{method}(&mut self.ports, input)\n    \
         }}\n}}"
    );
}

/// Forwards one owed query to the ports.
fn forward_query(out: &mut String, layout: &Layout, view: &QualifiedName) {
    let type_name = layout.type_name(view);
    let module = layout.module(layout.owner(view));
    let method = name::value_ident(&type_name);
    let _ = writeln!(
        out,
        "\nimpl<P: crate::{module}::obligations::{type_name}Query> \
         crate::{module}::obligations::{type_name}Query for Generated<P> {{\n    fn \
         {method}(&self) -> Result<Vec<crate::{module}::{type_name}>, UnmetObligation> {{\n        \
         crate::{module}::obligations::{type_name}Query::{method}(&self.ports)\n    }}\n}}"
    );
}

/// `crate::<module>::<Snapshot>` for an entity.
fn snapshot_path(layout: &Layout, entity: &QualifiedName) -> String {
    format!(
        "crate::{}::{}",
        layout.module(layout.owner(entity)),
        layout.entity_snapshot(entity)
    )
}

/// `crate::<module>::<Type>` for any declaration.
fn declared_path(layout: &Layout, declared: &QualifiedName) -> String {
    format!(
        "crate::{}::{}",
        layout.module(layout.owner(declared)),
        layout.type_name(declared)
    )
}

/// The private helpers the impls call, only those some impl uses.
fn helpers(out: &mut String, uses: &Uses) {
    if uses.helpers.contains("undeclared") {
        out.push_str(
            "\n/// The typed refusal of a request the model declares no outcome for.\nfn \
             undeclared(source: &'static str) -> UnmetObligation {\n    let capability = \
             \"command behaviour\";\n    UnmetObligation { capability, source }\n}\n",
        );
    }
    if uses.helpers.contains("decided") {
        out.push_str(
            "\n/// A guard's truth, where it has one: Unknown selects no branch, so the model \
             declares no outcome.\nfn decided(truth: Option<bool>, command: &'static str) -> \
             Result<bool, UnmetObligation> {\n    truth.ok_or_else(|| undeclared(command))\n}\n",
        );
    }
    if uses.helpers.contains("all") {
        out.push_str(
            "\n/// Three-valued conjunction: false wins, then Unknown.\nfn all(truths: \
             &[Option<bool>]) -> Option<bool> {\n    if truths.contains(&Some(false)) {\n        \
             Some(false)\n    } else if truths.contains(&None) {\n        None\n    } else {\n        \
             Some(true)\n    }\n}\n",
        );
    }
    if uses.helpers.contains("any") {
        out.push_str(
            "\n/// Three-valued disjunction: true wins, then Unknown.\nfn any(truths: \
             &[Option<bool>]) -> Option<bool> {\n    if truths.contains(&Some(true)) {\n        \
             Some(true)\n    } else if truths.contains(&None) {\n        None\n    } else {\n        \
             Some(false)\n    }\n}\n",
        );
    }
    if uses.helpers.contains("equal") {
        out.push_str(
            "\n/// Equality of two read values; an unread one is Unknown.\nfn equal<T: \
             PartialEq>(left: Option<T>, right: Option<T>) -> Option<bool> {\n    Some(left? == \
             right?)\n}\n",
        );
    }
    if uses.helpers.contains("compare_numbers") {
        out.push_str(NUMBERS);
    }
}

/// Exact comparison of two decimal renderings — an `Integer` is rendered the same way — without a
/// float, which would round the values a `Decimal` exists not to round.
const NUMBERS: &str = "
/// A decimal rendering as its sign, its whole digits and its fraction digits, without the zeros
/// that do not change its value; `None` where it is not a plain decimal.
fn number_parts(text: &str) -> Option<(bool, String, String)> {
    let (negative, digits) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let (whole, fraction) = digits.split_once('.').unwrap_or((digits, \"\"));
    if whole.is_empty() && fraction.is_empty() {
        return None;
    }
    if !whole.bytes().chain(fraction.bytes()).all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    let whole = whole.trim_start_matches('0').to_owned();
    let fraction = fraction.trim_end_matches('0').to_owned();
    let zero = whole.is_empty() && fraction.is_empty();
    Some((negative && !zero, whole, fraction))
}

/// Compares two decimal renderings exactly; an unread or unparsable one is Unknown.
fn compare_numbers(
    left: Option<String>,
    right: Option<String>,
    accepts: fn(core::cmp::Ordering) -> bool,
) -> Option<bool> {
    let (left, right) = (number_parts(&left?)?, number_parts(&right?)?);
    let magnitude = left
        .1
        .len()
        .cmp(&right.1.len())
        .then_with(|| left.1.cmp(&right.1))
        .then_with(|| left.2.cmp(&right.2));
    let ordering = match (left.0, right.0) {
        (false, false) => magnitude,
        (true, true) => magnitude.reverse(),
        (true, false) => core::cmp::Ordering::Less,
        (false, true) => core::cmp::Ordering::Greater,
    };
    Some(accepts(ordering))
}
";

// ---- one command -------------------------------------------------------------------------------

/// Everything rendering one command's impl needs.
struct Writer<'a> {
    ir: &'a EssIr,
    layout: &'a Layout,
    command: &'a ResolvedCommand,
    storages: &'a BTreeMap<QualifiedName, String>,
    uses: &'a mut Uses,
    bounds: Bounds,
}

/// Where the values a branch reads are held while it is rendered.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Held {
    /// No row is in scope.
    None,
    /// The row a subject-guarded command read before selection, as `held`.
    Selected,
    /// Inside a move's wrong-state arm: `held_state` and `before`.
    WrongState,
}

impl Writer<'_> {
    /// The command's whole impl block.
    fn implementation(&mut self) -> String {
        let command = self.command;
        let type_name = self.layout.type_name(&command.name);
        let module = self
            .layout
            .module(self.layout.owner(&command.name))
            .to_owned();
        let method = name::value_ident(&type_name);
        let body = self.body();
        let mut bounds: Vec<String> = Vec::new();
        if self.bounds.context {
            bounds.push("Context".to_owned());
        }
        for entity in &self.bounds.storages {
            bounds.push(self.storages[entity].clone());
        }
        let clause = if bounds.is_empty() {
            String::new()
        } else {
            format!("\nwhere\n    P: {},", bounds.join(" + "))
        };
        format!(
            "\n/// `{}`, generated: every outcome is one the specification fully \
             determines.\nimpl<P> crate::{module}::obligations::{type_name}Behavior for \
             Generated<P>{clause}\n{{\n    fn {method}(&mut self, input: \
             crate::{module}::{type_name}) -> Result<crate::{module}::{type_name}Outcome, \
             UnmetObligation> {{\n        let _ = &input;\n{body}    }}\n}}\n",
            command.name
        )
    }

    /// The method body: selection in the precedence order, then the branch taken.
    // The precedence order, step by step, in the order it is evaluated.
    #[allow(clippy::too_many_lines)]
    fn body(&mut self) -> String {
        let command = self.command;
        let mut out = String::new();
        for outcome in &command.outcomes {
            if let (ResolvedCondition::When { predicate }, Some(_), None) =
                (&outcome.condition, &outcome.error, &outcome.subject)
            {
                let truth = self.predicate(&Env::Input(command), predicate);
                let guard = self.decided(&truth);
                let answer = self.variant(outcome, Held::None, None);
                let _ = writeln!(
                    out,
                    "        // `{}`: an input-guarded refusal, before anything else is read.\n        \
                     if {guard} {{\n            return Ok({answer});\n        }}",
                    outcome.name
                );
            }
        }
        let guarded = determined::subject_guarded(command);
        let held = if guarded {
            let subject = determined::selection_subject(command)
                .expect("a subject-guarded command the plan generates reads one subject");
            let entity = self.ir.entity(&subject.entity);
            let storage = self.storage(&entity.name);
            let unknown = self.unknown_answer();
            let _ = writeln!(
                out,
                "        // The addressed row, read before the branches that select by it.\n        \
                 let Some(held) = {storage}::get(&self.ports, &input.{}) else {{\n            \
                 return {unknown};\n        }};\n        let _ = &held;",
                name::value_ident(&subject.instance.field().name)
            );
            for outcome in &command.outcomes {
                let Some(guard) = self.subject_guard(outcome, entity) else {
                    continue;
                };
                let taken = self.take(outcome, Held::Selected);
                let _ = writeln!(
                    out,
                    "        // `{}`: selected by the addressed row.\n        if {guard} \
                     {{\n{taken}        }}",
                    outcome.name
                );
            }
            Held::Selected
        } else {
            Held::None
        };
        for outcome in &command.outcomes {
            match &outcome.condition {
                ResolvedCondition::When { predicate } if outcome.error.is_none() => {
                    let truth = self.predicate(&Env::Input(command), predicate);
                    let guard = self.decided(&truth);
                    let taken = self.take(outcome, held);
                    let _ = writeln!(
                        out,
                        "        // `{}`: an accepting branch, in declaration order.\n        if \
                         {guard} {{\n{taken}        }}",
                        outcome.name
                    );
                }
                ResolvedCondition::External { .. } => {
                    let ask = self.external(outcome);
                    let taken = self.take(outcome, held);
                    let _ = writeln!(
                        out,
                        "        // `{}`: an external branch, where the context takes it.\n        \
                         if {ask} {{\n{taken}        }}",
                        outcome.name
                    );
                }
                ResolvedCondition::ExternalWhen { predicate, .. } => {
                    let ask = self.external(outcome);
                    let truth = self.predicate(&Env::Input(command), predicate);
                    let guard = self.decided(&truth);
                    let taken = self.take(outcome, held);
                    let _ = writeln!(
                        out,
                        "        // `{}`: an external branch, where the context takes it and its \
                         input is eligible.\n        if {ask} && {guard} {{\n{taken}        }}",
                        outcome.name
                    );
                }
                _ => {}
            }
        }
        if let Some(default) = command
            .outcomes
            .iter()
            .find(|outcome| outcome.condition == ResolvedCondition::Otherwise)
        {
            // The branch statements are written for a block; the default is the body's tail.
            let taken = self
                .take(default, held)
                .lines()
                .fold(String::new(), |mut taken, line| {
                    let _ = writeln!(taken, "{}", line.strip_prefix("    ").unwrap_or(line));
                    taken
                });
            let _ = write!(out, "        // `{}`: the default.\n{taken}", default.name);
        } else {
            self.uses.helpers.insert("undeclared");
            let _ = writeln!(
                out,
                "        // No declared branch answers this request.\n        \
                 Err(undeclared(\"{}\"))",
                command.name
            );
        }
        out
    }

    /// The storage trait of an entity, recorded as used and as a bound of this impl.
    fn storage(&mut self, entity: &QualifiedName) -> String {
        self.uses.storages.insert(entity.clone());
        self.bounds.storages.insert(entity.clone());
        self.storages[entity].clone()
    }

    /// Asks the context whether an external branch is taken.
    fn external(&mut self, outcome: &ResolvedOutcome) -> String {
        self.uses.external = true;
        self.bounds.context = true;
        format!(
            "self.ports.external(\"{}\", \"{}\")",
            self.command.name, outcome.name
        )
    }

    /// `decided(<truth>, "<command>")?`.
    fn decided(&mut self, truth: &str) -> String {
        self.uses.helpers.insert("decided");
        self.uses.helpers.insert("undeclared");
        format!("decided({truth}, \"{}\")?", self.command.name)
    }

    /// The guard of a branch that selects by the addressed row, or `None` for any other branch.
    fn subject_guard(
        &mut self,
        outcome: &ResolvedOutcome,
        entity: &ResolvedEntity,
    ) -> Option<String> {
        let command = self.command;
        let state_enum = declared_path(self.layout, entity.state_type.name());
        let states = |states: &mut dyn Iterator<Item = &ess_domain::entity::StateName>| {
            let patterns: Vec<String> = states
                .map(|state| format!("{state_enum}::{}", name::pascal(state.as_str())))
                .collect();
            format!("Some(matches!(held.state, {}))", patterns.join(" | "))
        };
        let (row, input) = match &outcome.condition {
            ResolvedCondition::SubjectField {
                field,
                equals,
                predicate,
            } => {
                let path = ess_primitives::facts::FactPath::new(field)
                    .expect("the compiler admitted the field");
                let compare = Predicate::Compare {
                    left: Operand::Fact(path),
                    op: CompareOp::Eq,
                    right: Operand::Literal(FactValue::Text(equals.clone())),
                };
                (
                    self.predicate(&Env::Subject(command, entity), &compare),
                    predicate.as_ref(),
                )
            }
            ResolvedCondition::SubjectPredicate { predicate, input } => (
                self.predicate(&Env::Subject(command, entity), predicate),
                input.as_ref(),
            ),
            ResolvedCondition::SubjectState { state, predicate } => {
                (states(&mut state.iter()), predicate.as_ref())
            }
            ResolvedCondition::StateChange {
                states: held,
                predicate,
                ..
            } => (states(&mut held.iter()), predicate.as_ref()),
            _ => return None,
        };
        let truth = match input {
            Some(input) => {
                self.uses.helpers.insert("all");
                format!(
                    "all(&[{row}, {}])",
                    self.predicate(&Env::Input(command), input)
                )
            }
            None => row,
        };
        Some(self.decided(&truth))
    }

    /// The answer for an identity no record carries.
    fn unknown_answer(&mut self) -> String {
        let command = self.command;
        let outcome_enum = format!("{}Outcome", declared_path(self.layout, &command.name));
        if let Some(declared) = command
            .outcomes
            .iter()
            .find(|outcome| outcome.condition == ResolvedCondition::UnknownInstance)
        {
            return format!("Ok({})", self.variant(declared, Held::None, None));
        }
        if let Some(declared) = ess_gen::unknown_instance::unknown_instance_answer(self.ir, command)
        {
            return format!(
                "Ok({outcome_enum}::{})",
                super::items::unknown_instance_variant(declared)
            );
        }
        if let Some(declared) = determined::unknown_answer(command) {
            format!("Ok({})", self.variant(declared, Held::None, None))
        } else {
            self.uses.helpers.insert("undeclared");
            format!("Err(undeclared(\"{}\"))", command.name)
        }
    }

    /// The answer for a move from a state it does not start in.
    fn wrong_state_answer(&mut self) -> String {
        let command = self.command;
        if let Some(declared) = command
            .outcomes
            .iter()
            .find(|outcome| outcome.condition == ResolvedCondition::WrongState)
        {
            format!("Ok({})", self.variant(declared, Held::WrongState, None))
        } else {
            self.uses.helpers.insert("undeclared");
            format!("Err(undeclared(\"{}\"))", command.name)
        }
    }

    /// The statements taking one selected branch, each path ending in a `return`.
    // One arm per effect, each ending in the answer it takes.
    #[allow(clippy::too_many_lines)]
    fn take(&mut self, outcome: &ResolvedOutcome, held: Held) -> String {
        let mut out = String::new();
        let Some(subject) = &outcome.subject else {
            let answer = self.variant(outcome, held, None);
            let _ = writeln!(out, "            return Ok({answer});");
            return out;
        };
        let entity = self.ir.entity(&subject.entity);
        let storage = self.storage(&entity.name);
        let module_entity = declared_path(self.layout, &entity.name);
        let any = format!(
            "crate::{}::Any{}",
            self.layout.module(self.layout.owner(&entity.name)),
            self.layout.type_name(&entity.name)
        );
        match (&subject.effect, &subject.instance) {
            (ResolvedEffect::Creates, ResolvedInstance::Observed { .. }) => {
                let identity_type = &entity.identity.type_ref;
                let generate = self.generate(identity_type);
                let _ = writeln!(
                    out,
                    "            let identity: {} = {generate};",
                    self.layout.absolute_type(identity_type)
                );
                let mut fields = vec![format!(
                    "                {}: identity.clone(),",
                    name::value_ident(&entity.identity.name)
                )];
                for field in &entity.fields {
                    let value = match outcome.sets.iter().find(|set| set.target == field.name) {
                        Some(set) => self.value(set, None),
                        None => "None".to_owned(),
                    };
                    fields.push(format!(
                        "                {}: {value},",
                        name::value_ident(&field.name)
                    ));
                }
                let _ = writeln!(
                    out,
                    "            let data = {module_entity}Data {{\n{}\n            }};",
                    fields.join("\n")
                );
                out.push_str(&Self::invariant_check(entity, "data"));
                let state = subject
                    .into
                    .clone()
                    .unwrap_or_else(|| entity.lifecycle.initial.clone());
                let constructor = if state == entity.lifecycle.initial {
                    "new".to_owned()
                } else {
                    name::value_ident(&format!("new-{state}"))
                };
                let _ = writeln!(
                    out,
                    "            {storage}::put(&mut self.ports, \
                     {any}::{state}({module_entity}::{constructor}(data)).snapshot());"
                );
                let answer = self.variant(outcome, held, Some("identity"));
                let _ = writeln!(out, "            return Ok({answer});");
            }
            (effect, ResolvedInstance::Supplied { field }) => {
                let unknown = self.unknown_answer();
                let identity = name::value_ident(&field.name);
                let _ = writeln!(
                    out,
                    "            let Some(held) = {storage}::get(&self.ports, &input.{identity}) \
                     else {{\n                return {unknown};\n            }};\n            let \
                     _ = &held;"
                );
                let reads_before = outcome_reads_before(outcome);
                match effect {
                    ResolvedEffect::Moves { transition } => {
                        let wrong = self.wrong_state_answer();
                        let wrong_reads = self.wrong_state_reads(entity);
                        if wrong_reads.0 {
                            out.push_str("            let held_state = held.state;\n");
                        }
                        if reads_before || wrong_reads.1 {
                            out.push_str("            let before = held.data.clone();\n");
                        }
                        let _ = writeln!(out, "            let moved = match held.refine() {{");
                        for from in &transition.from {
                            let _ = writeln!(
                                out,
                                "                {any}::{from}(instance) => \
                                 {any}::{}(instance.{}()),",
                                transition.to,
                                name::value_ident(&transition.name)
                            );
                        }
                        if transition.from.len() < entity.lifecycle.states.len() {
                            let _ = writeln!(out, "                _ => return {wrong},");
                        }
                        out.push_str("            };\n");
                        self.write_sets(&mut out, outcome, "moved.snapshot()", entity);
                        let _ = writeln!(out, "            {storage}::put(&mut self.ports, next);");
                    }
                    ResolvedEffect::Updates | ResolvedEffect::Preserves => {
                        if reads_before {
                            out.push_str("            let before = held.data.clone();\n");
                        }
                        let writes =
                            matches!(effect, ResolvedEffect::Updates) || !outcome.sets.is_empty();
                        if writes {
                            self.write_sets(&mut out, outcome, "held", entity);
                            let _ =
                                writeln!(out, "            {storage}::put(&mut self.ports, next);");
                        }
                    }
                    ResolvedEffect::Deletes => {
                        if reads_before {
                            out.push_str("            let before = held.data.clone();\n");
                        }
                        let _ = writeln!(
                            out,
                            "            {storage}::delete(&mut self.ports, &input.{identity});"
                        );
                    }
                    ResolvedEffect::Creates => {
                        unreachable!("a creation whose identity is supplied is an obligation")
                    }
                }
                let answer = self.variant(outcome, held, None);
                let _ = writeln!(out, "            return Ok({answer});");
            }
            (_, ResolvedInstance::Observed { .. }) => {
                unreachable!("an observed identity outside `creates:` is an obligation")
            }
        }
        out
    }

    /// Whether the wrong-state answer reads the held state, and whether it reads stored fields.
    fn wrong_state_reads(&self, entity: &ResolvedEntity) -> (bool, bool) {
        let Some(declared) = self
            .command
            .outcomes
            .iter()
            .find(|outcome| outcome.condition == ResolvedCondition::WrongState)
        else {
            return (false, false);
        };
        let Some(error) = &declared.error else {
            return (false, false);
        };
        let mut reads = (false, false);
        for field in &self.ir.error(error).fields {
            // A declared source (ess/19) reads the row's data where it reads the subject.
            if let Some(source) = declared
                .error_payload
                .iter()
                .find(|source| source.target == field.name)
            {
                reads.1 |= reads_subject(&source.value);
                continue;
            }
            match determined::held_field(entity, field) {
                Some(HeldField::State) => reads.0 = true,
                Some(HeldField::Field(_)) => reads.1 = true,
                None => {}
            }
        }
        reads
    }

    /// `let next = <from>;`, the branch's `sets:`, and the invariant check.
    fn write_sets(
        &mut self,
        out: &mut String,
        outcome: &ResolvedOutcome,
        from: &str,
        entity: &ResolvedEntity,
    ) {
        let mutable = if outcome.sets.is_empty() { "" } else { "mut " };
        let _ = writeln!(out, "            let {mutable}next = {from};");
        for set in &outcome.sets {
            let value = self.value(set, Some("before"));
            let _ = writeln!(
                out,
                "            next.data.{} = {value};",
                name::value_ident(&set.target)
            );
        }
        out.push_str(&Self::invariant_check(entity, "next.data"));
    }

    /// The entity invariant check before a write, where the entity declares any.
    ///
    /// Calls `broken_invariant`, which `story:generated-invariant-check` generates on every entity
    /// data type declaring `invariants:`; an entity declaring none has nothing to check.
    fn invariant_check(entity: &ResolvedEntity, data: &str) -> String {
        if entity.invariants.is_empty() {
            return String::new();
        }
        format!(
            "            if let Some(broken) = {data}.broken_invariant() {{\n                let \
             capability = \"entity invariant\";\n                return Err(UnmetObligation {{ \
             capability, source: broken }});\n            }}\n"
        )
    }

    /// The outcome variant, with its events and error, as an expression.
    fn variant(&mut self, outcome: &ResolvedOutcome, held: Held, created: Option<&str>) -> String {
        let command = self.command;
        let outcome_enum = format!("{}Outcome", declared_path(self.layout, &command.name));
        let variant = format!("{outcome_enum}::{}", name::pascal(outcome.name.as_str()));
        let emit = super::Emit {
            ir: self.ir,
            layout: self.layout,
            domain: self.layout.owner(&command.name),
        };
        let carried = super::items::outcome_event_fields(&emit, outcome);
        if carried.is_empty() && outcome.error.is_none() {
            return variant;
        }
        let mut members = Vec::new();
        for field in carried {
            let event = self.event(outcome, field.event, created);
            members.push(format!("{}: {event}", field.field));
        }
        if let Some(error) = &outcome.error {
            let declared = self.ir.error(error);
            let path = declared_path(self.layout, error.name());
            if declared.fields.is_empty() {
                members.push(format!("error: {path}"));
            } else {
                let entity = match held {
                    Held::Selected => determined::selection_subject(command)
                        .map(|subject| self.ir.entity(&subject.entity)),
                    Held::WrongState => determined::wrong_state_entity(self.ir, command),
                    Held::None => None,
                };
                // Where `{subject: …}` reads the row the refusal is answered for.
                let row = match held {
                    Held::Selected => Some("held.data"),
                    Held::WrongState => Some("before"),
                    Held::None => None,
                };
                let mut fields = Vec::new();
                for field in &declared.fields {
                    // A declared source first (ess/19, `story:error-payload-sources`), else the
                    // held row, as the plan admitted.
                    if let Some(source) = outcome
                        .error_payload
                        .iter()
                        .find(|source| source.target == field.name)
                    {
                        let value = self.value(source, row);
                        fields.push(format!("{}: {value}", name::value_ident(&field.name)));
                        continue;
                    }
                    let entity =
                        entity.expect("an error field with no source is answered from a held row");
                    let value = match (
                        determined::held_field(entity, field)
                            .expect("the plan admitted only held error fields"),
                        held,
                    ) {
                        (HeldField::State, Held::WrongState) => "held_state".to_owned(),
                        (HeldField::State, _) => "held.state".to_owned(),
                        (HeldField::Field(stored), Held::WrongState) => {
                            format!("before.{}.clone()", name::value_ident(&stored.name))
                        }
                        (HeldField::Field(stored), _) => {
                            format!("held.data.{}.clone()", name::value_ident(&stored.name))
                        }
                    };
                    fields.push(format!("{}: {value}", name::value_ident(&field.name)));
                }
                members.push(format!("error: {path} {{ {} }}", fields.join(", ")));
            }
        }
        format!("{variant} {{ {} }}", members.join(", "))
    }

    /// One emitted event, every field filled.
    fn event(
        &mut self,
        outcome: &ResolvedOutcome,
        event: &ess_compiler::ir::EventHandle,
        created: Option<&str>,
    ) -> String {
        let declared = self.ir.event(event);
        let path = declared_path(self.layout, event.name());
        if declared.fields.is_empty() {
            return path;
        }
        let determined = outcome
            .payload
            .iter()
            .find(|payload| &payload.event == event);
        let instance = outcome
            .subject
            .as_ref()
            .and_then(|subject| match &subject.instance {
                ResolvedInstance::Observed {
                    event: published,
                    field,
                } if published == event => Some(field.name.as_str()),
                _ => None,
            });
        let before = outcome
            .subject
            .as_ref()
            .is_some_and(|subject| !matches!(subject.effect, ResolvedEffect::Creates))
            .then_some("before");
        let mut fields = Vec::new();
        for field in &declared.fields {
            let source = determined
                .and_then(|payload| payload.fields.iter().find(|it| it.target == field.name));
            let value = match (instance == Some(field.name.as_str()), created, source) {
                (true, Some(created), _) => format!("{created}.clone()"),
                (_, _, Some(source)) => self.value(source, before),
                (_, _, None) => self.assigned(&field.type_ref),
            };
            fields.push(format!("{}: {value}", name::value_ident(&field.name)));
        }
        format!("{path} {{ {} }}", fields.join(", "))
    }

    /// A value the implementation assigns: absent where optional, else from the context.
    fn assigned(&mut self, target: &ResolvedTypeRef) -> String {
        if target.is_optional() {
            "None".to_owned()
        } else {
            self.generate(target)
        }
    }

    /// `self.ports.generate_<type>()`, recorded on the context.
    fn generate(&mut self, target: &ResolvedTypeRef) -> String {
        let fragment = name::type_fragment(&target.to_string());
        let method = name::value_ident(&format!("generate{fragment}"));
        self.uses.generates.insert(
            method.clone(),
            (self.layout.absolute_type(target), target.to_string()),
        );
        self.bounds.context = true;
        format!("self.ports.{method}()")
    }

    /// `self.ports.caller_<attribute>()`, recorded on the context: an `Option` of the attribute.
    fn caller(&mut self, attribute: &str, type_ref: &ResolvedTypeRef) -> String {
        let method = name::value_ident(&format!("caller_{attribute}"));
        self.uses.callers.insert(
            method.clone(),
            (self.layout.absolute_type(type_ref), attribute.to_owned()),
        );
        self.bounds.context = true;
        format!("self.ports.{method}()")
    }

    /// The value one source fills its field with. `before` names the held data before the
    /// outcome, where the branch holds a row.
    // One arm per value source the model declares, each rendering its own expression.
    #[allow(clippy::too_many_lines)]
    fn value(&mut self, field: &ResolvedPayloadField, before: Option<&str>) -> String {
        let target = &field.target_type;
        let wrap = |source: &ResolvedTypeRef, expression: String| {
            if source == target {
                expression
            } else {
                format!("Some({expression})")
            }
        };
        match &field.value {
            ResolvedPayloadValue::InputField {
                field: source,
                type_ref,
            } => wrap(
                type_ref,
                format!("input.{}.clone()", name::value_ident(source)),
            ),
            ResolvedPayloadValue::SubjectField {
                field: source,
                type_ref,
            } => wrap(
                type_ref,
                format!(
                    "{}.{}.clone()",
                    before.expect("the plan admits `{subject:}` only where a row is held"),
                    name::value_ident(source)
                ),
            ),
            ResolvedPayloadValue::CallerAttribute {
                attribute,
                type_ref,
            } => {
                let read = self.caller(attribute, type_ref);
                if type_ref == target {
                    self.uses.helpers.insert("undeclared");
                    format!(
                        "{read}.ok_or_else(|| undeclared(\"{}\"))?",
                        self.command.name
                    )
                } else {
                    read
                }
            }
            ResolvedPayloadValue::Literal { value } => literal(self.ir, self.layout, target, value),
            ResolvedPayloadValue::Generated => self.assigned(target),
            ResolvedPayloadValue::Cleared => "None".to_owned(),
            ResolvedPayloadValue::Increment { by } => {
                let before = before.expect("the plan admits `{increment:}` only on a held row");
                increment(
                    self.ir,
                    self.layout,
                    target,
                    &format!("{before}.{}", name::value_ident(&field.target)),
                    by,
                )
            }
            ResolvedPayloadValue::InputOrGenerated {
                field: source,
                otherwise,
                ..
            } => {
                let read = format!("input.{}.clone()", name::value_ident(source));
                if target.is_optional() && otherwise.is_none() {
                    return read;
                }
                let required = target.required();
                let fallback = match otherwise {
                    Some(text) => literal(self.ir, self.layout, required, text),
                    None => self.generate(required),
                };
                let chosen = format!("match {read} {{ Some(value) => value, None => {fallback} }}");
                if target.is_optional() {
                    format!("Some({chosen})")
                } else {
                    chosen
                }
            }
            ResolvedPayloadValue::Struct { fields } => {
                let ResolvedTypeRef::Declared { name: declared } = target.required() else {
                    unreachable!("the plan admits a struct source only for a struct")
                };
                let ResolvedBody::Struct {
                    fields: members, ..
                } = &self.ir.named_type(declared).body
                else {
                    unreachable!("the plan admits a struct source only for a struct")
                };
                let mut rendered = Vec::new();
                for member in members {
                    let value = match fields.iter().find(|source| source.target == member.name) {
                        Some(source) => self.value(source, before),
                        None => "None".to_owned(),
                    };
                    rendered.push(format!("{}: {value}", name::value_ident(&member.name)));
                }
                let built = format!(
                    "{} {{ {} }}",
                    self.layout.absolute_type(&ResolvedTypeRef::Declared {
                        name: declared.clone()
                    }),
                    rendered.join(", ")
                );
                if target.is_optional() {
                    format!("Some({built})")
                } else {
                    built
                }
            }
            ResolvedPayloadValue::RelatedField { .. }
            | ResolvedPayloadValue::ChangedCount
            | ResolvedPayloadValue::ResponseField { .. } => {
                unreachable!("the plan keeps this source's command an obligation")
            }
        }
    }

    // ---- guards ----------------------------------------------------------------------------------

    /// A predicate as an `Option<bool>` expression: `None` is Unknown.
    fn predicate(&mut self, env: &Env<'_>, predicate: &Predicate) -> String {
        match predicate {
            Predicate::Always => "Some(true)".to_owned(),
            Predicate::Never => "Some(false)".to_owned(),
            Predicate::All(children) | Predicate::Any(children) => {
                let helper = if matches!(predicate, Predicate::All(_)) {
                    "all"
                } else {
                    "any"
                };
                self.uses.helpers.insert(helper);
                let rendered: Vec<String> = children
                    .iter()
                    .map(|child| self.predicate(env, child))
                    .collect();
                format!("{helper}(&[{}])", rendered.join(", "))
            }
            Predicate::Not(child) => {
                format!("({}).map(|value| !value)", self.predicate(env, child))
            }
            Predicate::Defined(path) => {
                let resolved = self.resolve(env, path);
                format!("Some({}.is_some())", self.reference(&resolved))
            }
            Predicate::Truthy(path) => {
                let resolved = self.resolve(env, path);
                self.read(&resolved)
            }
            Predicate::Compare { left, op, right } => {
                let kind = [left, right]
                    .into_iter()
                    .find_map(|operand| match operand {
                        Operand::Fact(path) => Some(self.resolve(env, path).kind),
                        Operand::Literal(_) => None,
                    })
                    .expect("the plan admits comparisons reading a fact");
                let left = self.operand(env, left, &kind);
                let right = self.operand(env, right, &kind);
                if let Kind::Number(_) = kind {
                    self.uses.helpers.insert("compare_numbers");
                    let accepts = match op {
                        CompareOp::Eq => "is_eq",
                        CompareOp::Ne => "is_ne",
                        CompareOp::Lt => "is_lt",
                        CompareOp::Le => "is_le",
                        CompareOp::Gt => "is_gt",
                        CompareOp::Ge => "is_ge",
                    };
                    format!("compare_numbers({left}, {right}, core::cmp::Ordering::{accepts})")
                } else {
                    self.uses.helpers.insert("equal");
                    let equal = format!("equal({left}, {right})");
                    if *op == CompareOp::Ne {
                        format!("{equal}.map(|value| !value)")
                    } else {
                        equal
                    }
                }
            }
            Predicate::AnyOf { path, values } | Predicate::NoneOf { path, values } => {
                let resolved = self.resolve(env, path);
                self.uses.helpers.insert("any");
                self.uses.helpers.insert("equal");
                let checks: Vec<String> = values
                    .iter()
                    .map(|value| {
                        format!(
                            "equal({}, {})",
                            self.read(&resolved),
                            fact_literal(value, &resolved.kind)
                        )
                    })
                    .collect();
                let any = format!("any(&[{}])", checks.join(", "));
                if matches!(predicate, Predicate::NoneOf { .. }) {
                    format!("({any}).map(|value| !value)")
                } else {
                    any
                }
            }
            Predicate::TextMatch { path, op, value } => {
                let resolved = self.resolve(env, path);
                let FactValue::Text(literal) = value else {
                    unreachable!("the plan admits text tests of text literals")
                };
                format!(
                    "{}.map(|value| value.{}({literal:?}))",
                    self.read(&resolved),
                    op.keyword()
                )
            }
            _ => unreachable!("the plan admits only the guards `determined::supported` names"),
        }
    }

    /// Resolves a path the plan already checked.
    fn resolve(&self, env: &Env<'_>, path: &ess_primitives::facts::FactPath) -> Resolved {
        determined::resolve(self.ir, env, path)
            .expect("the plan admitted only guards whose paths resolve")
    }

    /// One comparison operand, normalized to what the comparison reads.
    fn operand(&mut self, env: &Env<'_>, operand: &Operand, kind: &Kind) -> String {
        match operand {
            Operand::Fact(path) => {
                let resolved = self.resolve(env, path);
                self.read(&resolved)
            }
            Operand::Literal(value) => fact_literal(value, kind),
        }
    }

    /// A path as `Option<&T>` at its end, before any normalization.
    fn reference(&mut self, resolved: &Resolved) -> String {
        let mut out = match &resolved.root {
            Root::Input(field) => format!("Some(&input.{})", name::value_ident(field)),
            Root::Stored(field) => format!("Some(&held.data.{})", name::value_ident(field)),
            Root::State => "Some(&held.state)".to_owned(),
            Root::Caller(attribute) => {
                format!("{}.as_ref()", self.caller(attribute, &resolved.root_type))
            }
        };
        for step in &resolved.steps {
            match step {
                Step::Optional => out.push_str(".and_then(|value| value.as_ref())"),
                Step::Newtype => out.push_str(".map(|value| &value.0)"),
                Step::Field(field) => {
                    let _ = write!(out, ".map(|value| &value.{})", name::value_ident(field));
                }
                Step::Count => {}
            }
        }
        out
    }

    /// A path as the `Option` of what its kind compares: a decimal rendering for a number, the text
    /// of a string, identity or enum, a `bool`.
    fn read(&mut self, resolved: &Resolved) -> String {
        let reference = self.reference(resolved);
        if resolved.steps.last() == Some(&Step::Count) {
            return format!("{reference}.map(|value| value.len().to_string())");
        }
        let leaf = leaf_type(self.ir, resolved);
        match &resolved.kind {
            Kind::Number(Primitive::Decimal) => format!("{reference}.map(|value| value.0.clone())"),
            Kind::Number(_) => format!("{reference}.map(|value| value.to_string())"),
            Kind::Text => match leaf {
                Some(Primitive::String) => format!("{reference}.map(|value| value.clone())"),
                _ => format!("{reference}.map(|value| value.0.clone())"),
            },
            Kind::Bool => format!("{reference}.map(|value| *value)"),
            Kind::Enum(type_ref, variants) => {
                let path = self.layout.absolute_type(type_ref);
                let arms: Vec<String> = variants
                    .iter()
                    .map(|variant| format!("{path}::{} => {variant:?}", name::pascal(variant)))
                    .collect();
                format!(
                    "{reference}.map(|value| match value {{ {} }}.to_owned())",
                    arms.join(", ")
                )
            }
            Kind::State => {
                let ResolvedTypeRef::Declared { name: state_type } = &resolved.root_type else {
                    unreachable!("a state path starts at the lifecycle's enum")
                };
                let path = declared_path(self.layout, state_type.name());
                let ResolvedBody::Enum { variants } = &self.ir.named_type(state_type).body else {
                    unreachable!("a lifecycle's state type is an enum")
                };
                let arms: Vec<String> = variants
                    .iter()
                    .map(|variant| {
                        format!(
                            "{path}::{} => {:?}",
                            name::pascal(&variant.name),
                            variant.name
                        )
                    })
                    .collect();
                format!(
                    "{reference}.map(|value| match value {{ {} }}.to_owned())",
                    arms.join(", ")
                )
            }
            Kind::Opaque => unreachable!("the plan admits no comparison of an opaque value"),
        }
    }
}

/// The primitive a resolved path ends in, under its newtypes.
fn leaf_type(ir: &EssIr, resolved: &Resolved) -> Option<Primitive> {
    let mut current = resolved.root_type.clone();
    for step in &resolved.steps {
        current = match (step, &current) {
            (Step::Optional, ResolvedTypeRef::Optional { of }) => (**of).clone(),
            (Step::Newtype, ResolvedTypeRef::Declared { name }) => {
                match &ir.named_type(name).body {
                    ResolvedBody::Newtype { of, .. } => of.clone(),
                    _ => return None,
                }
            }
            (Step::Field(field), ResolvedTypeRef::Declared { name }) => {
                match &ir.named_type(name).body {
                    ResolvedBody::Struct { fields, .. } => fields
                        .iter()
                        .find(|member| &member.name == field)?
                        .type_ref
                        .clone(),
                    _ => return None,
                }
            }
            _ => return None,
        };
    }
    match current {
        ResolvedTypeRef::Primitive { name } => Some(name),
        _ => None,
    }
}

/// A literal operand, in the normalized form its comparison reads.
fn fact_literal(value: &FactValue, kind: &Kind) -> String {
    match (value, kind) {
        (FactValue::Bool(value), _) => format!("Some({value})"),
        (FactValue::Number(number), _) => format!("Some({:?}.to_owned())", number.to_string()),
        (FactValue::Text(text), _) => format!("Some({text:?}.to_owned())"),
    }
}

/// `true` where the branch reads the held row as it was before the outcome.
fn outcome_reads_before(outcome: &ResolvedOutcome) -> bool {
    outcome.sets.iter().any(|set| reads_subject(&set.value))
        || outcome
            .payload
            .iter()
            .flat_map(|payload| &payload.fields)
            .any(|field| reads_subject(&field.value))
}

/// Whether one value source reads the row as it was before the outcome.
fn reads_subject(value: &ResolvedPayloadValue) -> bool {
    match value {
        ResolvedPayloadValue::SubjectField { .. } | ResolvedPayloadValue::Increment { .. } => true,
        ResolvedPayloadValue::Struct { fields } => {
            fields.iter().any(|field| reads_subject(&field.value))
        }
        _ => false,
    }
}

/// A literal the model writes, as a value of `target`.
fn literal(ir: &EssIr, layout: &Layout, target: &ResolvedTypeRef, text: &str) -> String {
    match target {
        ResolvedTypeRef::Optional { of } => format!("Some({})", literal(ir, layout, of, text)),
        ResolvedTypeRef::Primitive { name } => match name {
            Primitive::Integer | Primitive::Boolean => text.to_owned(),
            Primitive::String => format!("{text:?}.to_owned()"),
            Primitive::Decimal | Primitive::Uuid | Primitive::Timestamp | Primitive::Duration => {
                format!("{}({text:?}.to_owned())", super::layout::primitive(*name))
            }
            _ => unreachable!("the plan admits literals of these primitives only"),
        },
        ResolvedTypeRef::Declared { name } => {
            let path = layout.absolute_type(target);
            match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => {
                    format!("{path}({})", literal(ir, layout, of, text))
                }
                ResolvedBody::Enum { .. } => format!("{path}::{}", name::pascal(text)),
                _ => unreachable!("the plan admits literals of scalars only"),
            }
        }
        _ => unreachable!("the plan admits literals of scalars only"),
    }
}

/// The previous value of `read` plus `by`, through the newtypes of `target`.
fn increment(
    ir: &EssIr,
    layout: &Layout,
    target: &ResolvedTypeRef,
    read: &str,
    by: &str,
) -> String {
    match target {
        ResolvedTypeRef::Primitive { .. } => format!("{read} + {by}"),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => format!(
                "{}({})",
                layout.absolute_type(target),
                increment(ir, layout, of, &format!("{read}.0"), by)
            ),
            _ => unreachable!("the plan admits `{{increment:}}` of an Integer only"),
        },
        _ => unreachable!("the plan admits `{{increment:}}` of an Integer only"),
    }
}
