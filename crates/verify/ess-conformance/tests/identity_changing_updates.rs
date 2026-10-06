//! An `updates:` whose `sets:` writes the identity re-keys the record (ess/23, beyond10x/ess#429,
//! `docs/design/identity-changing-updates.md`), in the interpreter, in synthesis and in the Rust and
//! Go runners.
//!
//! The vault: `RenameSecret {name, new_name}` re-keys `demo.vault.Secret` from `name` to `new_name`,
//! carrying `value`; `taken` refuses a new name another secret already carries, the secret's own
//! included; `no-such-secret` answers a name no secret carries. Synthesis reads the row back under
//! the new name, the old name absent, and the same request again answered `no-such-secret`; the
//! collision is witnessed on an arranged second secret and on the secret's own name, both leaving
//! every row as it was. A target that ignores the write, keeps the old row, drops a carried field,
//! renames over a colliding secret, or renames onto its own name fails a scenario.

mod support_go;

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Step, Store};
use ess_conformance::interpret::Interpreted;
use ess_conformance::scenario::{ScenarioStep, ScenarioValue, ViewExpectation};
use ess_conformance::target::{
    ConformanceTarget, DeclaredErrorValue, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RedeliveryRequest, ScenarioContext,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};
use ess_conformance::ConformanceSuite;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/identity-changing-updates.yaml");

