//! Adversary pass 1 over E-U3 (Family F A4, dotted input value paths, beyond10x/ess#233,
//! `docs/design/expression-family-source22.md`, section "A4: dotted input value paths").
//!
//! The design binds synthesis to "exercise each source twice where fallback is present: primary
//! present, then primary absent with the required fallback present", and to cover "absent Optional
//! parents". Each case here builds one model the source format admits and asks whether the
//! synthesized suite contains the run that decides the value, then whether a target wrong only in
//! that run is told apart. The last case drives the declaration player the browser product ships.

use std::cell::RefCell;
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
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

/// One invocation: what was sent, and the error payload fields the scenario then expects.
type Invocation = (
    BTreeMap<String, ScenarioValue>,
    Option<BTreeMap<String, Node>>,
);

/// Every `(input, expected error fields)` one scenario sends to `command`, in order.
fn invocations(suite: &ConformanceSuite, id: &str, command: &str) -> Vec<Invocation> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}: {:#?}",
                    suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        );
    let mut out: Vec<Invocation> = Vec::new();
    for step in &scenario.steps {
        match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => out.push((input.clone(), None)),
            ScenarioStep::ExpectError { fields, .. } => {
                if let Some(last) = out.last_mut() {
                    last.1 = Some(fields.clone());
                }
            }
            _ => {}
        }
    }
    out
}

fn literal<'a>(input: &'a BTreeMap<String, ScenarioValue>, path: &[&str]) -> Option<&'a Node> {
    let (root, rest) = path.split_first()?;
    let ScenarioValue::Literal { value } = input.get(*root)? else {
        return None;
    };
    let mut value = value;
    for segment in rest {
        value = value.as_map()?.get(*segment)?;
    }
    (*value != Node::Null).then_some(value)
}

/// A target that forwards to the interpreter, after `steer` moves the request and `answer`
/// rewrites the result: wrong in one way only.
struct Steered {
    inner: Interpreted,
    command: &'static str,
    steer: fn(&mut BTreeMap<String, Node>),
    answer: fn(&BTreeMap<String, Node>, &mut SemanticCommandResult),
    last: RefCell<BTreeMap<String, Node>>,
}

impl Steered {
    fn new(
        ir: EssIr,
        command: &'static str,
        steer: fn(&mut BTreeMap<String, Node>),
        answer: fn(&BTreeMap<String, Node>, &mut SemanticCommandResult),
    ) -> Self {
        Self {
            inner: Interpreted::for_model(ir),
            command,
            steer,
            answer,
            last: RefCell::default(),
        }
    }
}

impl ConformanceTarget for Steered {
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
        let ours = request.command.to_string() == self.command;
        if ours {
            self.last.replace(request.input.clone());
            (self.steer)(&mut request.input);
        }
        let mut result = self.inner.execute_command(request)?;
        if ours {
            (self.answer)(&self.last.borrow(), &mut result);
        }
        Ok(result)
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

fn unchanged_input(_: &mut BTreeMap<String, Node>) {}
fn unchanged_answer(_: &BTreeMap<String, Node>, _: &mut SemanticCommandResult) {}

// ---- 1. an error payload's `else: input.<…>` is never taken --------------------------------------

/// An error payload that falls back to another input (`ess/22`, A4): `nickname` is read nowhere
/// else, so only the error payload's own fallback decides whether the refusal ever runs without it.
const ERROR_FALLBACK: &str = r"format: ess/22
system: probe
version: v1
domain: probe.notes

types:
  - {name: probe.notes.NoteId, kind: newtype, of: Uuid}
  - name: probe.notes.Settings
    kind: struct
    fields:
      - {name: label, type: String}

entities:
  - name: probe.notes.Note
    identity: {name: note_id, type: probe.notes.NoteId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}

errors:
  - name: probe.notes.Refused
    summary: The note is refused.
    fields:
      - {name: label, type: String}

events:
  - name: probe.notes.Opened
    fields:
      - {name: note_id, type: probe.notes.NoteId}
      - {name: label, type: String}

commands:
  - name: probe.notes.Open
    input:
      - {name: note_id, type: probe.notes.NoteId}
      - {name: nickname, type: Optional<String>}
      - {name: name, type: String}
      - {name: settings, type: probe.notes.Settings}
      - {name: reject, type: Boolean}
    outcomes:
      - name: refused
        when: reject == true
        error: probe.notes.Refused
        payload:
          probe.notes.Refused:
            label: FALLBACK
      - name: opened
        creates: probe.notes.Note
        instance: note_id
        emits: [probe.notes.Opened]
        payload:
          probe.notes.Opened:
            note_id: input.note_id
            label: input.name
        sets:
          label: input.name

views:
  - name: probe.notes.NoteDetails
    source: probe.notes.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: probe.notes.NoteId}
      - {name: label, type: String}
