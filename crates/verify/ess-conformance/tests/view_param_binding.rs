//! A view parameter compared by `==` with a stored field is sent the value the scenario stored
//! there, whatever the parameter is named, a member of a literal struct identity included
//! (beyond10x/ess#428).
//!
//! The binding is read off the filter: `owner == param.who` sends `who` the stored owner,
//! `ref.tenant == param.tenant` sends the tenant member of the identity the scenario created, and a
//! parameter named after one field but compared with another is sent the field it is compared
//! with. A parameter the filter does not decide keeps its name binding, so a same-named parameter
//! keeps its bytes. A member of an identity the scenario holds only as a captured instance or an
//! observed value has no projection, and is refused saying so.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::fmt::Write;

use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::{synthesize, Synthesis},
    AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue, ViewExpectation,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{consistency::ConsistencyToken, node::Node};
use sha2::{Digest, Sha256};

const MODEL: &str = include_str!("fixtures/view-param-binding.yaml");
const RELATED: &str = include_str!("fixtures/related-copied-view-parameter.yaml");
const CREATED: &str = "demo.vault.CreateItem/outcome/created";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("vault.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn scenario<'a>(synthesis: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || panic!("missing {id}: {:?}", synthesis.refusals),
            |(_, scenario)| scenario,
        )
}

fn refusals_naming(synthesis: &Synthesis, view: &str) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains(view))
        .collect()
}

/// The input of the send under test: the command step its own event expectation follows.
fn subject_input(scenario: &ConformanceScenario) -> &BTreeMap<String, ScenarioValue> {
    scenario
        .steps
        .iter()
        .enumerate()
        .find_map(|(at, step)| match step {
            ScenarioStep::ExecuteCommand { input, .. }
                if matches!(
                    scenario.steps.get(at + 2),
                    Some(ScenarioStep::ExpectEvent { .. } | ScenarioStep::ExpectEventValues { .. })
                ) =>
            {
                Some(input)
            }
            _ => None,
        })
        .expect("the send under test")
}

fn literal(value: &ScenarioValue) -> &Node {
    match value {
        ScenarioValue::Literal { value } => value,
        other => panic!("not a literal: {other:?}"),
    }
}

fn member<'a>(value: &'a Node, name: &str) -> &'a Node {
    match value {
        Node::Map(members) => &members[name],
        other => panic!("not a struct: {other:?}"),
    }
}

/// Every read of `view` in a scenario, by the parameters it sends.
fn reads<'a>(
    scenario: &'a ConformanceScenario,
    view: &str,
) -> Vec<&'a BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::QueryView { view: name, params } if name.to_string() == view => {
                Some(params)
            }
            _ => None,
        })
        .collect()
}

fn expectations<'a>(scenario: &'a ConformanceScenario, view: &str) -> Vec<&'a ViewExpectation> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                view: name,
                expectation,
            } if name.to_string() == view => Some(expectation),
            _ => None,
        })
        .collect()
}

#[test]
fn differently_named_param_binds_from_filter() {
    let synthesis = synthesize(&ir(MODEL));
    assert_eq!(
        refusals_naming(&synthesis, "demo.vault.ByOtherName"),
        Vec::<String>::new()
    );
    let created = scenario(&synthesis, CREATED);
    let owner = literal(&subject_input(created)["owner"]);
    let reads = reads(created, "demo.vault.ByOtherName");
    assert!(!reads.is_empty(), "{created:#?}");
    for params in reads {
        assert_eq!(literal(&params["who"]), owner, "{params:?}");
    }
    assert!(expectations(created, "demo.vault.ByOtherName")
        .iter()
        .any(|expectation| matches!(expectation, ViewExpectation::Contains { .. })));
}

