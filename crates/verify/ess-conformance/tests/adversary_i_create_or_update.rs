//! Adversary pass 1 on unit I of the 0.70.0 wave (beyond10x/ess#641): a create-or-update command
//! beside a related-row refusal, attacked on the interpreter with the phases exchanged, under every
//! declaration order of its branches, with a second related guard, an `Optional` related input, a
//! struct identity whose member a related predicate reads, a later command reading the stored
//! reference, at the lowest admitting format, and for determinism.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::command::precedence::{with_phase_order, Phase};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const BIND_MODEL: &str = include_str!("fixtures/create-or-update-beside-related-refusal.yaml");
const BRANCHES: [&str; 4] = ["bound", "no-agent", "rebound", "already-bound"];

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}\n{text}"))
}

fn validates(text: &str) -> Result<(), String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("model.yaml"), raw)])
        .map(|_| ())
        .map_err(|errors| errors.to_string())
}

fn statuses(model: EssIr, suite: &ConformanceSuite) -> BTreeMap<String, Status> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// Synthesis refuses nothing, files `branches` of `command`, and every scenario passes.
fn every_branch_passes(label: &str, text: &str, command: &str, branches: &[&str]) {
    let model = ir(text);
    let synthesis = synthesize(&model);
    let refused: Vec<String> = synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{:?}: {:?}", refusal.scenario, refusal.cause))
        .collect();
    let ids: Vec<String> = synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    let missing: Vec<String> = branches
        .iter()
        .map(|branch| format!("{command}/outcome/{branch}"))
        .filter(|id| !ids.contains(id))
        .collect();
    let failed: Vec<String> = statuses(model, &synthesis.suite)
        .into_iter()
        .filter(|(_, status)| *status != Status::Passed)
        .map(|(id, status)| format!("{id}: {status:?}"))
        .collect();
    assert!(
        refused.is_empty() && missing.is_empty() && failed.is_empty(),
        "{label}:\nrefused {refused:#?}\nmissing {missing:#?}\nfailed {failed:#?}"
    );
}

fn swap(a: Phase, b: Phase) -> [Phase; 8] {
    Phase::PRECEDENCE.map(|phase| {
        if phase == a {
            b
        } else if phase == b {
            a
        } else {
            phase
        }
    })
}

/// The suite synthesized under the one rule, run on the interpreter reading present-related before
/// existence (and so before held state): `already-bound` resends a stored agent no row carries, so
/// the exchanged order answers `no-agent` there. `bound` is not asserted: the interpreter's #282
/// preflight (`subject_refusals_before_present_related`) answers the unknown seat whatever the
/// order, which the generated faulty control in `ess-synth` covers instead.
#[test]
fn the_interpreter_with_present_related_before_existence_fails_already_bound() {
    let model = ir(BIND_MODEL);
    let suite = synthesize(&model).suite;
    let exchanged = with_phase_order(swap(Phase::Existence, Phase::PresentRelated), || {
        statuses(model.clone(), &suite)
    });
    assert_ne!(
        exchanged.get("demo.desk.Bind/outcome/already-bound"),
        Some(&Status::Passed),
        "{exchanged:#?}"
    );
}

/// The four `Bind` branches in every declaration order validation admits: precedence is the
/// plan's, not the file's. Orders where declaration and precedence disagree are refused by
/// validation and skipped; at least the fixture's order and one other must remain.
#[test]
fn every_declaration_order_of_the_four_branches_synthesizes_and_passes() {
    let start = BIND_MODEL
        .find("      - name: already-bound\n")
        .expect("fixture");
    let end = BIND_MODEL.find("views:\n").expect("fixture");
    let body = &BIND_MODEL[start..end];
    let mut blocks: Vec<String> = body
        .split("      - name: ")
        .filter(|block| !block.is_empty())
        .map(|block| format!("      - name: {block}"))
        .collect();
    assert_eq!(blocks.len(), 4, "{blocks:#?}");
    blocks.sort();
    let mut orders = Vec::new();
    permutations(&mut blocks, 0, &mut orders);
    assert_eq!(orders.len(), 24);
    let mut admitted = 0;
    for order in orders {
        let text = format!(
            "{}{}{}",
            &BIND_MODEL[..start],
            order.concat(),
            &BIND_MODEL[end..]
        );
        let names: Vec<&str> = order
            .iter()
            .map(|block| block["      - name: ".len()..].lines().next().unwrap_or(""))
            .collect();
        if validates(&text).is_err() {
            continue;
        }
        admitted += 1;
        every_branch_passes(&format!("{names:?}"), &text, "demo.desk.Bind", &BRANCHES);
    }
    assert!(admitted >= 2, "validation admits {admitted} orders");
}

