//! Adversary pass 2 against the arrangement unit (beyond10x/ess#198, #209, #199), after the
//! correction that makes an input-guarded refusal's witness refute its sibling refusals.
//!
//! * Every literal send a refusal scenario makes, run through the model's own interpreter, must
//!   select exactly the one outcome the scenario then requires — on overlapping refusals over one
//!   field, over different fields, three at once, an ordering bound beside an equality, an optional
//!   sibling, a stored-row command, and the repository fixture the correction changed.
//! * The correction must not refuse more than the model forces: each refusal that has an input
//!   selecting it alone keeps its scenario, and each one that has none is refused naming the
//!   sibling that claims it.
//! * The arranged half of #209 on a model with no identity view: a target that silently moves the
//!   record on refusal, which the command's own wrong-state branch makes visible, must fail.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store};
use ess_conformance::report::Status;
use ess_conformance::scenario::{CommandRef, ErrorRef, EventRef, OutcomeRef};
use ess_conformance::target::*;
use ess_conformance::{
    synthesize::Synthesis, AdmittedSuite, ConformanceScenario, Runner, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const INPUT_REFUSAL: &str = include_str!("fixtures/arrangement-input-refusal.yaml");
const EXPLORE_EXITS: &str = include_str!("fixtures/explore-external-exits.yaml");

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|e| panic!("{e:?}"))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {}: {}",
                refusal.cause.code(),
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.cause
            )
        })
        .collect()
}

fn ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn find<'a>(result: &'a Synthesis, id: &str) -> Option<&'a ConformanceScenario> {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

/// Every complete literal send of `command` in every `…/outcome/…` scenario, with the outcome the
/// scenario requires right after it, decided by the interpreter: the send must select exactly that
/// one outcome. Sends naming an arranged instance are left out (the interpreter would need the
/// arranged store), as are sends leaving an input out.
fn ambiguous_sends(text: &str, command: &str) -> Vec<String> {
    let ir = compiled(text);
    let result = ess_conformance::synthesize::synthesize(&ir);
    let name = command.parse().unwrap();
    let declared = ir
        .commands()
        .values()
        .find(|c| c.name.to_string() == command)
        .map(|c| c.input.len())
        .expect("command declared");
    let mut open = Vec::new();
    let mut checked = 0;
    for (id, scenario) in &result.suite.scenarios {
        let id = id.to_string();
        if !id.starts_with(&format!("{command}/outcome/")) {
            continue;
        }
        let steps = &scenario.steps;
        for (at, step) in steps.iter().enumerate() {
            let ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } = step
            else {
                continue;
            };
            if sent.to_string() != command {
                continue;
            }
            let literal: Option<BTreeMap<String, Node>> = input
                .iter()
                .map(|(field, value)| match value {
                    ScenarioValue::Literal { value } => Some((field.clone(), value.clone())),
                    _ => None,
                })
                .collect();
            let Some(literal) = literal.filter(|literal| literal.len() == declared) else {
                continue;
            };
            let required = steps[at + 1..].iter().find_map(|next| match next {
                ScenarioStep::ExpectOutcome { outcome } => Some(outcome.to_string()),
                ScenarioStep::ExecuteCommand { .. } => Some(String::from("<none required>")),
                _ => None,
            });
            let Some(required) = required.filter(|r| r != "<none required>") else {
                continue;
            };
            checked += 1;
            let taken = execute(
                &ir,
                &Store::default(),
                &name,
                &literal,
                &Externals::Withheld,
            )
            .unwrap_or_else(|why| panic!("{id}: {why}"));
            let outcomes: Vec<String> = taken
                .iter()
                .map(|step| {
                    step.outcome
                        .as_ref()
                        .map_or_else(|| "<undeclared>".to_owned(), ToString::to_string)
                })
                .collect();
            if outcomes != [required.clone()] {
                open.push(format!(
                    "{id}: {literal:?} requires {required}, the model allows {outcomes:?}"
                ));
            }
        }
    }
    assert!(
        checked > 0,
        "no literal send of {command} was checked: {:#?}",
        ids(&result)
    );
    open
}

