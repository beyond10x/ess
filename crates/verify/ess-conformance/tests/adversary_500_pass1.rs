//! Adversary pass 1 on beyond10x/ess#500 (`undeclared_fields: ignored` in the observers).
//!
//! Each test is one finding: a place where a specification opens a response object or a struct,
//! a target answers within that specification (declared fields present and typed, an extension
//! member beside them), and a conformance observation still reads the extension member.

use std::cell::Cell;
use std::collections::BTreeMap;

use ess_compiler::refs::OutcomeRef;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{authored, AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn object(json: &str) -> BTreeMap<String, Node> {
    serde_json::from_str(json).unwrap_or_else(|error| panic!("{error}: {json}"))
}

/// A target that answers every command with `response`, takes the outcome `outcome` (and
/// `retry` from the second call on), and publishes `events`.
struct Answering {
    outcome: &'static str,
    retry: &'static str,
    response: &'static str,
    events: Vec<(&'static str, &'static str)>,
    rows: &'static str,
    calls: Cell<usize>,
}

impl ConformanceTarget for Answering {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-500", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.calls.set(0);
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let retry = self.calls.get() > 0;
        self.calls.set(self.calls.get() + 1);
        let outcome = if retry { self.retry } else { self.outcome };
        let mut result =
            SemanticCommandResult::took(OutcomeRef::new(request.command, outcome.parse().unwrap()));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("actual-write").unwrap());
        if !retry {
            for (event, payload) in &self.events {
                let mut observed = ObservedEvent::new(event.parse().unwrap());
                observed.payload = object(payload);
                result = result.emitting(observed);
            }
        }
        result.response = Some(object(self.response));
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let rows: Vec<BTreeMap<String, Node>> = serde_json::from_str(self.rows).unwrap();
        Ok(SemanticViewResult { rows, total: None })
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
        Err(TargetError::unsupported("redelivery", "none"))
    }
    fn observe_invocations(
        &self,
        _: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Err(TargetError::unsupported("invocations", "none"))
    }
}

/// The status and checks of scenario `id` after running `suite` against `target`.
fn run_one(suite: &ConformanceSuite, id: &str, target: &Answering) -> (Status, String) {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report();
    let result = report
        .scenarios
        .into_iter()
        .find(|result| result.scenario.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"));
    let status = result.status;
    (status, format!("{result:#?}"))
}

// ---- F1: a retained-result replay of an opened response stays closed --------------------------

/// `tests/fixtures/retained-replay.yaml` under `ess/24`, with the command's response opened.
const OPENED_REPLAY: &str = r"format: ess/24
system: retained
version: v1
domain: retained.core
entities:
  - name: retained.core.Record
    identity: {name: record_id, type: Uuid}
    fields:
      - {name: value, type: String}
      - {name: stamp, type: Timestamp}
    lifecycle:
      initial: Committed
      states: [Committed]
      terminal: [Committed]
commands:
  - name: retained.core.Seed
    undeclared_fields: ignored
    input: [{name: document, type: String}]
    response:
      - {name: revision_id, type: Uuid}
      - {name: stamp, type: Timestamp}
      - {name: number, type: Integer}
      - {name: optional, type: Optional<String>}
      - {name: values, type: List<Integer>}
    outcomes:
      - name: seeded
        creates: retained.core.Record
        instance: record_id
        sets: {value: input.document}
        emits: [retained.core.Seeded]
        payload:
          retained.core.Seeded:
            record_id: {generated: true}
      - name: replayed
        external: retained logical input and trusted context match
        replays: seeded
events:
  - name: retained.core.Seeded
    fields: [{name: record_id, type: Uuid}]
views:
  - name: retained.core.Records
    source: retained.core.Record
    consistency: read_your_writes
    fields:
      - {name: record_id, type: Uuid}
      - {name: value, type: String}
      - {name: stamp, type: Timestamp}
      - {name: state, type: retained.core.Record.State}
";

const REPLAYED: &str = "retained.core.Seed/outcome/replayed";

/// The command declares `undeclared_fields: ignored`. A target that returns every declared field
/// plus one extension member, the same bytes on the original call and on the retry, conforms. The
/// replay scenario must then either pass or be refused at synthesis by name, as the one-time
/// response is (`one_time_response.rs`); it must not be emitted and fail an honest target.
#[test]
fn an_opened_response_with_a_retained_replay_fails_an_honest_target() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(OPENED_REPLAY));
    let refused_by_name = synthesis.refusals.iter().any(|refusal| {
        let text = refusal.to_string();
        text.contains(REPLAYED) && text.contains("undeclared_fields")
    });
    let Some(_) = synthesis
        .suite
        .scenarios
        .keys()
        .find(|id| id.to_string() == REPLAYED)
    else {
        assert!(
            refused_by_name,
            "the replay scenario is neither emitted nor refused by name: {:#?}",
            synthesis
                .refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        return;
    };
    let target = Answering {
        outcome: "seeded",
        retry: "replayed",
        response: r#"{"revision_id":"00000000-0000-4000-8000-000000000037","stamp":"2026-09-22T01:02:03Z","number":7,"values":[1,2],"extension":"x"}"#,
        events: vec![(
            "retained.core.Seeded",
            r#"{"record_id":"00000000-0000-4000-8000-000000000037"}"#,
        )],
        rows: r#"[{"record_id":"00000000-0000-4000-8000-000000000037","value":"actual document","stamp":"2026-09-22T01:02:03Z","state":"Committed"}]"#,
        calls: Cell::new(0),
    };
    let (status, detail) = run_one(&synthesis.suite, REPLAYED, &target);
    assert_eq!(status, Status::Passed, "{detail}");
}

