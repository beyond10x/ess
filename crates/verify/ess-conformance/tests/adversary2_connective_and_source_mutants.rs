//! Second adversary pass on `story:synthesis-kills-connective-and-source-mutants` (beyond10x/ess#154,
//! #155, #132, #160, #161): the neighbourhoods of the round-1 corrections.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::ir::{EssIr, ResolvedCommand};
use ess_compiler::source::SourceMap;
use ess_conformance::mutate::{self, Document, MutantClass};
use ess_conformance::scenario::{ConformanceSuite, ScenarioStep};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{flatten, when, Decision, ScenarioId};
use ess_domain::command::TestStrategy;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

fn documents(text: &str) -> (Vec<Document>, SourceMap) {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let mut texts = SourceMap::new();
    texts.insert("fixture.yaml".to_owned(), text.to_owned());
    (vec![(Source::new("fixture.yaml"), raw)], texts)
}

fn compiled(text: &str) -> EssIr {
    let (files, texts) = documents(text);
    mutate::compile(files, &texts).unwrap_or_else(|stillborn| {
        panic!("the model compiles: {} {}", stillborn.code, stillborn.cause)
    })
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

struct Invocation {
    scenario: String,
    command: String,
    input: BTreeMap<String, Node>,
    expected: String,
    arranged: BTreeMap<String, BTreeMap<String, Node>>,
}

fn invocations(synthesis: &Synthesis) -> Vec<Invocation> {
    let mut out = Vec::new();
    for (id, scenario) in &synthesis.suite.scenarios {
        let mut arranged: BTreeMap<String, BTreeMap<String, Node>> = BTreeMap::new();
        let mut pending: Option<(String, BTreeMap<String, Node>)> = None;
        for step in &scenario.steps {
            match step {
                ScenarioStep::ExecuteCommand { command, input, .. } => {
                    if let Some((command, input)) = pending.take() {
                        arranged.insert(command, input);
                    }
                    let literals = input
                        .iter()
                        .filter_map(|(field, value)| {
                            value
                                .as_literal()
                                .cloned()
                                .map(|node| (field.clone(), node))
                        })
                        .collect();
                    pending = Some((command.to_string(), literals));
                }
                ScenarioStep::ExpectOutcome { outcome } => {
                    if let Some((command, input)) = pending.take() {
                        let expected = outcome
                            .to_string()
                            .rsplit('/')
                            .next()
                            .expect("an outcome ref names its branch")
                            .to_owned();
                        out.push(Invocation {
                            scenario: id.to_string(),
                            command: command.clone(),
                            input: input.clone(),
                            expected,
                            arranged: arranged.clone(),
                        });
                        arranged.insert(command, input);
                    }
                }
                _ => {}
            }
        }
    }
    out
}

fn command<'ir>(ir: &'ir EssIr, name: &str) -> &'ir ResolvedCommand {
    ir.commands()
        .get(&QualifiedName::new(name).expect("a valid name"))
        .expect("the command is declared")
}

fn selected(ir: &EssIr, name: &str, input: &BTreeMap<String, Node>) -> String {
    let command = command(ir, name);
    let facts = flatten(ir, command, input).expect("a synthesised input fits its command");
    for outcome in &command.outcomes {
        if outcome.test_strategy == TestStrategy::ConstructInput {
            let guard = when(outcome).expect("a constructed branch has a guard");
            if facts.decide(guard) == Decision::Satisfied {
                return outcome.name.to_string();
            }
        }
    }
    command
        .outcomes
        .iter()
        .find(|outcome| outcome.test_strategy == TestStrategy::DefaultBranch)
        .map(|outcome| outcome.name.to_string())
        .expect("a default")
}

fn contradicted(synthesis: &Synthesis, original: &EssIr, name: &str) -> Vec<String> {
    invocations(synthesis)
        .into_iter()
        .filter(|invocation| invocation.command == name)
        .filter(|invocation| selected(original, name, &invocation.input) != invocation.expected)
        .map(|invocation| format!("{} {:?}", invocation.scenario, invocation.input))
        .collect()
}

