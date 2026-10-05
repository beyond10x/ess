//! Adversary pass 1 against the conditioned-binding unit (ess/22, beyond10x/ess#268 and
//! beyond10x/ess#194): the controls `docs/design/conditional-binding-failure-policies.md` names,
//! driven at the edges the unit's own tests do not reach.
use std::cell::Cell;
use std::collections::BTreeMap;

use ess_compiler::refs::{BindingRef, CommandRef};
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::report::Status;
use ess_conformance::{interpret::Interpreted, target::*};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::{ids::CorrelationId, node::Node, time::Timestamp};

const MODEL: &str = include_str!("fixtures/binding-condition.yaml");
const WHERE: &str = "      where: [defined(event.order), event.kind == ship]\n";
const ORDER_ID: &str = "      order_id: event.order.id\n";
const FROM_MESSAGE: &str = "      order_id: event.message_id\n";

const FALSE: &str = "received/binding/condition-false";
const ABSENT: &str = "received/binding/condition-absent";

fn admitted(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| format!("{error:?}"))?;
    Specification::assemble([(Source::new("messages.yaml"), raw)])
        .map_err(|errors| format!("{errors}"))
}

fn ir_of(text: &str) -> EssIr {
    let spec = admitted(text).unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn context() -> ScenarioContext {
    ScenarioContext::new(
        "demo.messages/authored/condition".parse().unwrap(),
        CorrelationId::new("condition").unwrap(),
    )
}

fn text(value: &str) -> Node {
    Node::Text(value.into())
}

fn observed(target: &Interpreted, binding: &str, command: &str) -> Vec<ObservedInvocation> {
    target
        .observe_invocations(InvocationObservationRequest {
            binding: BindingRef::new(ess_domain::binding::BindingName::new(binding).unwrap()),
            command: CommandRef::new(command.parse().unwrap()),
            correlation: context().correlation,
            deadline: Deadline::at(Timestamp::from_epoch_millis(0)),
        })
        .unwrap()
}

/// Publishes one `MessageReceived` on a fresh interpreted target.
fn receive(
    model: &str,
    kind: &str,
    order: Option<Node>,
) -> (Result<SemanticCommandResult, TargetError>, Interpreted) {
    let target = Interpreted::for_model(ir_of(model));
    target.begin_scenario(&context()).unwrap();
    let mut input = BTreeMap::from([
        ("message_id".to_owned(), text("m-1")),
        ("kind".to_owned(), text(kind)),
    ]);
    if let Some(order) = order {
        input.insert("order".into(), order);
    }
    let result = target.execute_command(SemanticCommandRequest {
        command: CommandRef::new("demo.messages.ReceiveMessage".parse().unwrap()),
        actor: None,
        caller: None,
        input,
        correlation: context().correlation,
    });
    (result, target)
}

fn synthesis(model: &str) -> ess_conformance::synthesize::Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(model))
}

fn statuses<T: ConformanceTarget>(
    suite: &ConformanceSuite,
    target: &T,
) -> BTreeMap<String, (Status, String)> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let checks = format!("{:?}", result.checks);
            (result.scenario.to_string(), (result.status, checks))
        })
        .collect()
}

// ---- Unknown: an unmet obligation of this binding, not a veto over its siblings ----------------

/// The design: Unknown "invokes nothing and is reported as an unmet obligation"; False skips "this
/// binding occurrence" and "other bindings for the event still run". Whether the unconditioned
/// sibling runs must not depend on whether its name sorts before or after the Unknown binding.
#[test]
fn an_unknown_condition_does_not_decide_by_name_order_whether_a_sibling_binding_runs() {
    let unknown = MODEL
        .replace(
            WHERE,
            "      where: [event.kind == ship, event.order.note == x]\n",
        )
        .replace(ORDER_ID, FROM_MESSAGE);
    let mut counts = BTreeMap::new();
    for sibling in ["logged", "zlogged"] {
        let model = unknown.replace("  - id: logged\n", &format!("  - id: {sibling}\n"));
        let (result, target) = receive(&model, "ship", None);
        assert!(result.is_err(), "{sibling}: Unknown is an unmet obligation");
        assert_eq!(
            observed(&target, "received", "demo.messages.MessageEvent").len(),
            0,
            "{sibling}: the Unknown binding invokes nothing"
        );
        counts.insert(
            sibling,
            observed(&target, sibling, "demo.messages.LogMessage").len(),
        );
    }
    assert_eq!(
        counts["logged"], counts["zlogged"],
        "the unconditioned sibling's invocations depend on its name's sort order relative to the \
         Unknown binding: {counts:?}"
    );
}

// ---- Kleene edges through the interpreter ------------------------------------------------------

