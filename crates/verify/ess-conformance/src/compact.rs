//! Closed live authoring lowered through original component contracts into native steps.
use crate::{
    authored::Source,
    coverage::{
        self, AdmittedInput, AuthoredSource, Counts, Disposition, Filter, Inventory, Knowledge,
        Origins, Scope, Selection, SourceIdentity,
    },
    models::Models,
    recipes::{self, Bound, Capture, Library, Location},
    scenario::{CommandRef, EventRef, InstanceName, PayloadShape, ScenarioPurpose, SuiteFormat},
    AdmittedSuite, ConformanceScenario, ConformanceSuite, EssSemanticRef, ScenarioId, ScenarioStep,
    ScenarioValue, SuiteProvenance,
};
use ess_composition::ServiceKey;
use ess_primitives::evidence::SpecDigest;
use std::collections::{BTreeMap, BTreeSet};

/// Compact live document format; legacy timeline formats keep their existing meaning.
pub const FORMAT: &str = "ess-scenario/3";

/// One named setup recipe.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Given {
    /// Recipe library key.
    pub recipe: InstanceName,
    /// Distinct name exposing this invocation's outputs.
    #[serde(rename = "as")]
    pub alias: InstanceName,
    /// Literals or outputs captured by an earlier fixture.
    #[serde(default)]
    pub with: BTreeMap<InstanceName, recipes::Value>,
}

/// A real command or a barrier that waits for an actual event.
#[derive(Debug, serde::Deserialize)]
#[serde(tag = "do", rename_all = "snake_case", deny_unknown_fields)]
pub enum Action {
    /// Invoke an admitted command.
    Command {
        /// Original model owner.
        service: ServiceKey,
        /// Declared command.
        command: CommandRef,
        /// Complete input.
        #[serde(default)]
        input: BTreeMap<String, recipes::Value>,
        /// Local name mapped to an actual response field.
        #[serde(default)]
        capture: BTreeMap<InstanceName, String>,
    },
    /// Wait without issuing a command.
    Observe {
        /// Original event owner.
        service: ServiceKey,
        /// Declared event.
        event: EventRef,
        /// Payload paths constrained by literals or earlier captures.
        matches: BTreeMap<String, recipes::Value>,
        /// Name this actual occurrence for later temporal assertions.
        #[serde(default)]
        anchor: Option<InstanceName>,
        /// Require an occurrence strictly after an earlier named observation.
        #[serde(default)]
        after: Option<InstanceName>,
    },
}

/// One payload-sensitive eventual assertion.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    /// Original model owner.
    pub service: ServiceKey,
    /// Declared event.
    pub event: EventRef,
    /// Original dotted payload paths.
    pub matches: BTreeMap<String, recipes::Value>,
    /// Name an eventual observation; required by later temporal references.
    #[serde(default)]
    pub anchor: Option<InstanceName>,
    /// Earlier occurrence for strict eventual ordering.
    #[serde(default)]
    pub after: Option<InstanceName>,
    /// Observed start of an absence or stability interval.
    #[serde(default)]
    pub since: Option<InstanceName>,
    /// Bounded absence length, in milliseconds.
    #[serde(default)]
    pub absent_for_ms: Option<u64>,
    /// Continuous snapshot stability length, in milliseconds.
    #[serde(default)]
    pub stable_for_ms: Option<u64>,
    /// Payload values that must hold on every scoped snapshot.
    #[serde(default)]
    pub required: BTreeMap<String, recipes::Value>,
}

/// A compact Given/When/Then document with no executable snippets or pretend timestamps.
#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Document {
    /// Exactly `ess-scenario/3`.
    #[serde(rename = "type")]
    pub format: String,
    /// Domain in an original selected model.
    pub domain: crate::scenario::DomainRef,
    /// Stable authored name.
    pub scenario: crate::scenario::AuthoredName,
    /// Human-readable claim.
    pub summary: ScenarioPurpose,
    /// Ordered fixture uses.
    #[serde(default)]
    pub given: Vec<Given>,
    /// Ordered stimulus and observation barriers.
    pub when: Vec<Action>,
    /// Nonempty final observations.
    pub then: Vec<Observation>,
}

