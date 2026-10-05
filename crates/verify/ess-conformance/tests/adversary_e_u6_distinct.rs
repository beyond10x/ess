//! Adversary, pass 1, for unit E-U6 (`distinct: {in, as, by}`, beyond10x/ess#237).
//!
//! Each case drives the implementation from `docs/design/expression-family-source22.md`, section
//! `distinct`, rather than from the unit's own vectors:
//!
//! - the finite-domain witness controls: a singleton enum with required length two and three
//!   Boolean keys with required length three have no distinct witness and must be refused by name;
//!   a two-variant enum and `[false, true]` must have one;
//! - "a decisive positive test also requires `list.count > 1` so vacuity cannot pass", for lists the
//!   unit's `keyed_lists` does not shape: one under a quantifier, one nested in an input struct, and
//!   a stored subject's list;
//! - "Rust/Go/TS runners agree on `distinct` over view rows" (final decision 1) where the binder has
//!   the name of a root field the view also publishes.
#![allow(clippy::too_many_lines)]

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, ConformanceSuite, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

mod support_go;

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir_of(text))
}

fn has_scenario(suite: &ConformanceSuite, id: &str) -> bool {
    suite.scenarios.keys().any(|key| key.to_string() == id)
}

/// The last literal input the scenario `id` sends, or `None` where the suite holds no such scenario.
fn sent(suite: &ConformanceSuite, id: &str) -> Option<BTreeMap<String, Node>> {
    let scenario = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)?;
    scenario.steps.iter().rev().find_map(|step| match step {
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
}

fn seq(node: Option<&Node>) -> Vec<Node> {
    match node {
        Some(Node::Seq(items)) => items.clone(),
        _ => Vec::new(),
    }
}

// ---- the finite-domain controls -------------------------------------------------------------------

/// One command over `List<element>` that refuses a length other than `length`, then a duplicate,
/// and otherwise accepts.
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
  - name: pool.keys.Two
    kind: enum
    variants: [Red, Green]
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

const ACCEPTED: &str = "pool.keys.Pick/outcome/accepted";

/// The design: "A singleton enum with required length two produces the named no-witness refusal …
/// Both finite cases preserve the element invariants and must not substitute an empty/one-item
/// list."
#[test]
fn adv_u6_a_singleton_enum_with_required_length_two_is_refused_by_name() {
    let result = synthesis(&finite("pool.keys.One", 2));
    let refused_accepted = result.refusals.iter().any(|refusal| {
        refusal
            .scenario
            .as_ref()
            .map(ToString::to_string)
            .as_deref()
            == Some(ACCEPTED)
    });
    assert!(
        refused_accepted && !has_scenario(&result.suite, ACCEPTED),
        "the accepting branch has no distinct witness of two `Only` keys and must be refused by \
         name, not sent: refusals {:#?}, accepted input {:#?}",
        result.refusals,
        sent(&result.suite, ACCEPTED)
    );
}

/// The design: "… while a two-variant enum must produce a healthy two-key witness."
#[test]
fn adv_u6_a_two_variant_enum_has_a_two_key_witness() {
    let result = synthesis(&finite("pool.keys.Two", 2));
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let keys = seq(sent(&result.suite, ACCEPTED)
        .expect("the accepting scenario")
        .get("keys"));
    assert_eq!(keys.len(), 2, "{keys:#?}");
    assert_ne!(keys[0], keys[1], "two different variants: {keys:#?}");
}

/// The design: "Use three Boolean keys with required length three to prove that a distinct witness
/// does not exist."
#[test]
fn adv_u6_three_boolean_keys_have_no_distinct_witness() {
    let result = synthesis(&finite("Boolean", 3));
    let refused_accepted = result.refusals.iter().any(|refusal| {
        refusal
            .scenario
            .as_ref()
            .map(ToString::to_string)
            .as_deref()
            == Some(ACCEPTED)
    });
    assert!(
        refused_accepted && !has_scenario(&result.suite, ACCEPTED),
        "three distinct Booleans do not exist: refusals {:#?}, accepted input {:#?}",
        result.refusals,
        sent(&result.suite, ACCEPTED)
    );
}

/// The design: "a length-two `[false,true]` case must have one."
#[test]
fn adv_u6_two_boolean_keys_have_a_distinct_witness() {
    let result = synthesis(&finite("Boolean", 2));
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let keys = seq(sent(&result.suite, ACCEPTED)
        .expect("the accepting scenario")
        .get("keys"));
    let mut flags: Vec<String> = keys.iter().map(|key| format!("{key:?}")).collect();
    flags.sort();
    assert_eq!(flags.len(), 2, "{keys:#?}");
    assert_ne!(flags[0], flags[1], "[false, true]: {keys:#?}");
}

// ---- decisive witnesses for lists `keyed_lists` does not shape ------------------------------------

/// The counterpart of the Boolean control: a key domain with enough values has a distinct witness at
/// every required length inside the bound, here three Strings.
#[test]
fn adv_u6_three_distinct_strings_are_witnessed() {
    let result = synthesis(&finite("String", 3));
    assert_eq!(
        result.refusals.len(),
        0,
        "three distinct Strings exist: {:#?}",
        result.refusals
    );
    let keys = seq(sent(&result.suite, ACCEPTED)
        .expect("the accepting scenario")
        .get("keys"));
    assert_eq!(keys.len(), 3, "{keys:#?}");
}

/// An `Optional` list and an `Optional` key member, each guarded by `distinct`.
#[test]
fn adv_u6_optional_lists_and_keys_are_witnessed_decisively() {
    let model = r"format: ess/22
system: pool
version: v1
domain: pool.opt
types:
  - name: pool.opt.File
    kind: struct
    fields:
      - {name: path, type: Optional<String>}
      - {name: size, type: Integer}
errors:
  - {name: pool.opt.DuplicateTag, summary: A tag repeats.}
  - {name: pool.opt.DuplicatePath, summary: Two files share a path.}
events:
  - {name: pool.opt.Accepted, fields: []}
commands:
  - name: pool.opt.Check
    input:
      - {name: tags, type: Optional<List<String>>}
      - {name: files, type: List<pool.opt.File>}
    outcomes:
      - name: duplicate-tag
        when: {not: {distinct: {in: tags, as: tag}}}
        error: pool.opt.DuplicateTag
      - name: duplicate-path
        when: {not: {distinct: {in: files, as: file, by: file.path}}}
        error: pool.opt.DuplicatePath
      - name: accepted
        emits: [pool.opt.Accepted]
";
    let result = synthesis(model);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let accepted = sent(&result.suite, "pool.opt.Check/outcome/accepted").expect("accepted");
    assert!(
        seq(accepted.get("tags")).len() >= 2 && seq(accepted.get("files")).len() >= 2,
        "the accepting witness holds two or more tags and files: {accepted:#?}"
    );
}

/// A `distinct` under a quantifier, which `keyed_lists` skips (`scope` non-empty).
const GROUPS: &str = r"format: ess/22
system: pool
version: v1
domain: pool.groups
types:
  - name: pool.groups.Member
    kind: struct
    fields:
      - {name: name, type: String}
  - name: pool.groups.Group
    kind: struct
    fields:
      - {name: members, type: List<pool.groups.Member>}
errors:
  - {name: pool.groups.Duplicate, summary: A group repeats a member.}
events:
  - {name: pool.groups.Accepted, fields: []}
commands:
  - name: pool.groups.Check
    input:
      - {name: groups, type: List<pool.groups.Group>}
    outcomes:
      - name: duplicate
        when: {not: {forall: {in: groups, as: group, that: {distinct: {in: group.members, as: member, by: member.name}}}}}
        error: pool.groups.Duplicate
      - name: accepted
        emits: [pool.groups.Accepted]
";

/// The design, Synthesis row: "Arrange both branches". `[{members: [a, a]}]` refuses; the branch is
/// witnessable inside the bound.
#[test]
fn adv_u6_a_duplicate_under_a_quantifier_is_witnessed() {
    let result = synthesis(GROUPS);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
}

/// The design: "Empty/one-element inputs remain controls, but a decisive positive test also
/// requires `list.count > 1` so vacuity cannot pass."
#[test]
fn adv_u6_a_distinct_under_a_quantifier_is_accepted_decisively() {
    let result = synthesis(GROUPS);
    let accepted = sent(&result.suite, "pool.groups.Check/outcome/accepted").expect("accepted");
    let groups = seq(accepted.get("groups"));
    let widest = groups
        .iter()
        .map(|group| match group {
            Node::Map(fields) => seq(fields.get("members")).len(),
            _ => 0,
        })
        .max()
        .unwrap_or(0);
    assert!(
        widest >= 2,
        "the accepting witness holds a group of two or more members, so a target that compares \
         nothing cannot pass it: {accepted:#?}"
    );
}

/// The same, for a list nested in an input struct.
#[test]
fn adv_u6_a_nested_input_list_is_witnessed_decisively() {
    let model = r"format: ess/22
system: pool
version: v1
domain: pool.nest
types:
  - name: pool.nest.File
    kind: struct
    fields:
      - {name: path, type: String}
      - {name: size, type: Integer}
  - name: pool.nest.Bundle
    kind: struct
    fields:
      - {name: files, type: List<pool.nest.File>}
errors:
  - {name: pool.nest.Duplicate, summary: Two files share a path.}
events:
  - {name: pool.nest.Accepted, fields: []}
commands:
  - name: pool.nest.Check
    input:
      - {name: bundle, type: pool.nest.Bundle}
    outcomes:
      - name: duplicate
        when: {not: {distinct: {in: bundle.files, as: file, by: file.path}}}
        error: pool.nest.Duplicate
      - name: accepted
        emits: [pool.nest.Accepted]
";
    let result = synthesis(model);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let accepted = sent(&result.suite, "pool.nest.Check/outcome/accepted").expect("accepted");
    let files = match accepted.get("bundle") {
        Some(Node::Map(fields)) => seq(fields.get("files")),
        _ => Vec::new(),
    };
    assert!(
        files.len() >= 2,
        "the accepting witness holds two or more files: {accepted:#?}"
    );
}

/// The longest `tags` list anywhere under `value`: a raw array, or the `{kind: literal, value: [...]}`
/// a command input or a view field carries one as.
fn widest(value: &serde_json::Value) -> usize {
    match value {
        serde_json::Value::Object(fields) => fields
            .iter()
            .map(|(key, value)| match (key.as_str(), value) {
                ("tags", serde_json::Value::Array(items)) => items.len(),
                ("tags", serde_json::Value::Object(literal)) => literal
                    .get("value")
                    .and_then(serde_json::Value::as_array)
                    .map_or(0, Vec::len),
                _ => widest(value),
            })
            .max()
            .unwrap_or(0),
        serde_json::Value::Array(items) => items.iter().map(widest).max().unwrap_or(0),
        _ => 0,
    }
}

/// The same, for a stored subject's list read by a `when_subject` guard.
#[test]
fn adv_u6_a_stored_subject_list_is_witnessed_decisively() {
    let model = r"format: ess/22
system: pool
version: v1
domain: pool.tags
entities:
  - name: pool.tags.Bundle
    identity: {name: bundle_id, type: Uuid}
    fields:
      - {name: tags, type: List<String>}
    lifecycle:
      initial: Open
      states: [Open, Sealed]
      terminal: [Sealed]
      transitions:
        - {name: seal, from: [Open], to: Sealed}
events:
  - name: pool.tags.Opened
    fields:
      - {name: bundle_id, type: Uuid}
  - name: pool.tags.Sealed
    fields: []
errors:
  - name: pool.tags.Duplicate
    fields: []
commands:
  - name: pool.tags.Open
    input:
      - {name: tags, type: List<String>}
    outcomes:
      - name: opened
        creates: pool.tags.Bundle
        instance: bundle_id
        sets: {tags: input.tags}
        emits: [pool.tags.Opened]
        payload:
          pool.tags.Opened:
            bundle_id: {generated: true}
  - name: pool.tags.Seal
    input:
      - {name: bundle_id, type: Uuid}
    outcomes:
      - name: duplicate
        when_subject:
          predicate: {not: {distinct: {in: tags, as: tag}}}
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
      - {name: bundle_id, type: Uuid}
      - {name: state, type: pool.tags.Bundle.State}
      - {name: tags, type: List<String>}
";
    let result = synthesis(model);
    let json = result.suite.to_canonical_json().expect("serialises");
    let document: serde_json::Value = serde_json::from_str(&json).expect("JSON");
    let scenario = |outcome: &str| {
        document["scenarios"]
            .as_object()
            .and_then(|scenarios| {
                scenarios
                    .iter()
                    .find(|(id, _)| *id == &format!("pool.tags.Seal/outcome/{outcome}"))
            })
            .map(|(_, scenario)| scenario.clone())
    };
    let (Some(duplicate), Some(sealed)) = (scenario("duplicate"), scenario("sealed")) else {
        panic!(
            "both branches of the stored guard are witnessed: refusals {:#?}",
            result.refusals
        );
    };
    assert!(
        widest(&duplicate) >= 2 && widest(&sealed) >= 2,
        "both branches arrange two or more stored tags: duplicate {duplicate:#}\nsealed {sealed:#}"
    );
}

// ---- runner agreement when the binder names a root field --------------------------------------------

/// A bundle whose view publishes `file` beside `files`, and whose invariant walks `files` under the
/// binder `file`.
const SHADOWED: &str = r"format: ess/22
system: pool
version: v1
domain: pool.files
types:
  - name: pool.files.File
    kind: struct
    fields:
      - {name: path, type: String}
entities:
  - name: pool.files.Bundle
    identity: {name: bundle_id, type: Uuid}
    fields:
      - {name: file, type: pool.files.File}
      - {name: files, type: List<pool.files.File>}
    invariants:
      - {distinct: {in: files, as: file, by: file.path}}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: pool.files.DuplicatePath, summary: Two files share a path.}
