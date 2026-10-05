//! Family F part A2 executed (beyond10x/ess#233, #244): one constant offset on a right-hand fact.
//!
//! A command refuses a lease whose range is too wide (`upper > lower + 5`), inverted
//! (`upper < lower - 3`), ends too late (`expires_at >= issued_at + 1h`) or too soon
//! (`expires_at < issued_at + 5m`), or exceeds an optional cap (`upper > cap + 10`); the lease it
//! creates holds `upper <= lower + 100` and `expires_at <= issued_at + 24h`. The suite is
//! synthesized from the model and run against the native interpreter, which must pass it, and
//! against targets wrong in one way each — the sign, the unit, the base — which must each fail a
//! scenario that decides the guard. The invariants persist as `{offset: …}` in `satisfies`, select
//! suite `/40`, and are executed by the Rust, Go and TypeScript runners against a healthy and a
//! broken row. `docs/design/expression-family-source22.md`, A2, is the design.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::facts::Number;
use ess_primitives::node::Node;
use ess_primitives::time::Rfc3339Instant;

mod support_go;

const MODEL: &str = r"format: ess/22
system: pool
version: v1
domain: pool.lease
entities:
  - name: pool.lease.Lease
    identity: {name: lease_id, type: Uuid}
    fields:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: issued_at, type: Timestamp}
      - {name: expires_at, type: Timestamp}
    invariants:
      - upper <= lower + 100
      - expires_at <= issued_at + 24h
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: pool.lease.TooWide, summary: The range is too wide.}
  - {name: pool.lease.Inverted, summary: The range is inverted.}
  - {name: pool.lease.TooLate, summary: The lease ends too late.}
  - {name: pool.lease.TooSoon, summary: The lease ends too soon.}
  - {name: pool.lease.OverCap, summary: The range is over its cap.}
events:
  - name: pool.lease.Opened
    fields:
      - {name: lease_id, type: Uuid}
commands:
  - name: pool.lease.Open
    input:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: cap, type: Optional<Integer>}
      - {name: issued_at, type: Timestamp}
      - {name: expires_at, type: Timestamp}
    outcomes:
      - name: too-wide
        when: upper > lower + 5
        error: pool.lease.TooWide
      - name: inverted
        when: upper < lower - 3
        error: pool.lease.Inverted
      - name: too-late
        when: expires_at >= issued_at + 1h
        error: pool.lease.TooLate
      - name: too-soon
        when: expires_at < issued_at + 5m
        error: pool.lease.TooSoon
      - name: over-cap
        when: upper > cap + 10
        error: pool.lease.OverCap
      - name: opened
        creates: pool.lease.Lease
        instance: lease_id
        emits: [pool.lease.Opened]
        payload:
          pool.lease.Opened: {lease_id: {generated: true}}
        sets: {lower: input.lower, upper: input.upper, issued_at: input.issued_at, expires_at: input.expires_at}
views:
  - name: pool.lease.Leases
    source: pool.lease.Lease
    consistency: read_your_writes
    fields:
      - {name: lease_id, type: Uuid}
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: issued_at, type: Timestamp}
      - {name: expires_at, type: Timestamp}
";

const OUTCOMES: [&str; 6] = [
    "too-wide", "inverted", "too-late", "too-soon", "over-cap", "opened",
];

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ir() -> EssIr {
    ir_of(MODEL)
}

fn suite() -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn scenario(outcome: &str) -> String {
    format!("pool.lease.Open/outcome/{outcome}")
}

// ---- the reference decision and the faulty ones --------------------------------------------------

/// How a target decides the offset guards.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Correct,
    /// `upper < lower + 3`: the subtraction read as an addition.
    IgnoresSign,
    /// `expires_at >= issued_at + 1s`: the hour read as a second.
    IgnoresUnit,
    /// `upper > 5`: the magnitude compared without its base.
    IgnoresBase,
    /// Breaks the Integer invariant in every view row: `upper` published as `lower + 101`.
    BrokenRows,
    /// Breaks the Timestamp invariant in every view row: `expires_at` published 25 hours on.
    BrokenInstants,
}