// ---- F2, F3: an authored response literal over an opened struct -------------------------------

const OPENED_ITEM: &str = r"format: ess/24
system: library
version: v1
domain: library.api
types:
  - name: library.api.Item
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: label, type: String}
commands:
  - name: library.api.Read
    response:
      - {name: item, type: library.api.Item}
    outcomes:
      - name: returned
        returns: true
";

fn authored_suite(scenario: &str) -> Result<ConformanceSuite, String> {
    let model = ir(OPENED_ITEM);
    let authored = authored::compile(&model, &[authored::Source::new("read.yaml", scenario)]);
    if !authored.refusals.is_empty() {
        return Err(format!("{:?}", authored.refusals));
    }
    let mut suite = ess_conformance::synthesize(&model).suite;
    suite.scenarios = authored.scenarios;
    suite.select_fresh_format();
    Ok(suite)
}

fn literal_scenario(item: &str) -> String {
    format!(
        "type: ess-scenario/4
domain: library.api
scenario: read-item
summary: The returned item carries the declared label.
timeline:
  - at: 2026-09-28T00:00:00Z
    command: library.api.Read
    outcome: returned
    response: {{item: {item}}}
"
    )
}

/// `library.api.Item` ignores undeclared fields. The author asserts the declared `label`; a target
/// whose item carries `label: nested` plus an extension member conforms, because an observer
/// "reads nothing of" an undeclared member. The literal comparison is complete equality, so the
/// extension member decides the verdict.
#[test]
fn an_authored_literal_over_an_opened_struct_fails_a_target_adding_an_extension_member() {
    let suite = authored_suite(&literal_scenario("{label: nested}")).expect("authored");
    let id = suite
        .scenarios
        .keys()
        .next()
        .expect("one scenario")
        .to_string();
    let target = Answering {
        outcome: "returned",
        retry: "returned",
        response: r#"{"item":{"label":"nested","extension":"x"}}"#,
        events: Vec::new(),
        rows: "[]",
        calls: Cell::new(0),
    };
    let (status, detail) = run_one(&suite, &id, &target);
    assert_eq!(status, Status::Passed, "{detail}");
}

/// The converse: an authored literal that names a key `library.api.Item` does not declare asserts
/// an undeclared member's value, which the story says is never readable. It is admitted, and the
/// suite then fails a target that leaves the extension out.
#[test]
fn an_authored_literal_may_assert_an_undeclared_member_of_an_opened_struct() {
    let admitted = authored_suite(&literal_scenario("{label: nested, extension: x}"));
    assert!(
        admitted.is_err(),
        "a literal naming an undeclared member was admitted as authority: {}",
        admitted
            .map(|suite| suite.to_canonical_json().unwrap())
            .unwrap_or_default()
    );
}

// ---- F4: a response-mapped event field of an opened struct type -------------------------------

const OPENED_VENDOR_MAPPED: &str = r"format: ess/24
system: catalog
version: v1
domain: catalog.orders
types:
  - name: catalog.orders.Vendor
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: vendor_id, type: String}
events:
  - name: catalog.orders.OrderPlaced
    fields:
      - {name: vendor, type: catalog.orders.Vendor}
actors:
  - name: catalog.orders.Buyer
    may:
      - catalog.orders.PlaceOrder
commands:
  - name: catalog.orders.PlaceOrder
    input:
      - {name: item, type: String}
    response:
      - {name: vendor, type: catalog.orders.Vendor}
    outcomes:
      - name: placed
        returns: true
        emits: [catalog.orders.OrderPlaced]
        payload:
          catalog.orders.OrderPlaced:
            vendor: {response: vendor}
";

/// `catalog.orders.Vendor` ignores undeclared fields. The target returns a vendor with an
/// extension member and publishes the event's vendor with its declared field only: every declared
/// value of the response is in the event. The response-payload observation compares the whole
/// returned value, extension member included, with the event's, so the extension member decides.
#[test]
fn a_mapped_event_field_must_repeat_the_extension_member_of_an_opened_struct() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(OPENED_VENDOR_MAPPED));
    let id = "catalog.orders.PlaceOrder/outcome/placed";
    assert!(
        synthesis
            .suite
            .scenarios
            .keys()
            .any(|scenario| scenario.to_string() == id),
        "{:#?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    let target = Answering {
        outcome: "placed",
        retry: "placed",
        response: r#"{"vendor":{"vendor_id":"v-1","region":"eu"}}"#,
        events: vec![(
            "catalog.orders.OrderPlaced",
            r#"{"vendor":{"vendor_id":"v-1"}}"#,
        )],
        rows: "[]",
        calls: Cell::new(0),
    };
    let (status, detail) = run_one(&synthesis.suite, id, &target);
    assert_eq!(status, Status::Passed, "{detail}");
}