/// Compilation result before output publication.
#[derive(Debug)]
pub struct Compilation {
    /// Exact native source manifest bound by the admitted suite's provenance digest.
    pub manifest: String,
    /// Admitted coverage input, complete for the explicit authored source set only.
    pub input: AdmittedInput,
    /// Exact canonical composition whose digest identifies this suite.
    pub composition: String,
    /// Every emitted step's outer-to-inner authored expansion locations.
    pub locations: BTreeMap<ScenarioId, Vec<Vec<Location>>>,
    /// Exact recipe identities reached by expansion.
    pub recipes: BTreeMap<InstanceName, recipes::SourceIdentity>,
}

#[derive(serde::Serialize)]
struct InputFile {
    digest: String,
    text: String,
}
impl InputFile {
    fn new(text: String) -> Self {
        Self {
            digest: coverage::digest(&text),
            text,
        }
    }
}
#[derive(serde::Serialize)]
struct InputManifest<'a> {
    format: &'static str,
    composition: InputFile,
    models: BTreeMap<ServiceKey, InputFile>,
    recipes: BTreeMap<SourceIdentity, InputFile>,
    scenarios: BTreeMap<SourceIdentity, InputFile>,
    locations: &'a BTreeMap<ScenarioId, Vec<Vec<Location>>>,
}
fn input_files(sources: &[Source]) -> Result<BTreeMap<SourceIdentity, InputFile>, String> {
    sources
        .iter()
        .map(|source| {
            Ok((
                SourceIdentity::new(source.origin.clone()).map_err(|e| e.to_string())?,
                InputFile::new(source.text.clone()),
            ))
        })
        .collect()
}

