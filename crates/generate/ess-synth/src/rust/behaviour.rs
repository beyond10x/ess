//! Generated behaviour: what the specification fully determines, written against generated ports.
//!
//! A command the plan marks generated (`crate::determined`) gets an implementation of its existing
//! `…Behavior` trait on [`Generated<P>`](module), so the component ports that already take a
//! behaviour bundle take this one unchanged, and an implementor who wants a different behaviour
//! still writes their own bundle. Everything the specification leaves to the implementor is a
//! port `P` supplies: one storage trait per entity (get, put and delete a snapshot by identity —
//! generated network entries supply an ephemeral store) and one context trait (the caller's attributes, the
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
//!
//! A command guarded by `when_related:` reads its one related row through the related entity's
//! storage port, and none for an absent reference, in the interpreter's order: a row named by the
//! input after `existing_instance:` and before the input-guarded refusals, its `exists: false`
//! answering an identity no row carries; a row named by a stored field of the addressed subject
//! after the input-guarded refusals and that row's existence and held state. Where the
//! present-related refusals come first (every stored reference; from ess/22 beside `wrong_state:`),
//! the branch the request selects without them has its subject's existence and held state checked
//! before them, and they answer before every accepting branch; otherwise the predicate branches are
//! read in declaration order with the accepting ones.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_compiler::ir::{
    EntityHandle, EssIr, ResolvedBody, ResolvedCommand, ResolvedCondition, ResolvedEffect,
    ResolvedEntity, ResolvedInstance, ResolvedOutcome, ResolvedPayloadField, ResolvedPayloadValue,
    ResolvedRelatedTest, ResolvedRelatedVia, ResolvedTypeRef,
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

mod query;

/// `true` where the workspace carries a `behaviour` module: some command's behaviour or some view's
/// query is generated.
pub(crate) fn used(ir: &EssIr) -> bool {
    determined::any_generated(ir) || crate::view_query::any_generated(ir)
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
    let external_commands = external_names(ir, plan);
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
                external_commands: &external_commands,
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
        let source = view.name.to_string();
        if plan.is_generated(CapabilityKind::ViewQuery, &source) {
            covered.insert(Capability {
                kind: CapabilityKind::ViewQuery,
                source,
            });
            impls.push_str(&query::implementation(
                ir, layout, view, &storages, &mut uses,
            ));
        } else if plan
            .obligation_of(CapabilityKind::ViewQuery, &source)
            .is_some()
        {
            forward_query(&mut impls, layout, view);
        }
    }

    let mut out = provenance.commented_for("//", REGENERATE);
    out.push_str(HEADER);
    for entity in &uses.storages {
        storage_trait(
            &mut out,
            ir,
            layout,
            &storages,
            entity,
            uses.listed.contains(entity),
        );
    }
    external_command(&mut out, layout, &external_commands);
    context_trait(&mut out, &uses);
    fallible_context(&mut out, &uses);
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
//! preserves the trait; generated network entries supply an ephemeral store. `Context` carries the caller's attributes,
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
/// `TryContext` (or its legacy `Context` blanket adapter) where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
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
pub(super) struct Uses {
    /// Entities whose storage trait some behaviour or query uses.
    pub(super) storages: BTreeSet<QualifiedName>,
    /// Entities whose rows some generated query lists: their storage trait carries `list`.
    pub(super) listed: BTreeSet<QualifiedName>,
    /// Caller attribute methods: name → returned type.
    pub(super) callers: BTreeMap<String, (String, String)>,
    /// Assigned-value methods: name → returned type.
    pub(super) generates: BTreeMap<String, (String, String)>,
    pub(super) assigned: BTreeMap<String, ResolvedTypeRef>,
    /// Some behaviour asks the context about an `external:` branch.
    pub(super) external: bool,
    pub(super) externals: BTreeSet<String>,
    /// Some behaviour reads the command clock: a guard of it orders an instant against the current
    /// time (ess/22, family F A3).
    pub(super) clock: bool,
    /// Helper functions used, by name.
    helpers: BTreeSet<&'static str>,
}