fn permutations(items: &mut Vec<String>, at: usize, out: &mut Vec<Vec<String>>) {
    if at == items.len() {
        out.push(items.clone());
        return;
    }
    for index in at..items.len() {
        items.swap(at, index);
        permutations(items, at + 1, out);
        items.swap(at, index);
    }
}

/// A second related guard over the same input: a present agent that is retired is refused too.
#[test]
fn a_second_related_predicate_over_the_same_agent() {
    let text = BIND_MODEL
        .replace(
            "        sets: {label: input.label}",
            "        sets: {label: input.label, retired: input.retired}",
        )
        .replace(
            "        error: demo.desk.NoAgent\n",
            "        error: demo.desk.NoAgent\n      - name: agent-retired\n        when_related: {via: input.agent_id, predicate: retired == true}\n        error: demo.desk.NoAgent\n",
        )
        .replace(
            "      - {name: label, type: String}\n",
            "      - {name: label, type: String}\n      - {name: retired, type: Boolean}\n",
        );
    assert!(text.contains("agent-retired"), "fixture moved");
    every_branch_passes(
        "second related predicate",
        &text,
        "demo.desk.Bind",
        &[
            "bound",
            "no-agent",
            "agent-retired",
            "rebound",
            "already-bound",
        ],
    );
}

const ALREADY_BOUND: &str = "      - name: already-bound\n        when_subject:\n          predicate: agent_id == input.agent_id\n        preserves: demo.desk.Binding\n        instance: seat_id\n";

/// The fixture's four branches with a second related refusal, a desk no row carries, named
/// through another input.
fn two_related_rows() -> String {
    let text = BIND_MODEL
        .replace(
            "  - {name: demo.desk.SeatId, kind: newtype, of: String}\n",
            "  - {name: demo.desk.SeatId, kind: newtype, of: String}\n  - {name: demo.desk.DeskId, kind: newtype, of: String}\n",
        )
        .replace(
            "  - name: demo.desk.Binding\n",
            "  - name: demo.desk.Desk\n    identity: {name: desk_id, type: demo.desk.DeskId}\n    fields: []\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\n  - name: demo.desk.Binding\n",
        )
        .replace(
            "  - {name: demo.desk.NoAgent, summary: No agent carries the identity., fields: []}\n",
            "  - {name: demo.desk.NoAgent, summary: No agent carries the identity., fields: []}\n  - {name: demo.desk.NoDesk, summary: No desk carries the identity., fields: []}\n",
        )
        .replace(
            "  - name: demo.desk.SeatBound\n",
            "  - name: demo.desk.DeskOpened\n    fields: [{name: desk_id, type: demo.desk.DeskId}]\n  - name: demo.desk.SeatBound\n",
        )
        .replace(
            "    may: [demo.desk.AddAgent, demo.desk.Bind]",
            "    may: [demo.desk.AddAgent, demo.desk.OpenDesk, demo.desk.Bind]",
        )
        .replace(
            "  - name: demo.desk.Bind\n    input:\n      - {name: seat_id, type: demo.desk.SeatId}\n      - {name: agent_id, type: demo.desk.AgentId}\n",
            "  - name: demo.desk.OpenDesk\n    input:\n      - {name: desk_id, type: demo.desk.DeskId}\n    outcomes:\n      - name: opened\n        creates: demo.desk.Desk\n        instance: desk_id\n        emits: [demo.desk.DeskOpened]\n        payload: {demo.desk.DeskOpened: {desk_id: input.desk_id}}\n  - name: demo.desk.Bind\n    input:\n      - {name: seat_id, type: demo.desk.SeatId}\n      - {name: agent_id, type: demo.desk.AgentId}\n      - {name: desk_id, type: demo.desk.DeskId}\n",
        )
        .replace(
            "        error: demo.desk.NoAgent\n",
            "        error: demo.desk.NoAgent\n      - name: no-desk\n        when_related: {via: input.desk_id, exists: false}\n        error: demo.desk.NoDesk\n",
        )
        .replace(
            "views:\n",
            "views:\n  - name: demo.desk.Desks\n    source: demo.desk.Desk\n    consistency: read_your_writes\n    fields:\n      - {name: desk_id, type: demo.desk.DeskId}\n",
        );
    assert!(text.contains("no-desk"), "fixture moved");
    assert!(text.contains(ALREADY_BOUND), "fixture moved");
    text
}

