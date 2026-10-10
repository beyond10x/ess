//! A response or struct declared `undeclared_fields: ignored` (`ess/24`, beyond10x/ess#500)
//! admits keys it does not declare in every observer, and nowhere else.
//!
//! The flag travels on `expect_direct_response` and `expect_response_payload` as two members
//! omitted when closed — `undeclared_fields: "ignored"` for the response object and
//! `undeclared_fields_ignored` for the opened struct declarations — and a suite carrying either is
//! `ess-conformance/48` (`/49` with coverage). Declared fields keep their presence and type checks.
//! The Rust, Go and TypeScript runners give the same verdict on the same returned values, and a
//! reader that admits only through `/47` refuses a `/48` suite by version before any callback.

mod support_go;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::ir::ResolvedCommand;
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::report::{CheckCode, Status};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{
    AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig, ScenarioResult,
};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const OUTCOME: &str = "catalog.orders.PlaceOrder/outcome/placed";
const COMMAND: &str = "catalog.orders.PlaceOrder";

/// One response observer: the reproduction's observation, opened as named, against an answer.
type Observer = fn(Opened, &str) -> Result<(), String>;

/// Where the specification writes `undeclared_fields: ignored`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Opened {
    /// Nowhere: every object stays closed.
    Nothing,
    /// On the command, opening its response object: the issue's reproduction with the key.
    Response,
    /// On `catalog.orders.Vendor`, a struct the response reaches beside the closed
    /// `catalog.orders.Shipping`.
    Vendor,
}

/// The reproduction of beyond10x/ess#500 (`.engineering/repro/500/spec/`) under `ess/24`, with the
/// key where `opened` says, two struct members when `nested`, and a response-mapped event when
/// `mapped`.
fn model(opened: Opened, nested: bool, mapped: bool) -> String {
    let key = "    undeclared_fields: ignored\n";
    let command_key = if opened == Opened::Response { key } else { "" };
    let vendor_key = if opened == Opened::Vendor { key } else { "" };
    let types = if nested {
        format!(
            "types:
  - name: catalog.orders.Vendor
    kind: struct
{vendor_key}    fields:
      - {{name: vendor_id, type: String}}
  - name: catalog.orders.Shipping
    kind: struct
    fields:
      - {{name: carrier, type: String}}
"
        )
    } else {
        String::new()
    };
    let members = if nested {
        "      - {name: vendor, type: catalog.orders.Vendor}\n      - {name: shipping, type: catalog.orders.Shipping}\n"
    } else {
        ""
    };
    let (events, outcome) = if mapped {
        (
            "events:\n  - name: catalog.orders.OrderPlaced\n    fields:\n      - {name: order_ref, type: String}\n",
            "        emits: [catalog.orders.OrderPlaced]\n        payload:\n          catalog.orders.OrderPlaced:\n            order_ref: {response: order_ref}\n",
        )
    } else {
        ("", "")
    };
    format!(
        "format: ess/24
system: catalog
version: v1
domain: catalog.orders

{types}{events}
actors:
  - name: catalog.orders.Buyer
    may:
      - catalog.orders.PlaceOrder

commands:
  - name: catalog.orders.PlaceOrder
{command_key}    input:
      - {{name: item, type: String}}
    response:
      - {{name: order_ref, type: String}}
{members}    outcomes:
      - name: placed
        returns: true
{outcome}"
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn command(ir: &EssIr) -> &ResolvedCommand {
    &ir.commands()[&COMMAND.parse().unwrap()]
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result.refusals.iter().map(ToString::to_string).collect()
}

/// The synthesized suite, asserted to hold the outcome scenario.
fn synthesized(opened: Opened, nested: bool, mapped: bool) -> ConformanceSuite {
    let result = synthesize(&ir(&model(opened, nested, mapped)));
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == OUTCOME),
        "{:#?}",
        refusals(&result)
    );
    result.suite
}