";

const REFUSED: &str = "probe.notes.Open/outcome/refused";

fn error_fallback(fallback: &str) -> EssIr {
    compiled(&ERROR_FALLBACK.replace(
        "FALLBACK",
        &format!("{{input: nickname, else: {fallback}}}"),
    ))
}

/// The refusal must run once with `nickname` left out, and then expect the fallback's value.
fn assert_fallback_run(ir: &EssIr, fallback: &[&str]) {
    let suite = suite_of(ir);
    let sent = invocations(&suite, REFUSED, "probe.notes.Open");
    let refusing: Vec<_> = sent
        .iter()
        .filter(|(input, _)| literal(input, &["reject"]) == Some(&Node::Bool(true)))
        .collect();
    assert_ne!(refusing.len(), 0, "the refusal is invoked: {sent:#?}");
    let absent = refusing
        .iter()
        .find(|(input, _)| literal(input, &["nickname"]).is_none());
    let Some((input, fields)) = absent else {
        panic!(
            "no refusing invocation leaves `nickname` out, so the error payload's \
             `else: input.{}` is never exercised: {refusing:#?}",
            fallback.join(".")
        );
    };
    assert_eq!(
        fields.as_ref().and_then(|fields| fields.get("label")),
        literal(input, fallback),
        "the refusal expects the fallback's value: {input:#?} {fields:#?}"
    );
}

#[test]
fn adv_error_payload_top_level_input_fallback_is_run_with_the_primary_absent() {
    assert_fallback_run(&error_fallback("input.name"), &["name"]);
}

#[test]
fn adv_error_payload_path_fallback_behind_a_top_level_primary_is_run_with_the_primary_absent() {
    assert_fallback_run(
        &error_fallback("input.settings.label"),
        &["settings", "label"],
    );
}

/// A target that ignores the error payload's fallback (reports another value where `nickname` is
/// absent) must fail the refusal scenario; the right one must pass it.
#[test]
fn adv_a_target_ignoring_the_error_payload_fallback_is_told_apart() {
    fn ignore(last: &BTreeMap<String, Node>, result: &mut SemanticCommandResult) {
        let absent = last
            .get("nickname")
            .is_none_or(|value| *value == Node::Null);
        if absent {
            if let Some(error) = &mut result.error {
                error.fields.insert(
                    "label".to_owned(),
                    Node::Text("implementation-chosen".to_owned()),
                );
            }
        }
    }
    let ir = error_fallback("input.name");
    let suite = suite_of(&ir);
    let healthy = run(
        &suite,
        &Steered::new(
            ir.clone(),
            "probe.notes.Open",
            unchanged_input,
            unchanged_answer,
        ),
    );
    assert!(healthy.contains_key(REFUSED), "{healthy:#?}");
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    let faulty = run(
        &suite,
        &Steered::new(ir, "probe.notes.Open", unchanged_input, ignore),
    );
    assert!(
        not_passed(&faulty)
            .iter()
            .any(|line| line.starts_with(REFUSED)),
        "a target that ignores the error payload's fallback passes every scenario: {faulty:#?}"
    );
}

// ---- 2. an Optional parent below the first one is never left out ---------------------------------

/// `previous.inner.label` crosses two `Optional`s. Absence of either selects the fallback (and
/// leaves `copied` absent); synthesis must witness the inner one with the outer one present, or a
/// target that handles only the outer absence is never told apart.
const NESTED: &str = r"format: ess/22
system: probe
version: v1
domain: probe.deep

