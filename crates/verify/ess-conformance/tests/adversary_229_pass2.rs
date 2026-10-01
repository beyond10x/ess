//! Adversary pass 2 for beyond10x/ess#229: the one-level bound on a related row of an entity being
//! arranged (`related_guard::nested`), the `exists: false` path moved ahead of it, and the pass-1
//! "no refusal" claims re-checked with a helper that filters on `Refusal::scenario`.
use std::sync::mpsc;
use std::time::Duration;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const RELEASE: &str = include_str!("fixtures/related-guard-release.yaml");

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

/// Synthesis on a thread, failing the case instead of hanging the suite where it does not finish.
fn synthesis_within(text: &str, seconds: u64) -> Synthesis {
    let (send, receive) = mpsc::channel();
    let ir = ir(text);
    std::thread::spawn(move || {
        let _ = send.send(ess_conformance::synthesize::synthesize(&ir));
    });
    receive
        .recv_timeout(Duration::from_secs(seconds))
        .unwrap_or_else(|_| panic!("synthesis did not finish within {seconds}s"))
}

/// The refusals whose `scenario` is exactly `id`, as `code: cause`. Filters on the field, not on
/// Debug text (pass 1's helper matched nothing).
fn refused(result: &Synthesis, id: &str) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|scenario| scenario.to_string() == id)
        })
        .map(|refusal| format!("{:?}: {}", refusal.code(), refusal.cause))
        .collect()
}

fn ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn has(result: &Synthesis, id: &str) -> bool {
    ids(result).iter().any(|held| held == id)
}

fn all_refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{} {:?}: {}",
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                refusal.code(),
                refusal.cause
            )
        })
        .collect()
}

/// Synthesized, with no refusal filed under its own id.
fn witnessed_clean(result: &Synthesis, id: &str) {
    assert!(
        has(result, id),
        "no scenario {id}\nscenarios: {:#?}\nrefusals: {:#?}",
        ids(result),
        all_refusals(result)
    );
    assert!(
        refused(result, id).is_empty(),
        "{id}: {:#?}",
        refused(result, id)
    );
}

// ---- shared model pieces -------------------------------------------------------------------

/// `AcceptCandidate` takes a `parent` candidate that must exist and must not be accepted (pass 1).
fn own_entity(text: &str) -> String {
    replaced(
        text,
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n    outcomes:\n",
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n      - {name: parent, type: demo.release.CandidateId}\n    outcomes:\n      - name: no-parent\n        when_related: {via: input.parent, exists: false}\n        error: demo.release.NoCandidate\n      - name: parent-not-accepted\n        when_related: {via: input.parent, predicate: state == Accepted}\n        error: demo.release.CandidateNotAccepted\n",
    )
}

