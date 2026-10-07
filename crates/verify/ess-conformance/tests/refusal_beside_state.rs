//! An input-guarded refusal beside branches selected by the held state (beyond10x/ess#227, first
//! filed as #213).
//!
//! `too-short` (`when: secret.count < 12`, `error:`) names no subject; `rotated` updates the record
//! in `Configured`; `not-configured` refuses every other state. The refusal is answered before
//! existence and before the held state (`docs/design/outcome-shapes.md` "Precedence", #209), so
//! its scenario sends the refused input for an identity nothing stored, and then once for a row
//! arranged in each state a sibling runs from, requiring the error, no event and the row
//! unchanged. The suites run against a hand-written target answering that precedence, and
//! against mutants of it that look at the state or the record first.
#![allow(clippy::map_unwrap_or, clippy::too_many_lines)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
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

const ROTATE: &str = include_str!("fixtures/refusal-beside-state.yaml");

const TOO_SHORT: &str = "demo.secrets.RotateSecret/outcome/too-short";
const ROTATED: &str = "demo.secrets.RotateSecret/outcome/rotated";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
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

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
        .unwrap_or_else(|| {
            panic!(
                "no scenario {id}\n ids: {:#?}\n refusals: {:#?}",
                result
                    .suite
                    .scenarios
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
                refusals(result)
            )
        })
}

fn sent(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => {
                Some(command.to_string().rsplit('.').next().unwrap().to_owned())
            }
            _ => None,
        })
        .collect()
}

// ---- the synthesized steps -----------------------------------------------------------------

#[test]
fn the_issue_reproduction_synthesizes_without_a_refusal() {
    let result = synthesis(ROTATE);
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    scenario(&result, TOO_SHORT);
    scenario(&result, ROTATED);
}

#[test]
fn the_refusal_is_sent_plain_first_then_for_a_row_in_every_state() {
    let result = synthesis(ROTATE);
    let refused = scenario(&result, TOO_SHORT);
    let commands = sent(refused);
    // The plain send: no record, the refusal answered before existence.
    assert_eq!(
        commands.first().map(String::as_str),
        Some("RotateSecret"),
        "{commands:?}"
    );
    // Then one arranged row per state: Pending (registered), Configured (registered, configured),
    // Revoked (registered, configured, revoked), each followed by the refused send.
    let rows = commands.iter().filter(|name| *name == "Register").count();
    assert_eq!(rows, 3, "{commands:?}");
    let sends = commands
        .iter()
        .filter(|name| *name == "RotateSecret")
        .count();
    let errors = refused
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExpectError { error, .. }
            if error.to_string() == "demo.secrets.SecretTooShort")
        })
        .count();
    assert_eq!(errors, sends, "every send requires the refusal");
    let arranged = refused
        .steps
        .iter()
        .filter(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                command.to_string() == "demo.secrets.RotateSecret"
                    && matches!(input.get("tenant_id"), Some(ScenarioValue::Instance { .. }))
            }
            _ => false,
        })
        .count();
    assert_eq!(arranged, 3, "{:#?}", refused.steps);
}

