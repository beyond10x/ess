//! Adversary pass 1 on the wave-2 `types` unit (beyond10x/ess#139, #138, #146).
//!
//! Each case runs a synthesized suite through the real `Runner` against a hand-written target, so
//! the verdict is the suite's own, not a reading of its leaves.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::Status, target::*, AdmittedSuite, ConformanceSuite, Runner, ScenarioStep,
};
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

/// What an implementation does with an absent value of each policy field.
#[derive(Clone, Copy, PartialEq)]
enum Spelling {
    /// `partner_ref` sent as `null`, `discount_code` left out: what the model declares.
    Declared,
    /// The two swapped: `partner_ref` left out, `discount_code` sent as `null`.
    Swapped,
}

/// Copies each input into the event payload, spelling an absent value per `Spelling`.
struct Orders(Spelling);

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
        let outcome =
            ess_compiler::refs::OutcomeRef::new(request.command, "placed".parse().unwrap());
        let mut result = SemanticCommandResult::took(outcome);
        let mut payload = BTreeMap::new();
        for (field, null_when_absent) in [
            ("partner_ref", self.0 == Spelling::Declared),
            ("discount_code", self.0 == Spelling::Swapped),
        ] {
            match request.input.get(field) {
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
        let mut event = ObservedEvent::new("demo.orders.Placed".parse().unwrap());
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

fn statuses<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> Vec<Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .scenarios
        .iter()
        .map(|scenario| scenario.status)
        .collect()
}

/// The issue's two fields, copied from inputs, with no guard anywhere: the ordinary shape of a
/// retrofitted command.
const PRESENCE: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
events:
  - name: demo.orders.Placed
    fields:
      - {name: partner_ref, type: Optional<String>, presence: null_when_absent}
      - {name: discount_code, type: Optional<String>, presence: omitted_when_absent}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: partner_ref, type: Optional<String>}
      - {name: discount_code, type: Optional<String>}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            partner_ref: input.partner_ref
            discount_code: input.discount_code
";

#[test]
fn adversary_139_the_declared_spelling_passes_the_synthesized_suite() {
    let suite = suite(PRESENCE);
    let statuses = statuses(&suite, &Orders(Spelling::Declared));
    assert!(!statuses.is_empty());
    assert!(
        statuses.iter().all(|status| *status == Status::Passed),
        "{statuses:?}"
    );
}

/// The unit's acceptance: "a suite that fails an implementation swapping the two policies". A
/// policy is only observable when the value is absent, so some scenario has to leave the input out.
#[test]
fn adversary_139_a_synthesized_suite_fails_an_implementation_swapping_the_policies() {
    let suite = suite(PRESENCE);
    let omitting = suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { input, .. }
                if !input.contains_key("partner_ref") || !input.contains_key("discount_code"))
        })
        .count();
    let statuses = statuses(&suite, &Orders(Spelling::Swapped));
    assert!(
        statuses.contains(&Status::Failed),
        "every scenario passed against the swapped implementation ({statuses:?}); \
         invocations that leave a policy field's input out: {omitting}"
    );
}

/// A suite synthesis writes as ordinary suite/24 is one the crate's own execution reader admits,
/// in memory and from its canonical bytes; otherwise no runner can execute a presence policy.
#[test]
fn adversary_139_the_synthesized_suite_24_is_admitted_for_execution() {
    let suite = suite(PRESENCE);
    assert_eq!(suite.provenance.suite_version.major(), 34);
    AdmittedSuite::from_suite(&suite).expect("suite/24 is admitted from memory");
    AdmittedSuite::from_json(&suite.to_canonical_json().unwrap())
        .expect("suite/24 is admitted from its canonical bytes");
}

/// The coverage counterpart, suite/25, built by the crate's own coverage writer.
#[test]
fn adversary_139_the_coverage_suite_25_is_built_and_admitted() {
    let input = ess_conformance::coverage_build::build(
        &ir(PRESENCE),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("the coverage writer refuses its own suite/25: {error:?}"));
    assert_eq!(
        input.selected().suite().provenance.suite_version.major(),
        35
    );
}