/// A returned object, from JSON.
fn object(json: &str) -> BTreeMap<String, Node> {
    serde_json::from_str(json).unwrap_or_else(|error| panic!("{error}: {json}"))
}

/// The observation step `tag` of the outcome scenario, as written.
fn step(suite: &ConformanceSuite, tag: &str) -> serde_json::Value {
    let json: serde_json::Value =
        serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap();
    json["scenarios"][OUTCOME]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["step"] == tag)
        .unwrap_or_else(|| panic!("no {tag} step"))
        .clone()
}

// ---- synthesis ------------------------------------------------------------------------------

#[test]
fn the_reproduction_with_the_key_is_suite_48_and_opens_the_response_object() {
    let suite = synthesized(Opened::Response, false, false);
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/48"
    );
    let response = &step(&suite, "expect_direct_response")["response"];
    assert_eq!(response["undeclared_fields"], "ignored", "{response:#}");
    assert!(
        response.get("undeclared_fields_ignored").is_none(),
        "{response:#}"
    );
}

#[test]
fn an_opened_struct_is_named_and_its_closed_sibling_is_not() {
    let suite = synthesized(Opened::Vendor, true, false);
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/48"
    );
    let response = &step(&suite, "expect_direct_response")["response"];
    assert_eq!(
        response["undeclared_fields_ignored"],
        serde_json::json!(["catalog.orders.Vendor"]),
        "{response:#}"
    );
    assert!(response.get("undeclared_fields").is_none(), "{response:#}");
}

#[test]
fn a_response_payload_observation_carries_the_flag_too() {
    let suite = synthesized(Opened::Response, false, true);
    let payload = &step(&suite, "expect_response_payload")["response"];
    assert_eq!(payload["undeclared_fields"], "ignored", "{payload:#}");
}

#[test]
fn a_closed_specification_keeps_its_format_and_its_bytes() {
    for (nested, mapped) in [(false, false), (true, false), (false, true)] {
        let result = synthesize(&ir(&model(Opened::Nothing, nested, mapped)));
        assert_eq!(
            result.suite.provenance.suite_version.to_string(),
            "ess-conformance/34",
            "{nested} {mapped}"
        );
        assert!(
            !result
                .suite
                .to_canonical_json()
                .unwrap()
                .contains("undeclared_fields"),
            "{nested} {mapped}"
        );
    }
}

// ---- the observers --------------------------------------------------------------------------

/// The direct-return observer for `opened`, and its verdict on each returned object.
fn direct(opened: Opened, nested: bool, actual: &str) -> Result<(), String> {
    let ir = ir(&model(opened, nested, false));
    let observation =
        ess_conformance::direct_response::Observation::of(&ir, command(&ir), None, BTreeMap::new())
            .unwrap_or_else(|error| panic!("{error}"));
    observation.compare(Some(&object(actual)))
}

