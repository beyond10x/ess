//! A wrong-state scenario refutes each sibling branch through any one of its conjuncts
//! (https://github.com/beyond10x/ess/issues/516).
//!
//! `Receive` moves a `Request` from `Awaiting` and declares, beside that move and its
//! `wrong_state` branch, three siblings that each combine an input guard with a subject guard:
//!
//! * `nonce-missing`: `when: not defined(nonce)`, `when_subject: defined(nonce)`;
//! * `nonce-mismatch`: `when: defined(nonce)`, `when_subject: defined(nonce) and nonce != input.nonce`;
//! * `nonce-not-sent`: `when: defined(nonce)`, `when_subject: not defined(nonce)`.
//!
//! No input refutes every `when:` alone — `not defined(nonce)` and `defined(nonce)` cannot both be
//! false. A branch holds only where both its conjuncts hold, so a row and input that make either
//! one false refute it: a `Sent` row holding a nonce, sent an input carrying none, refutes
//! `nonce-missing` through its subject guard and the other two through their input guards.
//!
//! Three more siblings are guarded by the row alone, and two of them quantify over the input:
//! `audience-mismatch` (`not (exists audience in input.aud: audience == {fact: client_id})`) and
//! `untrusted-audience` (an `aud` element neither the row's `client_id` nor one of its
//! `trusted_audiences`). Only a list holding the row's `client_id` refutes them, and the candidate
//! search drew `aud` from the input guards alone, which never name it: that, not the nonce
//! siblings, is what left every candidate refused. The search now also grounds a quantifier over
//! the input on the row, where the first search found nothing.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    synthesize::{synthesize, Synthesis},
    ConformanceScenario, ScenarioStep, ScenarioValue,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

/// The shape of #516, with neutral names: the row's `nonce` is chosen by the creating command's
/// input, so the arrangement can leave it defined or absent.
const EXCHANGE: &str = r#"format: ess/23
system: demo
version: v1
domain: demo.exchange
summary: A request answered once it awaits a response, its nonce checked against the one it sent.
types:
  - {name: demo.exchange.RequestId, kind: newtype, of: Uuid}
  - {name: demo.exchange.Nonce, kind: newtype, of: String}
  - {name: demo.exchange.ClientId, kind: newtype, of: String}
entities:
  - name: demo.exchange.Request
    identity: {name: id, type: demo.exchange.RequestId}
    fields:
      - {name: nonce, type: Optional<demo.exchange.Nonce>}
      - {name: issuer, type: String}
      - {name: client_id, type: demo.exchange.ClientId}
      - {name: trusted_audiences, type: List<demo.exchange.ClientId>}
    lifecycle:
      initial: Sent
      states: [Sent, Awaiting, Answered]
      terminal: [Answered]
      transitions:
        - {name: wait, from: [Sent], to: Awaiting}
        - {name: answer, from: [Awaiting], to: Answered}
actors:
  - name: demo.exchange.Client
    may: [demo.exchange.Send, demo.exchange.Wait, demo.exchange.Receive]
errors:
  - {name: demo.exchange.UnknownRequest, summary: No such request., fields: []}
  - {name: demo.exchange.NotAwaiting, summary: The request awaits no response., fields: []}
  - {name: demo.exchange.NonceMissing, summary: The response carries no nonce., fields: []}
  - {name: demo.exchange.NonceMismatch, summary: The response carries another nonce., fields: []}
  - {name: demo.exchange.NonceNotSent, summary: The request sent no nonce., fields: []}
  - {name: demo.exchange.IssuerMismatch, summary: Another issuer answered., fields: []}
  - {name: demo.exchange.AudienceMismatch, summary: The response is not addressed to this client., fields: []}
  - {name: demo.exchange.UntrustedAudience, summary: The response names an untrusted audience., fields: []}
events:
  - name: demo.exchange.Sent
    fields: [{name: id, type: demo.exchange.RequestId}]
  - name: demo.exchange.Waiting
    fields: [{name: id, type: demo.exchange.RequestId}]
  - name: demo.exchange.Answered
    fields: [{name: id, type: demo.exchange.RequestId}]
