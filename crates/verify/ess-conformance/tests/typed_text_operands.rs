//! Typed text operands executed (beyond10x/ess#200): a view filter's `starts_with`, `ends_with` and
//! `contains` against a view parameter, `{param: <name>}`, and a command guard's — a plain `when:`, a
//! `when_subject:` and a `when_related:` predicate — against a command input, `{input: <name>}`.
//!
//! `tests/fixtures/typed-text-operands.yaml` declares all of them. The suite is synthesized from the
//! model and run against the native interpreter, which must pass it, and against targets wrong in
//! one way each, which must each fail a scenario that decides the operand, by outcome or by row:
//!
//! - one that ignores the operand: a view answers every row, a guard's test always holds;
//! - one that compares with the operand's spelling, `{param: q}`, rather than its value;
//! - one that applies the operator the wrong way round, the operand tested against the fact;
//! - one that reads an absent parameter as the empty text, which every text contains.
//!
//! The same suite and targets run through the Rust runner, the generated Go runner (by transcript
//! parity) and the generated TypeScript runner (against a JavaScript implementation with the same
//! modes), and through the browser product. No suite carries a typed text operand: a guard is
//! decided at synthesis and a filter is asserted through the rows it shows (final review decision
//! 17), so the suite keeps the format it would have had.
//! `docs/design/expression-family-source22.md`, "Typed text operands (#200)", is the design.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::consistency::QueryConsistency;
use ess_primitives::node::Node;
use ess_primitives::predicate::TextOp;

mod support_go;

const MODEL: &str = include_str!("fixtures/typed-text-operands.yaml");

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("directory.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ir() -> EssIr {
    ir_of(MODEL)
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    assert_eq!(
        synthesis
            .refusals
            .iter()
            .map(|refusal| format!("{} {}", refusal.cause.code(), refusal.cause))
            .collect::<Vec<_>>(),
        Vec::<String>::new(),
        "nothing is refused"
    );
    synthesis.suite
}

fn scenario(command: &str, outcome: &str) -> String {
    format!("directory.people.{command}/outcome/{outcome}")
}

/// Every outcome a guard reading an input decides, beside the one that creates the rows the views
/// show.
const DECIDED: [(&str, &str); 9] = [
    ("Screen", "blocked"),
    ("Screen", "spoofed"),
    ("Screen", "flagged"),
    ("Screen", "screened"),
    ("Match", "matched"),
    ("Match", "mismatch"),
    ("Dial", "wrong-number"),
    ("Dial", "dialled"),
    ("AddContact", "added"),
];

// ---- the reference decision and the faulty ones --------------------------------------------------

/// How a target decides one string operator against its operand.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Correct,
    /// The operand is not read: a view answers every row, a guard's test holds whatever was sent.
    IgnoresOperand,
    /// The operand's spelling — `{param: q}`, `{input: blocked}` — is compared, not its value.
    LiteralSpelling,
    /// The operand is tested against the fact: `q.starts_with(name)`, not `name.starts_with(q)`.
    Reversed,
    /// An absent parameter is read as the empty text, which every text begins with, ends with and
    /// contains.
    AbsentAsEmpty,
}

impl Mode {
    fn label(self) -> &'static str {
        match self {
            Self::Correct => "healthy",
            Self::IgnoresOperand => "ignores",
            Self::LiteralSpelling => "literal",
            Self::Reversed => "reversed",
            Self::AbsentAsEmpty => "absent-empty",
        }
    }
}

/// `fact <op> operand` as `mode` decides it; `None` where the operand is absent and the mode reads
/// absence as the model does, unknown.
fn test(mode: Mode, op: TextOp, fact: &str, operand: Option<&str>, spelling: &str) -> Option<bool> {
    let operand = match (operand, mode) {
        (Some(operand), _) => operand,
        (None, Mode::AbsentAsEmpty) => "",
        (None, _) => return None,
    };
    Some(match mode {
        Mode::Correct | Mode::AbsentAsEmpty => op.holds(fact, operand),
        Mode::IgnoresOperand => true,
        Mode::LiteralSpelling => op.holds(fact, spelling),
        Mode::Reversed => op.holds(operand, fact),
    })
}