fn integer(input: &BTreeMap<String, Node>, name: &str) -> Option<i128> {
    match input.get(name)? {
        Node::Number(number) => number.as_i64().map(i128::from),
        _ => None,
    }
}

fn instant(input: &BTreeMap<String, Node>, name: &str) -> Option<Rfc3339Instant> {
    match input.get(name)? {
        Node::Text(text) => Rfc3339Instant::parse_rfc3339(text),
        _ => None,
    }
}

/// The branch `mode` takes for `input`, deciding each guard exactly as the model does but for the
/// one `mode` gets wrong; `None` where a guard reads an absent value.
fn decide(mode: Mode, input: &BTreeMap<String, Node>) -> Option<&'static str> {
    let (lower, upper) = (integer(input, "lower")?, integer(input, "upper")?);
    let (issued, expires) = (instant(input, "issued_at")?, instant(input, "expires_at")?);
    let wide = match mode {
        Mode::IgnoresBase => upper > 5,
        _ => upper > lower + 5,
    };
    let inverted = match mode {
        Mode::IgnoresSign => upper < lower + 3,
        _ => upper < lower - 3,
    };
    let late = match mode {
        Mode::IgnoresUnit => expires >= issued.plus_elapsed(1)?,
        _ => expires >= issued.plus_elapsed(3_600)?,
    };
    let soon = expires < issued.plus_elapsed(300)?;
    if wide {
        return Some("too-wide");
    }
    if inverted {
        return Some("inverted");
    }
    if late {
        return Some("too-late");
    }
    if soon {
        return Some("too-soon");
    }
    if upper > integer(input, "cap")? + 10 {
        return Some("over-cap");
    }
    Some("opened")
}

fn number(value: i128) -> Node {
    Node::Number(Number::from(i64::try_from(value).expect("an i64")))
}

/// `input` moved so that the model takes `outcome` for it: the interpreter answers for the branch
/// a faulty target decided.
fn toward(mut input: BTreeMap<String, Node>, outcome: &str) -> BTreeMap<String, Node> {
    let lower = integer(&input, "lower").unwrap_or(0);
    let issued = instant(&input, "issued_at").expect("an instant");
    let at = |seconds: i64| Node::Text(issued.plus_elapsed(seconds).expect("moves").to_rfc3339());
    input.insert("upper".to_owned(), number(lower));
    input.insert("cap".to_owned(), number(lower));
    input.insert("expires_at".to_owned(), at(600));
    match outcome {
        "too-wide" => drop(input.insert("upper".to_owned(), number(lower + 6))),
        "inverted" => drop(input.insert("upper".to_owned(), number(lower - 4))),
        "too-late" => drop(input.insert("expires_at".to_owned(), at(7_200))),
        "too-soon" => drop(input.insert("expires_at".to_owned(), at(0))),
        "over-cap" => drop(input.insert("cap".to_owned(), number(lower - 11))),
        _ => {}
    }
    input
}

/// The interpreter, with the offset guards decided by `mode`, or its rows broken by it.
struct Leases {
    inner: Interpreted,
    mode: Mode,
}

impl Leases {
    fn new(mode: Mode) -> Self {
        Self {
            inner: Interpreted::for_model(ir()),
            mode,
        }
    }
}

