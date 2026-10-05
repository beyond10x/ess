//! Suite `/40` end to end (`docs/design/expression-family-source22.md`, "Suite formats and old
//! readers"; final review decision 10): an ess/22 invariant ordering two Integer siblings is carried
//! as `{fact: …}`, selects `/40`, is admitted and executed by the Rust runner, and the same suite
//! relabelled to an older major is refused before any step runs.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = r"format: ess/22
system: pool
version: v1
domain: pool.lease
entities:
  - name: pool.lease.Pool
    identity: {name: pool_id, type: Uuid}
    fields:
      - {name: leased, type: Integer}
      - {name: capacity, type: Integer}
    invariants:
      - leased <= capacity
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: pool.lease.Opened
    fields:
      - {name: pool_id, type: Uuid}
commands:
  - name: pool.lease.Open
    input:
      - {name: leased, type: Integer}
      - {name: capacity, type: Integer}
    outcomes:
      - name: opened
        creates: pool.lease.Pool
        instance: pool_id
        emits: [pool.lease.Opened]
        payload:
          pool.lease.Opened: {pool_id: {generated: true}}
        sets: {leased: input.leased, capacity: input.capacity}
views:
  - name: pool.lease.Pools
    source: pool.lease.Pool
    consistency: read_your_writes
    fields:
      - {name: pool_id, type: Uuid}
      - {name: leased, type: Integer}
      - {name: capacity, type: Integer}
";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("pool.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

#[test]
fn a_sibling_invariant_selects_suite40_and_executes() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    let suite = synthesis.suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/40"
    );
    let json = suite.to_canonical_json().expect("admitted");
    assert!(json.contains(r#""fact": "capacity""#), "{json}");
    let admitted = AdmittedSuite::from_json(&json).expect("a /40 reader admits it");
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir()))
        .into_report();
    let failing: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{}: {:?}", scenario.scenario, scenario.status))
        .collect();
    assert_eq!(failing, Vec::<String>::new());
    assert!(
        report
            .scenarios
            .iter()
            .any(|scenario| scenario.scenario.to_string().contains("/invariant/")),
        "the invariant is asserted, not skipped"
    );
}

#[test]
fn suite39_relabel_refuses_fact_before_execution() {
    let json = ess_conformance::synthesize::synthesize(&ir())
        .suite
        .to_canonical_json()
        .expect("admitted");
    // `/34` and `/38` are older ordinary majors whose envelope a `/40` suite otherwise satisfies,
    // so the vocabulary is what refuses them: `/38` carries conditional aggregate measures and
    // still predates the expression vocabulary.
    for older in ["ess-conformance/34", "ess-conformance/38"] {
        let relabelled = json.replace("ess-conformance/40", older);
        let refused = AdmittedSuite::from_json(&relabelled).expect_err(older);
        let text = refused.to_string();
        assert!(text.contains("suite/40 or /41"), "{older}: {text}");
    }
}

// ---- the generated runners -------------------------------------------------------------------

mod support_go;

use ess_conformance::target::{
    ConformanceTarget, EventObservationRequest, ExternalOutcomeControl, ImplementationIdentity,
    ObservedEvent, RedeliveryRequest, ScenarioContext, SemanticCommandRequest,
    SemanticCommandResult, SemanticViewRequest, SemanticViewResult, TargetError,
};
use ess_primitives::node::Node;

/// The interpreter, with every view row's `capacity` published one below its `leased` when
/// `broken`: a target that breaks the invariant only where the suite reads it.
struct Pools {
    inner: Interpreted,
    broken: bool,
}