#[test]
fn struct_member_param_binds_from_literal_identity() {
    let synthesis = synthesize(&ir(MODEL));
    assert_eq!(
        refusals_naming(&synthesis, "demo.vault.ByTenant"),
        Vec::<String>::new()
    );
    let created = scenario(&synthesis, CREATED);
    let tenant = member(literal(&subject_input(created)["ref"]), "tenant");
    let reads = reads(created, "demo.vault.ByTenant");
    assert!(!reads.is_empty(), "{created:#?}");
    for params in reads {
        assert_eq!(literal(&params["tenant"]), tenant, "{params:?}");
    }
    let found = expectations(created, "demo.vault.ByTenant");
    assert!(
        found
            .iter()
            .any(|expectation| matches!(expectation, ViewExpectation::Contains { .. })),
        "{found:#?}"
    );
    assert!(
        found
            .iter()
            .any(|expectation| matches!(expectation, ViewExpectation::Excludes { .. })),
        "a decoy row of another tenant is excluded: {found:#?}"
    );
    let decoys: Vec<&Node> = created
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => input.get("ref"),
            _ => None,
        })
        .filter_map(|value| match value {
            ScenarioValue::Literal { value } => Some(member(value, "tenant")),
            _ => None,
        })
        .filter(|other| *other != tenant)
        .collect();
    assert!(!decoys.is_empty(), "a row of another tenant: {created:#?}");
}

#[test]
fn filter_binding_wins_over_name() {
    let synthesis = synthesize(&ir(MODEL));
    let created = scenario(&synthesis, CREATED);
    let input = subject_input(created);
    assert_ne!(input["owner"], input["delegate"], "the two fields differ");
    let delegate = literal(&input["delegate"]);
    let reads = reads(created, "demo.vault.ByCrossed");
    assert!(!reads.is_empty(), "{created:#?}");
    for params in reads {
        assert_eq!(literal(&params["owner"]), delegate, "{params:?}");
    }
    assert!(expectations(created, "demo.vault.ByCrossed")
        .iter()
        .any(|expectation| matches!(expectation, ViewExpectation::Contains { .. })));
}

fn digest(text: &str) -> String {
    let mut digest = String::new();
    for byte in Sha256::digest(text.as_bytes()) {
        write!(digest, "{byte:02x}").unwrap();
    }
    digest
}

/// A model whose only view compares a field with a parameter of the same name, and the #360
/// fixture: both bound by name before this change, and the filter agrees, so not a byte moves.
#[test]
fn same_named_param_bytes_unchanged() {
    let same = MODEL
        .split("  - name: demo.vault.ByOtherName")
        .next()
        .unwrap();
    for (name, model, pinned) in [
        (
            "same-named",
            same,
            "649161ab5493590d18efad48c44cb102c9c57f70b14b5980deb7543722e33bfe",
        ),
        (
            "related-copied",
            RELATED,
            "d5464390121dab5ffa0074d72d453060e377390e860153bbb07e1972c25a8059",
        ),
    ] {
        let synthesis = synthesize(&ir(model));
        let canonical = synthesis.suite.to_canonical_json().unwrap();
        assert_eq!(digest(&canonical), pinned, "{name}");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    IgnoreWho,
    IgnoreTenant,
    CrossedByOwner,
}

type Row = BTreeMap<String, Node>;

/// An honest item store, or one whose reads ignore one parameter.
struct Vault {
    mutant: Mutant,
    items: RefCell<Vec<Row>>,
    serial: Cell<u64>,
}

impl Vault {
    fn new(mutant: Mutant) -> Self {
        Self {
            mutant,
            items: RefCell::default(),
            serial: Cell::new(0),
        }
    }
}

impl ConformanceTarget for Vault {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("vault-428", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.items.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command: &CommandRef = &request.command;
        if command.to_string() != "demo.vault.CreateItem" {
            return Err(TargetError::unsupported("command", command.to_string()));
        }
        let input = &request.input;
        self.items.borrow_mut().push(Row::from([
            ("ref".into(), input["ref"].clone()),
            ("owner".into(), input["owner"].clone()),
            ("delegate".into(), input["delegate"].clone()),
        ]));
        self.serial.set(self.serial.get() + 1);
        Ok(SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            OutcomeName::new("created").unwrap(),
        ))
        .emitting(
            ObservedEvent::new("demo.vault.ItemCreated".parse().unwrap())
                .with("ref", input["ref"].clone()),
        )
        .with_consistency(ConsistencyToken::new(format!("seq:{}", self.serial.get())).unwrap()))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let param = |name: &str| request.params.get(name);
        let rows = self
            .items
            .borrow()
            .iter()
            .filter(|row| match request.view.to_string().as_str() {
                "demo.vault.BySameName" => param("owner") == row.get("owner"),
                "demo.vault.ByOtherName" => {
                    self.mutant == Mutant::IgnoreWho || param("who") == row.get("owner")
                }
                "demo.vault.ByTenant" => {
                    self.mutant == Mutant::IgnoreTenant
                        || param("tenant") == Some(member(&row["ref"], "tenant"))
                }
                "demo.vault.ByCrossed" if self.mutant == Mutant::CrossedByOwner => {
                    param("owner") == row.get("owner")
                }
                "demo.vault.ByCrossed" => param("owner") == row.get("delegate"),
                _ => false,
            })
            .cloned()
            .collect::<Vec<_>>();
        Ok(SemanticViewResult::of(rows))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "none"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("binding", "none"))
    }
}

