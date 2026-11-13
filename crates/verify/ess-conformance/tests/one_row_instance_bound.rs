//! The one-row check after an update reads the instance its scenario bound
//! (`story:the-one-row-check-reads-the-instance-its-scenario-bound`).
//!
//! `one_row` appends a snapshot of the identity's row to the updating branch's scenario. It named
//! the instance from a second run of the branch, which is not the instance the filed scenario's
//! own arrangement bound once the enum an input carries has more variants: the scenario then
//! ended in `ESS-CF-SUITE: no earlier step bound the instance`.
//!
//! * The roster model below, with its actor-kind enum at three and at four variants, synthesizes a
//!   suite no step of which reads an instance no earlier step binds, and the interpreter passes it
//!   whole.
//! * Moving a variant changes no scenario id.
//! * The same walk, before any run, over every synthesized scenario of every model the repository
//!   holds.
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::synthesize::synthesize;
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const ROSTER: &str = r"format: ess/33
system: demo
version: v1
domain: demo.roster
summary: Members join a team, leave it and join it again.
types:
  - {name: demo.roster.MemberName, kind: newtype, of: String}
  - {name: demo.roster.TeamName, kind: newtype, of: String}
  - {name: demo.roster.CallerKind, kind: enum, variants: [@VARIANTS@]}
  - name: demo.roster.Caller
    kind: struct
    fields:
      - {name: kind, type: demo.roster.CallerKind}
      - {name: member, type: Optional<demo.roster.MemberName>}
  - name: demo.roster.MemberKey
    kind: struct
    fields:
      - {name: member, type: demo.roster.MemberName}
      - {name: team, type: demo.roster.TeamName}
entities:
  - name: demo.roster.Member
    identity: {name: key, type: demo.roster.MemberKey}
    fields:
      - {name: joined_by, type: demo.roster.Caller}
    lifecycle:
      initial: Active
      states: [Active, Left]
      terminal: []
      transitions:
        - {name: leave, from: [Active], to: Left}
        - {name: rejoin, from: [Left], to: Active}
actors:
  - name: demo.roster.Clerk
    may: [demo.roster.JoinTeam, demo.roster.LeaveTeam, demo.roster.Members]
errors:
  - name: demo.roster.CallerNotHuman
    fields:
      - {name: caller, type: demo.roster.Caller}
  - name: demo.roster.AlreadyJoined
    fields:
      - {name: key, type: demo.roster.MemberKey}
  - name: demo.roster.NotJoined
    fields:
      - {name: key, type: demo.roster.MemberKey}
commands:
  - name: demo.roster.JoinTeam
    input:
      - {name: caller, type: demo.roster.Caller}
      - {name: key, type: demo.roster.MemberKey}
    outcomes:
      - name: caller-not-human
        when: caller.kind != Human
        error: demo.roster.CallerNotHuman
        payload:
          demo.roster.CallerNotHuman:
            caller: input.caller
      - name: joined
        unknown_instance: true
        creates: demo.roster.Member
        instance: key
        sets:
          joined_by: input.caller
        emits: [demo.roster.MemberJoined]
        payload:
          demo.roster.MemberJoined:
            key: input.key
      - name: rejoined
        moves: demo.roster.Member.rejoin
        instance: key
        sets:
          joined_by: input.caller
        emits: [demo.roster.MemberJoined]
        payload:
          demo.roster.MemberJoined:
            key: input.key
      - name: already-joined
        wrong_state: true
        error: demo.roster.AlreadyJoined
        payload:
          demo.roster.AlreadyJoined:
            key: input.key
  - name: demo.roster.LeaveTeam
    input:
      - {name: caller, type: demo.roster.Caller}
      - {name: key, type: demo.roster.MemberKey}
    outcomes:
      - name: caller-not-human
        when: caller.kind != Human
        error: demo.roster.CallerNotHuman
        payload:
          demo.roster.CallerNotHuman:
            caller: input.caller
      - name: left
        moves: demo.roster.Member.leave
        instance: key
        emits: [demo.roster.MemberLeft]
        payload:
          demo.roster.MemberLeft:
            key: input.key
      - name: not-joined
        wrong_state: true
        error: demo.roster.NotJoined
        payload:
          demo.roster.NotJoined:
            key: input.key
      - name: unknown-member
        unknown_instance: true
        error: demo.roster.NotJoined
        payload:
          demo.roster.NotJoined:
            key: input.key