fn model(outcomes: &str, inputs: &str) -> String {
    format!(
        r"format: ess/16
system: demo
version: v1
domain: demo.pins
summary: Pins refused by overlapping input guards, otherwise set.
types:
  - {{name: demo.pins.PinId, kind: newtype, of: Uuid}}
entities:
  - name: demo.pins.Pin
    identity: {{name: pin_id, type: demo.pins.PinId}}
    fields: []
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
actors:
  - {{name: demo.pins.Service, may: [demo.pins.SetPin]}}
errors:
  - {{name: demo.pins.A, summary: A., fields: []}}
  - {{name: demo.pins.B, summary: B., fields: []}}
  - {{name: demo.pins.C, summary: C., fields: []}}
commands:
  - name: demo.pins.SetPin
    input:
{inputs}
    outcomes:
{outcomes}
      - name: set
        creates: demo.pins.Pin
        instance: pin_id
        emits: [demo.pins.PinSet]
        payload:
          demo.pins.PinSet: {{pin_id: {{generated: true}}}}
events:
  - name: demo.pins.PinSet
    fields:
      - {{name: pin_id, type: demo.pins.PinId}}
"
    )
}

const COUNT_FLAG: &str = "      - {name: count, type: Integer}
      - {name: flag, type: Boolean}";

/// `count < 5` inside `count < 12`, a disjoint `count >= 100`, and `flag == true` over another
/// field.
fn nested() -> String {
    model(
        "      - {name: far-too-short, when: count < 5, error: demo.pins.A}
      - {name: too-short, when: count < 12, error: demo.pins.B}
      - {name: big, when: count >= 100, error: demo.pins.C}
      - {name: flagged, when: flag == true, error: demo.pins.C}",
        COUNT_FLAG,
    )
}

/// Three refusals over one field that pairwise overlap and each keep an input of their own:
/// `count < 5`, `3 <= count < 10`, `8 <= count < 30`. Alone: `low` below 3, `mid` 5..7, `high`
/// 10..29, `set` from 30.
fn three() -> String {
    model(
        "      - {name: low, when: count < 5, error: demo.pins.A}
      - {name: mid, when: {all: [count >= 3, count < 10]}, error: demo.pins.B}
      - {name: high, when: {all: [count >= 8, count < 30]}, error: demo.pins.C}",
        COUNT_FLAG,
    )
}

/// An ordering bound beside an equality on its boundary value: `count < 12` and `count == 11`.
fn bound_and_equality() -> String {
    model(
        "      - {name: too-short, when: count < 12, error: demo.pins.A}
      - {name: eleven, when: count == 11, error: demo.pins.B}",
        COUNT_FLAG,
    )
}

/// A refusal beside a refusal reading an optional input.
fn optional_sibling() -> String {
    model(
        "      - {name: negative, when: count < 0, error: demo.pins.A}
      - {name: memo-empty, when: memo == \"\", error: demo.pins.B}",
        "      - {name: count, type: Integer}
      - {name: memo, type: Optional<String>}",
    )
}

// ---- every literal refusal send selects exactly the outcome it requires --------------------

#[test]
fn nested_refusals_each_send_selects_exactly_the_required_outcome() {
    let open = ambiguous_sends(&nested(), "demo.pins.SetPin");
    assert!(open.is_empty(), "{open:#?}");
}

#[test]
fn three_overlapping_refusals_each_send_selects_exactly_the_required_outcome() {
    let open = ambiguous_sends(&three(), "demo.pins.SetPin");
    assert!(open.is_empty(), "{open:#?}");
}

#[test]
fn a_bound_beside_an_equality_on_its_boundary_selects_exactly_the_required_outcome() {
    let open = ambiguous_sends(&bound_and_equality(), "demo.pins.SetPin");
    assert!(open.is_empty(), "{open:#?}");
}

#[test]
fn a_refusal_beside_an_optional_sibling_selects_exactly_the_required_outcome() {
    let open = ambiguous_sends(&optional_sibling(), "demo.pins.SetPin");
    assert!(open.is_empty(), "{open:#?}");
}

/// The repository fixture whose suite the correction changed: `low: score <= 3` and
/// `high: score >= 1` overlap for 1..3. Before the correction `low` was sent `1` and `3`, both of
/// which `high` also claims.
#[test]
fn explore_external_exits_rate_sends_select_exactly_the_required_outcome() {
    let open = ambiguous_sends(EXPLORE_EXITS, "exploreexit.desk.Rate");
    assert!(open.is_empty(), "{open:#?}");
}

#[test]
fn the_input_refusal_fixture_plain_sends_select_exactly_the_required_outcome() {
    let open = ambiguous_sends(INPUT_REFUSAL, "vault.acct.Configure");
    assert!(open.is_empty(), "{open:#?}");
}

// ---- the correction refuses what the model forces, and nothing more ------------------------

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&compiled(text))
}

