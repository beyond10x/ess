//! Adversary pass 1 on beyond10x/ess#463: a row-set selector over the members of a struct identity
//! (ess/23). Each case drives synthesis and the interpreter, the honest target, against the shelf
//! fixture or a variant of it, and a faulty target is a mutant of the model the interpreter runs.

mod support_go;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, AdvancingClock, ConformanceSuite, Ids, Runner, RunnerConfig};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const SHELVES: &str = include_str!("fixtures/row-set-struct-identity.yaml");

const SEALED: &str = "demo.vault.Store/outcome/sealed";
const STORED: &str = "demo.vault.Store/outcome/stored";

const SELECTOR: &str =
    "{all: [at.region == input.place.region, at.shelf == input.place.shelf, mode == sealed]}";
const PLACE: &str = "  - name: demo.vault.Place\n    kind: struct\n    fields:\n      - {name: region, type: String}\n      - {name: shelf, type: String}\n";

fn replaced(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "`{from}` is in the model");
    text.replace(from, to)
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("shelves.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("admitted: {errors}\n{text}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn synthesized(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(text))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// The suite of `text`, which synthesizes with no refusal and which the interpreter passes whole.
fn passing_suite(text: &str) -> ConformanceSuite {
    let synthesis = synthesized(text);
    assert_eq!(refusals(&synthesis), Vec::<String>::new(), "no refusal");
    let verdicts =
        support_go::rust_outcomes(&synthesis.suite, &Interpreted::for_model(ir_of(text)));
    assert_eq!(
        support_go::not_passed(&verdicts),
        Vec::<&str>::new(),
        "{verdicts:#?}"
    );
    synthesis.suite
}

fn failed_by(suite: &ConformanceSuite, mutant: &str) -> Vec<String> {
    let verdicts = support_go::rust_outcomes(suite, &Interpreted::for_model(ir_of(mutant)));
    support_go::not_passed(&verdicts)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn with_selector(text: &str, selector: &str) -> String {
    replaced(text, SELECTOR, selector)
}

// ---- a partial selector against a target that compares the whole struct ------------------------

/// The selector pins `at.region` only. A target that selects by the whole struct — every member of
/// the identity equal to the place sent — selects fewer rows than the specification says, and a
/// decisive suite catches it: some scenario must select a row whose `shelf` differs from the one
/// sent.
#[test]
fn adv_partial_member_selector_whole_struct_target_fails() {
    let partial = with_selector(
        SHELVES,
        "{all: [at.region == input.place.region, mode == sealed]}",
    );
    let suite = passing_suite(&partial);
    let whole = with_selector(SHELVES, SELECTOR);
    let failed = failed_by(&suite, &whole);
    assert!(
        !failed.is_empty(),
        "a target selecting by the whole struct identity passes every scenario of the partial \
         selector's suite: {:#?}",
        suite.scenarios.keys().collect::<Vec<_>>()
    );
}

// ---- nested, Optional and non-String members --------------------------------------------------

#[test]
fn adv_nested_struct_identity_members_witness_and_kill_each_dropped_member() {
    let nested = replaced(
        SHELVES,
        PLACE,
        "  - name: demo.vault.Slot\n    kind: struct\n    fields:\n      - {name: row, type: String}\n      - {name: col, type: String}\n  - name: demo.vault.Place\n    kind: struct\n    fields:\n      - {name: region, type: String}\n      - {name: slot, type: demo.vault.Slot}\n",
    );
    let members = [
        "at.region == input.place.region",
        "at.slot.row == input.place.slot.row",
        "at.slot.col == input.place.slot.col",
    ];
    let model = with_selector(
        &nested,
        &format!("{{all: [{}, mode == sealed]}}", members.join(", ")),
    );
    let suite = passing_suite(&model);
    for dropped in members {
        let kept: Vec<&str> = members.iter().copied().filter(|m| *m != dropped).collect();
        let mutant = with_selector(
            &nested,
            &format!("{{all: [{}, mode == sealed]}}", kept.join(", ")),
        );
        assert!(
            !failed_by(&suite, &mutant).is_empty(),
            "a target dropping `{dropped}` passes every scenario"
        );
    }
}

#[test]
fn adv_optional_member_identity_witnesses_and_kills_dropped_member() {
    let model = replaced(
        SHELVES,
        "      - {name: shelf, type: String}\n",
        "      - {name: shelf, type: 'Optional<String>'}\n",
    );
    let suite = passing_suite(&model);
    let mutant = with_selector(
        &model,
        "{all: [at.region == input.place.region, mode == sealed]}",
    );
    assert!(
        !failed_by(&suite, &mutant).is_empty(),
        "a target dropping the Optional member passes every scenario"
    );
}

#[test]
fn adv_integer_member_beside_string_member_kills_dropped_integer() {
    let model = replaced(
        SHELVES,
        "      - {name: shelf, type: String}\n",
        "      - {name: shelf, type: Integer}\n",
    );
    let suite = passing_suite(&model);
    let mutant = with_selector(
        &model,
        "{all: [at.region == input.place.region, mode == sealed]}",
    );
    assert!(
        !failed_by(&suite, &mutant).is_empty(),
        "a target dropping the Integer member passes every scenario"
    );
}

/// An Integer member alone does not scope the selector: refused as unscoped, as a plain Integer
/// field is.
#[test]
fn adv_integer_member_alone_is_refused_as_unscoped() {
    let model = with_selector(
        &replaced(
            SHELVES,
            "      - {name: shelf, type: String}\n",
            "      - {name: shelf, type: Integer}\n",
        ),
        "{all: [at.shelf == input.place.shelf, mode == sealed]}",
    );
    let synthesis = synthesized(&model);
    let refused = refusals(&synthesis);
    for branch in ["demo.vault.Store/sealed", "demo.vault.Store/stored"] {
        assert!(
            refused
                .iter()
                .any(|r| r.contains(branch) && r.contains("selects rows by no equality")),
            "{branch}: {refused:#?}"
        );
    }
}

// ---- the `instances: {where:}` sibling ----------------------------------------------------------

/// `SealRegion {region}` seals every shelf of a region: a bulk update whose `where:` reads a member
/// of the struct identity (the sibling the story names). Validate refuses it (`unobservable_fact`:
/// the identity is no observable root of a bulk `where:`), so no synthesis gate is reached.
#[test]
fn adv_bulk_update_where_on_identity_member() {
    let bulk = |filter: &str| {
        let mut model = replaced(
            SHELVES,
            "  - name: demo.vault.Store\n",
            &format!(
                "  - name: demo.vault.SealRegion\n    input:\n      - {{name: region, type: String}}\n    outcomes:\n      - name: sealed-all\n        updates: demo.vault.Shelf\n        instances: {{where: {filter}}}\n        sets: {{mode: sealed}}\n        emits: [demo.vault.RegionSealed]\n        payload:\n          demo.vault.RegionSealed: {{region: input.region}}\n  - name: demo.vault.Store\n"
            ),
        );
        model = replaced(
            &model,
            "  - name: demo.vault.Stored\n",
            "  - name: demo.vault.RegionSealed\n    fields: [{name: region, type: String}]\n  - name: demo.vault.Stored\n",
        );
        replaced(
            &model,
            "may: [demo.vault.OpenShelf, demo.vault.Store]",
            "may: [demo.vault.OpenShelf, demo.vault.SealRegion, demo.vault.Store]",
        )
    };
    let raw = RawSpecFile::parse(&bulk("at.region == input.region")).expect("parses");
    let errors = Specification::assemble([(Source::new("shelves.yaml"), raw)])
        .expect_err("a bulk `where:` reading a member of the identity is refused at validate");
    assert!(errors.to_string().contains("unobservable_fact"), "{errors}");
}

// ---- the Go and TypeScript runtimes on the new fixture ----------------------------------------

#[test]
fn adv_go_gives_the_reference_verdicts_on_the_struct_identity_fixture() {
    let suite = passing_suite(SHELVES);
    let healthy = support_go::assert_parity(
        "adv463-healthy",
        &suite,
        Interpreted::for_model(ir_of(SHELVES)),
    );
    assert_eq!(support_go::not_passed(&healthy), Vec::<&str>::new());
    let dropped = with_selector(
        SHELVES,
        "{all: [at.shelf == input.place.shelf, mode == sealed]}",
    );
    let faulty = support_go::assert_parity(
        "adv463-drop",
        &suite,
        Interpreted::for_model(ir_of(&dropped)),
    );
    assert!(!support_go::not_passed(&faulty).is_empty(), "{faulty:#?}");
}

const DRIVER: &str = r"
import {readFileSync, writeFileSync} from 'node:fs';
import {runWith, unsupported, setWallNow} from './dist/runtime.js';
const [suiteFile, transcriptFile, divergenceFile, wall] = process.argv.slice(2);
setWallNow(() => Number(wall));
const transcript = JSON.parse(readFileSync(transcriptFile, 'utf8'));
const divergences = [];
const target = () => {
  let scenario = '';
  let used = new Map();
  const next = (method, key, request) => {
    const matching = (transcript[scenario] ?? []).filter(e => e.method === method && e.key === key);
    const n = used.get(method + '\0' + key) ?? 0;
    used.set(method + '\0' + key, n + 1);
    let entry = matching[n];
    if (entry === undefined && matching.length > 0 && ['query_view', 'observe_events'].includes(method)) {
      entry = matching[matching.length - 1];
    }
    if (entry === undefined) {
      divergences.push(`${scenario}: ${method} \`${key}\` call ${n + 1} was never made by the reference runner`);
      throw new Error('transcript divergence');
    }
    if (method === 'execute_command') {
      const sent = JSON.stringify(request.input ?? {}, Object.keys(request.input ?? {}).sort());
      const want = JSON.stringify(entry.request?.input ?? {}, Object.keys(entry.request?.input ?? {}).sort());
      if (sent !== want) {
        divergences.push(`${scenario}: ${method} \`${key}\` sent ${sent}, the reference runner sent ${want}`);
        throw new Error('transcript divergence');
      }
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
      const r = next('execute_command', request.command, request);
      return {
        outcome: r.outcome ?? undefined, error: r.error ?? undefined,
        consistency: r.consistency ?? undefined, directEvents: r.direct_events ?? [],
        response: r.response ?? undefined,
      };
    },
    queryView: async request => {
      const r = next('query_view', request.view, request);
      return {rows: r.rows ?? [], total: r.total ?? undefined};
    },
    observeEvents: async request => next('observe_events', request.event, request)
      .map(event => ({event: event.event, payload: event.payload})),
    establishEntity: async request => next('establish_entity', request.entity, request),
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

fn typescript_package(label: &str, suite: &ConformanceSuite) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adv463-ts-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
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

const WALL_MS: u64 = 1_791_115_199_900;

fn typescript_parity<T: ess_conformance::target::ConformanceTarget>(
    label: &str,
    suite: &ConformanceSuite,
    target: T,
) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let recorder = support_go::Recorder::new(target);
    let report = Runner::new(
        RunnerConfig::default(),
        ess_conformance::now_offset::WithWall::new(AdvancingClock::default(), || {
            ess_primitives::time::Timestamp::from_epoch_millis(WALL_MS)
        }),
        Ids::for_suite(suite),
    )
    .run_admitted(&admitted, &recorder)
    .into_report();
    let rust: BTreeMap<String, String> = report
        .scenarios
        .into_iter()
        .map(|result| {
            let status = match result.status {
                ess_conformance::report::Status::Passed => "passed",
                ess_conformance::report::Status::Failed => "failed",
                ess_conformance::report::Status::Error => "error",
                ess_conformance::report::Status::Unsupported => "unsupported",
            };
            (result.scenario.to_string(), status.to_owned())
        })
        .collect();
    let package = typescript_package(label, suite);
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
        .arg(WALL_MS.to_string())
        .env("ESS_REPORT_FORMAT", "2")
        .env("ESS_REPORT_OUT", &report)
        .output()
        .expect("node runs");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_ne!(output.status.code(), Some(2), "{label}: {log}");
    let divergences = std::fs::read_to_string(&divergence).unwrap_or_default();
    assert_eq!(divergences, "", "{label}: {log}");
    let report: serde_json::Value = serde_json::from_slice(
        &std::fs::read(&report).unwrap_or_else(|error| panic!("{label}: {error}: {log}")),
    )
    .unwrap();
    let mut typescript = BTreeMap::new();
    for (status, ids) in report["outcomes"].as_object().expect("report/2 outcomes") {
        for id in ids.as_array().expect("ids") {
            typescript.insert(id.as_str().unwrap().to_owned(), status.clone());
        }
    }
    let _ = std::fs::remove_dir_all(package.parent().unwrap());
    assert_eq!(
        typescript, rust,
        "{label}: per-scenario verdicts, TypeScript (left) and Rust (right)\n{log}"
    );
    rust
}

#[test]
fn adv_typescript_gives_the_reference_verdicts_on_the_struct_identity_fixture() {
    let suite = passing_suite(SHELVES);
    let healthy = typescript_parity("healthy", &suite, Interpreted::for_model(ir_of(SHELVES)));
    assert_eq!(support_go::not_passed(&healthy), Vec::<&str>::new());
    assert!(
        healthy.contains_key(SEALED) && healthy.contains_key(STORED),
        "{healthy:#?}"
    );
    let dropped = with_selector(
        SHELVES,
        "{all: [at.region == input.place.region, mode == sealed]}",
    );
    let faulty = typescript_parity("drop", &suite, Interpreted::for_model(ir_of(&dropped)));
    assert!(!support_go::not_passed(&faulty).is_empty(), "{faulty:#?}");
}

// ---- a partial selector against a target that keys rows by the selected member -------------------

/// The decision a target makes that keys shelves by `at.region` alone, the latest open of a region
/// replacing the earlier one (a map keyed by the partial selector's member): `sealed` where the
/// newest shelf opened in the place's region is sealed, `stored` otherwise. Simulated on the steps.
fn latest_wins_disagrees(steps: &[ess_conformance::scenario::ScenarioStep]) -> bool {
    use ess_conformance::scenario::{ScenarioStep, ScenarioValue};
    use ess_primitives::node::Node;
    let member = |value: &Node, name: &str| match value {
        Node::Map(members) => members.get(name).cloned(),
        _ => None,
    };
    let mut latest: BTreeMap<Node, Node> = BTreeMap::new();
    for (at, step) in steps.iter().enumerate() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        let literal = |field: &str| {
            input
                .get(field)
                .and_then(ScenarioValue::as_literal)
                .cloned()
        };
        match command.to_string().as_str() {
            "demo.vault.OpenShelf" => {
                let (place, mode) = (literal("at").unwrap(), literal("mode").unwrap());
                latest.insert(member(&place, "region").unwrap(), mode);
            }
            "demo.vault.Store" => {
                let place = literal("place").unwrap();
                let sealed = latest.get(&member(&place, "region").unwrap())
                    == Some(&Node::Text("sealed".to_owned()));
                let expected = steps[at + 1..].iter().find_map(|step| match step {
                    ScenarioStep::ExpectOutcome { outcome } => Some(outcome.outcome.to_string()),
                    _ => None,
                });
                let answered = if sealed { "sealed" } else { "stored" };
                if expected.as_deref() != Some(answered) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// `at.region == input.place.region` selects many shelves of one region. A target that treats the
/// partial member as the record's key, the newest open of a region replacing the older, answers
/// from one row and not from the set; a decisive suite has a scenario where that row decides
/// otherwise than the set (a sealed shelf of the region opened before an unsealed one).
#[test]
fn adv_partial_member_selector_latest_wins_unique_target_fails() {
    let partial = with_selector(
        SHELVES,
        "{all: [at.region == input.place.region, mode == sealed]}",
    );
    let suite = passing_suite(&partial);
    let killed: Vec<String> = suite
        .scenarios
        .iter()
        .filter(|(_, scenario)| latest_wins_disagrees(&scenario.steps))
        .map(|(id, _)| id.to_string())
        .collect();
    let steps: Vec<_> = [SEALED, STORED]
        .iter()
        .map(|id| {
            suite
                .scenarios
                .iter()
                .find(|(scenario, _)| scenario.to_string() == *id)
                .map(|(_, scenario)| scenario.steps.clone())
        })
        .collect();
    assert!(
        !killed.is_empty(),
        "a target keying shelves by `at.region` alone, newest wins, passes every scenario:\n{steps:#?}"
    );
}

/// The same shape over a plain `String` field at `ess/22`, through no gate this unit added: the
/// order of decoys before selected rows is the arrangement's own, older than the unit.
#[test]
fn adv_plain_field_selector_latest_wins_unique_target_fails_at_ess22() {
    let plain = replaced(
        &replaced(
            &replaced(
                &with_selector(
                    SHELVES,
                    "{all: [region == input.place.region, mode == sealed]}",
                ),
                "format: ess/23",
                "format: ess/22",
            ),
            "    identity: {name: at, type: demo.vault.Place}\n    fields:\n      - {name: mode, type: String}\n",
            "    identity: {name: at, type: demo.vault.Place}\n    fields:\n      - {name: region, type: String}\n      - {name: mode, type: String}\n",
        ),
        "        sets: {mode: input.mode}\n",
        "        sets: {mode: input.mode, region: input.at.region}\n",
    );
    let suite = passing_suite(&plain);
    let killed = suite
        .scenarios
        .values()
        .filter(|scenario| latest_wins_disagrees(&scenario.steps))
        .count();
    assert!(
        killed > 0,
        "a region-keyed newest-wins target passes every scenario"
    );
}