#[test]
fn kleene_any_with_one_unknown_and_one_true_invokes_and_not_over_unknown_invokes_nothing() {
    let any = MODEL
        .replace(
            WHERE,
            "      where: {any: [event.order.note == x, event.kind == ship]}\n",
        )
        .replace(ORDER_ID, FROM_MESSAGE);
    let (result, target) = receive(&any, "ship", None);
    result.expect("any(Unknown, True) is True");
    assert_eq!(
        observed(&target, "received", "demo.messages.MessageEvent").len(),
        1
    );

    let not = MODEL
        .replace(WHERE, "      where: {not: event.order.note == x}\n")
        .replace(ORDER_ID, FROM_MESSAGE);
    let (result, target) = receive(&not, "ship", None);
    assert!(result.is_err(), "not(Unknown) is Unknown, never a skip");
    assert_eq!(
        observed(&target, "received", "demo.messages.MessageEvent").len(),
        0
    );
    // Present-but-member-missing: the parent is present, the compared child is not.
    let parent = Node::Map(BTreeMap::from([("id".into(), text("o-1"))]));
    let (result, target) = receive(&not, "ship", Some(parent));
    assert!(result.is_err(), "not(Unknown) with the parent present");
    assert_eq!(
        observed(&target, "received", "demo.messages.MessageEvent").len(),
        0
    );
}

// ---- #194: a parent proved present is not its Optional child ------------------------------------

/// `defined(event.order.note)` is satisfiable by a trigger that sends an order with a note. The
/// unit's own docs say a gap is recorded only where the trigger cannot be varied safely; here it can.
#[test]
fn defined_of_an_optional_child_is_witnessed_rather_than_refused() {
    let child = MODEL.replace(
        WHERE,
        "      where: [defined(event.order.note), event.kind == ship]\n",
    );
    let child = child.replace(ORDER_ID, "      order_id: event.order.note\n");
    let synthesis = synthesis(&child);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .filter(|refusal| format!("{refusal:?}").contains("ConditionUnarranged"))
        .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
        .collect();
    assert_eq!(
        refused.len(),
        0,
        "a satisfiable child condition is refused as unarranged: {refused:#?}"
    );
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.iter().any(|id| id == "received/binding/mapping"),
        "{ids:#?}"
    );
}

/// The suite synthesized for `defined(event.order.note)` must catch a target that tests only the
/// parent (the `optional_child_still_requires_proof` control): the order present with no note is
/// the one fact that separates them. Both bindings map from an always-present field, so only the
/// condition differs.
#[test]
fn a_target_that_tests_only_the_parent_fails_some_condition_witness() {
    let honest = MODEL
        .replace(
            WHERE,
            "      where: [defined(event.order.note), event.kind == ship]\n",
        )
        .replace(ORDER_ID, FROM_MESSAGE);
    let faulty = MODEL.replace(ORDER_ID, FROM_MESSAGE);
    let synthesis = synthesis(&honest);
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    let statuses = statuses(&synthesis.suite, &Interpreted::for_model(ir_of(&faulty)));
    let failed: Vec<&String> = statuses
        .iter()
        .filter(|(_, (status, _))| *status == Status::Failed)
        .map(|(id, _)| id)
        .collect();
    assert!(
        !failed.is_empty(),
        "a target firing on an order without a note passes every scenario: suite {ids:#?}, \
         refusals {:#?}",
        synthesis
            .refusals
            .iter()
            .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
            .collect::<Vec<_>>()
    );
}

// ---- negative witnesses on an honest target ----------------------------------------------------