/// `already-bound` (a held seat) and `bound` (a seat no row carries) never both hold, and the
/// fixture with one related refusal validates. With a second related refusal over another input,
/// validation reports them as selected together. This asserts today's refusal; when the story
/// lands it must validate.
#[test]
fn a_create_or_update_beside_two_related_refusals_validates() {
    let text = two_related_rows();
    assert!(
        validates(&text).is_err(),
        "today validation reports `already-bound` and `bound` as selected together; once this \
         validates, assert `is_ok()`: story:two-input-named-related-refusals-beside-a-held-state-branch-validate"
    );
}

/// The same two related refusals without the held-state branch: every branch synthesizes and
/// passes.
#[test]
fn a_second_related_row_through_another_input() {
    let text = two_related_rows().replace(ALREADY_BOUND, "");
    every_branch_passes(
        "second related row",
        &text,
        "demo.desk.Bind",
        &["bound", "no-agent", "no-desk", "rebound"],
    );
}

/// The related input is `Optional`: an absent agent names no row and is not looked up.
#[test]
fn an_optional_agent() {
    let text = BIND_MODEL
        .replace(
            "      - {name: agent_id, type: demo.desk.AgentId}\n    relations:\n      - {name: agent, kind: references, target: demo.desk.Agent, cardinality: one, via: agent_id}\n",
            "      - {name: agent_id, type: Optional<demo.desk.AgentId>}\n",
        )
        .replace(
            "      - {name: seat_id, type: demo.desk.SeatId}\n      - {name: agent_id, type: demo.desk.AgentId}\n    outcomes:",
            "      - {name: seat_id, type: demo.desk.SeatId}\n      - {name: agent_id, type: Optional<demo.desk.AgentId>}\n    outcomes:",
        )
        .replace(
            "      - {name: state, type: demo.desk.Binding.State}\n      - {name: agent_id, type: demo.desk.AgentId}\n",
            "      - {name: state, type: demo.desk.Binding.State}\n      - {name: agent_id, type: Optional<demo.desk.AgentId>}\n",
        );
    assert_eq!(text.matches("Optional<").count(), 3, "fixture moved");
    every_branch_passes("optional agent", &text, "demo.desk.Bind", &BRANCHES);
}

/// The seat's identity is a struct, and a related predicate reads a member of it.
#[test]
fn a_struct_identity_whose_member_a_related_predicate_reads() {
    let text = BIND_MODEL
        .replace(
            "  - {name: demo.desk.SeatId, kind: newtype, of: String}\n",
            "  - name: demo.desk.SeatKey\n    kind: struct\n    fields:\n      - {name: team, type: String}\n      - {name: seat, type: String}\n",
        )
        .replace(
            "      - {name: label, type: String}\n",
            "      - {name: label, type: String}\n      - {name: team, type: String}\n",
        )
        .replace(
            "        sets: {label: input.label}",
            "        sets: {label: input.label, team: input.team}",
        )
        .replace("seat_id, type: demo.desk.SeatId", "key, type: demo.desk.SeatKey")
        .replace("instance: seat_id", "instance: key")
        .replace("{seat_id: input.seat_id}", "{key: input.key}")
        .replace(
            "        error: demo.desk.NoAgent\n",
            "        error: demo.desk.NoAgent\n      - name: elsewhere\n        when_related: {via: input.agent_id, predicate: team != input.key.team}\n        error: demo.desk.NoAgent\n",
        );
    assert!(!text.contains("seat_id"), "fixture moved:\n{text}");
    every_branch_passes(
        "struct identity",
        &text,
        "demo.desk.Bind",
        &["bound", "no-agent", "elsewhere", "rebound", "already-bound"],
    );
}

