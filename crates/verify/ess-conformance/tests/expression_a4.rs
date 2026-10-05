//! Family F part A4 executed (beyond10x/ess#233, `docs/design/expression-family-source22.md`):
//! values copied from members of struct inputs into stored fields, an event payload and an error
//! payload, with an input fallback after `else:`.
//!
//! The suite is synthesized from the model — every path sent present, then the Optional parent
//! left out so the fallback and the absent value are asserted — and run against the native
//! interpreter, which must pass it, and against targets wrong in one way each, which must each fail
//! the scenario that decides the value: one reading a sibling of the same type, one reading the
//! same-named top-level input, one reading the struct's first member, one reading the parent
//! object, one reading the root object, one unwrapping an absent Optional parent, one ignoring the
//! fallback and one storing the fallback's spelling. Rule 8's existence controls are repeated with
//! the creation identity read through a nested required path.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

mod support_go;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/input-value-paths.yaml");
const EXISTENCE: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/upsert-by-existence.yaml");

const OPENED: &str = "leases.pool.Open/outcome/opened";
const REJECTED: &str = "leases.pool.Open/outcome/rejected";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn ir() -> EssIr {
    compiled(MODEL)
}

fn suite_of(ir: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(ir);
    assert_eq!(
        synthesis.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

fn suite() -> ConformanceSuite {
    suite_of(&ir())
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            if result.status != Status::Passed {
                for diagnostic in result.diagnostics() {
                    eprintln!("{}: {diagnostic:?}", result.scenario);
                }
            }
            (result.scenario.to_string(), result.status)
        })
        .collect()
}

fn not_passed(statuses: &BTreeMap<String, Status>) -> Vec<String> {
    statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect()
}

/// Every input one scenario sends, in order.
fn sent(suite: &ConformanceSuite, id: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(|| panic!("no scenario {id}"), |(_, scenario)| scenario);
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "leases.pool.Open" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .collect()
}

fn member<'a>(input: &'a BTreeMap<String, ScenarioValue>, path: &[&str]) -> Option<&'a Node> {
    let (root, rest) = path.split_first()?;
    let ScenarioValue::Literal { value } = input.get(*root)? else {
        return None;
    };
    let mut value = value;
    for segment in rest {
        value = value.as_map()?.get(*segment)?;
    }
    Some(value)
}

// ---- synthesis -----------------------------------------------------------------------------------

#[test]
fn a4_synthesis_sends_each_path_present_and_its_optional_parent_absent() {
    let suite = suite();
    assert!(
        !ess_conformance::expression_format::used_by(&suite),
        "A4 persists no new predicate vocabulary"
    );
    for id in [OPENED, REJECTED] {
        let inputs = sent(&suite, id);
        let present = inputs
            .iter()
            .find(|input| member(input, &["previous", "label"]).is_some())
            .unwrap_or_else(|| panic!("{id}: one invocation sends `previous`: {inputs:#?}"));
        let absent = inputs
            .iter()
            .find(|input| !input.contains_key("previous"))
            .unwrap_or_else(|| panic!("{id}: one invocation leaves `previous` out: {inputs:#?}"));
        assert!(
            member(absent, &["settings", "defaults", "label"]).is_some(),
            "{id}: the fallback is sent where the primary is absent"
        );
        // Two siblings of one type and the same-named decoy are sent apart, so a target reading the
        // wrong one is told apart.
        let opening = member(present, &["opening", "generation_id"]);
        assert!(opening.is_some());
        assert_ne!(
            opening,
            member(present, &["previous", "generation_id"]),
            "{id}"
        );
        assert_ne!(opening, member(present, &["generation_id"]), "{id}");
        assert_ne!(
            member(present, &["previous", "label"]),
            member(present, &["settings", "defaults", "label"]),
            "{id}"
        );
    }
}

#[test]
fn a4_the_native_interpreter_passes_every_scenario() {
    let suite = suite();
    let statuses = run(&suite, &Interpreted::for_model(ir()));
    assert!(
        statuses.contains_key(OPENED) && statuses.contains_key(REJECTED),
        "{statuses:#?}"
    );
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
}

// ---- the interpreter, directly ------------------------------------------------------------------

fn text(value: &str) -> Node {
    Node::Text(value.to_owned())
}

fn map(entries: &[(&str, Node)]) -> Node {
    Node::Map(
        entries
            .iter()
            .map(|(key, value)| ((*key).to_owned(), value.clone()))
            .collect(),
    )
}

