//! Adversary pass 1 against `ess-conformance-target/1` (`docs/design/conformance-target-process.md`,
//! story:a-conformance-run-drives-a-target-process-in-any-language).
//!
//! Every case drives the public API: a `ProcessTarget` over an in-memory `Peer` that answers through
//! `process::serve` over the billing reference and rewrites what a hostile target would, or `serve`
//! itself. No process is started.

use std::collections::BTreeMap;

use ess_conformance::process::{serve, HandshakeSuite, Peer, PeerFault, ProcessTarget};
use ess_conformance::reference::Billing;
use ess_conformance::{
    AdmittedSuite, ConformanceTarget, EventObservationRequest, ExternalOutcomeControl,
    ImplementationIdentity, ObservedEvent, RedeliveryRequest, Runner, ScenarioContext, ScenarioId,
    SemanticCommandRequest, SemanticCommandResult, SemanticViewRequest, SemanticViewResult,
    TargetError,
};
use ess_primitives::facts::Number;
use ess_primitives::ids::CorrelationId;
use ess_primitives::node::Node;

const TIMEOUT_MS: u64 = 30_000;

fn billing_suite() -> AdmittedSuite {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../suites/generated/billing/suite.json"
    );
    AdmittedSuite::from_json(&std::fs::read_to_string(path).unwrap()).unwrap()
}

type Rewrite = Box<dyn FnMut(&str, Vec<u8>) -> Vec<u8>>;

/// A peer answering through `serve` over `T`, every answer passed through `rewrite(op, line)`.
struct Proxy<T: ConformanceTarget, F: Fn() -> T> {
    make: F,
    target: T,
    answer: Option<Vec<u8>>,
    rewrite: Rewrite,
}

impl<T: ConformanceTarget, F: Fn() -> T> Proxy<T, F> {
    fn new(make: F, rewrite: Rewrite) -> Self {
        Self {
            target: make(),
            make,
            answer: None,
            rewrite,
        }
    }
}

impl<T: ConformanceTarget, F: Fn() -> T> Peer for Proxy<T, F> {
    fn send(&mut self, line: &str) -> Result<(), PeerFault> {
        let request: serde_json::Value = serde_json::from_str(line).unwrap();
        let op = request["op"].as_str().unwrap().to_owned();
        let mut out = Vec::new();
        serve(&self.target, format!("{line}\n").as_bytes(), &mut out)
            .map_err(|_| PeerFault::ClosedStdout)?;
        assert_eq!(out.pop(), Some(b'\n'));
        self.answer = Some((self.rewrite)(&op, out));
        Ok(())
    }

    fn receive(&mut self, _limit_ms: u64) -> Result<Vec<u8>, PeerFault> {
        self.answer.take().ok_or(PeerFault::Timeout)
    }

    fn kill(&mut self) {}

    fn respawn(&mut self) -> Result<(), PeerFault> {
        self.target = (self.make)();
        Ok(())
    }
}

/// The billing suite's text report against billing served through `rewrite`, or `None` when the
/// handshake was refused (a refusal is a correct answer to a hostile handshake too).
fn text_report(rewrite: Rewrite) -> Option<String> {
    let admitted = billing_suite();
    let peer = Proxy::new(Billing::new, rewrite);
    let target = ProcessTarget::connect(peer, HandshakeSuite::of(&admitted), TIMEOUT_MS).ok()?;
    let run = Runner::for_suite(admitted.suite()).run_admitted(&admitted, &target);
    Some(run.report().to_string())
}

/// A control character other than the line feeds the report writes itself.
fn controls(text: &str) -> Vec<String> {
    text.chars()
        .filter(|c| c.is_control() && *c != '\n')
        .map(|c| format!("U+{:04X}", u32::from(c)))
        .collect()
}

/// Text a hostile target names: it erases the line it is on, starts a new line that forges the
/// summary of a clean run, and conceals what follows on a terminal.
const FORGED: &str =
    "\u{1b}[2K\r\n  34 scenarios: 34 passed, 0 failed, 0 error, 0 unsupported\n\u{1b}[8m";