impl ConformanceTarget for Leases {
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
        if let (Some(correct), Some(faulty)) = (
            decide(Mode::Correct, &request.input),
            decide(self.mode, &request.input),
        ) {
            if correct != faulty {
                request.input = toward(request.input, faulty);
            }
        }
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        for row in &mut result.rows {
            match self.mode {
                Mode::BrokenRows => {
                    if let Some(lower) = integer(row, "lower") {
                        row.insert("upper".to_owned(), number(lower + 101));
                    }
                }
                Mode::BrokenInstants => {
                    if let Some(issued) = instant(row, "issued_at") {
                        let late = issued.plus_elapsed(25 * 3_600).expect("moves");
                        row.insert("expires_at".to_owned(), Node::Text(late.to_rfc3339()));
                    }
                }
                _ => {}
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

/// The last input each scenario deciding a branch sends.
fn sent(suite: &ConformanceSuite, id: &str) -> BTreeMap<String, Node> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario);
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(
                input
                    .iter()
                    .filter_map(|(name, value)| match value {
                        ess_conformance::ScenarioValue::Literal { value } => {
                            Some((name.clone(), value.clone()))
                        }
                        _ => None,
                    })
                    .collect(),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{id} sends the command"))
}

// ---- synthesis and the interpreter ---------------------------------------------------------------

#[test]
fn a2_every_branch_is_witnessed_at_its_boundary_and_the_interpreter_passes() {
    let suite = suite();
    for outcome in OUTCOMES {
        let input = sent(&suite, &scenario(outcome));
        assert_eq!(
            decide(Mode::Correct, &input),
            Some(outcome),
            "{outcome} is witnessed by an input that takes it: {input:#?}"
        );
    }
    let statuses = run(&suite, &Interpreted::for_model(ir()));
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
    for outcome in OUTCOMES {
        assert_eq!(
            statuses.get(&scenario(outcome)),
            Some(&Status::Passed),
            "{statuses:#?}"
        );
    }
    assert!(
        statuses.keys().any(|id| id.contains("/invariant/")),
        "the invariants are asserted, not skipped: {statuses:#?}"
    );
}

fn caught(mode: Mode, deciding: &[&str]) {
    let suite = suite();
    let healthy = run(&suite, &Leases::new(Mode::Correct));
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    let failing = not_passed(&run(&suite, &Leases::new(mode)));
    assert!(
        failing.iter().any(|line| deciding
            .iter()
            .any(|outcome| line.starts_with(&scenario(outcome)))),
        "{mode:?} fails a scenario that decides its guard: {failing:#?}"
    );
    assert!(
        failing.iter().all(|line| line.contains("Failed")),
        "{mode:?} fails by outcome, not by setup or admission: {failing:#?}"
    );
}

#[test]
fn a2_a_target_ignoring_the_sign_fails() {
    caught(
        Mode::IgnoresSign,
        &["inverted", "opened", "too-late", "too-soon", "over-cap"],
    );
}

#[test]
fn a2_a_target_ignoring_the_unit_fails() {
    caught(Mode::IgnoresUnit, &["too-soon", "opened", "over-cap"]);
}

#[test]
fn a2_a_target_ignoring_the_base_fails() {
    caught(
        Mode::IgnoresBase,
        &[
            "too-wide", "inverted", "opened", "too-late", "too-soon", "over-cap",
        ],
    );
}

/// The outcome the native interpreter takes for one request, or why it took none.
fn interpreted(input: &[(&str, Node)]) -> String {
    let target = Interpreted::for_model(ir());
    let correlation = ess_primitives::ids::CorrelationId::new("a2-direct").expect("an id");
    let id = ess_conformance::ScenarioId::parse(&scenario("opened")).expect("a scenario id");
    target
        .begin_scenario(&ScenarioContext::new(id, correlation.clone()))
        .expect("begins");
    match target.execute_command(SemanticCommandRequest {
        command: ess_compiler::refs::CommandRef::new("pool.lease.Open".parse().unwrap()),
        actor: None,
        caller: None,
        input: input
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect(),
        correlation,
    }) {
        Ok(result) => result
            .outcome
            .map(|outcome| outcome.to_string())
            .unwrap_or_default(),
        Err(error) => format!("{error:?}"),
    }
}

fn int(value: i64) -> Node {
    Node::Number(Number::from(value))
}

fn at(text: &str) -> Node {
    Node::Text(text.to_owned())
}

const T0: &str = "2020-01-01T00:00:00Z";

/// Ten minutes on: past the five-minute floor and before the hour, so no time guard decides.
const OK: &str = "2020-01-01T00:10:00Z";

fn request(lower: i64, upper: i64, cap: Option<i64>, expires: &str) -> Vec<(&'static str, Node)> {
    let mut input = vec![
        ("lower", int(lower)),
        ("upper", int(upper)),
        ("issued_at", at(T0)),
        ("expires_at", at(expires)),
    ];
    if let Some(cap) = cap {
        input.push(("cap", int(cap)));
    }
    input
}

#[test]
fn a2_the_interpreter_decides_each_boundary_and_one_unit_either_side() {
    for (input, want) in [
        (request(2, 7, Some(7), OK), "pool.lease.Open/opened"),
        (request(2, 8, Some(8), OK), "pool.lease.Open/too-wide"),
        (request(2, -1, Some(2), OK), "pool.lease.Open/opened"),
        (request(2, -2, Some(2), OK), "pool.lease.Open/inverted"),
        (
            request(0, 0, Some(0), "2020-01-01T00:59:59Z"),
            "pool.lease.Open/opened",
        ),
        (
            request(0, 0, Some(0), "2020-01-01T01:00:00Z"),
            "pool.lease.Open/too-late",
        ),
        (
            request(0, 0, Some(0), "2020-01-01T00:05:00Z"),
            "pool.lease.Open/opened",
        ),
        (
            request(0, 0, Some(0), "2020-01-01T00:04:59Z"),
            "pool.lease.Open/too-soon",
        ),
        (request(0, 5, Some(0), OK), "pool.lease.Open/opened"),
        (request(0, 6, Some(0), OK), "pool.lease.Open/too-wide"),
        (request(5, 10, Some(-1), OK), "pool.lease.Open/over-cap"),
        (request(5, 10, Some(0), OK), "pool.lease.Open/opened"),
    ] {
        assert_eq!(interpreted(&input), want, "{input:?}");
    }
}

#[test]
fn a2_extremes_are_exact_in_the_interpreter() {
    // `lower + 5` past i64::MAX: a wrapping target reads `upper > MIN + 2` and refuses; the exact
    // sum is above `upper`, so the lease opens, and its invariant `upper <= lower + 100` holds.
    assert_eq!(
        interpreted(&request(i64::MAX - 2, i64::MAX, Some(i64::MAX), OK)),
        "pool.lease.Open/opened"
    );
    // `lower - 3` past i64::MIN: a wrapping target reads `upper < MAX - 1` and refuses as inverted.
    assert_eq!(
        interpreted(&request(i64::MIN + 1, i64::MIN, Some(i64::MIN), OK)),
        "pool.lease.Open/opened"
    );
    assert_eq!(
        interpreted(&request(i64::MIN, i64::MAX, Some(i64::MAX), OK)),
        "pool.lease.Open/too-wide",
        "the opposite end against the largest offset"
    );
}

#[test]
fn a2_an_absent_optional_base_is_unknown() {
    let absent = interpreted(&request(0, 0, None, OK));
    assert!(
        absent.contains("Unsupported") && absent.contains("over-cap") && absent.contains("Unknown"),
        "{absent}"
    );
}

// ---- the persisted vocabulary: suite /40 and /41 ---------------------------------------------------

#[test]
fn a2_the_offset_invariants_select_suite40_and_old_relabels_are_refused() {
    let suite = suite();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/40"
    );
    let json = suite.to_canonical_json().expect("admitted");
    assert!(
        json.contains(r#""offset": {"#) && json.contains(r#""add": "24h""#),
        "{json}"
    );
    AdmittedSuite::from_json(&json).expect("a /40 reader admits it");
    // `/34` and `/38` are older ordinary majors whose envelope a `/40` suite otherwise satisfies, so
    // the vocabulary is what refuses them.
    for older in ["ess-conformance/34", "ess-conformance/38"] {
        let relabelled = json.replace("ess-conformance/40", older);
        let refused = AdmittedSuite::from_json(&relabelled).expect_err(older);
        let text = refused.to_string();
        assert!(text.contains("suite/40 or /41"), "{older}: {text}");
    }
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
    assert!(selected.original_json().contains(r#""offset""#));
}

#[test]
fn a2_the_rust_runner_fails_each_broken_invariant() {
    let suite = suite();
    for (mode, invariant) in [
        (Mode::BrokenRows, "invariant"),
        (Mode::BrokenInstants, "invariant"),
    ] {
        let failing = not_passed(&run(&suite, &Leases::new(mode)));
        assert!(
            failing.iter().any(|line| line.contains(invariant)),
            "{mode:?}: {failing:#?}"
        );
    }
}

#[test]
fn a2_go_runs_the_offset_suite_at_parity_with_the_rust_runner() {
    let suite = suite();
    let healthy = support_go::assert_parity("a2-healthy", &suite, Leases::new(Mode::Correct));
    assert_eq!(support_go::not_passed(&healthy), Vec::<&str>::new());
    for mode in [Mode::BrokenRows, Mode::BrokenInstants] {
        let broken = support_go::assert_parity(&format!("a2-{mode:?}"), &suite, Leases::new(mode));
        assert!(
            support_go::not_passed(&broken)
                .iter()
                .any(|id| id.contains("/invariant/")),
            "{mode:?}: {broken:#?}"
        );
    }
}

/// A JavaScript implementation of the pool: the interpreter's answers for every branch, and rows
/// whose `upper` breaks the Integer invariant when broken.
const JS_TARGET: &str = r"const mode = process.env.ESS_TARGET_MODE ?? 'healthy';
const errors = {
  'too-wide': 'pool.lease.TooWide',
  inverted: 'pool.lease.Inverted',
  'too-late': 'pool.lease.TooLate',
  'too-soon': 'pool.lease.TooSoon',
  'over-cap': 'pool.lease.OverCap',
};
const big = (value) => BigInt(String(value));
const seconds = (text) => Math.floor(Date.parse(text) / 1000);
class Leases {
  rows = [];
  seq = 0;
  identity() { return { name: 'leases', version: '1' }; }
  beginScenario() { this.rows = []; }
  endScenario() {}
  decide(input) {
    const lower = big(input.lower), upper = big(input.upper);
    const issued = seconds(input.issued_at), expires = seconds(input.expires_at);
    if (upper > lower + 5n) return 'too-wide';
    if (upper < lower - 3n) return 'inverted';
    if (expires >= issued + 3600) return 'too-late';
    if (expires < issued + 300) return 'too-soon';
    if (input.cap === undefined || input.cap === null) throw new Error('an absent cap is unknown');
    if (upper > big(input.cap) + 10n) return 'over-cap';
    return 'opened';
  }
  executeCommand({ command, input }) {
    if (command !== 'pool.lease.Open') throw new Error(`unexpected ${command}`);
    const outcome = this.decide(input);
    if (outcome !== 'opened') {
      return { outcome, consistency: `seq:${this.seq}`, error: errors[outcome] };
    }
    this.seq += 1;
    const id = `00000000-0000-4000-8000-${String(this.seq).padStart(12, '0')}`;
    this.rows.push({
      lease_id: id,
      lower: input.lower,
      upper: input.upper,
      issued_at: input.issued_at,
      expires_at: input.expires_at,
    });
    return {
      outcome: 'opened',
      consistency: `seq:${this.seq}`,
      directEvents: [{ event: 'pool.lease.Opened', payload: { lease_id: id } }],
    };
  }
  queryView() {
    return {
      rows: this.rows.map((row) =>
        mode === 'broken' ? { ...row, upper: Number(row.lower) + 101 } : { ...row },
      ),
    };
  }
  observeEvents() { throw new Error('unused'); }
  configureExternalOutcome() { throw new Error('nothing is external'); }
  redeliverEvent() { throw new Error('no bindings'); }
}
export function makeTarget() { return new Leases(); }
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
        .join("expression-a2")
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
fn a2_typescript_runs_the_offset_suite_and_fails_the_broken_invariant() {
    let Some(healthy) = typescript_verdicts("healthy") else {
        return;
    };
    let failing: Vec<&String> = healthy
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id)
        .collect();
    assert_eq!(failing, Vec::<&String>::new(), "{healthy:#?}");
    assert!(
        healthy.keys().any(|id| id.contains("/invariant/")),
        "{healthy:#?}"
    );
    let broken = typescript_verdicts("broken").expect("tools were found once");
    assert!(
        broken
            .iter()
            .any(|(id, status)| id.contains("/invariant/") && status == "failed"),
        "{broken:#?}"
    );
}

// ---- the shared vectors in the generated Go reader --------------------------------------------

#[test]
fn a2_the_generated_go_reader_answers_the_shared_offset_vectors() {
    let document = serde_json::json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "pool",
            "specification_version": "v1", "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"pool.lease/authored/offset": {"purpose": "Compare with an offset",
            "steps": [{"step": "expect_view", "view": "pool.lease.Leases",
                "expectation": {"expect": "satisfies", "predicate": "upper >= lower"}}],
            "source": []}}
    })
    .to_string();
    let suite = AdmittedSuite::from_json(&document).expect("admitted");
    let directory = std::env::temp_dir().join(format!("ess-a2-offset-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("essconform")).expect("directory");
    for artifact in ess_conformance::go::emit(suite.suite()).expect("emitted") {
        std::fs::write(directory.join(artifact.path), artifact.contents).expect("written");
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/a2\n\ngo 1.24\n",
    )
    .expect("go.mod");
    std::fs::write(
        directory.join("essconform/offset_operand_test.go"),
        include_str!("fixtures/offset-operand.go"),
    )
    .expect("fixture");
    std::fs::write(
        directory.join("essconform/offset-operand.json"),
        include_str!("../../../specify/ess-primitives/tests/vectors/offset-operand.json"),
    )
    .expect("vectors");
    let result = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestOffsetOperand",
            "-count=1",
            "-v",
        ])
        .env("GOWORK", "off")
        .current_dir(&directory)
        .output()
        .expect("go runs");
    let printed = format!(
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    eprintln!("{printed}");
    assert!(
        result.status.success()
            && printed.contains("--- PASS: TestOffsetOperand ")
            && printed.contains("--- PASS: TestOffsetOperandFaults"),
        "Go offset reader: {}",
        result.status
    );
}