fn verdict(mutant: Mutant) -> Status {
    let synthesis = synthesize(&ir(MODEL));
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap();
    let report = Runner::for_suite(&synthesis.suite)
        .run_admitted(&admitted, &Vault::new(mutant))
        .into_report();
    report
        .scenarios
        .into_iter()
        .find(|result| result.scenario.to_string() == CREATED)
        .map(|result| {
            assert!(
                mutant != Mutant::None || result.status == Status::Passed,
                "{result:#?}"
            );
            result.status
        })
        .expect("the creation scenario ran")
}

#[test]
fn ignored_param_mutant_fails() {
    assert_eq!(verdict(Mutant::None), Status::Passed);
    for mutant in [
        Mutant::IgnoreWho,
        Mutant::IgnoreTenant,
        Mutant::CrossedByOwner,
    ] {
        assert_eq!(verdict(mutant), Status::Failed, "{mutant:?} survived");
    }
    // The Go runtime reads the same suite to the same verdicts.
    let synthesis = synthesize(&ir(MODEL));
    for mutant in [Mutant::None, Mutant::IgnoreTenant] {
        support_go::assert_parity(
            &format!("view-param-{mutant:?}"),
            &synthesis.suite,
            Vault::new(mutant),
        );
    }
}

/// The fixture with a command that updates an existing item: its scenario holds the item's
/// identity only as the instance an arrangement captured.
fn with_touch() -> String {
    MODEL
        .replace(
            "    may: [demo.vault.CreateItem]",
            "    may: [demo.vault.CreateItem, demo.vault.Touch]",
        )
        .replace(
            "events:\n",
            "  - name: demo.vault.Touch
    input:
      - {name: ref, type: demo.vault.Ref}
      - {name: owner, type: String}
    outcomes:
      - name: touched
        updates: demo.vault.Item
        instance: ref
        sets: {owner: input.owner}
        emits: [demo.vault.Touched]
        payload:
          demo.vault.Touched: {owner: input.owner}
events:
  - name: demo.vault.Touched
    fields:
      - {name: owner, type: String}
",
        )
}

#[test]
fn instance_struct_member_refused_by_name() {
    let synthesis = synthesize(&ir(&with_touch()));
    let refused: Vec<String> = refusals_naming(&synthesis, "demo.vault.ByTenant")
        .into_iter()
        .filter(|refusal| refusal.contains("demo.vault.Touch/"))
        .collect();
    assert_eq!(refused.len(), 1, "{:#?}", synthesis.refusals);
    assert!(
        refused[0].contains("`ref.tenant` is a member of `ref`")
            && refused[0].contains("no member projection"),
        "{}",
        refused[0]
    );
    // The creation, which sent the identity as a literal, still binds it.
    assert_eq!(
        refusals_naming(&synthesis, "demo.vault.ByTenant")
            .iter()
            .filter(|refusal| refusal.contains("demo.vault.CreateItem/"))
            .count(),
        0
    );
}