fn request(previous: bool) -> BTreeMap<String, Node> {
    let mut input = BTreeMap::from([
        (
            "lease_id".to_owned(),
            text("00000000-0000-4000-8000-000000000001"),
        ),
        ("generation_id".to_owned(), text("decoy")),
        (
            "opening".to_owned(),
            map(&[("generation_id", text("gen-open")), ("label", text("open"))]),
        ),
        (
            "sealed".to_owned(),
            map(&[
                ("generation_id", text("gen-sealed")),
                ("label", text("sealed")),
            ]),
        ),
        (
            "settings".to_owned(),
            map(&[("defaults", map(&[("label", text("default"))]))]),
        ),
        ("reject".to_owned(), Node::Bool(false)),
    ]);
    if previous {
        input.insert(
            "previous".to_owned(),
            map(&[("generation_id", text("gen-prev")), ("label", text("prev"))]),
        );
    }
    input
}

fn opened_event(input: BTreeMap<String, Node>) -> BTreeMap<String, Node> {
    let target = Interpreted::for_model(ir());
    let correlation = ess_primitives::ids::CorrelationId::new("a4-direct").expect("an id");
    let scenario = ess_conformance::ScenarioId::parse(OPENED).expect("a scenario id");
    target
        .begin_scenario(&ScenarioContext::new(scenario, correlation.clone()))
        .expect("begins");
    let result = target
        .execute_command(SemanticCommandRequest {
            command: ess_compiler::refs::CommandRef::new("leases.pool.Open".parse().unwrap()),
            actor: None,
            caller: None,
            input,
            correlation,
        })
        .unwrap_or_else(|error| panic!("{error:?}"));
    assert_eq!(
        result.outcome.map(|outcome| outcome.to_string()).as_deref(),
        Some("leases.pool.Open/opened")
    );
    result
        .direct_events
        .into_iter()
        .next()
        .expect("the opened event")
        .payload
}

#[test]
fn a4_interpreter_reads_each_path_structurally() {
    let payload = opened_event(request(true));
    assert_eq!(payload.get("generation_id"), Some(&text("gen-open")));
    assert_eq!(payload.get("previous"), Some(&text("gen-prev")));
    assert_eq!(payload.get("label"), Some(&text("prev")));
}

#[test]
fn a4_interpreter_an_absent_parent_is_absence_and_selects_the_fallback() {
    let payload = opened_event(request(false));
    assert_eq!(payload.get("generation_id"), Some(&text("gen-open")));
    assert!(
        payload
            .get("previous")
            .is_none_or(|value| *value == Node::Null),
        "an absent parent leaves the value absent: {payload:?}"
    );
    assert_eq!(payload.get("label"), Some(&text("default")));
}

// ---- faulty targets -------------------------------------------------------------------------------

/// How a target reads the paths.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Correct,
    /// `previous.generation_id` read where `opening.generation_id` is written.
    Sibling,
    /// The same-named top-level `generation_id` read instead of `opening.generation_id`.
    Decoy,
    /// `sealed.generation_id`, the struct's first member, read instead of `sealed.label`.
    FirstChild,
    /// The parent object `opening` published where its member is written.
    Parent,
    /// The root object `settings` published where the fallback `settings.defaults.label` is.
    Root,
    /// An absent `previous` unwrapped into empty members instead of left absent.
    UnwrapAbsent,
    /// The fallback ignored: some other value stored where the primary is absent.
    IgnoreFallback,
    /// The fallback's spelling stored instead of its value.
    LiteralFallback,
}

/// The interpreter, with the paths read as `mode` says instead of as the model says: the request is
/// moved so that the interpreter copies what the faulty reading would, or the answer is rewritten
/// where the faulty value has another shape.
struct Leases {
    inner: Interpreted,
    mode: Mode,
    last: RefCell<BTreeMap<String, Node>>,
}

impl Leases {
    fn new(mode: Mode) -> Self {
        Self::of(ir(), mode)
    }

    fn of(ir: EssIr, mode: Mode) -> Self {
        Self {
            inner: Interpreted::for_model(ir),
            mode,
            last: RefCell::default(),
        }
    }