events:
  - name: demo.roster.MemberJoined
    fields:
      - {name: key, type: demo.roster.MemberKey}
  - name: demo.roster.MemberLeft
    fields:
      - {name: key, type: demo.roster.MemberKey}
views:
  - name: demo.roster.Members
    source: demo.roster.Member
    consistency: read_your_writes
    fields:
      - {name: key, type: demo.roster.MemberKey}
      - {name: state, type: demo.roster.Member.State}
      - {name: joined_by, type: demo.roster.Caller}
";

fn roster(variants: &str) -> EssIr {
    ir(&ROSTER.replace("@VARIANTS@", variants))
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// The instance names every value of `json` refers to, in document order.
fn references(json: &serde_json::Value, found: &mut Vec<String>) {
    match json {
        serde_json::Value::Object(map) => {
            if map.get("kind").and_then(|kind| kind.as_str()) == Some("instance") {
                if let Some(name) = map.get("instance").and_then(|name| name.as_str()) {
                    found.push(name.to_owned());
                }
            }
            map.values().for_each(|value| references(value, found));
        }
        serde_json::Value::Array(items) => items.iter().for_each(|item| references(item, found)),
        _ => {}
    }
}

/// Each reference of a scenario to an instance no earlier step binds, as `<scenario> step <n>
/// (<kind>): <instance>`. A step binds its instance after its own references are read.
fn unbound_references(suite: &ConformanceSuite) -> Vec<String> {
    let mut unbound = Vec::new();
    for (id, scenario) in &suite.scenarios {
        let mut bound: BTreeSet<String> = BTreeSet::new();
        for (position, step) in scenario.steps.iter().enumerate() {
            let json = serde_json::to_value(step).expect("a step serializes");
            let mut named = Vec::new();
            references(&json, &mut named);
            for name in named {
                if !bound.contains(&name) {
                    unbound.push(format!(
                        "{id} step {} ({}): {name}",
                        position + 1,
                        json["step"].as_str().unwrap_or("?")
                    ));
                }
            }
            match step {
                ScenarioStep::CaptureInstance { instance, .. }
                | ScenarioStep::EstablishEntity { instance, .. } => {
                    bound.insert(instance.to_string());
                }
                ScenarioStep::ResolveFixtures { fixtures } => bound.extend(
                    fixtures
                        .fields
                        .iter()
                        .filter(|field| field.entity.is_some())
                        .map(|field| field.name.to_string()),
                ),
                _ => {}
            }
        }
    }
    unbound
}

fn not_passed(ir: EssIr, suite: &ConformanceSuite) -> Vec<String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    let results: BTreeMap<String, (Status, Vec<String>)> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| {
            let diagnostics = result
                .diagnostics()
                .map(|diagnostic| format!("{diagnostic:?}"))
                .collect();
            (result.scenario.to_string(), (result.status, diagnostics))
        })
        .collect();
    results
        .into_iter()
        .filter(|(_, (status, _))| *status != Status::Passed)
        .map(|(id, (status, diagnostics))| format!("{id}: {status:?} {diagnostics:?}"))
        .collect()
}

