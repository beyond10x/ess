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
        let mut constraints = BTreeMap::new();
        for name in declarations.keys() {
            let declared = ir.types().get(name).ok_or("missing one-time declaration")?;
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
        let mut registry = TypeRegistry::new();
        for (name, body) in &self.declarations {
            let mut raw = body.body();
            if let Some(rules) = self.constraints.get(name) {
                let RawTypeBody::Newtype {
                    alphabet,
                    prefix,
                    invariants,
                    ..
                } = &mut raw
                else {
                    return Err("one-time constraint authority requires a String newtype".into());
                };
                if !self.is_string(&TypeRef::Named(name.clone())) {
                    return Err("one-time constraint authority requires a String newtype".into());
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
            .map_err(|_| "invalid one-time declaration")?;
            registry
                .insert(declared)
                .map_err(|_| "duplicate one-time declaration")?;
        }
        if self
            .constraints
            .keys()
            .any(|name| !self.declarations.contains_key(name))
        {
            return Err("unrelated one-time constraint authority".into());
        }
        for declared in registry.iter() {
            if !declared.validate_alphabet(&registry).is_empty()
                || !declared.validate_prefix(&registry).is_empty()
                || !declared.validate_invariants(&registry).is_empty()
            {
                return Err("invalid one-time String constraints".into());
            }
        }
        Ok(())
    }

    fn is_string<'a>(&'a self, mut ty: &'a TypeRef) -> bool {
        let mut seen = BTreeSet::new();
        loop {
            match ty {
                TypeRef::Primitive(Primitive::String) => return true,
                TypeRef::Named(name) if seen.insert(name) => {
                    let Some(Declaration::Newtype { of }) = self.declarations.get(name) else {
                        return false;
                    };
                    ty = of;
                }
                _ => return false,
            }
        }
    }
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
