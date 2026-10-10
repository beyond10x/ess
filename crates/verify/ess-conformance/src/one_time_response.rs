//! Closed source-derived authority for bounded one-time disclosure observations.
use crate::{
    scenario::{CommandRef, EventRef, OutcomeRef},
    selection::Declaration,
};
use ess_domain::{
    types::{Primitive, RawNamedType, RawTypeBody},
    Field, QualifiedName, TypeRef, TypeRegistry,
};
use ess_primitives::predicate::Predicate;
use std::collections::{BTreeMap, BTreeSet};

mod cells;
pub use cells::{Aspect, Cell};
pub(crate) mod produce;

/// Whether a compiled model requires private disclosure observations.
/// Model-aware recording/import callers check this before observing or persisting target data.
pub fn marked_model(ir: &ess_compiler::EssIr) -> bool {
    ir.commands().values().any(|command| {
        command
            .outcomes
            .iter()
            .any(|outcome| !outcome.one_time_response.is_empty())
    })
}

/// Ordinary execution vocabulary containing a one-time trace policy.
pub const ORDINARY: u32 = 34;
/// Corresponding inventory-bearing execution vocabulary.
pub const COVERAGE: u32 = 35;
/// Maximum number of actual plaintext values retained by one trace.
pub const MAX_CAPTURES: usize = 256;
/// Maximum total UTF-8 capture bytes, independent of observed-payload bounds.
pub const MAX_CAPTURE_BYTES: usize = 1_048_576;

/// Concrete constraints on a reachable String newtype, never arbitrary metadata.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StringConstraints {
    /// Unicode scalar alphabet, when declared.
    pub alphabet: Option<String>,
    /// Exact literal prefix, when declared.
    pub prefix: Option<String>,
    /// Parsed predicates over `value`, preserving the shared predicate grammar.
    pub invariants: Vec<Predicate>,
}

/// Complete return schema, including constraints the legacy shape DTO cannot carry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Response {
    /// Complete declared response, not just the marked fields.
    pub fields: Vec<Field>,
    /// Exactly the reachable finite shape declarations.
    pub declarations: BTreeMap<QualifiedName, Declaration>,
    /// String-newtype rules keyed by the exact nominal declaration they constrain.
    pub constraints: BTreeMap<QualifiedName, StringConstraints>,
}

/// One exact successful originating outcome and its sole disclosure positions.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Origin {
    /// Command whose actual invocation produces this response.
    pub command: CommandRef,
    /// Successful selected outcome; command must match the command above.
    pub outcome: OutcomeRef,
    /// Actual response validation authority.
    pub response: Response,
    /// Nonempty distinct field names; each resolves to required String.
    pub fields: Vec<String>,
}

/// A finite independent-log observation window anchored after an actual operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventAuthority {
    /// Declared published event.
    pub event: EventRef,
    /// Source-derived finite delivery observation window; zero means immediate only.
    pub within_ms: u64,
}

/// A finite independent-log observation window anchored after an actual operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventWindow {
    /// Declared event to observe independently of direct command events.
    pub event: EventRef,
    /// Zero-based command/query step after which the window opens.
    pub after_step: usize,
    /// Duration whose completion must be established, not merely an early answer.
    pub within_ms: u64,
}

/// Scenario-wide authority. It contains no observed value or target exemption.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    /// Every marked origin reachable in this trace, in deterministic source order.
    pub origins: Vec<Origin>,
    /// Origins this witness must actually observe successfully; no empty-origin pass.
    pub required_origins: Vec<OutcomeRef>,
    /// Complete declared publication catalog required by this trace's source inventory.
    pub events: Vec<EventAuthority>,
    /// Independently observed event windows, including immediate (zero) scans.
    pub event_windows: Vec<EventWindow>,
}