/// A later command reads the agent a stored seat names: its accepting branch needs a seat whose
/// agent exists, which only `Bind` stores, and its `exists: false` branch a seat `bound` stored for
/// an agent no row carries, as the unit's own `no-agent` witness does.
#[test]
fn a_later_command_reading_the_stored_agent() {
    let text = BIND_MODEL
        .replace(
            "  - {name: demo.desk.NoAgent, summary: No agent carries the identity., fields: []}\n",
            "  - {name: demo.desk.NoAgent, summary: No agent carries the identity., fields: []}\n  - {name: demo.desk.NoSeat, summary: No seat carries the identity., fields: []}\n",
        )
        .replace(
            "    may: [demo.desk.AddAgent, demo.desk.Bind]",
            "    may: [demo.desk.AddAgent, demo.desk.Bind, demo.desk.Ping]",
        )
        .replace(
            "views:\n",
            "  - name: demo.desk.Ping\n    input:\n      - {name: seat_id, type: demo.desk.SeatId}\n    outcomes:\n      - name: agent-gone\n        when_related: {via: agent_id, exists: false}\n        error: demo.desk.NoAgent\n      - name: pinged\n        preserves: demo.desk.Binding\n        instance: seat_id\nviews:\n",
        );
    // Today synthesis calls `agent-gone` unreachable. Once the story lands, replace this with
    // `every_branch_passes("later stored read", &text, "demo.desk.Ping", &["agent-gone", "pinged"])`.
    let refused: Vec<String> = synthesize(&ir(&text))
        .refusals
        .iter()
        .map(|refusal| format!("{:?}: {:?}", refusal.scenario, refusal.cause))
        .collect();
    assert!(
        refused.iter().any(|refusal| refusal.contains("agent-gone")),
        "today synthesis refuses Ping/agent-gone as unreachable, counting the create-or-update \
         creator as refusing a missing agent first: \
         story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present\n{refused:#?}"
    );
}

/// `ess/30` lifts the missing-row refusal of a stored reference where every writer of the field
/// "refuses first" an identity no row carries (`ess-domain` `command/reference_integrity.rs`).
/// Under the one rule `Bind/bound` answers an unknown seat at step 2, before `no-agent` at step 4,
/// and stores an agent no row carries (the unit's own `bound` and `no-agent` witnesses), so a
/// later command reading that stored agent with no `exists: false` branch must stay refused.
#[test]
fn a_stored_agent_written_by_the_creator_is_not_proven_present() {
    let text = BIND_MODEL
        .replace(
            "    may: [demo.desk.AddAgent, demo.desk.Bind]",
            "    may: [demo.desk.AddAgent, demo.desk.Bind, demo.desk.Ping]",
        )
        .replace(
            "views:\n",
            "  - name: demo.desk.Ping\n    input:\n      - {name: seat_id, type: demo.desk.SeatId}\n    outcomes:\n      - name: labelled\n        when_related: {via: agent_id, predicate: label == \"on\"}\n        preserves: demo.desk.Binding\n        instance: seat_id\n      - name: pinged\n        preserves: demo.desk.Binding\n        instance: seat_id\nviews:\n",
        );
    assert!(
        validates(&text.replace("format: ess/31", "format: ess/29")).is_err(),
        "below ess/30 the missing-row refusal stands"
    );
    // Today ess/31 accepts it; once the story lands this must be `is_err()`.
    assert!(
        validates(&text).is_ok(),
        "today ess/31 proves the stored agent present although `bound` stores one no row \
         carries; when it refuses, assert `is_err()`: \
         story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present"
    );
}

/// The lowest format admitting the shape (`ess/27`) synthesizes and passes every branch.
#[test]
fn every_branch_passes_at_ess27() {
    every_branch_passes(
        "ess/27",
        &BIND_MODEL.replace("format: ess/31", "format: ess/27"),
        "demo.desk.Bind",
        &BRANCHES,
    );
}

/// Synthesis of the shape is a function of the model: two runs, one result.
#[test]
fn synthesis_of_the_shape_is_deterministic() {
    let first: Synthesis = synthesize(&ir(BIND_MODEL));
    let second: Synthesis = synthesize(&ir(BIND_MODEL));
    assert_eq!(first, second);
}
