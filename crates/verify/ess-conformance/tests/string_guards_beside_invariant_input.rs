//! Three string guards on one `Optional` input, beside an input whose newtype carries an
//! invariant, leave every branch its scenario
//! (<https://github.com/beyond10x/ess/issues/511>).
//!
//! `SignIn` takes a `scope` whose newtype admits only texts naming `basic` as one of its
//! space-separated words, a `state` the success branch publishes under a presence policy, and a
//! `hint` that three refusals each read with `defined(hint)` and one string operator. The base
//! witness of `scope` is its own path, which the invariant refuses, so every candidate built on it
//! is dropped. Beside a present client row, the `external:` branches and the success branch need an
//! input that refutes every hint guard and sends a registered return address; with the third guard
//! declared, the bounded walk spent every candidate before it reached an admitted `scope` and no
//! such input was found.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    flatten,
    synthesize::{synthesize, Synthesis},
    when,
    witness::{candidates, Distinction},
    ConformanceScenario, ScenarioStep,
};
use ess_domain::{
    name::QualifiedName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{node::Node, predicate::Predicate};

/// The shape of #511, with neutral names.
const ACCESS: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.access
summary: A sign-in request refused for a malformed hint, an unregistered return address or by a provider.
types:
  - {name: demo.access.ClientId, kind: newtype, of: String}
  - {name: demo.access.ReturnUri, kind: newtype, of: String}
  - {name: demo.access.Hint, kind: newtype, of: String}
  - {name: demo.access.State, kind: newtype, of: String}
  - name: demo.access.Scope
    kind: newtype
    of: String
    invariants:
      - any:
          - value == "basic"
          - {value: {starts_with: "basic "}}
          - {value: {ends_with: " basic"}}
          - {value: {contains: " basic "}}
entities:
  - name: demo.access.Client
    identity: {name: client_id, type: demo.access.ClientId}
    fields:
      - {name: return_uris, type: List<demo.access.ReturnUri>}
    lifecycle:
      initial: Registered
      states: [Registered]
      terminal: [Registered]
actors:
  - name: demo.access.Caller
    may: [demo.access.Register, demo.access.SignIn]
errors:
  - {name: demo.access.NotRedirected, summary: The client or return address is unknown., fields: []}
  - {name: demo.access.InvalidRequest, summary: The request is malformed., fields: []}
  - {name: demo.access.LoginRequired, summary: The caller must sign in interactively., fields: []}
  - {name: demo.access.AccessDenied, summary: The request was denied., fields: []}
events:
  - name: demo.access.Registered
    fields:
      - {name: client_id, type: demo.access.ClientId}
  - name: demo.access.SignedIn
    fields:
      - {name: client_id, type: demo.access.ClientId}
      - {name: state, type: Optional<demo.access.State>, presence: omitted_when_absent}
commands:
  - name: demo.access.Register
    input:
      - {name: return_uris, type: List<demo.access.ReturnUri>}
    response:
      - {name: client_id, type: demo.access.ClientId}
    outcomes:
      - name: registered
        creates: demo.access.Client
        instance: client_id
        sets: {return_uris: input.return_uris}
        emits: [demo.access.Registered]
        payload: {demo.access.Registered: {client_id: {response: client_id}}}
        returns: true
  - name: demo.access.SignIn
    input:
      - {name: client_id, type: demo.access.ClientId}
      - {name: scope, type: demo.access.Scope}
      - {name: return_uri, type: demo.access.ReturnUri}
      - {name: hint, type: Optional<demo.access.Hint>}
      - {name: state, type: Optional<demo.access.State>}
    outcomes:
      - name: unknown-client
        when_related: {via: input.client_id, exists: false}
        error: demo.access.NotRedirected
      - name: return-uri-not-registered
        when_related:
          via: input.client_id
          predicate: {not: {exists: {in: return_uris, as: uri, that: uri == input.return_uri}}}
        error: demo.access.NotRedirected
      - name: hint-leads
        when: {all: [{hint: {defined: true}}, {hint: {starts_with: "none "}}]}
        error: demo.access.InvalidRequest
      - name: hint-between
        when: {all: [{hint: {defined: true}}, {hint: {contains: " none "}}]}
        error: demo.access.InvalidRequest
      - name: hint-trails
        when: {all: [{hint: {defined: true}}, {hint: {ends_with: " none"}}]}
        error: demo.access.InvalidRequest
      - name: login-required
        external: the caller is not signed in and may not be asked to
        error: demo.access.LoginRequired
      - name: access-denied
        external: the caller or the provider denied the request
        error: demo.access.AccessDenied
      - name: signed-in
        emits: [demo.access.SignedIn]
        payload: {demo.access.SignedIn: {client_id: input.client_id, state: input.state}}
views:
  - name: demo.access.Clients
    source: demo.access.Client
    consistency: read_your_writes
    fields:
      - {name: client_id, type: demo.access.ClientId}
      - {name: return_uris, type: List<demo.access.ReturnUri>}
"#;

/// The branches beside a present client row that no hint guard and no related guard claims.
const UNCLAIMED: [&str; 3] = ["login-required", "access-denied", "signed-in"];

/// Every branch of `SignIn`.
const BRANCHES: [&str; 8] = [
    "unknown-client",
    "return-uri-not-registered",
    "hint-leads",
    "hint-between",
    "hint-trails",
    "login-required",
    "access-denied",
    "signed-in",
];

/// The id of the scenario that witnesses `branch`.
fn scenario_id(branch: &str) -> String {
    format!("demo.access.SignIn/outcome/{branch}")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("access.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

fn refusals_about(result: &Synthesis, branch: &str) -> Vec<String> {
    let id = scenario_id(branch);
    result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, branch: &str) -> Option<&'a ConformanceScenario> {
    let id = scenario_id(branch);
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

fn requires(scenario: &ConformanceScenario, branch: &str) -> bool {
    let outcome = format!("demo.access.SignIn/{branch}");
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome: expected }
            if expected.to_string() == outcome)
    })
}

