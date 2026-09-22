//! Exact, command-local retained-result observations, independent of event payload mappings.
use crate::scenario::{CommandRef, EventRef, InstanceName, OutcomeRef, ScenarioStep};
use crate::selection::Declaration;
use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedCommand, ResolvedOutcome};
use ess_domain::{Field, Primitive, QualifiedName, TypeRef};
use ess_primitives::node::Node;
use std::collections::{BTreeMap, BTreeSet};

/// Closed authority for one original invocation and its retained-result retry.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(try_from = "RawObservation")]
pub struct Observation {
    /// Scenario-local write-once result snapshot name.
    pub snapshot: InstanceName,
    /// Original command and successful outcome.
    pub origin: OutcomeRef,
    /// Retained-result branch of the same command.
    pub replay: OutcomeRef,
    /// Original observed entity instance, bound before capture.
    pub instance: InstanceName,
    /// The declared original invocation surface carrying this subject's identity.
    pub identity: Identity,
    /// Complete ordered response declaration.
    pub fields: Vec<Field>,
    /// Exact reachable nominal declaration authority.
    pub declarations: BTreeMap<QualifiedName, Declaration>,
}
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct RawObservation {
    snapshot: InstanceName,
    origin: OutcomeRef,
    replay: OutcomeRef,
    instance: InstanceName,
    identity: Identity,
    fields: Vec<Field>,
    declarations: BTreeMap<QualifiedName, Declaration>,
}
impl TryFrom<RawObservation> for Observation {
    type Error = String;
    fn try_from(r: RawObservation) -> Result<Self, String> {
        let result = Self {
            snapshot: r.snapshot,
            origin: r.origin,
            replay: r.replay,
            instance: r.instance,
            identity: r.identity,
            fields: r.fields,
            declarations: r.declarations,
        };
        result.validate()?;
        Ok(result)
    }
}
/// Identity authority retained from the original outcome, without a new caller-supplied id.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Identity {
    /// Existing subject selected by the original command input.
    Input {
        /// Original identity input field.
        field: String,
    },
    /// New subject whose identity the original success actually emitted.
    Event {
        /// The original identity-bearing event.
        event: EventRef,
        /// Its identity payload field.
        field: String,
    },
}
impl Identity {
    /// Read exactly the declared original identity, rejecting ambiguous event occurrences.
    pub fn read<'a>(
        &self,
        input: &'a BTreeMap<String, Node>,
        events: &'a [crate::target::ObservedEvent],
    ) -> Result<&'a Node, String> {
        match self {
            Self::Input { field } => input
                .get(field)
                .ok_or_else(|| "original input identity is missing".into()),
            Self::Event { event, field } => {
                let mut matching = events.iter().filter(|e| &e.event == event);
                let original = matching
                    .next()
                    .ok_or("original identity event is missing")?;
                if matching.next().is_some() {
                    return Err("original identity event is ambiguous".into());
                }
                original
                    .payload
                    .get(field)
                    .ok_or_else(|| "original event identity is missing".into())
            }
        }
    }
}

impl Observation {
    /// Resolve the complete response and original subject without inventing an event mapping.
    pub fn of(
        ir: &EssIr,
        command: &ResolvedCommand,
        outcome: &ResolvedOutcome,
        instance: InstanceName,
    ) -> Result<Self, String> {
        let replay = outcome
            .replays
            .as_ref()
            .ok_or("outcome has no replay origin")?;
        let fields: Vec<_> = command
            .response
            .iter()
            .map(|f| Field::new(&f.name, crate::accessor::unresolve(&f.type_ref)))
            .collect();
        let declarations = declarations_for(ir, &fields)?;
        let command = CommandRef::new(command.name.clone());
        let result = Self {
            snapshot: InstanceName::new("retained-result").map_err(|e| e.to_string())?,
            origin: OutcomeRef::new(command.clone(), replay.origin.clone()),
            replay: OutcomeRef::new(command, outcome.name.clone()),
            instance,
            identity: match &replay.subject.instance {
                ess_compiler::ir::ResolvedInstance::Supplied { field } => Identity::Input {
                    field: field.name.clone(),
                },
                ess_compiler::ir::ResolvedInstance::Observed { event, field } => Identity::Event {
                    event: EventRef::from(event),
                    field: field.name.clone(),
                },
            },
            fields,
            declarations,
        };
        result.validate()?;
        Ok(result)
    }
    /// Admit a finite exact-comparison schema before any target callback.
    pub fn validate(&self) -> Result<(), String> {
        let (Identity::Input { field } | Identity::Event { field, .. }) = &self.identity;
        // Reuse the domain's field parser instead of inventing a second identifier grammar.
        let _: Field = serde_json::from_value(serde_json::json!({"name": field, "type": "String"}))
            .map_err(|error| error.to_string())?;
        if self.origin.command != self.replay.command || self.origin == self.replay {
            return Err("replay must name a distinct origin in the same command".into());
        }
        ExactShape {
            fields: &self.fields,
            declarations: &self.declarations,
        }
        .validate()?;
        if serde_json::to_vec(self).map_err(|e| e.to_string())?.len() > 1_048_576 {
            return Err("replay schema byte limit".into());
        }
        Ok(())
    }
}

