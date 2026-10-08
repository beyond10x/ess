//! A returned String-newtype value is checked against its `alphabet:`, `prefix:` and `value`
//! invariants instead of costing its outcome every scenario (beyond10x/ess#499).
//!
//! The rules travel on `expect_direct_response` and `expect_response_payload` as the closed
//! `constraints` carrier the one-time profile already uses, and a suite carrying one is
//! `ess-conformance/46` (`/47` with coverage). The Rust, Go and TypeScript runners give the same
//! verdict on the same returned values. Record invariants and reading-attached response types stay
//! refused, each by name.

mod support_go;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

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

const OUTCOME: &str = "catalog.items.Register/outcome/registered";
const ALPHABET: &str = "    alphabet: 'ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-'\n";
const PREFIX: &str = "    prefix: 'IT-'\n";
const INVARIANT: &str = "    invariants:\n      - 'value.count >= 4'\n";

/// The catalog with `rule` as the body of `catalog.items.Code`, and the command's `emits`.
fn model(rule: &str, mapped: bool) -> String {
    let (events, outcome) = if mapped {
        (
            "events:\n  - name: catalog.items.Registered\n    fields:\n      - {name: code, type: catalog.items.Code}\n",
            "        emits: [catalog.items.Registered]\n        payload:\n          catalog.items.Registered:\n            code: {response: code}\n",
        )
    } else {
        ("", "")
    };
    format!(
        "format: ess/23
system: catalog
version: v1
domain: catalog.items

components:
  - component: catalog-service
    owns:
      domains: [catalog.items]
    accepts:
      commands: [catalog.items.Register]
    reached_by: network

types:
  - name: catalog.items.Code
    kind: newtype
    of: String
{rule}{events}
commands:
  - name: catalog.items.Register
    input:
      - {{name: label, type: String}}
    response:
      - {{name: code, type: catalog.items.Code}}
    outcomes:
      - name: registered
        returns: true
{outcome}"
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result.refusals.iter().map(ToString::to_string).collect()
}

/// The synthesized suite for `rule`, asserted to hold the outcome scenario and no ESS-SYNTH-001.
fn synthesized(rule: &str, mapped: bool) -> ConformanceSuite {
    let result = synthesize(&ir(&model(rule, mapped)));
    let refused = refusals(&result);
    assert!(
        refused
            .iter()
            .all(|refusal| !refusal.contains("ESS-SYNTH-001")),
        "{refused:#?}"
    );
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == OUTCOME),
        "{refused:#?}"
    );
    result.suite
}

// ---- synthesis ------------------------------------------------------------------------------

#[test]
fn an_alphabet_on_a_returned_newtype_keeps_the_outcome_scenario_at_suite_46() {
    let suite = synthesized(ALPHABET, false);
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/46"
    );
    let bytes = suite.to_canonical_json().unwrap();
    let json: serde_json::Value = serde_json::from_str(&bytes).unwrap();
    let step = json["scenarios"][OUTCOME]["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| step["step"] == "expect_direct_response")
        .expect("a direct response observation")
        .clone();
    assert_eq!(
        step["response"]["constraints"]["catalog.items.Code"],
        serde_json::json!({
            "alphabet": "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-",
            "prefix": null,
            "invariants": []
        })
    );
}

#[test]
fn a_prefix_and_a_value_invariant_keep_the_outcome_scenario_too() {
    for rule in [PREFIX, INVARIANT] {
        let suite = synthesized(rule, false);
        assert_eq!(
            suite.provenance.suite_version.to_string(),
            "ess-conformance/46",
            "{rule}"
        );
        assert!(suite
            .to_canonical_json()
            .unwrap()
            .contains("\"constraints\""));
    }
    // The type-coverage refusal for the invariant type is a separate question and stays.
    let result = synthesize(&ir(&model(INVARIANT, false)));
    assert!(
        refusals(&result)
            .iter()
            .any(|refusal| refusal.contains("ESS-SYNTH-013")
                && refusal.contains("catalog.items.Code")),
        "{:#?}",
        refusals(&result)
    );
}