    fn steer(&self, mut request: SemanticCommandRequest) -> SemanticCommandRequest {
        if request.command.to_string() != "leases.pool.Open" {
            return request;
        }
        self.last.replace(request.input.clone());
        let input = &mut request.input;
        let at = |input: &BTreeMap<String, Node>, root: &str, member: &str| {
            input
                .get(root)
                .and_then(Node::as_map)
                .and_then(|members| members.get(member))
                .cloned()
        };
        let put = |input: &mut BTreeMap<String, Node>, root: &str, member: &str, value: Node| {
            if let Some(Node::Map(members)) = input.get_mut(root) {
                members.insert(member.to_owned(), value);
            }
        };
        let absent = !input.contains_key("previous");
        match self.mode {
            Mode::Sibling => {
                if let Some(value) = at(input, "previous", "generation_id") {
                    put(input, "opening", "generation_id", value);
                }
            }
            Mode::Decoy => {
                if let Some(value) = input.get("generation_id").cloned() {
                    put(input, "opening", "generation_id", value);
                }
            }
            Mode::FirstChild => {
                if let Some(value) = at(input, "sealed", "generation_id") {
                    put(input, "sealed", "label", value);
                }
            }
            Mode::UnwrapAbsent if absent => {
                input.insert(
                    "previous".to_owned(),
                    map(&[("generation_id", text("")), ("label", text(""))]),
                );
            }
            Mode::IgnoreFallback if absent => {
                if let Some(Node::Map(settings)) = input.get_mut("settings") {
                    settings.insert(
                        "defaults".to_owned(),
                        map(&[("label", text("implementation-chosen"))]),
                    );
                }
            }
            Mode::LiteralFallback if absent => {
                if let Some(Node::Map(settings)) = input.get_mut("settings") {
                    settings.insert(
                        "defaults".to_owned(),
                        map(&[("label", text("input.settings.defaults.label"))]),
                    );
                }
            }
            Mode::Correct
            | Mode::Parent
            | Mode::Root
            | Mode::UnwrapAbsent
            | Mode::IgnoreFallback
            | Mode::LiteralFallback => {}
        }
        request
    }