pub(crate) fn declarations_for(
    ir: &EssIr,
    fields: &[Field],
) -> Result<BTreeMap<QualifiedName, Declaration>, String> {
    let mut pending: Vec<_> = fields
        .iter()
        .flat_map(|f| f.type_ref.named_dependencies().into_iter().cloned())
        .collect();
    let mut declarations = BTreeMap::new();
    while let Some(name) = pending.pop() {
        if declarations.contains_key(&name) {
            continue;
        }
        if declarations.len() >= 4096 {
            return Err("replay declaration limit".into());
        }
        let ty = ir.types().get(&name).ok_or("missing response type")?;
        if ty.reading.is_some()
            || matches!(&ty.body, ResolvedBody::Newtype { invariants, .. } | ResolvedBody::Struct { invariants, .. } if !invariants.is_empty())
        {
            return Err("replay response invariant/reading observer is unsupported".into());
        }
        let body = match &ty.body {
            ResolvedBody::Newtype { of, .. } => Declaration::Newtype {
                of: crate::accessor::unresolve(of),
            },
            ResolvedBody::Struct { fields, .. } => Declaration::Struct {
                fields: fields
                    .iter()
                    .map(|f| Field::new(&f.name, crate::accessor::unresolve(&f.type_ref)))
                    .collect(),
            },
            ResolvedBody::Enum { variants } => Declaration::Enum {
                variants: variants.iter().map(|v| v.name().to_owned()).collect(),
            },
            ResolvedBody::Union { tag, variants } => Declaration::Union {
                tag: tag.clone(),
                variants: variants
                    .iter()
                    .map(|(k, v)| (k.clone(), crate::accessor::unresolve(v)))
                    .collect(),
            },
        };
        for ty in body.references() {
            pending.extend(ty.named_dependencies().into_iter().cloned());
        }
        declarations.insert(name, body);
    }
    Ok(declarations)
}

pub(crate) struct ExactShape<'a> {
    pub fields: &'a [Field],
    pub declarations: &'a BTreeMap<QualifiedName, Declaration>,
}
impl ExactShape<'_> {
    pub fn validate(&self) -> Result<(), String> {
        if self.fields.is_empty() || self.fields.len() > 256 || self.declarations.len() > 4096 {
            return Err("replay schema resource limit".into());
        }
        let mut registry = ess_domain::TypeRegistry::new();
        for (name, body) in self.declarations {
            let declared = ess_domain::NamedType::try_from(ess_domain::types::RawNamedType {
                name: name.clone(),
                body: body.body(),
                naming: ess_domain::Naming::default(),
                reading: None,
            })
            .map_err(|e| e.to_string())?;
            registry.insert(declared).map_err(|e| e.to_string())?;
        }
        let mut used = BTreeSet::new();
        let mut names = BTreeSet::new();
        for field in self.fields {
            if field.name.is_empty() || !names.insert(&field.name) {
                return Err("duplicate replay field".into());
            }
            registry
                .resolve(&field.type_ref, "replay")
                .into_result(())
                .map_err(|e| e.to_string())?;
            self.check_type(&field.type_ref, &mut used, &mut BTreeSet::new(), 0)?;
        }
        if used.len() != self.declarations.len() {
            return Err("unrelated replay declaration".into());
        }
        if serde_json::to_vec(&(self.fields, self.declarations))
            .map_err(|e| e.to_string())?
            .len()
            > 1_048_576
        {
            return Err("replay schema byte limit".into());
        }
        Ok(())
    }
    fn check_type(
        &self,
        ty: &TypeRef,
        used: &mut BTreeSet<QualifiedName>,
        stack: &mut BTreeSet<QualifiedName>,
        depth: usize,
    ) -> Result<(), String> {
        if depth > 128 {
            return Err("replay type depth limit".into());
        }
        match ty {
            TypeRef::Named(name) => {
                if !stack.insert(name.clone()) {
                    return Err("recursive replay response".into());
                }
                if !used.contains(name) {
                    for ty in self
                        .declarations
                        .get(name)
                        .ok_or("missing replay type")?
                        .references()
                    {
                        self.check_type(ty, used, stack, depth + 1)?;
                    }
                    used.insert(name.clone());
                }
                stack.remove(name);
            }
            TypeRef::List(of) | TypeRef::Optional(of) => {
                self.check_type(of, used, stack, depth + 1)?;
            }
            TypeRef::Map(key, of) => {
                if *key != Primitive::String {
                    return Err("replay map key must be String".into());
                }
                self.check_type(of, used, stack, depth + 1)?;
            }
            TypeRef::Primitive(Primitive::Decimal | Primitive::Binary64) => {
                return Err("Decimal and Binary64 replay results are unsupported".into())
            }
            TypeRef::Primitive(_) => {}
        }
        Ok(())
    }
}