fn assert_kept(result: &Synthesis, names: &[&str]) {
    for name in names {
        let id = format!("demo.pins.SetPin/outcome/{name}");
        assert!(
            find(result, &id).is_some(),
            "{id} withdrawn although an input selects it alone: {:#?}",
            refusals(result)
        );
    }
}

fn refusal_for(result: &Synthesis, name: &str) -> Option<String> {
    let id = format!("demo.pins.SetPin/outcome/{name}");
    refusals(result).into_iter().find(|line| line.contains(&id))
}

#[test]
fn nested_refusals_keep_every_scenario_an_input_selects_alone() {
    let result = synthesis(&nested());
    // Superseded by the precedence order (#227 correction 1): of two refusals an input selects,
    // the first declared answers, so `far-too-short`, declared first, owns `count < 5` and keeps
    // its scenario; it was withdrawn as having no input of its own.
    assert_kept(
        &result,
        &["far-too-short", "too-short", "big", "flagged", "set"],
    );
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
}

#[test]
fn three_overlapping_refusals_keep_every_scenario() {
    let result = synthesis(&three());
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    assert_kept(&result, &["low", "mid", "high", "set"]);
}

#[test]
fn an_equality_on_a_bound_is_refused_naming_the_bound_and_the_bound_keeps_its_scenario() {
    let result = synthesis(&bound_and_equality());
    assert_kept(&result, &["too-short", "set"]);
    let eleven = refusal_for(&result, "eleven").expect("count == 11 lies inside count < 12");
    assert!(eleven.contains("too-short (count < 12)"), "{eleven}");
}

#[test]
fn a_refusal_beside_an_optional_sibling_keeps_both_scenarios() {
    let result = synthesis(&optional_sibling());
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    assert_kept(&result, &["negative", "memo-empty", "set"]);
}

// ---- #209 arranged half with no identity view -----------------------------------------------

/// How the hand-written account service answers `Configure`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// Input refusals first, then existence, then the move (the documented precedence).
    Correct,
    /// [`Mode::Correct`], except a refusal for a stored account moves it to `Configured` anyway,
    /// with no event. Visible without any view: the next valid `Configure` of that account answers
    /// `already-configured` where the model answers `configured`.
    MovesOnRefusal,
}

struct Accounts {
    mode: Mode,
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Accounts {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            minted: Cell::new(0),
            rows: RefCell::default(),
            published: RefCell::default(),
        }
    }
}