fn text<'a>(input: &'a BTreeMap<String, Node>, name: &str) -> Option<&'a str> {
    match input.get(name)? {
        Node::Text(text) => Some(text),
        _ => None,
    }
}

/// One stored contact, as the target holds it.
#[derive(Clone, Debug)]
struct Contact {
    name: String,
    note: String,
    phone: String,
}

/// Whether `contact` matches the three texts `input` sends, as `mode` reads the conjunction.
fn matches(mode: Mode, contact: &Contact, input: &BTreeMap<String, Node>) -> Option<bool> {
    let initial = test(
        mode,
        TextOp::StartsWith,
        &contact.name,
        text(input, "initial"),
        "{input: initial}",
    )?;
    let tail = test(
        mode,
        TextOp::EndsWith,
        &contact.phone,
        text(input, "tail"),
        "{input: tail}",
    )?;
    let term = test(
        mode,
        TextOp::Contains,
        &contact.note,
        text(input, "term"),
        "{input: term}",
    )?;
    Some(initial && tail && term)
}

/// The outcome `mode` takes for one request, where a guard reading an input decides it.
fn decide(
    mode: Mode,
    command: &str,
    input: &BTreeMap<String, Node>,
    contacts: &BTreeMap<String, Contact>,
) -> Option<&'static str> {
    match command {
        "directory.people.Screen" => {
            let caller = text(input, "caller")?;
            if test(
                mode,
                TextOp::StartsWith,
                caller,
                text(input, "blocked"),
                "{input: blocked}",
            )? {
                return Some("blocked");
            }
            if test(
                mode,
                TextOp::EndsWith,
                caller,
                text(input, "carrier"),
                "{input: carrier}",
            )? {
                return Some("spoofed");
            }
            if test(
                mode,
                TextOp::Contains,
                caller,
                text(input, "digits"),
                "{input: digits}",
            )? {
                return Some("flagged");
            }
            Some("screened")
        }
        "directory.people.Match" => {
            let contact = contacts.get(text(input, "contact_id")?)?;
            Some(if matches(mode, contact, input)? {
                "matched"
            } else {
                "mismatch"
            })
        }
        "directory.people.Dial" => {
            let contact = contacts.get(text(input, "contact_id")?)?;
            Some(if matches(mode, contact, input)? {
                "dialled"
            } else {
                "wrong-number"
            })
        }
        _ => None,
    }
}

/// A text no witness begins with, ends with or contains.
const NOWHERE: &str = "\u{2603}\u{2603}";

/// `input` moved so that the model takes `outcome` for it: the interpreter answers for the branch a
/// faulty target decided.
fn toward(
    mut input: BTreeMap<String, Node>,
    outcome: &str,
    contacts: &BTreeMap<String, Contact>,
) -> BTreeMap<String, Node> {
    let set = |input: &mut BTreeMap<String, Node>, name: &str, value: &str| {
        input.insert(name.to_owned(), Node::Text(value.to_owned()));
    };
    let caller = text(&input, "caller").unwrap_or_default().to_owned();
    match outcome {
        "blocked" => set(&mut input, "blocked", &caller),
        "spoofed" => {
            set(&mut input, "blocked", NOWHERE);
            set(&mut input, "carrier", &caller);
        }
        "flagged" => {
            set(&mut input, "blocked", NOWHERE);
            set(&mut input, "carrier", NOWHERE);
            set(&mut input, "digits", &caller);
        }
        "screened" => {
            for name in ["blocked", "carrier", "digits"] {
                set(&mut input, name, NOWHERE);
            }
        }
        "matched" | "dialled" => {
            let contact = text(&input, "contact_id")
                .and_then(|id| contacts.get(id))
                .cloned();
            if let Some(contact) = contact {
                set(&mut input, "initial", &contact.name);
                set(&mut input, "tail", &contact.phone);
                set(&mut input, "term", &contact.note);
            }
        }
        "mismatch" | "wrong-number" => set(&mut input, "initial", NOWHERE),
        _ => {}
    }
    input
}

/// The interpreter, with every typed text operand decided by `mode`.
struct Directory {
    inner: Interpreted,
    mode: Mode,
    contacts: RefCell<BTreeMap<String, Contact>>,
}

impl Directory {
    fn new(mode: Mode) -> Self {
        Self {
            inner: Interpreted::for_model(ir()),
            mode,
            contacts: RefCell::default(),
        }
    }
}