#[test]
fn an_unconstrained_response_keeps_its_format_and_carries_no_constraints() {
    let result = synthesize(&ir(&model("", false)));
    assert!(result.refusals.is_empty(), "{:#?}", refusals(&result));
    assert_eq!(
        result.suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
    assert!(!result
        .suite
        .to_canonical_json()
        .unwrap()
        .contains("constraints"));
}

fn refusal_of(text: &str) -> Vec<String> {
    let result = synthesize(&ir(text));
    assert!(
        !result
            .suite
            .scenarios
            .keys()
            .any(|id| id.to_string() == OUTCOME),
        "the outcome must stay refused"
    );
    refusals(&result)
        .into_iter()
        .filter(|refusal| refusal.contains("ESS-SYNTH-001"))
        .collect()
}

#[test]
fn a_record_invariant_on_a_response_type_is_refused_by_that_name() {
    let text = model("", false)
        .replace(
            "  - name: catalog.items.Code\n    kind: newtype\n    of: String\n",
            "  - name: catalog.items.Code\n    kind: struct\n    fields:\n      - {name: low, type: Integer}\n      - {name: high, type: Integer}\n    invariants:\n      - 'low <= high'\n",
        );
    let refused = refusal_of(&text);
    assert!(
        refused
            .iter()
            .any(|refusal| refusal
                .contains("record invariant on a response type `catalog.items.Code`")),
        "{refused:#?}"
    );
}

#[test]
fn a_reading_attached_response_type_is_refused_by_that_name() {
    let reading = "    reading:\n      encoding: offset_date_time_text\n      origins: [{role: producer_process, offset: encoded_offset}]\n";
    let refused = refusal_of(&model(reading, false));
    assert!(
        refused
            .iter()
            .any(|refusal| refusal.contains("reading-attached response type `catalog.items.Code`")),
        "{refused:#?}"
    );
}

// ---- the native runner ----------------------------------------------------------------------

/// A catalog service returning `code` from every `Register`, and publishing it when `emits`.
struct Catalog {
    code: &'static str,
    emits: bool,
}

impl ConformanceTarget for Catalog {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("catalog-499", "1"))
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
            OutcomeName::new("registered").unwrap(),
        ));
        if self.emits {
            result = result.emitting(
                ObservedEvent::new("catalog.items.Registered".parse().unwrap())
                    .with("code", Node::from(self.code)),
            );
        }
        result.response = Some(BTreeMap::from([("code".into(), Node::from(self.code))]));
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

fn run(suite: &ConformanceSuite, target: &Catalog) -> BTreeMap<String, ScenarioResult> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(suite)
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect()
}

/// Each rule, one value that keeps it and one that breaks it.
const VECTORS: [(&str, &str, &str); 3] = [
    (ALPHABET, "AB-12", "ab-12"),
    (PREFIX, "IT-7", "XX-7"),
    (INVARIANT, "abcd", "abc"),
];

#[test]
fn the_native_runner_fails_a_returned_value_that_breaks_a_rule_and_passes_one_that_keeps_it() {
    for (rule, good, bad) in VECTORS {
        let suite = synthesized(rule, false);
        let passed = run(
            &suite,
            &Catalog {
                code: good,
                emits: false,
            },
        );
        assert_eq!(
            passed[OUTCOME].status,
            Status::Passed,
            "{good}: {:#?}",
            passed[OUTCOME]
        );
        let failed = run(
            &suite,
            &Catalog {
                code: bad,
                emits: false,
            },
        );
        assert_eq!(
            failed[OUTCOME].status,
            Status::Failed,
            "{bad}: {:#?}",
            failed[OUTCOME]
        );
        assert!(
            failed[OUTCOME]
                .checks
                .iter()
                .any(|check| check.status == Status::Failed && check.code == CheckCode::Payload),
            "{bad}: {:#?}",
            failed[OUTCOME]
        );
    }
}