fn sent(synthesis: &Synthesis, name: &str) -> Vec<(String, BTreeMap<String, Node>)> {
    invocations(synthesis)
        .into_iter()
        .filter(|invocation| invocation.command == name)
        .map(|invocation| (invocation.expected, invocation.input))
        .collect()
}

/// Every mutant of `class` the audit enumerates for `text`, with the synthesized suite of each
/// checked against the unmutated model: the ids of those no scenario kills.
fn survivors(text: &str, class: MutantClass, name: &str) -> Vec<String> {
    let (files, texts) = documents(text);
    let original = compiled(text);
    assert!(
        contradicted(&synthesize(&original), &original, name).is_empty(),
        "the unmutated model agrees with itself"
    );
    let mutants = mutate::mutants(&files, &[class]);
    assert!(!mutants.is_empty(), "the audit enumerates a mutant");
    let mut out = Vec::new();
    for mutant in mutants {
        let mutated = mutate::apply(&files, &mutant.mutation).expect("the site exists");
        let Ok(ir) = mutate::compile(mutated, &texts) else {
            continue;
        };
        let suite = synthesize(&ir);
        if contradicted(&suite, &original, name).is_empty() {
            out.push(format!(
                "{} survives: sends {:?}",
                mutant.id,
                sent(&suite, name)
            ));
        }
    }
    out
}

// ---- #155 on nested input-guard connectives --------------------------------------------------------

const CHECK: &str = r"
format: ess/13
system: gate
version: v1
domain: gate.check
events:
  - name: gate.check.Passed
    fields:
      - {name: a, type: Integer}
errors:
  - name: gate.check.Refused
    summary: Refused.
    fields: []
commands:
  - name: gate.check.Check
    input:
      - {name: a, type: Integer}
      - {name: b, type: Integer}
      - {name: c, type: Integer}
    outcomes:
      - name: refused
        when:
GUARD
        error: gate.check.Refused
      - name: passed
        emits: [gate.check.Passed]
        payload:
          gate.check.Passed: {a: input.a}
";

/// `any: [all: [a > 10, b > 10], c > 10]` and `all: [any: [a > 10, b > 10], c > 10]`: the audit
/// enumerates a `guard-connective` mutant for the outer and the inner connective of each, and every
/// one of them must synthesize an invocation the unmutated model answers differently.
#[test]
fn adversary2_nested_input_guard_connective_mutants_are_all_killed() {
    let mut out = Vec::new();
    for (shape, guard) in [
        (
            "any over all",
            "          any:\n            - all: [a > 10, b > 10]\n            - c > 10",
        ),
        (
            "all over any",
            "          all:\n            - any: [a > 10, b > 10]\n            - a < 100",
        ),
        (
            "any over all, shared field",
            "          any:\n            - all: [a > 10, b > 10]\n            - a > 50",
        ),
    ] {
        for survivor in survivors(
            &CHECK.replace("GUARD", guard),
            MutantClass::GuardConnective,
            "gate.check.Check",
        ) {
            out.push(format!("{shape}: {survivor}"));
        }
    }
    assert!(out.is_empty(), "{out:#?}");
}

/// `all: [any: [a > 10, b > 10], c > 10]` over three `Integer` inputs: the guarded branch is
/// witnessed at all (e.g. `a: 11, b: 1, c: 11`), and its inner `any`→`all` mutant is killed. The
/// witness search gives up after 64 candidates, so neither the branch nor either connective mutant
/// gets a scenario.
#[test]
#[ignore = "story:witness-search-beyond-64-candidates: the witness search stops at 64 candidates for a guard over three inputs"]
fn adversary2_a_three_input_nested_guard_is_witnessed_and_its_mutants_killed() {
    let text = CHECK.replace(
        "GUARD",
        "          all:\n            - any: [a > 10, b > 10]\n            - c > 10",
    );
    let synthesis = synthesize(&compiled(&text));
    assert!(
        sent(&synthesis, "gate.check.Check")
            .iter()
            .any(|(expected, _)| expected == "refused"),
        "the guarded branch is witnessed: {:#?}",
        refusals(&synthesis)
    );
    let out = survivors(&text, MutantClass::GuardConnective, "gate.check.Check");
    assert!(out.is_empty(), "{out:#?}");
}