/// The view each filtered view reads its rows from, the field its filter reads, the operator, the
/// parameter and its spelling.
fn filtered(view: &str) -> Option<(&'static str, TextOp, &'static str)> {
    match view {
        "directory.people.ByName" => Some(("name", TextOp::StartsWith, "q")),
        "directory.people.ByPhone" => Some(("phone", TextOp::EndsWith, "tail")),
        "directory.people.ByNote" => Some(("note", TextOp::Contains, "term")),
        _ => None,
    }
}

impl ConformanceTarget for Directory {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.contacts.borrow_mut().clear();
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command = request.command.to_string();
        {
            let contacts = self.contacts.borrow();
            if let (Some(correct), Some(faulty)) = (
                decide(Mode::Correct, &command, &request.input, &contacts),
                decide(self.mode, &command, &request.input, &contacts),
            ) {
                if correct != faulty {
                    request.input = toward(request.input, faulty, &contacts);
                }
            }
        }
        let input = request.input.clone();
        let result = self.inner.execute_command(request)?;
        if command == "directory.people.AddContact" {
            let id = result.direct_events.iter().find_map(|event| {
                match event.payload.get("contact_id") {
                    Some(Node::Text(id)) => Some(id.clone()),
                    _ => None,
                }
            });
            if let Some(id) = id {
                self.contacts.borrow_mut().insert(
                    id,
                    Contact {
                        name: text(&input, "name").unwrap_or_default().to_owned(),
                        note: text(&input, "note").unwrap_or_default().to_owned(),
                        phone: text(&input, "phone").unwrap_or_default().to_owned(),
                    },
                );
            }
        }
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let Some((field, op, param)) = filtered(&request.view.to_string()) else {
            return self.inner.query_view(request);
        };
        if self.mode == Mode::Correct {
            return self.inner.query_view(request);
        }
        let operand = match request.params.get(param) {
            Some(Node::Text(text)) => Some(text.clone()),
            _ => None,
        };
        let spelling = format!("{{param: {param}}}");
        let every = self.inner.query_view(SemanticViewRequest {
            view: "directory.people.Contacts".parse().expect("a view"),
            params: BTreeMap::new(),
            ..request
        })?;
        Ok(SemanticViewResult::of(
            every
                .rows
                .into_iter()
                .filter(|row| match row.get(field) {
                    Some(Node::Text(fact)) => {
                        test(self.mode, op, fact, operand.as_deref(), &spelling) == Some(true)
                    }
                    _ => false,
                })
                .map(|row| {
                    row.into_iter()
                        .filter(|(name, _)| name == "contact_id" || name == field)
                        .collect::<ViewRow>()
                }),
        ))
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(r)
    }
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

/// Every view read in `id`, with the parameters it sends.
fn reads(suite: &ConformanceSuite, id: &str) -> Vec<(String, BTreeMap<String, ScenarioValue>)> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario);
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::QueryView { view, params } => Some((view.to_string(), params.clone())),
            ScenarioStep::EventuallyView { view, params, .. } => {
                Some((view.to_string(), params.clone()))
            }
            _ => None,
        })
        .collect()
}

// ---- synthesis and the interpreter ---------------------------------------------------------------

