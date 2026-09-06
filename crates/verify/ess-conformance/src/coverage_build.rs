//! Fresh coverage inventory from the actual synthesis and authored compiler results.
use crate::{
    coverage::{
        self, AdmittedInput, AuthoredSource, Counts, Disposition, Effect, Filter, Inventory,
        Knowledge, Origin, Origins, Outside, OutsideReason, Refusal, RefusalScope, Retained, Scope,
        Selection, SourceIdentity,
    },
    AdmissionError, AdmittedSuite, ConformanceSuite, EssSemanticRef, ScenarioId,
};
use ess_compiler::EssIr;
use std::collections::BTreeMap;

/// One explicitly identified original UTF-8 authored input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoverageSource {
    identity: SourceIdentity,
    text: String,
}
type Candidate = Result<(ScenarioId, crate::ConformanceScenario), Vec<crate::authored::Refusal>>;

/// An independently compiled batch, bound to its model and every original source.
#[derive(Debug, Clone)]
pub struct AuthoredBatch {
    provenance: crate::SuiteProvenance,
    sources: Vec<CoverageSource>,
    candidates: Vec<Candidate>,
}
impl AuthoredBatch {
    /// Number of candidates this compilation accepted, before later final merging.
    pub fn accepted(&self) -> usize {
        self.candidates.iter().filter(|c| c.is_ok()).count()
    }
    /// Every original occurrence from this compilation, before later final merging.
    pub fn refusals(&self) -> impl Iterator<Item = &crate::authored::Refusal> {
        self.candidates
            .iter()
            .filter_map(|c| c.as_ref().err())
            .flatten()
    }
    /// Exact sources this independent compilation consumed.
    pub fn sources(&self) -> &[CoverageSource] {
        &self.sources
    }
}
/// Compile one closed input set independently. Final merging is a separate operation.
pub fn compile_sources(
    ir: &EssIr,
    sources: &[CoverageSource],
) -> Result<AuthoredBatch, AdmissionError> {
    crate::admission::model(ir)?;
    let mut ordered = BTreeMap::new();
    for source in sources {
        if ordered.insert(&source.identity, source).is_some() {
            return Err(coverage::error("repeated authored input identity"));
        }
    }
    let mut seen = BTreeMap::new();
    let mut candidates = Vec::new();
    let mut sources = Vec::new();
    for source in ordered.values() {
        let raw = crate::authored::Source::new(source.identity.as_str(), &source.text);
        let candidate = crate::authored::compile_one(ir, &raw, &seen);
        if let Ok((id, _)) = &candidate {
            seen.insert(id.clone(), source.identity.as_str().into());
        }
        sources.push((*source).clone());
        candidates.push(candidate);
    }
    Ok(AuthoredBatch {
        provenance: crate::SuiteProvenance::of(ir),
        sources,
        candidates,
    })
}
/// Merge separately compiled batches, retaining first ownership by checked source identity.
pub fn merge_batches(
    ir: &EssIr,
    batches: &[AuthoredBatch],
    scope: Scope,
    origins: Origins,
) -> Result<AdmittedInput, AdmissionError> {
    merge_inventory(ir, batches, scope, origins, false)
}
impl CoverageSource {
    /// Check the supplied identity; retain every original text byte.
    pub fn new(
        identity: impl Into<String>,
        text: impl Into<String>,
    ) -> Result<Self, AdmissionError> {
        Ok(Self {
            identity: SourceIdentity::new(identity)?,
            text: text.into(),
        })
    }
    /// Checked input-root-relative identity.
    pub fn identity(&self) -> &SourceIdentity {
        &self.identity
    }
    /// Original UTF-8 source bytes represented as a string.
    pub fn text(&self) -> &str {
        &self.text
    }
}
/// Build an opt-in known inventory from a model and the complete explicitly supplied source set.
pub fn build(
    ir: &EssIr,
    sources: &[CoverageSource],
    scope: Scope,
    origins: Origins,
) -> Result<AdmittedInput, AdmissionError> {
    merge_batches(ir, &[compile_sources(ir, sources)?], scope, origins)
}
/// Enumerate known generated obligations even when a fresh origin selection excludes them.
///
/// This differs from authored-only acquisition: every generated candidate and refusal is retained.
pub fn build_with_known_generated(
    ir: &EssIr,
    sources: &[CoverageSource],
    scope: Scope,
    origins: Origins,
) -> Result<AdmittedInput, AdmissionError> {
    merge_inventory(ir, &[compile_sources(ir, sources)?], scope, origins, true)
}
fn merge_inventory(
    ir: &EssIr,
    batches: &[AuthoredBatch],
    scope: Scope,
    origins: Origins,
    known_generated: bool,
) -> Result<AdmittedInput, AdmissionError> {
    crate::admission::model(ir)?;
    let synthesis = if known_generated || origins.includes(Origin::Generated) {
        crate::synthesize(ir)
    } else {
        crate::Synthesis {
            suite: ConformanceSuite::new(crate::SuiteProvenance::of(ir)),
            refusals: Vec::new(),
            outside: Vec::new(),
        }
    };
    finish_inventory(ir, batches, scope, origins, synthesis)
}
fn finish_inventory(
    ir: &EssIr,
    batches: &[AuthoredBatch],
    scope: Scope,
    origins: Origins,
    synthesis: crate::Synthesis,
) -> Result<AdmittedInput, AdmissionError> {
    crate::admission::model(ir)?;
    let component = match &scope {
        Scope::System => None,
        Scope::Component { component } => Some(
            ir.components()
                .get(component)
                .ok_or_else(|| coverage::error(format!("unknown component {component}")))?,
        ),
    };
    let mut ordered = BTreeMap::new();
    for batch in batches {
        if batch.provenance != crate::SuiteProvenance::of(ir) {
            return Err(coverage::error(
                "authored batch belongs to a different model",
            ));
        }
        for (source, candidate) in batch.sources.iter().zip(&batch.candidates) {
            if ordered
                .insert(&source.identity, (source, candidate))
                .is_some()
            {
                return Err(coverage::error("repeated authored input identity"));
            }
        }
    }
    let mut suite = synthesis.suite;
    suite.provenance.suite_version =
        crate::scenario::SuiteFormat::parse(coverage::COVERAGE_SUITE_FORMAT)
            .expect("constant suite version");
    suite.provenance.component = component.map(|c| c.name.to_string());
    let mut owners: BTreeMap<ScenarioId, Retained> = suite
        .scenarios
        .keys()
        .map(|id| {
            (
                id.clone(),
                Retained {
                    origin: Origin::Generated,
                    source: None,
                },
            )
        })
        .collect();
    let mut inventory = Inventory {
        selection: Selection {
            scope,
            origins,
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
    for refusal in synthesis.refusals {
        inventory.refused.push(Refusal {
            origin: Origin::Generated,
            scenario: refusal.scenario,
            subject: Some(refusal.subject),
            source: None,
            code: refusal.cause.code().to_string(),
            message: refusal.cause.to_string(),
            effect: generated_effect(&refusal.cause),
            retained: None,
            scope: RefusalScope::InScope,
            needs: Vec::new(),
        });
    }
    let rejected_needs = append_authored(
        ir,
        component,
        ordered,
        &mut suite,
        &mut inventory,
        &mut owners,
    )?;
    partition(
        &mut suite,
        &mut inventory,
        &owners,
        component.map(|c| (ir, c)),
    );
    classify(&mut inventory, &owners, &rejected_needs);
    inventory.sort_and_count()?;
    let original = coverage::suite_document(&suite, &inventory)?;
    AdmittedInput::from_suite(AdmittedSuite::from_json(&original)?)
}

fn append_authored(
    ir: &EssIr,
    component: Option<&ess_compiler::ir::ResolvedComponent>,
    ordered: BTreeMap<&SourceIdentity, (&CoverageSource, &Candidate)>,
    suite: &mut ConformanceSuite,
    inventory: &mut Inventory,
    owners: &mut BTreeMap<ScenarioId, Retained>,
) -> Result<BTreeMap<SourceIdentity, Vec<EssSemanticRef>>, AdmissionError> {
    let mut rejected_needs = BTreeMap::new();
    for (identity, (source, compiled)) in ordered {
        let raw = crate::authored::Source::new(identity.as_str(), &source.text);
        let (scenario, disposition) = match compiled.clone() {
            Ok((id, scenario)) => {
                if suite.scenarios.contains_key(&id) {
                    let needs = component
                        .map_or_else(Vec::new, |c| crate::synthesize::needs_of(ir, c, &scenario));
                    rejected_needs.insert(identity.clone(), needs);
                    let first = owners
                        .get(&id)
                        .and_then(|r| r.source.as_ref())
                        .map_or_else(|| "generated scenario".into(), |s| s.as_str().into());
                    inventory.refused.push(authored_refusal(
                        &crate::authored::Refusal {
                            origin: identity.as_str().into(),
                            scenario: Some(id.clone()),
                            cause: crate::authored::Cause::Duplicate { first },
                        },
                        identity,
                    )?);
                    (Some(id), Disposition::Refused)
                } else {
                    owners.insert(
                        id.clone(),
                        Retained {
                            origin: Origin::Authored,
                            source: Some(identity.clone()),
                        },
                    );
                    suite.scenarios.insert(id.clone(), scenario);
                    (Some(id), Disposition::Accepted)
                }
            }
            Err(refusals) => {
                // A duplicate's survivor cannot prove its rejected candidate's dependencies.
                // Compile that candidate independently when possible; otherwise stay in scope.
                if let (Some(component), Ok((_, candidate))) = (
                    component,
                    crate::authored::compile_one(ir, &raw, &BTreeMap::new()),
                ) {
                    rejected_needs.insert(
                        identity.clone(),
                        crate::synthesize::needs_of(ir, component, &candidate),
                    );
                }
                let scenario = refusals.first().and_then(|r| r.scenario.clone());
                for refusal in refusals {
                    inventory
                        .refused
                        .push(authored_refusal(&refusal, identity)?);
                }
                (scenario, Disposition::Refused)
            }
        };
        inventory.authored_sources.insert(
            identity.clone(),
            AuthoredSource {
                digest: coverage::digest(&source.text),
                scenario,
                disposition,
            },
        );
    }
    Ok(rejected_needs)
}

fn partition(
    suite: &mut ConformanceSuite,
    inventory: &mut Inventory,
    owners: &BTreeMap<ScenarioId, Retained>,
    component: Option<(&EssIr, &ess_compiler::ir::ResolvedComponent)>,
) {
    suite.scenarios.retain(|id, scenario| {
        let origin = owners[id].origin;
        let needs = component.map_or_else(Vec::new, |(ir, c)| {
            crate::synthesize::needs_of(ir, c, scenario)
        });
        let reason = if !inventory.selection.origins.includes(origin) {
            Some(OutsideReason::OriginSelection)
        } else if needs.is_empty() {
            None
        } else {
            Some(OutsideReason::OtherComponent)
        };
        if let Some(reason) = reason {
            inventory.outside.push(Outside {
                scenario: id.clone(),
                origin,
                reason,
                needs: if reason == OutsideReason::OtherComponent {
                    needs
                } else {
                    Vec::new()
                },
            });
            false
        } else {
            match origin {
                Origin::Generated => inventory.generated.push(id.clone()),
                Origin::Authored => inventory.authored.push(id.clone()),
            }
            true
        }
    });
}
fn classify(
    inventory: &mut Inventory,
    owners: &BTreeMap<ScenarioId, Retained>,
    rejected_needs: &BTreeMap<SourceIdentity, Vec<EssSemanticRef>>,
) {
    for refusal in &mut inventory.refused {
        refusal.retained = refusal
            .scenario
            .as_ref()
            .and_then(|id| owners.get(id))
            .cloned();
        if !inventory.selection.origins.includes(refusal.origin) {
            refusal.scope = RefusalScope::OutsideOrigin;
            continue;
        }
        let proof = if refusal.effect == Effect::CheckNotEmitted {
            inventory
                .outside
                .iter()
                .find(|o| {
                    Some(&o.scenario) == refusal.scenario.as_ref()
                        && o.reason == OutsideReason::OtherComponent
                })
                .map(|o| &o.needs)
        } else {
            refusal.source.as_ref().and_then(|s| rejected_needs.get(s))
        };
        if let Some(needs) = proof.filter(|n| !n.is_empty()) {
            refusal.scope = RefusalScope::OutsideComponent;
            refusal.needs.clone_from(needs);
        }
    }
}
fn generated_effect(cause: &crate::RefusalCause) -> Effect {
    use crate::RefusalCause;
    match cause {
        RefusalCause::ViewUndecidable { .. }
        | RefusalCause::OrderUnwitnessed { .. }
        | RefusalCause::InvariantUnobservable { .. }
        | RefusalCause::RefusalUndeclared { .. } => Effect::CheckNotEmitted,
        RefusalCause::NoWitness(_)
        | RefusalCause::GuardUnevaluable(_)
        | RefusalCause::GuardUnsatisfiable { .. }
        | RefusalCause::InstanceRequired { .. }
        | RefusalCause::NotSynthesisedYet { .. }
        | RefusalCause::DuplicateScenario
        | RefusalCause::StrategyWithoutGuard { .. }
        | RefusalCause::WitnessRejected(_)
        | RefusalCause::BindingUnobservable { .. }
        | RefusalCause::ValueInvariantUnwitnessed { .. } => Effect::CandidateNotEmitted,
    }
}
fn authored_refusal(
    refusal: &crate::authored::Refusal,
    source: &SourceIdentity,
) -> Result<Refusal, AdmissionError> {
    use crate::authored::Cause;
    // Exhaustive producer mapping: a new cause requires an explicit wire decision.
    let effect = match &refusal.cause {
        Cause::UnsupportedBinary64 { .. } => {
            return Err(coverage::error(
                "Binary64 model refusal is not authored coverage",
            ))
        }
        Cause::Unreadable { .. }
        | Cause::UnsupportedFormat { .. }
        | Cause::Duplicate { .. }
        | Cause::UndeclaredDomain { .. }
        | Cause::UndeclaredEntity { .. }
        | Cause::UndeclaredCommand { .. }
        | Cause::UndeclaredOutcome { .. }
        | Cause::UndeclaredActor { .. }
        | Cause::ActorMayNot { .. }
        | Cause::UndeclaredEvent { .. }
        | Cause::UndeclaredError { .. }
        | Cause::UndeclaredView { .. }
        | Cause::UndeclaredField { .. }
        | Cause::MissingField { .. }
        | Cause::ValueRejected { .. }
        | Cause::UndeclaredVariant { .. }
        | Cause::UndeclaredState { .. }
        | Cause::UnarrangedInstance { .. }
        | Cause::UnboundInstance { .. }
        | Cause::Unobserved { .. }
        | Cause::NotComparable { .. }
        | Cause::InstanceMistyped { .. }
        | Cause::UnorderedTimeline { .. }
        | Cause::Unordered { .. }
        | Cause::AmbiguousClaim { .. }
        | Cause::UnreadablePredicate { .. }
        | Cause::NothingHappens
        | Cause::UnmarkedInstant { .. }
        | Cause::DuplicateInstant { .. }
        | Cause::VacuousWindow { .. }
        | Cause::QuietAboutNothing { .. }
        | Cause::WindowContradictsTimeline { .. }
        | Cause::AmbiguousWindow { .. }
        | Cause::HaltsAtNothing { .. }
        | Cause::InvalidPredicate { .. } => Effect::CandidateNotEmitted,
    };
    Ok(Refusal {
        origin: Origin::Authored,
        scenario: refusal.scenario.clone(),
        subject: None,
        source: Some(source.clone()),
        code: refusal.code().to_string(),
        message: refusal.cause.to_string(),
        effect,
        retained: None,
        scope: RefusalScope::InScope,
        needs: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn billing() -> EssIr {
        use ess_compiler::{resolve::compile, source::SourceMap};
        use ess_domain::{
            spec::{RawSpecFile, Specification},
            system::Source,
        };
        let originals = [
            (
                "system.yaml",
                include_str!("../../../../examples/billing/system.yaml"),
            ),
            (
                "components.yaml",
                include_str!("../../../../examples/billing/components.yaml"),
            ),
            (
                "topology.yaml",
                include_str!("../../../../examples/billing/topology.yaml"),
            ),
            (
                "domains/invoice.yaml",
                include_str!("../../../../examples/billing/domains/invoice.yaml"),
            ),
            (
                "domains/email.yaml",
                include_str!("../../../../examples/billing/domains/email.yaml"),
            ),
        ];
        let mut sources = SourceMap::new();
        let parsed: Vec<_> = originals
            .iter()
            .map(|(name, text)| {
                sources.insert(*name, *text);
                (Source::new(*name), RawSpecFile::parse(text).unwrap())
            })
            .collect();
        compile(&Specification::assemble(parsed).unwrap(), &sources).unwrap()
    }
    #[test]
    fn actual_duplicate_insertion_keeps_the_generated_survivor_and_refusal_through_execution() {
        let ir = billing();
        let mut synthesis = crate::synthesize(&ir);
        let id = ScenarioId::parse("billing.invoice.CreateInvoice/outcome/accepted").unwrap();
        let first = synthesis.suite.scenarios[&id].clone();
        assert!(synthesis.refusals.is_empty());
        crate::synthesize::insert(
            &mut synthesis.suite,
            id.clone(),
            first.clone(),
            &mut synthesis.refusals,
        );
        assert_eq!(synthesis.suite.scenarios[&id], first);
        assert_eq!(synthesis.refusals.len(), 1);
        assert_eq!(
            synthesis.refusals[0].cause,
            crate::RefusalCause::DuplicateScenario
        );
        let input =
            finish_inventory(&ir, &[], Scope::System, Origins::Generated, synthesis).unwrap();
        let inventory = input.selected().coverage().unwrap();
        assert_eq!(inventory.counts.generated, 29);
        assert_eq!(inventory.counts.refused, 1);
        let refusal = &inventory.refused[0];
        assert_eq!(refusal.scenario.as_ref(), Some(&id));
        assert_eq!(refusal.code, "ESS-SYNTH-007");
        assert_eq!(refusal.message, "a second scenario claimed this id");
        assert_eq!(refusal.effect, Effect::CandidateNotEmitted);
        assert_eq!(
            refusal.retained,
            Some(Retained {
                origin: Origin::Generated,
                source: None
            })
        );
        assert_eq!(refusal.scope, RefusalScope::InScope);
        let selected = input.select(&[id]).unwrap();
        let run = crate::Runner::for_suite(selected.selected().suite())
            .run_admitted(selected.selected(), &crate::reference::Billing::new());
        let report = crate::CountReport::from_run(&run, selected.selected()).unwrap();
        assert_eq!(report.counts().passed, 1);
        assert_eq!(report.counts().total, 1);
        assert_eq!(report.execution_status(), crate::CountStatus::Passed);
        assert_eq!(
            report.conformance_status(),
            crate::CountStatus::Inconclusive
        );
        let json = report.to_canonical_json().unwrap();
        assert_eq!(
            crate::CountReport::from_json(&json, selected.selected()).unwrap(),
            report
        );
    }
}