// ---- #160 neighbourhood: Decimal and Timestamp boundaries ------------------------------------------

const MEASURE: &str = r"
format: ess/13
system: meter
version: v1
domain: meter.read
events:
  - name: meter.read.Taken
    fields:
      - {name: amount, type: Decimal}
errors:
  - name: meter.read.Refused
    summary: Refused.
    fields: []
commands:
  - name: meter.read.Take
    input:
      - {name: amount, type: Decimal}
      - {name: at, type: Timestamp}
    outcomes:
      - name: refused
        when: GUARD
        error: meter.read.Refused
      - name: taken
        emits: [meter.read.Taken]
        payload:
          meter.read.Taken: {amount: input.amount}
";

/// A strictness swap on a `Decimal` literal with a fraction and on a `Timestamp` instant, every
/// ordering operator: each `guard-boundary` mutant is killed.
#[test]
fn adversary2_decimal_and_timestamp_boundary_mutants_are_killed() {
    let mut out = Vec::new();
    for op in [">", ">=", "<", "<="] {
        for guard in [
            format!("'amount {op} 10.5'"),
            format!("'at {op} \"2026-01-01T00:00:00Z\"'"),
        ] {
            for survivor in survivors(
                &MEASURE.replace("GUARD", &guard),
                MutantClass::GuardBoundary,
                "meter.read.Take",
            ) {
                out.push(format!("{guard}: {survivor}"));
            }
        }
    }
    assert!(out.is_empty(), "{out:#?}");
}

// ---- #155 on a nested stored-field connective ------------------------------------------------------

const PARCELS: &str = r"
format: ess/13
system: shipping
version: v1
domain: shipping.parcel
types:
  - {name: shipping.parcel.Service, kind: enum, variants: [Standard, Express]}
  - {name: shipping.parcel.ParcelId, kind: newtype, of: Uuid}
entities:
  - name: shipping.parcel.Parcel
    identity: {name: parcel_id, type: shipping.parcel.ParcelId}
    fields:
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
    lifecycle:
      initial: Created
      states: [Created, Dispatched]
      terminal: [Dispatched]
      transitions:
        - {name: dispatch, from: [Created], to: Dispatched}
events:
  - name: shipping.parcel.Created
    fields:
      - {name: parcel_id, type: shipping.parcel.ParcelId}
  - name: shipping.parcel.Dispatched
    fields: []
errors:
  - name: shipping.parcel.Refused
    summary: Refused.
    fields: []
commands:
  - name: shipping.parcel.Create
    input:
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
    outcomes:
      - name: created
        creates: shipping.parcel.Parcel
        instance: parcel_id
        sets: {service: input.service, weight_kg: input.weight_kg}
        emits: [shipping.parcel.Created]
        payload:
          shipping.parcel.Created:
            parcel_id: {generated: true}
  - name: shipping.parcel.Dispatch
    input:
      - {name: parcel_id, type: shipping.parcel.ParcelId}
    outcomes:
      - name: refused
        when_subject:
          predicate:
            any:
              - all: [service == Express, weight_kg > 20]
              - weight_kg > 50
        error: shipping.parcel.Refused
      - name: dispatched
        moves: shipping.parcel.Parcel.dispatch
        instance: parcel_id
        emits: [shipping.parcel.Dispatched]
views:
  - name: shipping.parcel.Parcels
    source: shipping.parcel.Parcel
    consistency: read_your_writes
    fields:
      - {name: parcel_id, type: shipping.parcel.ParcelId}
      - {name: state, type: shipping.parcel.Parcel.State}
      - {name: service, type: shipping.parcel.Service}
      - {name: weight_kg, type: Integer}
";