impl Observation {
    /// Validate an entire actual response, including optional presence and undeclared members.
    pub fn admit_result(&self, response: Option<&BTreeMap<String, Node>>) -> Result<(), String> {
        self.validate()?;
        let response = response.ok_or("command returned no response")?;
        if response
            .keys()
            .any(|k| !self.fields.iter().any(|f| &f.name == k))
        {
            return Err("undeclared response field".into());
        }
        let mut bytes = 0;
        for field in &self.fields {
            crate::selection::validate_response_value(
                &field.type_ref,
                response.get(&field.name),
                &self.declarations,
                &mut bytes,
            )?;
        }
        if bytes > 1_048_576 {
            return Err("replay response byte limit".into());
        }
        Ok(())
    }
    /// Exact admitted comparison: integers never pass through floating point; absence is not null.
    pub fn compare(
        &self,
        original: &BTreeMap<String, Node>,
        replay: Option<&BTreeMap<String, Node>>,
    ) -> Result<(), String> {
        self.admit_result(Some(original))?;
        self.admit_result(replay)?;
        if replay != Some(original) {
            return Err("retry result differs from retained original response".into());
        }
        Ok(())
    }
}

/// Whether a suite needs retained-result vocabulary.
pub fn used_by(suite: &crate::ConformanceSuite) -> bool {
    suite.scenarios.values().any(|s| {
        s.steps.iter().any(|step| {
            matches!(
                step,
                ScenarioStep::CaptureCommandResult { .. }
                    | ScenarioStep::SnapshotCompleteSubject { .. }
                    | ScenarioStep::ExpectCompleteSubjectUnchanged { .. }
                    | ScenarioStep::ExpectReplayResult { .. }
                    | ScenarioStep::ExpectNoEvents
            )
        })
    })
}

