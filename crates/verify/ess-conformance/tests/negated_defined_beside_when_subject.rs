//! What synthesis can witness of a branch guarded by `when: defined(<optional input>)` beside a
//! `when_subject:` predicate comparing a stored field with that same input, and of its
//! `guard-negate` mutant `when: not defined(<optional input>)`.
//!
//! `Fulfil` refuses `stale-version` where the caller names an `expected_version` and the order's
//! `version` differs. The baseline branch is witnessed: the input carries a version and the row
//! holds another. The mutant's branch is selected on no row: its `when:` admits only an input
//! leaving `expected_version` out, and `version != input.expected_version` over an absent operand
//! is `Unknown`, never `True` (Kleene evaluation, `ess_primitives::predicate`). The model
//! interpreter answers such a send as undetermined rather than as the refusal, so no scenario of
//! the mutant's suite can observe it, and synthesis refuses the branch (`ESS-SYNTH-003`). Scoring
//! that mutant is the mutation audit's (story
//! `negated-defined-optional-input-witnessed-beside-when-subject`).
//!
//! The second half keeps the reproduction of
//! <https://github.com/beyond10x/ess/issues/511>, which has a different cause: there the absent
//! value of the `Optional` input is among the candidates, and what runs out is the walk over a
//! leaf whose base breaks its type's invariant.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    synthesize::{synthesize, Synthesis},
    ConformanceScenario, ScenarioStep,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

/// The story's shape, with neutral names (`.engineering/repro/negated-defined-optional/`).
const ORDER: &str = include_str!("fixtures/negated-defined-beside-subject.yaml");

const STALE: &str = "demo.order.Fulfil/outcome/stale-version";
const BASELINE_GUARD: &str = "        when: defined(expected_version)\n";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
}

/// The spec with `from` replaced by `to`, exactly once.
fn edited(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "`{from}` occurs once");
    text.replace(from, to)
}

/// The `guard-negate` mutant of `stale-version`, as the audit writes it.
fn negated(text: &str) -> String {
    edited(
        text,
        BASELINE_GUARD,
        "        when: not defined(expected_version)\n",
    )
}

fn refusals_about(result: &Synthesis, id: &str) -> Vec<String> {
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

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}; refusals: {:#?}",
                    refusals_about(result, id)
                )
            },
            |(_, scenario)| scenario,
        )
}

fn written(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

/// The value the scenario's last invocation of `command` sends at `field`, `None` where it leaves
/// the field out or writes it absent.
fn last_sent(scenario: &ConformanceScenario, command: &str, field: &str) -> Option<Node> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input),
            _ => None,
        })
        .next_back()
        .unwrap_or_else(|| panic!("the scenario invokes {command}"))
        .get(field)
        .and_then(|value| value.as_literal().cloned())
        .filter(|node| !matches!(node, Node::Null))
}

#[test]
fn the_baseline_branch_is_witnessed_with_the_input_present() {
    let result = synthesis(ORDER);
    assert_eq!(refusals_about(&result, STALE), Vec::<String>::new());
    let witness = scenario(&result, STALE);
    assert!(
        last_sent(witness, "demo.order.Fulfil", "expected_version").is_some(),
        "`defined(expected_version)` is satisfied only by a present input"
    );
}

/// The mutant's branch holds on no row: the absent operand leaves the subject guard `Unknown`.
#[test]
fn the_negated_guard_is_selected_on_no_row_and_synthesis_refuses_it() {
    let result = synthesis(&negated(ORDER));
    assert!(!written(&result, STALE), "{STALE} is not written");
    let refused = refusals_about(&result, STALE);
    let [only] = refused.as_slice() else {
        panic!("{STALE}: one refusal, found {refused:#?}");
    };
    assert!(only.starts_with("ESS-SYNTH-003"), "{only}");
}

/// Beside a subject guard that does not compare the input, the negated guard is witnessed with the
/// input absent: the refusal above is the comparison's, not the negation's.
#[test]
fn beside_a_subject_guard_reading_no_input_the_negated_guard_is_witnessed_absent() {
    let text = negated(&edited(
        ORDER,
        "              - version != input.expected_version\n",
        "              - version != 0\n",
    ));
    let result = synthesis(&text);
    assert_eq!(refusals_about(&result, STALE), Vec::<String>::new());
    let witness = scenario(&result, STALE);
    assert_eq!(
        last_sent(witness, "demo.order.Fulfil", "expected_version"),
        None,
        "`not defined(expected_version)` is satisfied only by an absent input"
    );
}

/// The shape of <https://github.com/beyond10x/ess/issues/511>, with neutral names: three string
/// guards each requiring `defined: true` on one `Optional` newtype input, a `when_related` guard
/// over a list of the related row, two `external:` outcomes and an accepting default. The
/// `Optional<State>` input copied into a published field with a presence policy doubles every
/// candidate, and `scope`'s base breaks its type's invariant.
const SIGN_IN: &str = r#"format: ess/23
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

/// The branches the related-row search witnesses beside a present client row.
const BESIDE_THE_ROW: [&str; 3] = [
    "demo.access.SignIn/outcome/login-required",
    "demo.access.SignIn/outcome/access-denied",
    "demo.access.SignIn/outcome/signed-in",
];

const HINT_BETWEEN: &str = "      - name: hint-between\n        when: {all: [{hint: {defined: true}}, {hint: {contains: \" none \"}}]}\n        error: demo.access.InvalidRequest\n";

fn assert_every_branch_witnessed(text: &str) {
    let result = synthesis(text);
    for id in BESIDE_THE_ROW {
        assert_eq!(refusals_about(&result, id), Vec::<String>::new(), "{id}");
        assert!(written(&result, id), "{id} is written");
    }
}

/// Two of the three string guards: every branch beside the row is witnessed.
#[test]
fn with_two_string_guards_every_branch_beside_the_related_row_is_witnessed() {
    assert_every_branch_witnessed(&edited(SIGN_IN, HINT_BETWEEN, ""));
}

/// The three string guards together leave the bounded walk on candidates whose `scope` breaks its
/// invariant, so the grounded `return_uri` is never paired with a hint every guard refutes.
#[test]
#[ignore = "https://github.com/beyond10x/ess/issues/511: candidate walk spent on inadmissible `scope` bases (witness.rs), not this story's cause"]
fn with_three_string_guards_every_branch_beside_the_related_row_is_witnessed() {
    assert_every_branch_witnessed(SIGN_IN);
}
