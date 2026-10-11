//! beyond10x/ess#521: a `when_related:` guard whose predicate reads a member of a struct input
//! (`team != input.key.team`) is honoured where that struct input is a captured instance, as it is
//! where the struct is a literal. `Reassign` addresses the seat by the key an earlier `Assign`
//! captured; the person it sends must be in that key's team for `reassigned`, and in another for
//! `elsewhere`. `Assign`, whose key is a literal, keeps the scenarios it had byte for byte.
use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{interpret::Interpreted, report::Status, AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;
use sha2::{Digest, Sha256};

const DESK: &str = include_str!("fixtures/related-guard-captured-struct.yaml");

/// The scenarios whose key is a literal, with the digest of their bytes as synthesis wrote them
/// before beyond10x/ess#521 was fixed (`1d2ce5dc03`).
const LITERAL: &[(&str, &str)] = &[
    (
        "demo.desk.Assign/outcome/assigned",
        "358fa4e2eab28628e6fd3f15f45a5b32312061f681bd0191af80be7d39ba3ce1",
    ),
    (
        "demo.desk.Assign/outcome/elsewhere",
        "d6b5465a0903767dc54530ffbeaa68377647d43bbbbc50de8238518e39385235",
    ),
    (
        "demo.desk.Assign/outcome/nobody",
        "18fba0207aff41e574c77d392ce32a331bcf8c03ce67f6aef62b01bbc589f054",
    ),
    (
        "demo.desk.Join/outcome/joined",
        "5215d0331fbd9e4483711d61c33707a3a30630abbe4b0565ee5b316e2653a34d",
    ),
];

fn model() -> EssIr {
    let spec =
        Specification::assemble([(Source::new("desk.yaml"), RawSpecFile::parse(DESK).unwrap())])
            .unwrap_or_else(|error| panic!("{error}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// Every synthesized scenario, by id, as the JSON a suite stores it as.
fn scenarios() -> BTreeMap<String, Value> {
    ess_conformance::synthesize(&model())
        .suite
        .scenarios
        .iter()
        .map(|(id, scenario)| (id.to_string(), serde_json::to_value(scenario).unwrap()))
        .collect()
}

fn digest(value: &Value) -> String {
    Sha256::digest(serde_json::to_string(value).unwrap().as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            write!(hex, "{byte:02x}").unwrap();
            hex
        })
}

/// What a step's input field holds once the instances the scenario captured are read as the
/// values they were captured from.
fn resolve(value: &Value, captured: &BTreeMap<String, Value>) -> Value {
    match value["kind"].as_str() {
        Some("literal") => value["value"].clone(),
        Some("instance") => captured
            .get(value["instance"].as_str().unwrap())
            .unwrap_or_else(|| panic!("{value} names no captured instance"))
            .clone(),
        other => panic!("an input value of kind {other:?}: {value}"),
    }
}

/// The branch the model answers a `Join`-populated desk with for one `Assign` or `Reassign`, read
/// off the guards in declaration order: no person, a person in another team than the key's, or
/// the accepting branch.
fn expected(
    command: &str,
    person: &Value,
    key: &Value,
    teams: &BTreeMap<String, Value>,
) -> &'static str {
    let accepting = if command == "demo.desk.Assign" {
        "assigned"
    } else {
        "reassigned"
    };
    match teams.get(person.as_str().unwrap()) {
        None => "nobody",
        Some(team) if *team != key["team"] => "elsewhere",
        Some(_) => accepting,
    }
}

/// Every `Assign` and `Reassign` a scenario sends, with whether its key was a captured instance,
/// the branch the scenario expects and the branch the model answers.
fn sends(steps: &[Value]) -> Vec<(String, bool, String, &'static str)> {
    let mut captured: BTreeMap<String, Value> = BTreeMap::new();
    let mut teams: BTreeMap<String, Value> = BTreeMap::new();
    let mut last: Option<BTreeMap<String, Value>> = None;
    let mut out = Vec::new();
    for (at, step) in steps.iter().enumerate() {
        match step["step"].as_str().unwrap() {
            "execute_command" => {
                let command = step["command"].as_str().unwrap();
                let input = step["input"].as_object().unwrap();
                let sent: BTreeMap<String, Value> = input
                    .iter()
                    .map(|(name, value)| (name.clone(), resolve(value, &captured)))
                    .collect();
                if command == "demo.desk.Join" {
                    teams.insert(
                        sent["person_id"].as_str().unwrap().to_owned(),
                        sent["team"].clone(),
                    );
                } else {
                    let taken = steps[at + 1]["outcome"]["outcome"]
                        .as_str()
                        .unwrap_or_else(|| panic!("no outcome after {step}"))
                        .to_owned();
                    out.push((
                        command.to_owned(),
                        input["key"]["kind"] == "instance",
                        taken,
                        expected(command, &sent["person_id"], &sent["key"], &teams),
                    ));
                }
                last = Some(sent);
            }
            "capture_instance" => {
                let field = step["field"].as_str().unwrap();
                let sent = last.as_ref().expect("a capture after a send");
                captured.insert(
                    step["instance"].as_str().unwrap().to_owned(),
                    sent[field].clone(),
                );
            }
            _ => {}
        }
    }
    out
}

/// The acceptance's first half: every send whose key is a captured instance expects the branch the
/// related row's team decides — the refusal where it differs from the captured key's team — and
/// such sends exist, for both the refusal and the accepting branch.
#[test]
fn a_captured_struct_key_expects_the_branch_the_related_row_decides() {
    let mut seen = BTreeMap::new();
    for (id, scenario) in scenarios() {
        let steps = scenario["steps"].as_array().unwrap();
        for (command, captured, taken, model) in sends(steps) {
            assert_eq!(
                taken, model,
                "{id}: {command} expects `{taken}`, the model answers `{model}`"
            );
            if captured {
                *seen.entry(taken).or_insert(0usize) += 1;
            }
        }
    }
    for branch in ["elsewhere", "reassigned"] {
        assert!(
            seen.get(branch).is_some_and(|count| *count > 0),
            "no send of a captured key expects `{branch}`: {seen:?}"
        );
    }
}

/// The interpreted target passes the synthesized suite, every scenario of it.
#[test]
fn the_interpreted_target_passes_the_suite() {
    let ir = model();
    let suite = ess_conformance::synthesize(&ir).suite;
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let report = Runner::for_suite(&suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    assert_eq!(report.scenarios.len(), suite.scenarios.len());
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|case| case.status != Status::Passed)
        .map(|case| format!("{:?}: {:?}", case.scenario, case.status))
        .collect();
    assert_eq!(failed, Vec::<String>::new());
}

/// The acceptance's second half: the scenarios whose key is a literal keep their bytes.
#[test]
fn a_literal_struct_key_keeps_its_scenarios_byte_for_byte() {
    let here = scenarios();
    let moved: Vec<String> = LITERAL
        .iter()
        .filter_map(|(id, base)| {
            let now = here.get(*id).map_or_else(|| "missing".to_owned(), digest);
            (now != *base).then(|| format!("{id}\n  base {base}\n  here {now}"))
        })
        .collect();
    assert_eq!(moved, Vec::<String>::new());
}