/// Compile the entire explicit source set atomically. Any refused source prevents output;
/// none is silently dropped from a successful inventory.
pub fn compile(
    models: &Models<'_>,
    library: &Library,
    sources: &[Source],
) -> Result<Compilation, String> {
    if sources.is_empty() || sources.len() > 4096 {
        return Err("compact compilation requires 1..4096 explicit sources".into());
    }
    refuse_ambiguous_exports(models)?;
    let composition = models.composition().to_canonical_json();
    let digest = coverage::digest(&composition);
    let mut suite = ConformanceSuite::new(SuiteProvenance {
        live_inputs_digest: None,
        suite_version: SuiteFormat::parse("ess-conformance/11").expect("constant format"),
        system: models.composition().composition().to_string(),
        specification_version: models.composition().format().into(),
        spec_digest: SpecDigest::new(&digest[7..]).expect("native digest"),
        contract_digest: SpecDigest::new(&digest[7..]).expect("native digest"),
        component: None,
    });
    let mut inventory = Inventory {
        selection: Selection {
            scope: Scope::System,
            origins: Origins::Authored,
            filter: Filter::All,
        },
        knowledge: Knowledge::CompleteInventory,
        generated: Vec::new(),
        authored: Vec::new(),
        outside: Vec::new(),
        refused: Vec::new(),
        authored_sources: BTreeMap::new(),
        counts: Counts {
            generated: 0,
            authored: 0,
            outside: 0,
            refused: 0,
        },
    };
    let mut locations = BTreeMap::new();
    let mut recipes = BTreeMap::new();
    for source in sources {
        let identity = SourceIdentity::new(source.origin.clone()).map_err(|e| e.to_string())?;
        if inventory.authored_sources.contains_key(&identity) {
            return Err(format!("duplicate source {}", source.origin));
        }
        let (id, lower) = lower_source(models, library, source, &mut recipes)?;
        suite
            .insert(id.clone(), lower.scenario)
            .map_err(|id| format!("duplicate scenario {id}"))?;
        locations.insert(id.clone(), lower.locations);
        inventory.authored.push(id.clone());
        inventory.authored_sources.insert(
            identity,
            AuthoredSource {
                digest: coverage::digest(&source.text),
                scenario: Some(id),
                disposition: Disposition::Accepted,
            },
        );
    }
    inventory.sort_and_count().map_err(|e| e.to_string())?;
    let manifest = coverage::canonical(&InputManifest {
        format: "ess-live-inputs/1",
        composition: InputFile::new(composition.clone()),
        models: models
            .canonical_originals()
            .into_iter()
            .map(|(key, text)| (key, InputFile::new(text)))
            .collect(),
        recipes: input_files(library.sources())?,
        scenarios: input_files(sources)?,
        locations: &locations,
    })
    .map_err(|e| e.to_string())?;
    suite.provenance.live_inputs_digest =
        Some(SpecDigest::new(&coverage::digest(&manifest)[7..]).expect("native digest"));
    let original =
        coverage::compact_suite_document(&suite, &inventory).map_err(|e| e.to_string())?;
    let input =
        AdmittedInput::from_suite(AdmittedSuite::from_json(&original).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    Ok(Compilation {
        manifest,
        input,
        composition,
        locations,
        recipes,
    })
}

fn lower_source<'a, 'm, 's>(
    models: &'m Models<'a>,
    library: &Library,
    source: &'s Source,
    recipes: &mut BTreeMap<InstanceName, recipes::SourceIdentity>,
) -> Result<(ScenarioId, Lowering<'a, 'm, 's>), String> {
    let doc: Document =
        serde_yaml::from_str(&source.text).map_err(|e| format!("{}: {e}", source.origin))?;
    if doc.format != FORMAT || doc.then.is_empty() || doc.when.is_empty() {
        return Err(format!(
            "{}: expected {FORMAT} with nonempty when and then",
            source.origin
        ));
    }
    let id = ScenarioId::Authored {
        domain: doc.domain.clone(),
        name: doc.scenario,
    };
    let mut lower = Lowering::new(models, &source.origin, doc.summary);
    let mut aliases = BTreeSet::new();
    for (index, given) in doc.given.iter().enumerate() {
        if !aliases.insert(&given.alias) {
            return Err(format!(
                "{}: duplicate fixture alias {}",
                source.origin, given.alias
            ));
        }
        lower.given(given, index, library, recipes)?;
    }
    for (index, action) in doc.when.iter().enumerate() {
        lower.action(action, index)?;
    }
    for (index, claim) in doc.then.iter().enumerate() {
        lower.observation(claim, vec![lower.location(format!("/then/{index}"))])?;
    }
    let events = lower
        .scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::CheckLive { trace } => Some(trace.events()),
            _ => None,
        })
        .flatten()
        .collect::<BTreeSet<_>>();
    if !events.is_empty() {
        lower.scenario.steps.insert(
            0,
            ScenarioStep::CheckLive {
                trace: crate::live_trace::Check::Open { events },
            },
        );
        lower.locations.insert(0, vec![lower.location("/".into())]);
    }
    if !lower.domains.contains(&doc.domain.to_string()) {
        return Err(format!(
            "{}: scenario domain is not a dependency of its selected contracts",
            source.origin
        ));
    }
    Ok((id, lower))
}

fn refuse_ambiguous_exports(models: &Models<'_>) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for service in models.composition().services().values() {
        for name in service
            .commands()
            .iter()
            .map(ToString::to_string)
            .chain(service.events().iter().map(ToString::to_string))
        {
            if !seen.insert(name.clone()) {
                return Err(format!(
                    "ambiguous executable name across selected services: {name}"
                ));
            }
        }
    }
    Ok(())
}

