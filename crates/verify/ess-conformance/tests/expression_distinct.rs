//! Family F part C executed (beyond10x/ess#237): `distinct: {in, as, by}`.
//!
//! A command refuses a bundle whose files share a path, whose tags repeat, or whose stamps name one
//! instant twice; the bundle it opens holds all three distinct. The suite is synthesized from the
//! model and run against the native interpreter, which must pass it, and against targets wrong in
//! one way each — comparing neighbours only, comparing whole files instead of their paths, comparing
//! instants by their spelling — which must each fail a scenario that decides the guard. The
//! invariants persist as `{distinct: …}` in `satisfies`, select suite `/40`, and are executed by
//! the Rust, Go and TypeScript runners against a healthy row, a row holding a duplicate, and a row
//! whose key is absent, which no runner may read as distinct.
//! `docs/design/expression-family-source22.md`, section `distinct`, is the design.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use ess_primitives::time::Rfc3339Instant;

mod support_go;

const MODEL: &str = r"format: ess/22
system: pool
version: v1
domain: pool.files
types:
  - name: pool.files.File
    kind: struct
    fields:
      - {name: path, type: String}
      - {name: size, type: Integer}
entities:
  - name: pool.files.Bundle
    identity: {name: bundle_id, type: Uuid}
    fields:
      - {name: files, type: List<pool.files.File>}
      - {name: tags, type: List<String>}
      - {name: stamps, type: List<Timestamp>}
    invariants:
      - {distinct: {in: files, as: file, by: file.path}}
      - {distinct: {in: tags, as: tag}}
      - {distinct: {in: stamps, as: stamp}}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: pool.files.DuplicatePath, summary: Two files share a path.}
  - {name: pool.files.DuplicateTag, summary: A tag repeats.}
  - {name: pool.files.DuplicateStamp, summary: Two stamps name one instant.}
events:
  - name: pool.files.Opened
    fields:
      - {name: bundle_id, type: Uuid}
commands:
  - name: pool.files.Open
    input:
      - {name: files, type: List<pool.files.File>}
      - {name: tags, type: List<String>}
      - {name: stamps, type: List<Timestamp>}
    outcomes:
      - name: duplicate-path
        when: {not: {distinct: {in: files, as: file, by: file.path}}}
        error: pool.files.DuplicatePath
      - name: duplicate-tag
        when: {not: {distinct: {in: tags, as: tag}}}
        error: pool.files.DuplicateTag
      - name: duplicate-stamp
        when: {not: {distinct: {in: stamps, as: stamp}}}
        error: pool.files.DuplicateStamp
      - name: opened
        creates: pool.files.Bundle
        instance: bundle_id
        emits: [pool.files.Opened]
        payload:
          pool.files.Opened: {bundle_id: {generated: true}}
        sets: {files: input.files, tags: input.tags, stamps: input.stamps}
views:
  - name: pool.files.Bundles
    source: pool.files.Bundle
    consistency: read_your_writes
    fields:
      - {name: bundle_id, type: Uuid}
      - {name: files, type: List<pool.files.File>}
      - {name: tags, type: List<String>}
      - {name: stamps, type: List<Timestamp>}
";

const OUTCOMES: [&str; 4] = [
    "duplicate-path",
    "duplicate-tag",
    "duplicate-stamp",
    "opened",
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
    format!("pool.files.Open/outcome/{outcome}")
}

// ---- the reference decision and the faulty ones --------------------------------------------------

/// How a target decides the three guards, or what it publishes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Correct,
    /// Compares each element with its neighbour only.
    AdjacentOnly,
    /// Compares whole files rather than their paths.
    IgnoresBy,
    /// Compares stamps by their spelling rather than the instant.
    ComparesSpellings,
    /// Publishes every row with its first file repeated: the path invariant is false.
    BrokenRows,
    /// Publishes every row with its first file's path left out: the path invariant is unknown,
    /// which a runner reading an absent key as distinct would pass.
    DropsKey,
}

fn list<'a>(input: &'a BTreeMap<String, Node>, name: &str) -> &'a [Node] {
    match input.get(name) {
        Some(Node::Seq(items)) => items,
        _ => &[],
    }
}