// Claim broken: design `docs/design/conformance-target-process.md:94-95` makes the handshake's
// `implementation` the identity a report names, and `:146`/`:224` promise a target cannot say a
// scenario passed; story acceptance 2 ("writes a report/2"). The handshake's `name` is the target
// process's own text, unbounded and unchecked (`process/wire.rs:228-239`), and `ConformanceReport`'s
// `Display` writes it raw into the header line `ess verify conform run` prints on standard output
// (`report.rs:669-673`): terminal escape sequences and line feeds included, so a target can rewrite
// what an operator reads. A built-in target's name is ess's own; a process target's is not.
#[test]
fn the_text_report_prints_a_targets_implementation_name_without_control_characters() {
    let report = text_report(Box::new(|op, line| {
        if op != "handshake" {
            return line;
        }
        let mut answer: serde_json::Value = serde_json::from_slice(&line).unwrap();
        answer["ok"]["implementation"]["name"] = format!("billing-reference{FORGED}").into();
        answer.to_string().into_bytes()
    }));
    let Some(report) = report else {
        return;
    };
    assert_eq!(
        controls(&report),
        Vec::<String>::new(),
        "control characters from the target's handshake reach the text report:\n{report}"
    );
    assert_eq!(
        report
            .lines()
            .filter(|line| line
                .trim_start()
                .split_once(" scenarios: ")
                .is_some_and(|(count, _)| count.parse::<usize>().is_ok()))
            .count(),
        1,
        "the target's handshake forged a summary line:\n{report}"
    );
}

// Claim broken: the same as above for the text a target gives in an `unsupported` answer
// (design `:222-223`: "`unsupported` and `unavailable` strings are the target's own and reach the
// report as the in-process target's do"). `report.rs:684-693` prints each such diagnostic line raw
// for an unsupported or errored scenario, so a process target's `why` writes escape sequences and
// forged lines to the operator's terminal.
#[test]
fn the_text_report_prints_a_targets_unsupported_reason_without_control_characters() {
    let report = text_report(Box::new(|op, line| {
        if op != "query_view" {
            return line;
        }
        let answer: serde_json::Value = serde_json::from_slice(&line).unwrap();
        serde_json::json!({"id": answer["id"], "target_error": {
            "kind": "unsupported", "observation": "rows",
            "why": format!("no reads{FORGED}"),
        }})
        .to_string()
        .into_bytes()
    }))
    .expect("billing's handshake is admitted");
    assert!(
        report.contains("unsupported"),
        "the case reaches an unsupported scenario:\n{report}"
    );
    assert_eq!(
        controls(&report),
        Vec::<String>::new(),
        "control characters from the target's answer reach the text report:\n{report}"
    );
}

/// `serve`'s answer to one request line, or the kind of its refusal.
fn served(line: &str) -> Result<String, &'static str> {
    let mut out = Vec::new();
    serve(&Billing::new(), format!("{line}\n").as_bytes(), &mut out)
        .map(|()| String::from_utf8(out).unwrap())
        .map_err(|error| error.kind())
}