#[test]
fn a_response_mapped_payload_of_a_constrained_type_is_admitted_and_checked() {
    for (rule, good, bad) in VECTORS {
        let suite = synthesized(rule, true);
        assert_eq!(
            suite.provenance.suite_version.to_string(),
            "ess-conformance/46"
        );
        let json: serde_json::Value =
            serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap();
        let payload = json["scenarios"][OUTCOME]["steps"]
            .as_array()
            .unwrap()
            .iter()
            .find(|step| step["step"] == "expect_response_payload")
            .expect("a response payload observation")
            .clone();
        assert!(
            payload["response"]["constraints"]
                .get("catalog.items.Code")
                .is_some(),
            "{payload:#}"
        );
        let passed = run(
            &suite,
            &Catalog {
                code: good,
                emits: true,
            },
        );
        assert_eq!(
            passed[OUTCOME].status,
            Status::Passed,
            "{good}: {:#?}",
            passed[OUTCOME]
        );
        let failed = run(
            &suite,
            &Catalog {
                code: bad,
                emits: true,
            },
        );
        assert_eq!(
            failed[OUTCOME].status,
            Status::Failed,
            "{bad}: {:#?}",
            failed[OUTCOME]
        );
    }
}

/// The payload observation alone fails a broken value: the direct observation is taken out.
#[test]
fn the_payload_observation_checks_the_rule_on_its_own() {
    let (rule, _, bad) = VECTORS[0];
    let mut suite = synthesized(rule, true);
    for scenario in suite.scenarios.values_mut() {
        scenario.steps.retain(|step| {
            !matches!(
                step,
                ess_conformance::ScenarioStep::ExpectDirectResponse { .. }
            )
        });
    }
    let failed = run(
        &suite,
        &Catalog {
            code: bad,
            emits: true,
        },
    );
    assert_eq!(
        failed[OUTCOME].status,
        Status::Failed,
        "{:#?}",
        failed[OUTCOME]
    );
}

#[test]
fn the_model_interpreter_passes_its_own_synthesized_suite() {
    for rule in [ALPHABET, PREFIX, INVARIANT] {
        for mapped in [false, true] {
            let text = model(rule, mapped);
            let suite = synthesized(rule, mapped);
            let admitted = AdmittedSuite::from_suite(&suite).unwrap();
            let run = Runner::for_suite(&suite).run_admitted(
                &admitted,
                &ess_conformance::interpret::Interpreted::for_model(ir(&text)),
            );
            let outcome = run
                .scenarios
                .iter()
                .find(|result| result.scenario.to_string() == OUTCOME)
                .expect("the outcome ran");
            assert_eq!(outcome.status, Status::Passed, "{rule}: {outcome:#?}");
        }
    }
}

// ---- formats and admission ------------------------------------------------------------------

/// The suite's canonical JSON with its version relabelled to `major`.
fn relabelled(suite: &ConformanceSuite, major: u32) -> String {
    let mut document: serde_json::Value =
        serde_json::from_str(&suite.to_canonical_json().unwrap()).unwrap();
    document["provenance"]["suite_version"] = format!("ess-conformance/{major}").into();
    serde_json::to_string(&document).unwrap()
}

#[test]
fn constraints_under_an_older_major_are_refused_by_name() {
    let suite = synthesized(ALPHABET, true);
    let Err(error) = AdmittedSuite::from_json(&relabelled(&suite, 44)) else {
        panic!("a /44 suite cannot carry response constraints");
    };
    let error = error.to_string();
    assert!(error.contains("suite/46 or /47"), "{error}");
}