#[test]
fn t200_every_branch_is_witnessed_and_every_view_is_read_with_its_parameter() {
    let suite = suite();
    for (command, outcome) in DECIDED {
        let id = scenario(command, outcome);
        assert!(
            suite.scenarios.keys().any(|key| key.to_string() == id),
            "{id} is synthesized: {:#?}",
            suite
                .scenarios
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
    }
    let read = reads(&suite, &scenario("AddContact", "added"));
    for (view, param) in [
        ("directory.people.ByName", "q"),
        ("directory.people.ByPhone", "tail"),
        ("directory.people.ByNote", "term"),
    ] {
        assert!(
            read.iter().any(|(name, params)| name == view
                && matches!(
                    params.get(param).and_then(ScenarioValue::as_literal),
                    Some(Node::Text(text)) if !text.is_empty()
                )),
            "{view} is read with `{param}`: {read:#?}"
        );
    }
    // The optional parameter is also left out, which shows no row.
    assert!(
        read.iter()
            .any(|(name, params)| name == "directory.people.ByPhone" && params.is_empty()),
        "ByPhone is read without its optional parameter: {read:#?}"
    );
}

/// The first input `id` sends `command` with, as texts.
fn first_sent(suite: &ConformanceSuite, id: &str, command: &str) -> BTreeMap<String, String> {
    inputs(suite, id, command)
        .into_iter()
        .next()
        .unwrap_or_else(|| panic!("{id} sends {command}"))
}

/// The last input `id` sends `command` with, as texts.
fn sent(suite: &ConformanceSuite, id: &str, command: &str) -> BTreeMap<String, String> {
    inputs(suite, id, command)
        .into_iter()
        .last()
        .unwrap_or_else(|| panic!("{id} sends {command}"))
}

/// Every input `id` sends `command` with, as texts, in step order.
fn inputs(suite: &ConformanceSuite, id: &str, command: &str) -> Vec<BTreeMap<String, String>> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario);
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(
                input
                    .iter()
                    .filter_map(|(name, value)| match value.as_literal() {
                        Some(Node::Text(text)) => Some((name.clone(), text.clone())),
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        })
        .collect()
}

#[test]
fn t200_a_guard_is_witnessed_by_a_proper_part_and_refuted_at_the_deciding_character() {
    let suite = suite();
    let screen = |outcome: &str| {
        sent(
            &suite,
            &scenario("Screen", outcome),
            "directory.people.Screen",
        )
    };
    // Each satisfying operand is a proper part of the caller, so the operator tested the wrong way
    // round refuses it.
    let blocked = screen("blocked");
    assert!(
        blocked["caller"].starts_with(&blocked["blocked"])
            && blocked["caller"] != blocked["blocked"],
        "{blocked:?}"
    );
    let spoofed = screen("spoofed");
    assert!(
        spoofed["caller"].ends_with(&spoofed["carrier"]) && spoofed["caller"] != spoofed["carrier"],
        "{spoofed:?}"
    );
    let flagged = screen("flagged");
    assert!(
        flagged["caller"].contains(&flagged["digits"])
            && !flagged["caller"].starts_with(&flagged["digits"])
            && !flagged["caller"].ends_with(&flagged["digits"]),
        "{flagged:?}"
    );
    // The refuting branch is sent an operand one character from a prefix of the caller.
    let screened = screen("screened");
    let (caller, operand) = (&screened["caller"], &screened["blocked"]);
    let shared = caller
        .chars()
        .zip(operand.chars())
        .take_while(|(left, right)| left == right)
        .count();
    assert_eq!(
        shared + 1,
        operand.chars().count(),
        "`{operand}` differs from a prefix of `{caller}` at its last character only"
    );
    // A stored row's guard: the matched branch sends parts of the row the contact was created with,
    // the first one the scenario creates; the rows after it are the views' further rows.
    let created = first_sent(
        &suite,
        &scenario("Match", "matched"),
        "directory.people.AddContact",
    );
    let matched = sent(
        &suite,
        &scenario("Match", "matched"),
        "directory.people.Match",
    );
    assert!(
        created["name"].starts_with(&matched["initial"])
            && created["name"] != matched["initial"]
            && created["phone"].ends_with(&matched["tail"])
            && created["note"].contains(&matched["term"]),
        "{created:?} {matched:?}"
    );
}

#[test]
fn t200_the_interpreter_passes_the_suite() {
    let suite = suite();
    let statuses = run(&suite, &Interpreted::for_model(ir()));
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
    for (command, outcome) in DECIDED {
        assert_eq!(
            statuses.get(&scenario(command, outcome)),
            Some(&Status::Passed),
            "{statuses:#?}"
        );
    }
}

fn caught(mode: Mode, deciding: &[&str]) {
    let suite = suite();
    let healthy = run(&suite, &Directory::new(Mode::Correct));
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    let failing = not_passed(&run(&suite, &Directory::new(mode)));
    for prefix in deciding {
        assert!(
            failing.iter().any(|line| line.starts_with(prefix)),
            "{mode:?} fails a scenario of {prefix}: {failing:#?}"
        );
    }
    assert!(
        failing.iter().all(|line| line.contains("Failed")),
        "{mode:?} fails by outcome or row, not by setup or admission: {failing:#?}"
    );
}