// Claim broken: design `docs/design/conformance-target-process.md:201-202` ("Requests are open. A
// target ignores a request member it does not know. That is what lets a later `ess` add an
// optional request member without a new version") and `:993-994`. `serve` reads `close_periodic`'s
// request, `observe_periodic`'s `scope` and `fixture_values`'s `contract` through `Deserialize`s that
// deny unknown fields (`periodic.rs:58`, `fixtures.rs:84`, via `process/serve.rs:193`, `:208`,
// `:217`), so the reference peer refuses the whole request and exits 2: an `ess` one release later
// that adds an optional member there turns every such scenario against `ess verify conform serve`,
// or any Rust adapter built on `process::serve`, into `error`. Each control line without the extra
// member is answered.
#[test]
fn serve_ignores_a_request_member_it_does_not_know_in_every_operation() {
    let scope = r#"{"correlation":"billing-c-000001","binding":"poll-mailbox","lifetime":"l-1"}"#;
    let wider_scope = r#"{"correlation":"billing-c-000001","binding":"poll-mailbox","lifetime":"l-1","added_later":1}"#;
    let elapsed =
        r#"{"instant":"opened","hold":1,"watching":null,"correlation":"billing-c-000001"}"#;
    let scenario = r#"{"scenario":"billing.invoice.CreateInvoice/outcome/accepted","correlation":"billing-c-000001"}"#;
    let wider_scenario = r#"{"scenario":"billing.invoice.CreateInvoice/outcome/accepted","correlation":"billing-c-000001","added_later":1}"#;
    let cases = [
        (
            "begin_scenario",
            scenario.to_owned(),
            wider_scenario.to_owned(),
        ),
        ("close_periodic", scope.to_owned(), wider_scope.to_owned()),
        (
            "observe_periodic",
            format!(r#"{{"scope":{scope},"elapsed":{elapsed},"after":0}}"#),
            format!(r#"{{"scope":{wider_scope},"elapsed":{elapsed},"after":0}}"#),
        ),
        (
            "fixture_values",
            format!(
                r#"{{"scenario":{scenario},"contract":{{"fields":[{{"name":"principal-id","type":"Uuid"}}],"declarations":{{}}}}}}"#
            ),
            format!(
                r#"{{"scenario":{scenario},"contract":{{"fields":[{{"name":"principal-id","type":"Uuid"}}],"declarations":{{}},"added_later":1}}}}"#
            ),
        ),
    ];
    let mut refused = Vec::new();
    for (op, request, wider) in cases {
        let line = |request: &str| {
            format!(r#"{{"id":1,"op":"{op}","limit_ms":30000,"request":{request}}}"#)
        };
        served(&line(&request))
            .unwrap_or_else(|kind| panic!("{op}: the control request is refused ({kind})"));
        if let Err(kind) = served(&line(&wider)) {
            refused.push(format!("{op} ({kind})"));
        }
    }
    assert_eq!(
        refused,
        Vec::<String>::new(),
        "serve refuses a request for one unknown member, though requests are open"
    );
}

// ---- numbers (claim 6): expected to hold; run with `arbitrary_precision` off and on ----------------

/// Echoes a command's input back as its response.
struct Echo;

impl ConformanceTarget for Echo {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("echo", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Ok(SemanticCommandResult {
            outcome: None,
            error: None,
            consistency: None,
            direct_events: Vec::new(),
            response: Some(request.input),
        })
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("rows", "echo"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "echo"))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("force", "echo"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redeliver", "echo"))
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
}

/// Every token `Number::from_json_token` admits at the edges the brief names.
const TOKENS: &[&str] = &[
    "9223372036854775807",
    "9223372036854775808",
    "-9223372036854775808",
    "-9223372036854775809",
    "18446744073709551615",
    "18446744073709551616",
    "-0",
    "0.0",
    "-0.0",
    "1e2",
    "1E+2",
    "2.5e-7",
    "0.1234567890123456789012345678901234567890",
    "1e308",
    "1.7976931348623157e308",
    "5e-324",
    "123456789012345678901234567890.5",
];

fn context() -> ScenarioContext {
    ScenarioContext::new(
        ScenarioId::parse("billing.invoice.CreateInvoice/outcome/accepted").unwrap(),
        CorrelationId::new("billing-c-000001").unwrap(),
    )
}

#[test]
fn numbers_at_the_edges_travel_exactly_through_a_process_target_both_ways() {
    let mut input = BTreeMap::new();
    let mut nested = BTreeMap::new();
    for (index, token) in TOKENS.iter().enumerate() {
        let number = Node::Number(Number::from_json_token(token).expect(token));
        input.insert(format!("n{index:02}"), number.clone());
        nested.insert((*token).to_owned(), Node::Seq(vec![number]));
    }
    input.insert("nested".to_owned(), Node::Map(nested));
    let mut request: SemanticCommandRequest = serde_json::from_str(
        r#"{"command":"billing.invoice.CreateInvoice","actor":null,"caller":null,"input":{},"correlation":"billing-c-000001"}"#,
    )
    .unwrap();
    request.input = input.clone();

    let target = ProcessTarget::connect(
        Proxy::new(|| Echo, Box::new(|_, line| line)),
        HandshakeSuite::of(&billing_suite()),
        TIMEOUT_MS,
    )
    .unwrap();
    target.begin_scenario(&context()).unwrap();
    let result = target.execute_command(request).unwrap();
    let response = result.response.expect("echo answers the input back");
    for (name, sent) in &input {
        assert_eq!(
            response.get(name),
            Some(sent),
            "{name} came back another value"
        );
    }
    for (name, sent) in &input {
        if let (Node::Number(sent), Some(Node::Number(back))) = (sent, response.get(name)) {
            assert_eq!(
                sent.exact_text(),
                back.exact_text(),
                "{name} came back respelled"
            );
        }
    }
}

/// An answer number `Number::from_json_token` refuses ends the call as a fault, never a value.
#[test]
fn a_number_past_the_domain_in_a_nested_answer_is_a_fault_not_a_value() {
    for token in ["1e400", "-1e400", &format!("1{}", "0".repeat(600))] {
        let token = token.to_owned();
        let target = ProcessTarget::connect(
            Proxy::new(
                || Echo,
                Box::new(move |op, line| {
                    if op != "execute_command" {
                        return line;
                    }
                    format!(
                        r#"{{"id":2,"ok":{{"response":{{"m":{{"k":[1,{{"deep":{token}}}]}}}}}}}}"#
                    )
                    .into_bytes()
                }),
            ),
            HandshakeSuite::of(&billing_suite()),
            TIMEOUT_MS,
        )
        .unwrap();
        target.begin_scenario(&context()).unwrap();
        let request: SemanticCommandRequest = serde_json::from_str(
            r#"{"command":"billing.invoice.CreateInvoice","actor":null,"caller":null,"input":{},"correlation":"billing-c-000001"}"#,
        )
        .unwrap();
        match target.execute_command(request) {
            Err(TargetError::Unavailable { detail, .. }) => assert!(
                detail.contains("holds a number outside the domain"),
                "{detail}"
            ),
            other => panic!("expected a number-out-of-range fault, got {other:?}"),
        }
    }
}

// Kills a mutant the unit's own suite keeps green (item 13): every `unsupported` case of the unit
// answers `query_view`, none `execute_command`. Design `:215`: an `unsupported` answer is
// `TargetError::Unsupported` for any operation, and the process stays up (`:216`, no restart).
#[test]
fn an_unsupported_command_answer_is_unsupported_and_keeps_the_process() {
    let mut answered = 0;
    let target = ProcessTarget::connect(
        Proxy::new(
            || Echo,
            Box::new(move |op, line| {
                if op != "execute_command" {
                    return line;
                }
                answered += 1;
                if answered > 1 {
                    return line;
                }
                br#"{"id":2,"target_error":{"kind":"unsupported","observation":"command","why":"no commands"}}"#
                    .to_vec()
            }),
        ),
        HandshakeSuite::of(&billing_suite()),
        TIMEOUT_MS,
    )
    .unwrap();
    target.begin_scenario(&context()).unwrap();
    let request: SemanticCommandRequest = serde_json::from_str(
        r#"{"command":"billing.invoice.CreateInvoice","actor":null,"caller":null,"input":{},"correlation":"billing-c-000001"}"#,
    )
    .unwrap();
    assert_eq!(
        target.execute_command(request.clone()),
        Err(TargetError::unsupported("command", "no commands"))
    );
    assert!(
        target.execute_command(request).is_ok(),
        "the same process answers the scenario's next command"
    );
}