/// The precondition of the case above, independent of whether any reader admits the suite: a
/// policy is observable only on an absent value, so some invocation must leave each policy
/// field's input out. Witness rule 1 fills every optional unless a guard reads `defined(x)`.
#[test]
fn adversary_139_some_invocation_leaves_each_policy_field_absent() {
    let suite = suite(PRESENCE);
    for field in ["partner_ref", "discount_code"] {
        let invocations: Vec<_> = suite
            .scenarios
            .values()
            .flat_map(|scenario| &scenario.steps)
            .filter_map(|step| match step {
                ScenarioStep::ExecuteCommand { input, .. } => Some(input),
                _ => None,
            })
            .collect();
        assert!(!invocations.is_empty());
        assert!(
            invocations.iter().any(|input| !input.contains_key(field)),
            "every one of {} invocations sends `{field}`, so no scenario observes how its absence \
             is spelled and the declared policy is never exercised",
            invocations.len()
        );
    }
}

/// Returns a response and an event, both without `item`: the key left out rather than sent as
/// `null`.
struct Receipts;

impl ConformanceTarget for Receipts {
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
        let outcome =
            ess_compiler::refs::OutcomeRef::new(request.command, "cancelled".parse().unwrap());
        let mut result = SemanticCommandResult::took(outcome);
        result.response = Some(BTreeMap::new());
        let mut event = ObservedEvent::new("demo.api.Returned".parse().unwrap());
        event
            .payload
            .insert("receipt".into(), Node::Text("generated-37".into()));
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

/// `tests/fixtures/response-payload.yaml` at ess/15 with the response field `Optional` and declared
/// `null_when_absent`: #139's own case is "two Optional fields of one response".
fn response_model() -> String {
    include_str!("fixtures/response-payload.yaml")
        .replace("format: ess/4", "format: ess/15")
        .replace(
            "    response:\n      - {name: item, type: demo.api.Item}",
            "    response:\n      - {name: item, type: Optional<demo.api.Item>, presence: null_when_absent}",
        )
        .replace(
            "      - {name: item, type: demo.api.Item}\n      - {name: receipt",
            "      - {name: item, type: Optional<demo.api.Item>}\n      - {name: receipt",
        )
}

#[test]
fn adversary_139_a_response_field_declared_null_when_absent_fails_an_omitted_key() {
    let model = response_model();
    assert!(model.contains("presence: null_when_absent"), "{model}");
    let suite = suite(&model);
    let statuses = statuses(&suite, &Receipts);
    assert!(
        statuses.contains(&Status::Failed),
        "the response left `item` out, which `null_when_absent` forbids, and every scenario \
         passed: {statuses:?}"
    );
}

#[test]
fn adversary_139_a_suite_that_writes_a_presence_key_is_at_least_suite_24() {
    let suite = suite(&response_model());
    let json = suite.to_canonical_json().unwrap();
    assert!(
        !json.contains("\"presence\"") || suite.provenance.suite_version.major() >= 24,
        "the suite writes `presence` (in a response observation's fields) as {}, which the Go \
         and TypeScript runtimes read and drop the key from",
        suite.provenance.suite_version
    );
}

/// The payload an `Echo` publishes for an input, or `None` for a request it rejects.
type Mapping = fn(&BTreeMap<String, Node>) -> Option<BTreeMap<String, Node>>;

/// A target that answers every command with one outcome and one event, whose payload `map` derives
/// from the input; `None` is a request the implementation rejects (its own grammar, say a 400).
struct Echo {
    outcome: &'static str,
    event: &'static str,
    map: Mapping,
}

impl ConformanceTarget for Echo {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("echo-fixture", "1"))
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
        let Some(payload) = (self.map)(&request.input) else {
            return Ok(SemanticCommandResult::undeclared());
        };
        let outcome =
            ess_compiler::refs::OutcomeRef::new(request.command, self.outcome.parse().unwrap());
        let mut result = SemanticCommandResult::took(outcome);
        let mut event = ObservedEvent::new(self.event.parse().unwrap());
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

/// #138's repro, a newtype of `Json`, delivered unchanged.
const JSON: &str = "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - {name: demo.msgs.Body, kind: newtype, of: Json}
events:
  - name: demo.msgs.Sent
    fields:
      - {name: body, type: demo.msgs.Body}
actors:
  - {name: demo.msgs.Sender, may: [demo.msgs.Send]}
commands:
  - name: demo.msgs.Send
    input:
      - {name: body, type: demo.msgs.Body}
    outcomes:
      - name: sent
        emits: [demo.msgs.Sent]
        payload:
          demo.msgs.Sent: {body: input.body}
";

#[test]
fn adversary_138_a_json_body_delivered_unchanged_passes_and_a_stringified_one_fails() {
    let suite = suite(JSON);
    let unchanged = Echo {
        outcome: "sent",
        event: "demo.msgs.Sent",
        map: |input| Some(input.clone()),
    };
    let statuses_unchanged = statuses(&suite, &unchanged);
    assert!(
        statuses_unchanged.iter().all(|s| *s == Status::Passed),
        "{statuses_unchanged:?}"
    );
    // What #138 says the String workaround misstated: the document sent as a string.
    let stringified = Echo {
        outcome: "sent",
        event: "demo.msgs.Sent",
        map: |input| {
            let body = serde_json::to_string(input.get("body")?).ok()?;
            Some(BTreeMap::from([("body".to_owned(), Node::Text(body))]))
        },
    };
    assert!(statuses(&suite, &stringified).contains(&Status::Failed));
}

/// #146's repro: a channel that starts with `/` and may not contain `*`, as the alphabet of every
/// printable ASCII character but `*`, sent to an implementation that rejects anything else.
fn channel_model(extra: &str, guard: &str) -> String {
    let alphabet: String = (' '..='~')
        .filter(|c| *c != '*' && *c != '"' && *c != '\\')
        .collect();
    format!(
        "format: ess/15
system: demo
version: v1
domain: demo.msgs
types:
  - name: demo.msgs.Channel
    kind: newtype
    of: String
    prefix: \"/\"
    alphabet: \"{alphabet}\"
{extra}errors:
  - name: demo.msgs.Refused
    fields: []
events:
  - name: demo.msgs.Posted
    fields:
      - {{name: channel, type: demo.msgs.Channel}}
actors:
  - {{name: demo.msgs.Poster, may: [demo.msgs.Post]}}
commands:
  - name: demo.msgs.Post
    input:
      - {{name: channel, type: demo.msgs.Channel}}
    outcomes:
{guard}      - name: posted
        emits: [demo.msgs.Posted]
        payload:
          demo.msgs.Posted: {{channel: input.channel}}
"
    )
}

fn channel_grammar(input: &BTreeMap<String, Node>) -> Option<BTreeMap<String, Node>> {
    let Some(Node::Text(channel)) = input.get("channel") else {
        return None;
    };
    (channel.starts_with('/') && !channel.contains('*')).then(|| input.clone())
}

#[test]
fn adversary_146_the_issue_repro_passes_against_an_implementation_enforcing_its_grammar() {
    let suite = suite(&channel_model("", ""));
    let target = Echo {
        outcome: "posted",
        event: "demo.msgs.Posted",
        map: channel_grammar,
    };
    let statuses = statuses(&suite, &target);
    assert!(!statuses.is_empty());
    assert!(
        statuses.iter().all(|s| *s == Status::Passed),
        "{statuses:?}"
    );
}

/// A prefix longer than the longest value a length invariant admits leaves the type with no
/// value. Either the model is refused or synthesis says it cannot witness it; a suite that goes on
/// to send a value is sending one the type does not have.
#[test]
fn adversary_146_a_prefix_longer_than_the_length_bound_is_not_silently_witnessed() {
    let text = "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.OrderRef
    kind: newtype
    of: String
    prefix: \"ORD-\"
    invariants: [value.count <= 3]
events:
  - name: demo.orders.Placed
    fields:
      - {name: order_ref, type: demo.orders.OrderRef}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: order_ref, type: demo.orders.OrderRef}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed: {order_ref: input.order_ref}
";
    let raw = RawSpecFile::parse(text).unwrap();
    let Ok(spec) = Specification::assemble([(Source::new("orders.yaml"), raw)]) else {
        return;
    };
    let synthesis =
        ess_conformance::synthesize::synthesize(&compile(&spec, &SourceMap::new()).unwrap());
    let sent: Vec<&Node> = synthesis
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => match input.get("order_ref") {
                Some(ess_conformance::ScenarioValue::Literal { value }) => Some(value),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert!(
        !synthesis.refusals.is_empty(),
        "the model validated, synthesis refused nothing, and the suite sends {sent:?}"
    );
}
