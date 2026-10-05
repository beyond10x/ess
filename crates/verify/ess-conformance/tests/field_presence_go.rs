//! The generated Go runtime holds an event's payload to each field's presence policy (suite/24,
//! beyond10x/ess#139) and gives the reference verdicts (beyond10x/ess#188).
//!
//! The model is `tests/field_presence_synthesized.rs`'s. The target publishes `Placed` with each
//! absent optional value spelled as its policy says; each mutant spells one of them the other way,
//! which both runners must fail.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::refs::OutcomeRef;
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::target::*;
use ess_conformance::ConformanceSuite;
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = "format: ess/15
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
      - {name: partner_ref, type: Optional<String>, presence: null_when_absent}
      - {name: discount_code, type: Optional<String>, presence: omitted_when_absent}
      - {name: note, type: Optional<String>}
      - {name: receipt, type: demo.orders.Receipt}
      - {name: later, type: Optional<demo.orders.Receipt>}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: partner_ref, type: Optional<String>}
      - {name: discount_code, type: Optional<String>}
      - {name: note, type: Optional<String>}
      - {name: receipt, type: demo.orders.Receipt}
      - {name: later, type: Optional<demo.orders.Receipt>}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            partner_ref: input.partner_ref
            discount_code: input.discount_code
            note: input.note
            receipt: input.receipt
            later: input.later
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(MODEL));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    synthesis.suite
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    /// Every absent optional value spelled as its policy says.
    Correct,
    /// The two policies swapped: `partner_ref` left out, `discount_code` sent as `null`.
    Swapped,
    /// `receipt.code` left out where it is declared `null_when_absent`.
    OmitsReceiptCode,
}

/// A value copied from the input, or — where the input carries none — `null`, or nothing at all.
fn spell(input: &BTreeMap<String, Node>, field: &str, null: bool) -> Option<Node> {
    match input.get(field) {
        Some(Node::Null) | None => null.then_some(Node::Null),
        Some(value) => Some(value.clone()),
    }
}

struct Orders(Mode);

impl ConformanceTarget for Orders {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("presence-fixture", "1"))
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
        let input = &request.input;
        let swapped = self.0 == Mode::Swapped;
        let mut payload = BTreeMap::new();
        for (field, null) in [
            ("partner_ref", !swapped),
            ("discount_code", swapped),
            ("note", false),
        ] {
            if let Some(value) = spell(input, field, null) {
                payload.insert(field.to_owned(), value);
            }
        }
        let receipt = |value: Option<&Node>, required: bool| -> Option<Node> {
            match value {
                Some(Node::Map(receipt)) => Some(Node::Map(
                    spell(receipt, "code", self.0 != Mode::OmitsReceiptCode)
                        .map(|code| BTreeMap::from([("code".to_owned(), code)]))
                        .unwrap_or_default(),
                )),
                _ if required => Some(Node::Map(BTreeMap::new())),
                _ => None,
            }
        };
        if let Some(value) = receipt(input.get("receipt"), true) {
            payload.insert("receipt".to_owned(), value);
        }
        if let Some(value) = receipt(input.get("later"), false) {
            payload.insert("later".to_owned(), value);
        }
        let mut event = ObservedEvent::new("demo.orders.Placed".parse().unwrap());
        event.payload = payload;
        Ok(SemanticCommandResult::took(OutcomeRef::new(
            request.command.clone(),
            OutcomeName::new("placed").unwrap(),
        ))
        .emitting(event))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

#[test]
fn go_gives_the_reference_verdict_for_every_presence_spelling() {
    let suite = suite();
    for mode in [Mode::Correct, Mode::Swapped, Mode::OmitsReceiptCode] {
        let verdicts = support_go::assert_parity(
            &format!("presence-{mode:?}").to_lowercase(),
            &suite,
            Orders(mode),
        );
        assert_eq!(
            support_go::not_passed(&verdicts).is_empty(),
            mode == Mode::Correct,
            "{mode:?}: {verdicts:?}"
        );
    }
}

#[test]
fn go_gives_the_reference_verdict_on_the_coverage_input_25() {
    use ess_conformance::coverage::{Origins, Scope};
    let input =
        ess_conformance::coverage_build::build(&ir(MODEL), &[], Scope::System, Origins::Generated)
            .unwrap_or_else(|error| panic!("{error:?}"));
    assert_eq!(
        input
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/35"
    );
    for mode in [Mode::Correct, Mode::Swapped] {
        let label = format!("presence-coverage-{mode:?}").to_lowercase();
        let compared = support_go::compare_input(
            &label,
            &input,
            Orders(mode),
            &support_go::Options::default(),
        );
        let verdicts = support_go::assert_compared(&label, compared);
        assert_eq!(
            support_go::not_passed(&verdicts).is_empty(),
            mode == Mode::Correct,
            "{mode:?}: {verdicts:?}"
        );
    }
}

/// A response whose fields carry presence policies, mapped into an event (suite/24).
const RESPONSE: &str = "format: ess/15
system: demo
version: v1
domain: demo.api
events:
  - name: demo.api.Returned
    fields:
      - {name: reference, type: Optional<String>}
      - {name: receipt, type: String}
commands:
  - name: demo.api.Cancel
    response:
      - {name: reference, type: Optional<String>, presence: null_when_absent}
      - {name: code, type: Optional<String>, presence: omitted_when_absent}
    outcomes:
      - name: cancelled
        emits: [demo.api.Returned]
        payload:
          demo.api.Returned:
            reference: {response: reference}
            receipt: {generated: true}
";

/// How the response spells its two absent values.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Reply {
    /// `reference` sent as `null`, `code` left out.
    Correct,
    /// `reference` left out where it is declared `null_when_absent`.
    OmitsReference,
    /// `code` sent as `null` where it is declared `omitted_when_absent`.
    NullCode,
}

struct Api(Reply);

impl ConformanceTarget for Api {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "response-presence-fixture",
            "1",
        ))
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
        let mut response = BTreeMap::new();
        if self.0 != Reply::OmitsReference {
            response.insert("reference".to_owned(), Node::Null);
        }
        if self.0 == Reply::NullCode {
            response.insert("code".to_owned(), Node::Null);
        }
        let mut event = ObservedEvent::new("demo.api.Returned".parse().unwrap());
        event.payload = BTreeMap::from([
            ("reference".to_owned(), Node::Null),
            ("receipt".to_owned(), Node::Text("r-1".to_owned())),
        ]);
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            request.command.clone(),
            OutcomeName::new("cancelled").unwrap(),
        ))
        .emitting(event);
        result.response = Some(response);
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("views", "the model declares none"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

#[test]
fn go_gives_the_reference_verdict_for_every_response_presence_spelling() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(RESPONSE));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    for reply in [Reply::Correct, Reply::OmitsReference, Reply::NullCode] {
        let verdicts = support_go::assert_parity(
            &format!("response-presence-{reply:?}").to_lowercase(),
            &suite,
            Api(reply),
        );
        assert_eq!(
            support_go::not_passed(&verdicts).is_empty(),
            reply == Reply::Correct,
            "{reply:?}: {verdicts:?}"
        );
    }
}