#[test]
fn t200_a_target_ignoring_the_operand_fails() {
    caught(
        Mode::IgnoresOperand,
        &[
            "directory.people.AddContact/outcome/added",
            "directory.people.Screen/outcome/",
            "directory.people.Match/outcome/",
            "directory.people.Dial/outcome/",
        ],
    );
}

#[test]
fn t200_a_target_comparing_with_the_operands_spelling_fails() {
    caught(
        Mode::LiteralSpelling,
        &[
            "directory.people.AddContact/outcome/added",
            "directory.people.Screen/outcome/",
            "directory.people.Match/outcome/",
            "directory.people.Dial/outcome/",
        ],
    );
}

#[test]
fn t200_a_target_applying_the_operator_the_wrong_way_round_fails() {
    caught(
        Mode::Reversed,
        &[
            "directory.people.AddContact/outcome/added",
            "directory.people.Screen/outcome/",
            "directory.people.Match/outcome/",
            "directory.people.Dial/outcome/",
        ],
    );
}

#[test]
fn t200_a_target_reading_an_absent_parameter_as_empty_text_fails() {
    caught(
        Mode::AbsentAsEmpty,
        &["directory.people.AddContact/outcome/added"],
    );
}

// ---- the interpreter, one request at a time ------------------------------------------------------

fn context(id: &str) -> ScenarioContext {
    ScenarioContext::new(
        ess_conformance::ScenarioId::parse(id).expect("a scenario id"),
        ess_primitives::ids::CorrelationId::new("t200-direct").expect("an id"),
    )
}

fn command(target: &Interpreted, name: &str, input: &[(&str, Option<&str>)]) -> String {
    match target.execute_command(SemanticCommandRequest {
        command: ess_compiler::refs::CommandRef::new(
            format!("directory.people.{name}").parse().unwrap(),
        ),
        actor: None,
        caller: None,
        input: input
            .iter()
            .filter_map(|(field, value)| {
                value.map(|value| ((*field).to_owned(), Node::Text(value.to_owned())))
            })
            .collect(),
        correlation: ess_primitives::ids::CorrelationId::new("t200-direct").expect("an id"),
    }) {
        Ok(result) => result
            .outcome
            .map_or_else(|| "none".to_owned(), |outcome| outcome.outcome.to_string()),
        Err(error) => format!("{error:?}"),
    }
}

fn view(target: &Interpreted, name: &str, params: &[(&str, &str)]) -> Vec<String> {
    target
        .query_view(SemanticViewRequest {
            view: format!("directory.people.{name}").parse().unwrap(),
            params: params
                .iter()
                .map(|(param, value)| ((*param).to_owned(), Node::Text((*value).to_owned())))
                .collect(),
            consistency: QueryConsistency::Current,
            correlation: ess_primitives::ids::CorrelationId::new("t200-direct").expect("an id"),
            deadline: Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(0)),
        })
        .unwrap_or_else(|error| panic!("{name}: {error:?}"))
        .rows
        .iter()
        .map(|row| match row.get("contact_id") {
            Some(Node::Text(id)) => id.clone(),
            other => panic!("no identity: {other:?}"),
        })
        .collect()
}