/// The rows (express, weight) a `Dispatch` requiring `branch` was sent against.
fn parcel_rows(synthesis: &Synthesis, branch: &str) -> Vec<(bool, f64)> {
    invocations(synthesis)
        .into_iter()
        .filter(|invocation| {
            invocation.command == "shipping.parcel.Dispatch" && invocation.expected == branch
        })
        .filter_map(|invocation| {
            let create = invocation.arranged.get("shipping.parcel.Create")?;
            let express = matches!(create.get("service")?, Node::Text(text) if text == "Express");
            let weight = match create.get("weight_kg")? {
                Node::Number(number) => number.get(),
                _ => return None,
            };
            Some((express, weight))
        })
        .collect()
}

/// `any: [all: [service == Express, weight_kg > 20], weight_kg > 50]` over stored fields. Rewriting
/// the inner `all` to `any` refuses Express parcels of 20 kg or less and Standard parcels of 21–50
/// kg, so some `dispatched` row must be one of those; rewriting the outer `any` to `all` dispatches
/// every refused row that is not an Express parcel over 50 kg, so some `refused` row must be one of
/// those.
#[test]
fn adversary2_nested_stored_field_connective_is_witnessed_for_both_mutants() {
    let synthesis = synthesize(&compiled(PARCELS));
    let refused = parcel_rows(&synthesis, "refused");
    let dispatched = parcel_rows(&synthesis, "dispatched");
    assert!(
        !refused.is_empty() && !dispatched.is_empty(),
        "both branches are witnessed: {:?}",
        refusals(&synthesis)
    );
    let outer_killed = refused
        .iter()
        .any(|(express, weight)| !(*express && *weight > 50.0));
    let inner_killed = dispatched.iter().any(|(express, weight)| {
        (*express && *weight <= 20.0) || (!*express && *weight > 20.0 && *weight <= 50.0)
    });
    assert!(
        outer_killed && inner_killed,
        "outer any->all killed: {outer_killed}, inner all->any killed: {inner_killed}; \
         refused rows {refused:?}, dispatched rows {dispatched:?}"
    );
}

// ---- #161: two literal writes, one of which no arrangement can change -------------------------------

const RECORDED: &str = r"
format: ess/13
system: rec
version: v1
domain: rec.calls
types:
  - {name: rec.calls.CallId, kind: newtype, of: Uuid}
entities:
  - name: rec.calls.Call
    identity: {name: call_id, type: rec.calls.CallId}
    fields:
      - {name: recording, type: Boolean}
      - {name: archived, type: Boolean}
      - {name: minutes, type: Integer}
    lifecycle:
      initial: Live
      states: [Live, Ended]
      terminal: [Ended]
      transitions:
        - {name: end, from: [Live], to: Ended}
events:
  - name: rec.calls.CallStarted
    fields:
      - {name: call_id, type: rec.calls.CallId}
  - name: rec.calls.Changed
    fields: []
errors:
  - name: rec.calls.TooLong
    summary: Too long.
    fields: []
views:
  - name: rec.calls.Calls
    source: rec.calls.Call
    consistency: read_your_writes
    fields:
      - {name: call_id, type: rec.calls.CallId}
      - {name: state, type: rec.calls.Call.State}
      - {name: recording, type: Boolean}
      - {name: archived, type: Boolean}
      - {name: minutes, type: Integer}
commands:
  - name: rec.calls.StartCall
    input:
      - {name: recording, type: Boolean}
      - {name: minutes, type: Integer}
    outcomes:
      - name: started
        creates: rec.calls.Call
        instance: call_id
        sets: {recording: input.recording, minutes: input.minutes, archived: true}
        emits: [rec.calls.CallStarted]
        payload:
          rec.calls.CallStarted: {call_id: {generated: true}}
  - name: rec.calls.EndCall
    input:
      - {name: call_id, type: rec.calls.CallId}
    outcomes:
      - name: ended
        moves: rec.calls.Call.end
        instance: call_id
        emits: [rec.calls.Changed]
BRANCH
";