impl Response {
    /// Derive complete authority from compiled response declarations, retaining String constraints.
    pub fn of(
        ir: &ess_compiler::EssIr,
        command: &ess_compiler::ir::ResolvedCommand,
    ) -> Result<Self, String> {
        let fields: Vec<_> = command
            .response
            .iter()
            .map(|field| {
                let mut result =
                    Field::new(&field.name, crate::accessor::unresolve(&field.type_ref));
                result.naming.presence = field.naming.presence;
                result
            })
            .collect();
        let declarations = crate::typed_fields::one_time_declarations(ir, &fields)?;
        // The one-time trace compares the response as a closed record in every runner, so a
        // response or struct that ignores undeclared fields (`ess/24`, beyond10x/ess#500) is
        // refused by name rather than failing an honest target.
        if command.undeclared_fields.is_ignored() {
            return Err(format!(
                "one-time response of `{}` declares `undeclared_fields: ignored`, which the \
                 one-time observer does not carry",
                command.name
            ));
        }
        if let Some(opened) = crate::undeclared_fields::opened(ir, &declarations)
            .into_iter()
            .next()
        {
            return Err(format!(
                "one-time response of `{}` reaches `{opened}`, which declares \
                 `undeclared_fields: ignored`, and the one-time observer does not carry it",
                command.name
            ));
        }
        let constraints = string_constraints(ir, &declarations, "one-time")?;
        let result = Self {
            fields,
            declarations,
            constraints,
        };
        result.validate()?;
        Ok(result)
    }

    /// Validate finite closed type authority before any target callback.
    pub fn validate(&self) -> Result<(), String> {
        if self.fields.is_empty() || self.fields.len() > 256 || self.declarations.len() > 4096 {
            return Err("one-time response schema bound".into());
        }
        crate::typed_fields::validate([self.fields.as_slice()], &self.declarations)?;
        validate_constraints(&self.declarations, &self.constraints, "one-time")
    }

    fn is_string(&self, ty: &TypeRef) -> bool {
        is_string(&self.declarations, ty)
    }
}

/// Whether `ty` is `String` or reaches it through newtypes alone.
pub(crate) fn is_string<'a>(
    declarations: &'a BTreeMap<QualifiedName, Declaration>,
    mut ty: &'a TypeRef,
) -> bool {
    let mut seen = BTreeSet::new();
    loop {
        match ty {
            TypeRef::Primitive(Primitive::String) => return true,
            TypeRef::Named(name) if seen.insert(name) => {
                let Some(Declaration::Newtype { of }) = declarations.get(name) else {
                    return false;
                };
                ty = of;
            }
            _ => return false,
        }
    }
}

/// The String-newtype rules of every constrained declaration among `declarations`, taken from the
/// compiled source, keyed by the nominal type they constrain. `noun` names the profile in a refusal.
pub(crate) fn string_constraints(
    ir: &ess_compiler::EssIr,
    declarations: &BTreeMap<QualifiedName, Declaration>,
    noun: &str,
) -> Result<BTreeMap<QualifiedName, StringConstraints>, String> {
    let mut constraints = BTreeMap::new();
    for name in declarations.keys() {
        let declared = ir
            .types()
            .get(name)
            .ok_or_else(|| format!("missing {noun} declaration"))?;
        if let ess_compiler::ir::ResolvedBody::Newtype {
            alphabet,
            prefix,
            invariants,
            ..
        } = &declared.body
        {
            if declared.body.is_constrained() {
                constraints.insert(
                    name.clone(),
                    StringConstraints {
                        alphabet: alphabet.clone(),
                        prefix: prefix.clone(),
                        invariants: invariants
                            .iter()
                            .map(|invariant| invariant.predicate.clone())
                            .collect(),
                    },
                );
            }
        }
    }
    Ok(constraints)
}

