//! Adversary pass 1 for beyond10x/ess#229: synthesis of a `when_related` guard over the related
//! row's held `state` (`ess/20`), beyond the one shape the unit's fixture covers — a refusing state
//! reached only after two moves, `state in [..]`, `state` beside a stored field, and a related row
//! of the command's own entity.
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::Synthesis, ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const RELEASE: &str = include_str!("fixtures/related-guard-release.yaml");

const NOT_ACCEPTED_GUARD: &str =
    "        when_related: {via: input.candidate, predicate: state != Accepted}\n";

const PUBLISH: &str = "demo.release.PublishRelease";

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn synthesis(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("release.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    ess_conformance::synthesize::synthesize(&ir)
}

fn refusals_for(result: &Synthesis, id: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| format!("{refusal:?}"))
        .filter(|text| text.contains(id))
        .collect()
}

fn scenario<'a>(result: &'a Synthesis, id: &str) -> &'a ConformanceScenario {
    result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map_or_else(
            || {
                panic!(
                    "no scenario {id}\n scenarios: {:#?}\n refusals: {:#?}",
                    result
                        .suite
                        .scenarios
                        .keys()
                        .map(ToString::to_string)
                        .collect::<Vec<_>>(),
                    result
                        .refusals
                        .iter()
                        .map(|refusal| format!("{refusal:?}"))
                        .collect::<Vec<_>>()
                )
            },
            |(_, scenario)| scenario,
        )
}

/// The input of the last send of `command` in the scenario.
fn sending(scenario: &ConformanceScenario, command: &str) -> BTreeMap<String, ScenarioValue> {
    scenario
        .steps
        .iter()
        .rev()
        .find_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the scenario sends {command}"))
}

/// The state a `Candidates` view expectation of the scenario says `candidate` holds — the last
/// one, the state it holds when the command is sent.
fn observed_state(scenario: &ConformanceScenario, candidate: &ScenarioValue) -> Option<String> {
    let candidate = serde_json::to_value(candidate).unwrap();
    scenario
        .steps
        .iter()
        .filter_map(|step| {
            let step = serde_json::to_value(step).unwrap();
            let fields = &step["expectation"]["fields"];
            (step["step"] == "expect_view"
                && step["view"] == "demo.release.Candidates"
                && fields["candidate_id"] == candidate)
                .then(|| fields["state"]["value"].as_str().map(str::to_owned))
                .flatten()
        })
        .next_back()
}

/// Both sides are synthesized with no refusal, and the related row named by `via` is observed in
/// one of `refusing` before the refusal and in one of `accepting` before the success.
fn witnessed(
    text: &str,
    command: &str,
    via: &str,
    (refusal, refusing): (&str, &[&str]),
    (success, accepting): (&str, &[&str]),
) {
    let result = synthesis(text);
    for (outcome, states) in [(refusal, refusing), (success, accepting)] {
        let id = format!("{command}/outcome/{outcome}");
        let found = scenario(&result, &id);
        assert!(
            refusals_for(&result, &id).is_empty(),
            "{id}: {:#?}",
            refusals_for(&result, &id)
        );
        let input = sending(found, command);
        let named = input
            .get(via)
            .unwrap_or_else(|| panic!("{id}: no `{via}` in {input:?}"));
        let observed = observed_state(found, named);
        assert!(
            observed
                .as_deref()
                .is_some_and(|state| states.contains(&state)),
            "{id}: the related row is observed in {observed:?}, expected one of {states:?}"
        );
    }
}