const RENAMED: &str = "demo.vault.RenameSecret/outcome/renamed";
const TAKEN: &str = "demo.vault.RenameSecret/outcome/taken";
const NO_SUCH: &str = "demo.vault.RenameSecret/outcome/no-such-secret";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("the model parses");
    let spec = Specification::assemble([(Source::new("vault.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    assert_eq!(synthesis.refusals.len(), 0, "{:#?}", synthesis.refusals);
    synthesis.suite
}

fn steps_of(suite: &ConformanceSuite, id: &str) -> Vec<ScenarioStep> {
    suite
        .scenarios
        .iter()
        .find(|(scenario, _)| scenario.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"))
        .1
        .steps
        .clone()
}

// ---- the interpreter -----------------------------------------------------------------------------

fn name(text: &str) -> ess_domain::name::QualifiedName {
    text.parse().unwrap()
}

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn one(ir: &EssIr, store: &Store, command: &str, input: &[(&str, &str)]) -> Step {
    let input: BTreeMap<String, Node> = input
        .iter()
        .map(|(field, value)| ((*field).to_owned(), text(value)))
        .collect();
    let mut steps = execute(ir, store, &name(command), &input, &Externals::Withheld)
        .unwrap_or_else(|undetermined| panic!("{command}: {undetermined:?}"));
    assert_eq!(steps.len(), 1, "{steps:#?}");
    steps.remove(0)
}

fn taken(step: &Step) -> String {
    step.outcome
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_default()
}

fn error(step: &Step) -> Option<String> {
    step.error.as_ref().map(|error| error.error.to_string())
}

/// Two secrets, `alpha` holding `a` and `beta` holding `b`.
fn stored(ir: &EssIr) -> Store {
    let first = one(
        ir,
        &Store::default(),
        "demo.vault.StoreSecret",
        &[("name", "alpha"), ("value", "a")],
    );
    one(
        ir,
        &first.next,
        "demo.vault.StoreSecret",
        &[("name", "beta"), ("value", "b")],
    )
    .next
}

fn value_of(store: &Store, identity: &str) -> Option<Node> {
    store
        .instance_typed(&name("demo.vault.Secret"), &text(identity))
        .map(|row| row.fields.get("value").cloned().unwrap_or(Node::Null))
}

#[test]
fn rename_reads_old_identity_absent_and_new_present() {
    let ir = ir();
    let store = stored(&ir);
    let step = one(
        &ir,
        &store,
        "demo.vault.RenameSecret",
        &[("name", "alpha"), ("new_name", "gamma")],
    );
    assert_eq!(taken(&step), "demo.vault.RenameSecret/renamed", "{step:#?}");
    assert_eq!(error(&step), None);
    assert_eq!(
        value_of(&step.next, "alpha"),
        None,
        "the old identity names nothing"
    );
    assert_eq!(
        value_of(&step.next, "gamma"),
        Some(text("a")),
        "`value` is carried over"
    );
    assert_eq!(
        value_of(&step.next, "beta"),
        Some(text("b")),
        "another secret is untouched"
    );
    // The view reads the row under its new identity.
    let rows: Vec<(String, String)> = step
        .next
        .text_instances()
        .map(|(entity, identity, _)| (entity.to_string(), identity.to_owned()))
        .collect();
    assert_eq!(
        rows,
        [
            ("demo.vault.Secret".to_owned(), "beta".to_owned()),
            ("demo.vault.Secret".to_owned(), "gamma".to_owned()),
        ]
    );
}

#[test]
fn rename_old_identity_answers_unknown_instance() {
    let ir = ir();
    let store = stored(&ir);
    let renamed = one(
        &ir,
        &store,
        "demo.vault.RenameSecret",
        &[("name", "alpha"), ("new_name", "gamma")],
    );
    // The same request again: `alpha` names nothing, and existence answers before the collision
    // `gamma` would now be.
    let again = one(
        &ir,
        &renamed.next,
        "demo.vault.RenameSecret",
        &[("name", "alpha"), ("new_name", "gamma")],
    );
    assert_eq!(
        taken(&again),
        "demo.vault.RenameSecret/no-such-secret",
        "{again:#?}"
    );
    assert_eq!(error(&again).as_deref(), Some("demo.vault.NoSuchSecret"));
    assert_eq!(again.events.len(), 0);
    assert_eq!(again.next, renamed.next);
}

#[test]
fn rename_collision_takes_declared_refusal() {
    let ir = ir();
    let store = stored(&ir);
    let step = one(
        &ir,
        &store,
        "demo.vault.RenameSecret",
        &[("name", "alpha"), ("new_name", "beta")],
    );
    assert_eq!(taken(&step), "demo.vault.RenameSecret/taken", "{step:#?}");
    assert_eq!(error(&step).as_deref(), Some("demo.vault.NameTaken"));
    assert_eq!(step.events.len(), 0);
    assert_eq!(step.next, store, "both rows read unchanged");
}

#[test]
fn rename_to_own_identity_is_the_collision() {
    let ir = ir();
    let store = stored(&ir);
    let step = one(
        &ir,
        &store,
        "demo.vault.RenameSecret",
        &[("name", "alpha"), ("new_name", "alpha")],
    );
    assert_eq!(taken(&step), "demo.vault.RenameSecret/taken", "{step:#?}");
    assert_eq!(error(&step).as_deref(), Some("demo.vault.NameTaken"));
    assert_eq!(step.next, store);
}

// ---- synthesis -----------------------------------------------------------------------------------

fn position(steps: &[ScenarioStep], find: &dyn Fn(&ScenarioStep) -> bool) -> usize {
    steps
        .iter()
        .position(find)
        .unwrap_or_else(|| panic!("{steps:#?}"))
}

fn outcome_is(step: &ScenarioStep, outcome: &str) -> bool {
    matches!(step, ScenarioStep::ExpectOutcome { outcome: taken } if taken.outcome.as_str() == outcome)
}

#[test]
fn synthesis_reads_the_new_identity_and_not_the_old() {
    let steps = steps_of(&suite(), RENAMED);
    let sent = position(&steps, &|step| outcome_is(step, "renamed"));
    let ScenarioStep::ExecuteCommand { input, .. } = &steps[sent - 1] else {
        panic!("{steps:#?}")
    };
    let old = input["name"].clone();
    let new = input["new_name"].clone();
    assert!(
        matches!(old, ScenarioValue::Instance { .. }),
        "the arranged secret: {old:?}"
    );
    assert!(matches!(new, ScenarioValue::Literal { .. }), "{new:?}");
    // The row under the new identity, carrying the arranged value.
    let present = position(&steps, &|step| {
        matches!(step, ScenarioStep::ExpectView { expectation: ViewExpectation::Contains { fields }, .. }
            if fields.get("name") == Some(&new) && fields.contains_key("value"))
    });
    assert!(present > sent, "{steps:#?}");
    // No row under the old one.
    let absent = position(
        &steps,
        &|step| matches!(step, ScenarioStep::ExpectSubjectAbsent { subject, .. } if subject.get("name") == Some(&old)),
    );
    assert!(absent > sent, "{steps:#?}");
    // No expectation reads the old identity as present.
    assert!(
        !steps.iter().any(|step| matches!(step,
            ScenarioStep::ExpectView { expectation: ViewExpectation::Contains { fields }, .. }
                if fields.get("name") == Some(&old))),
        "{steps:#?}"
    );
    // The same request again takes the unknown-instance answer.
    let again = position(&steps, &|step| outcome_is(step, "no-such-secret"));
    assert!(again > sent, "{steps:#?}");
    let ScenarioStep::ExecuteCommand { input: resent, .. } = &steps[again - 1] else {
        panic!("{steps:#?}")
    };
    assert_eq!(resent, input);
}

#[test]
fn synthesis_witnesses_the_collision_on_another_row_and_on_its_own() {
    let steps = steps_of(&suite(), TAKEN);
    let sends: Vec<usize> = steps
        .iter()
        .enumerate()
        .filter(|(_, step)| outcome_is(step, "taken"))
        .map(|(at, _)| at)
        .collect();
    assert_eq!(
        sends.len(),
        2,
        "another secret's name, then its own: {steps:#?}"
    );
    let input = |at: usize| match &steps[at - 1] {
        ScenarioStep::ExecuteCommand { input, .. } => input.clone(),
        other => panic!("{other:?}"),
    };
    let (first, second) = (input(sends[0]), input(sends[1]));
    assert_ne!(first["new_name"], first["name"], "{first:?}");
    assert_eq!(second["new_name"], second["name"], "{second:?}");
    // Every whole immediate view is snapshotted before the first send and read unchanged after
    // each.
    let snapshot = position(&steps, &|step| {
        matches!(step, ScenarioStep::SnapshotView { .. })
    });
    assert!(snapshot < sends[0], "{steps:#?}");
    let unchanged: Vec<usize> = steps
        .iter()
        .enumerate()
        .filter(|(_, step)| matches!(step, ScenarioStep::ExpectViewUnchanged { .. }))
        .map(|(at, _)| at)
        .collect();
    assert!(
        unchanged.iter().any(|at| *at > sends[0] && *at < sends[1])
            && unchanged.iter().any(|at| *at > sends[1]),
        "{steps:#?}"
    );
    for send in sends {
        assert!(
            steps[send..]
                .iter()
                .any(|step| matches!(step, ScenarioStep::ExpectError { error, .. } if error.to_string() == "demo.vault.NameTaken")),
            "{steps:#?}"
        );
    }
}

#[test]
fn the_interpreter_passes_the_synthesized_suite() {
    let suite = suite();
    let verdicts = support_go::rust_outcomes(&suite, &Interpreted::for_model(ir()));
    assert_eq!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "{verdicts:#?}"
    );
    for id in [RENAMED, TAKEN, NO_SUCH] {
        assert_eq!(verdicts[id], "passed", "{id}");
    }
}

// ---- a hand-written target, healthy and wrong in one way each ------------------------------------

/// How the target renames; every other request it answers correctly.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Correct,
    /// Answers `renamed` and leaves the row under its old name.
    IgnoresTheWrite,
    /// Writes the row under the new name and keeps the old one.
    KeepsTheOldRow,
    /// Writes the row under the new name without the carried `value`.
    DropsCarriedFields,
    /// Renames over a secret that already carries the new name.
    RenamesOverACollision,
    /// Treats a rename onto the secret's own name as a successful no-op.
    RenamesOntoItself,
}

const MODES: [Mode; 6] = [
    Mode::Correct,
    Mode::IgnoresTheWrite,
    Mode::KeepsTheOldRow,
    Mode::DropsCarriedFields,
    Mode::RenamesOverACollision,
    Mode::RenamesOntoItself,
];

struct Vault {
    mode: Mode,
    rows: RefCell<BTreeMap<String, Option<String>>>,
    published: RefCell<Vec<ObservedEvent>>,
}

impl Vault {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            rows: RefCell::new(BTreeMap::new()),
            published: RefCell::new(Vec::new()),
        }
    }

    fn took(command: &str, outcome: &str) -> SemanticCommandResult {
        let mut result = SemanticCommandResult::took(ess_compiler::refs::OutcomeRef::new(
            ess_compiler::refs::CommandRef::new(command.parse().unwrap()),
            outcome.parse().unwrap(),
        ));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write").unwrap());
        result
    }

    fn refused(command: &str, outcome: &str, error: &str) -> SemanticCommandResult {
        let mut result = Self::took(command, outcome);
        result.error = Some(DeclaredErrorValue::new(error.parse().unwrap()));
        result
    }

    fn event(&self, name: &str, payload: &[(&str, &str)]) -> ObservedEvent {
        let mut event = ObservedEvent::new(name.parse().unwrap());
        for (field, value) in payload {
            event.payload.insert((*field).to_owned(), text(value));
        }
        self.published.borrow_mut().push(event.clone());
        event
    }

    fn rename(&self, old: &str, new: &str) -> SemanticCommandResult {
        const RENAME: &str = "demo.vault.RenameSecret";
        let held = self.rows.borrow().get(old).cloned();
        let Some(value) = held else {
            return Self::refused(RENAME, "no-such-secret", "demo.vault.NoSuchSecret");
        };
        let own = old == new;
        let collides = self.rows.borrow().contains_key(new);
        let refuse = match self.mode {
            Mode::RenamesOverACollision => own,
            Mode::RenamesOntoItself => collides && !own,
            _ => collides,
        };
        if refuse {
            return Self::refused(RENAME, "taken", "demo.vault.NameTaken");
        }
        {
            let mut rows = self.rows.borrow_mut();
            match self.mode {
                Mode::IgnoresTheWrite => {}
                Mode::KeepsTheOldRow => {
                    rows.insert(new.to_owned(), value);
                }
                Mode::DropsCarriedFields => {
                    rows.remove(old);
                    rows.insert(new.to_owned(), None);
                }
                _ => {
                    rows.remove(old);
                    rows.insert(new.to_owned(), value);
                }
            }
        }
        let mut result = Self::took(RENAME, "renamed");
        result.direct_events.push(self.event(
            "demo.vault.SecretRenamed",
            &[("name", old), ("new_name", new)],
        ));
        result
    }
}

impl ConformanceTarget for Vault {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "identity-changing-updates",
            "1",
        ))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        self.published.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let field = |field: &str| {
            request
                .input
                .get(field)
                .and_then(Node::as_text)
                .unwrap_or_default()
                .to_owned()
        };
        match request.command.to_string().as_str() {
            "demo.vault.StoreSecret" => {
                const STORE: &str = "demo.vault.StoreSecret";
                let name = field("name");
                if self.rows.borrow().contains_key(&name) {
                    return Ok(Self::refused(
                        STORE,
                        "already-stored",
                        "demo.vault.SecretExists",
                    ));
                }
                self.rows
                    .borrow_mut()
                    .insert(name.clone(), Some(field("value")));
                let mut result = Self::took(STORE, "stored");
                result
                    .direct_events
                    .push(self.event("demo.vault.SecretStored", &[("name", &name)]));
                Ok(result)
            }
            "demo.vault.RenameSecret" => Ok(self.rename(&field("name"), &field("new_name"))),
            other => panic!("unexpected command {other}"),
        }
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult {
            rows: self
                .rows
                .borrow()
                .iter()
                .map(|(name, value)| {
                    let mut fields = BTreeMap::from([("name".to_owned(), text(name))]);
                    if let Some(value) = value {
                        fields.insert("value".to_owned(), text(value));
                    }
                    fields
                })
                .collect(),
            total: None,
        })
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external outcomes", "unused"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
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
}