/// Admit String-newtype rules against the closed declarations they travel with: each names a
/// declared String newtype, and the rules are the ones a declaration could carry.
pub(crate) fn validate_constraints(
    declarations: &BTreeMap<QualifiedName, Declaration>,
    constraints: &BTreeMap<QualifiedName, StringConstraints>,
    noun: &str,
) -> Result<(), String> {
    let mut registry = TypeRegistry::new();
    for (name, body) in declarations {
        let mut raw = body.body();
        if let Some(rules) = constraints.get(name) {
            let RawTypeBody::Newtype {
                alphabet,
                prefix,
                invariants,
                ..
            } = &mut raw
            else {
                return Err(format!(
                    "{noun} constraint authority requires a String newtype"
                ));
            };
            if !is_string(declarations, &TypeRef::Named(name.clone())) {
                return Err(format!(
                    "{noun} constraint authority requires a String newtype"
                ));
            }
            alphabet.clone_from(&rules.alphabet);
            prefix.clone_from(&rules.prefix);
            *invariants = rules
                .invariants
                .iter()
                .map(|predicate| {
                    serde_json::from_value(
                        serde_json::to_value(predicate).map_err(|_| "invalid predicate")?,
                    )
                    .map_err(|_| "invalid predicate")
                })
                .collect::<Result<_, _>>()?;
        }
        let declared = ess_domain::NamedType::try_from(RawNamedType {
            name: name.clone(),
            body: raw,
            naming: ess_domain::Naming::default(),
            reading: None,
        })
        .map_err(|_| format!("invalid {noun} declaration"))?;
        registry
            .insert(declared)
            .map_err(|_| format!("duplicate {noun} declaration"))?;
    }
    if constraints
        .keys()
        .any(|name| !declarations.contains_key(name))
    {
        return Err(format!("unrelated {noun} constraint authority"));
    }
    for declared in registry.iter() {
        if !declared.validate_alphabet(&registry).is_empty()
            || !declared.validate_prefix(&registry).is_empty()
            || !declared.validate_invariants(&registry).is_empty()
        {
            return Err(format!("invalid {noun} String constraints"));
        }
    }
    Ok(())
}

/// Whether `predicate` decides over a lone `value` text fact: it reads `value` or its `.count`
/// and nothing else, through comparisons with literals, membership and text matches, and does
/// not quantify, offset, derive or read a calendar window. The one-time evaluators in every
/// runner hold the same profile; an invariant outside it is refused at synthesis by name.
pub(crate) fn decides_over_value(predicate: &Predicate) -> bool {
    use ess_primitives::predicate::{Operand, Predicate as P};
    let path = |path: &ess_primitives::facts::FactPath| {
        let text = path.to_string();
        text == "value" || text == "value.count"
    };
    let operand = |operand: &Operand| match operand {
        Operand::Fact(fact) => path(fact),
        Operand::Literal(_) => true,
        Operand::Offset(_) | Operand::Derived(_) => false,
    };
    match predicate {
        P::Always | P::Never => true,
        P::All(children) | P::Any(children) => children.iter().all(decides_over_value),
        P::Not(inner) => decides_over_value(inner),
        P::Compare { left, right, .. } => operand(left) && operand(right),
        P::Truthy(fact)
        | P::Defined(fact)
        | P::AnyOf { path: fact, .. }
        | P::NoneOf { path: fact, .. }
        | P::FoldMatch { path: fact, .. } => path(fact),
        P::TextMatch {
            path: fact, value, ..
        } => path(fact) && value.as_literal().is_some(),
        P::Forall(_) | P::Exists(_) | P::Distinct(_) | P::Window(_) => false,
    }
}

/// How a value broke a String-newtype rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Breach {
    /// The value is not what its declaration admits, or breaks a declared rule.
    Payload,
    /// An invariant could not be decided over the value.
    Undecided,
}