/// The candidate gains a third state, `Withdrawn`, reached from `Accepted` only.
fn three_states() -> String {
    let text = replaced(
        RELEASE,
        "      states: [Proposed, Accepted]\n      terminal: [Accepted]\n      transitions:\n        - {name: accept, from: [Proposed], to: Accepted}\n",
        "      states: [Proposed, Accepted, Withdrawn]\n      terminal: [Withdrawn]\n      transitions:\n        - {name: accept, from: [Proposed], to: Accepted}\n        - {name: withdraw, from: [Accepted], to: Withdrawn}\n",
    );
    let text = replaced(
        &text,
        "      - demo.release.DraftRelease\n",
        "      - demo.release.DraftRelease\n      - demo.release.WithdrawCandidate\n",
    );
    let text = replaced(
        &text,
        "  - name: demo.release.CandidateAccepted\n",
        "  - name: demo.release.CandidateWithdrawn\n    fields: [{name: candidate_id, type: demo.release.CandidateId}]\n  - name: demo.release.CandidateAccepted\n",
    );
    replaced(
        &text,
        "  - name: demo.release.DraftRelease\n",
        "  - name: demo.release.WithdrawCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n    outcomes:\n      - name: withdrawn\n        moves: demo.release.Candidate.withdraw\n        instance: candidate_id\n        emits: [demo.release.CandidateWithdrawn]\n        payload: {demo.release.CandidateWithdrawn: {candidate_id: input.candidate_id}}\n  - name: demo.release.DraftRelease\n",
    )
}

fn guarded(text: &str, refusal: &str) -> String {
    replaced(
        text,
        NOT_ACCEPTED_GUARD,
        &format!(
            "        when_related:\n          via: input.candidate\n          predicate: {refusal}\n"
        ),
    )
}

#[test]
fn adversary_229_a_refusing_state_two_moves_deep_is_witnessed() {
    witnessed(
        &guarded(&three_states(), "state == Withdrawn"),
        PUBLISH,
        "candidate",
        ("not-accepted", &["Withdrawn"]),
        ("published", &["Proposed", "Accepted"]),
    );
}

#[test]
fn adversary_229_state_in_a_list_is_witnessed_on_both_sides() {
    witnessed(
        &guarded(&three_states(), "{state: {in: [Proposed, Withdrawn]}}"),
        PUBLISH,
        "candidate",
        ("not-accepted", &["Proposed", "Withdrawn"]),
        ("published", &["Accepted"]),
    );
}

#[test]
fn adversary_229_an_accepting_state_two_moves_deep_is_witnessed() {
    // Only a withdrawn candidate publishes: the success needs both moves.
    witnessed(
        &guarded(&three_states(), "state != Withdrawn"),
        PUBLISH,
        "candidate",
        ("not-accepted", &["Proposed", "Accepted"]),
        ("published", &["Withdrawn"]),
    );
}

/// The release fixture with a stored `channel` on the candidate, set from the proposing input and
/// shown by the candidates view.
fn with_channel() -> String {
    let text = replaced(
        RELEASE,
        "types:\n",
        "types:\n  - {name: demo.release.Channel, kind: enum, variants: [Stable, Beta]}\n",
    );
    let text = replaced(
        &text,
        "    fields: []\n",
        "    fields:\n      - {name: channel, type: demo.release.Channel}\n",
    );
    let text = replaced(
        &text,
        "    input: []\n",
        "    input:\n      - {name: channel, type: demo.release.Channel}\n",
    );
    let text = replaced(
        &text,
        "        payload: {demo.release.CandidateProposed: {candidate_id: {generated: true}}}\n",
        "        payload: {demo.release.CandidateProposed: {candidate_id: {generated: true}}}\n        sets: {channel: input.channel}\n",
    );
    replaced(
        &text,
        "      - {name: state, type: demo.release.Candidate.State}\n",
        "      - {name: state, type: demo.release.Candidate.State}\n      - {name: channel, type: demo.release.Channel}\n",
    )
}

#[test]
fn adversary_229_state_beside_a_stored_field_is_witnessed() {
    let text = guarded(
        &with_channel(),
        "{any: [state != Accepted, channel == Beta]}",
    );
    witnessed(
        &text,
        PUBLISH,
        "candidate",
        ("not-accepted", &["Proposed", "Accepted"]),
        ("published", &["Accepted"]),
    );
}

#[test]
fn adversary_229_state_and_a_stored_field_both_needed_to_refuse_is_witnessed() {
    // Refused only for an accepted beta candidate: the refusal needs the move and the field.
    let text = guarded(
        &with_channel(),
        "{all: [state == Accepted, channel == Beta]}",
    );
    witnessed(
        &text,
        PUBLISH,
        "candidate",
        ("not-accepted", &["Accepted"]),
        ("published", &["Proposed", "Accepted"]),
    );
}