/// The response-payload observer for `opened`, its verdict on `actual` and a payload carrying
/// `order_ref: "o-1"`.
fn payload(opened: Opened, actual: &str) -> Result<(), String> {
    let ir = ir(&model(opened, false, true));
    let command = command(&ir);
    let observations =
        ess_conformance::response::Observation::of(&ir, command, &command.outcomes[0])
            .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(observations.len(), 1, "{observations:#?}");
    observations[0].compare(Some(&object(actual)), &object(r#"{"order_ref":"o-1"}"#))
}

#[test]
fn an_opened_response_admits_an_undeclared_key_and_still_checks_its_declared_field() {
    let observers: [Observer; 2] = [|opened, actual| direct(opened, false, actual), payload];
    for observe in observers {
        assert_eq!(
            observe(Opened::Response, r#"{"order_ref":"o-1","extension":"x"}"#),
            Ok(())
        );
        assert_eq!(observe(Opened::Response, r#"{"order_ref":"o-1"}"#), Ok(()));
        // A missing declared field, and one of the wrong type, still fail.
        assert!(observe(Opened::Response, r#"{"extension":"x"}"#).is_err());
        assert!(observe(Opened::Response, r#"{"order_ref":7,"extension":"x"}"#).is_err());
        // A closed response still refuses the undeclared key.
        let closed = observe(Opened::Nothing, r#"{"order_ref":"o-1","extension":"x"}"#)
            .expect_err("a closed response refuses an undeclared key");
        assert!(closed.contains("undeclared field"), "{closed}");
    }
}

#[test]
fn an_opened_struct_admits_extras_at_that_struct_only() {
    let shipping = r#"{"carrier":"c-1"}"#;
    let vendor = r#"{"vendor_id":"v-1","region":"eu"}"#;
    let returned = |vendor: &str, shipping: &str| {
        format!(r#"{{"order_ref":"o-1","vendor":{vendor},"shipping":{shipping}}}"#)
    };
    assert_eq!(
        direct(Opened::Vendor, true, &returned(vendor, shipping)),
        Ok(())
    );
    // The closed sibling still refuses an extra member.
    assert!(direct(
        Opened::Vendor,
        true,
        &returned(vendor, r#"{"carrier":"c-1","tracking":"t"}"#)
    )
    .is_err());
    // The opened struct still requires and types its declared member.
    assert!(direct(
        Opened::Vendor,
        true,
        &returned(r#"{"region":"eu"}"#, shipping)
    )
    .is_err());
    assert!(direct(
        Opened::Vendor,
        true,
        &returned(r#"{"vendor_id":7}"#, shipping)
    )
    .is_err());
    // Opening the struct does not open the response object.
    assert!(direct(
        Opened::Vendor,
        true,
        &format!(
            r#"{{"order_ref":"o-1","vendor":{vendor},"shipping":{shipping},"extension":"x"}}"#
        )
    )
    .is_err());
    // Without the key the same struct is closed.
    assert!(direct(Opened::Nothing, true, &returned(vendor, shipping)).is_err());
}

// ---- the native runner ----------------------------------------------------------------------

/// An order service answering every `PlaceOrder` with `response`, and publishing
/// `order_ref: "o-1"` when `emits`.
struct Orders {
    response: &'static str,
    emits: bool,
}

impl ConformanceTarget for Orders {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("orders-500", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let command: &CommandRef = &request.command;
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            command.clone(),
            OutcomeName::new("placed").unwrap(),
        ));
        if self.emits {
            result = result.emitting(
                ObservedEvent::new("catalog.orders.OrderPlaced".parse().unwrap())
                    .with("order_ref", Node::from("o-1")),
            );
        }
        result.response = Some(object(self.response));
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported(
            "view",
            "the catalog declares none",
        ))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "none"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("binding", "none"))
    }
}

fn run(suite: &ConformanceSuite, target: &Orders) -> BTreeMap<String, ScenarioResult> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect()
}

/// Each specification, and each returned object with the verdict every runner must give it.
const VECTORS: [(Opened, bool, &str, &str); 9] = [
    (
        Opened::Response,
        false,
        r#"{"order_ref":"o-1","extension":"x"}"#,
        "passed",
    ),
    (Opened::Response, false, r#"{"extension":"x"}"#, "failed"),
    (
        Opened::Response,
        false,
        r#"{"order_ref":7,"extension":"x"}"#,
        "failed",
    ),
    (
        Opened::Nothing,
        false,
        r#"{"order_ref":"o-1","extension":"x"}"#,
        "failed",
    ),
    (Opened::Nothing, false, r#"{"order_ref":"o-1"}"#, "passed"),
    (
        Opened::Vendor,
        true,
        r#"{"order_ref":"o-1","vendor":{"vendor_id":"v-1","region":"eu"},"shipping":{"carrier":"c-1"}}"#,
        "passed",
    ),
    (
        Opened::Vendor,
        true,
        r#"{"order_ref":"o-1","vendor":{"vendor_id":"v-1"},"shipping":{"carrier":"c-1","tracking":"t"}}"#,
        "failed",
    ),
    (
        Opened::Vendor,
        true,
        r#"{"order_ref":"o-1","vendor":{"region":"eu"},"shipping":{"carrier":"c-1"}}"#,
        "failed",
    ),
    (
        Opened::Nothing,
        true,
        r#"{"order_ref":"o-1","vendor":{"vendor_id":"v-1","region":"eu"},"shipping":{"carrier":"c-1"}}"#,
        "failed",
    ),
];

#[test]
fn the_native_runner_passes_an_extension_member_only_where_the_specification_admits_it() {
    for (opened, nested, response, verdict) in VECTORS {
        let suite = synthesized(opened, nested, false);
        let results = run(
            &suite,
            &Orders {
                response,
                emits: false,
            },
        );
        let result = &results[OUTCOME];
        assert_eq!(
            result.status.to_string(),
            verdict,
            "{opened:?} {response}: {result:#?}"
        );
        if verdict == "failed" {
            assert!(
                result
                    .checks
                    .iter()
                    .any(|check| check.status == Status::Failed && check.code == CheckCode::Payload),
                "{opened:?} {response}: {result:#?}"
            );
        }
    }
}

#[test]
fn the_payload_observation_admits_the_extension_on_its_own() {
    let mut suite = synthesized(Opened::Response, false, true);
    for scenario in suite.scenarios.values_mut() {
        scenario.steps.retain(|step| {
            !matches!(
                step,
                ess_conformance::ScenarioStep::ExpectDirectResponse { .. }
            )
        });
    }
    for (response, verdict) in [
        (r#"{"order_ref":"o-1","extension":"x"}"#, Status::Passed),
        (r#"{"order_ref":7,"extension":"x"}"#, Status::Failed),
    ] {
        let results = run(
            &suite,
            &Orders {
                response,
                emits: true,
            },
        );
        assert_eq!(
            results[OUTCOME].status, verdict,
            "{response}: {:#?}",
            results[OUTCOME]
        );
    }
}

#[test]
fn the_model_interpreter_passes_its_own_synthesized_suite() {
    for (opened, nested, mapped) in [
        (Opened::Response, false, false),
        (Opened::Response, false, true),
        (Opened::Vendor, true, false),
    ] {
        let suite = synthesized(opened, nested, mapped);
        let admitted = AdmittedSuite::from_suite(&suite).unwrap();
        let run = Runner::for_suite(&suite).run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir(&model(opened, nested, mapped))),
        );
        let outcome = run
            .scenarios
            .iter()
            .find(|result| result.scenario.to_string() == OUTCOME)
            .expect("the outcome ran");
        assert_eq!(outcome.status, Status::Passed, "{opened:?}: {outcome:#?}");
    }
}

// ---- formats and admission ------------------------------------------------------------------

/// The suite's canonical JSON, edited.
fn edited(suite: &ConformanceSuite, edit: impl FnOnce(&mut serde_json::Value)) -> String {
    let mut document: serde_json::Value =
        serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap();
    edit(&mut document);
    serde_json::to_string(&document).unwrap()
}

/// The suite's canonical JSON with its version relabelled to `major`.
fn relabelled(suite: &ConformanceSuite, major: u32) -> String {
    edited(suite, |document| {
        document["provenance"]["suite_version"] = format!("ess-conformance/{major}").into();
    })
}

#[test]
fn an_opened_response_under_an_older_major_is_refused_by_name() {
    for (opened, nested, mapped) in [
        (Opened::Response, false, false),
        (Opened::Response, false, true),
        (Opened::Vendor, true, false),
    ] {
        let suite = synthesized(opened, nested, mapped);
        let Err(error) = AdmittedSuite::from_json(&relabelled(&suite, 46)) else {
            panic!("a /46 suite cannot carry an opened response");
        };
        let error = error.to_string();
        assert!(error.contains("suite/48 or /49"), "{opened:?}: {error}");
    }
}

#[test]
fn a_reader_of_48_refuses_a_later_major_by_version() {
    let suite = synthesized(Opened::Response, false, false);
    assert!(AdmittedSuite::from_json(&relabelled(&suite, 48)).is_ok());
    assert!(AdmittedSuite::from_json(&relabelled(&suite, 50)).is_err());
}

/// One forgery of a direct response observation.
type Forge = Box<dyn Fn(&mut serde_json::Value)>;

/// The outcome scenario's direct response observation inside a suite document.
fn direct_response(document: &mut serde_json::Value) -> &mut serde_json::Value {
    let step = document["scenarios"][OUTCOME]["steps"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|step| step["step"] == "expect_direct_response")
        .unwrap();
    &mut step["response"]
}

#[test]
fn a_present_member_never_spells_closed_or_names_a_stranger() {
    let suite = synthesized(Opened::Vendor, true, false);
    let forged: [(&str, Forge); 4] = [
        (
            "refused spelled out",
            Box::new(|value: &mut serde_json::Value| value["undeclared_fields"] = "refused".into()),
        ),
        (
            "an empty list",
            Box::new(|value: &mut serde_json::Value| {
                value["undeclared_fields_ignored"] = serde_json::json!([]);
            }),
        ),
        (
            "a name twice",
            Box::new(|value: &mut serde_json::Value| {
                value["undeclared_fields_ignored"] =
                    serde_json::json!(["catalog.orders.Vendor", "catalog.orders.Vendor"]);
            }),
        ),
        (
            "a name that is no declaration",
            Box::new(|value: &mut serde_json::Value| {
                value["undeclared_fields_ignored"] = serde_json::json!(["catalog.orders.Absent"]);
            }),
        ),
    ];
    for (label, forge) in forged {
        let json = edited(&suite, |document| forge(direct_response(document)));
        assert!(
            AdmittedSuite::from_json(&json).is_err(),
            "{label} is admitted"
        );
    }
}

#[test]
fn the_coverage_counterpart_is_49() {
    use ess_conformance::coverage::{Origins, Scope};
    let input = ess_conformance::coverage_build::build(
        &ir(&model(Opened::Response, false, false)),
        &[],
        Scope::System,
        Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        input
            .selected()
            .suite()
            .provenance
            .suite_version
            .to_string(),
        "ess-conformance/49"
    );
    let compared = support_go::compare_input(
        "coverage-49",
        &input,
        Orders {
            response: r#"{"order_ref":"o-1","extension":"x"}"#,
            emits: false,
        },
        &support_go::Options::default(),
    );
    let verdicts = support_go::assert_compared("coverage-49", compared);
    assert_eq!(verdicts[OUTCOME], "passed");
}

fn scratch(label: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("undeclared-fields-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

// ---- Go ------------------------------------------------------------------------------------

#[test]
fn the_go_runner_gives_the_native_verdicts_on_the_same_vectors() {
    for (index, (opened, nested, response, verdict)) in VECTORS.into_iter().enumerate() {
        let suite = synthesized(opened, nested, false);
        let label = format!("go-{index}");
        let go = support_go::assert_parity(
            &label,
            &suite,
            Orders {
                response,
                emits: false,
            },
        );
        assert_eq!(go[OUTCOME], verdict, "{label}: {opened:?} {response}");
    }
    let suite = synthesized(Opened::Response, false, true);
    let go = support_go::assert_parity(
        "go-mapped",
        &suite,
        Orders {
            response: r#"{"order_ref":"o-1","extension":"x"}"#,
            emits: true,
        },
    );
    assert_eq!(go[OUTCOME], "passed");
}

#[test]
fn an_older_go_reader_refuses_a_suite_48_by_version() {
    let suite = synthesized(Opened::Response, false, false);
    let directory =
        support_go::package("older-reader-48", &suite, &[support_go::TRANSCRIPT_TARGET]);
    let runtime = directory.join("essconform/runtime.go");
    let text = std::fs::read_to_string(&runtime).unwrap();
    assert!(
        text.contains("const newestSuiteMajor = 49\n"),
        "the runtime reads /49"
    );
    std::fs::write(
        &runtime,
        text.replace(
            "const newestSuiteMajor = 49\n",
            "const newestSuiteMajor = 47\n",
        ),
    )
    .unwrap();
    let recorder = support_go::Recorder::new(Orders {
        response: r#"{"order_ref":"o-1","extension":"x"}"#,
        emits: false,
    });
    let replayed = support_go::replay(&directory, &recorder, &[]);
    std::fs::remove_dir_all(&directory).unwrap();
    assert!(!replayed.go.success, "{}", replayed.go.log);
    assert_eq!(replayed.go.outcomes.len(), 0, "{}", replayed.go.log);
    assert!(
        replayed.go.log.contains("ess-conformance/48"),
        "{}",
        replayed.go.log
    );
}

// ---- TypeScript ----------------------------------------------------------------------------

#[test]
fn the_typescript_runner_gives_the_native_verdicts_on_the_same_vectors() {
    for (index, (opened, nested, response, verdict)) in VECTORS.into_iter().enumerate() {
        let suite = synthesized(opened, nested, false);
        let label = format!("ts-{index}");
        let typescript = typescript_parity(
            &label,
            &suite,
            Orders {
                response,
                emits: false,
            },
        );
        assert_eq!(
            typescript[OUTCOME], verdict,
            "{label}: {opened:?} {response}"
        );
    }
    let suite = synthesized(Opened::Response, false, true);
    let typescript = typescript_parity(
        "ts-mapped",
        &suite,
        Orders {
            response: r#"{"order_ref":"o-1","extension":"x"}"#,
            emits: true,
        },
    );
    assert_eq!(typescript[OUTCOME], "passed");
}

#[test]
fn an_older_typescript_reader_refuses_a_suite_48_by_version() {
    let suite = synthesized(Opened::Response, false, false);
    let package = typescript_package("older-reader-48", &suite);
    let runtime = package.join("dist/runtime.js");
    let text = std::fs::read_to_string(&runtime).unwrap();
    let entry = "'ess-conformance/48': 48,";
    assert!(text.contains(entry), "the TypeScript runtime reads /48");
    std::fs::write(&runtime, text.replace(entry, "")).unwrap();
    let (log, report) = typescript_run(
        &package,
        &suite,
        Orders {
            response: r#"{"order_ref":"o-1","extension":"x"}"#,
            emits: false,
        },
    );
    let _ = std::fs::remove_dir_all(package.parent().unwrap());
    assert!(report.is_none(), "{log}");
    assert!(log.contains("ess-conformance/48"), "{log}");
}

/// Replays a recorded transcript (`support_go::Recorder`) as a TypeScript target.
const DRIVER: &str = r"
import {readFileSync, writeFileSync} from 'node:fs';
import {runWith, unsupported} from './dist/runtime.js';
const [suiteFile, transcriptFile, divergenceFile] = process.argv.slice(2);
const transcript = JSON.parse(readFileSync(transcriptFile, 'utf8'));
const divergences = [];
const target = () => {
  let scenario = '';
  let used = new Map();
  const next = (method, key) => {
    const matching = (transcript[scenario] ?? []).filter(e => e.method === method && e.key === key);
    const n = used.get(method + '\0' + key) ?? 0;
    used.set(method + '\0' + key, n + 1);
    const entry = matching[n];
    if (entry === undefined) {
      divergences.push(`${scenario}: ${method} \`${key}\` call ${n + 1} was never made by the reference runner`);
      throw new Error('transcript divergence');
    }
    if (entry.error === 'unsupported') throw unsupported('recorded');
    if (entry.error !== null) throw new Error(entry.error);
    return entry.result;
  };
  return {
    identity: async () => ({name: 'transcript', version: '1'}),
    beginScenario: async context => { scenario = context.scenario; used = new Map(); },
    endScenario: async () => {},
    executeCommand: async request => {
      const r = next('execute_command', request.command);
      return {
        outcome: r.outcome ?? undefined, error: r.error ?? undefined,
        consistency: r.consistency ?? undefined, directEvents: r.direct_events ?? [],
        response: r.response ?? undefined,
      };
    },
    queryView: async request => {
      const r = next('query_view', request.view);
      return {rows: r.rows ?? [], total: r.total ?? undefined};
    },
    observeEvents: async request => next('observe_events', request.event)
      .map(event => ({event: event.event, payload: event.payload})),
    configureExternalOutcome: async () => { throw unsupported('none is external'); },
    redeliverEvent: async () => { throw unsupported('not needed'); },
    observeInvocations: async () => { throw unsupported('not needed'); },
  };
};
const scope = {diagnostic() {}, skip() {}, async test(_name, body) { try { await body(this); } catch {} }};
try { await runWith(scope, target, readFileSync(suiteFile, 'utf8')); }
catch (error) { console.error(String(error)); process.exitCode = 2; }
writeFileSync(divergenceFile, divergences.join('\n'));
";

/// The TypeScript runtime package for `suite`, compiled.
fn typescript_package(label: &str, suite: &ConformanceSuite) -> PathBuf {
    let directory = scratch(&format!("ts-{label}"));
    for artifact in ess_conformance::ts::emit(suite).unwrap_or_else(|error| panic!("{error}")) {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    let package = directory.join("essconform");
    let mut compile = Command::new("tsc");
    if let Some(modules) = std::env::var_os("ESS_TYPES_NODE") {
        compile
            .arg("--typeRoots")
            .arg(Path::new(&modules).join("@types"));
    }
    let output = compile
        .args(["--project", "tsconfig.json", "--noCheck"])
        .current_dir(&package)
        .output()
        .expect("the TypeScript compiler runs");
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(package.join("transcript.mjs"), DRIVER).unwrap();
    package
}

/// Runs the compiled package against `target`'s recorded answers: the log and the report, if any.
fn typescript_run(
    package: &Path,
    suite: &ConformanceSuite,
    target: Orders,
) -> (String, Option<serde_json::Value>) {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let recorder = support_go::Recorder::new(target);
    let _ = Runner::new(
        RunnerConfig::default(),
        AdvancingClock::default(),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &recorder);
    let suite_file = package.join("suite.json");
    std::fs::write(&suite_file, admitted.original_json()).unwrap();
    let transcript = package.join("transcript.json");
    std::fs::write(&transcript, recorder.transcript().to_string()).unwrap();
    let divergence = package.join("divergence.txt");
    let report = package.join("report.json");
    let output = Command::new("node")
        .arg(package.join("transcript.mjs"))
        .arg(&suite_file)
        .arg(&transcript)
        .arg(&divergence)
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .output()
        .expect("node runs");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let divergences = std::fs::read_to_string(&divergence).unwrap_or_default();
    assert_eq!(divergences, "", "{log}");
    let report = std::fs::read(&report)
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap());
    (log, report)
}

/// The TypeScript verdicts against `target`'s recorded answers, asserted equal to the Rust
/// reference's, which are returned.
fn typescript_parity(
    label: &str,
    suite: &ConformanceSuite,
    target: Orders,
) -> BTreeMap<String, String> {
    let rust: BTreeMap<String, String> = run(suite, &target)
        .into_iter()
        .map(|(id, result)| (id, result.status.to_string()))
        .collect();
    let package = typescript_package(label, suite);
    let (log, report) = typescript_run(&package, suite, target);
    let _ = std::fs::remove_dir_all(package.parent().unwrap());
    let report = report.unwrap_or_else(|| panic!("{label}: no report\n{log}"));
    let mut typescript = BTreeMap::new();
    for (status, ids) in report["outcomes"].as_object().expect("report/2 outcomes") {
        for id in ids.as_array().expect("ids") {
            typescript.insert(id.as_str().unwrap().to_owned(), status.clone());
        }
    }
    assert_eq!(
        typescript, rust,
        "{label}: per-scenario verdicts, TypeScript (left) and Rust (right)\n{log}"
    );
    rust
}