/// Check `value`, declared as `ty`, against every String-newtype rule reachable from it: the
/// value itself, record fields, union variants and `Optional`, `List` and `Map` elements.
pub(crate) fn check(
    declarations: &BTreeMap<QualifiedName, Declaration>,
    constraints: &BTreeMap<QualifiedName, StringConstraints>,
    ty: &TypeRef,
    value: &ess_primitives::node::Node,
) -> Result<(), Breach> {
    use ess_primitives::node::Node;
    let recurse = |ty: &TypeRef, value: &Node| check(declarations, constraints, ty, value);
    match (ty, value) {
        (TypeRef::Named(name), _) => {
            if let Some(rules) = constraints.get(name) {
                let Node::Text(text) = value else {
                    return Err(Breach::Payload);
                };
                if rules.alphabet.as_ref().is_some_and(|alphabet| {
                    text.chars().any(|character| !alphabet.contains(character))
                }) || rules
                    .prefix
                    .as_ref()
                    .is_some_and(|prefix| !text.starts_with(prefix))
                {
                    return Err(Breach::Payload);
                }
                let mut facts = ess_primitives::facts::FactStore::new();
                facts.set(
                    ess_primitives::facts::FactPath::new("value").map_err(|_| Breach::Payload)?,
                    ess_primitives::facts::FactValue::Text(text.clone()),
                );
                for predicate in &rules.invariants {
                    match predicate.evaluate(&facts) {
                        ess_primitives::predicate::Truth::True => {}
                        ess_primitives::predicate::Truth::False => return Err(Breach::Payload),
                        ess_primitives::predicate::Truth::Unknown => return Err(Breach::Undecided),
                    }
                }
            }
            match declarations.get(name).ok_or(Breach::Payload)? {
                Declaration::Newtype { of } => recurse(of, value)?,
                Declaration::Struct { fields } => {
                    let Node::Map(values) = value else {
                        return Err(Breach::Payload);
                    };
                    for field in fields {
                        if let Some(value) = values.get(&field.name) {
                            recurse(&field.type_ref, value)?;
                        }
                    }
                }
                Declaration::Union { tag, variants } => {
                    let Node::Map(values) = value else {
                        return Err(Breach::Payload);
                    };
                    let Some(Node::Text(label)) = values.get(tag) else {
                        return Err(Breach::Payload);
                    };
                    let ty = variants.get(label).ok_or(Breach::Payload)?;
                    match (ty, values.get(ess_gen::schema::union_content_key(tag))) {
                        (Some(ty), Some(value)) => recurse(ty, value)?,
                        // A unit variant (ess/22) is the tag alone.
                        (None, Some(_)) => return Err(Breach::Payload),
                        (_, None) => {}
                    }
                }
                Declaration::Enum { .. } => {}
            }
        }
        (TypeRef::Optional(_), Node::Null) => {}
        (TypeRef::Optional(of), _) => recurse(of, value)?,
        (TypeRef::List(of), Node::Seq(values)) => {
            for value in values {
                recurse(of, value)?;
            }
        }
        (TypeRef::Map(_, of), Node::Map(values)) => {
            for value in values.values() {
                recurse(of, value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

impl Trace {
    /// Admit closed origin ownership and bounded event-window references.
    pub fn validate(&self, scenario: &crate::scenario::ConformanceScenario) -> Result<(), String> {
        let steps = &scenario.steps;
        if self.origins.is_empty()
            || self.origins.len() > MAX_CAPTURES
            || self.event_windows.len() > 65_536
        {
            return Err("one-time trace authority bound".into());
        }
        let mut origins = BTreeSet::new();
        let mut schemas = BTreeMap::new();
        for origin in &self.origins {
            if origin.command != origin.outcome.command
                || !origins.insert(&origin.outcome)
                || origin.fields.is_empty()
                || origin.fields.len() > MAX_CAPTURES
            {
                return Err("invalid or duplicate one-time origin".into());
            }
            if !steps.iter().any(|step| matches!(step, crate::ScenarioStep::ExecuteCommand { command, .. }
                | crate::ScenarioStep::ExecuteCommandWithoutInput { command, .. } if command == &origin.command)) {
                return Err("one-time origin has no invocation in its trace".into());
            }
            origin.response.validate()?;
            if !scenario.source.contains(&origin.command.clone().into())
                || !scenario.source.contains(&origin.outcome.clone().into())
            {
                return Err("one-time origin is absent from source dependencies".into());
            }
            if schemas
                .insert(&origin.command, &origin.response)
                .is_some_and(|prior| prior != &origin.response)
            {
                return Err("one command has contradictory one-time response schemas".into());
            }
            let mut fields = BTreeSet::new();
            for field in &origin.fields {
                let Some(declared) = origin
                    .response
                    .fields
                    .iter()
                    .find(|declared| &declared.name == field)
                else {
                    return Err("one-time origin names an undeclared response field".into());
                };
                if !fields.insert(field) || !origin.response.is_string(&declared.type_ref) {
                    return Err("one-time fields must be distinct required Strings".into());
                }
            }
        }
        let mut required = BTreeSet::new();
        if self.required_origins.is_empty() {
            return Err("one-time trace must require an observed origin".into());
        }
        for origin in &self.required_origins {
            if !origins.contains(origin) || !required.insert(origin) {
                return Err("invalid or duplicate required one-time origin".into());
            }
        }
        if serde_json::to_vec(self)
            .map_err(|_| "invalid one-time authority")?
            .len()
            > 1_048_576
        {
            return Err("one-time authority byte limit".into());
        }
        self.validate_windows(scenario)
    }

    fn validate_windows(
        &self,
        scenario: &crate::scenario::ConformanceScenario,
    ) -> Result<(), String> {
        let steps = &scenario.steps;
        let mut events = BTreeSet::new();
        for authority in &self.events {
            let event = &authority.event;
            if !events.insert(event) || !scenario.source.contains(&event.clone().into()) {
                return Err("one-time event catalog contradicts source dependencies".into());
            }
        }
        let mut windows = BTreeSet::new();
        for window in &self.event_windows {
            if !events.contains(&window.event) {
                return Err("one-time window names an event outside its catalog".into());
            }
            if !windows.insert((&window.event, window.after_step, window.within_ms)) {
                return Err("duplicate one-time event window".into());
            }
            if !matches!(
                steps.get(window.after_step),
                Some(
                    crate::ScenarioStep::ExecuteCommand { .. }
                        | crate::ScenarioStep::ExecuteCommandWithoutInput { .. }
                        | crate::ScenarioStep::QueryView { .. }
                        | crate::ScenarioStep::EventuallyView { .. }
                )
            ) {
                return Err("one-time event window is not anchored to an operation".into());
            }
        }
        for (index, step) in steps.iter().enumerate() {
            if matches!(
                step,
                crate::ScenarioStep::ExecuteCommand { .. }
                    | crate::ScenarioStep::ExecuteCommandWithoutInput { .. }
                    | crate::ScenarioStep::QueryView { .. }
                    | crate::ScenarioStep::EventuallyView { .. }
            ) {
                for authority in &self.events {
                    if !windows.contains(&(&authority.event, index, 0))
                        || !windows.contains(&(&authority.event, index, authority.within_ms))
                    {
                        return Err(
                            "one-time trace omits a required event observation window".into()
                        );
                    }
                }
            }
        }
        Ok(())
    }
}

/// Whether the new closed policy occurs in an assembled suite.
pub fn used_by(suite: &crate::ConformanceSuite) -> bool {
    suite.scenarios.iter().any(|(id, scenario)| {
        matches!(id, crate::ScenarioId::Disclosure { .. }) || scenario.one_time_response.is_some()
    })
}

pub(crate) fn admit(suite: &crate::ConformanceSuite) -> Result<(), crate::AdmissionError> {
    for (id, scenario) in &suite.scenarios {
        if let crate::ScenarioId::Disclosure { cell } = id {
            if suite.provenance.suite_version.major() < ORDINARY {
                return Err(crate::AdmissionError::new(
                    "UnsupportedVocabulary",
                    "$suite",
                    "disclosure identity requires suite/34 or /35",
                ));
            }
            cell.validate(scenario).map_err(|reason| {
                crate::AdmissionError::new("InvalidOneTimeResponse", "$suite", reason)
            })?;
        }
        if let Some(trace) = &scenario.one_time_response {
            if suite.provenance.suite_version.major() < ORDINARY {
                return Err(crate::AdmissionError::new(
                    "UnsupportedVocabulary",
                    "$suite",
                    "one-time response authority requires suite/34 or /35",
                ));
            }
            trace.validate(scenario).map_err(|reason| {
                crate::AdmissionError::new("InvalidOneTimeResponse", "$suite", reason)
            })?;
        }
    }
    Ok(())
}