/// `AcceptCandidate` takes a `parent` candidate that must not be accepted yet: the related row is a
/// row of the command's own entity, beside the subject it moves.
fn own_entity() -> String {
    replaced(
        RELEASE,
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n    outcomes:\n",
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n      - {name: parent, type: demo.release.CandidateId}\n    outcomes:\n      - name: no-parent\n        when_related: {via: input.parent, exists: false}\n        error: demo.release.NoCandidate\n      - name: parent-not-accepted\n        when_related: {via: input.parent, predicate: state == Accepted}\n        error: demo.release.CandidateNotAccepted\n",
    )
}

#[test]
fn adversary_229_state_of_a_related_row_of_the_commands_own_entity_is_witnessed() {
    let text = own_entity();
    let command = "demo.release.AcceptCandidate";
    witnessed(
        &text,
        command,
        "parent",
        ("parent-not-accepted", &["Accepted"]),
        ("accepted", &["Proposed"]),
    );
    // The parent and the subject are distinct rows: a target reading the subject's state (always
    // `Proposed` here, the move's `from`) in place of the parent's fails the refusal.
    let result = synthesis(&text);
    for outcome in ["parent-not-accepted", "accepted"] {
        let input = sending(
            scenario(&result, &format!("{command}/outcome/{outcome}")),
            command,
        );
        assert_ne!(
            input.get("parent"),
            input.get("candidate_id"),
            "{outcome}: {input:?}"
        );
    }
}

#[test]
fn adversary_229_the_unit_fixtures_success_survives_a_guarded_mover_of_the_related_row() {
    // The unit's own `PublishRelease`, unchanged; only the command that moves a candidate to
    // `Accepted` now also refuses for a missing or accepted parent. An accepted candidate is still
    // reachable (accept a candidate whose parent is a proposed one), so `published` is witnessable.
    witnessed(
        &own_entity(),
        PUBLISH,
        "candidate",
        ("not-accepted", &["Proposed"]),
        ("published", &["Accepted"]),
    );
}

#[test]
fn adversary_229_the_unit_fixtures_success_survives_a_mover_guarded_on_another_entity() {
    // `AcceptCandidate` refuses only when the release it names does not exist: the related row of
    // `PublishRelease` is moved by a command that itself reads a related row (of `Release`).
    let text = replaced(
        RELEASE,
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n    outcomes:\n",
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n      - {name: release, type: demo.release.ReleaseId}\n    outcomes:\n      - name: no-release\n        when_related: {via: input.release, exists: false}\n        error: demo.release.NoCandidate\n",
    );
    witnessed(
        &text,
        PUBLISH,
        "candidate",
        ("not-accepted", &["Proposed"]),
        ("published", &["Accepted"]),
    );
}

/// The candidates `AcceptCandidate` moves in the scenario, other than `named`.
fn accepted_besides(scenario: &ConformanceScenario, named: &ScenarioValue) -> usize {
    scenario
        .steps
        .iter()
        .filter(|step| {
            matches!(step, ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "demo.release.AcceptCandidate"
                    && input.get("candidate_id") != Some(named))
        })
        .count()
}

#[test]
fn adversary_229_the_refusal_is_sent_between_accepted_decoys_as_the_documents_say() {
    // predicates.md: "the refusal is sent for a candidate still `Proposed` between two `Accepted`
    // decoys, the default for an `Accepted` one between two `Proposed` decoys, so a target that
    // ... reads another row's state, fails". Two decoys accepted means two other candidates moved.
    let result = synthesis(RELEASE);
    let id = format!("{PUBLISH}/outcome/not-accepted");
    let found = scenario(&result, &id);
    let named = sending(found, PUBLISH)["candidate"].clone();
    assert_eq!(
        accepted_besides(found, &named),
        2,
        "{id}: {:#?}",
        found.steps
    );
    let id = format!("{PUBLISH}/outcome/published");
    let found = scenario(&result, &id);
    let named = sending(found, PUBLISH)["candidate"].clone();
    assert_eq!(
        accepted_besides(found, &named),
        0,
        "{id}: {:#?}",
        found.steps
    );
}