// ---- below ess/22 nothing moves -------------------------------------------------------------------

#[test]
fn a2_a_model_without_an_offset_keeps_its_suite_format() {
    let mut plain = MODEL.to_owned();
    for (offset, literal) in [
        ("upper <= lower + 100", "upper >= 0"),
        ("expires_at <= issued_at + 24h", "lower >= 0"),
        ("when: upper > lower + 5", "when: upper > 5"),
        ("when: upper < lower - 3", "when: upper < 0"),
        (
            "when: expires_at >= issued_at + 1h",
            "when: expires_at >= \"2030-01-01T00:00:00Z\"",
        ),
        (
            "when: expires_at < issued_at + 5m",
            "when: expires_at < \"2000-01-01T00:00:00Z\"",
        ),
        ("when: upper > cap + 10", "when: cap > 10"),
    ] {
        assert!(plain.contains(offset), "the model writes `{offset}`");
        plain = plain.replace(offset, literal);
    }
    let model = ir_of(&plain);
    assert!(
        !model.to_canonical_json().contains("\"offset\""),
        "the control carries no offset: {plain}"
    );
    let suite = ess_conformance::synthesize::synthesize(&model).suite;
    assert!(!ess_conformance::expression_format::used_by(&suite));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
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
fn a2_the_browser_product_admits_and_runs_the_offset_suite() {
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
    let handle = product
        .load(&manifest, blobs)
        .expect("a /40 suite is admitted");
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
        passed.contains("/invariant/") && passed.contains("too-late"),
        "{passed}"
    );
}