/// Ask the very same renderers for the ports a selected set of methods reads.
pub(super) fn requirements(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
    selected: Option<&crate::served::Reachable>,
) -> Uses {
    let storages = storage_names(ir, layout);
    let external_commands = external_names(ir, plan);
    let mut uses = Uses::default();
    for command in ir.commands().values() {
        if plan.is_generated(CapabilityKind::CommandBehavior, &command.name.to_string())
            && selected.is_none_or(|selected| selected.commands.contains(&command.name))
        {
            let _ = Writer {
                ir,
                layout,
                command,
                storages: &storages,
                external_commands: &external_commands,
                uses: &mut uses,
                bounds: Bounds::default(),
            }
            .implementation();
        }
    }
    for view in ir.views().values() {
        if plan.is_generated(CapabilityKind::ViewQuery, &view.name.to_string())
            && selected.is_none_or(|selected| selected.views.contains(&view.name))
        {
            let _ = query::implementation(ir, layout, view, &storages, &mut uses);
        }
    }
    uses
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
pub(super) fn storage_names(ir: &EssIr, layout: &Layout) -> BTreeMap<QualifiedName, String> {
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

/// Names for commands whose generated body calls the external port. Reserve every base first so
/// a collision suffix cannot steal another command's natural name.
fn external_names(ir: &EssIr, plan: &SynthesisPlan) -> BTreeMap<QualifiedName, String> {
    let candidates: BTreeMap<_, _> = ir
        .commands()
        .values()
        .filter(|command| {
            plan.is_generated(CapabilityKind::CommandBehavior, &command.name.to_string())
                && command.outcomes.iter().any(|outcome| {
                    matches!(
                        outcome.condition,
                        ResolvedCondition::External { .. } | ResolvedCondition::ExternalWhen { .. }
                    )
                })
        })
        .map(|command| {
            (
                command.name.clone(),
                name::type_fragment(&command.name.to_string()),
            )
        })
        .collect();
    let mut reserved: BTreeSet<String> = candidates.values().cloned().collect();
    let mut allocated = BTreeSet::new();
    candidates
        .into_iter()
        .map(|(command, base)| {
            let mut variant = base.clone();
            if !allocated.insert(base.clone()) {
                let mut suffix = 2;
                loop {
                    variant = format!("{base}{suffix}");
                    if reserved.insert(variant.clone()) {
                        break;
                    }
                    suffix += 1;
                }
            }
            (command, variant)
        })
        .collect()
}

/// The executing input, borrowed without conversion; absent when no generated command asks it.
fn external_command(out: &mut String, layout: &Layout, commands: &BTreeMap<QualifiedName, String>) {
    if commands.is_empty() {
        return;
    }
    out.push_str("\n/// The exact executing command input supplied to an external decision.\n///\n/// This supplies facts, not authority: the context must verify its request-bound proof.\n#[derive(Debug, Clone, Copy)]\npub enum ExternalCommand<'a> {\n");
    for (command, variant) in commands {
        let module = layout.module(layout.owner(command));
        let input = layout.type_name(command);
        let _ = writeln!(out, "    /// The executing `{command}` input.\n    {variant}(&'a crate::{module}::{input}),");
    }
    out.push_str("}\n\nimpl ExternalCommand<'_> {\n    /// The canonical qualified identity of this command.\n    pub fn name(&self) -> &'static str {\n        match self {\n");
    for (command, variant) in commands {
        let _ = writeln!(out, "            Self::{variant}(_) => \"{command}\",");
    }
    out.push_str("        }\n    }\n}\n");
}

/// One entity's storage port.
fn storage_trait(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    storages: &BTreeMap<QualifiedName, String>,
    entity: &QualifiedName,
    listed: bool,
) {
    let declared = &ir.entities()[entity];
    let snapshot = snapshot_path(layout, entity);
    let identity = layout.absolute_type(&declared.identity.type_ref);
    let list = if listed {
        format!(
            "\n\n    /// Every stored instance, in the order the store keeps them: the order a \
             generated query\n    /// answers an unordered view in.\n    fn list(&self) -> \
             Vec<{snapshot}>;"
        )
    } else {
        String::new()
    };
    let _ = writeln!(
        out,
        "\n/// Where `{entity}` is stored — a port the implementor provides.\n///\n/// Keyed by the \
         identity `{}`. Generated network entries supply an ephemeral implementation; durable storage remains a port.\npub trait {} \
         {{\n    /// The instance with this identity, or `None` where none is stored.\n    \
         fn get(&self, identity: &{identity}) -> Option<{snapshot}>;\n\n    /// Stores this \
         instance under its identity, replacing what was held.\n    fn put(&mut self, snapshot: \
         {snapshot});\n\n    /// Removes the instance with this identity.\n    fn delete(&mut \
         self, identity: &{identity});{list}\n}}",
        declared.identity.name, storages[entity]
    );
}

/// The context port: only the methods some generated behaviour asks.
fn context_trait(out: &mut String, uses: &Uses) {
    if uses.callers.is_empty() && uses.generates.is_empty() && !uses.external && !uses.clock {
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
             whatever decides it.\n    fn external(&mut self, command: ExternalCommand<'_>, outcome: \
             &'static str) -> bool;\n",
        );
    }
    if uses.clock {
        separate(out);
        out.push_str(
            "    /// The instant of the command decision being made now, or `None` where this \
             context has no\n    /// clock. Read once per decision, before any guard: every \
             guard ordering an instant against\n    /// the current time reads that one instant, \
             and a decision that needs it and has none is\n    /// refused naming the command \
             clock, never decided with another instant.\n    fn command_clock(&mut self) -> \
             Option<crate::primitives::Timestamp>;\n",
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

/// Forwards one owed query to the ports, with the parameters the view declares.
fn forward_query(out: &mut String, layout: &Layout, view: &ess_compiler::ir::ResolvedView) {
    let params = super::port::view_params(layout, "crate", view);
    let view = &view.name;
    let type_name = layout.type_name(view);
    let module = layout.module(layout.owner(view));
    let method = name::value_ident(&type_name);
    let _ = writeln!(
        out,
        "\nimpl<P: crate::{module}::obligations::{type_name}Query> \
         crate::{module}::obligations::{type_name}Query for Generated<P> {{\n    fn \
         {method}(&self{}) -> Result<Vec<crate::{module}::{type_name}>, UnmetObligation> {{\n        \
         crate::{module}::obligations::{type_name}Query::{method}(&self.ports{})\n    }}\n}}",
        super::port::signature(&params, ""),
        params.iter().fold(String::new(), |mut passed, (ident, _)| {
            let _ = write!(passed, ", {ident}");
            passed
        }),
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
    if uses.helpers.contains("decided_at") {
        out.push_str(DECIDED_AT);
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
    let parts = [
        "compare_numbers",
        "number_key",
        "number_order",
        "sum_values",
    ]
    .iter()
    .any(|helper| uses.helpers.contains(helper));
    if parts {
        out.push_str(NUMBER_PARTS);
    }
    if uses.helpers.contains("compare_numbers") {
        out.push_str(COMPARE_NUMBERS);
    }
    if uses.helpers.contains("compare_instants") {
        out.push_str(COMPARE_INSTANTS);
    }
    if uses.helpers.contains("compare_offset_integers") {
        out.push_str(COMPARE_OFFSET_INTEGERS);
    }
    if uses.helpers.contains("compare_offset_instants") {
        out.push_str(COMPARE_OFFSET_INSTANTS);
    }
    if uses.helpers.contains("compare_with_now") {
        out.push_str(COMPARE_WITH_NOW);
    }
    if uses.helpers.contains("compare_instants")
        || uses.helpers.contains("compare_offset_instants")
        || uses.helpers.contains("compare_with_now")
    {
        out.push_str(INSTANT_OF);
    }
    for (helper, text) in QUERY_HELPERS {
        if uses.helpers.contains(helper) {
            out.push_str(text);
        }
    }
}

// Exact comparison of two decimal renderings — an `Integer` is rendered the same way — without a
// float, which would round the values a `Decimal` exists not to round. The two halves are one text
// where a guard compares numbers; a query that only keys, orders or sums them needs the first.

/// A decimal rendering's sign and digits.
const NUMBER_PARTS: &str = "
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
";

/// A guard's exact numeric comparison.
const COMPARE_NUMBERS: &str = "
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

/// A guard's comparison of two `Timestamp` facts by the instants they name
/// (`docs/design/expression-family-source22.md`, decision 2).
const COMPARE_INSTANTS: &str = "
/// Compares two RFC 3339 renderings by the instant each names; an unread one, or one that names
/// no instant, is Unknown — never ordered by its spelling.
fn compare_instants(
    left: Option<String>,
    right: Option<String>,
    accepts: fn(core::cmp::Ordering) -> bool,
) -> Option<bool> {
    Some(accepts(instant_of(&left?)?.cmp(&instant_of(&right?)?)))
}
";

/// The guard comparisons with one constant offset of a fact
/// (`docs/design/expression-family-source22.md`, A2), each written only where a guard uses it.
const COMPARE_OFFSET_INTEGERS: &str = "
/// Compares an `Integer` rendering with another moved by a constant, exactly: every `i64 ± i64`
/// fits `i128`, so nothing wraps, saturates or rounds; an unread one is Unknown.
fn compare_offset_integers(
    left: Option<String>,
    base: Option<String>,
    offset: i128,
    accepts: fn(core::cmp::Ordering) -> bool,
) -> Option<bool> {
    let left: i128 = left?.parse().ok()?;
    let base: i128 = base?.parse().ok()?;
    Some(accepts(left.cmp(&(base + offset))))
}
";

/// See [`COMPARE_OFFSET_INTEGERS`].
const COMPARE_OFFSET_INSTANTS: &str = "
/// Compares an RFC 3339 rendering with another moved by elapsed seconds, by the instants they name;
/// an unread one, one that names no instant, or a moved instant no `date-time` spells is Unknown.
fn compare_offset_instants(
    left: Option<String>,
    base: Option<String>,
    seconds: i64,
    accepts: fn(core::cmp::Ordering) -> bool,
) -> Option<bool> {
    let left = instant_of(&left?)?;
    let (base, nanos) = instant_of(&base?)?;
    let moved = base.checked_add(seconds)?;
    if !(-62_167_219_200..=253_402_300_799).contains(&moved) {
        return None;
    }
    Some(accepts(left.cmp(&(moved, nanos))))
}
";

/// The call a guard ordering an instant against the current time is rendered as: the stored or
/// input instant on the left, the decision's one instant moved by whole seconds on the right
/// (`docs/design/expression-family-source22.md`, A3).
const COMPARE_WITH_NOW_CALL: &str = "compare_with_now(";

/// A guard's ordering of an instant against the decision's one instant (A3).
const COMPARE_WITH_NOW: &str = "
/// Orders an RFC 3339 rendering against the decision's one instant moved by `seconds`, by the
/// instants each names; an unread value, no instant to read, or one that names no instant is
/// Unknown — never ordered by its spelling, and never read from another clock.
fn compare_with_now(
    value: Option<String>,
    now: &Result<Option<crate::primitives::Timestamp>, UnmetObligation>,
    seconds: i64,
    accepts: fn(core::cmp::Ordering) -> bool,
) -> Option<bool> {
    let (at, nanos) = instant_of(&value?)?;
    let (decided, decided_nanos) = instant_of(&now.as_ref().ok()?.as_ref()?.0)?;
    Some(accepts((at, nanos).cmp(&(decided.checked_add(seconds)?, decided_nanos))))
}
";

/// The decision of a guard reading the decision's one instant (A3).
const DECIDED_AT: &str = "
/// A guard's truth, where it has one. Unknown with no instant to read is the command clock this
/// decision needs and its context did not supply: the context's own unavailable answer where it
/// named one, otherwise the missing command clock, named as such. Any other Unknown selects no
/// branch, so the model declares no outcome.
fn decided_at(
    truth: Option<bool>,
    now: &Result<Option<crate::primitives::Timestamp>, UnmetObligation>,
    command: &'static str,
) -> Result<bool, UnmetObligation> {
    match (truth, now) {
        (Some(truth), _) => Ok(truth),
        (None, Err(unavailable)) => Err(unavailable.clone()),
        (None, Ok(None)) => Err(UnmetObligation { capability: \"command clock\", source: command }),
        (None, Ok(Some(_))) => Err(undeclared(command)),
    }
}
";

/// The instant an RFC 3339 `date-time` names, shared by every instant comparison.
const INSTANT_OF: &str = "
/// The instant an RFC 3339 `date-time` names, as seconds from the epoch and nanoseconds.
fn instant_of(text: &str) -> Option<(i64, u32)> {
    let bytes = text.as_bytes();
    let digits = |from: usize, to: usize| -> Option<u32> {
        let slice = bytes.get(from..to)?;
        if slice.is_empty() || !slice.iter().all(u8::is_ascii_digit) {
            return None;
        }
        slice
            .iter()
            .try_fold(0u32, |total, digit| Some(total * 10 + u32::from(digit - b'0')))
    };
    let at = |index: usize, expected: &[u8]| bytes.get(index).is_some_and(|b| expected.contains(b));
    if !(at(4, b\"-\") && at(7, b\"-\") && at(10, b\"Tt\") && at(13, b\":\") && at(16, b\":\")) {
        return None;
    }
    let (year, month, day) = (i64::from(digits(0, 4)?), digits(5, 7)?, digits(8, 10)?);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let length = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    let (hour, minute, second) = (digits(11, 13)?, digits(14, 16)?, digits(17, 19)?);
    if day < 1 || day > length || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let mut position = 19;
    let mut nanos = 0u32;
    if at(position, b\".\") {
        let mut end = position + 1;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        let width = end - position - 1;
        if width == 0 || width > 9 {
            return None;
        }
        nanos = digits(position + 1, end)? * 10u32.pow(u32::try_from(9 - width).ok()?);
        position = end;
    }
    let offset = match bytes.get(position..)? {
        b\"Z\" | b\"z\" => 0i64,
        [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
            let (hours, minutes) = (digits(position + 1, position + 3)?, digits(position + 4, position + 6)?);
            if hours > 23 || minutes > 59 {
                return None;
            }
            let magnitude = i64::from(hours * 3600 + minutes * 60);
            if *sign == b'-' { -magnitude } else { magnitude }
        }
        _ => return None,
    };
    let shifted = if month <= 2 { year - 1 } else { year };
    let era = shifted.div_euclid(400);
    let of_era = shifted - era * 400;
    let march = i64::from(if month > 2 { month - 3 } else { month + 9 });
    let of_year = (153 * march + 2) / 5 + i64::from(day) - 1;
    let of_cycle = of_era * 365 + of_era / 4 - of_era / 100 + of_year;
    let days = era * 146_097 + of_cycle - 719_468;
    Some((days * 86_400 + i64::from(hour * 3600 + minute * 60 + second) - offset, nanos))
}
";

/// What a generated query computes with, by name, in the order they are written; only the ones some
/// query uses are. Each follows `ess_conformance::aggregate` (`docs/design/aggregate-views.md`).
const QUERY_HELPERS: [(&str, &str); 13] = [
    (
        "unrepresentable",
        "
/// The typed refusal of a row a declared type cannot hold — a sum past the `Integer` range, about
/// which the specification makes no claim.
fn unrepresentable(source: &'static str) -> UnmetObligation {
    let capability = \"view query\";
    UnmetObligation { capability, source }
}
",
    ),
    (
        "count",
        "
/// A number of rows, as the `Integer` it is reported as.
fn count(rows: usize) -> i64 {
    i64::try_from(rows).unwrap_or(i64::MAX)
}
",
    ),
    (
        "distinct",
        "
/// How many different values `values` holds, each already in the form its equality compares.
fn distinct(values: impl Iterator<Item = String>) -> i64 {
    let mut seen: Vec<String> = Vec::new();
    for value in values {
        if !seen.contains(&value) {
            seen.push(value);
        }
    }
    count(seen.len())
}
",
    ),
    (
        "number_key",
        "
/// A decimal rendering in the one spelling every rendering of its value shares, so `1.0` and `1`
/// are one value; text that is no number stays as it is.
fn number_key(text: &str) -> String {
    match number_parts(text) {
        Some((negative, whole, fraction)) => {
            format!(\"{}{whole}.{fraction}\", if negative { \"-\" } else { \"\" })
        }
        None => text.to_owned(),
    }
}
",
    ),
    (
        "number_order",
        "
/// Two decimal renderings by their exact value; text that is no number by its bytes.
fn number_order(left: &str, right: &str) -> core::cmp::Ordering {
    let (Some(left), Some(right)) = (number_parts(left), number_parts(right)) else {
        return left.as_bytes().cmp(right.as_bytes());
    };
    let magnitude = left
        .1
        .len()
        .cmp(&right.1.len())
        .then_with(|| left.1.cmp(&right.1))
        .then_with(|| left.2.cmp(&right.2));
    match (left.0, right.0) {
        (false, false) => magnitude,
        (true, true) => magnitude.reverse(),
        (true, false) => core::cmp::Ordering::Less,
        (false, true) => core::cmp::Ordering::Greater,
    }
}
",
    ),
    (
        "text_order",
        "
/// Two texts by their UTF-8 bytes.
fn text_order(left: &str, right: &str) -> core::cmp::Ordering {
    left.as_bytes().cmp(right.as_bytes())
}
",
    ),
    (
        "instant",
        "
/// The instant an RFC 3339 `date-time` names, as seconds from the epoch and nanoseconds.
fn instant(text: &str) -> Option<(i64, u32)> {
    let bytes = text.as_bytes();
    let digits = |from: usize, to: usize| -> Option<u32> {
        let slice = bytes.get(from..to)?;
        if slice.is_empty() || !slice.iter().all(u8::is_ascii_digit) {
            return None;
        }
        slice
            .iter()
            .try_fold(0u32, |total, digit| Some(total * 10 + u32::from(digit - b'0')))
    };
    let at = |index: usize, expected: &[u8]| bytes.get(index).is_some_and(|b| expected.contains(b));
    if !(at(4, b\"-\") && at(7, b\"-\") && at(10, b\"Tt\") && at(13, b\":\") && at(16, b\":\")) {
        return None;
    }
    let (year, month, day) = (i64::from(digits(0, 4)?), digits(5, 7)?, digits(8, 10)?);
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let length = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => return None,
    };
    if day < 1 || day > length {
        return None;
    }
    let (hour, minute, second) = (digits(11, 13)?, digits(14, 16)?, digits(17, 19)?);
    if hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    let mut position = 19;
    let mut nanos = 0u32;
    if at(position, b\".\") {
        let start = position + 1;
        let mut end = start;
        while bytes.get(end).is_some_and(u8::is_ascii_digit) {
            end += 1;
        }
        let width = end - start;
        if width == 0 || width > 9 {
            return None;
        }
        nanos = digits(start, end)? * 10u32.pow(u32::try_from(9 - width).ok()?);
        position = end;
    }
    let offset = match bytes.get(position..)? {
        b\"Z\" | b\"z\" => 0i64,
        [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
            let (hours, minutes) = (digits(position + 1, position + 3)?, digits(position + 4, position + 6)?);
            if hours > 23 || minutes > 59 {
                return None;
            }
            let magnitude = i64::from(hours * 3600 + minutes * 60);
            if *sign == b'-' {
                -magnitude
            } else {
                magnitude
            }
        }
        _ => return None,
    };
    let shifted = year - i64::from(month <= 2);
    let era = if shifted >= 0 { shifted } else { shifted - 399 } / 400;
    let year_of_era = shifted - era * 400;
    let month = i64::from(month);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Some((
        days * 86_400 + i64::from(hour * 3600 + minute * 60 + second) - offset,
        nanos,
    ))
}
",
    ),
    (
        "instant_key",
        "
/// An RFC 3339 rendering as the instant it names, so one instant spelled with two offsets is one
/// value; text that names no instant stays as it is.
fn instant_key(text: &str) -> String {
    match instant(text) {
        Some((seconds, nanos)) => format!(\"{seconds}.{nanos:09}\"),
        None => text.to_owned(),
    }
}
",
    ),
    (
        "instant_order",
        "
/// Two RFC 3339 renderings by the instant each names; text that names none by its bytes.
fn instant_order(left: &str, right: &str) -> core::cmp::Ordering {
    match (instant(left), instant(right)) {
        (Some(left), Some(right)) => left.cmp(&right),
        _ => left.as_bytes().cmp(right.as_bytes()),
    }
}
",
    ),
    (
        "sum_values",
        "
/// The exact sum of decimal renderings: how many there were, and the sum as `units × 10^-scale`;
/// `None` where one is not a plain decimal or the sum leaves `i128`.
fn sum_values(values: impl Iterator<Item = String>) -> Option<(usize, i128, u32)> {
    let (mut present, mut units, mut scale) = (0usize, 0i128, 0u32);
    for value in values {
        let (negative, whole, fraction) = number_parts(&value)?;
        let digits: i128 = if whole.is_empty() && fraction.is_empty() {
            0
        } else {
            format!(\"{whole}{fraction}\").parse().ok()?
        };
        let digits = if negative { -digits } else { digits };
        let places = u32::try_from(fraction.len()).ok()?;
        let common = scale.max(places);
        let widen = |units: i128, from: u32| units.checked_mul(10i128.checked_pow(common - from)?);
        units = widen(units, scale)?.checked_add(widen(digits, places)?)?;
        scale = common;
        present += 1;
    }
    Some((present, units, scale))
}
",
    ),
    (
        "spell",
        "
/// `units × 10^-scale`, spelled without trailing zeros.
fn spell(units: i128, scale: u32) -> String {
    let digits = units.unsigned_abs().to_string();
    let width = usize::try_from(scale).unwrap_or(0);
    let padded = format!(\"{digits:0>width$}\", width = width + 1);
    let (whole, fraction) = padded.split_at(padded.len() - width);
    let fraction = fraction.trim_end_matches('0');
    let mut text = String::new();
    if units < 0 {
        text.push('-');
    }
    text.push_str(whole);
    if !fraction.is_empty() {
        text.push('.');
        text.push_str(fraction);
    }
    text
}
",
    ),
    (
        "average",
        "
/// The mean of `present` values summing to `units × 10^-scale`, rounded to six fractional digits,
/// ties to even; `Some(None)` over no value, `None` where it leaves `i128`.
fn average(present: usize, units: i128, scale: u32) -> Option<Option<String>> {
    if present == 0 {
        return Some(None);
    }
    let count = i128::try_from(present).ok()?;
    let (numerator, denominator) = if scale <= 6 {
        (units.checked_mul(10i128.checked_pow(6 - scale)?)?, count)
    } else {
        (units, count.checked_mul(10i128.checked_pow(scale - 6)?)?)
    };
    let magnitude = numerator.checked_abs()?;
    let (mut quotient, remainder) = (magnitude / denominator, magnitude % denominator);
    let twice = remainder.checked_mul(2)?;
    if twice > denominator || (twice == denominator && quotient % 2 == 1) {
        quotient += 1;
    }
    Some(Some(spell(if numerator < 0 { -quotient } else { quotient }, 6)))
}
",
    ),
    (
        "extreme",
        "
/// The value whose reading `order` ranks `wins` against every other (`Less` for a minimum,
/// `Greater` for a maximum), the first of equal ones; `None` over no value.
fn extreme<T>(
    values: impl Iterator<Item = (String, T)>,
    wins: core::cmp::Ordering,
    order: fn(&str, &str) -> core::cmp::Ordering,
) -> Option<T> {
    let mut best: Option<(String, T)> = None;
    for (read, value) in values {
        let replaces = match &best {
            Some((held, _)) => order(&read, held) == wins,
            None => true,
        };
        if replaces {
            best = Some((read, value));
        }
    }
    best.map(|(_, value)| value)
}
",
    ),
];

// ---- one command -------------------------------------------------------------------------------

/// Everything rendering one command's impl needs.
struct Writer<'a> {
    ir: &'a EssIr,
    layout: &'a Layout,
    command: &'a ResolvedCommand,
    storages: &'a BTreeMap<QualifiedName, String>,
    external_commands: &'a BTreeMap<QualifiedName, String>,
    uses: &'a mut Uses,
    bounds: Bounds,
}

/// A fallible companion preserves the existing context API and its implementations.
fn fallible_context(out: &mut String, uses: &Uses) {
    if uses.callers.is_empty() && uses.generates.is_empty() && !uses.external && !uses.clock {
        return;
    }
    out.push_str("\n/// Context answers that may be unavailable, without fabricated values.\n/// Existing `Context` implementations receive the blanket adapter.\npub trait TryContext {\n");
    let mut adapter = String::from("\nimpl<T: Context + ?Sized> TryContext for T {\n");
    for (method, (ty, attribute)) in &uses.callers {
        let _ = writeln!(out, "/// The caller's `{attribute}`, or a named unavailable answer.\nfn try_{method}(&self) -> Result<Option<{ty}>, UnmetObligation>;");
        let _ = writeln!(adapter, "fn try_{method}(&self) -> Result<Option<{ty}>, UnmetObligation> {{ Ok(Context::{method}(self)) }}");
    }
    for (method, (ty, _)) in &uses.generates {
        let _ = writeln!(out, "/// Assigns the value, or names the unavailable answer.\nfn try_{method}(&mut self) -> Result<{ty}, UnmetObligation>;");
        let _ = writeln!(adapter, "fn try_{method}(&mut self) -> Result<{ty}, UnmetObligation> {{ Ok(Context::{method}(self)) }}");
    }
    if uses.external {
        out.push_str("/// Decides the named external branch, or names the unavailable answer.\nfn try_external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> Result<bool, UnmetObligation>;\n");
        adapter.push_str("fn try_external(&mut self, command: ExternalCommand<'_>, outcome: &'static str) -> Result<bool, UnmetObligation> { Ok(Context::external(self, command, outcome)) }\n");
    }
    if uses.clock {
        out.push_str("/// The decision's one instant, `None` where the context has no clock, or a named unavailable answer.\nfn try_command_clock(&mut self) -> Result<Option<crate::primitives::Timestamp>, UnmetObligation>;\n");
        adapter.push_str("fn try_command_clock(&mut self) -> Result<Option<crate::primitives::Timestamp>, UnmetObligation> { Ok(Context::command_clock(self)) }\n");
    }
    out.push_str("}\n");
    adapter.push_str("}\n");
    out.push_str(&adapter);
    out.push_str("\n/// An unavailable runtime context answer, rather than a new planned capability.\npub fn unmet_context(source: &'static str) -> UnmetObligation { UnmetObligation { capability: \"context answer\", source } }\n");
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
            bounds.push("TryContext".to_owned());
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
        if determined::reads_clock(command) {
            // The decision's one instant, read once at its edge before any guard or row is read
            // (ess/22, family F A3); every guard ordering an instant against `now` reads it.
            self.uses.clock = true;
            self.bounds.context = true;
            out.push_str(
                "        // The decision's one instant: read once, before any guard, and read by \
                 every guard that\n        // orders an instant against the current time.\n        \
                 let now = self.ports.try_command_clock();\n",
            );
        }
        let related = determined::related(command);
        let orders =
            related.is_some() && determined::orders_present_related_refusal(self.ir, command);
        // A related row named by the input is read first: `existing_instance:`, then a missing
        // row's `exists: false`, answer before the input-guarded refusals (ess/18).
        if let Some((ResolvedRelatedVia::Input { field, type_ref }, entity)) = related {
            if let Some(existing) = determined::existing_instance(command) {
                out.push_str(&self.existing_lookup(existing));
            }
            let field = name::value_ident(field);
            let reference = if type_ref.is_optional() {
                format!("input.{field}.as_ref()")
            } else {
                format!("Some(&input.{field})")
            };
            out.push_str(&self.related_read(entity, &reference, &format!("input.{field}")));
        }
        for outcome in &command.outcomes {
            if let (ResolvedCondition::When { predicate }, Some(_), None) =
                (&outcome.condition, &outcome.error, &outcome.subject)
            {
                let truth = self.guards().predicate(&Env::Input(command), predicate);
                let guard = self.decided(&truth);
                let answer = self.variant(outcome, Held::None, None);
                let _ = writeln!(
                    out,
                    "        // `{}`: an input-guarded refusal, before the addressed subject is loaded.\n        \
                     if {guard} {{\n            return Ok({answer});\n        }}",
                    outcome.name
                );
            }
        }
        if !matches!(related, Some((ResolvedRelatedVia::Input { .. }, _))) {
            if let Some(existing) = determined::existing_instance(command) {
                out.push_str(&self.existing_lookup(existing));
            }
        }
        if let Some((ResolvedRelatedVia::Subject { field, type_ref }, entity)) = related {
            // A stored reference is read from the addressed row, as it was before the branch, once
            // that row's existence and held state have answered (ess/22, beyond10x/ess#304).
            let subject = determined::addressed_subject(command)
                .expect("the plan admits a stored reference on an addressed subject");
            let addressed = self.ir.entity(&subject.entity);
            let storage = self.storage(&addressed.name);
            let unknown = self.unknown_arm(12);
            let _ = writeln!(
                out,
                "        // The addressed row, whose stored `{field}` names the related row.\n        \
                 let Some(held) = {storage}::get(&self.ports, &input.{}) else {{\n{unknown}        \
                 }};\n        let _ = &held;",
                name::value_ident(&subject.instance.field().name)
            );
            out.push_str(&self.precheck(false));
            let member = name::value_ident(field);
            let reference = if type_ref.is_optional() {
                format!("held.data.{member}.as_ref()")
            } else {
                format!("Some(&held.data.{member})")
            };
            out.push_str(&self.related_read(entity, &reference, &format!("subject.{field}")));
        } else if orders {
            out.push_str(&self.precheck(true));
        }
        if orders {
            for outcome in command
                .outcomes
                .iter()
                .filter(|outcome| determined::is_present_related_refusal(outcome))
            {
                let guard = self.related_guard(outcome);
                let taken = self.take(outcome, Held::None);
                let _ = writeln!(
                    out,
                    "        // `{}`: a present related row's refusal, before every accepting \
                     branch.\n        if let Some(related) = &related {{\n        if {guard} \
                     {{\n{taken}        }}\n        }}",
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
            let unknown = self.unknown_arm(12);
            let _ = writeln!(
                out,
                "        // The addressed row, read before the branches that select by it.\n        \
                 let Some(held) = {storage}::get(&self.ports, &input.{}) else {{\n{unknown}        \
                 }};\n        let _ = &held;",
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
                    let truth = self.guards().predicate(&Env::Input(command), predicate);
                    let guard = self.decided(&truth);
                    let taken = self.take(outcome, held);
                    let _ = writeln!(
                        out,
                        "        // `{}`: an accepting branch, in declaration order.\n        if \
                         {guard} {{\n{taken}        }}",
                        outcome.name
                    );
                }
                // A present related row selects its predicate branches in declaration order; an
                // absent reference selects none. Already answered where they come first.
                ResolvedCondition::Related {
                    test: ResolvedRelatedTest::Holds { .. },
                    ..
                } if !(orders && determined::is_present_related_refusal(outcome)) => {
                    let guard = self.related_guard(outcome);
                    let taken = self.take(outcome, held);
                    let _ = writeln!(
                        out,
                        "        // `{}`: selected by the present related row, in declaration \
                         order.\n        if let Some(related) = &related {{\n        if {guard} \
                         {{\n{taken}        }}\n        }}",
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
                    let truth = self.guards().predicate(&Env::Input(command), predicate);
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

    /// The `existing_instance:` refusal, answered for an identity a record already carries: the
    /// storage lookup of the identity the command's creation would take, after the input-guarded
    /// refusals and before any branch is taken (beyond10x/ess#310). An optional identity input the
    /// request leaves out names a new identity, which no record carries.
    fn existing_lookup(&mut self, existing: &ResolvedOutcome) -> String {
        let command = self.command;
        let (creation, (field, optional)) = command
            .outcomes
            .iter()
            .find_map(|outcome| {
                determined::identity_input(outcome).map(|identity| (outcome, identity))
            })
            .expect("the plan admits `existing_instance:` beside a creation read from the input");
        let entity = &creation
            .subject
            .as_ref()
            .expect("a creation names its subject")
            .entity;
        let ir = self.ir;
        let storage = self.storage(&ir.entity(entity).name);
        let answer = self.variant(existing, Held::None, None);
        let field = name::value_ident(field);
        let found = if optional {
            format!(
                "input.{field}.as_ref().is_some_and(|identity| {storage}::get(&self.ports, \
                 identity).is_some())"
            )
        } else {
            format!("{storage}::get(&self.ports, &input.{field}).is_some()")
        };
        format!(
            "        // `{}`: an identity a record already carries, before any branch is taken.\n        \
             if {found} {{\n            return Ok({answer});\n        }}\n",
            existing.name
        )
    }

    /// The related row a `when_related:` guard reads (ess/18, ess/22; beyond10x/ess#319), as
    /// `related`: read by identity through the related entity's storage port where `reference`
    /// names one, and no row read where it is absent. A reference naming no row takes the
    /// `exists: false` branch, which the compiler requires wherever a predicate branch reads it.
    fn related_read(&mut self, entity: &EntityHandle, reference: &str, named: &str) -> String {
        let related = self.ir.entity(entity);
        let storage = self.storage(&related.name);
        let mut out = format!(
            "        // `when_related:` reads the `{}` row `{named}` names, through its storage \
             port; an absent\n        // reference reads no row and selects no related \
             branch.\n        let reference = {reference};\n        let related = \
             reference.and_then(|identity| {storage}::get(&self.ports, identity));\n",
            related.name
        );
        if let Some(absent) = determined::related_absent(self.command) {
            let taken = self.take(absent, Held::None);
            let _ = write!(
                out,
                "        // `{}`: the reference names an identity no row carries.\n        if \
                 reference.is_some() && related.is_none() {{\n{taken}        }}\n",
                absent.name
            );
        }
        out.push_str("        let _ = &related;\n");
        out
    }

    /// The guard of a `when_related:` predicate branch over the present row `related`, with its
    /// input guard, as `decided(…)?`.
    fn related_guard(&mut self, outcome: &ResolvedOutcome) -> String {
        let command = self.command;
        let ResolvedCondition::Related {
            entity,
            test: ResolvedRelatedTest::Holds { predicate },
            input,
            ..
        } = &outcome.condition
        else {
            unreachable!("only a predicate branch reads the present related row")
        };
        let entity = self.ir.entity(entity);
        let row = self
            .guards_on("related")
            .predicate(&Env::Subject(command, entity), predicate);
        let truth = match input {
            Some(input) => {
                self.uses.helpers.insert("all");
                format!(
                    "all(&[{row}, {}])",
                    self.guards().predicate(&Env::Input(command), input)
                )
            }
            None => row,
        };
        self.decided(&truth)
    }

    /// The addressed-row existence and held-state answers of the branch the request selects with
    /// no present-related refusal read, before the related refusals (beyond10x/ess#282, #304), as
    /// the interpreter orders them. `with_related` reads the present row's accepting branches, as
    /// an input reference is read by then; a stored reference is not read yet.
    fn precheck(&mut self, with_related: bool) -> String {
        let command = self.command;
        let mut arms = String::new();
        let mut breaks = false;
        for outcome in &command.outcomes {
            match &outcome.condition {
                ResolvedCondition::When { predicate } if outcome.error.is_none() => {
                    let truth = self.guards().predicate(&Env::Input(command), predicate);
                    let guard = self.decided(&truth);
                    let check = self.check_subject(outcome);
                    breaks = true;
                    let _ = writeln!(
                        arms,
                        "            if {guard} {{\n{check}                break 'precheck;\n            \
                         }}"
                    );
                }
                ResolvedCondition::Related {
                    test: ResolvedRelatedTest::Holds { .. },
                    ..
                } if with_related && outcome.error.is_none() => {
                    let guard = self.related_guard(outcome);
                    let check = self.check_subject(outcome);
                    breaks = true;
                    let _ = writeln!(
                        arms,
                        "            if let Some(related) = &related {{\n            if {guard} \
                         {{\n{check}                break 'precheck;\n            }}\n            }}"
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
            arms.push_str(&self.check_subject(default));
        }
        let label = if breaks { "'precheck: " } else { "" };
        format!(
            "        // The addressed row's existence and held state, for the branch the request \
             selects\n        // before any present related row's refusal.\n        \
             {label}{{\n{arms}        }}\n"
        )
    }

    /// The existence and held-state answers of one selected branch's subject, returning where
    /// either refuses; nothing for a branch addressing no existing row.
    fn check_subject(&mut self, outcome: &ResolvedOutcome) -> String {
        let Some(subject) = &outcome.subject else {
            return String::new();
        };
        let ResolvedInstance::Supplied { field } = &subject.instance else {
            return String::new();
        };
        if subject.effect == ResolvedEffect::Creates {
            return String::new();
        }
        let entity = self.ir.entity(&subject.entity);
        let storage = self.storage(&entity.name);
        let unknown = self.unknown_arm(20);
        let mut out = format!(
            "                {{\n                let Some(held) = {storage}::get(&self.ports, \
             &input.{}) else {{\n{unknown}                }};\n                let _ = &held;\n",
            name::value_ident(&field.name)
        );
        if let ResolvedEffect::Moves { transition } = &subject.effect {
            if transition.from.len() < entity.lifecycle.states.len() {
                let wrong = self.wrong_state_answer();
                let reads = self.wrong_state_reads(entity);
                if reads.0 {
                    out.push_str("                let held_state = held.state;\n");
                }
                if reads.1 {
                    out.push_str("                let before = held.data.clone();\n");
                }
                let state_enum = declared_path(self.layout, entity.state_type.name());
                let patterns: Vec<String> = transition
                    .from
                    .iter()
                    .map(|state| format!("{state_enum}::{}", name::pascal(state.as_str())))
                    .collect();
                let _ = writeln!(
                    out,
                    "                if !matches!(held.state, {}) {{\n                    return \
                     {wrong};\n                }}",
                    patterns.join(" | ")
                );
            }
        }
        out.push_str("                }\n");
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
        self.uses
            .externals
            .insert(format!("{}/{}", self.command.name, outcome.name));
        self.bounds.context = true;
        format!(
            "self.ports.try_external(ExternalCommand::{}(&input), \"{}\")?",
            self.external_commands[&self.command.name], outcome.name
        )
    }

    /// `decided(<truth>, "<command>")?`.
    fn decided(&mut self, truth: &str) -> String {
        self.uses.helpers.insert("undeclared");
        // A guard reading the decision's instant that is Unknown with no instant to read is the
        // command clock missing, named as such (ess/22, family F A3).
        if truth.contains(COMPARE_WITH_NOW_CALL) {
            self.uses.helpers.insert("decided_at");
            return format!("decided_at({truth}, &now, \"{}\")?", self.command.name);
        }
        self.uses.helpers.insert("decided");
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
                    kind: ess_primitives::predicate::CompareKind::Value,
                    left: Operand::Fact(path),
                    op: CompareOp::Eq,
                    right: Operand::Literal(FactValue::Text(equals.clone())),
                };
                (
                    self.guards()
                        .predicate(&Env::Subject(command, entity), &compare),
                    predicate.as_ref(),
                )
            }
            ResolvedCondition::SubjectPredicate { predicate, input } => (
                self.guards()
                    .predicate(&Env::Subject(command, entity), predicate),
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
                    self.guards().predicate(&Env::Input(command), input)
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

    /// The statements answering an identity no record carries, each line indented by `depth`
    /// spaces and ending in a `return`: the creation of create-or-update where the command
    /// declares one (beyond10x/ess#310), else the answer [`Self::unknown_answer`] names.
    fn unknown_arm(&mut self, depth: usize) -> String {
        let command = self.command;
        if let Some(creation) = command
            .outcomes
            .iter()
            .find(|outcome| determined::creates_unknown(outcome))
        {
            let pad = " ".repeat(depth.saturating_sub(12));
            return self.create(creation, Held::None).lines().fold(
                String::new(),
                |mut out, line| {
                    let _ = writeln!(out, "{pad}{line}");
                    out
                },
            );
        }
        let unknown = self.unknown_answer();
        format!("{}return {unknown};\n", " ".repeat(depth))
    }

    /// The statements of a creation, at the indentation of a branch block, ending in its `return`.
    ///
    /// The new identity follows its declared event payload source, or the context where the
    /// specification leaves it undetermined.
    fn create(&mut self, outcome: &ResolvedOutcome, held: Held) -> String {
        let mut out = String::new();
        let subject = outcome
            .subject
            .as_ref()
            .expect("a creation names its subject");
        let entity = self.ir.entity(&subject.entity);
        let storage = self.storage(&entity.name);
        let module_entity = declared_path(self.layout, &entity.name);
        let any = format!(
            "crate::{}::Any{}",
            self.layout.module(self.layout.owner(&entity.name)),
            self.layout.type_name(&entity.name)
        );
        let identity_type = &entity.identity.type_ref;
        let assigned = match determined::identity_source(outcome) {
            Some(source) => self.value(source, None),
            None => self.generate(identity_type),
        };
        let _ = writeln!(
            out,
            "            let identity: {} = {assigned};",
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
        let answer = self.variant(outcome, held, Some("identity"));
        let _ = writeln!(out, "            let answer = {answer};");
        let _ = writeln!(
            out,
            "            {storage}::put(&mut self.ports, \
             {any}::{state}({module_entity}::{constructor}(data)).snapshot());"
        );
        out.push_str("            return Ok(answer);\n");
        out
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
        let any = format!(
            "crate::{}::Any{}",
            self.layout.module(self.layout.owner(&entity.name)),
            self.layout.type_name(&entity.name)
        );
        match (&subject.effect, &subject.instance) {
            (ResolvedEffect::Creates, ResolvedInstance::Observed { .. }) => {
                out.push_str(&self.create(outcome, held));
            }
            (effect, ResolvedInstance::Supplied { field }) => {
                let identity = name::value_ident(&field.name);
                let unknown = self.unknown_arm(16);
                let _ = writeln!(
                    out,
                    "            let Some(held) = {storage}::get(&self.ports, &input.{identity}) \
                     else {{\n{unknown}            }};\n            let _ = &held;"
                );
                let reads_before = outcome_reads_before(outcome);
                let mut commit = String::new();
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
                        let _ =
                            writeln!(commit, "            {storage}::put(&mut self.ports, next);");
                    }
                    ResolvedEffect::Updates | ResolvedEffect::Preserves => {
                        if reads_before {
                            out.push_str("            let before = held.data.clone();\n");
                        }
                        let writes =
                            matches!(effect, ResolvedEffect::Updates) || !outcome.sets.is_empty();
                        if writes {
                            self.write_sets(&mut out, outcome, "held", entity);
                            let _ = writeln!(
                                commit,
                                "            {storage}::put(&mut self.ports, next);"
                            );
                        }
                    }
                    ResolvedEffect::Deletes => {
                        if reads_before {
                            out.push_str("            let before = held.data.clone();\n");
                        }
                        let _ = writeln!(
                            commit,
                            "            {storage}::delete(&mut self.ports, &input.{identity});"
                        );
                    }
                    ResolvedEffect::Creates => {
                        unreachable!("a creation whose identity is supplied is an obligation")
                    }
                }
                let answer = self.variant(outcome, held, None);
                let _ = writeln!(
                    out,
                    "            let answer = {answer};\n{commit}            return Ok(answer);"
                );
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
        self.uses.assigned.insert(method.clone(), target.clone());
        self.uses.generates.insert(
            method.clone(),
            (self.layout.absolute_type(target), target.to_string()),
        );
        self.bounds.context = true;
        format!("self.ports.try_{method}()?")
    }

    /// The value one source fills its field with. `before` names the held data before the
    /// outcome, where the branch holds a row.
    // One arm per value source the model declares, each rendering its own expression.
    fn value(&mut self, field: &ResolvedPayloadField, before: Option<&str>) -> String {
        let previous = before.map(|row| format!("{row}.{}", name::value_ident(&field.target)));
        self.value_at(field, before, previous)
    }

    #[allow(clippy::too_many_lines)]
    fn value_at(
        &mut self,
        field: &ResolvedPayloadField,
        before: Option<&str>,
        previous: Option<String>,
    ) -> String {
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
                let read = self.guards().caller(attribute, type_ref);
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
            ResolvedPayloadValue::Increment { by } => increment(
                self.ir,
                self.layout,
                target,
                previous
                    .as_deref()
                    .expect("the plan admits `{increment:}` only on a held row"),
                by,
            ),
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
                // A structured event payload may have the same name and shape as an Optional
                // stored field without reading that field. Only an increment consumes the value
                // at the corresponding previous path; delaying the unwrap until that is true
                // keeps output-only structures independent of the held row.
                let previous = previous
                    .filter(|_| increments_previous(&field.value))
                    .map(|read| {
                        if target.is_optional() {
                            self.uses.helpers.insert("undeclared");
                            format!(
                                "{read}.as_ref().ok_or_else(|| undeclared(\"{}\"))?",
                                self.command.name
                            )
                        } else {
                            read
                        }
                    });
                let mut rendered = Vec::new();
                for member in members {
                    let value = match fields.iter().find(|source| source.target == member.name) {
                        Some(source) => self.value_at(
                            source,
                            before,
                            previous
                                .as_ref()
                                .map(|read| format!("{read}.{}", name::value_ident(&member.name))),
                        ),
                        None => "None".to_owned(),
                    };
                    rendered.push(format!("{}: {value}", name::value_ident(&member.name)));
                }
                let built = format!(
                    "{} {{ {} }}",
                    declared_path(self.layout, declared.name()),
                    rendered.join(", ")
                );
                let built = self.layout.reference_value(declared.name(), built);
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

    /// The guard renderer over this impl's uses and bounds, reading the held row as `held`.
    fn guards(&mut self) -> Guards<'_> {
        self.guards_on("held")
    }

    /// The guard renderer reading a stored field or `state` from `row`.
    fn guards_on(&mut self, row: &'static str) -> Guards<'_> {
        Guards {
            ir: self.ir,
            layout: self.layout,
            uses: &mut *self.uses,
            bounds: &mut self.bounds,
            row,
        }
    }
}

// ---- guards --------------------------------------------------------------------------------------

/// Renders a guard — a command's, or a view's `filter:` — as an `Option<bool>` expression, recording
/// what it asks of the ports and the helpers.
struct Guards<'a> {
    ir: &'a EssIr,
    layout: &'a Layout,
    uses: &'a mut Uses,
    bounds: &'a mut Bounds,
    /// The variable a stored field or `state` is read from: a snapshot, or a reference to one.
    row: &'a str,
}

/// The generated acceptor of an ordering `op` decides by: `core::cmp::Ordering::<acceptor>`.
fn acceptor(op: CompareOp) -> &'static str {
    match op {
        CompareOp::Eq => "is_eq",
        CompareOp::Ne => "is_ne",
        CompareOp::Lt => "is_lt",
        CompareOp::Le => "is_le",
        CompareOp::Gt => "is_gt",
        CompareOp::Ge => "is_ge",
    }
}

impl Guards<'_> {
    /// `self.ports.caller_<attribute>()`, recorded on the context: an `Option` of the attribute.
    fn caller(&mut self, attribute: &str, type_ref: &ResolvedTypeRef) -> String {
        let method = name::value_ident(&format!("caller_{attribute}"));
        self.uses.callers.insert(
            method.clone(),
            (self.layout.absolute_type(type_ref), attribute.to_owned()),
        );
        self.bounds.context = true;
        format!("self.ports.try_{method}()?")
    }

    /// A predicate as an `Option<bool>` expression: `None` is Unknown.
    #[allow(clippy::too_many_lines)]
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
            Predicate::Compare {
                left,
                op,
                right: Operand::Offset(offset),
                ..
            } => self.offset(env, left, *op, offset),
            Predicate::Compare {
                left, op, right, ..
            } => {
                let kind = [left, right]
                    .into_iter()
                    .find_map(|operand| match operand {
                        Operand::Fact(path) => Some(self.resolve(env, path).kind),
                        Operand::Literal(_) | Operand::Offset(_) => None,
                    })
                    .expect("the plan admits comparisons reading a fact");
                let accepts = acceptor(*op);
                if let (Kind::Instant, Some(current)) =
                    (&kind, determined::current_time(right, *op))
                {
                    return self.with_now(env, left, &kind, accepts, current.offset_seconds());
                }
                let left = self.operand(env, left, &kind);
                let right = self.operand(env, right, &kind);
                if let Kind::Number(_) | Kind::Instant = kind {
                    // Two `Timestamp`s compare by the instants they name (decision 2).
                    let helper = if kind == Kind::Instant {
                        "compare_instants"
                    } else {
                        "compare_numbers"
                    };
                    self.uses.helpers.insert(helper);
                    format!("{helper}({left}, {right}, core::cmp::Ordering::{accepts})")
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

    /// An instant ordered against the current time: it reads the decision's one instant, `now`,
    /// moved by the operand's whole `seconds` (ess/22, family F A3).
    fn with_now(
        &mut self,
        env: &Env<'_>,
        left: &Operand,
        kind: &Kind,
        accepts: &str,
        seconds: i64,
    ) -> String {
        let left = self.operand(env, left, kind);
        self.uses.helpers.insert("compare_with_now");
        format!(
            "{COMPARE_WITH_NOW_CALL}{left}, &now, {seconds}, \
             core::cmp::Ordering::{accepts})"
        )
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
            Operand::Offset(_) => unreachable!("an offset is compared by `offset`"),
        }
    }

    /// `left <op> base ± magnitude` (`docs/design/expression-family-source22.md`, A2): two
    /// `Integer` renderings compared with the exact sum in `i128`, or two `Timestamp` renderings
    /// compared as instants after moving the base by elapsed seconds — Unknown where a value is
    /// absent or the moved instant is past what a `date-time` spells.
    fn offset(
        &mut self,
        env: &Env<'_>,
        left: &Operand,
        op: CompareOp,
        offset: &ess_primitives::predicate::OffsetOperand,
    ) -> String {
        use ess_primitives::predicate::{OffsetDirection, OffsetMagnitude};
        let base = self.resolve(env, &offset.base);
        let left = self.operand(env, left, &base.kind);
        let base = self.read(&base);
        let accepts = acceptor(op);
        let negate = offset.direction == OffsetDirection::Subtract;
        match offset.magnitude {
            OffsetMagnitude::Integer(_) => {
                self.uses.helpers.insert("compare_offset_integers");
                let magnitude = offset
                    .magnitude
                    .integer()
                    .expect("the plan admits whole Integer magnitudes");
                let delta = if negate {
                    format!("-{magnitude}")
                } else {
                    magnitude.to_string()
                };
                format!(
                    "compare_offset_integers({left}, {base}, {delta}_i128, \
                     core::cmp::Ordering::{accepts})"
                )
            }
            OffsetMagnitude::ElapsedSeconds { seconds, .. } => {
                self.uses.helpers.insert("compare_offset_instants");
                let delta = if negate { -seconds } else { seconds };
                format!(
                    "compare_offset_instants({left}, {base}, {delta}_i64, \
                     core::cmp::Ordering::{accepts})"
                )
            }
        }
    }

    /// A path as `Option<&T>` at its end, before any normalization.
    fn reference(&mut self, resolved: &Resolved) -> String {
        let mut out = match &resolved.root {
            Root::Input(field) => format!("Some(&input.{})", name::value_ident(field)),
            Root::Stored(field) => format!("Some(&{}.data.{})", self.row, name::value_ident(field)),
            Root::State => format!("Some(&{}.state)", self.row),
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

    /// A path as the `Option<String>` an aggregate reads: [`Self::read`], with a `Timestamp` as its
    /// rendering and a `bool` as `true` or `false`.
    fn text(&mut self, resolved: &Resolved) -> String {
        match &resolved.kind {
            Kind::Opaque | Kind::Instant => {
                format!("{}.map(|value| value.0.clone())", self.reference(resolved))
            }
            Kind::Bool => format!("{}.map(|value| value.to_string())", self.read(resolved)),
            _ => self.read(resolved),
        }
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
            Kind::Number(Primitive::Decimal) | Kind::Instant => {
                format!("{reference}.map(|value| value.0.clone())")
            }
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

/// Whether one value source needs the value at its own target path before the outcome.
fn increments_previous(value: &ResolvedPayloadValue) -> bool {
    match value {
        ResolvedPayloadValue::Increment { .. } => true,
        ResolvedPayloadValue::Struct { fields } => {
            fields.iter().any(|field| increments_previous(&field.value))
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