    /// The answer as a target reading a whole object where a member is written would publish it.
    fn rewrite(&self, mut result: SemanticCommandResult) -> SemanticCommandResult {
        let last = self.last.borrow();
        let (field, value) = match self.mode {
            Mode::Parent => ("generation_id", last.get("opening").cloned()),
            Mode::Root if !last.contains_key("previous") => {
                ("label", last.get("settings").cloned())
            }
            _ => return result,
        };
        let Some(value) = value else {
            return result;
        };
        for event in &mut result.direct_events {
            if event.payload.contains_key(field) {
                event.payload.insert(field.to_owned(), value.clone());
            }
        }
        if let Some(error) = &mut result.error {
            if error.fields.contains_key(field) {
                error.fields.insert(field.to_owned(), value);
            }
        }
        result
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
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let result = self.inner.execute_command(self.steer(request))?;
        Ok(self.rewrite(result))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
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

fn caught(mode: Mode) {
    let suite = suite();
    let healthy = run(&suite, &Leases::new(Mode::Correct));
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    let faulty = run(&suite, &Leases::new(mode));
    let failing = not_passed(&faulty);
    assert!(
        failing
            .iter()
            .any(|line| line.starts_with(OPENED) || line.starts_with(REJECTED)),
        "{mode:?} fails a scenario that decides a path: {failing:#?}"
    );
    assert!(
        failing.iter().all(|line| line.contains("Failed")),
        "{mode:?} fails by outcome, event or row, not by setup or admission: {failing:#?}"
    );
}

#[test]
fn a4_fault_sibling_of_the_same_type() {
    caught(Mode::Sibling);
}

#[test]
fn a4_fault_same_named_top_level_decoy() {
    caught(Mode::Decoy);
}

#[test]
fn a4_fault_first_child() {
    caught(Mode::FirstChild);
}

#[test]
fn a4_fault_reads_the_parent() {
    caught(Mode::Parent);
}

#[test]
fn a4_fault_reads_the_root() {
    caught(Mode::Root);
}

#[test]
fn a4_fault_unwraps_an_absent_optional() {
    caught(Mode::UnwrapAbsent);
}

#[test]
fn a4_fault_ignores_the_fallback() {
    caught(Mode::IgnoreFallback);
}

#[test]
fn a4_fault_stores_the_fallback_spelling() {
    caught(Mode::LiteralFallback);
}

// ---- the generated Go suite runner ---------------------------------------------------------------

#[test]
fn a4_the_go_runner_agrees_with_the_rust_runner_healthy_and_faulty() {
    let suite = suite();
    let healthy = support_go::assert_parity("a4-healthy", &suite, Leases::new(Mode::Correct));
    assert_eq!(support_go::not_passed(&healthy), Vec::<&str>::new());
    // Wrong on every request, refusals included: the generated Go runner compares a refusal's
    // error payload as the Rust runner does.
    for (label, mode) in [
        ("a4-decoy", Mode::Decoy),
        ("a4-ignored", Mode::IgnoreFallback),
        ("a4-sibling", Mode::Sibling),
        ("a4-first-child", Mode::FirstChild),
        ("a4-unwrap", Mode::UnwrapAbsent),
        ("a4-fallback", Mode::LiteralFallback),
        ("a4-parent", Mode::Parent),
        ("a4-root", Mode::Root),
    ] {
        let faulty = support_go::assert_parity(label, &suite, Leases::new(mode));
        let failing = support_go::not_passed(&faulty);
        assert!(failing.contains(&OPENED), "{mode:?}: {faulty:#?}");
        if matches!(mode, Mode::Sibling | Mode::Decoy | Mode::Parent) {
            assert!(
                failing.contains(&REJECTED),
                "{mode:?}: the wrong error payload fails the refusal: {faulty:#?}"
            );
        }
    }
}

// ---- the generated TypeScript suite runner -------------------------------------------------------

/// A JavaScript implementation of `leases.pool.Open`, reading each path as `ESS_TARGET_MODE` says:
/// right, or wrong in one way each — the refusal's error payload included.
const JS_TARGET: &str = r"const mode = process.env.ESS_TARGET_MODE ?? 'healthy';
class Leases {
  rows = [];
  identity() { return { name: 'leases', version: '1' }; }
  beginScenario() { this.rows = []; }
  endScenario() {}
  executeCommand({ command, input }) {
    if (command !== 'leases.pool.Open') throw new Error(`unexpected ${command}`);
    const previous = input.previous ?? null;
    let generation = input.opening.generation_id;
    let sealed = input.sealed.label;
    let label = previous !== null ? previous.label : input.settings.defaults.label;
    let previousGeneration = previous !== null ? previous.generation_id : null;
    if (mode === 'sibling' && previous !== null) generation = previous.generation_id;
    if (mode === 'decoy') generation = input.generation_id;
    if (mode === 'first-child') sealed = input.sealed.generation_id;
    if (mode === 'unwrap' && previous === null) { previousGeneration = ''; label = ''; }
    if (mode === 'fallback-spelling' && previous === null) label = 'input.settings.defaults.label';
    if (input.reject === true) {
      return {
        outcome: 'rejected',
        error: 'leases.pool.Rejected',
        errorPayload: { generation_id: generation, label },
        directEvents: [],
      };
    }
    this.rows.push({
      lease_id: input.lease_id,
      generation_id: generation,
      previous_generation: previousGeneration,
      label,
      sealed_label: sealed,
      copied: { label: input.opening.label },
    });
    return {
      outcome: 'opened',
      consistency: `seq:${this.rows.length}`,
      directEvents: [{
        event: 'leases.pool.Opened',
        payload: { lease_id: input.lease_id, generation_id: generation, previous: previousGeneration, label },
      }],
    };
  }
  queryView() { return { rows: this.rows.map((row) => ({ ...row })) }; }
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
    let suite = suite();
    let admitted = AdmittedSuite::from_suite(&suite).expect("admits");
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("expression-a4")
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
    if verdicts.values().any(|status| status != "passed") {
        eprintln!("{mode}:\n{printed}");
    }
    let _ = std::fs::remove_dir_all(&root);
    Some(verdicts)
}

#[test]
fn a4_the_typescript_runner_passes_a_right_target_and_fails_each_wrong_one() {
    let Some(healthy) = typescript_verdicts("healthy") else {
        return;
    };
    assert!(
        healthy.contains_key(OPENED) && healthy.contains_key(REJECTED),
        "{healthy:#?}"
    );
    let failing: Vec<&String> = healthy
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id)
        .collect();
    assert_eq!(failing, Vec::<&String>::new(), "{healthy:#?}");
    for mode in [
        "sibling",
        "decoy",
        "first-child",
        "unwrap",
        "fallback-spelling",
    ] {
        let faulty = typescript_verdicts(mode).expect("tools were found once");
        assert_eq!(
            faulty.get(OPENED).map(String::as_str),
            Some("failed"),
            "{mode}: {faulty:#?}"
        );
        // The refusal carries the same paths in its error payload; only the first member is
        // read for a stored field alone.
        let refusal = if mode == "first-child" {
            "passed"
        } else {
            "failed"
        };
        assert_eq!(
            faulty.get(REJECTED).map(String::as_str),
            Some(refusal),
            "{mode}: {faulty:#?}"
        );
    }
}

// ---- the further fallback runs of a creating branch with a view ---------------------------------

/// Two literal fallbacks on a creating branch whose rows a view shows: `nick` falls back to
/// `Fixed`, `other` to `Other`. `FORMAT` is replaced.
const TWO_FALLBACKS: &str = r"format: FORMAT
system: probe
version: v1
domain: probe.pair

types:
  - {name: probe.pair.NoteId, kind: newtype, of: Uuid}

entities:
  - name: probe.pair.Note
    identity: {name: note_id, type: probe.pair.NoteId}
    fields:
      - {name: label, type: String}
      - {name: copied, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}

events:
  - name: probe.pair.Opened
    fields:
      - {name: note_id, type: probe.pair.NoteId}
      - {name: label, type: String}

commands:
  - name: probe.pair.Open
    input:
      - {name: note_id, type: probe.pair.NoteId}
      - {name: nick, type: Optional<String>}
      - {name: other, type: Optional<String>}
    outcomes:
      - name: opened
        creates: probe.pair.Note
        instance: note_id
        emits: [probe.pair.Opened]
        payload:
          probe.pair.Opened:
            note_id: input.note_id
            label: {input: nick, else: Fixed}
        sets:
          label: {input: nick, else: Fixed}
          copied: {input: other, else: Other}

views:
  - name: probe.pair.NoteDetails
    source: probe.pair.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: probe.pair.NoteId}
      - {name: label, type: String}
      - {name: copied, type: String}