commands:
  - name: demo.exchange.Send
    input:
      - {name: nonce, type: Optional<demo.exchange.Nonce>}
      - {name: issuer, type: String}
      - {name: client_id, type: demo.exchange.ClientId}
      - {name: trusted_audiences, type: List<demo.exchange.ClientId>}
    outcomes:
      - name: sent
        creates: demo.exchange.Request
        instance: id
        sets: {nonce: input.nonce, issuer: input.issuer, client_id: input.client_id, trusted_audiences: input.trusted_audiences}
        emits: [demo.exchange.Sent]
        payload: {demo.exchange.Sent: {id: {generated: true}}}
  - name: demo.exchange.Wait
    input: [{name: id, type: demo.exchange.RequestId}]
    outcomes:
      - name: waiting
        moves: demo.exchange.Request.wait
        instance: id
        emits: [demo.exchange.Waiting]
        payload: {demo.exchange.Waiting: {id: input.id}}
  - name: demo.exchange.Receive
    input:
      - {name: id, type: demo.exchange.RequestId}
      - {name: nonce, type: Optional<demo.exchange.Nonce>}
      - {name: iss, type: String}
      - {name: aud, type: List<demo.exchange.ClientId>}
    outcomes:
      - name: unknown-request
        unknown_instance: true
        error: demo.exchange.UnknownRequest
      - name: not-awaiting
        wrong_state: true
        error: demo.exchange.NotAwaiting
      - name: nonce-missing
        when: {nonce: {defined: false}}
        when_subject: {predicate: {nonce: {defined: true}}}
        error: demo.exchange.NonceMissing
      - name: nonce-mismatch
        when: {nonce: {defined: true}}
        when_subject: {predicate: {all: [{nonce: {defined: true}}, "nonce != input.nonce"]}}
        error: demo.exchange.NonceMismatch
      - name: nonce-not-sent
        when: {nonce: {defined: true}}
        when_subject: {predicate: {nonce: {defined: false}}}
        error: demo.exchange.NonceNotSent
      - name: issuer-mismatch
        when_subject: {predicate: "issuer != input.iss"}
        error: demo.exchange.IssuerMismatch
      - name: audience-mismatch
        when_subject: {predicate: {not: {exists: {in: input.aud, as: audience, that: {audience: {eq: {fact: client_id}}}}}}}
        error: demo.exchange.AudienceMismatch
      - name: untrusted-audience
        when_subject: {predicate: {exists: {in: input.aud, as: audience, that: {all: [{audience: {ne: {fact: client_id}}}, {not: {exists: {in: trusted_audiences, as: trusted, that: {trusted: {eq: {fact: audience}}}}}}]}}}}
        error: demo.exchange.UntrustedAudience
      - name: answered
        moves: demo.exchange.Request.answer
        instance: id
        emits: [demo.exchange.Answered]
        payload: {demo.exchange.Answered: {id: input.id}}
views:
  - name: demo.exchange.Requests
    source: demo.exchange.Request
    consistency: read_your_writes
    fields:
      - {name: id, type: demo.exchange.RequestId}
      - {name: nonce, type: Optional<demo.exchange.Nonce>}
      - {name: issuer, type: String}
      - {name: client_id, type: demo.exchange.ClientId}
      - {name: trusted_audiences, type: List<demo.exchange.ClientId>}
      - {name: state, type: demo.exchange.Request.State}
"#;

const SENT: &str = "demo.exchange.Request/state/Sent/refuses/demo.exchange.Receive";
const NOT_AWAITING: &str = "demo.exchange.Receive/not-awaiting";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("exchange.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn synthesis(text: &str) -> Synthesis {
    synthesize(&ir(text))
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

/// The input of every invocation of `command` in the scenario, in step order.
fn inputs(scenario: &ConformanceScenario, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

fn requires(scenario: &ConformanceScenario, outcome: &str) -> bool {
    scenario.steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExpectOutcome { outcome: expected }
            if expected.to_string() == outcome)
    })
}