#[test]
fn t200_the_interpreter_decides_each_operator_byte_wise_and_case_sensitively() {
    let target = Interpreted::for_model(ir());
    target
        .begin_scenario(&context(&scenario("Screen", "blocked")))
        .expect("begins");
    let screen = |blocked: &str, carrier: Option<&str>, digits: &str| {
        command(
            &target,
            "Screen",
            &[
                ("caller", Some("+44 20 7946 0000")),
                ("blocked", Some(blocked)),
                ("carrier", carrier),
                ("digits", Some(digits)),
            ],
        )
    };
    assert_eq!(screen("+44", Some("x"), "x"), "blocked");
    assert_eq!(screen("+45", Some("0000"), "x"), "spoofed");
    assert_eq!(screen("+45", Some("1000"), "7946"), "flagged");
    assert_eq!(screen("+45", Some("1000"), "7947"), "screened");
    // The empty text is a prefix of every text, byte for byte.
    assert_eq!(screen("", Some("x"), "x"), "blocked");
    // A prefix longer than the caller is no prefix of it.
    assert_eq!(screen("+44 20 7946 0000 1", Some("1"), "x"), "screened");

    // Rows: the parameter is read case-sensitively, an empty one shows every row and an absent
    // optional one shows none.
    for (name, note, phone) in [
        ("Ada", "met at the fair", "+44 1"),
        ("adam", "owes a call", "+33 1"),
    ] {
        command(
            &target,
            "AddContact",
            &[
                ("name", Some(name)),
                ("note", Some(note)),
                ("phone", Some(phone)),
            ],
        );
    }
    assert_eq!(view(&target, "ByName", &[("q", "Ada")]).len(), 1);
    assert_eq!(view(&target, "ByName", &[("q", "ada")]).len(), 1);
    assert_eq!(view(&target, "ByName", &[("q", "ADA")]).len(), 0);
    assert_eq!(view(&target, "ByName", &[("q", "")]).len(), 2);
    assert_eq!(view(&target, "ByNote", &[("term", "a")]).len(), 2);
    assert_eq!(view(&target, "ByNote", &[("term", "fair")]).len(), 1);
    assert_eq!(view(&target, "ByPhone", &[("tail", " 1")]).len(), 2);
    assert_eq!(view(&target, "ByPhone", &[("tail", "+44 1")]).len(), 1);
    assert_eq!(view(&target, "ByPhone", &[]).len(), 0);
}

// ---- the suite format --------------------------------------------------------------------------

#[test]
fn t200_no_suite_carries_a_typed_text_operand_and_the_format_is_kept() {
    let suite = suite();
    let json = suite.to_canonical_json().expect("admitted");
    for spelling in [r#"{"param""#, r#"{"input""#] {
        assert!(!json.contains(spelling), "{spelling} in {json}");
    }
    assert!(!ess_conformance::expression_format::used_by(&suite));
    let major = suite.provenance.suite_version.major();
    assert!(major < 40, "{}", suite.provenance.suite_version);
    AdmittedSuite::from_json(&json).expect("its own reader admits it");
}

// ---- the Go and TypeScript runners -------------------------------------------------------------

const FAULTS: [Mode; 4] = [
    Mode::IgnoresOperand,
    Mode::LiteralSpelling,
    Mode::Reversed,
    Mode::AbsentAsEmpty,
];

#[test]
fn t200_go_runs_the_suite_at_parity_and_fails_every_faulty_target() {
    let suite = suite();
    let healthy = support_go::assert_parity("t200-healthy", &suite, Directory::new(Mode::Correct));
    assert_eq!(support_go::not_passed(&healthy), Vec::<&str>::new());
    for mode in FAULTS {
        let verdicts = support_go::assert_parity(
            &format!("t200-{}", mode.label()),
            &suite,
            Directory::new(mode),
        );
        assert!(
            support_go::not_passed(&verdicts)
                .iter()
                .any(|id| id.starts_with("directory.people.AddContact/outcome/added")),
            "{mode:?}: {verdicts:#?}"
        );
    }
}

/// A JavaScript implementation of the directory, deciding every typed text operand as
/// `ESS_TARGET_MODE` says: `healthy`, `ignores`, `literal`, `reversed` or `absent-empty`.
const JS_TARGET: &str = r"const mode = process.env.ESS_TARGET_MODE ?? 'healthy';
const holds = (op, text, needle) =>
  op === 'starts_with' ? text.startsWith(needle) : op === 'ends_with' ? text.endsWith(needle) : text.includes(needle);
const test = (op, fact, operand, spelling) => {
  if (operand === undefined || operand === null) {
    if (mode !== 'absent-empty') return undefined;
    operand = '';
  }
  switch (mode) {
    case 'ignores': return true;
    case 'literal': return holds(op, fact, spelling);
    case 'reversed': return holds(op, operand, fact);
    default: return holds(op, fact, operand);
  }
};
const matches = (row, input) =>
  test('starts_with', row.name, input.initial, '{input: initial}') &&
  test('ends_with', row.phone, input.tail, '{input: tail}') &&
  test('contains', row.note, input.term, '{input: term}');
