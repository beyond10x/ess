//! Adversary pass 1 against E-U9, typed text operands `{param: <name>}` / `{input: <name>}`
//! (beyond10x/ess#200).
//!
//! Two families of case:
//!
//! - faulty targets the unit's own suite does not model, run against the suite synthesized from the
//!   unit's fixture (`tests/fixtures/typed-text-operands.yaml`): an operator swapped for another, an
//!   operator that folds ASCII case, a guard reading a different input of the same command, and a
//!   "search" implementation applying `contains` for every operator;
//! - small variant models at sites and shapes the unit's fixture does not reach: a negated filter, a
//!   filter over a nested struct field, a measure's `where:` (an admitted `{param:}` site), with a
//!   required and an Optional parameter, and a `when_subject:` over a nested stored field. For each,
//!   the interpreter must pass the suite synthesized for it, and a target that ignores the operand
//!   (emulated by sending the empty text, which every text begins with, ends with and contains) must
//!   fail a scenario, unless synthesis refused the construct by name.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use ess_primitives::predicate::TextOp;

const FIXTURE: &str = include_str!("fixtures/typed-text-operands.yaml");

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals_of(ir: &EssIr) -> Vec<String> {
    ess_conformance::synthesize::synthesize(ir)
        .refusals
        .iter()
        .map(|refusal| format!("{} {}", refusal.cause.code(), refusal.cause))
        .collect()
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

fn text<'a>(input: &'a BTreeMap<String, Node>, name: &str) -> Option<&'a str> {
    match input.get(name)? {
        Node::Text(text) => Some(text),
        _ => None,
    }
}

// ---- part 1: further faulty targets against the unit's own fixture --------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Correct,
    /// `starts_with` decided as `ends_with`, `ends_with` as `starts_with`, `contains` as
    /// `starts_with`.
    SwappedOperator,
    /// Both sides compared after ASCII lower-casing.
    FoldsCase,
    /// Each guard reads a different input of the same command; views are correct.
    WrongInput,
    /// Every operator decided as `contains` (a "search" box implemented as `LIKE %q%`).
    ContainsForAll,
}

fn holds(mode: Mode, op: TextOp, fact: &str, operand: &str) -> bool {
    match mode {
        Mode::Correct | Mode::WrongInput => op.holds(fact, operand),
        Mode::SwappedOperator => {
            let swapped = match op {
                TextOp::StartsWith => TextOp::EndsWith,
                TextOp::EndsWith | TextOp::Contains => TextOp::StartsWith,
            };
            swapped.holds(fact, operand)
        }
        Mode::FoldsCase => op.holds(&fact.to_ascii_lowercase(), &operand.to_ascii_lowercase()),
        Mode::ContainsForAll => fact.contains(operand),
    }
}

/// `fact <op> input[own]`, or `input[wrong]` in [`Mode::WrongInput`]; `None` when that input is
/// absent.
fn test(
    mode: Mode,
    op: TextOp,
    fact: &str,
    input: &BTreeMap<String, Node>,
    own: &str,
    wrong: &str,
) -> Option<bool> {
    let name = if mode == Mode::WrongInput { wrong } else { own };
    let operand = text(input, name)?;
    Some(holds(mode, op, fact, operand))
}

#[derive(Clone, Debug)]
struct Contact {
    name: String,
    note: String,
    phone: String,
}

fn matches(mode: Mode, contact: &Contact, input: &BTreeMap<String, Node>) -> Option<bool> {
    let initial = test(
        mode,
        TextOp::StartsWith,
        &contact.name,
        input,
        "initial",
        "term",
    )?;
    let tail = test(
        mode,
        TextOp::EndsWith,
        &contact.phone,
        input,
        "tail",
        "initial",
    )?;
    let term = test(mode, TextOp::Contains, &contact.note, input, "term", "tail")?;
    Some(initial && tail && term)
}