";

/// The interpreter, choosing `nick`'s fallback by `other`'s absence where `wrong` says so.
struct OtherAbsence(Interpreted, bool);

impl ConformanceTarget for OtherAbsence {
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.0.establish_entity(request)
    }
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn execute_command(
        &self,
        mut request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if self.1 && request.command.to_string() == "probe.pair.Open" {
            let other = request
                .input
                .get("other")
                .is_some_and(|value| *value != Node::Null);
            if other {
                request
                    .input
                    .entry("nick".to_owned())
                    .or_insert_with(|| Node::Text(String::new()));
            } else {
                request.input.remove("nick");
            }
        }
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.0.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(r)
    }
}

/// A run that leaves fallbacks out creates a row of its own, and the view is asked for that row by
/// the instance the run captured, not by the first event the scenario saw: the interpreter passes
/// its own suite in every format (the ess/16 omission run once asked for the first run's identity).
#[test]
fn a4_a_fallback_run_expects_its_own_row() {
    for format in ["ess/16", "ess/22"] {
        let ir = compiled(&TWO_FALLBACKS.replace("FORMAT", format));
        let suite = suite_of(&ir);
        let statuses = run(&suite, &OtherAbsence(Interpreted::for_model(ir), false));
        assert_eq!(not_passed(&statuses), Vec::<String>::new(), "{format}");
    }
}

/// From ess/22 each Optional a fallback hangs on is left out in a run of its own, so a target that
/// chooses one fallback by another input's absence fails; below it the one run keeps leaving them
/// out together, and such a target is not told apart there.
#[test]
fn a4_from_ess22_each_fallback_is_left_out_alone() {
    let source22 = compiled(&TWO_FALLBACKS.replace("FORMAT", "ess/22"));
    let suite = suite_of(&source22);
    let faulty = run(
        &suite,
        &OtherAbsence(Interpreted::for_model(source22), true),
    );
    assert_ne!(not_passed(&faulty).len(), 0, "{faulty:#?}");
    let source16 = compiled(&TWO_FALLBACKS.replace("FORMAT", "ess/16"));
    let suite = suite_of(&source16);
    let faulty = run(
        &suite,
        &OtherAbsence(Interpreted::for_model(source16), true),
    );
    assert_eq!(
        not_passed(&faulty),
        Vec::<String>::new(),
        "below ess/22 the omission run keeps its bytes"
    );
}

// ---- mutation ------------------------------------------------------------------------------------