impl ConformanceTarget for Pools {
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
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        if self.broken {
            for row in &mut result.rows {
                if let Some(Node::Number(leased)) = row.get("leased").cloned() {
                    let below = ess_primitives::facts::Number::new(leased.get() - 1.0).unwrap();
                    row.insert("capacity".to_owned(), Node::Number(below));
                }
            }
        }
        Ok(result)
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

const INVARIANT: &str = "pool.lease.Pool/invariant/after/pool.lease.Open/opened";

#[test]
fn go_reads_suite40_at_parity_with_the_rust_runner() {
    let suite = ess_conformance::synthesize::synthesize(&ir()).suite;
    let healthy = support_go::assert_parity(
        "suite40-healthy",
        &suite,
        Pools {
            inner: Interpreted::for_model(ir()),
            broken: false,
        },
    );
    assert_eq!(support_go::not_passed(&healthy), Vec::<&str>::new());
    let broken = support_go::assert_parity(
        "suite40-broken",
        &suite,
        Pools {
            inner: Interpreted::for_model(ir()),
            broken: true,
        },
    );
    assert!(
        support_go::not_passed(&broken).contains(&INVARIANT),
        "{broken:#?}"
    );
    assert_eq!(broken[INVARIANT], "failed");
}

/// A JavaScript implementation of the pool, the same in both modes but for `capacity` on a row.
const JS_TARGET: &str = r"const mode = process.env.ESS_TARGET_MODE ?? 'healthy';
class Pools {
  rows = [];
  seq = 0;
  identity() { return { name: 'pools', version: '1' }; }
  beginScenario() { this.rows = []; }
  endScenario() {}
  executeCommand({ command, input }) {
    if (command !== 'pool.lease.Open') throw new Error(`unexpected ${command}`);
    this.seq += 1;
    const id = `00000000-0000-4000-8000-${String(this.seq).padStart(12, '0')}`;
    this.rows.push({ pool_id: id, leased: input.leased, capacity: input.capacity });
    return {
      outcome: 'opened',
      consistency: `seq:${this.seq}`,
      directEvents: [{ event: 'pool.lease.Opened', payload: { pool_id: id } }],
    };
  }
  queryView() {
    return {
      rows: this.rows.map((row) =>
        mode === 'broken' ? { ...row, capacity: Number(row.leased) - 1 } : { ...row },
      ),
    };
  }
  observeEvents() { throw new Error('unused'); }
  configureExternalOutcome() { throw new Error('nothing is external'); }
  redeliverEvent() { throw new Error('no bindings'); }
}
export function makeTarget() { return new Pools(); }
";

/// The TypeScript runner's verdicts for the `/40` suite against [`JS_TARGET`] in `mode`; `None`
/// where `tsc` or `node` is missing.
fn typescript_verdicts(mode: &str) -> Option<std::collections::BTreeMap<String, String>> {
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
    let suite = ess_conformance::synthesize::synthesize(&ir()).suite;
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("expression-suite40")
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
    let mut verdicts = std::collections::BTreeMap::new();
    for (status, ids) in document["outcomes"].as_object().expect("outcomes") {
        for id in ids.as_array().expect("a list") {
            verdicts.insert(id.as_str().expect("an id").to_owned(), status.clone());
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    Some(verdicts)
}

#[test]
fn typescript_reads_suite40_and_fails_the_broken_invariant() {
    let Some(healthy) = typescript_verdicts("healthy") else {
        return;
    };
    assert!(
        !healthy.is_empty(),
        "the TypeScript run published no verdicts"
    );
    let failing: Vec<&String> = healthy
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id)
        .collect();
    assert_eq!(failing, Vec::<&String>::new(), "{healthy:#?}");
    let broken = typescript_verdicts("broken").expect("tools were found once");
    assert_eq!(
        broken.get(INVARIANT).map(String::as_str),
        Some("failed"),
        "{broken:#?}"
    );
}

#[test]
fn a_coverage_input_carrying_the_vocabulary_selects_suite41() {
    let input = ess_conformance::coverage_build::build(
        &ir(),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("{error}"));
    let selected = input.selected();
    assert_eq!(
        selected.suite().provenance.suite_version.to_string(),
        "ess-conformance/41"
    );
    let original = selected.original_json();
    assert!(original.contains(r#""fact""#), "{original}");
    ess_conformance::AdmittedSuite::from_json(original).expect("a /41 reader admits it");
}