/// The trigger publishes a `note` message; an unrelated, sound chain re-publishes it as `ship` with
/// the order. The conditioned binding then correctly invokes for the second occurrence, under the
/// same correlation. An honest target must not fail a synthesized scenario: either the witness
/// accounts for the chain or synthesis names it as unarrangeable.
#[test]
fn a_chain_republishing_the_event_does_not_fail_an_honest_target() {
    let chain = MODEL
        .replace(
            "  - name: demo.messages.MessageLogged\n",
            "  - name: demo.messages.Forwarded\n    fields:\n      - {name: message_id, type: String}\n      - {name: order, type: Optional<demo.messages.Ref>}\n  - name: demo.messages.MessageLogged\n",
        )
        .replace(
            "  - name: demo.messages.LogMessage\n",
            "  - name: demo.messages.Forward\n    input:\n      - {name: message_id, type: String}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: forwarded\n        emits: [demo.messages.Forwarded]\n        payload:\n          demo.messages.Forwarded: {message_id: input.message_id, order: input.order}\n  - name: demo.messages.Reship\n    input:\n      - {name: message_id, type: String}\n      - {name: kind, type: demo.messages.Kind}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: reshipped\n        emits: [demo.messages.MessageReceived]\n        payload:\n          demo.messages.MessageReceived: {message_id: input.message_id, kind: input.kind, order: input.order}\n  - name: demo.messages.LogMessage\n",
        )
        + "  - id: forward\n    when:\n      event: demo.messages.MessageReceived\n      where: event.kind == note\n    invoke: {command: demo.messages.Forward}\n    mapping:\n      message_id: event.message_id\n      order: event.order\n    delivery: at_least_once\n    on_failure: retry\n  - id: reship\n    when: {event: demo.messages.Forwarded}\n    invoke: {command: demo.messages.Reship}\n    mapping:\n      message_id: event.message_id\n      kind: ship\n      order: event.order\n    delivery: at_least_once\n    on_failure: retry\n";
    let synthesis = synthesis(&chain);
    let statuses = statuses(&synthesis.suite, &Interpreted::for_model(ir_of(&chain)));
    let failed: Vec<(&String, &String)> = statuses
        .iter()
        .filter(|(_, (status, _))| *status == Status::Failed)
        .map(|(id, (_, checks))| (id, checks))
        .collect();
    assert_eq!(
        failed.len(),
        0,
        "the honest interpreter fails synthesized scenarios: {failed:#?}"
    );
}

/// Two conditioned bindings on one event, with disjoint conditions: each one's witnesses hold on an
/// honest target while the other fires.
#[test]
fn two_conditioned_bindings_where_only_one_holds_pass_on_an_honest_target() {
    let two = MODEL.replace(
        "  - id: logged\n    when: {event: demo.messages.MessageReceived}\n",
        "  - id: logged\n    when:\n      event: demo.messages.MessageReceived\n      where: event.kind == note\n",
    );
    let synthesis = synthesis(&two);
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    assert!(
        ids.iter().any(|id| id == "logged/binding/condition-false"),
        "{ids:#?}"
    );
    let statuses = statuses(&synthesis.suite, &Interpreted::for_model(ir_of(&two)));
    for (id, (status, checks)) in &statuses {
        assert_ne!(*status, Status::Failed, "{id}: {checks}");
    }
}

// ---- a faulty target that fires late ------------------------------------------------------------

/// Delegates to the interpreter, and hides every invocation from the first `hidden` observations of
/// a scenario: a target whose dispatch is asynchronous and slow.
struct Late {
    inner: Interpreted,
    hidden: u32,
    asks: Cell<u32>,
}