#[test]
fn each_arranged_row_is_a_record_of_its_own() {
    let result = synthesis(ROTATE);
    let refused = scenario(&result, TOO_SHORT);
    let instances: Vec<String> = refused
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.secrets.RotateSecret" =>
            {
                match input.get("tenant_id") {
                    Some(ScenarioValue::Instance { instance }) => Some(instance.to_string()),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect();
    let distinct: BTreeSet<&String> = instances.iter().collect();
    assert_eq!(distinct.len(), instances.len(), "{instances:?}");
}

// ---- run against a target ------------------------------------------------------------------

/// How the hand-written secrets service answers `RotateSecret`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    /// The input check first, then existence, then the held state (the documented precedence).
    Correct,
    /// Reads the held state before the input: `rotated` in `Configured`, `not-configured`
    /// elsewhere, and the input checked only where neither answers — never, here.
    StateFirst,
    /// Looks the record up before the input check: an unknown identity is `not-configured`.
    ExistenceFirst,
    /// Checks the input for every record except one resting in this state.
    LaxIn(&'static str),
    /// Refuses, and writes the sent secret onto the stored record anyway.
    WritesOnRefusal,
    /// Refuses, and publishes `SecretRotated` for the stored record anyway.
    EmitsOnRefusal,
    /// Tries `rotated` (held state and its `mode` guard) before the input check, then the
    /// refusal, then the default: correct wherever the two do not overlap.
    SiblingFirst,
}

struct Secrets {
    mode: Mode,
    /// Whether `rotated` also reads `mode == Rotate` (the [`guarded`] model).
    guarded: bool,
    minted: Cell<u64>,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Secrets {
    fn new(mode: Mode, guarded: bool) -> Self {
        Self {
            mode,
            guarded,
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

fn event(name: &str, id: &str) -> ObservedEvent {
    ObservedEvent::new(name.parse::<EventRef>().unwrap()).with("tenant_id", Node::Text(id.into()))
}

impl ConformanceTarget for Secrets {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("secrets-fixture", "1"))
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
        let moved = |rows: &mut BTreeMap<String, BTreeMap<String, Node>>,
                     id: &str,
                     from: &str,
                     to: &str,
                     name: &str,
                     published: &str,
                     wrong: (&str, &str)| {
            let Some(row) = rows
                .get_mut(id)
                .filter(|row| row["state"] == Node::Text(from.into()))
            else {
                return refuse(wrong.0, wrong.1);
            };
            row.insert("state".to_owned(), Node::Text(to.into()));
            SemanticCommandResult::took(branch(&command, name)).emitting(event(published, id))
        };
        let result = match command.to_string().as_str() {
            "demo.secrets.Register" => {
                let id = format!("00000000-0000-4000-8000-{n:012}");
                rows.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("tenant_id".to_owned(), Node::Text(id.clone())),
                        ("state".to_owned(), Node::Text("Pending".to_owned())),
                        (
                            "secret".to_owned(),
                            Node::Text("initial-secret-value".to_owned()),
                        ),
                    ]),
                );
                SemanticCommandResult::took(branch(&command, "registered"))
                    .emitting(event("demo.secrets.Registered", &id))
            }
            "demo.secrets.Configure" => moved(
                &mut rows,
                &text(request.input.get("tenant_id")),
                "Pending",
                "Configured",
                "configured",
                "demo.secrets.Configured",
                ("not-pending", "demo.secrets.NotPending"),
            ),
            "demo.secrets.Revoke" => moved(
                &mut rows,
                &text(request.input.get("tenant_id")),
                "Configured",
                "Revoked",
                "revoked",
                "demo.secrets.Revoked",
                ("not-active", "demo.secrets.NotActive"),
            ),
            "demo.secrets.RotateSecret" => {
                let id = text(request.input.get("tenant_id"));
                let secret = text(request.input.get("secret"));
                let held = rows
                    .get(&id)
                    .map(|row| text(row.get("state")))
                    .unwrap_or_default();
                let selects = !self.guarded || text(request.input.get("mode")) == "Rotate";
                let sibling = held == "Configured" && selects;
                let checks = match self.mode {
                    Mode::StateFirst => false,
                    Mode::SiblingFirst => !sibling,
                    Mode::ExistenceFirst => !held.is_empty(),
                    Mode::LaxIn(state) => held != state,
                    _ => true,
                };
                if checks && secret.chars().count() < 12 {
                    let mut answer = refuse("too-short", "demo.secrets.SecretTooShort");
                    if let Some(row) = rows.get_mut(&id) {
                        match self.mode {
                            Mode::WritesOnRefusal => {
                                row.insert("secret".to_owned(), Node::Text(secret));
                            }
                            Mode::EmitsOnRefusal => {
                                answer = answer.emitting(event("demo.secrets.SecretRotated", &id));
                            }
                            _ => {}
                        }
                    }
                    answer
                } else if sibling {
                    let row = rows.get_mut(&id).expect("held");
                    row.insert("secret".to_owned(), Node::Text(secret));
                    SemanticCommandResult::took(branch(&command, "rotated"))
                        .emitting(event("demo.secrets.SecretRotated", &id))
                } else {
                    refuse("not-configured", "demo.secrets.NotConfigured")
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

/// Every scenario of `model` that did not pass against `mode`, with its checks.
fn failures_of(model: &str, mode: Mode, guarded: bool) -> BTreeMap<String, String> {
    let result = synthesis(model);
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Secrets::new(mode, guarded))
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

/// Every scenario of [`ROTATE`] that did not pass against `mode`.
fn failures(mode: Mode) -> BTreeMap<String, String> {
    failures_of(ROTATE, mode, false)
}

#[test]
fn every_scenario_passes_a_target_answering_the_input_before_the_state() {
    let failed = failures(Mode::Correct);
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_target_answering_the_held_state_before_the_input_fails_the_refusal() {
    let failed = failures(Mode::StateFirst);
    assert!(failed.contains_key(TOO_SHORT), "{failed:#?}");
}

#[test]
fn a_target_looking_the_record_up_first_fails_the_refusal() {
    let failed = failures(Mode::ExistenceFirst);
    assert!(failed.contains_key(TOO_SHORT), "{failed:#?}");
}

#[test]
fn a_target_skipping_the_input_check_in_any_one_state_fails_the_refusal() {
    for state in ["Pending", "Configured", "Revoked"] {
        let failed = failures(Mode::LaxIn(state));
        assert!(failed.contains_key(TOO_SHORT), "{state}: {failed:#?}");
    }
}

#[test]
fn a_target_writing_or_publishing_on_refusal_fails() {
    for mode in [Mode::WritesOnRefusal, Mode::EmitsOnRefusal] {
        let failed = failures(mode);
        assert!(failed.contains_key(TOO_SHORT), "{mode:?}: {failed:#?}");
    }
}

// ---- a sibling that reads the input as well --------------------------------------------------

/// [`ROTATE`] with `rotated` also guarded by `mode == Rotate`, a closed input the refusal does not
/// read: in `Configured`, `{secret: <short>, mode: Rotate}` satisfies both.
fn guarded() -> String {
    let replace = |text: &str, from: &str, to: &str| {
        let out = text.replace(from, to);
        assert_ne!(out, text, "`{from}` is in the fixture");
        out
    };
    let text = replace(
        ROTATE,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Mode, kind: enum, variants: [Freeze, Rotate]}\n",
    );
    let text = replace(
        &text,
        "      - {name: secret, type: String}\n    outcomes:\n      - name: too-short\n",
        "      - {name: secret, type: String}\n      - {name: mode, type: demo.secrets.Mode}\n    outcomes:\n      - name: too-short\n",
    );
    replace(
        &text,
        "        when_subject_state: Configured\n",
        "        when_subject_state: Configured\n        when: mode == Rotate\n",
    )
}

#[test]
fn a_guarded_sibling_model_passes_a_target_answering_the_input_first() {
    let result = synthesis(&guarded());
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    let failed = failures_of(&guarded(), Mode::Correct, true);
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_target_trying_the_guarded_sibling_first_fails_the_refusal_at_their_overlap() {
    let failed = failures_of(&guarded(), Mode::SiblingFirst, true);
    assert!(failed.contains_key(TOO_SHORT), "{failed:#?}");
}

#[test]
fn the_overlap_is_sent_for_the_row_in_the_state_the_sibling_runs_in() {
    let result = synthesis(&guarded());
    let refused = scenario(&result, TOO_SHORT);
    let overlap = refused.steps.iter().any(|step| match step {
        ScenarioStep::ExecuteCommand { command, input, .. } => {
            command.to_string() == "demo.secrets.RotateSecret"
                && matches!(input.get("tenant_id"), Some(ScenarioValue::Instance { .. }))
                && matches!(input.get("mode"), Some(ScenarioValue::Literal { value })
                    if *value == Node::Text("Rotate".to_owned()))
        }
        _ => false,
    });
    assert!(overlap, "{:#?}", refused.steps);
}

// ---- the other held-state and stored-row shapes --------------------------------------------

fn replace(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// `rotated` selected by `when_state_changes: true` on `configure` (from `Pending`).
fn state_changes() -> String {
    replace(
        ROTATE,
        "        when_subject_state: Configured\n        updates: demo.secrets.Configuration\n",
        "        when_state_changes: true\n        moves: demo.secrets.Configuration.configure\n",
    )
}

/// `rotated` selected by a stored enum field, `when_subject: {field: tier, equals: Basic}`.
fn stored_field() -> String {
    let text = replace(
        ROTATE,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Tier, kind: enum, variants: [Basic, Gold]}\n",
    );
    let text = replace(
        &text,
        "      - {name: secret, type: String}\n    lifecycle:",
        "      - {name: secret, type: String}\n      - {name: tier, type: demo.secrets.Tier}\n    lifecycle:",
    );
    let text = replace(
        &text,
        "        sets: {secret: \"initial-secret-value\"}\n",
        "        sets: {secret: \"initial-secret-value\", tier: Basic}\n",
    );
    let text = replace(
        &text,
        "      - {name: state, type: demo.secrets.Configuration.State}\n      - {name: secret, type: String}\n",
        "      - {name: state, type: demo.secrets.Configuration.State}\n      - {name: secret, type: String}\n      - {name: tier, type: demo.secrets.Tier}\n",
    );
    replace(
        &text,
        "        when_subject_state: Configured\n",
        "        when_subject: {field: tier, equals: Basic}\n",
    )
}

/// How many `RotateSecret` sends in `scenario` name an arranged record.
fn arranged_sends(scenario: &ConformanceScenario) -> usize {
    scenario
        .steps
        .iter()
        .filter(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. } => {
                command.to_string() == "demo.secrets.RotateSecret"
                    && matches!(input.get("tenant_id"), Some(ScenarioValue::Instance { .. }))
            }
            _ => false,
        })
        .count()
}

#[test]
fn beside_a_state_change_guard_the_refusal_is_sent_for_a_row_in_every_state() {
    let result = synthesis(&state_changes());
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    let refused = scenario(&result, TOO_SHORT);
    assert_eq!(
        sent(refused).first().map(String::as_str),
        Some("RotateSecret")
    );
    assert_eq!(arranged_sends(refused), 3, "{:#?}", refused.steps);
}

#[test]
fn beside_a_stored_field_guard_the_refusal_is_sent_for_an_arranged_row() {
    let result = synthesis(&stored_field());
    // No creation stores `Gold`, so the default is unreachable here; the refusal is not.
    let about_it: Vec<String> = refusals(&result)
        .into_iter()
        .filter(|refusal| refusal.contains(TOO_SHORT))
        .collect();
    assert!(about_it.is_empty(), "{about_it:#?}");
    let refused = scenario(&result, TOO_SHORT);
    assert!(arranged_sends(refused) >= 1, "{:#?}", refused.steps);
}

// ---- the model's own interpreter selects the refusal first ---------------------------------

mod interpreter {
    use super::*;
    use ess_conformance::interpret::execute::{execute, Externals, Store};
    use ess_conformance::interpret::Interpreted;

    const TENANT: &str = "00000000-0000-4000-8000-000000000001";

    fn rotate(secret: &str, tenant: &str) -> BTreeMap<String, Node> {
        BTreeMap::from([
            ("tenant_id".to_owned(), Node::Text(tenant.to_owned())),
            ("secret".to_owned(), Node::Text(secret.to_owned())),
        ])
    }

    /// The store after `Register`, then each of `moves` (`Configure`, `Revoke`) for its record.
    fn stored(ir: &EssIr, moves: &[&str]) -> (Store, String) {
        let one = |store: &Store, command: &str, input: &BTreeMap<String, Node>| {
            let mut steps = execute(
                ir,
                store,
                &command.parse().unwrap(),
                input,
                &Externals::Withheld,
            )
            .unwrap_or_else(|why| panic!("{command}: {why}"));
            assert_eq!(steps.len(), 1, "{command}");
            steps.remove(0)
        };
        let registered = one(&Store::default(), "demo.secrets.Register", &BTreeMap::new());
        let Some(Node::Text(tenant)) = registered.events[0].payload.get("tenant_id").cloned()
        else {
            panic!("{:?}", registered.events)
        };
        let mut store = registered.next;
        for command in moves {
            let input = BTreeMap::from([("tenant_id".to_owned(), Node::Text(tenant.clone()))]);
            store = one(&store, command, &input).next;
        }
        (store, tenant)
    }

    fn answer(ir: &EssIr, store: &Store, input: &BTreeMap<String, Node>) -> Vec<String> {
        execute(
            ir,
            store,
            &"demo.secrets.RotateSecret".parse().unwrap(),
            input,
            &Externals::Withheld,
        )
        .unwrap_or_else(|why| panic!("{why}"))
        .into_iter()
        .map(|step| {
            assert!(step.events.is_empty(), "{:?}", step.events);
            assert_eq!(&step.next, store, "a refusal changes nothing");
            format!(
                "{} {}",
                step.outcome
                    .map(|outcome| outcome.to_string())
                    .unwrap_or_default(),
                step.error
                    .map(|error| error.error.to_string())
                    .unwrap_or_default()
            )
        })
        .collect()
    }

    const REFUSED: &str = "demo.secrets.RotateSecret/too-short demo.secrets.SecretTooShort";

    #[test]
    fn a_refused_input_is_answered_for_an_identity_nothing_stored() {
        let ir = ir(ROTATE);
        assert_eq!(
            answer(&ir, &Store::default(), &rotate("short", TENANT)),
            [REFUSED]
        );
    }

    #[test]
    fn a_refused_input_is_answered_for_a_record_in_every_state() {
        let ir = ir(ROTATE);
        for moves in [
            &[][..],
            &["demo.secrets.Configure"][..],
            &["demo.secrets.Configure", "demo.secrets.Revoke"][..],
        ] {
            let (store, tenant) = stored(&ir, moves);
            assert_eq!(
                answer(&ir, &store, &rotate("short", &tenant)),
                [REFUSED],
                "{moves:?}"
            );
        }
    }

    #[test]
    fn an_input_the_refusal_does_not_claim_still_reads_the_held_state_and_is_not_guessed() {
        let ir = ir(ROTATE);
        for (moves, outcome) in [
            (&[][..], "not-configured"),
            (&["demo.secrets.Configure"][..], "rotated"),
            (
                &["demo.secrets.Configure", "demo.secrets.Revoke"][..],
                "not-configured",
            ),
        ] {
            let (store, tenant) = stored(&ir, moves);
            let steps = execute(
                &ir,
                &store,
                &"demo.secrets.RotateSecret".parse().unwrap(),
                &rotate("long-enough-secret", &tenant),
                &Externals::Withheld,
            )
            .expect("the actual held state decides the branch");
            assert_eq!(steps.len(), 1);
            let step = &steps[0];
            assert_eq!(
                step.outcome.as_ref().unwrap().to_string(),
                format!("demo.secrets.RotateSecret/{outcome}")
            );
            if outcome == "rotated" {
                assert_eq!(step.events.len(), 1);
                let instance = step
                    .next
                    .instance(&"demo.secrets.Configuration".parse().unwrap(), &tenant)
                    .unwrap();
                assert_eq!(
                    instance.fields["secret"],
                    Node::Text("long-enough-secret".to_owned())
                );
            } else {
                assert_eq!(step.next, store);
                assert_eq!(step.events.len(), 0);
                assert_eq!(
                    step.error.as_ref().unwrap().error.to_string(),
                    "demo.secrets.NotConfigured"
                );
            }
        }
    }

    /// The interpreter executes the sends and the observations that prove a refusal leaves
    /// the configured state unchanged.
    #[test]
    fn the_plain_sends_and_view_observations_pass_against_the_interpreted_model() {
        let ir = ir(ROTATE);
        let result = ess_conformance::synthesize::synthesize(&ir);
        let admitted =
            AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
        let report = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(ir.clone()))
            .into_report();
        let run = report
            .scenarios
            .iter()
            .find(|run| run.scenario.to_string() == TOO_SHORT)
            .expect("the refusal scenario ran");
        assert_eq!(run.status, Status::Passed, "{:?}", run.checks);
        assert!(run
            .checks
            .iter()
            .all(|check| check.status == Status::Passed));
        assert!(run
            .checks
            .iter()
            .any(|check| check.about.starts_with("view ")));
        assert!(run
            .checks
            .iter()
            .any(|check| check.about.starts_with("subject snapshot ")));
        assert!(
            run.checks
                .iter()
                .any(|check| check.about == "outcome demo.secrets.RotateSecret/too-short"),
            "{:?}",
            run.checks
        );
    }
}

// ---- two refusals overlapping: the first declared answers -----------------------------------

const FROZEN: &str = "demo.secrets.RotateSecret/outcome/frozen";
const HALTED: &str = "demo.secrets.RotateSecret/outcome/halted";

/// [`ROTATE`] with `frozen: mode == Freeze` declared before `halted: mode != Rotate`: `Freeze`
/// selects both and is `frozen`'s, `Pause` is `halted`'s alone.
fn two_refusals() -> String {
    let text = replace(
        ROTATE,
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.secrets.TenantId, kind: newtype, of: Uuid}\n  - {name: demo.secrets.Mode, kind: enum, variants: [Rotate, Freeze, Pause]}\n",
    );
    let text = replace(
        &text,
        "      - {name: secret, type: String}\n    outcomes:\n",
        "      - {name: secret, type: String}\n      - {name: mode, type: demo.secrets.Mode}\n    outcomes:\n",
    );
    replace(
        &text,
        "        error: demo.secrets.SecretTooShort\n      - name: rotated\n",
        "        error: demo.secrets.SecretTooShort\n      - name: frozen\n        when: mode == Freeze\n        error: demo.secrets.NotPending\n      - name: halted\n        when: mode != Rotate\n        error: demo.secrets.NotActive\n      - name: rotated\n",
    )
}

fn modes_sent(scenario: &ConformanceScenario) -> BTreeSet<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.secrets.RotateSecret" =>
            {
                match input.get("mode") {
                    Some(ScenarioValue::Literal {
                        value: Node::Text(mode),
                    }) => Some(mode.clone()),
                    other => Some(format!("{other:?}")),
                }
            }
            _ => None,
        })
        .collect()
}

#[test]
fn the_later_of_two_overlapping_refusals_is_witnessed_only_where_it_answers() {
    let result = synthesis(&two_refusals());
    assert!(refusals(&result).is_empty(), "{:#?}", refusals(&result));
    assert_eq!(
        modes_sent(scenario(&result, HALTED)),
        BTreeSet::from(["Pause".to_owned()]),
        "{:#?}",
        scenario(&result, HALTED).steps
    );
    assert_eq!(
        modes_sent(scenario(&result, FROZEN)),
        BTreeSet::from(["Freeze".to_owned()]),
        "{:#?}",
        scenario(&result, FROZEN).steps
    );
}
