//! Family F part C executed (beyond10x/ess#233): the UTF-8 byte length of a String.
//!
//! A command refuses a label longer than eight UTF-8 bytes (`label.utf8_bytes > 8`) and a code of
//! exactly four (`code.utf8_bytes == 4`); the note it files holds `label.utf8_bytes <= 64`. The
//! suite is synthesized from the model and run against the native interpreter, which must pass it,
//! and against targets that measure the text wrongly — by Unicode scalar values, by UTF-16 code
//! units, by graphemes — which must each fail a scenario that decides a guard: the witnesses carry
//! multi-byte text, so no measure but the byte length agrees with every one. The invariant persists
//! as `{utf8_bytes: label}` in `satisfies`, selects suite `/40`, and is executed by the Rust, Go and
//! TypeScript runners against a healthy and a broken row. `docs/design/expression-family-source22.md`,
//! "String `.utf8_bytes`", is the design.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

mod support_go;

const MODEL: &str = r"format: ess/22
system: notes
version: v1
domain: notes.core
entities:
  - name: notes.core.Note
    identity: {name: note_id, type: Uuid}
    fields:
      - {name: label, type: String}
      - {name: code, type: String}
    invariants:
      - label.utf8_bytes <= 64
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: notes.core.TooLong, summary: The label is too long.}
  - {name: notes.core.WideCode, summary: The code is four bytes wide.}
events:
  - name: notes.core.Filed
    fields:
      - {name: note_id, type: Uuid}
commands:
  - name: notes.core.File
    input:
      - {name: label, type: String}
      - {name: code, type: String}
    outcomes:
      - name: too-long
        when: label.utf8_bytes > 8
        error: notes.core.TooLong
      - name: wide-code
        when: code.utf8_bytes == 4
        error: notes.core.WideCode
      - name: filed
        creates: notes.core.Note
        instance: note_id
        emits: [notes.core.Filed]
        payload:
          notes.core.Filed: {note_id: {generated: true}}
        sets: {label: input.label, code: input.code}
views:
  - name: notes.core.Notes
    source: notes.core.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: Uuid}
      - {name: label, type: String}
      - {name: code, type: String}
";

const OUTCOMES: [&str; 3] = ["too-long", "wide-code", "filed"];

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
    format!("notes.core.File/outcome/{outcome}")
}

// ---- the reference measure and the faulty ones ----------------------------------------------

/// How a target measures a text, or which of its rows it breaks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Correct,
    /// Unicode scalar values: Rust's `chars().count()`, Go's `utf8.RuneCountInString`, the
    /// code points JavaScript's `[...text]` counts.
    Scalars,
    /// UTF-16 code units: JavaScript's `text.length`.
    Utf16Units,
    /// Graphemes, as far as these witnesses need: a combining mark joins the scalar before it.
    Graphemes,
    /// Publishes every row's label as 33 `é`: 66 bytes, 33 scalars, which breaks the invariant.
    BrokenRows,
}

impl Mode {
    fn measure(self, text: &str) -> usize {
        match self {
            Self::Correct | Self::BrokenRows => text.len(),
            Self::Scalars => text.chars().count(),
            Self::Utf16Units => text.encode_utf16().count(),
            Self::Graphemes => text
                .chars()
                .filter(|character| !('\u{300}'..='\u{36f}').contains(character))
                .count(),
        }
    }
}

fn text<'a>(input: &'a BTreeMap<String, Node>, name: &str) -> Option<&'a str> {
    match input.get(name)? {
        Node::Text(text) => Some(text),
        _ => None,
    }
}

/// The branch `mode` takes for `input`, deciding each guard as the model does but for the measure
/// `mode` gets wrong; `None` where a guard reads an absent value.
fn decide(mode: Mode, input: &BTreeMap<String, Node>) -> Option<&'static str> {
    let (label, code) = (text(input, "label")?, text(input, "code")?);
    if mode.measure(label) > 8 {
        return Some("too-long");
    }
    if mode.measure(code) == 4 {
        return Some("wide-code");
    }
    Some("filed")
}