/// The model with the `hint-between` refusal removed: two string guards, the shape that already
/// synthesized every branch.
fn two_guards(text: &str) -> String {
    let between = "      - name: hint-between\n        when: {all: [{hint: {defined: true}}, {hint: {contains: \" none \"}}]}\n        error: demo.access.InvalidRequest\n";
    assert!(text.contains(between), "hint-between is declared");
    text.replace(between, "")
}

#[test]
fn the_unclaimed_branches_are_not_refused_beside_three_string_guards() {
    let result = synthesis(ACCESS);
    let refused: Vec<String> = UNCLAIMED
        .iter()
        .flat_map(|branch| {
            refusals_about(&result, branch)
                .into_iter()
                .map(move |refusal| format!("{branch}: {refusal}"))
        })
        .collect();
    assert_eq!(refused, Vec::<String>::new());
}

#[test]
fn every_branch_gets_its_scenario_beside_three_string_guards() {
    let result = synthesis(ACCESS);
    for branch in BRANCHES {
        let witness = scenario(&result, branch).unwrap_or_else(|| {
            panic!(
                "no scenario for {branch}; refusals: {:#?}",
                refusals_about(&result, branch)
            )
        });
        assert!(
            requires(witness, branch),
            "{branch} requires its own outcome"
        );
    }
}

#[test]
fn two_string_guards_still_get_every_branch() {
    let result = synthesis(&two_guards(ACCESS));
    for branch in BRANCHES.iter().filter(|branch| **branch != "hint-between") {
        assert!(
            scenario(&result, branch).is_some(),
            "no scenario for {branch}; refusals: {:#?}",
            refusals_about(&result, branch)
        );
    }
}

#[test]
fn the_interpreted_target_passes_every_branch() {
    let model = ir(ACCESS);
    let suite = synthesize(&model).suite;
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        )
        .into_report();
    for branch in BRANCHES {
        let id = scenario_id(branch);
        let run = report
            .scenarios
            .iter()
            .find(|run| run.scenario.to_string() == id)
            .unwrap_or_else(|| panic!("{id} is run"));
        assert_eq!(
            run.status,
            ess_conformance::report::Status::Passed,
            "{}: {:?}",
            run.scenario,
            run.checks
        );
    }
}