const views = {
  'directory.people.ByName': ['name', 'starts_with', 'q'],
  'directory.people.ByPhone': ['phone', 'ends_with', 'tail'],
  'directory.people.ByNote': ['note', 'contains', 'term'],
};
class Directory {
  rows = [];
  seq = 0;
  identity() { return { name: 'directory', version: '1' }; }
  beginScenario() { this.rows = []; }
  endScenario() {}
  answer(outcome, error) { return { outcome, consistency: `seq:${this.seq}`, error }; }
  accept(outcome, event, payload) {
    this.seq += 1;
    return { outcome, consistency: `seq:${this.seq}`, directEvents: [{ event, payload }] };
  }
  executeCommand({ command, input }) {
    switch (command) {
      case 'directory.people.AddContact': {
        this.seq += 1;
        const id = `00000000-0000-4000-8000-${String(this.seq).padStart(12, '0')}`;
        this.rows.push({ contact_id: id, state: 'Listed', name: input.name, note: input.note, phone: input.phone });
        return {
          outcome: 'added',
          consistency: `seq:${this.seq}`,
          directEvents: [{ event: 'directory.people.ContactAdded', payload: { contact_id: id } }],
        };
      }
      case 'directory.people.Screen': {
        const caller = input.caller;
        if (test('starts_with', caller, input.blocked, '{input: blocked}')) return this.answer('blocked', 'directory.people.Blocked');
        const spoofed = test('ends_with', caller, input.carrier, '{input: carrier}');
        if (spoofed === undefined) throw new Error('an absent carrier is unknown');
        if (spoofed) return this.answer('spoofed', 'directory.people.Spoofed');
        if (test('contains', caller, input.digits, '{input: digits}')) return this.answer('flagged', 'directory.people.Flagged');
        return this.accept('screened', 'directory.people.CallScreened', { caller });
      }
      case 'directory.people.Match': {
        const row = this.rows.find((held) => held.contact_id === input.contact_id);
        if (!row) return this.answer('unknown-contact', 'directory.people.UnknownContact');
        if (matches(row, input)) return this.accept('matched', 'directory.people.ContactMatched', { contact_id: input.contact_id });
        return this.answer('mismatch', 'directory.people.Mismatch');
      }
      case 'directory.people.Dial': {
        const row = this.rows.find((held) => held.contact_id === input.contact_id);
        if (!row) return this.answer('no-contact', 'directory.people.NoContact');
        if (!matches(row, input)) return this.answer('wrong-number', 'directory.people.WrongNumber');
        return this.accept('dialled', 'directory.people.Dialled', { contact_id: input.contact_id });
      }
      default: throw new Error(`unexpected ${command}`);
    }
  }
  queryView({ view, params }) {
    if (view === 'directory.people.Contacts') return { rows: this.rows.map((row) => ({ ...row })) };
    const [field, op, param] = views[view];
    return {
      rows: this.rows
        .filter((row) => test(op, row[field], params?.[param], `{param: ${param}}`) === true)
        .map((row) => ({ contact_id: row.contact_id, [field]: row[field] })),
    };
  }
  observeEvents() { throw new Error('unused'); }
  configureExternalOutcome() { throw new Error('nothing is external'); }
  redeliverEvent() { throw new Error('no bindings'); }
}
export function makeTarget() { return new Directory(); }
";

