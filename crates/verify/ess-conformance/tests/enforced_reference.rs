//! A stored reference whose row cannot be missing needs no `exists: false` branch (`ess/30`,
//! beyond10x/ess#509, `docs/design/generic-language-features.md` § G2, "Enforced references"):
//! synthesis witnesses the reading command's branches over the present row it arranges through
//! the writer that checks it, and the interpreted model passes them.

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner, ScenarioStep};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// Ports name the node they were added to; adding a port refuses a missing node, and no command
/// removes a node, so renaming a port declares no missing-node branch.
const BOARD: &str = include_str!("fixtures/enforced-reference.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("board.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn suite(model: &EssIr) -> ConformanceSuite {
    let synthesis = ess_conformance::synthesize::synthesize(model);
    assert!(
        synthesis.refusals.is_empty(),
        "nothing is refused: {:#?}",
        synthesis.refusals
    );
    synthesis.suite
}

/// The commands `id`'s scenario sends, in order.
fn sent(suite: &ConformanceSuite, id: &str) -> Vec<String> {
    let (_, scenario) = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| {
            panic!(
                "no scenario {id} among {:?}",
                suite
                    .scenarios
                    .keys()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        });
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_reading_commands_branches_are_witnessed_over_a_checked_row() {
    let model = ir(BOARD);
    let suite = suite(&model);
    for id in [
        "integrity.board.Rename/outcome/node-sealed",
        "integrity.board.Rename/outcome/renamed",
    ] {
        let sent = sent(&suite, id);
        assert!(
            sent.iter()
                .any(|command| command == "integrity.board.AddPort"),
            "{id} arranges the port through the writer that checks the node: {sent:?}"
        );
        assert_eq!(
            sent.last().map(String::as_str),
            Some("integrity.board.Rename")
        );
    }
    assert!(
        !suite.scenarios.keys().any(|key| key
            .to_string()
            .starts_with("integrity.board.Rename/outcome/no-")),
        "no scenario sends a port whose node is missing"
    );
    let admitted = AdmittedSuite::from_suite(&suite).expect("the suite is admitted");
    let run = Runner::for_suite(admitted.suite()).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(model),
    );
    assert_ne!(run.scenarios.len(), 0);
    let failed: Vec<_> = run
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

/// The scenarios of `BOARD`'s suite that fail against the interpreted `target`, by id.
fn failing_on(target: &str) -> Vec<String> {
    let suite = suite(&ir(BOARD));
    let admitted = AdmittedSuite::from_suite(&suite).expect("the suite is admitted");
    let run = Runner::for_suite(admitted.suite()).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(ir(target)),
    );
    run.scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| result.scenario.to_string())
        .collect()
}

/// Acceptance 3 of `story:enforced-reference-needs-no-missing-target-branch`: the referenced row is
/// read again after every accepting branch that writes ports or nodes, so a target that removes it
/// fails the scenario run after the branch that removes it.
#[test]
fn a_target_that_deletes_the_referenced_row_fails() {
    // The target removes a port's node when the port is renamed; it answers the node it removed
    // as missing, which the specification it is held to never does.
    let target = BOARD
        .replacen(
            "      - name: node-sealed\n",
            "      - name: no-node\n        when_related: {via: node_id, exists: false}\n        \
             error: integrity.board.NodeNotFound\n      - name: node-sealed\n",
            1,
        )
        .replacen(
            "        sets: {name: input.name}\n",
            "        sets: {name: input.name}\n        affects:\n          - entity: \
             integrity.board.Node\n            where: node_id == subject.node_id\n            \
             deletes: integrity.board.Node\n",
            1,
        );
    assert_ne!(target, BOARD);
    let failed = failing_on(&target);
    assert!(
        !failed.is_empty(),
        "the deleting target passes every scenario"
    );
    assert!(
        failed
            .iter()
            .any(|id| id == "integrity.board.Port/invariant/after/integrity.board.Rename/renamed"),
        "the scenario reading the node again after the rename fails: {failed:?}"
    );
}

/// The views `id`'s scenario requires something of, in order, each with the expectation's kind.
fn required(suite: &ConformanceSuite, id: &str) -> Vec<(String, String)> {
    let (_, scenario) = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .unwrap_or_else(|| panic!("no scenario {id}"));
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExpectView {
                view, expectation, ..
            }
            | ScenarioStep::EventuallyView {
                view, expectation, ..
            } => {
                let kind = serde_json::to_value(expectation).expect("an expectation serialises")
                    ["expect"]
                    .as_str()
                    .unwrap_or_default()
                    .to_owned();
                Some((view.to_string(), kind))
            }
            _ => None,
        })
        .collect()
}

#[test]
fn the_node_is_read_again_after_every_branch_writing_a_port_or_a_node() {
    let suite = suite(&ir(BOARD));
    let integrity: Vec<String> = suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .filter(|id| id.starts_with("integrity.board.Port/invariant/after/"))
        .collect();
    assert_eq!(
        integrity,
        [
            "integrity.board.Port/invariant/after/integrity.board.AddNode/added",
            "integrity.board.Port/invariant/after/integrity.board.AddPort/added",
            "integrity.board.Port/invariant/after/integrity.board.Rename/renamed",
            "integrity.board.Port/invariant/after/integrity.board.Seal/sealed",
        ],
        "one scenario per accepting branch writing a port or a node, none for a refusal"
    );
    for id in &integrity {
        assert_eq!(
            required(&suite, id).last(),
            Some(&("integrity.board.Nodes".to_owned(), "contains".to_owned())),
            "{id} ends reading the node a port names"
        );
    }
    // Sealing a node no arranged port names: the scenario points a port at it first.
    let sent = sent(
        &suite,
        "integrity.board.Port/invariant/after/integrity.board.Seal/sealed",
    );
    assert_eq!(
        sent,
        [
            "integrity.board.AddNode",
            "integrity.board.AddPort",
            "integrity.board.Seal"
        ]
    );
}

#[test]
fn declared_invariants_and_the_node_read_share_one_scenario() {
    // A port also declares an invariant: the scenario `invariants` files for each port branch
    // gains the node read, and stays one scenario under one id.
    let board = BOARD.replacen(
        "    relations:\n      - {name: node, kind: references",
        "    invariants: ['name != \"none\"']\n    relations:\n      - {name: node, kind: references",
        1,
    );
    assert_ne!(board, BOARD);
    let model = ir(&board);
    let suite = suite(&model);
    for branch in ["AddPort/added", "Rename/renamed"] {
        let id = format!("integrity.board.Port/invariant/after/integrity.board.{branch}");
        let required = required(&suite, &id);
        assert!(
            required.contains(&("integrity.board.Ports".to_owned(), "satisfies".to_owned())),
            "{id} still reads the declared invariant: {required:?}"
        );
        assert_eq!(
            required.last(),
            Some(&("integrity.board.Nodes".to_owned(), "contains".to_owned())),
            "{id} ends reading the node: {required:?}"
        );
    }
    let admitted = AdmittedSuite::from_suite(&suite).expect("the suite is admitted");
    let run = Runner::for_suite(admitted.suite()).run_admitted(
        &admitted,
        &ess_conformance::interpret::Interpreted::for_model(model),
    );
    let failed: Vec<_> = run
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}