events:
  - name: pool.files.Opened
    fields:
      - {name: bundle_id, type: Uuid}
commands:
  - name: pool.files.Open
    input:
      - {name: file, type: pool.files.File}
      - {name: files, type: List<pool.files.File>}
    outcomes:
      - name: duplicate-path
        when: {not: {distinct: {in: files, as: file, by: file.path}}}
        error: pool.files.DuplicatePath
      - name: opened
        creates: pool.files.Bundle
        instance: bundle_id
        emits: [pool.files.Opened]
        payload:
          pool.files.Opened: {bundle_id: {generated: true}}
        sets: {file: input.file, files: input.files}
views:
  - name: pool.files.Bundles
    source: pool.files.Bundle
    consistency: read_your_writes
    fields:
      - {name: bundle_id, type: Uuid}
      - {name: file, type: pool.files.File}
      - {name: files, type: List<pool.files.File>}
";

const SHADOWED_INVARIANT: &str = "pool.files.Bundle/invariant/after/pool.files.Open/opened";

/// The interpreter, publishing every row with its first file's `path` left out — so the invariant
/// is Unknown for that row — and the root `file` holding a path no element holds.
struct DropsFirstPath {
    inner: Interpreted,
}

impl ConformanceTarget for DropsFirstPath {
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
        for row in &mut result.rows {
            if let Some(Node::Seq(files)) = row.get_mut("files") {
                if let Some(Node::Map(first)) = files.first_mut() {
                    first.remove("path");
                }
            }
            row.insert(
                "file".to_owned(),
                Node::Map([("path".to_owned(), Node::Text("outer-only".to_owned()))].into()),
            );
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

fn shadowed_suite() -> ConformanceSuite {
    let result = synthesis(SHADOWED);
    assert_eq!(
        result.refusals.len(),
        0,
        "nothing is refused: {:#?}",
        result.refusals
    );
    let opened = sent(&result.suite, "pool.files.Open/outcome/opened").expect("opened");
    assert!(
        seq(opened.get("files")).len() >= 2,
        "the published list holds two or more files: {opened:#?}"
    );
    result.suite
}

/// Rust reads the first element's absent key as Unknown and does not pass the row; the Go runner
/// must not pass it either, whatever name it gives an undecidable row.
#[test]
fn adv_u6_go_does_not_read_a_root_field_for_an_elements_absent_key() {
    let suite = shadowed_suite();
    let (rust, replayed) = support_go::compare(
        "adv-u6-shadowed",
        &suite,
        DropsFirstPath {
            inner: Interpreted::for_model(ir_of(SHADOWED)),
        },
    );
    assert_ne!(
        rust.get(SHADOWED_INVARIANT).map(String::as_str),
        Some("passed"),
        "Rust: {rust:#?}"
    );
    assert_ne!(
        replayed
            .go
            .outcomes
            .get(SHADOWED_INVARIANT)
            .map(String::as_str),
        Some("passed"),
        "Go passes a row whose first element has no key, reading the root `file.path` in its place \
         (Rust: {:?}):\n{}",
        rust.get(SHADOWED_INVARIANT),
        replayed.go.log
    );
}

/// [`DropsFirstPath`] in JavaScript.
const SHADOWED_JS: &str = r"class Bundles {
  rows = [];
  seq = 0;
  identity() { return { name: 'bundles', version: '1' }; }
  beginScenario() { this.rows = []; }
  endScenario() {}
  executeCommand({ command, input }) {
    if (command !== 'pool.files.Open') throw new Error(`unexpected ${command}`);
    const paths = input.files.map((file) => file.path);
    if (new Set(paths).size !== paths.length) {
      return { outcome: 'duplicate-path', consistency: `seq:${this.seq}`, error: 'pool.files.DuplicatePath' };
    }
    this.seq += 1;
    const id = `00000000-0000-4000-8000-${String(this.seq).padStart(12, '0')}`;
    this.rows.push({ bundle_id: id, file: input.file, files: input.files });
    return {
      outcome: 'opened',
      consistency: `seq:${this.seq}`,
      directEvents: [{ event: 'pool.files.Opened', payload: { bundle_id: id } }],
    };
  }
  queryView() {
    return {
      rows: this.rows.map((row) => {
        const [first, ...rest] = row.files;
        const { path, ...kept } = first;
        return { ...row, file: { path: 'outer-only' }, files: [kept, ...rest] };
      }),
    };
  }
  observeEvents() { throw new Error('unused'); }
  configureExternalOutcome() { throw new Error('nothing is external'); }
  redeliverEvent() { throw new Error('no bindings'); }
}
export function makeTarget() { return new Bundles(); }
";

#[test]
fn adv_u6_typescript_does_not_read_a_root_field_for_an_elements_absent_key() {
    use std::process::Command;
    for name in ["tsc", "node"] {
        assert!(
            Command::new(name)
                .arg("--version")
                .output()
                .is_ok_and(|output| output.status.success()),
            "`{name}` is on PATH"
        );
    }
    let admitted = AdmittedSuite::from_suite(&shadowed_suite()).expect("admits");
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("adv-u6-shadowed")
        .join(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&root);
    for artifact in ess_conformance::ts::emit(admitted.suite()).expect("the package emits") {
        let path = root.join(&artifact.path);
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("a directory");
        std::fs::write(path, artifact.contents).expect("writes");
    }
    let dir = root.join(ess_conformance::ts::PACKAGE);
    for (name, contents) in [
        ("target.mjs", SHADOWED_JS),
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
    let _ = std::fs::remove_dir_all(&root);
    let document: serde_json::Value = serde_json::from_str(&text).expect("report/2 is JSON");
    let mut verdicts = BTreeMap::new();
    for (status, ids) in document["outcomes"].as_object().expect("outcomes") {
        for id in ids.as_array().expect("a list") {
            verdicts.insert(id.as_str().expect("an id").to_owned(), status.clone());
        }
    }
    assert!(
        verdicts.contains_key(SHADOWED_INVARIANT),
        "the invariant ran: {verdicts:#?}"
    );
    assert_ne!(
        verdicts.get(SHADOWED_INVARIANT).map(String::as_str),
        Some("passed"),
        "TypeScript passes a row whose first element has no key, reading the root `file.path` in \
         its place: {verdicts:#?}"
    );
}