/// The TypeScript runner's verdicts for the suite against [`JS_TARGET`] in `mode`; `None` where
/// `tsc` or `node` is missing.
fn typescript_verdicts(mode: &str) -> Option<BTreeMap<String, String>> {
    use std::process::Command;
    for name in ["tsc", "node"] {
        if !Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            println!("skipped: no `{name}` on PATH");
            return None;
        }
    }
    let admitted = AdmittedSuite::from_suite(&suite()).expect("admits");
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("typed-text-operands")
        .join(format!("{mode}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for artifact in ess_conformance::ts::emit(admitted.suite()).expect("the package emits") {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
        std::fs::write(path, artifact.contents).expect("writes");
    }
    let dir = root.join(ess_conformance::ts::PACKAGE);
    for (name, contents) in [
        ("target.mjs", JS_TARGET),
        (
            "driver.mjs",
            include_str!("fixtures/typescript-parity-driver.mjs"),
        ),
        (
            "runtime-test.tsconfig.json",
            r#"{"extends":"./tsconfig.json","compilerOptions":{"types":[],"noCheck":true}}"#,
        ),
    ] {
        std::fs::write(dir.join(name), contents).expect("writes");
    }
    let compiled = Command::new("tsc")
        .args(["--project", "runtime-test.tsconfig.json"])
        .current_dir(&dir)
        .output()
        .expect("tsc runs");
    assert!(
        compiled.status.success(),
        "{}{}",
        String::from_utf8_lossy(&compiled.stdout),
        String::from_utf8_lossy(&compiled.stderr)
    );
    let report = dir.join("report.json");
    let output = Command::new("node")
        .args(["--test", "driver.mjs"])
        .env("ESS_TARGET_MODE", mode)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .current_dir(&dir)
        .output()
        .expect("node runs");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::fs::read_to_string(&report)
        .unwrap_or_else(|_| panic!("the run wrote no report:\n{printed}"));
    let document: serde_json::Value = serde_json::from_str(&text).expect("report/2 is JSON");
    let mut verdicts = BTreeMap::new();
    for (status, ids) in document["outcomes"].as_object().expect("outcomes") {
        for id in ids.as_array().expect("a list") {
            verdicts.insert(id.as_str().expect("an id").to_owned(), status.clone());
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    Some(verdicts)
}

#[test]
fn t200_typescript_runs_the_suite_and_fails_every_faulty_target() {
    let Some(healthy) = typescript_verdicts(Mode::Correct.label()) else {
        return;
    };
    let failing: Vec<&String> = healthy
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id)
        .collect();
    assert_eq!(failing, Vec::<&String>::new(), "{healthy:#?}");
    for (command, outcome) in DECIDED {
        assert_eq!(
            healthy.get(&scenario(command, outcome)).map(String::as_str),
            Some("passed"),
            "{healthy:#?}"
        );
    }
    for mode in FAULTS {
        let verdicts = typescript_verdicts(mode.label()).expect("tools were found once");
        assert!(
            verdicts.iter().any(|(id, status)| id
                .starts_with("directory.people.AddContact/outcome/added")
                && status == "failed"),
            "{mode:?}: {verdicts:#?}"
        );
        if mode != Mode::AbsentAsEmpty {
            for command in ["Screen", "Match", "Dial"] {
                assert!(
                    verdicts.iter().any(|(id, status)| id
                        .starts_with(&format!("directory.people.{command}/outcome/"))
                        && status == "failed"),
                    "{mode:?} fails a {command} scenario: {verdicts:#?}"
                );
            }
        }
    }
}

// ---- the browser product ---------------------------------------------------------------------

/// The browser product installs the interpreter, as an adopter installs their implementation.
struct Browser;

impl ess_conformance::web_execution::Installation for Browser {
    type Target = Interpreted;
    type Clock = ess_conformance::AdvancingClock;
    fn create(
        _: &ess_conformance::web_execution::RunContext,
    ) -> ess_conformance::web_execution::Result<
        ess_conformance::web_execution::Installed<Self::Target, Self::Clock>,
    > {
        Ok(ess_conformance::web_execution::Installed {
            target: Interpreted::for_model(ir()),
            clock: ess_conformance::AdvancingClock::default(),
            config: ess_conformance::RunnerConfig::default(),
        })
    }
}

#[test]
fn t200_the_browser_product_runs_the_suite() {
    use ess_conformance::web_execution::{bundle, Product};
    let admitted = AdmittedSuite::from_suite(&suite()).expect("admits");
    let (manifest, blobs) = bundle::create(
        &[bundle::SourceDocument {
            path: "sources/0000.yaml".into(),
            text: MODEL.into(),
        }],
        &bundle::Execution::Ordinary(admitted),
    )
    .expect("bundles");
    let mut product = Product::<Browser>::new();
    let handle = product.load(&manifest, blobs).expect("loads");
    let completed = product.run(handle, [7; 16], 1).expect("runs");
    let report: serde_json::Value = serde_json::from_str(&completed.report).expect("JSON");
    let outcomes = report["outcomes"].as_object().expect("outcomes");
    for (status, ids) in outcomes {
        if status != "passed" {
            assert_eq!(
                ids.as_array().map(Vec::len),
                Some(0),
                "{status}: {report:#}"
            );
        }
    }
    let passed = outcomes["passed"].to_string();
    assert!(
        passed.contains("directory.people.Screen/outcome/blocked")
            && passed.contains("directory.people.AddContact/outcome/added"),
        "{passed}"
    );
}