impl ConformanceTarget for Late {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.asks.set(0);
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn execute_command_without_input(
        &self,
        request: AbsentInputRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command_without_input(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn configure_external_outcome_repeatedly(
        &self,
        request: ExternalOutcomeControl,
        times: std::num::NonZeroU32,
    ) -> Result<(), TargetError> {
        self.inner
            .configure_external_outcome_repeatedly(request, times)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn deliver_event(&self, request: EventDeliveryRequest) -> Result<(), TargetError> {
        self.inner.deliver_event(request)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn observe_invocations(
        &self,
        request: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        let ask = self.asks.get() + 1;
        self.asks.set(ask);
        let seen = self.inner.observe_invocations(request)?;
        Ok(if ask <= self.hidden { Vec::new() } else { seen })
    }
}

/// `late_unwanted_invocation_fails`: a target that ignores the condition and whose invocation
/// becomes visible only at the last ask of the window still fails the negative witnesses.
#[test]
fn a_target_that_fires_late_within_the_window_fails_both_negative_witnesses() {
    let suite = synthesis(MODEL).suite;
    let ignoring = MODEL
        .replace(WHERE, "      where: true\n")
        .replace(ORDER_ID, FROM_MESSAGE);
    let late = Late {
        inner: Interpreted::for_model(ir_of(&ignoring)),
        hidden: 49,
        asks: Cell::new(0),
    };
    let statuses = statuses(&suite, &late);
    for id in [FALSE, ABSENT] {
        assert_eq!(statuses[id].0, Status::Failed, "{id}: {}", statuses[id].1);
    }
}

// ---- admission edges ---------------------------------------------------------------------------

#[test]
fn a_union_reached_through_a_newtype_alias_is_refused_as_a_condition_path() {
    let union = MODEL
        .replace(
            "  - name: demo.messages.Kind\n",
            "  - name: demo.messages.Choice\n    kind: union\n    tag: kind\n    variants:\n      ref: demo.messages.Ref\n      text: String\n  - name: demo.messages.Alias\n    kind: newtype\n    of: demo.messages.Choice\n  - name: demo.messages.Kind\n",
        )
        .replace(
            "      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: received\n",
            "      - {name: order, type: Optional<demo.messages.Ref>}\n      - {name: choice, type: Optional<demo.messages.Alias>}\n    outcomes:\n      - name: received\n",
        )
        .replace(
            "      - {name: order, type: Optional<demo.messages.Ref>}\n  - name: demo.messages.OrderMessaged\n",
            "      - {name: order, type: Optional<demo.messages.Ref>}\n      - {name: choice, type: Optional<demo.messages.Alias>}\n  - name: demo.messages.OrderMessaged\n",
        )
        .replace(
            "kind: input.kind, order: input.order}",
            "kind: input.kind, order: input.order, choice: input.choice}",
        )
        .replace(ORDER_ID, FROM_MESSAGE);
    // The model itself is admitted without a condition reading through the union.
    admitted(&union).unwrap_or_else(|errors| panic!("the union model is admitted: {errors}"));
    let through = union.replace(WHERE, "      where: event.choice.id == x\n");
    let errors = admitted(&through).expect_err("a path through a union alias is refused");
    assert!(errors.contains("when.where"), "{errors}");
}

#[test]
fn a_variant_of_another_enum_is_refused() {
    let other = MODEL
        .replace(
            "  - name: demo.messages.Kind\n    kind: enum\n    variants: [note, ship]\n",
            "  - name: demo.messages.Kind\n    kind: enum\n    variants: [note, ship]\n  - name: demo.messages.Mode\n    kind: enum\n    variants: [fast, slow]\n",
        )
        .replace(ORDER_ID, FROM_MESSAGE);
    admitted(&other).unwrap_or_else(|errors| panic!("{errors}"));
    let errors = admitted(&other.replace(WHERE, "      where: event.kind == fast\n"))
        .expect_err("a variant of another enum is refused");
    assert!(errors.contains("fast"), "{errors}");
}

// ---- the generated Go runtime against the late target -------------------------------------------

mod support_go;

/// The Go runtime gives the native verdicts for a target that ignores the condition and fires late
/// within the window, and sends no request the reference runner did not.
#[test]
fn go_gives_the_native_verdicts_for_a_target_that_fires_late() {
    let suite = synthesis(MODEL).suite;
    let ignoring = MODEL
        .replace(WHERE, "      where: true\n")
        .replace(ORDER_ID, FROM_MESSAGE);
    let late = Late {
        inner: Interpreted::for_model(ir_of(&ignoring)),
        hidden: 49,
        asks: Cell::new(0),
    };
    let verdicts = support_go::assert_parity("binding-condition-late", &suite, late);
    for id in [FALSE, ABSENT] {
        assert_eq!(verdicts[id], "failed", "{id}: {verdicts:#?}");
    }
}

// ---- the design's own shape: a discriminator each publishing branch writes as a literal --------

/// The design's example is `event.kind == Ship` on an event several commands publish. Here one
/// command publishes `kind: ship` and another `kind: note`, each as a literal of its branch; the
/// shipping trigger with an order makes the condition hold, and the noting trigger makes it fail
/// with every member present. Both are triggers synthesis already builds.
#[test]
fn a_discriminator_written_as_a_branch_literal_is_witnessed_on_both_sides() {
    let literal = MODEL
        .replace(
            "  - name: demo.messages.ReceiveMessage\n    input:\n      - {name: message_id, type: String}\n      - {name: kind, type: demo.messages.Kind}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: received\n        emits: [demo.messages.MessageReceived]\n        payload:\n          demo.messages.MessageReceived: {message_id: input.message_id, kind: input.kind, order: input.order}\n",
            "  - name: demo.messages.ShipMessage\n    input:\n      - {name: message_id, type: String}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: shipped\n        emits: [demo.messages.MessageReceived]\n        payload:\n          demo.messages.MessageReceived: {message_id: input.message_id, kind: ship, order: input.order}\n  - name: demo.messages.NoteMessage\n    input:\n      - {name: message_id, type: String}\n      - {name: order, type: Optional<demo.messages.Ref>}\n    outcomes:\n      - name: noted\n        emits: [demo.messages.MessageReceived]\n        payload:\n          demo.messages.MessageReceived: {message_id: input.message_id, kind: note, order: input.order}\n",
        );
    assert_ne!(literal, MODEL, "the edit applied");
    let synthesis = synthesis(&literal);
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{:?} {}", refusal.scenario, refusal.cause))
        .collect();
    for id in ["received/binding/mapping", "received/binding/flow", FALSE] {
        assert!(
            ids.iter().any(|written| written == id),
            "{id} is not synthesized: suite {ids:#?}, refusals {refused:#?}"
        );
    }
}