fn ids(ir: &EssIr) -> Vec<String> {
    synthesize(ir)
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn one_row_roster_three_variants_reads_only_bound_instances_and_passes() {
    let model = roster("Human, Agent, Daemon");
    let result = synthesize(&model);
    assert_eq!(unbound_references(&result.suite), Vec::<String>::new());
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}

#[test]
fn one_row_roster_four_variants_reads_only_bound_instances_and_passes() {
    let model = roster("Human, Agent, ControlPlane, Daemon");
    let result = synthesize(&model);
    assert_eq!(unbound_references(&result.suite), Vec::<String>::new());
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}

/// The roster with its one view limited to the members who are active, so no view without a filter
/// carries the one-row snapshot, and the instance the second run of `rejoined` names is one no step
/// of its scenario binds: the scenario is withdrawn rather than filed with that snapshot.
#[test]
fn one_row_filtered_only_view_withdraws_where_the_second_run_instance_is_unbound() {
    let text = ROSTER
        .replace("@VARIANTS@", "Human, Agent, ControlPlane, Daemon")
        .replace(
            "    consistency: read_your_writes\n",
            "    consistency: read_your_writes\n    filter: state == Active\n",
        );
    assert!(text.contains("filter: state == Active"));
    let model = ir(&text);
    let result = synthesize(&model);
    let rejoined = "demo.roster.JoinTeam/outcome/rejoined";
    assert!(
        result
            .suite
            .scenarios
            .keys()
            .all(|key| key.to_string() != rejoined),
        "`rejoined` is withdrawn"
    );
    assert!(
        result.refusals.iter().any(|refusal| refusal
            .scenario
            .as_ref()
            .is_some_and(|scenario| scenario.to_string() == rejoined)
            && format!("{:?}", refusal.cause).contains("one-row check")),
        "and refused for the one-row check"
    );
    assert_eq!(unbound_references(&result.suite), Vec::<String>::new());
    assert_eq!(not_passed(model, &result.suite), Vec::<String>::new());
}

#[test]
fn one_row_roster_variant_order_changes_no_scenario_id() {
    let last = ids(&roster("Human, Agent, Daemon, ControlPlane"));
    assert_eq!(ids(&roster("ControlPlane, Human, Agent, Daemon")), last);
    assert_eq!(ids(&roster("Human, Agent, ControlPlane, Daemon")), last);
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if path.is_dir() {
            if !["target", "node_modules", ".git", ".engineering"].contains(&name.as_str()) {
                walk(&path, out);
            }
        } else if path
            .extension()
            .is_some_and(|ext| ext == "yaml" || ext == "yml")
        {
            out.push(path);
        }
    }
}

/// Synthesis of this model takes minutes in a debug build; the suite bytes table excludes it too.
const UNAFFORDABLE: &str =
    "crates/generate/ess-synth/tests/fixtures/conditional-measures-generated.yaml";

fn compiled(documents: Vec<(Source, RawSpecFile)>, sources: &SourceMap) -> Option<EssIr> {
    let spec = Specification::assemble(documents).ok()?;
    compile(&spec, sources).ok()
}

/// Every model this tree holds that compiles, by label: each ESS source file alone, and each
/// directory holding a `system.yaml` as one system, as `synthesis_suite_bytes_table` reads them.
fn repository_models(root: &Path) -> Vec<(String, EssIr)> {
    let mut files = Vec::new();
    walk(root, &mut files);
    files.sort();
    let mut found = Vec::new();
    let mut systems = Vec::new();
    for path in &files {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        if !text.lines().any(|line| line.starts_with("format: ess/")) {
            continue;
        }
        if path.file_name().is_some_and(|name| name == "system.yaml") {
            systems.push(path.parent().unwrap().to_path_buf());
        }
        let label = path.strip_prefix(root).unwrap().display().to_string();
        if label == UNAFFORDABLE {
            continue;
        }
        let Ok(raw) = RawSpecFile::parse(&text) else {
            continue;
        };
        let mut sources = SourceMap::new();
        sources.insert(label.clone(), text);
        if let Some(ir) = compiled(vec![(Source::new(label.clone()), raw)], &sources) {
            found.push((label, ir));
        }
    }
    for dir in systems {
        let mut members = Vec::new();
        walk(&dir, &mut members);
        members.sort();
        let mut sources = SourceMap::new();
        let mut documents = Vec::new();
        let mut unparsed = false;
        for path in members {
            let text = std::fs::read_to_string(&path).unwrap();
            let label = path.strip_prefix(&dir).unwrap().display().to_string();
            match RawSpecFile::parse(&text) {
                Ok(raw) => {
                    sources.insert(label.clone(), text);
                    documents.push((Source::new(label), raw));
                }
                Err(_) => unparsed = true,
            }
        }
        if unparsed {
            continue;
        }
        let name = dir.strip_prefix(root).unwrap().display().to_string();
        if let Some(ir) = compiled(documents, &sources) {
            found.push((format!("system-dir:{name}"), ir));
        }
    }
    found
}

#[test]
fn one_row_no_repository_scenario_reads_an_instance_no_earlier_step_binds() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let models = repository_models(&root);
    assert!(
        models.len() > 100,
        "the walk reads the repository's models: {}",
        models.len()
    );
    let unbound: Vec<String> = models
        .iter()
        .flat_map(|(label, ir)| {
            unbound_references(&synthesize(ir).suite)
                .into_iter()
                .map(move |reference| format!("{label}: {reference}"))
        })
        .collect();
    assert_eq!(unbound, Vec::<String>::new());
}