/// Reject unbound, overwritten, or substituted captures before target callbacks.
pub(crate) fn validate_steps(steps: &[ScenarioStep]) -> Result<(), String> {
    crate::subject::validate_steps(steps)?;
    validate_bindings(steps)?;
    let mut active: Option<&Observation> = None;
    let mut views = BTreeSet::new();
    let mut unchanged = BTreeSet::new();
    let mut invocation = 0_usize;
    let mut captured_at = 0_usize;
    let mut arguments = None;
    let mut captured_arguments = None;
    let mut queried = None;
    for step in steps {
        match step {
            ScenarioStep::ConfigureExternalOutcome { .. }
                if active.is_some() || !unchanged.is_empty() =>
            {
                return Err("retained result retry cannot use fault injection".into())
            }
            ScenarioStep::ExecuteCommand {
                command: c,
                actor,
                input,
            } => {
                if !unchanged.is_empty() {
                    return Err("command interrupted retained subject comparison".into());
                }
                if let Some(capture) = active {
                    if views.is_empty()
                        || captured_arguments != Some((actor, input))
                        || c != &capture.replay.command
                        || invocation != captured_at
                    {
                        return Err(
                            "retry substituted invocation or preceded original subject snapshot"
                                .into(),
                        );
                    }
                }
                invocation += 1;
                queried = None;
                arguments = Some((actor, input));
            }
            ScenarioStep::CaptureCommandResult { capture } => {
                if active.is_some() || !unchanged.is_empty() {
                    return Err("aliased or incomplete retained result snapshot".into());
                }
                active = Some(capture);
                captured_at = invocation;
                captured_arguments = arguments;
                views.clear();
            }
            ScenarioStep::ExpectReplayResult { capture } => {
                if active != Some(capture) || views.is_empty() || invocation != captured_at + 1 {
                    return Err(
                        "replay lacks original subject snapshots and a subsequent retry".into(),
                    );
                }
                unchanged.clone_from(&views);
                active = None;
            }
            ScenarioStep::SnapshotCompleteSubject { .. } if !unchanged.is_empty() => {
                return Err("original subject snapshot overwritten after retry".into())
            }
            ScenarioStep::SnapshotCompleteSubject { view, subject, .. } if active.is_some() => {
                let expected = &active.expect("guarded").instance;
                if queried != Some((view, invocation)) || invocation != captured_at || subject.is_empty() || !subject.values().all(|v| matches!(v, crate::ScenarioValue::Instance { instance } if instance == expected)) || !views.insert(view) {
                    return Err("original subject snapshot is aliased, overwritten, or names a different identity".into());
                }
            }
            ScenarioStep::ExpectCompleteSubjectUnchanged { view } if !unchanged.is_empty() => {
                if queried != Some((view, invocation)) || !unchanged.remove(view) {
                    return Err("replay compares a different subject snapshot".into());
                }
            }
            ScenarioStep::QueryView { view, .. } => queried = Some((view, invocation)),
            _ => {}
        }
    }
    if active.is_some() || !unchanged.is_empty() {
        return Err("retained result has no complete subject comparison".into());
    }
    Ok(())
}

fn validate_bindings(steps: &[ScenarioStep]) -> Result<(), String> {
    let mut bindings = BTreeMap::new();
    let mut captures = BTreeMap::new();
    let mut command = None;
    let mut outcome = None;
    let mut input = None;
    let mut invocation = 0_usize;
    for step in steps {
        match step {
            ScenarioStep::ExecuteCommand {
                command: c,
                input: args,
                ..
            } => {
                invocation += 1;
                command = Some(c);
                outcome = None;
                input = Some(args);
            }
            ScenarioStep::ExpectOutcome { outcome: o } => outcome = Some(o),
            ScenarioStep::CaptureInstance {
                instance,
                event,
                field,
                ..
            } => {
                if captures
                    .values()
                    .any(|c: &&Observation| &c.instance == instance)
                {
                    return Err("captured subject identity was overwritten".into());
                }
                bindings.insert(instance, Some((event, field, invocation)));
            }
            ScenarioStep::EstablishEntity { instance, .. } => {
                if captures
                    .values()
                    .any(|c: &&Observation| &c.instance == instance)
                {
                    return Err("captured subject identity was overwritten".into());
                }
                bindings.insert(instance, None);
            }
            ScenarioStep::CaptureCommandResult { capture } => {
                capture.validate()?;
                if command != Some(&capture.origin.command)
                    || outcome != Some(&capture.origin)
                    || !bindings.contains_key(&capture.instance)
                    || captures.insert(&capture.snapshot, capture).is_some()
                {
                    return Err("invalid or overwritten original result capture".into());
                }
                let selected = match &capture.identity {
                    Identity::Input { field } => input.and_then(|i| i.get(field)).is_some_and(|v| matches!(v, crate::ScenarioValue::Instance { instance } if instance == &capture.instance)),
                    Identity::Event { event, field } => bindings.get(&capture.instance) == Some(&Some((event, field, invocation))),
                };
                if !selected {
                    return Err(
                        "original capture is not bound to the declared invocation identity".into(),
                    );
                }
            }
            ScenarioStep::ExpectReplayResult { capture }
                if captures.get(&capture.snapshot) != Some(&capture)
                    || command != Some(&capture.replay.command)
                    || outcome != Some(&capture.replay) =>
            {
                return Err("replay has no matching original capture".into());
            }
            _ => {}
        }
    }
    Ok(())
}

pub(crate) fn admit_suite(
    suite: &crate::ConformanceSuite,
) -> Result<(), crate::admission::AdmissionError> {
    use crate::admission::AdmissionError;
    if used_by(suite) && suite.provenance.suite_version.major() < 12 {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "$suite",
            "retained results require suite/12 or /13",
        ));
    }
    for scenario in suite.scenarios.values() {
        validate_steps(&scenario.steps)
            .map_err(|e| AdmissionError::new("InvalidReplay", "$suite", e))?;
    }
    Ok(())
}