/// The one `Send` and the one `Receive` of the scenario: the row's creation and the invocation
/// under test.
fn sent_and_received(
    witness: &ConformanceScenario,
) -> (
    BTreeMap<String, ScenarioValue>,
    BTreeMap<String, ScenarioValue>,
) {
    let send = inputs(witness, "demo.exchange.Send");
    let receive = inputs(witness, "demo.exchange.Receive");
    let ([send], [receive]) = (send.as_slice(), receive.as_slice()) else {
        panic!("one Send and one Receive: {send:#?} {receive:#?}");
    };
    (send.clone(), receive.clone())
}

/// Whether the scenario's value for `field` is present: neither left out nor written as absent.
fn carries(input: &BTreeMap<String, ScenarioValue>, field: &str) -> bool {
    input.get(field).is_some_and(|value| {
        value
            .as_literal()
            .is_some_and(|node| !matches!(node, Node::Null))
    })
}

/// The literal list the scenario sends or stores at `field`.
fn list(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Vec<Node> {
    match input.get(field).and_then(ScenarioValue::as_literal) {
        Some(Node::Seq(items)) => items.clone(),
        other => panic!("`{field}` is a literal list: {other:#?}"),
    }
}

/// `audience-mismatch` and `untrusted-audience` are refuted by the row: the input's `aud` holds
/// the row's `client_id`, and every element of it is that or one the row trusts.
fn assert_audiences_refuted(
    send: &BTreeMap<String, ScenarioValue>,
    receive: &BTreeMap<String, ScenarioValue>,
) {
    let client = send
        .get("client_id")
        .and_then(ScenarioValue::as_literal)
        .unwrap_or_else(|| panic!("Send sends a literal client_id: {send:#?}"));
    let trusted = list(send, "trusted_audiences");
    let aud = list(receive, "aud");
    assert!(aud.contains(client), "audience-mismatch holds: {aud:?}");
    assert!(
        aud.iter()
            .all(|audience| audience == client || trusted.contains(audience)),
        "untrusted-audience holds: {aud:?} against {client:?} and {trusted:?}"
    );
}

#[test]
fn the_issue_shape_synthesizes_its_wrong_state_scenario() {
    let result = synthesis(EXCHANGE);
    assert_eq!(refusals_about(&result, SENT), Vec::<String>::new());
    let witness = scenario(&result, SENT);
    assert!(
        requires(witness, NOT_AWAITING),
        "{SENT} requires `not-awaiting`"
    );
}

#[test]
fn each_sibling_is_refuted_through_one_of_its_conjuncts() {
    let result = synthesis(EXCHANGE);
    let witness = scenario(&result, SENT);
    let (send, receive) = sent_and_received(witness);
    let (row, input) = (carries(&send, "nonce"), carries(&receive, "nonce"));
    // nonce-missing: `not defined(input.nonce)` and `defined(nonce)`.
    assert!(input || !row, "nonce-missing holds: {send:#?} {receive:#?}");
    // nonce-mismatch: `defined(input.nonce)` and `defined(nonce) and nonce != input.nonce`.
    assert!(
        !input || !row || send.get("nonce") == receive.get("nonce"),
        "nonce-mismatch holds: {send:#?} {receive:#?}"
    );
    // nonce-not-sent: `defined(input.nonce)` and `not defined(nonce)`.
    assert!(
        !input || row,
        "nonce-not-sent holds: {send:#?} {receive:#?}"
    );
    assert_audiences_refuted(&send, &receive);
    // issuer-mismatch: `issuer != input.iss`.
    assert_eq!(
        send.get("issuer"),
        receive.get("iss"),
        "issuer-mismatch holds"
    );
}

#[test]
fn the_interpreted_target_passes_the_wrong_state_scenario() {
    let model = ir(EXCHANGE);
    let suite = synthesize(&model).suite;
    assert!(
        suite.scenarios.keys().any(|key| key.to_string() == SENT),
        "{SENT} is written"
    );
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).unwrap();
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(model),
        )
        .into_report();
    let run = report
        .scenarios
        .iter()
        .find(|run| run.scenario.to_string() == SENT)
        .unwrap_or_else(|| panic!("{SENT} is run"));
    assert_eq!(
        run.status,
        ess_conformance::report::Status::Passed,
        "{}: {:?}",
        run.scenario,
        run.checks
    );
}