fn path_of(file: &Node) -> Option<Node> {
    match file {
        Node::Map(fields) => fields.get("path").cloned(),
        _ => None,
    }
}

fn instant_of(stamp: &Node) -> Option<Node> {
    match stamp {
        Node::Text(text) => {
            Rfc3339Instant::parse_rfc3339(text).map(|at| Node::Text(at.to_rfc3339()))
        }
        _ => None,
    }
}

/// Whether two of `keys` are equal: every pair, or neighbours only.
fn repeats(keys: &[Node], adjacent_only: bool) -> bool {
    keys.iter().enumerate().any(|(index, key)| {
        keys[index + 1..]
            .iter()
            .take(if adjacent_only { 1 } else { usize::MAX })
            .any(|other| other == key)
    })
}

fn keys(items: &[Node], key: impl Fn(&Node) -> Option<Node>) -> Vec<Node> {
    items.iter().filter_map(key).collect()
}

/// The branch `mode` takes for `input`, deciding each guard exactly as the model does but for the
/// one way `mode` gets wrong.
fn decide(mode: Mode, input: &BTreeMap<String, Node>) -> &'static str {
    let adjacent = mode == Mode::AdjacentOnly;
    let files = list(input, "files");
    let paths = if mode == Mode::IgnoresBy {
        files.to_vec()
    } else {
        keys(files, path_of)
    };
    if repeats(&paths, adjacent) {
        return "duplicate-path";
    }
    if repeats(list(input, "tags"), adjacent) {
        return "duplicate-tag";
    }
    let stamps = list(input, "stamps");
    let instants = if mode == Mode::ComparesSpellings {
        stamps.to_vec()
    } else {
        keys(stamps, instant_of)
    };
    if repeats(&instants, adjacent) {
        return "duplicate-stamp";
    }
    "opened"
}

/// How a list's key is read off one of its elements.
type KeyOf = fn(&Node) -> Option<Node>;

/// `input` with every list up to `outcome`'s own made distinct, so the model takes `outcome`: the
/// interpreter answers for the branch a faulty target decided.
fn toward(mut input: BTreeMap<String, Node>, outcome: &str) -> BTreeMap<String, Node> {
    let lists: [(&str, KeyOf); 3] = [
        ("files", path_of),
        ("tags", |tag| Some(tag.clone())),
        ("stamps", instant_of),
    ];
    let keep = OUTCOMES
        .iter()
        .position(|name| *name == outcome)
        .unwrap_or(3);
    for (name, key) in lists.iter().take(keep) {
        let mut seen = Vec::new();
        let distinct: Vec<Node> = list(&input, name)
            .iter()
            .filter(|item| {
                let Some(key) = key(item) else { return true };
                if seen.contains(&key) {
                    return false;
                }
                seen.push(key);
                true
            })
            .cloned()
            .collect();
        input.insert((*name).to_owned(), Node::Seq(distinct));
    }
    input
}

/// The interpreter, with the guards decided by `mode`, or its rows broken by it.
struct Bundles {
    inner: Interpreted,
    mode: Mode,
}

impl Bundles {
    fn new(mode: Mode) -> Self {
        Self {
            inner: Interpreted::for_model(ir()),
            mode,
        }
    }
}