/// A `sets-retarget` mutant of a path writes the same-named top-level input instead; the suite the
/// mutated model synthesizes fails the right implementation, so the mutant is killed.
#[test]
fn a4_mutation_retargets_a_path_to_its_top_level_decoy_and_kills_it() {
    use ess_conformance::mutate::{self, MutantClass, Verdict};
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), MODEL.to_owned());
    let files = vec![(Source::new("model.yaml"), raw)];
    let retargets: Vec<_> = mutate::mutants(&files, &[MutantClass::SetsRetarget])
        .into_iter()
        .filter(|mutant| mutant.id.contains("generation_id"))
        .collect();
    assert_eq!(
        retargets.len(),
        1,
        "one path names a declared top-level input: {retargets:#?}"
    );
    let ir = mutate::compile(files.clone(), &texts).expect("the model compiles");
    let report = mutate::audit(&files, &texts, &[MutantClass::SetsRetarget], || {
        Interpreted::for_model(ir.clone())
    })
    .unwrap_or_else(|refusal| panic!("{refusal}"));
    let verdicts: Vec<(String, Verdict)> = report
        .mutants
        .iter()
        .map(|entry| (entry.id.clone(), entry.verdict))
        .collect();
    assert!(
        verdicts
            .iter()
            .any(|(id, verdict)| id.contains("generation_id") && *verdict == Verdict::Killed),
        "{verdicts:#?}"
    );
}

// ---- rule 8: existence reads a nested identity structurally -------------------------------------

fn nested_existence() -> String {
    let model = EXISTENCE
        .replace("format: ess/16\n", "format: ess/22\n")
        .replace(
            "  - {name: demo.items.Label, kind: newtype, of: String}\n",
            "  - {name: demo.items.Label, kind: newtype, of: String}
  - name: demo.items.Booking
    kind: struct
    fields:
      - {name: slot_id, type: demo.items.ItemId}\n",
        )
        .replace(
            "    input:\n      - {name: slot_id, type: demo.items.ItemId}\n      - {name: label, type: demo.items.Label}\n    outcomes:\n      - name: booked",
            "    input:\n      - {name: slot_id, type: demo.items.ItemId}\n      - {name: booking, type: demo.items.Booking}\n      - {name: label, type: demo.items.Label}\n    outcomes:\n      - name: booked",
        )
        .replace(
            "demo.items.SlotBooked: {slot_id: input.slot_id, label: input.label}",
            "demo.items.SlotBooked: {slot_id: input.booking.slot_id, label: input.label}",
        );
    assert_ne!(model, EXISTENCE);
    model
}

/// The duplicate-booking refusal, decided by the identity a nested path names.
#[test]
fn a4_existence_reads_a_nested_identity() {
    let ir = compiled(&nested_existence());
    let suite = suite_of(&ir);
    let statuses = run(&suite, &Interpreted::for_model(ir));
    assert!(
        statuses
            .keys()
            .any(|id| id.starts_with("demo.items.BookSlot/") && id.contains("already-booked")),
        "the refusal is witnessed: {statuses:#?}"
    );
    assert_eq!(not_passed(&statuses), Vec::<String>::new());
}

/// A target that reads the top-level decoy `slot_id` as the identity — the same-named input beside
/// the path — books twice and fails the refusal.
#[test]
fn a4_existence_fault_reads_the_top_level_identity() {
    struct Decoy(Interpreted);
    impl ConformanceTarget for Decoy {
        fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
            self.0.establish_entity(request)
        }
        fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
            self.0.identity()
        }
        fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
            self.0.begin_scenario(scenario)
        }
        fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
            self.0.end_scenario(scenario)
        }
        fn execute_command(
            &self,
            mut request: SemanticCommandRequest,
        ) -> Result<SemanticCommandResult, TargetError> {
            if request.command.to_string() == "demo.items.BookSlot" {
                if let Some(decoy) = request.input.get("slot_id").cloned() {
                    if let Some(Node::Map(booking)) = request.input.get_mut("booking") {
                        booking.insert("slot_id".to_owned(), decoy);
                    }
                }
            }
            self.0.execute_command(request)
        }
        fn query_view(
            &self,
            request: SemanticViewRequest,
        ) -> Result<SemanticViewResult, TargetError> {
            self.0.query_view(request)
        }
        fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
            self.0.configure_external_outcome(r)
        }
        fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
            self.0.redeliver_event(r)
        }
        fn observe_events(
            &self,
            r: EventObservationRequest,
        ) -> Result<Vec<ObservedEvent>, TargetError> {
            self.0.observe_events(r)
        }
    }
    let ir = compiled(&nested_existence());
    let suite = suite_of(&ir);
    let statuses = run(&suite, &Decoy(Interpreted::for_model(ir)));
    let failing = not_passed(&statuses);
    assert!(
        failing
            .iter()
            .any(|line| line.starts_with("demo.items.BookSlot/") && line.contains("Failed")),
        "{failing:#?}"
    );
}