/// `input` moved so that the model takes `outcome` for it: the interpreter answers for the branch
/// a faulty target decided.
fn toward(mut input: BTreeMap<String, Node>, outcome: &str) -> BTreeMap<String, Node> {
    input.insert("label".to_owned(), Node::Text("ab".to_owned()));
    input.insert("code".to_owned(), Node::Text("c".to_owned()));
    match outcome {
        "too-long" => drop(input.insert("label".to_owned(), Node::Text("abcdefghi".to_owned()))),
        "wide-code" => drop(input.insert("code".to_owned(), Node::Text("abcd".to_owned()))),
        _ => {}
    }
    input
}

/// The interpreter, with the byte-length guards decided by `mode`, or its rows broken by it.
struct Notes {
    inner: Interpreted,
    mode: Mode,
}

impl Notes {
    fn new(mode: Mode) -> Self {
        Self {
            inner: Interpreted::for_model(ir()),
            mode,
        }
    }
}

impl ConformanceTarget for Notes {
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
        if self.mode == Mode::BrokenRows {
            for row in &mut result.rows {
                row.insert("label".to_owned(), Node::Text("é".repeat(33)));
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
fn utf8_every_branch_is_witnessed_and_the_interpreter_passes() {
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
        "the invariant is asserted, not skipped: {statuses:#?}"
    );
}

#[test]
fn utf8_the_witnesses_are_multibyte_at_the_bound() {
    let suite = suite();
    let long = sent(&suite, &scenario("too-long"));
    let label = text(&long, "label").expect("a label");
    assert_eq!(label.len(), 9, "one byte over the bound: {label:?}");
    assert!(
        label.chars().count() <= 8,
        "fewer scalars than bytes: {label:?}"
    );
    let wide = sent(&suite, &scenario("wide-code"));
    let code = text(&wide, "code").expect("a code");
    assert_eq!(code.len(), 4, "at the bound: {code:?}");
    assert_ne!(
        code.chars().count(),
        4,
        "fewer scalars than bytes: {code:?}"
    );
}

fn caught(mode: Mode, deciding: &[&str]) {
    let suite = suite();
    let healthy = run(&suite, &Notes::new(Mode::Correct));
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    let failing = not_passed(&run(&suite, &Notes::new(mode)));
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
fn utf8_a_target_counting_scalars_fails() {
    caught(Mode::Scalars, &["too-long", "wide-code"]);
}

#[test]
fn utf8_a_target_counting_utf16_units_fails() {
    caught(Mode::Utf16Units, &["too-long", "wide-code"]);
}

#[test]
fn utf8_a_target_counting_graphemes_fails() {
    caught(Mode::Graphemes, &["too-long", "wide-code"]);
}

/// The outcome the native interpreter takes for one request, or why it took none.
fn interpreted(label: &str, code: &str) -> String {
    let target = Interpreted::for_model(ir());
    let correlation = ess_primitives::ids::CorrelationId::new("utf8-direct").expect("an id");
    let id = ess_conformance::ScenarioId::parse(&scenario("filed")).expect("a scenario id");
    target
        .begin_scenario(&ScenarioContext::new(id, correlation.clone()))
        .expect("begins");
    match target.execute_command(SemanticCommandRequest {
        command: ess_compiler::refs::CommandRef::new("notes.core.File".parse().unwrap()),
        actor: None,
        caller: None,
        input: [
            ("label".to_owned(), Node::Text(label.to_owned())),
            ("code".to_owned(), Node::Text(code.to_owned())),
        ]
        .into(),
        correlation,
    }) {
        Ok(result) => result
            .outcome
            .map(|outcome| {
                outcome
                    .to_string()
                    .rsplit('/')
                    .next()
                    .unwrap_or_default()
                    .to_owned()
            })
            .unwrap_or_default(),
        Err(error) => format!("{error:?}"),
    }
}

#[test]
fn utf8_the_interpreter_decides_at_and_beside_the_byte_bound() {
    for (label, code, outcome) in [
        ("", "c", "filed"),
        ("abcdefgh", "c", "filed"),
        ("abcdefghi", "c", "too-long"),
        ("éééé", "c", "filed"),
        ("éééé!", "c", "too-long"),
        ("😀😀", "c", "filed"),
        ("😀😀a", "c", "too-long"),
        ("cafe\u{301}abc", "c", "too-long"),
        ("caf\u{e9}abc", "c", "filed"),
        ("ab", "😀", "wide-code"),
        ("ab", "é!!", "wide-code"),
        ("ab", "€", "filed"),
        ("ab", "€!", "wide-code"),
        ("ab", "abc", "filed"),
    ] {
        assert_eq!(
            interpreted(label, code),
            outcome,
            "{label:?} ({} bytes), {code:?} ({} bytes)",
            label.len(),
            code.len()
        );
    }
}

// ---- the persisted vocabulary: suite /40 and /41 ---------------------------------------------------

#[test]
fn utf8_the_invariant_selects_suite40_and_old_relabels_are_refused() {
    let suite = suite();
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/40"
    );
    let json = suite.to_canonical_json().expect("admitted");
    assert!(json.contains(r#""utf8_bytes": "label""#), "{json}");
    AdmittedSuite::from_json(&json).expect("a /40 reader admits it");
    // `/38` is an older ordinary major whose envelope a `/40` suite otherwise satisfies.
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
    assert!(selected.original_json().contains(r#""utf8_bytes""#));
}

#[test]
fn utf8_the_rust_runner_fails_the_broken_invariant() {
    let failing = not_passed(&run(&suite(), &Notes::new(Mode::BrokenRows)));
    assert!(
        failing.iter().any(|line| line.contains("/invariant/")),
        "{failing:#?}"
    );
}

#[test]
fn utf8_go_runs_the_byte_length_suite_at_parity_with_the_rust_runner() {
    let suite = suite();
    let healthy = support_go::assert_parity("utf8-healthy", &suite, Notes::new(Mode::Correct));
    assert_eq!(support_go::not_passed(&healthy), Vec::<&str>::new());
    let broken = support_go::assert_parity("utf8-broken", &suite, Notes::new(Mode::BrokenRows));
    assert!(
        support_go::not_passed(&broken)
            .iter()
            .any(|id| id.contains("/invariant/")),
        "{broken:#?}"
    );
    let scalars = support_go::assert_parity("utf8-scalars", &suite, Notes::new(Mode::Scalars));
    assert!(
        support_go::not_passed(&scalars)
            .iter()
            .any(|id| id.contains("too-long") || id.contains("wide-code")),
        "{scalars:#?}"
    );
}

/// A JavaScript implementation of the notes: decides each guard by the UTF-8 byte length, or wrong
/// in the one way its mode says, and publishes rows whose label breaks the invariant when broken.
const JS_TARGET: &str = r"const mode = process.env.ESS_TARGET_MODE ?? 'healthy';
const measure = (text) => {
  if (mode === 'utf16') return text.length;
  if (mode === 'codepoints') return [...text].length;
  return new TextEncoder().encode(text).length;
};
class Notes {
  rows = [];
  seq = 0;
  identity() { return { name: 'notes', version: '1' }; }
  beginScenario() { this.rows = []; }
  endScenario() {}
  executeCommand({ command, input }) {
    if (command !== 'notes.core.File') throw new Error(`unexpected ${command}`);
    if (measure(input.label) > 8) {
      return { outcome: 'too-long', consistency: `seq:${this.seq}`, error: 'notes.core.TooLong' };
    }
    if (measure(input.code) === 4) {
      return { outcome: 'wide-code', consistency: `seq:${this.seq}`, error: 'notes.core.WideCode' };
    }
    this.seq += 1;
    const id = `00000000-0000-4000-8000-${String(this.seq).padStart(12, '0')}`;
    this.rows.push({ note_id: id, label: input.label, code: input.code });
    return {
      outcome: 'filed',
      consistency: `seq:${this.seq}`,
      directEvents: [{ event: 'notes.core.Filed', payload: { note_id: id } }],
    };
  }
  queryView() {
    return {
      rows: this.rows.map((row) =>
        mode === 'broken' ? { ...row, label: 'é'.repeat(33) } : { ...row },
      ),
    };
  }
  observeEvents() { throw new Error('unused'); }
  configureExternalOutcome() { throw new Error('nothing is external'); }
  redeliverEvent() { throw new Error('no bindings'); }
}
export function makeTarget() { return new Notes(); }
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
        .join("expression-utf8-bytes")
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
fn utf8_typescript_runs_the_byte_length_suite_and_every_fault_fails() {
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
    for mode in ["utf16", "codepoints"] {
        let faulty = typescript_verdicts(mode).expect("tools were found once");
        assert!(
            faulty.iter().any(
                |(id, status)| (id.contains("too-long") || id.contains("wide-code"))
                    && status == "failed"
            ),
            "{mode}: {faulty:#?}"
        );
    }
}

// ---- the shared vectors in the generated Go reader --------------------------------------------

#[test]
fn utf8_the_generated_go_reader_answers_the_shared_vectors() {
    let document = serde_json::json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "notes",
            "specification_version": "v1", "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"notes.core/authored/bytes": {"purpose": "Measure a label",
            "steps": [{"step": "expect_view", "view": "notes.core.Notes",
                "expectation": {"expect": "satisfies", "predicate": "label != code"}}],
            "source": []}}
    })
    .to_string();
    let suite = AdmittedSuite::from_json(&document).expect("admitted");
    let directory = std::env::temp_dir().join(format!("ess-utf8-bytes-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("essconform")).expect("directory");
    for artifact in ess_conformance::go::emit(suite.suite()).expect("emitted") {
        std::fs::write(directory.join(artifact.path), artifact.contents).expect("written");
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/utf8\n\ngo 1.24\n",
    )
    .expect("go.mod");
    std::fs::write(
        directory.join("essconform/utf8_bytes_test.go"),
        include_str!("fixtures/utf8-bytes.go"),
    )
    .expect("fixture");
    std::fs::write(
        directory.join("essconform/utf8-bytes.json"),
        include_str!("../../../specify/ess-primitives/tests/vectors/utf8-bytes.json"),
    )
    .expect("vectors");
    let result = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestUtf8Bytes",
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
            && printed.contains("--- PASS: TestUtf8Bytes ")
            && printed.contains("--- PASS: TestUtf8BytesFaults"),
        "Go byte-length reader: {}",
        result.status
    );
}

// ---- nothing else moves ---------------------------------------------------------------------------

#[test]
fn utf8_a_model_without_the_selector_keeps_its_suite_format() {
    let mut plain = MODEL.to_owned();
    for (derived, literal) in [
        ("label.utf8_bytes <= 64", "label != \"\""),
        ("when: label.utf8_bytes > 8", "when: label == \"x\""),
        ("when: code.utf8_bytes == 4", "when: code == \"y\""),
    ] {
        assert!(plain.contains(derived), "the model writes `{derived}`");
        plain = plain.replace(derived, literal);
    }
    let model = ir_of(&plain);
    assert!(!model.to_canonical_json().contains("utf8_bytes"), "{plain}");
    let suite = ess_conformance::synthesize::synthesize(&model).suite;
    assert!(!ess_conformance::expression_format::used_by(&suite));
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/34"
    );
}

#[test]
fn utf8_a_member_named_utf8_bytes_selects_no_new_format() {
    let text = MODEL
        .replace(
            "entities:\n",
            "types:\n  - name: notes.core.Sizes\n    kind: struct\n    fields:\n      - {name: utf8_bytes, type: Integer}\nentities:\n",
        )
        .replace(
            "      - {name: code, type: String}\n    invariants:",
            "      - {name: code, type: String}\n      - {name: sizes, type: notes.core.Sizes}\n    invariants:",
        )
        .replace("label.utf8_bytes <= 64", "sizes.utf8_bytes <= 64")
        .replace("when: label.utf8_bytes > 8", "when: label == \"x\"")
        .replace("when: code.utf8_bytes == 4", "when: code == \"y\"")
        .replace(
            "      - {name: code, type: String}\n    outcomes:",
            "      - {name: code, type: String}\n      - {name: sizes, type: notes.core.Sizes}\n    outcomes:",
        )
        .replace("sets: {label: input.label, code: input.code}", "sets: {label: input.label, code: input.code, sizes: input.sizes}");
    let model = ir_of(&text);
    let suite = ess_conformance::synthesize::synthesize(&model).suite;
    assert!(
        !ess_conformance::expression_format::used_by(&suite),
        "{text}"
    );
    assert_ne!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/40"
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
fn utf8_the_browser_product_admits_and_runs_the_byte_length_suite() {
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
        passed.contains("/invariant/") && passed.contains("too-long"),
        "{passed}"
    );
}

// ---- a text's `.count` beside a byte length in `satisfies` (decision 1) ----------------------------

/// The interpreter over `model`, publishing every row's label as `label` when one is given.
struct Rows {
    inner: Interpreted,
    label: Option<String>,
}

impl ConformanceTarget for Rows {
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
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        if let Some(label) = &self.label {
            for row in &mut result.rows {
                row.insert("label".to_owned(), Node::Text(label.clone()));
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

#[test]
fn utf8_a_text_count_beside_a_byte_length_is_asserted_on_view_rows() {
    let text = MODEL.replace(
        "      - label.utf8_bytes <= 64\n",
        "      - {all: [label.count <= 40, label.utf8_bytes <= 64]}\n",
    );
    let model = ir_of(&text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let suite = synthesis.suite;
    assert_eq!(
        suite.provenance.suite_version.to_string(),
        "ess-conformance/40"
    );
    let json = suite.to_canonical_json().expect("admitted");
    assert!(json.contains("label.count"), "{json}");
    let target = |label: Option<&str>| Rows {
        inner: Interpreted::for_model(ir_of(&text)),
        label: label.map(str::to_owned),
    };
    assert_eq!(
        not_passed(&run(&suite, &target(None))),
        Vec::<String>::new()
    );
    // 41 ASCII bytes: within the byte bound, one past the count bound.
    let failing = not_passed(&run(&suite, &target(Some(&"a".repeat(41)))));
    assert!(
        failing.iter().any(|line| line.contains("/invariant/")),
        "{failing:#?}"
    );
    // 21 four-byte scalars: within the count bound, past the byte bound.
    let failing = not_passed(&run(&suite, &target(Some(&"😀".repeat(21)))));
    assert!(
        failing.iter().any(|line| line.contains("/invariant/")),
        "{failing:#?}"
    );
    let parity =
        support_go::assert_parity("utf8-count-broken", &suite, target(Some(&"a".repeat(41))));
    assert!(
        support_go::not_passed(&parity)
            .iter()
            .any(|id| id.contains("/invariant/")),
        "{parity:#?}"
    );
}

#[test]
fn utf8_a_text_count_alone_stays_out_of_satisfies() {
    let text = MODEL
        .replace(
            "      - label.utf8_bytes <= 64\n",
            "      - label.count <= 40\n",
        )
        .replace("when: label.utf8_bytes > 8", "when: label == \"x\"")
        .replace("when: code.utf8_bytes == 4", "when: code == \"y\"");
    let synthesis = ess_conformance::synthesize::synthesize(&ir_of(&text));
    assert!(
        synthesis
            .refusals
            .iter()
            .any(|refusal| format!("{refusal:?}").contains("InvariantUnobservable")),
        "{:#?}",
        synthesis.refusals
    );
    assert_ne!(
        synthesis.suite.provenance.suite_version.to_string(),
        "ess-conformance/40"
    );
}
