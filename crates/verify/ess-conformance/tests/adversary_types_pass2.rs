//! Adversary pass 2 on the wave-2 `types` unit (beyond10x/ess#139), after correction round 1.
//!
//! Each case runs a synthesized suite through the real `Runner` against a hand-written target that
//! spells an absent value against the declared policy. The acceptance statement is "a suite that
//! fails an implementation swapping the two policies"; these cases move the policy field to the
//! positions correction round 1 did not reach.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{report::Status, target::*, AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite(text: &str) -> ConformanceSuite {
    let result = ess_conformance::synthesize::synthesize(&ir(text));
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    result.suite
}

/// Answers one command with an outcome, one event and its payload, all derived from the input.
type Answer = fn(&BTreeMap<String, Node>) -> (&'static str, &'static str, BTreeMap<String, Node>);

struct Fixture(Answer);

impl ConformanceTarget for Fixture {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("presence-pass2-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let (outcome, event_name, payload) = (self.0)(&request.input);
        let outcome =
            ess_compiler::refs::OutcomeRef::new(request.command, outcome.parse().unwrap());
        let mut result = SemanticCommandResult::took(outcome);
        let mut event = ObservedEvent::new(event_name.parse().unwrap());
        event.payload = payload;
        result.direct_events.push(event);
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", "unused"))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "unused"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
    fn observe_invocations(
        &self,
        _: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Err(TargetError::unsupported("invocations", "unused"))
    }
}

fn statuses(suite: &ConformanceSuite, target: &Fixture) -> Vec<Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .scenarios
        .iter()
        .map(|scenario| scenario.status)
        .collect()
}

/// `value` with every `null` and every absent `field` spelled the swapped way: a
/// `null_when_absent` field left out.
fn drop_nulls(value: &Node) -> Node {
    match value {
        Node::Map(map) => Node::Map(
            map.iter()
                .filter(|(_, v)| !matches!(v, Node::Null))
                .map(|(k, v)| (k.clone(), drop_nulls(v)))
                .collect(),
        ),
        other => other.clone(),
    }
}

/// A policy on a member of a struct that is always present: `receipt.code`. The unit's own test
/// (`field_presence_synthesized.rs`) shows the leaf carries `null_when_absent`; this asks whether
/// any scenario ever makes the member absent, so an implementation that leaves it out fails.
const STRUCT_MEMBER: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Receipt
    kind: struct
    fields:
      - {name: code, type: Optional<String>, presence: null_when_absent}
events:
  - name: demo.orders.Placed
    fields:
      - {name: receipt, type: demo.orders.Receipt}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: receipt, type: demo.orders.Receipt}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            receipt: input.receipt
";

#[test]
fn adversary2_139_a_struct_member_policy_fails_an_implementation_that_omits_the_key() {
    let suite = suite(STRUCT_MEMBER);
    // Declared: an absent `code` sent as null.
    let declared = Fixture(|input| {
        let mut receipt = match input.get("receipt") {
            Some(Node::Map(map)) => map.clone(),
            _ => BTreeMap::new(),
        };
        receipt.entry("code".to_owned()).or_insert(Node::Null);
        (
            "placed",
            "demo.orders.Placed",
            BTreeMap::from([("receipt".to_owned(), Node::Map(receipt))]),
        )
    });
    let declared_statuses = statuses(&suite, &declared);
    assert!(
        declared_statuses.iter().all(|s| *s == Status::Passed),
        "the declared spelling fails: {declared_statuses:?}"
    );
    // Swapped: an absent `code` left out, which `null_when_absent` forbids.
    let swapped = Fixture(|input| {
        let receipt = input
            .get("receipt")
            .map_or(Node::Map(BTreeMap::new()), drop_nulls);
        (
            "placed",
            "demo.orders.Placed",
            BTreeMap::from([("receipt".to_owned(), receipt)]),
        )
    });
    let swapped_statuses = statuses(&suite, &swapped);
    let sends: Vec<String> = suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .filter_map(|step| match step {
            ess_conformance::ScenarioStep::ExecuteCommand { input, .. } => {
                Some(format!("{:?}", input.get("receipt")))
            }
            _ => None,
        })
        .collect();
    assert!(
        swapped_statuses.contains(&Status::Failed),
        "`receipt.code` is declared null_when_absent and every scenario passed an implementation \
         that leaves it out ({swapped_statuses:?}); the receipts the suite sends: {sends:?}"
    );
}

/// The policy event is published only by a branch the base witness does not take: `express` is a
/// `Boolean` whose base is `true`, so the base candidate goes to `rushed`, and `placed` is reached
/// only by a candidate built with every optional filled.
const GUARDED: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
events:
  - name: demo.orders.Rushed
    fields: []
  - name: demo.orders.Placed
    fields:
      - {name: partner_ref, type: Optional<String>, presence: null_when_absent}
      - {name: discount_code, type: Optional<String>, presence: omitted_when_absent}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: express, type: Boolean}
      - {name: partner_ref, type: Optional<String>}
      - {name: discount_code, type: Optional<String>}
    outcomes:
      - name: rushed
        when: express
        emits: [demo.orders.Rushed]
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            partner_ref: input.partner_ref
            discount_code: input.discount_code