types:
  - {name: probe.deep.NoteId, kind: newtype, of: Uuid}
  - name: probe.deep.Inner
    kind: struct
    fields:
      - {name: label, type: String}
  - name: probe.deep.Outer
    kind: struct
    fields:
      - {name: tag, type: String}
      - {name: inner, type: Optional<probe.deep.Inner>}

entities:
  - name: probe.deep.Note
    identity: {name: note_id, type: probe.deep.NoteId}
    fields:
      - {name: label, type: String}
      - {name: copied, type: Optional<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}

events:
  - name: probe.deep.Opened
    fields:
      - {name: note_id, type: probe.deep.NoteId}
      - {name: label, type: String}

commands:
  - name: probe.deep.Open
    input:
      - {name: note_id, type: probe.deep.NoteId}
      - {name: previous, type: Optional<probe.deep.Outer>}
      - {name: name, type: String}
    outcomes:
      - name: opened
        creates: probe.deep.Note
        instance: note_id
        emits: [probe.deep.Opened]
        payload:
          probe.deep.Opened:
            note_id: input.note_id
            label: {input: previous.inner.label, else: input.name}
        sets:
          label: {input: previous.inner.label, else: input.name}
          copied: input.previous.inner.label

views:
  - name: probe.deep.NoteDetails
    source: probe.deep.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: probe.deep.NoteId}
      - {name: label, type: String}
      - {name: copied, type: Optional<String>}
";

const DEEP_OPENED: &str = "probe.deep.Open/outcome/opened";

#[test]
fn adv_an_inner_optional_parent_is_left_out_with_the_outer_one_present() {
    let suite = suite_of(&compiled(NESTED));
    let sent = invocations(&suite, DEEP_OPENED, "probe.deep.Open");
    assert!(
        sent.iter()
            .any(|(input, _)| literal(input, &["previous", "inner", "label"]).is_some()),
        "the primary is sent present: {sent:#?}"
    );
    assert!(
        sent.iter()
            .any(|(input, _)| literal(input, &["previous"]).is_none()),
        "the outer parent is left out: {sent:#?}"
    );
    assert!(
        sent.iter().any(|(input, _)| {
            literal(input, &["previous"]).is_some()
                && literal(input, &["previous", "inner"]).is_none()
        }),
        "no invocation sends `previous` with `previous.inner` absent, so the inner absent \
         Optional parent is never witnessed: {sent:#?}"
    );
}

/// A target that unwraps an absent inner parent into an empty member (reads the primary after its
/// absence) where the outer one is present must fail; the right one must pass.
#[test]
fn adv_a_target_unwrapping_an_absent_inner_parent_is_told_apart() {
    fn unwrap(input: &mut BTreeMap<String, Node>) {
        if let Some(Node::Map(previous)) = input.get_mut("previous") {
            let absent = previous
                .get("inner")
                .is_none_or(|value| *value == Node::Null);
            if absent {
                previous.insert(
                    "inner".to_owned(),
                    Node::Map(BTreeMap::from([(
                        "label".to_owned(),
                        Node::Text(String::new()),
                    )])),
                );
            }
        }
    }
    let ir = compiled(NESTED);
    let suite = suite_of(&ir);
    let healthy = run(
        &suite,
        &Steered::new(
            ir.clone(),
            "probe.deep.Open",
            unchanged_input,
            unchanged_answer,
        ),
    );
    assert!(healthy.contains_key(DEEP_OPENED), "{healthy:#?}");
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    let faulty = run(
        &suite,
        &Steered::new(ir, "probe.deep.Open", unwrap, unchanged_answer),
    );
    assert!(
        not_passed(&faulty)
            .iter()
            .any(|line| line.starts_with(DEEP_OPENED)),
        "a target that unwraps an absent inner Optional passes every scenario: {faulty:#?}"
    );
}

// ---- 3. the declaration player notes a sent path as a missing input ------------------------------

const PATHS: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/input-value-paths.yaml");