const GUARDED_RECORD: &str = r"  - name: rec.calls.Record
    input:
      - {name: call_id, type: rec.calls.CallId}
    outcomes:
      - name: too-long
        when_subject:
          predicate:
            all:
              - minutes > 60
        error: rec.calls.TooLong
      - name: recorded
        updates: rec.calls.Call
        instance: call_id
        sets: {recording: true, archived: true}
        emits: [rec.calls.Changed]
";

const PLAIN_RECORD: &str = r"  - name: rec.calls.Record
    input:
      - {name: call_id, type: rec.calls.CallId}
    outcomes:
      - name: recorded
        updates: rec.calls.Call
        instance: call_id
        sets: {recording: true, archived: true}
        emits: [rec.calls.Changed]
";

/// `Record` writes `recording: true` and `archived: true`; every arrangement holds `archived: true`,
/// but `recording` can be arranged `false`. A row where `recording` differs exists and must be the
/// one witnessed, on the plain path and on the stored-field-guard path alike — one unchangeable
/// write must not make the search give up on the one it can change.
#[test]
fn adversary2_a_changeable_literal_write_is_changed_beside_one_no_row_changes() {
    let mut unchanged = Vec::new();
    for (path, branch) in [
        ("plain", PLAIN_RECORD),
        ("stored-field guard", GUARDED_RECORD),
    ] {
        let synthesis = synthesize(&compiled(&RECORDED.replace("BRANCH", branch)));
        let arranged: Vec<Option<Node>> = invocations(&synthesis)
            .into_iter()
            .filter(|invocation| {
                invocation.scenario == "rec.calls.Record/outcome/recorded"
                    && invocation.command == "rec.calls.Record"
                    && invocation.expected == "recorded"
            })
            .map(|invocation| {
                invocation
                    .arranged
                    .get("rec.calls.StartCall")
                    .and_then(|create| create.get("recording").cloned())
            })
            .collect();
        assert!(
            !arranged.is_empty(),
            "{path}: recorded is invoked: {:?}",
            refusals(&synthesis)
        );
        for recording in arranged {
            if recording != Some(Node::Bool(false)) {
                unchanged.push(format!(
                    "{path}: arranges `recording: {recording:?}` before writing `true`"
                ));
            }
        }
    }
    assert!(unchanged.is_empty(), "{unchanged:#?}");
}

// ---- backward compatibility: the committed generated suites ----------------------------------------

fn example(name: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("`{name}` validates:\n{errors}"));
    ess_compiler::resolve::compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("`{name}` resolves:\n{diagnostics}"))
}

/// Every generated scenario of each committed suite is still what synthesis produces from its
/// example, so the change moved no committed artifact and dropped no scenario from one.
#[test]
fn adversary2_committed_generated_suites_are_still_what_synthesis_produces() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../suites/generated");
    let mut drift = Vec::new();
    for name in ["billing", "gatepass", "oracle-fixture"] {
        let text = std::fs::read_to_string(root.join(name).join("suite.json"))
            .expect("the committed suite is readable");
        let committed = ConformanceSuite::from_json(&text).expect("the committed suite parses");
        let fresh = synthesize(&example(name)).suite;
        let generated: BTreeMap<&ScenarioId, _> = committed
            .scenarios
            .iter()
            .filter(|(id, _)| !matches!(id, ScenarioId::Authored { .. }))
            .collect();
        let now: BTreeMap<&ScenarioId, _> = fresh.scenarios.iter().collect();
        eprintln!(
            "{name}: committed generated {} scenarios, synthesized now {}",
            generated.len(),
            now.len()
        );
        for (id, scenario) in &generated {
            match now.get(id) {
                None => drift.push(format!("{name}: `{id}` no longer synthesized")),
                Some(fresh) if fresh != scenario => {
                    drift.push(format!("{name}: `{id}` changed"));
                }
                Some(_) => {}
            }
        }
        for id in now.keys() {
            if !generated.contains_key(id) {
                drift.push(format!("{name}: `{id}` is new"));
            }
        }
    }
    assert!(drift.is_empty(), "{drift:#?}");
}