/// The `SignIn` command of the model.
fn sign_in(ir: &EssIr) -> &ess_compiler::ir::ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new("demo.access.SignIn").expect("a valid name"))
        .expect("the model declares SignIn")
}

/// The three hint guards, in declaration order.
fn hint_guards(command: &ess_compiler::ir::ResolvedCommand) -> Vec<&Predicate> {
    command
        .outcomes
        .iter()
        .filter(|outcome| outcome.name.as_str().starts_with("hint-"))
        .map(|outcome| when(outcome).expect("a hint branch is taken by a `when`"))
        .collect()
}

/// Whether `scope` is a text the `Scope` invariant admits: `basic` as one of its words.
fn admitted_scope(input: &std::collections::BTreeMap<String, Node>) -> bool {
    matches!(input.get("scope"), Some(Node::Text(text))
        if text == "basic"
            || text.starts_with("basic ")
            || text.ends_with(" basic")
            || text.contains(" basic "))
}

#[test]
fn every_candidate_holds_a_scope_its_type_admits() {
    let model = ir(ACCESS);
    let command = sign_in(&model);
    let guards = hint_guards(command);
    assert_eq!(guards.len(), 3, "three hint guards");
    let options = candidates(&model, command, &guards, Distinction::PLAIN).unwrap();
    assert!(!options.is_empty(), "some candidate is admitted");
    let refused: Vec<_> = options
        .iter()
        .filter(|input| !admitted_scope(input))
        .collect();
    assert!(
        refused.is_empty(),
        "candidates the type refuses: {refused:#?}"
    );
}

/// A one-letter code a guard reads beside a list of slots (beyond10x/ess#809): `{invariant}` is the
/// code's invariant over its length and `{pick}` the declared type of the input `pick`.
fn one_letter(invariant: &str, pick: &str) -> String {
    format!(
        r#"format: ess/37
system: depot
version: v1
domain: depot.racks
types:
  - name: depot.racks.Code
    kind: newtype
    of: String
    alphabet: "ABCDEFGHIJKLMNOPQRSTUVWXYZ"
    invariants:
      - {invariant}
  - name: depot.racks.Slot
    kind: struct
    fields:
      - {{name: code, type: depot.racks.Code}}
      - {{name: label, type: String}}
  - name: depot.racks.Layout
    kind: struct
    fields:
      - {{name: slots, type: "List<depot.racks.Slot>"}}
entities:
  - name: depot.racks.Shelf
    identity: {{name: shelf_id, type: String}}
    fields:
      - {{name: layout, type: Optional<depot.racks.Layout>}}
      - {{name: pick, type: {pick}}}
    lifecycle: {{initial: Placed, states: [Placed], terminal: [Placed], transitions: []}}
actors:
  - name: depot.racks.Clerk
    may: [depot.racks.PlaceShelf, depot.racks.Shelves]
errors:
  - {{name: depot.racks.UnknownPick, fields: []}}
commands:
  - name: depot.racks.PlaceShelf
    input:
      - {{name: shelf_id, type: String}}
      - {{name: layout, type: Optional<depot.racks.Layout>}}
      - {{name: pick, type: {pick}}}
    outcomes:
      - name: unknown-pick
        when:
          all:
            - defined(pick)
            - not: {{exists: {{in: layout.slots, as: s, that: s.code == pick}}}}
        error: depot.racks.UnknownPick
      - name: placed
        unknown_instance: true
        creates: depot.racks.Shelf
        instance: shelf_id
        sets: {{layout: input.layout, pick: input.pick}}
        emits: [depot.racks.ShelfPlaced]
        payload:
          depot.racks.ShelfPlaced: {{shelf_id: input.shelf_id}}
      - name: taken
        preserves: depot.racks.Shelf
        instance: shelf_id
events:
  - name: depot.racks.ShelfPlaced
    fields:
      - {{name: shelf_id, type: String}}
views:
  - name: depot.racks.Shelves
    source: depot.racks.Shelf
    consistency: read_your_writes
    fields:
      - {{name: shelf_id, type: String}}
      - {{name: layout, type: Optional<depot.racks.Layout>}}
      - {{name: pick, type: {pick}}}
"#
    )
}