/// `nonce-missing` without its subject guard and `nonce-mismatch` with `defined(nonce)` alone:
/// an input carrying no nonce selects `nonce-missing` on any row, and one carrying a nonce selects
/// `nonce-mismatch` on a row holding one and `nonce-not-sent` on a row holding none. Every row and
/// input leaves some sibling with every conjunct holding.
fn every_conjunct_holds(text: &str) -> String {
    let missing = "      - name: nonce-missing\n        when: {nonce: {defined: false}}\n";
    let mismatch = "        when_subject: {predicate: {all: [{nonce: {defined: true}}, \"nonce != input.nonce\"]}}\n";
    let changed = text
        .replace(
            &format!(
                "{missing}        when_subject: {{predicate: {{nonce: {{defined: true}}}}}}\n"
            ),
            missing,
        )
        .replace(
            mismatch,
            "        when_subject: {predicate: {nonce: {defined: true}}}\n",
        );
    assert_eq!(
        changed.matches("when_subject").count() + 1,
        text.matches("when_subject").count(),
        "nonce-missing's subject guard removed"
    );
    assert!(!changed.contains("nonce != input.nonce"));
    changed
}

#[test]
fn a_sibling_whose_every_conjunct_holds_still_refuses_the_scenario() {
    let result = synthesis(&every_conjunct_holds(EXCHANGE));
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == SENT),
        "{SENT} is not written"
    );
    let refused = refusals_about(&result, SENT);
    let [only] = refused.as_slice() else {
        panic!("{SENT}: one refusal, found {refused:#?}");
    };
    assert!(only.starts_with("ESS-SYNTH-003"), "{only}");
}

/// The command without its three nonce siblings: only the row-guarded ones beside the move.
fn without_nonce_siblings(text: &str) -> String {
    let start = text
        .find("      - name: nonce-missing\n")
        .expect("nonce-missing is declared");
    let end = text
        .find("      - name: issuer-mismatch\n")
        .expect("issuer-mismatch is declared");
    format!("{}{}", &text[..start], &text[end..])
}

#[test]
fn a_quantifier_over_the_input_is_refuted_through_the_row_it_compares_with() {
    // No nonce sibling at all: the audience siblings alone left every candidate refused, since no
    // candidate's `aud` held the row's `client_id`.
    let text = without_nonce_siblings(EXCHANGE);
    let result = synthesis(&text);
    assert_eq!(refusals_about(&result, SENT), Vec::<String>::new());
    let witness = scenario(&result, SENT);
    assert!(
        requires(witness, NOT_AWAITING),
        "{SENT} requires `not-awaiting`"
    );
    let (send, receive) = sent_and_received(witness);
    assert_audiences_refuted(&send, &receive);
    assert_eq!(
        send.get("issuer"),
        receive.get("iss"),
        "issuer-mismatch holds"
    );
}

#[test]
fn an_audience_the_row_cannot_hold_still_refuses_the_scenario() {
    // `audience-mismatch` now also claims every list holding the row's `client_id`: whatever the
    // row and the input, it or the original claim holds, and the scenario stays refused.
    let text = EXCHANGE.replace(
        "{not: {exists: {in: input.aud, as: audience, that: {audience: {eq: {fact: client_id}}}}}}",
        "{any: [{not: {exists: {in: input.aud, as: audience, that: {audience: {eq: {fact: client_id}}}}}}, {exists: {in: input.aud, as: audience, that: {audience: {eq: {fact: client_id}}}}}]}",
    );
    assert_ne!(text, EXCHANGE);
    let result = synthesis(&text);
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|key| key.to_string() == SENT),
        "{SENT} is not written"
    );
    let refused = refusals_about(&result, SENT);
    let [only] = refused.as_slice() else {
        panic!("{SENT}: one refusal, found {refused:#?}");
    };
    assert!(only.starts_with("ESS-SYNTH-003"), "{only}");
}