#[test]
fn the_coverage_counterpart_is_47_and_the_browser_replay_refuses_it_by_version() {
    use ess_conformance::coverage::{Origins, Scope};
    let input = ess_conformance::coverage_build::build(
        &ir(&model(ALPHABET, false)),
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
        "ess-conformance/47"
    );
    let compared = support_go::compare_input(
        "coverage-47",
        &input,
        Catalog {
            code: "ab",
            emits: false,
        },
        &support_go::Options::default(),
    );
    let verdicts = support_go::assert_compared("coverage-47", compared);
    assert_eq!(verdicts[OUTCOME], "failed");

    let root = scratch("browser");
    std::fs::write(
        root.join("admission.js"),
        include_str!("../assets/coverage-admission.js"),
    )
    .unwrap();
    std::fs::write(root.join("suite.json"), input.selected().original_json()).unwrap();
    std::fs::write(
        root.join("harness.mjs"),
        "import {readFileSync} from 'node:fs'\nimport {admitSuite} from './admission.js'\ntry { await admitSuite(readFileSync('suite.json', 'utf8')); console.log('admitted') } catch (error) { console.log(String(error)) }\n",
    )
    .unwrap();
    let node = std::env::var_os("ESS_NODE").unwrap_or_else(|| "node".into());
    let output = Command::new(&node)
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("node runs");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        stdout.contains("replay requires suite/5, /9 or /35"),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn scratch(label: &str) -> PathBuf {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "direct-response-constraints-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    root
}

// ---- Go ------------------------------------------------------------------------------------

#[test]
fn the_go_runner_gives_the_native_verdicts_on_the_same_vectors() {
    for (rule, good, bad) in VECTORS {
        for mapped in [false, true] {
            let suite = synthesized(rule, mapped);
            for (code, verdict) in [(good, "passed"), (bad, "failed")] {
                let label = format!("go-{code}-{mapped}");
                let go = support_go::assert_parity(
                    &label,
                    &suite,
                    Catalog {
                        code,
                        emits: mapped,
                    },
                );
                assert_eq!(go[OUTCOME], verdict, "{label}");
            }
        }
    }
}

#[test]
fn an_older_go_reader_refuses_a_suite_46_by_version() {
    let suite = synthesized(ALPHABET, false);
    let directory =
        support_go::package("older-reader-46", &suite, &[support_go::TRANSCRIPT_TARGET]);
    let runtime = directory.join("essconform/runtime.go");
    let text = std::fs::read_to_string(&runtime).unwrap();
    assert!(
        text.contains("const newestSuiteMajor = 47\n"),
        "the runtime reads /47"
    );
    std::fs::write(
        &runtime,
        text.replace(
            "const newestSuiteMajor = 47\n",
            "const newestSuiteMajor = 45\n",
        ),
    )
    .unwrap();
    let recorder = support_go::Recorder::new(Catalog {
        code: "AB",
        emits: false,
    });
    let replayed = support_go::replay(&directory, &recorder, &[]);
    std::fs::remove_dir_all(&directory).unwrap();
    assert!(!replayed.go.success, "{}", replayed.go.log);
    assert_eq!(replayed.go.outcomes.len(), 0, "{}", replayed.go.log);
    assert!(
        replayed.go.log.contains("ess-conformance/46"),
        "{}",
        replayed.go.log
    );
}

// ---- TypeScript ----------------------------------------------------------------------------

#[test]
fn the_typescript_runner_gives_the_native_verdicts_on_the_same_vectors() {
    for (rule, good, bad) in VECTORS {
        for mapped in [false, true] {
            let suite = synthesized(rule, mapped);
            for (code, verdict) in [(good, "passed"), (bad, "failed")] {
                let label = format!("ts-{code}-{mapped}");
                let typescript = typescript_parity(
                    &label,
                    &suite,
                    Catalog {
                        code,
                        emits: mapped,
                    },
                );
                assert_eq!(typescript[OUTCOME], verdict, "{label}");
            }
        }
    }
}

#[test]
fn an_older_typescript_reader_refuses_a_suite_46_by_version() {
    let suite = synthesized(ALPHABET, false);
    let package = typescript_package("older-reader-46", &suite);
    let runtime = package.join("dist/runtime.js");
    let text = std::fs::read_to_string(&runtime).unwrap();
    let entry = "'ess-conformance/46': 46,";
    assert!(text.contains(entry), "the TypeScript runtime reads /46");
    std::fs::write(&runtime, text.replace(entry, "")).unwrap();
    let (log, report) = typescript_run(
        &package,
        &suite,
        Catalog {
            code: "AB",
            emits: false,
        },
    );
    let _ = std::fs::remove_dir_all(package.parent().unwrap());
    assert!(report.is_none(), "{log}");
    assert!(log.contains("ess-conformance/46"), "{log}");
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
    target: Catalog,
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
    target: Catalog,
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