/// The `unknown-pick` scenario of `text` sends one letter of the alphabet as `pick`, and passes on
/// the interpreted target; no refusal names the branch.
fn assert_unknown_pick_witnessed(text: &str) {
    let model = ir(text);
    let result = synthesize(&model);
    let id = "depot.racks.PlaceShelf/outcome/unknown-pick";
    let refused: Vec<String> = result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect();
    assert_eq!(refused, Vec::<String>::new(), "{text}");
    let (_, witness) = result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"));
    let picks: Vec<&Node> = witness
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => match input.get("pick") {
                Some(ess_conformance::ScenarioValue::Literal { value }) => Some(value),
                _ => None,
            },
            _ => None,
        })
        .collect();
    assert!(
        picks.iter().any(|pick| matches!(pick, Node::Text(text)
            if text.chars().count() == 1 && text.chars().all(|c| c.is_ascii_uppercase()))),
        "{picks:?}"
    );
    let admitted = ess_conformance::AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = ess_conformance::Runner::for_suite(&result.suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        )
        .into_report();
    let run = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string() == id)
        .unwrap_or_else(|| panic!("{id} is run"));
    assert_eq!(
        run.status,
        ess_conformance::report::Status::Passed,
        "{:?}",
        run.checks
    );
}

#[test]
fn a_guard_read_optional_code_of_one_letter_is_witnessed() {
    assert_unknown_pick_witnessed(&one_letter(
        "value.count == 1",
        "Optional<depot.racks.Code>",
    ));
}

#[test]
fn a_guard_read_code_under_a_byte_or_upper_bound_invariant_is_witnessed() {
    for invariant in ["value.count <= 1", "value.utf8_bytes == 1"] {
        assert_unknown_pick_witnessed(&one_letter(invariant, "Optional<depot.racks.Code>"));
    }
    assert_unknown_pick_witnessed(&one_letter("value.count == 1", "depot.racks.Code"));
}

/// The guard's own candidates stay on the ladder beside the invariant's: with `pick` required, the
/// accepting branch needs a slot code equal to it, which only the guard's copy of one side onto
/// the other offers.
#[test]
fn a_literal_the_guard_compares_with_stays_a_candidate() {
    let result = synthesize(&ir(&one_letter("value.count == 1", "depot.racks.Code")));
    let id = "depot.racks.PlaceShelf/outcome/placed";
    assert!(
        result
            .suite
            .scenarios
            .iter()
            .any(|(key, _)| key.to_string() == id),
        "no scenario {id}"
    );
}

/// Review of the #809 fix: the `unknown-pick` scenario of `text` is written, no refusal names it,
/// and it passes on the interpreted target.
fn assert_unknown_pick_written_and_passing(text: &str) {
    let model = ir(text);
    let result = synthesize(&model);
    let id = "depot.racks.PlaceShelf/outcome/unknown-pick";
    let refused: Vec<String> = result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect();
    assert_eq!(refused, Vec::<String>::new(), "{text}");
    let admitted = ess_conformance::AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = ess_conformance::Runner::for_suite(&result.suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        )
        .into_report();
    let run = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string() == id)
        .unwrap_or_else(|| panic!("{id} is run"));
    assert_eq!(
        run.status,
        ess_conformance::report::Status::Passed,
        "{:?}",
        run.checks
    );
}

/// Review of the #809 fix: an invariant of several conjuncts, a length window and a prefix,
/// admits `QA` and `QAB`; the guard-read `pick` gets no candidate the whole invariant admits, and
/// the branch stays refused as on 0.98.0 (`pick = "IBVD"`, "no candidate of the 1 tried").
#[test]
fn a_guard_read_code_under_a_length_window_and_a_prefix_is_witnessed() {
    assert_unknown_pick_written_and_passing(&one_letter(
        "value.count >= 2\n      - value.count <= 3\n      - {value: {starts_with: \"Q\"}}",
        "Optional<depot.racks.Code>",
    ));
}

/// Review of the #809 fix: an exact length beside a prefix and a suffix (`AZ` is the one text).
#[test]
fn a_guard_read_code_under_a_length_a_prefix_and_a_suffix_is_witnessed() {
    assert_unknown_pick_written_and_passing(&one_letter(
        "value.count == 2\n      - {value: {starts_with: \"A\"}}\n      - {value: {ends_with: \"Z\"}}",
        "Optional<depot.racks.Code>",
    ));
}