struct Lowering<'a, 'm, 's> {
    models: &'m Models<'a>,
    origin: &'s str,
    scenario: ConformanceScenario,
    locations: Vec<Vec<Location>>,
    captures: BTreeMap<String, Capture<'a>>,
    slots: BTreeMap<String, InstanceName>,
    domains: BTreeSet<String>,
}
impl<'a, 'm, 's> Lowering<'a, 'm, 's> {
    fn action(&mut self, action: &Action, index: usize) -> Result<(), String> {
        let stack = vec![self.location(format!("/when/{index}"))];
        match action {
            Action::Command {
                service,
                command,
                input,
                capture,
            } => {
                let owner = self
                    .models
                    .command(service, command)
                    .ok_or_else(|| format!("unadmitted command {service}:{command}"))?;
                let values = input
                    .iter()
                    .map(|(key, value)| Ok((key.clone(), self.resolve(value)?)))
                    .collect::<Result<_, String>>()?;
                let mut captures = Vec::new();
                for (name, field) in capture {
                    let declared = owner
                        .declaration()
                        .response
                        .iter()
                        .find(|f| &f.name == field)
                        .ok_or_else(|| format!("{command}: unknown response field {field}"))?;
                    captures.push((
                        name.to_string(),
                        Capture {
                            slot: format!("when/{index}/{name}"),
                            service: service.clone(),
                            command: owner.clone(),
                            field: declared,
                        },
                    ));
                }
                self.command(
                    service,
                    &owner,
                    &values,
                    &captures.iter().map(|(_, c)| c.clone()).collect::<Vec<_>>(),
                    &stack,
                )?;
                for (name, capture) in captures {
                    self.expose(&name, capture)?;
                }
            }
            Action::Observe {
                service,
                event,
                matches,
                anchor,
                after,
            } => {
                if let Some(anchor) = anchor {
                    let claim = self.claim(service, event, matches)?;
                    self.emit(
                        ScenarioStep::CheckLive {
                            trace: crate::live_trace::Check::Await {
                                anchor: anchor.clone(),
                                after: after.clone(),
                                claim,
                            },
                        },
                        stack,
                    )?;
                } else {
                    if after.is_some() {
                        return Err("an ordered observation needs an anchor name".into());
                    }
                    self.observe(service, event, matches, stack)?;
                }
            }
        }
        Ok(())
    }