/// A stored `channel` on the candidate, set from the proposing input and shown by the view.
fn with_channel(text: &str) -> String {
    let text = replaced(
        text,
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

const NOT_ACCEPTED_GUARD: &str =
    "        when_related: {via: input.candidate, predicate: state != Accepted}\n";

fn guarded(text: &str, refusal: &str) -> String {
    replaced(
        text,
        NOT_ACCEPTED_GUARD,
        &format!(
            "        when_related:\n          via: input.candidate\n          predicate: {refusal}\n"
        ),
    )
}

// ---- attack 1: the nesting bound -----------------------------------------------------------

/// A reads B reads A: accepting a candidate needs a published release, publishing a release needs
/// an accepted candidate. Neither can come first, so both successes are refused — in bounded time,
/// naming the bound, and not by recursion.
fn mutual() -> String {
    replaced(
        RELEASE,
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n    outcomes:\n",
        "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n      - {name: release, type: demo.release.ReleaseId}\n    outcomes:\n      - name: no-release\n        when_related: {via: input.release, exists: false}\n        error: demo.release.NoCandidate\n      - name: release-not-published\n        when_related: {via: input.release, predicate: state != Published}\n        error: demo.release.CandidateNotAccepted\n",
    )
}

#[test]
fn adversary_229_p2_mutual_nesting_through_two_entities_terminates_and_refuses() {
    let result = synthesis_within(&mutual(), 120);
    for id in [
        "demo.release.PublishRelease/outcome/published",
        "demo.release.AcceptCandidate/outcome/accepted",
    ] {
        assert!(!has(&result, id), "{id} cannot be witnessed");
        assert!(!refused(&result, id).is_empty(), "{id} is refused, by id");
    }
    // The refusals the model can be witnessed for still are.
    for id in [
        "demo.release.PublishRelease/outcome/not-accepted",
        "demo.release.AcceptCandidate/outcome/release-not-published",
    ] {
        witnessed_clean(&result, id);
    }
}

/// A reads B reads A where both can come first: publishing needs an accepted candidate; accepting
/// needs a release that is *not* published, which a fresh one is. The nested release is arranged
/// fresh and selects `accepted`; nothing recurses.
#[test]
fn adversary_229_p2_mutual_nesting_that_has_a_witness_synthesizes_and_admits() {
    let text = replaced(
        &mutual(),
        "predicate: state != Published}",
        "predicate: state == Published}",
    );
    let result = synthesis_within(&text, 120);
    for id in [
        "demo.release.PublishRelease/outcome/published",
        "demo.release.PublishRelease/outcome/not-accepted",
        "demo.release.AcceptCandidate/outcome/accepted",
        "demo.release.AcceptCandidate/outcome/release-not-published",
    ] {
        witnessed_clean(&result, id);
    }
    AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
}

/// The search path (`subject_fact::successors`) sends the mover at `Distinction::PLAIN`, so every
/// nested parent it arranges lands at one distinction, `27_720 * 9`. The refusal here needs an
/// accepted beta candidate, between decoys, so the stored-field search runs the mover more than
/// once in one scenario. Every scenario must bind each instance once (admission), and the guarded
/// branches must be witnessed.
#[test]
fn adversary_229_p2_searched_rows_each_get_their_own_nested_parent() {
    let text = guarded(
        &with_channel(&own_entity(RELEASE)),
        "{all: [state == Accepted, channel == Beta]}",
    );
    let result = synthesis_within(&text, 300);
    for id in [
        "demo.release.PublishRelease/outcome/not-accepted",
        "demo.release.PublishRelease/outcome/published",
        "demo.release.AcceptCandidate/outcome/accepted",
    ] {
        witnessed_clean(&result, id);
    }
    AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
}

// ---- attack 2 and 4: suites below ess/20 ---------------------------------------------------

/// `ess/19`, no `state` anywhere: closing a node needs an existing parent node, and archiving
/// needs a closed one. Archiving's arrangement drives `CloseNode` while arranging a `Node`, which
/// at the base is refused (`arranging.contains(&entity)` before anything else) and here takes the
/// new, ungated `nested` path. A document that declares no `ess/20` construct must synthesize as
/// it did at the base.
const TREE: &str = "format: ess/19
system: demo
version: v1
domain: demo.tree
summary: A node is closed only under an existing parent node.
types:
  - {name: demo.tree.NodeId, kind: newtype, of: Uuid}
entities:
  - name: demo.tree.Node
    identity: {name: node_id, type: demo.tree.NodeId}
    fields: []
    lifecycle:
      initial: Open
      states: [Open, Closed, Archived]
      terminal: [Archived]
      transitions:
        - {name: close, from: [Open], to: Closed}
        - {name: archive, from: [Closed], to: Archived}
errors:
  - {name: demo.tree.NoParent, summary: No node carries that identity., fields: []}
events:
  - name: demo.tree.NodePlanted
    fields: [{name: node_id, type: demo.tree.NodeId}]
  - name: demo.tree.NodeClosed
    fields: [{name: node_id, type: demo.tree.NodeId}]
  - name: demo.tree.NodeArchived
    fields: [{name: node_id, type: demo.tree.NodeId}]
actors:
  - name: demo.tree.Gardener
    may: [demo.tree.PlantNode, demo.tree.CloseNode, demo.tree.ArchiveNode]
commands:
  - name: demo.tree.PlantNode
    input: []
    outcomes:
      - name: planted
        creates: demo.tree.Node
        instance: node_id
        emits: [demo.tree.NodePlanted]
        payload: {demo.tree.NodePlanted: {node_id: {generated: true}}}
  - name: demo.tree.CloseNode
    input:
      - {name: node_id, type: demo.tree.NodeId}
      - {name: parent, type: demo.tree.NodeId}
    outcomes:
      - name: no-parent
        when_related: {via: input.parent, exists: false}
        error: demo.tree.NoParent
      - name: closed
        moves: demo.tree.Node.close
        instance: node_id
        emits: [demo.tree.NodeClosed]
        payload: {demo.tree.NodeClosed: {node_id: input.node_id}}
  - name: demo.tree.ArchiveNode
    input:
      - {name: node_id, type: demo.tree.NodeId}
    outcomes:
      - name: archived
        moves: demo.tree.Node.archive
        instance: node_id
        emits: [demo.tree.NodeArchived]
        payload: {demo.tree.NodeArchived: {node_id: input.node_id}}
views:
  - name: demo.tree.Nodes
    source: demo.tree.Node
    consistency: read_your_writes
    fields:
      - {name: node_id, type: demo.tree.NodeId}
      - {name: state, type: demo.tree.Node.State}
";

#[test]
fn adversary_229_p2_an_ess_19_suite_with_a_same_entity_mover_is_unchanged() {
    for format in ["ess/19", "ess/18"] {
        let text = TREE.replacen("format: ess/19\n", &format!("format: {format}\n"), 1);
        let result = synthesis(&text);
        // The base refuses the archive success: its route drives `CloseNode` while a `Node` is
        // being arranged, and a related row of an entity being arranged was never arranged.
        assert!(
            !has(&result, "demo.tree.ArchiveNode/outcome/archived"),
            "{format}: a document with no ess/20 construct gains a scenario the base did not \
             synthesize\nscenarios: {:#?}",
            ids(&result)
        );
    }
}

/// `ess/18`: a node is planted as a root where its named parent does not exist, as a child where
/// it does. The creator of `Node` is itself a related-row reader of `Node`; at the base both
/// creators were refused while a `Node` is arranged (the `arranging` check came first), and here
/// the `exists: false` creator is sent ahead of it.
fn planted() -> String {
    let text = replaced(TREE, "format: ess/19\n", "format: ess/18\n");
    // Closing reads no related row here: only the creator does.
    let text = replaced(
        &text,
        "      - {name: parent, type: demo.tree.NodeId}\n    outcomes:\n      - name: no-parent\n        when_related: {via: input.parent, exists: false}\n        error: demo.tree.NoParent\n      - name: closed\n",
        "    outcomes:\n      - name: closed\n",
    );
    replaced(
        &text,
        "  - name: demo.tree.PlantNode\n    input: []\n    outcomes:\n      - name: planted\n        creates: demo.tree.Node\n        instance: node_id\n        emits: [demo.tree.NodePlanted]\n        payload: {demo.tree.NodePlanted: {node_id: {generated: true}}}\n",
        "  - name: demo.tree.PlantNode\n    input:\n      - {name: parent, type: demo.tree.NodeId}\n    outcomes:\n      - name: root\n        when_related: {via: input.parent, exists: false}\n        creates: demo.tree.Node\n        instance: node_id\n        emits: [demo.tree.NodePlanted]\n        payload: {demo.tree.NodePlanted: {node_id: {generated: true}}}\n      - name: planted\n        creates: demo.tree.Node\n        instance: node_id\n        emits: [demo.tree.NodePlanted]\n        payload: {demo.tree.NodePlanted: {node_id: {generated: true}}}\n",
    )
}

#[test]
fn adversary_229_p2_an_ess_18_exists_false_creator_synthesizes_as_at_the_base() {
    let result = synthesis(&planted());
    // At the base no `Node` can be arranged by either creator, so no move of a node is witnessed.
    assert!(
        !has(&result, "demo.tree.ArchiveNode/outcome/archived"),
        "a document with no ess/20 construct gains a scenario the base did not synthesize\n\
         scenarios: {:#?}",
        ids(&result)
    );
}

// ---- attack 3: pass 1's "no refusal" claims, re-checked ------------------------------------

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

#[test]
fn adversary_229_p2_pass_1_no_refusal_claims_hold_by_scenario_field() {
    let publish = |outcome: &str| format!("demo.release.PublishRelease/outcome/{outcome}");
    let accept = |outcome: &str| format!("demo.release.AcceptCandidate/outcome/{outcome}");
    let cases: Vec<(&str, String, Vec<String>)> = vec![
        (
            "the unit fixture",
            RELEASE.to_owned(),
            vec![publish("no-candidate"), publish("not-accepted"), publish("published")],
        ),
        (
            "two moves deep, refusing",
            guarded(&three_states(), "state == Withdrawn"),
            vec![publish("not-accepted"), publish("published")],
        ),
        (
            "state in a list",
            guarded(&three_states(), "{state: {in: [Proposed, Withdrawn]}}"),
            vec![publish("not-accepted"), publish("published")],
        ),
        (
            "two moves deep, accepting",
            guarded(&three_states(), "state != Withdrawn"),
            vec![publish("not-accepted"), publish("published")],
        ),
        (
            "state beside a stored field",
            guarded(
                &with_channel(RELEASE),
                "{any: [state != Accepted, channel == Beta]}",
            ),
            vec![publish("not-accepted"), publish("published")],
        ),
        (
            "state and a stored field both needed",
            guarded(
                &with_channel(RELEASE),
                "{all: [state == Accepted, channel == Beta]}",
            ),
            vec![publish("not-accepted"), publish("published")],
        ),
        (
            "a related row of the command's own entity",
            own_entity(RELEASE),
            vec![
                accept("parent-not-accepted"),
                accept("accepted"),
                publish("not-accepted"),
                publish("published"),
            ],
        ),
        (
            "a mover guarded on another entity",
            replaced(
                RELEASE,
                "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n    outcomes:\n",
                "  - name: demo.release.AcceptCandidate\n    input:\n      - {name: candidate_id, type: demo.release.CandidateId}\n      - {name: release, type: demo.release.ReleaseId}\n    outcomes:\n      - name: no-release\n        when_related: {via: input.release, exists: false}\n        error: demo.release.NoCandidate\n",
            ),
            vec![publish("not-accepted"), publish("published")],
        ),
    ];
    let mut failures = Vec::new();
    for (name, text, expected) in cases {
        let result = synthesis(&text);
        for id in expected {
            if !has(&result, &id) {
                failures.push(format!("{name}: no scenario {id}"));
            }
            let named = refused(&result, &id);
            if !named.is_empty() {
                failures.push(format!("{name}: {id} refused {named:?}"));
            }
        }
        if let Err(error) = AdmittedSuite::from_suite(&result.suite) {
            failures.push(format!("{name}: suite not admitted: {error}"));
        }
    }
    assert!(failures.is_empty(), "{failures:#?}");
}