/// Review of the #809 fix, the control beside the two above: a length window alone is witnessed.
#[test]
fn a_guard_read_code_under_a_length_window_alone_is_witnessed() {
    assert_unknown_pick_written_and_passing(&one_letter(
        "value.count >= 2\n      - value.count <= 3",
        "Optional<depot.racks.Code>",
    ));
}

/// Review of the #809 fix: an invariant no text satisfies stays a refusal of the branch, and
/// synthesis ends (no hang, no panic).
#[test]
fn a_guard_read_code_no_text_satisfies_stays_refused() {
    let result = synthesize(&ir(&one_letter(
        "value.count >= 3\n      - value.count <= 2",
        "Optional<depot.racks.Code>",
    )));
    let id = "depot.racks.PlaceShelf/outcome/unknown-pick";
    assert!(
        !result
            .suite
            .scenarios
            .iter()
            .any(|(key, _)| key.to_string() == id),
        "{id} is written over an invariant no text satisfies"
    );
    assert!(
        result.refusals.iter().any(|refusal| refusal
            .scenario
            .as_ref()
            .is_some_and(|scenario| scenario.to_string() == id)),
        "{id} is refused"
    );
}

/// A literal the guard compares a guard-read input with stays on that input's ladder beside the
/// invariant's values, where the type refuses the value synthesis started from (#809, acceptance
/// 4): the branch selected by `pick == "Q"` is sent `Q`, which the invariant admits, beside the
/// values the invariant's own literal `Z` adds to the ladder.
#[test]
fn a_literal_the_guard_compares_with_is_the_value_sent() {
    let guarded = "        when:\n          all:\n            - defined(pick)\n            - not: {exists: {in: layout.slots, as: s, that: s.code == pick}}\n";
    let text = one_letter(
        "value.count == 1\n      - 'value != \"Z\"'",
        "depot.racks.Code",
    );
    assert!(text.contains(guarded), "the guard is in the model");
    let text = text.replace(guarded, "        when: 'pick == \"Q\"'\n");
    let result = synthesize(&ir(&text));
    let id = "depot.racks.PlaceShelf/outcome/unknown-pick";
    let (_, witness) = result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"));
    // The `pick` of every step the scenario expects `unknown-pick` from.
    let refused: Vec<&Node> = witness
        .steps
        .windows(2)
        .filter_map(|pair| match pair {
            [ScenarioStep::ExecuteCommand { input, .. }, ScenarioStep::ExpectOutcome { outcome }]
                if outcome.to_string() == "depot.racks.PlaceShelf/unknown-pick" =>
            {
                match input.get("pick") {
                    Some(ess_conformance::ScenarioValue::Literal { value }) => Some(value),
                    _ => None,
                }
            }
            _ => None,
        })
        .collect();
    assert!(!refused.is_empty(), "{text}");
    assert!(
        refused
            .iter()
            .all(|pick| **pick == Node::Text("Q".to_owned())),
        "{refused:?}"
    );
    assert_unknown_pick_written_and_passing(&text);
}

/// The input an `external:` branch beside a registered client is sent: every hint guard refuted
/// and the return address equal to the one the row registers, which the related search grounds
/// as a guard over the input.
#[test]
fn a_candidate_refutes_every_hint_guard_at_the_registered_return_address() {
    let model = ir(ACCESS);
    let command = sign_in(&model);
    let registered: Predicate =
        Predicate::parse_expression("return_uri == \"return-a\"").expect("a valid predicate");
    let mut guards = hint_guards(command);
    guards.push(&registered);
    let options = candidates(&model, command, &guards, Distinction::PLAIN).unwrap();
    let found = options.iter().any(|input| {
        let facts = flatten(&model, command, input).expect("a candidate fits the input");
        facts.decide(&registered).is_satisfied()
            && guards[..3]
                .iter()
                .all(|guard| matches!(facts.decide(guard), ess_conformance::Decision::Refuted(_)))
    });
    assert!(
        found,
        "no candidate of {} refutes every hint guard at the registered return address",
        options.len()
    );
}