fn branch(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn text(node: Option<&Node>) -> String {
    match node {
        Some(Node::Text(value)) => value.clone(),
        other => panic!("expected text, got {other:?}"),
    }
}

impl ConformanceTarget for Accounts {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("accounts-fixture-pass2", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        self.published.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let refuse = |name: &str, error: &str| {
            SemanticCommandResult::took(branch(&command, name))
                .with_error(DeclaredErrorValue::new(error.parse::<ErrorRef>().unwrap()))
        };
        let mut rows = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "vault.acct.Onboard" => {
                let id = format!("acct-{n}");
                rows.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("account_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("New".to_owned())),
                        ("issuer".to_owned(), Node::Text(String::new())),
                    ]),
                );
                let event = ObservedEvent::new("vault.acct.Onboarded".parse::<EventRef>().unwrap())
                    .with("account_id", Node::Text(id));
                SemanticCommandResult::took(branch(&command, "onboarded")).emitting(event)
            }
            "vault.acct.Configure" => {
                let id = text(request.input.get("id"));
                let secret = text(request.input.get("secret"));
                let issuer = text(request.input.get("issuer"));
                let refused = if id.is_empty() {
                    Some(("id-required", "vault.acct.IdRequired"))
                } else if secret.chars().count() < 12 {
                    Some(("secret-too-short", "vault.acct.SecretTooShort"))
                } else if issuer.is_empty() {
                    Some(("missing-configuration", "vault.acct.MissingConfiguration"))
                } else {
                    None
                };
                if let Some((name, error)) = refused {
                    if self.mode == Mode::MovesOnRefusal {
                        if let Some(row) = rows.get_mut(&id) {
                            row.insert("state".to_owned(), Node::Text("Configured".to_owned()));
                        }
                    }
                    refuse(name, error)
                } else {
                    match rows.get_mut(&id) {
                        None => refuse("already-configured", "vault.acct.AlreadyConfigured"),
                        Some(row) if row["state"] != Node::Text("New".to_owned()) => {
                            refuse("already-configured", "vault.acct.AlreadyConfigured")
                        }
                        Some(row) => {
                            row.insert("state".to_owned(), Node::Text("Configured".to_owned()));
                            row.insert("issuer".to_owned(), Node::Text(issuer));
                            SemanticCommandResult::took(branch(&command, "configured")).emitting(
                                ObservedEvent::new(
                                    "vault.acct.Configured".parse::<EventRef>().unwrap(),
                                )
                                .with("account_id", Node::Text(id)),
                            )
                        }
                    }
                }
            }
            other => panic!("unexpected command {other}"),
        };
        for event in &result.direct_events {
            self.published.borrow_mut().push(event.clone());
        }
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.rows.borrow().values().cloned().collect::<Vec<_>>(),
        ))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .published
            .borrow()
            .iter()
            .filter(|event| event.event == request.event)
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

fn without_view() -> String {
    let cut = INPUT_REFUSAL
        .find("views:")
        .expect("the fixture declares a view");
    INPUT_REFUSAL[..cut].to_owned()
}

fn failures(text: &str, mode: Mode) -> BTreeMap<String, String> {
    let result = synthesis(text);
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Accounts::new(mode))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|run| run.status != Status::Passed)
        .map(|run| {
            (
                run.scenario.to_string(),
                format!("{:?}: {:?}", run.status, run.checks),
            )
        })
        .collect()
}

/// The control: the correct target passes the no-view suite, so a failure below is the mutant's.
#[test]
fn issue_209_no_view_suite_passes_the_correct_target() {
    let failed = failures(&without_view(), Mode::Correct);
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The same target with its view: moving the record on refusal is caught by the unchanged-row
/// claim.
#[test]
fn issue_209_with_a_view_moving_the_record_on_refusal_is_caught() {
    let failed = failures(INPUT_REFUSAL, Mode::MovesOnRefusal);
    assert!(
        failed.contains_key("vault.acct.Configure/outcome/secret-too-short"),
        "{failed:#?}"
    );
}

/// Without a view the arranged half requires the error and no event, and nothing more. The
/// record's state is still observable through the command itself: the wrong-state branch answers
/// a second `Configure` of a moved account. So a target that moves the account on refusal and
/// publishes nothing passes every scenario.
///
/// This pins that known gap as it stands today. The probe through the command is
/// `story:a-no-view-arranged-half-probes-the-row-through-the-command`; when it lands, flip this
/// case to require that both refusal scenarios fail against this target.
#[test]
fn issue_209_no_view_arranged_half_does_not_yet_catch_a_target_moving_the_record_on_refusal() {
    let failed = failures(&without_view(), Mode::MovesOnRefusal);
    assert!(
        !failed.contains_key("vault.acct.Configure/outcome/secret-too-short")
            && !failed.contains_key("vault.acct.Configure/outcome/missing-configuration"),
        "this case pins a known gap, where a target moving the stored account on refusal passes \
         with no view. It is filed as \
         story:a-no-view-arranged-half-probes-the-row-through-the-command. A refusal scenario \
         now fails against this target, so flip the case to require both to fail once that story \
         lands: {failed:#?}"
    );
}