fn decide(
    mode: Mode,
    command: &str,
    input: &BTreeMap<String, Node>,
    contacts: &BTreeMap<String, Contact>,
) -> Option<&'static str> {
    match command {
        "directory.people.Screen" => {
            let caller = text(input, "caller")?;
            if test(mode, TextOp::StartsWith, caller, input, "blocked", "digits")? {
                return Some("blocked");
            }
            if test(mode, TextOp::EndsWith, caller, input, "carrier", "blocked")? {
                return Some("spoofed");
            }
            if test(mode, TextOp::Contains, caller, input, "digits", "blocked")? {
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

/// `input` moved so that the model takes `outcome` for it.
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

struct Directory {
    inner: Interpreted,
    mode: Mode,
    contacts: RefCell<BTreeMap<String, Contact>>,
}

impl Directory {
    fn new(mode: Mode) -> Self {
        Self {
            inner: Interpreted::for_model(ir_of(FIXTURE)),
            mode,
            contacts: RefCell::default(),
        }
    }
}

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
        if matches!(self.mode, Mode::Correct | Mode::WrongInput) {
            return self.inner.query_view(request);
        }
        let operand = match request.params.get(param) {
            Some(Node::Text(text)) => Some(text.clone()),
            _ => None,
        };
        let every = self.inner.query_view(SemanticViewRequest {
            view: "directory.people.Contacts".parse().expect("a view"),
            params: BTreeMap::new(),
            ..request
        })?;
        let mode = self.mode;
        Ok(SemanticViewResult::of(
            every
                .rows
                .into_iter()
                .filter(|row| match (row.get(field), operand.as_deref()) {
                    (Some(Node::Text(fact)), Some(operand)) => holds(mode, op, fact, operand),
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

fn fixture_suite() -> ConformanceSuite {
    let ir = ir_of(FIXTURE);
    assert_eq!(refusals_of(&ir), Vec::<String>::new(), "nothing refused");
    ess_conformance::synthesize::synthesize(&ir).suite
}

fn caught(mode: Mode, deciding: &[&str]) {
    let suite = fixture_suite();
    let healthy = run(&suite, &Directory::new(Mode::Correct));
    assert_eq!(
        not_passed(&healthy),
        Vec::<String>::new(),
        "healthy control"
    );
    let failing = not_passed(&run(&suite, &Directory::new(mode)));
    let missed: Vec<&&str> = deciding
        .iter()
        .filter(|prefix| !failing.iter().any(|line| line.starts_with(**prefix)))
        .collect();
    assert_eq!(
        missed,
        Vec::<&&str>::new(),
        "{mode:?}: no failing scenario under these prefixes; failing: {failing:#?}"
    );
}

const VIEWS_AND_GUARDS: [&str; 4] = [
    "directory.people.AddContact/outcome/added",
    "directory.people.Screen/outcome/",
    "directory.people.Match/outcome/",
    "directory.people.Dial/outcome/",
];

#[test]
fn adv_u9_a_target_swapping_the_operator_fails() {
    caught(Mode::SwappedOperator, &VIEWS_AND_GUARDS);
}

#[test]
#[ignore = "follow-up: guard text witnesses need case- and position-deciding rows"]
fn adv_u9_a_target_folding_ascii_case_fails() {
    caught(Mode::FoldsCase, &VIEWS_AND_GUARDS);
}

#[test]
fn adv_u9_a_guard_reading_another_input_fails() {
    caught(
        Mode::WrongInput,
        &[
            "directory.people.Screen/outcome/",
            "directory.people.Match/outcome/",
            "directory.people.Dial/outcome/",
        ],
    );
}

#[test]
#[ignore = "follow-up: guard text witnesses need case- and position-deciding rows"]
fn adv_u9_a_target_deciding_every_operator_as_contains_fails() {
    caught(Mode::ContainsForAll, &VIEWS_AND_GUARDS);
}

// ---- part 2: variant models, a target that ignores one operand --------------------------------

/// The interpreter with the operand of one view parameter or one command input replaced by the
/// empty text before it decides: every text begins with, ends with and contains it, so the string
/// operator under it holds of every observed fact — the operand ignored.
struct Blanked {
    inner: Interpreted,
    views: Vec<(&'static str, &'static str)>,
    commands: Vec<(&'static str, &'static str)>,
}

impl ConformanceTarget for Blanked {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.inner.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
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
        for (name, field) in &self.commands {
            if command == *name && request.input.contains_key(*field) {
                request
                    .input
                    .insert((*field).to_owned(), Node::Text(String::new()));
            }
        }
        self.inner.execute_command(request)
    }
    fn query_view(
        &self,
        mut request: SemanticViewRequest,
    ) -> Result<SemanticViewResult, TargetError> {
        let view = request.view.to_string();
        for (name, param) in &self.views {
            if view == *name {
                request
                    .params
                    .insert((*param).to_owned(), Node::Text(String::new()));
            }
        }
        self.inner.query_view(request)
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

const SHELF: &str = r"format: ess/22
system: shelf
version: v1
domain: shelf.items
types:
  - {name: shelf.items.ItemId, kind: newtype, of: Uuid}
  - name: shelf.items.Profile
    kind: struct
    fields:
      - {name: title, type: String}
entities:
  - name: shelf.items.Item
    identity: {name: item_id, type: shelf.items.ItemId}
    fields:
      - {name: team, type: String}
      - {name: label, type: String}
      - {name: profile, type: shelf.items.Profile}
    lifecycle: {initial: Listed, states: [Listed], terminal: [Listed]}
errors:
  - {name: shelf.items.Refused, summary: The item is refused., fields: []}
  - {name: shelf.items.UnknownItem, summary: No item carries the identity., fields: []}
events:
  - name: shelf.items.ItemAdded
    fields:
      - {name: item_id, type: shelf.items.ItemId}
  - name: shelf.items.Checked
    fields:
      - {name: item_id, type: shelf.items.ItemId}
  - name: shelf.items.Tagged
    fields:
      - {name: label, type: String}
commands:
  - name: shelf.items.AddItem
    input:
      - {name: team, type: String}
      - {name: label, type: String}
      - {name: profile, type: shelf.items.Profile}
    outcomes:
      - name: added
        creates: shelf.items.Item
        instance: item_id
        sets: {team: input.team, label: input.label, profile: input.profile}
        emits: [shelf.items.ItemAdded]
        payload:
          shelf.items.ItemAdded: {item_id: {generated: true}}
  - name: shelf.items.Check
    input:
      - {name: item_id, type: shelf.items.ItemId}
      - {name: part, type: String}
    outcomes:
      - name: refused
        when_subject:
          predicate: SUBJECT
        error: shelf.items.Refused
      - name: checked
        updates: shelf.items.Item
        instance: item_id
        emits: [shelf.items.Checked]
        payload:
          shelf.items.Checked: {item_id: input.item_id}
      - {name: unknown-item, unknown_instance: true, error: shelf.items.UnknownItem}
  - name: shelf.items.Tag
    input:
      - {name: label, type: String}
      - {name: tail, type: String}
    outcomes:
      - name: refused
        when: GUARD
        error: shelf.items.Refused
      - name: tagged
        emits: [shelf.items.Tagged]
        payload:
          shelf.items.Tagged: {label: input.label}
views:
  - name: shelf.items.All
    source: shelf.items.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: shelf.items.ItemId}
      - {name: state, type: shelf.items.Item.State}
      - {name: team, type: String}
      - {name: label, type: String}
      - {name: profile, type: shelf.items.Profile}
  - name: shelf.items.Filtered
    source: shelf.items.Item
    consistency: read_your_writes
    params: [{name: q, type: PARAM}]
    filter: FILTER
    fields:
      - {name: item_id, type: shelf.items.ItemId}
      - {name: label, type: String}
  - name: shelf.items.Hits
    source: shelf.items.Item
    consistency: read_your_writes
    params: [{name: p, type: MEASURE_PARAM}]
    group_by: [team]
    fields:
      - {name: team, type: String}
      - {name: rows, type: Integer, aggregate: {count: {}}}
      - {name: hits, type: Integer, aggregate: {count: {}, where: MEASURE}}
components:
  - component: shelf-service
    summary: Holds every item.
    owns: {domains: [shelf.items]}
    accepts:
      commands: [shelf.items.AddItem, shelf.items.Check, shelf.items.Tag]
    publishes:
      events: [shelf.items.ItemAdded, shelf.items.Checked, shelf.items.Tagged]
    reached_by: network
";

/// The shelf model with every site set to a plain, literal-free baseline unless overridden.
struct Shelf {
    filter: &'static str,
    param: &'static str,
    measure: &'static str,
    measure_param: &'static str,
    subject: &'static str,
    guard: &'static str,
}

const BASELINE: Shelf = Shelf {
    filter: "{label: {starts_with: {param: q}}}",
    param: "String",
    measure: "{label: {starts_with: {param: p}}}",
    measure_param: "String",
    subject: "{label: {starts_with: {input: part}}}",
    guard: "{label: {ends_with: {input: tail}}}",
};

impl Shelf {
    fn text(&self) -> String {
        SHELF
            .replace("FILTER", self.filter)
            .replace("MEASURE_PARAM", self.measure_param)
            .replace("MEASURE", self.measure)
            .replace("PARAM", self.param)
            .replace("SUBJECT", self.subject)
            .replace("GUARD", self.guard)
    }
}

/// The interpreter passes the suite synthesized for `shelf`; and, unless synthesis refused a
/// construct mentioning `site`, the target ignoring the operand fails a scenario under `prefix`.
fn ignored_operand_is_caught(
    shelf: &Shelf,
    site: &str,
    views: Vec<(&'static str, &'static str)>,
    commands: Vec<(&'static str, &'static str)>,
    prefix: &str,
) {
    let ir = ir_of(&shelf.text());
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let refusals = refusals_of(&ir);
    let healthy = run(&synthesis.suite, &Interpreted::for_model(ir.clone()));
    assert_eq!(
        not_passed(&healthy),
        Vec::<String>::new(),
        "the interpreter passes its own suite; refusals: {refusals:#?}"
    );
    if refusals.iter().any(|refusal| refusal.contains(site)) {
        eprintln!("{site}: refused by name: {refusals:#?}");
        return;
    }
    let faulty = run(
        &synthesis.suite,
        &Blanked {
            inner: Interpreted::for_model(ir),
            views,
            commands,
        },
    );
    let failing = not_passed(&faulty);
    assert!(
        failing.iter().any(|line| line.starts_with(prefix)),
        "{site}: a target ignoring the operand passes every scenario under {prefix}; \
         failing: {failing:#?}; scenarios: {:#?}; refusals: {refusals:#?}",
        healthy.keys().collect::<Vec<_>>()
    );
}

#[test]
fn adv_u9_baseline_filter_ignoring_the_parameter_fails() {
    ignored_operand_is_caught(
        &BASELINE,
        "shelf.items.Filtered",
        vec![("shelf.items.Filtered", "q")],
        vec![],
        "shelf.items.AddItem/",
    );
}

#[test]
fn adv_u9_negated_filter_ignoring_the_parameter_fails() {
    ignored_operand_is_caught(
        &Shelf {
            filter: "{not: {label: {starts_with: {param: q}}}}",
            ..BASELINE
        },
        "shelf.items.Filtered",
        vec![("shelf.items.Filtered", "q")],
        vec![],
        "shelf.items.AddItem/",
    );
}

#[test]
fn adv_u9_nested_field_filter_ignoring_the_parameter_fails() {
    let shelf = Shelf {
        filter: "{profile.title: {contains: {param: q}}}",
        ..BASELINE
    };
    // The arrangement does not reach a nested field's text: the view is refused by name,
    // ESS-SYNTH-005, rather than asserted on a parameter nothing arranged (coordinator, N3).
    let refusals = refusals_of(&ir_of(&shelf.text()));
    assert!(
        refusals
            .iter()
            .any(|refusal| refusal.starts_with("ESS-SYNTH-005")
                && refusal.contains("shelf.items.Filtered")),
        "{refusals:#?}"
    );
    ignored_operand_is_caught(
        &shelf,
        "shelf.items.Filtered",
        vec![("shelf.items.Filtered", "q")],
        vec![],
        "shelf.items.AddItem/",
    );
}

#[test]
fn adv_u9_measure_where_ignoring_the_parameter_fails() {
    ignored_operand_is_caught(
        &BASELINE,
        "shelf.items.Hits",
        vec![("shelf.items.Hits", "p")],
        vec![],
        "shelf.items.AddItem/",
    );
}

#[test]
fn adv_u9_optional_measure_parameter_suite_holds_and_decides() {
    ignored_operand_is_caught(
        &Shelf {
            measure_param: "Optional<String>",
            ..BASELINE
        },
        "shelf.items.Hits",
        vec![("shelf.items.Hits", "p")],
        vec![],
        "shelf.items.AddItem/",
    );
}

#[test]
fn adv_u9_nested_when_subject_ignoring_the_input_fails() {
    ignored_operand_is_caught(
        &Shelf {
            subject: "{profile.title: {ends_with: {input: part}}}",
            ..BASELINE
        },
        "shelf.items.Check",
        vec![],
        vec![("shelf.items.Check", "part")],
        "shelf.items.Check/",
    );
}

#[test]
fn adv_u9_negated_plain_guard_ignoring_the_input_fails() {
    ignored_operand_is_caught(
        &Shelf {
            guard: "{not: {label: {ends_with: {input: tail}}}}",
            ..BASELINE
        },
        "shelf.items.Tag",
        vec![],
        vec![("shelf.items.Tag", "tail")],
        "shelf.items.Tag/",
    );
}

// ---- part 3: the same two faults against literal operators (origin control) -------------------

const LITERAL: &str = r#"format: ess/22
system: phones
version: v1
domain: phones.calls
errors:
  - {name: phones.calls.Blocked, summary: Blocked., fields: []}
  - {name: phones.calls.Spoofed, summary: Spoofed., fields: []}
  - {name: phones.calls.Flagged, summary: Flagged., fields: []}
events:
  - name: phones.calls.Screened
    fields:
      - {name: caller, type: String}
commands:
  - name: phones.calls.Screen
    input:
      - {name: caller, type: String}
    outcomes:
      - name: blocked
        when: {caller: {starts_with: "Ab"}}
        error: phones.calls.Blocked
      - name: spoofed
        when: {caller: {ends_with: "Yz"}}
        error: phones.calls.Spoofed
      - name: flagged
        when: {caller: {contains: "Mn"}}
        error: phones.calls.Flagged
      - name: screened
        emits: [phones.calls.Screened]
        payload:
          phones.calls.Screened: {caller: input.caller}
components:
  - component: phones-service
    summary: Screens calls.
    owns: {domains: [phones.calls]}
    accepts: {commands: [phones.calls.Screen]}
    publishes: {events: [phones.calls.Screened]}
    reached_by: network
"#;

/// The interpreter over [`LITERAL`], with its literal guards decided by `mode`: a request the mode
/// decides differently is moved to a caller the model decides that way.
struct Literal {
    inner: Interpreted,
    mode: Mode,
}

fn literal_decision(mode: Mode, caller: &str) -> &'static str {
    if holds(mode, TextOp::StartsWith, caller, "Ab") {
        "blocked"
    } else if holds(mode, TextOp::EndsWith, caller, "Yz") {
        "spoofed"
    } else if holds(mode, TextOp::Contains, caller, "Mn") {
        "flagged"
    } else {
        "screened"
    }
}

impl ConformanceTarget for Literal {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if let Some(caller) = text(&request.input, "caller").map(str::to_owned) {
            let faulty = literal_decision(self.mode, &caller);
            if literal_decision(Mode::Correct, &caller) != faulty {
                let moved = match faulty {
                    "blocked" => "Ab",
                    "spoofed" => "Yz",
                    "flagged" => "Mn",
                    _ => "q",
                };
                request
                    .input
                    .insert("caller".to_owned(), Node::Text(moved.to_owned()));
            }
        }
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
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

fn literal_caught(mode: Mode) {
    let ir = ir_of(LITERAL);
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let healthy = run(
        &suite,
        &Literal {
            inner: Interpreted::for_model(ir.clone()),
            mode: Mode::Correct,
        },
    );
    assert_eq!(
        not_passed(&healthy),
        Vec::<String>::new(),
        "healthy control"
    );
    let failing = not_passed(&run(
        &suite,
        &Literal {
            inner: Interpreted::for_model(ir),
            mode,
        },
    ));
    assert!(
        failing
            .iter()
            .any(|line| line.starts_with("phones.calls.Screen/")),
        "{mode:?} passes every literal-guard scenario: {:#?}",
        healthy.keys().collect::<Vec<_>>()
    );
}

#[test]
#[ignore = "follow-up: guard text witnesses need case- and position-deciding rows"]
fn adv_u9_control_literal_guards_against_a_case_folding_target() {
    literal_caught(Mode::FoldsCase);
}

#[test]
#[ignore = "follow-up: guard text witnesses need case- and position-deciding rows"]
fn adv_u9_control_literal_guards_against_contains_for_all() {
    literal_caught(Mode::ContainsForAll);
}

// ---- part 4: an Optional parameter left out under a disjunction --------------------------------

/// The read leaving an Optional text-operand parameter out is asserted to exclude the subject's
/// row. Under `any:` with a second, literal disjunct the row may still be shown — `Unknown or True`
/// is `True` — so the interpreter must still pass the suite synthesized for it.
#[test]
fn adv_u9_optional_parameter_left_out_under_any_keeps_the_interpreter_green() {
    for filter in [
        "{any: [{label: {ends_with: {param: q}}}, {team: {starts_with: \"T\"}}]}",
        "{any: [{team: {starts_with: \"T\"}}, {label: {ends_with: {param: q}}}]}",
    ] {
        let shelf = Shelf {
            filter,
            param: "Optional<String>",
            ..BASELINE
        };
        let ir = ir_of(&shelf.text());
        let synthesis = ess_conformance::synthesize::synthesize(&ir);
        let healthy = run(&synthesis.suite, &Interpreted::for_model(ir.clone()));
        assert_eq!(
            not_passed(&healthy),
            Vec::<String>::new(),
            "{filter}: the interpreter passes its own suite; refusals: {:#?}",
            refusals_of(&ir)
        );
    }
}