impl ConformanceTarget for Bundles {
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
        let (correct, faulty) = (
            decide(Mode::Correct, &request.input),
            decide(self.mode, &request.input),
        );
        if correct != faulty {
            request.input = toward(request.input, faulty);
        }
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        for row in &mut result.rows {
            let Some(Node::Seq(files)) = row.get_mut("files") else {
                continue;
            };
            match (self.mode, files.first().cloned()) {
                (Mode::BrokenRows, Some(first)) => files.push(first),
                (Mode::DropsKey, Some(Node::Map(mut first))) => {
                    first.remove("path");
                    files[0] = Node::Map(first);
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

/// The last input the scenario `id` sends.
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
fn distinct_every_branch_is_witnessed_and_the_interpreter_passes() {
    let suite = suite();
    for outcome in OUTCOMES {
        let input = sent(&suite, &scenario(outcome));
        assert_eq!(
            decide(Mode::Correct, &input),
            outcome,
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

#[test]
fn distinct_the_witnesses_are_decisive_not_vacuous() {
    let suite = suite();
    let opened = sent(&suite, &scenario("opened"));
    for name in ["files", "tags", "stamps"] {
        assert!(
            list(&opened, name).len() >= 2,
            "the distinct case holds two or more `{name}`: {opened:#?}"
        );
    }
    let duplicate = sent(&suite, &scenario("duplicate-path"));
    let files = list(&duplicate, "files");
    let paths = keys(files, path_of);
    assert!(
        repeats(&paths, false) && !repeats(&paths, true),
        "the duplicate path is not between neighbours: {files:#?}"
    );
    assert!(
        !repeats(files, false),
        "the files differ besides their path: {files:#?}"
    );
    let duplicate = sent(&suite, &scenario("duplicate-stamp"));
    let stamps = list(&duplicate, "stamps");
    assert!(
        repeats(&keys(stamps, instant_of), false) && !repeats(stamps, false),
        "one instant spelled two ways: {stamps:#?}"
    );
}

fn caught(mode: Mode, deciding: &[&str]) {
    let suite = suite();
    let healthy = run(&suite, &Bundles::new(Mode::Correct));
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    let failing = not_passed(&run(&suite, &Bundles::new(mode)));
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
fn distinct_a_target_comparing_neighbours_only_fails() {
    caught(
        Mode::AdjacentOnly,
        &["duplicate-path", "duplicate-tag", "duplicate-stamp"],
    );
}

#[test]
fn distinct_a_target_ignoring_by_fails() {
    caught(Mode::IgnoresBy, &["duplicate-path"]);
}

#[test]
fn distinct_a_target_comparing_spellings_fails() {
    caught(Mode::ComparesSpellings, &["duplicate-stamp"]);
}

// ---- the persisted vocabulary: suite /40 and /41 ---------------------------------------------------

/// Removes the key kind of every `distinct` under `value`, counting them.
fn unkind(value: &mut serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(fields) => {
            let mut removed = 0;
            if let Some(serde_json::Value::Object(distinct)) = fields.get_mut("distinct") {
                removed += usize::from(distinct.remove("kind").is_some());
            }
            removed + fields.values_mut().map(unkind).sum::<usize>()
        }
        serde_json::Value::Array(items) => items.iter_mut().map(unkind).sum(),
        _ => 0,
    }
}

#[test]
fn distinct_the_invariants_select_suite40_and_old_relabels_are_refused() {
    let suite = suite();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/40"
    );
    let json = suite.to_canonical_json().expect("admitted");
    assert!(
        json.contains(r#""distinct": {"#)
            && json.contains(r#""kind": "string""#)
            && json.contains(r#""kind": "timestamp""#),
        "{json}"
    );
    AdmittedSuite::from_json(&json).expect("a /40 reader admits it");
    for older in ["ess-conformance/34", "ess-conformance/38"] {
        let relabelled = json.replace("ess-conformance/40", older);
        let refused = AdmittedSuite::from_json(&relabelled).expect_err(older);
        let text = refused.to_string();
        assert!(text.contains("suite/40 or /41"), "{older}: {text}");
    }
    let mut document: serde_json::Value = serde_json::from_str(&json).expect("JSON");
    assert!(unkind(&mut document) >= 3, "every key kind is written");
    let unkinded = document.to_string();
    let refused = AdmittedSuite::from_json(&unkinded).expect_err("a key without its kind");
    assert!(refused.to_string().contains("kind"), "{refused}");
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
    assert!(selected.original_json().contains(r#""distinct""#));
}

#[test]
fn distinct_the_rust_runner_fails_a_duplicate_row_and_an_absent_key() {
    let suite = suite();
    for mode in [Mode::BrokenRows, Mode::DropsKey] {
        let failing = not_passed(&run(&suite, &Bundles::new(mode)));
        assert!(
            failing.iter().any(|line| line.contains("/invariant/")),
            "{mode:?}: {failing:#?}"
        );
    }
}

#[test]
fn distinct_go_runs_the_suite_at_parity_with_the_rust_runner() {
    let suite = suite();
    let healthy =
        support_go::assert_parity("distinct-healthy", &suite, Bundles::new(Mode::Correct));
    assert_eq!(support_go::not_passed(&healthy), Vec::<&str>::new());
    let broken =
        support_go::assert_parity("distinct-broken", &suite, Bundles::new(Mode::BrokenRows));
    assert!(
        support_go::not_passed(&broken)
            .iter()
            .any(|id| id.contains("/invariant/")),
        "{broken:#?}"
    );
    // An absent key is Unknown, which neither runner passes. They name it differently, as they do
    // every undecidable row: Rust reports the scenario `error`, Go `failed` (`runtime.go`'s
    // `decide` → `fail`). That split predates this construct; parity is asserted on everything else.
    let invariant = "pool.files.Bundle/invariant/after/pool.files.Open/opened";
    let (rust, replayed) =
        support_go::compare("distinct-dropped", &suite, Bundles::new(Mode::DropsKey));
    assert_eq!(
        rust.get(invariant).map(String::as_str),
        Some("error"),
        "{rust:#?}"
    );
    assert_eq!(
        replayed.go.outcomes.get(invariant).map(String::as_str),
        Some("failed"),
        "{}",
        replayed.go.log
    );
    assert!(
        replayed.go.log.contains("cannot be decided"),
        "Go reads the absent key as Unknown: {}",
        replayed.go.log
    );
    for (id, verdict) in &rust {
        if id != invariant {
            assert_eq!(replayed.go.outcomes.get(id), Some(verdict), "{id}");
        }
    }
    assert!(
        replayed.divergences.is_empty(),
        "{:?}",
        replayed.divergences
    );
}

/// A JavaScript implementation of the bundles: every guard decided as the model decides it, or by
/// neighbours only (`adjacent`), whole files (`whole`) or stamp spellings (`spelling`), and rows that
/// repeat a file (`broken`) or drop a path (`dropped`).
const JS_TARGET: &str = r"const mode = process.env.ESS_TARGET_MODE ?? 'healthy';
const errors = {
  'duplicate-path': 'pool.files.DuplicatePath',
  'duplicate-tag': 'pool.files.DuplicateTag',
  'duplicate-stamp': 'pool.files.DuplicateStamp',
};
const repeats = (keys) =>
  mode === 'adjacent'
    ? keys.some((key, index) => index > 0 && key === keys[index - 1])
    : new Set(keys).size !== keys.length;
class Bundles {
  rows = [];
  seq = 0;
  identity() { return { name: 'bundles', version: '1' }; }
  beginScenario() { this.rows = []; }
  endScenario() {}
  decide(input) {
    const paths = input.files.map((file) => (mode === 'whole' ? JSON.stringify(file) : file.path));
    if (repeats(paths)) return 'duplicate-path';
    if (repeats(input.tags)) return 'duplicate-tag';
    const stamps = input.stamps.map((stamp) => (mode === 'spelling' ? stamp : Date.parse(stamp)));
    if (repeats(stamps)) return 'duplicate-stamp';
    return 'opened';
  }
  executeCommand({ command, input }) {
    if (command !== 'pool.files.Open') throw new Error(`unexpected ${command}`);
    const outcome = this.decide(input);
    if (outcome !== 'opened') {
      return { outcome, consistency: `seq:${this.seq}`, error: errors[outcome] };
    }
    this.seq += 1;
    const id = `00000000-0000-4000-8000-${String(this.seq).padStart(12, '0')}`;
    this.rows.push({ bundle_id: id, files: input.files, tags: input.tags, stamps: input.stamps });
    return {
      outcome: 'opened',
      consistency: `seq:${this.seq}`,
      directEvents: [{ event: 'pool.files.Opened', payload: { bundle_id: id } }],
    };
  }
  queryView() {
    return {
      rows: this.rows.map((row) => {
        if (mode === 'broken') return { ...row, files: [...row.files, row.files[0]] };
        if (mode === 'dropped') {
          const [first, ...rest] = row.files;
          const { path, ...kept } = first;
          return { ...row, files: [kept, ...rest] };
        }
        return { ...row };
      }),
    };
  }
  observeEvents() { throw new Error('unused'); }
  configureExternalOutcome() { throw new Error('nothing is external'); }
  redeliverEvent() { throw new Error('no bindings'); }
}
export function makeTarget() { return new Bundles(); }
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
        .join("expression-distinct")
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
fn distinct_typescript_runs_the_suite_and_fails_a_duplicate_row_and_an_absent_key() {
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
    for mode in ["broken", "dropped"] {
        let broken = typescript_verdicts(mode).expect("tools were found once");
        assert!(
            broken
                .iter()
                .any(|(id, status)| id.contains("/invariant/") && status == "failed"),
            "{mode}: {broken:#?}"
        );
    }
}

/// The scenarios a faulty guard decision can fail: every outcome a duplicate decides.
const DECIDING: [&str; 3] = ["duplicate-path", "duplicate-tag", "duplicate-stamp"];

#[test]
fn distinct_go_fails_each_faulty_guard_at_parity_with_the_rust_runner() {
    let suite = suite();
    for mode in [Mode::AdjacentOnly, Mode::IgnoresBy, Mode::ComparesSpellings] {
        let verdicts =
            support_go::assert_parity(&format!("distinct-{mode:?}"), &suite, Bundles::new(mode));
        let failing = support_go::not_passed(&verdicts);
        assert!(
            failing.iter().any(|id| DECIDING
                .iter()
                .any(|outcome| id.starts_with(&scenario(outcome)))),
            "{mode:?}: {verdicts:#?}"
        );
        assert!(
            failing.iter().all(|id| verdicts[*id] == "failed"),
            "{mode:?} fails by outcome: {verdicts:#?}"
        );
    }
}

#[test]
fn distinct_typescript_fails_each_faulty_guard() {
    for mode in ["adjacent", "whole", "spelling"] {
        let Some(verdicts) = typescript_verdicts(mode) else {
            return;
        };
        assert!(
            verdicts.iter().any(|(id, status)| status == "failed"
                && DECIDING
                    .iter()
                    .any(|outcome| id.starts_with(&scenario(outcome)))),
            "{mode}: {verdicts:#?}"
        );
    }
}

// ---- the shared vectors in the generated Go reader --------------------------------------------

#[test]
fn distinct_the_generated_go_reader_answers_the_shared_vectors() {
    let document = serde_json::json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "pool",
            "specification_version": "v1", "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"pool.files/authored/distinct": {"purpose": "Hold a list distinct",
            "steps": [{"step": "expect_view", "view": "pool.files.Bundles",
                "expectation": {"expect": "satisfies", "predicate": "defined(bundle_id)"}}],
            "source": []}}
    })
    .to_string();
    let suite = AdmittedSuite::from_json(&document).expect("admitted");
    let directory = support_go::Scratch::adopt(
        std::env::temp_dir().join(format!("ess-distinct-{}", std::process::id())),
    );
    std::fs::create_dir_all(directory.join("essconform")).expect("directory");
    for artifact in ess_conformance::go::emit(suite.suite()).expect("emitted") {
        std::fs::write(directory.join(artifact.path), artifact.contents).expect("written");
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/distinct\n\ngo 1.24\n",
    )
    .expect("go.mod");
    std::fs::write(
        directory.join("essconform/distinct_test.go"),
        include_str!("fixtures/distinct.go"),
    )
    .expect("fixture");
    std::fs::write(
        directory.join("essconform/distinct.json"),
        include_str!("../../../specify/ess-primitives/tests/vectors/distinct.json"),
    )
    .expect("vectors");
    let result = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestDistinct",
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
            && printed.contains("--- PASS: TestDistinct ")
            && printed.contains("--- PASS: TestDistinctFaults")
            && printed.contains("--- PASS: TestDistinctAdmission"),
        "Go distinct reader: {}",
        result.status
    );
}

// ---- below ess/22 and without distinct nothing moves --------------------------------------------

#[test]
fn distinct_a_model_without_distinct_keeps_its_suite_format() {
    let mut plain = MODEL.to_owned();
    for (distinct, literal) in [
        (
            "      - {distinct: {in: files, as: file, by: file.path}}\n",
            "      - defined(bundle_id)\n",
        ),
        ("      - {distinct: {in: tags, as: tag}}\n", ""),
        ("      - {distinct: {in: stamps, as: stamp}}\n", ""),
        (
            "when: {not: {distinct: {in: files, as: file, by: file.path}}}",
            "when: {forall: {in: files, as: file, that: file.size > 5}}",
        ),
        (
            "when: {not: {distinct: {in: tags, as: tag}}}",
            "when: {exists: {in: tags, as: tag, that: tag == x}}",
        ),
        (
            "when: {not: {distinct: {in: stamps, as: stamp}}}",
            "when: {exists: {in: stamps, as: stamp, that: stamp == y}}",
        ),
    ] {
        assert!(plain.contains(distinct), "the model writes `{distinct}`");
        plain = plain.replacen(distinct, literal, 1);
    }
    let model = ir_of(&plain);
    assert!(
        !model.to_canonical_json().contains("\"distinct\""),
        "the control carries no distinct: {plain}"
    );
    let suite = ess_conformance::synthesize::synthesize(&model).suite;
    assert!(!ess_conformance::expression_format::used_by(&suite));
    assert_ne!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/40"
    );
    assert!(!suite
        .to_canonical_json()
        .expect("serialises")
        .contains("distinct"));
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
fn distinct_the_browser_product_admits_and_runs_the_suite() {
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
        passed.contains("/invariant/") && passed.contains("duplicate-stamp"),
        "{passed}"
    );
}

// ---- finite key domains (correction round) --------------------------------------------------------

/// A command over `List<element>` that refuses a length other than `length`, then a duplicate.
fn finite(element: &str, length: usize) -> String {
    format!(
        r"format: ess/22
system: pool
version: v1
domain: pool.keys
types:
  - name: pool.keys.One
    kind: enum
    variants: [Only]
errors:
  - {{name: pool.keys.WrongLength, summary: The list has the wrong length.}}
  - {{name: pool.keys.Duplicate, summary: Two keys are equal.}}
events:
  - {{name: pool.keys.Accepted, fields: []}}
commands:
  - name: pool.keys.Pick
    input:
      - {{name: keys, type: 'List<{element}>'}}
    outcomes:
      - name: wrong-length
        when: keys.count != {length}
        error: pool.keys.WrongLength
      - name: duplicate
        when: {{not: {{distinct: {{in: keys, as: key}}}}}}
        error: pool.keys.Duplicate
      - name: accepted
        emits: [pool.keys.Accepted]
"
    )
}

/// The design: "A finite/singleton key domain which cannot supply enough unequal values yields the
/// existing named no-witness refusal". Not `GuardUnsatisfiable`, which says a longer search might
/// have found one; and the two refusing branches are still witnessed at the required length.
#[test]
fn distinct_a_finite_key_domain_too_small_for_the_length_is_no_witness() {
    for (element, length, domain) in [
        ("pool.keys.One", 2, "pool.keys.One"),
        ("Boolean", 3, "Boolean"),
    ] {
        let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&finite(element, length)));
        let accepted: Vec<_> = synthesis
            .refusals
            .iter()
            .filter(|refusal| {
                refusal
                    .scenario
                    .as_ref()
                    .map(ToString::to_string)
                    .as_deref()
                    == Some("pool.keys.Pick/outcome/accepted")
            })
            .collect();
        assert_eq!(accepted.len(), 1, "{element}: {:#?}", synthesis.refusals);
        match &accepted[0].cause {
            ess_conformance::synthesize::RefusalCause::NoWitness(gap) => {
                assert_eq!(gap.path, "keys", "{gap:?}");
                assert!(gap.type_ref.contains(domain), "{gap:?}");
            }
            other => panic!("{element}: a named no-witness refusal, not {other:?}"),
        }
        assert_eq!(
            synthesis.refusals.len(),
            1,
            "{element}: {:#?}",
            synthesis.refusals
        );
        let duplicate = sent(&synthesis.suite, "pool.keys.Pick/outcome/duplicate");
        assert_eq!(
            list(&duplicate, "keys").len(),
            length,
            "{element}: {duplicate:#?}"
        );
    }
}

// ---- a stored subject's list (correction round) ----------------------------------------------------

/// A bundle opened with `tags` and sealed unless its stored tags repeat; with `cap`, opening more
/// than one tag is refused, so no stored list holds two.
fn stored(cap: bool) -> String {
    let refused = if cap {
        "      - name: too-many\n        when: tags.count > 1\n        error: pool.tags.TooMany\n"
    } else {
        ""
    };
    format!(
        r"format: ess/22
system: pool
version: v1
domain: pool.tags
entities:
  - name: pool.tags.Bundle
    identity: {{name: bundle_id, type: Uuid}}
    fields:
      - {{name: tags, type: List<String>}}
    lifecycle:
      initial: Open
      states: [Open, Sealed]
      terminal: [Sealed]
      transitions:
        - {{name: seal, from: [Open], to: Sealed}}
events:
  - name: pool.tags.Opened
    fields:
      - {{name: bundle_id, type: Uuid}}
  - {{name: pool.tags.Sealed, fields: []}}
errors:
  - {{name: pool.tags.Duplicate, fields: []}}
  - {{name: pool.tags.TooMany, fields: []}}
commands:
  - name: pool.tags.Open
    input:
      - {{name: tags, type: List<String>}}
    outcomes:
{refused}      - name: opened
        creates: pool.tags.Bundle
        instance: bundle_id
        sets: {{tags: input.tags}}
        emits: [pool.tags.Opened]
        payload:
          pool.tags.Opened:
            bundle_id: {{generated: true}}
  - name: pool.tags.Seal
    input:
      - {{name: bundle_id, type: Uuid}}
    outcomes:
      - name: duplicate
        when_subject:
          predicate: {{not: {{distinct: {{in: tags, as: tag}}}}}}
        error: pool.tags.Duplicate
      - name: sealed
        moves: pool.tags.Bundle.seal
        instance: bundle_id
        emits: [pool.tags.Sealed]
views:
  - name: pool.tags.Bundles
    source: pool.tags.Bundle
    consistency: read_your_writes
    fields:
      - {{name: bundle_id, type: Uuid}}
      - {{name: state, type: pool.tags.Bundle.State}}
      - {{name: tags, type: List<String>}}
"
    )
}

/// The tags the scenario `id` opens its bundle with.
fn opened_tags(suite: &ConformanceSuite, id: &str) -> Vec<Node> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario);
    scenario
        .steps
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "pool.tags.Open" =>
            {
                match input.get("tags") {
                    Some(ess_conformance::ScenarioValue::Literal {
                        value: Node::Seq(items),
                    }) => Some(items.clone()),
                    _ => None,
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("{id} opens a bundle"))
}

/// The design: "a decisive positive test also requires `list.count > 1`", for a stored list too:
/// both branches of a `when_subject` `distinct` arrange two or more stored tags, and the
/// interpreter passes both.
#[test]
fn distinct_both_branches_of_a_stored_guard_arrange_two_or_more_elements() {
    let model = ir_of(&stored(false));
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    // The model declares no refusal for sealing a sealed bundle (`RefusalUndeclared`), which is
    // not what this checks; no branch of the guard is refused.
    assert!(
        synthesis.refusals.iter().all(|refusal| !matches!(
            refusal.cause,
            ess_conformance::synthesize::RefusalCause::NoWitness(_)
                | ess_conformance::synthesize::RefusalCause::GuardUnsatisfiable { .. }
        )),
        "{:#?}",
        synthesis.refusals
    );
    for outcome in ["duplicate", "sealed"] {
        let tags = opened_tags(
            &synthesis.suite,
            &format!("pool.tags.Seal/outcome/{outcome}"),
        );
        assert!(tags.len() >= 2, "{outcome}: {tags:#?}");
    }
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model));
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
}

/// Where no row the arrangement can leave holds two stored tags, the accepting branch is refused
/// by name rather than witnessed over a list that decides nothing.
#[test]
fn distinct_a_stored_list_that_cannot_hold_two_is_refused_by_name() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&stored(true)));
    let sealed = synthesis
        .refusals
        .iter()
        .find(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some("pool.tags.Seal/outcome/sealed")
        })
        .unwrap_or_else(|| panic!("sealed is refused: {:#?}", synthesis.refusals));
    match &sealed.cause {
        ess_conformance::synthesize::RefusalCause::NoWitness(gap) => {
            assert_eq!(gap.path, "tags", "{gap:?}");
            assert!(gap.reason.contains("vacuously"), "{gap:?}");
        }
        other => panic!("a named no-witness refusal, not {other:?}"),
    }
}