    fn new(models: &'m Models<'a>, origin: &'s str, purpose: ScenarioPurpose) -> Self {
        Self {
            models,
            origin,
            scenario: ConformanceScenario {
                purpose,
                steps: Vec::new(),
                source: BTreeSet::new(),
            },
            locations: Vec::new(),
            captures: BTreeMap::new(),
            slots: BTreeMap::new(),
            domains: BTreeSet::new(),
        }
    }
    fn location(&self, pointer: String) -> Location {
        Location {
            origin: self.origin.into(),
            pointer,
        }
    }
    fn given(
        &mut self,
        given: &Given,
        index: usize,
        library: &Library,
        recipes: &mut BTreeMap<InstanceName, recipes::SourceIdentity>,
    ) -> Result<(), String> {
        let args = given
            .with
            .iter()
            .map(|(key, value)| Ok((key.clone(), self.resolve(value)?)))
            .collect::<Result<_, String>>()?;
        let expansion =
            library.expand_with_bindings(self.models, &given.recipe, &given.alias, &args)?;
        recipes.extend(expansion.sources);
        for invocation in expansion.commands {
            let mut stack = vec![self.location(format!("/given/{index}"))];
            stack.extend(invocation.locations);
            self.command(
                &invocation.service,
                &invocation.command,
                &invocation.input,
                &invocation.captures,
                &stack,
            )?;
        }
        for (output, capture) in expansion.outputs {
            self.expose(&format!("{}.{}", given.alias, output), capture)?;
        }
        Ok(())
    }
    fn expose(&mut self, name: &str, capture: Capture<'a>) -> Result<(), String> {
        if self.captures.insert(name.to_owned(), capture).is_some() {
            return Err(format!("duplicate capture {name}"));
        }
        Ok(())
    }
    fn resolve(&self, value: &recipes::Value) -> Result<Bound<'a>, String> {
        match value {
            recipes::Value::Literal(value) => Ok(Bound::Literal(value.clone())),
            recipes::Value::Capture(name) => self
                .captures
                .get(name)
                .cloned()
                .map(Bound::Capture)
                .ok_or_else(|| format!("unknown or forward capture {name}")),
            recipes::Value::Parameter(_) => {
                Err("parameters are only valid inside fixture recipes".into())
            }
        }
    }
    fn value(&self, bound: &Bound<'_>) -> Result<ScenarioValue, String> {
        match bound {
            Bound::Literal(value) => Ok(ScenarioValue::Literal {
                value: value.clone(),
            }),
            Bound::Capture(capture) => self
                .slots
                .get(&capture.slot)
                .cloned()
                .map(|instance| ScenarioValue::Instance { instance })
                .ok_or_else(|| format!("unbound actual response {}", capture.slot)),
        }
    }
    fn emit(&mut self, step: ScenarioStep, locations: Vec<Location>) -> Result<(), String> {
        if self.scenario.steps.len() >= 16384 {
            return Err("compact expanded step limit".into());
        }
        self.scenario.steps.push(step);
        self.locations.push(locations);
        Ok(())
    }
    fn dependency(&mut self, name: &str) {
        if let Some((domain, _)) = name.rsplit_once('.') {
            self.domains.insert(domain.into());
        }
    }
    fn command(
        &mut self,
        service: &ServiceKey,
        owner: &crate::models::Owned<'a, ess_compiler::ir::ResolvedCommand>,
        values: &BTreeMap<String, Bound<'a>>,
        captures: &[Capture<'a>],
        locations: &[Location],
    ) -> Result<(), String> {
        let declared = owner.declaration();
        let command = CommandRef::new(declared.name.clone());
        self.dependency(&command.to_string());
        self.scenario
            .source
            .insert(EssSemanticRef::from(command.clone()));
        self.scenario
            .source
            .extend(owner.types().keys().cloned().map(EssSemanticRef::from));
        for (key, value) in values {
            let field = declared
                .input
                .iter()
                .find(|f| &f.name == key)
                .ok_or_else(|| format!("{command}: unknown input {key}"))?;
            recipes::check_value(service, owner, field, value)?;
        }
        if let Some(field) = declared
            .input
            .iter()
            .find(|f| !f.type_ref.is_optional() && !values.contains_key(&f.name))
        {
            return Err(format!("{command}: missing required input {}", field.name));
        }
        let input = values
            .iter()
            .map(|(key, value)| Ok((key.clone(), self.value(value)?)))
            .collect::<Result<_, String>>()?;
        self.emit(
            ScenarioStep::ExecuteCommand {
                command: command.clone(),
                input,
                actor: None,
            },
            locations.to_vec(),
        )?;
        for capture in captures {
            let mut shape = PayloadShape::new();
            crate::synthesize::describe(
                owner.model(),
                &capture.field.type_ref,
                &capture.field.name,
                false,
                0,
                &mut shape,
            );
            if shape
                .leaves()
                .get(&capture.field.name)
                .is_none_or(|leaf| leaf.optional)
            {
                return Err(format!(
                    "{command}: capture needs a required scalar or container field {}",
                    capture.field.name
                ));
            }
            let instance = InstanceName::new(format!("capture-{}", self.slots.len()))
                .map_err(|e| e.to_string())?;
            if self
                .slots
                .insert(capture.slot.clone(), instance.clone())
                .is_some()
            {
                return Err("duplicate internal capture slot".into());
            }
            self.emit(
                ScenarioStep::CaptureResponse {
                    command: command.clone(),
                    instance,
                    field: capture.field.name.clone(),
                    shape,
                },
                locations.to_vec(),
            )?;
        }
        Ok(())
    }
    fn observe(
        &mut self,
        service: &ServiceKey,
        event: &EventRef,
        matches: &BTreeMap<String, recipes::Value>,
        locations: Vec<Location>,
    ) -> Result<(), String> {
        let claim = self.claim(service, event, matches)?;
        self.emit(
            ScenarioStep::EventuallyMatchingEvent {
                event: claim.event,
                matches: claim.matches,
                shape: claim.shape,
            },
            locations,
        )
    }
    fn observation(
        &mut self,
        observation: &Observation,
        locations: Vec<Location>,
    ) -> Result<(), String> {
        use crate::live_trace::Check;
        let o = observation;
        let temporal = o.anchor.is_some()
            || o.since.is_some()
            || o.absent_for_ms.is_some()
            || o.stable_for_ms.is_some();
        if !temporal {
            if o.after.is_some() || !o.required.is_empty() {
                return Err("temporal fields require a named observation".into());
            }
            return self.observe(&o.service, &o.event, &o.matches, locations);
        }
        let claim = self.claim(&o.service, &o.event, &o.matches)?;
        let trace = match (&o.anchor, &o.since, o.absent_for_ms, o.stable_for_ms) {
            (Some(anchor), None, None, None) if o.required.is_empty() => Check::Await {
                anchor: anchor.clone(),
                after: o.after.clone(),
                claim,
            },
            (None, Some(anchor), Some(duration_ms), None)
                if o.required.is_empty() && o.after.is_none() =>
            {
                Check::Absent {
                    anchor: anchor.clone(),
                    duration_ms,
                    claim,
                }
            }
            (None, Some(anchor), None, Some(duration_ms)) if o.after.is_none() => {
                let required = self.claim(&o.service, &o.event, &o.required)?;
                Check::Stable {
                    anchor: anchor.clone(),
                    duration_ms,
                    claim,
                    required: required.matches,
                    shape: required.shape,
                }
            }
            _ => return Err("conflicting or incomplete temporal observation fields".into()),
        };
        self.emit(ScenarioStep::CheckLive { trace }, locations)
    }
    fn claim(
        &mut self,
        service: &ServiceKey,
        event: &EventRef,
        matches: &BTreeMap<String, recipes::Value>,
    ) -> Result<crate::live_trace::Claim, String> {
        let owner = self
            .models
            .event(service, event)
            .ok_or_else(|| format!("unadmitted event {service}:{event}"))?;
        self.dependency(&event.to_string());
        self.scenario
            .source
            .insert(EssSemanticRef::from(event.clone()));
        self.scenario
            .source
            .extend(owner.types().keys().cloned().map(EssSemanticRef::from));
        let declared = crate::synthesize::payload_shape(owner.model(), event);
        let mut shape = PayloadShape::new();
        let mut values = BTreeMap::new();
        for (path, authored) in matches {
            let leaf = declared
                .leaves()
                .get(path)
                .ok_or_else(|| format!("{event}: undeclared or unsupported payload path {path}"))?;
            let bound = self.resolve(authored)?;
            match &bound {
                Bound::Literal(value) if !leaf.admits(Some(value)) => {
                    return Err(format!("{event}: literal violates {path}"))
                }
                Bound::Capture(capture) => {
                    let mut captured = PayloadShape::new();
                    crate::synthesize::describe(
                        capture.command.model(),
                        &capture.field.type_ref,
                        &capture.field.name,
                        false,
                        0,
                        &mut captured,
                    );
                    if captured.leaves().get(&capture.field.name).map(|v| &v.holds)
                        != Some(&leaf.holds)
                    {
                        return Err(format!(
                            "{event}: capture representation differs from {path}"
                        ));
                    }
                }
                Bound::Literal(_) => {}
            }
            shape.insert(path, leaf.clone());
            values.insert(path.clone(), self.value(&bound)?);
        }
        Ok(crate::live_trace::Claim {
            event: event.clone(),
            matches: values,
            shape,
        })
    }
}