/// What a mode must fail: nothing when healthy; the rename's scenario for a fault in the rename,
/// the collision's for a fault in the collision.
fn check(label: &str, mode: Mode, verdicts: &BTreeMap<String, String>) {
    let failed = support_go::not_passed(verdicts);
    match mode {
        Mode::Correct => assert_eq!(failed, Vec::<&str>::new(), "{label}: {verdicts:#?}"),
        Mode::IgnoresTheWrite | Mode::KeepsTheOldRow | Mode::DropsCarriedFields => {
            assert_eq!(verdicts[RENAMED], "failed", "{label}: {verdicts:#?}");
        }
        Mode::RenamesOverACollision | Mode::RenamesOntoItself => {
            assert_eq!(verdicts[TAKEN], "failed", "{label}: {verdicts:#?}");
        }
    }
}

#[test]
fn the_rust_runner_fails_every_faulty_target() {
    let suite = suite();
    for mode in MODES {
        let verdicts = support_go::rust_outcomes(&suite, &Vault::new(mode));
        check(&format!("rust-{mode:?}"), mode, &verdicts);
    }
}

#[test]
fn the_go_runner_gives_the_reference_verdict_for_every_mode() {
    let suite = suite();
    for mode in MODES {
        let label = format!("go-429-{mode:?}").to_lowercase();
        let verdicts = support_go::assert_parity(&label, &suite, Vault::new(mode));
        check(&label, mode, &verdicts);
    }
}