/// Vue stand-in: enough for `player.js` to build its scenarios and run `setup()`, so its exported
/// API steps acts exactly as the page's Step button does.
const VUE_STUB: &str = "export const reactive = (value) => value
export const computed = (read) => ({ get value() { return read() } })
export const watch = () => {}
export function createApp(options) {
  return { component() {}, provide() {}, config: {}, mount() { options.setup() } }
}
";

/// Steps every scenario to its end and prints each `missingInput` note with whether the act's
/// input carries the named source, read member by member.
const HARNESS: &str = "import { readFileSync } from 'node:fs'
globalThis.window = { addEventListener() {} }
globalThis.document = { getElementById() { return null } }
const here = new URL('.', import.meta.url)
globalThis.fetch = async (name) => ({ json: async () => JSON.parse(readFileSync(new URL(name, here), 'utf8')) })
const { default: api } = await import('./player.js')
const carried = (input, path) => {
  const [root, ...rest] = path.split('.')
  let value = input[root]
  if (!value || value.kind !== 'literal') return false
  value = value.value
  for (const member of rest) {
    if (value === null || typeof value !== 'object' || !Object.hasOwn(value, member)) return false
    value = value[member]
  }
  return value !== null
}
const notes = []
for (const scenario of api.scenarios) {
  api.select(scenario.name)
  for (let i = 0; i < scenario.acts.length; i += 1) api.step()
  for (const effect of api.state.world.unknownEffects) {
    if (!effect.missingInput) continue
    const act = scenario.acts[effect.at]
    notes.push({ scenario: scenario.name, from: effect.from, note: effect.missingInput, carried: carried(act.input, effect.from) })
  }
}
console.log(JSON.stringify(notes))
";

#[test]
fn adv_the_player_notes_no_missing_input_for_a_path_the_act_sends() {
    use std::process::Command;
    if !Command::new("node")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
    {
        println!("skipped: no `node` on PATH");
        return;
    }
    let ir = compiled(PATHS);
    let suite = suite_of(&ir);
    let artifacts = ess_conformance::web::emit(&ir, &suite).unwrap_or_else(|e| panic!("{e}"));
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-e-u3-player-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for (path, artifact) in &artifacts {
        let at = root.join(path);
        std::fs::create_dir_all(at.parent().expect("a parent")).expect("a directory");
        std::fs::write(at, &artifact.contents).expect("writes");
    }
    std::fs::write(root.join("assets/vue.esm-browser.prod.js"), VUE_STUB).expect("writes");
    std::fs::write(root.join("harness.mjs"), HARNESS).expect("writes");
    let output = Command::new("node")
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("node runs");
    let printed = String::from_utf8_lossy(&output.stdout).into_owned();
    assert!(
        output.status.success(),
        "{printed}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let notes: Vec<serde_json::Value> =
        serde_json::from_str(printed.trim()).unwrap_or_else(|e| panic!("{e}: {printed}"));
    let _ = std::fs::remove_dir_all(&root);
    let false_notes: Vec<&serde_json::Value> = notes
        .iter()
        .filter(|note| note["carried"] == serde_json::Value::Bool(true))
        .collect();
    assert_eq!(
        false_notes.len(),
        0,
        "the player says an input the act sends is missing: {false_notes:#?}"
    );
}

// ---- 4. every Optional the omission run leaves out is left out together -------------------------

/// `label` falls back when `outer.inner` is absent; `copied` reads `previous.label`, absent with
/// `previous`. Two unrelated Optionals, each deciding its own value.
const PAIR: &str = r"format: ess/22
system: probe
version: v1
domain: probe.pair

types:
  - {name: probe.pair.NoteId, kind: newtype, of: Uuid}
  - name: probe.pair.Inner
    kind: struct
    fields:
      - {name: label, type: String}
  - name: probe.pair.Outer
    kind: struct
    fields:
      - {name: tag, type: String}
      - {name: inner, type: Optional<probe.pair.Inner>}

entities:
  - name: probe.pair.Note
    identity: {name: note_id, type: probe.pair.NoteId}
    fields:
      - {name: label, type: String}
      - {name: copied, type: Optional<String>}
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
      - {name: outer, type: probe.pair.Outer}
      - {name: previous, type: Optional<probe.pair.Inner>}
      - {name: name, type: String}
    outcomes:
      - name: opened
        creates: probe.pair.Note
        instance: note_id
        emits: [probe.pair.Opened]
        payload:
          probe.pair.Opened:
            note_id: input.note_id
            label: {input: outer.inner.label, else: input.name}
        sets:
          label: {input: outer.inner.label, else: input.name}
          copied: input.previous.label

views:
  - name: probe.pair.NoteDetails
    source: probe.pair.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: probe.pair.NoteId}
      - {name: label, type: String}
      - {name: copied, type: Optional<String>}