";

fn guarded(
    input: &BTreeMap<String, Node>,
    swapped: bool,
) -> (&'static str, &'static str, BTreeMap<String, Node>) {
    if matches!(input.get("express"), Some(Node::Bool(true))) {
        return ("rushed", "demo.orders.Rushed", BTreeMap::new());
    }
    let mut payload = BTreeMap::new();
    for (field, null_when_absent) in [("partner_ref", !swapped), ("discount_code", swapped)] {
        match input.get(field) {
            Some(Node::Null) | None => {
                if null_when_absent {
                    payload.insert(field.to_owned(), Node::Null);
                }
            }
            Some(value) => {
                payload.insert(field.to_owned(), value.clone());
            }
        }
    }
    ("placed", "demo.orders.Placed", payload)
}

#[test]
fn adversary2_139_a_policy_event_on_a_guarded_branch_fails_a_swapped_implementation() {
    let suite = suite(GUARDED);
    let declared_statuses = statuses(&suite, &Fixture(|input| guarded(input, false)));
    assert!(
        declared_statuses.iter().all(|s| *s == Status::Passed),
        "the declared spelling fails: {declared_statuses:?}"
    );
    let swapped_statuses = statuses(&suite, &Fixture(|input| guarded(input, true)));
    assert!(
        swapped_statuses.contains(&Status::Failed),
        "the `placed` branch publishes both policy fields and every scenario passed an \
         implementation swapping them ({swapped_statuses:?})"
    );
}

/// Presence declared only where no leaf carries it — a command input, and a member of a struct
/// under an `Optional` event field — writes no `presence` key, and so needs no suite/24 or /25.
const UNCARRIED: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Receipt
    kind: struct
    fields:
      - {name: code, type: Optional<String>, presence: omitted_when_absent}
events:
  - name: demo.orders.Placed
    fields:
      - {name: later, type: Optional<demo.orders.Receipt>}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: note, type: Optional<String>, presence: null_when_absent}
      - {name: later, type: Optional<demo.orders.Receipt>}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            later: input.later
";

#[test]
fn adversary2_139_a_suite_with_no_carried_policy_stays_below_suite_24() {
    let ordinary = suite(UNCARRIED);
    let json = ordinary.to_canonical_json().unwrap();
    assert!(!json.contains("\"presence\""), "{json}");
    assert!(
        ordinary.provenance.suite_version.major() < 24,
        "{}",
        ordinary.provenance.suite_version
    );
    let coverage = ess_conformance::coverage_build::build(
        &ir(UNCARRIED),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error:?}"));
    let suite = coverage.selected().suite();
    assert!(!suite.to_canonical_json().unwrap().contains("\"presence\""));
    assert!(
        suite.provenance.suite_version.major() < 24,
        "{}",
        suite.provenance.suite_version
    );
}

/// Both policies in one event, each fed by something that is never absent — a literal-free
/// `{generated: true}` and an input with an `else:` — beside one fed by a plain input. Only the
/// plain input is left out of the base; the suite still passes the declared spelling.
const MIXED: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
events:
  - name: demo.orders.Placed
    fields:
      - {name: receipt, type: Optional<String>, presence: null_when_absent}
      - {name: minted, type: Optional<String>, presence: omitted_when_absent}
      - {name: partner_ref, type: Optional<String>, presence: omitted_when_absent}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: minted, type: Optional<String>}
      - {name: partner_ref, type: Optional<String>}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            receipt: {generated: true}
            minted: {input: minted, else: {generated: true}}
            partner_ref: input.partner_ref
";

#[test]
fn adversary2_139_generated_and_defaulted_policy_fields_beside_an_input_pass_the_declared_spelling()
{
    let suite = suite(MIXED);
    assert_eq!(suite.provenance.suite_version.major(), 24);
    let declared = Fixture(|input| {
        let mut payload = BTreeMap::from([
            ("receipt".to_owned(), Node::Text("generated-1".into())),
            (
                "minted".to_owned(),
                input
                    .get("minted")
                    .cloned()
                    .unwrap_or(Node::Text("generated-2".into())),
            ),
        ]);
        if let Some(value) = input.get("partner_ref") {
            payload.insert("partner_ref".to_owned(), value.clone());
        }
        ("placed", "demo.orders.Placed", payload)
    });
    let statuses = statuses(&suite, &declared);
    assert!(
        statuses.iter().all(|s| *s == Status::Passed),
        "{statuses:?}"
    );
    let omitted = suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .any(|step| {
            matches!(step, ess_conformance::ScenarioStep::ExecuteCommand { input, .. }
                if !input.contains_key("partner_ref") && input.contains_key("minted"))
        });
    assert!(
        omitted,
        "the base leaves out the plain input only, never the defaulted one"
    );
}