";

const PAIR_OPENED: &str = "probe.pair.Open/outcome/opened";

/// A target that decides `label`'s fallback by the absence of `previous` (the wrong Optional)
/// rather than of `outer.inner` must fail; the right one must pass.
#[test]
fn adv_a_target_choosing_the_fallback_by_another_optional_is_told_apart() {
    fn other_optional(input: &mut BTreeMap<String, Node>) {
        let previous = input
            .get("previous")
            .is_some_and(|value| *value != Node::Null);
        if let Some(Node::Map(outer)) = input.get_mut("outer") {
            if previous {
                outer.entry("inner".to_owned()).or_insert_with(|| {
                    Node::Map(BTreeMap::from([(
                        "label".to_owned(),
                        Node::Text(String::new()),
                    )]))
                });
            } else {
                outer.remove("inner");
            }
        }
    }
    let ir = compiled(PAIR);
    let suite = suite_of(&ir);
    let sent = invocations(&suite, PAIR_OPENED, "probe.pair.Open");
    let healthy = run(
        &suite,
        &Steered::new(
            ir.clone(),
            "probe.pair.Open",
            unchanged_input,
            unchanged_answer,
        ),
    );
    assert!(healthy.contains_key(PAIR_OPENED), "{healthy:#?}");
    assert_eq!(not_passed(&healthy), Vec::<String>::new());
    let faulty = run(
        &suite,
        &Steered::new(ir, "probe.pair.Open", other_optional, unchanged_answer),
    );
    assert!(
        not_passed(&faulty)
            .iter()
            .any(|line| line.starts_with(PAIR_OPENED)),
        "a target that falls back on the wrong Optional's absence passes every scenario; \
         the suite sends `previous` and `outer.inner` only both present or both absent: \
         {sent:#?}"
    );
}

// ---- 5. a path's `sets-retarget` mutant is stillborn when the decoy has another type ---------------

/// The fixture with one more top-level input named like a member a path ends in, `label`, of
/// another type. A top-level `sets-retarget` mutant is only made toward an input of the same type,
/// so it always compiles; a path's must hold to the same rule, or it is stillborn rather than a
/// decoy an implementation could read.
#[test]
fn adv_a_path_retarget_mutant_compiles() {
    use ess_conformance::mutate::{self, MutantClass};
    let model = PATHS.replacen(
        "      - {name: reject, type: Boolean}\n",
        "      - {name: reject, type: Boolean}\n      - {name: label, type: Integer}\n",
        1,
    );
    assert_ne!(model, PATHS);
    compiled(&model);
    let raw = RawSpecFile::parse(&model).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), model.clone());
    let files = vec![(Source::new("model.yaml"), raw)];
    let retargets = mutate::mutants(&files, &[MutantClass::SetsRetarget]);
    assert_ne!(retargets.len(), 0, "a path ends in `label`");
    let stillborn: Vec<String> = retargets
        .iter()
        .filter(|mutant| {
            let applied = mutate::apply(&files, &mutant.mutation)
                .unwrap_or_else(|error| panic!("{}: {error}", mutant.id));
            mutate::compile(applied, &texts).is_err()
        })
        .map(|mutant| format!("{}: {}", mutant.id, mutant.change))
        .collect();
    assert_eq!(
        stillborn,
        Vec::<String>::new(),
        "a path's retarget names a same-named input of another type"
    );
}
