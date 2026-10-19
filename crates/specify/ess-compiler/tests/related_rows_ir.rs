//! A selector's rows quantifier travels in the compiled IR as the predicate's canonical node
//! (`docs/design/related-rows-quantifier.md`, "Format and compatibility", beyond10x/ess#551):
//! `{exists: {in: {entity: <E>}, as, that}}` inside `ResolvedRowSelection.where`. No IR shape
//! changes, so a model without the construct keeps its bytes, and a selector with an ordinary
//! quantifier over a `List` field compiles alike at `ess/26` and at `ess/27`.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use sha2::{Digest, Sha256};

const HEAD: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/ordered-related-head.yaml");
const READS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/filtered-related-reads.yaml");
const LIST: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/aggregate-list-parameter.yaml");
const PRECONDITIONS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/explore-preconditions-list.yaml");

/// The best open slot of a group whose two sides each hold a `Present` membership.
const MATCHABLE: &str = "format: ess/27
system: demo
version: v1
domain: demo.slots
types:
  - {name: demo.slots.SlotId, kind: newtype, of: Uuid}
  - {name: demo.slots.MembershipId, kind: newtype, of: Uuid}
  - {name: demo.slots.PairId, kind: newtype, of: Uuid}
  - {name: demo.slots.Side, kind: enum, variants: [Left, Right]}
entities:
  - name: demo.slots.Slot
    identity: {name: slot_id, type: demo.slots.SlotId}
    fields:
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
  - name: demo.slots.Membership
    identity: {name: membership_id, type: demo.slots.MembershipId}
    fields:
      - {name: slot_id, type: demo.slots.SlotId}
      - {name: side, type: demo.slots.Side}
    lifecycle:
      initial: Present
      states: [Present, Removed]
      terminal: [Removed]
      transitions:
        - {name: remove, from: [Present], to: Removed}
  - name: demo.slots.Pair
    identity: {name: pair_id, type: demo.slots.PairId}
    fields:
      - {name: group_id, type: String}
    lifecycle:
      initial: Made
      states: [Made, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Made], to: Done}
errors:
  - {name: demo.slots.NoMatchableSlot, summary: No slot of the group can match., fields: []}
events:
  - name: demo.slots.SlotOpened
    fields:
      - {name: slot_id, type: demo.slots.SlotId}
  - name: demo.slots.SlotClosed
    fields:
      - {name: slot_id, type: demo.slots.SlotId}
  - name: demo.slots.Joined
    fields:
      - {name: membership_id, type: demo.slots.MembershipId}
  - name: demo.slots.Left
    fields:
      - {name: membership_id, type: demo.slots.MembershipId}
  - name: demo.slots.Finished
    fields:
      - {name: pair_id, type: demo.slots.PairId}
  - name: demo.slots.PairMade
    fields:
      - {name: pair_id, type: demo.slots.PairId}
      - {name: slot_id, type: demo.slots.SlotId}
commands:
  - name: demo.slots.OpenSlot
    input:
      - {name: slot_id, type: demo.slots.SlotId}
      - {name: group_id, type: String}
      - {name: score, type: Integer}
    outcomes:
      - name: opened
        creates: demo.slots.Slot
        instance: slot_id
        sets: {group_id: input.group_id, score: input.score}
        emits: [demo.slots.SlotOpened]
        payload:
          demo.slots.SlotOpened: {slot_id: input.slot_id}
  - name: demo.slots.CloseSlot
    input:
      - {name: slot_id, type: demo.slots.SlotId}
    outcomes:
      - name: closed
        moves: demo.slots.Slot.close
        instance: slot_id
        emits: [demo.slots.SlotClosed]
        payload:
          demo.slots.SlotClosed: {slot_id: input.slot_id}
  - name: demo.slots.Join
    input:
      - {name: membership_id, type: demo.slots.MembershipId}
      - {name: slot_id, type: demo.slots.SlotId}
      - {name: side, type: demo.slots.Side}
    outcomes:
      - name: joined
        creates: demo.slots.Membership
        instance: membership_id
        sets: {slot_id: input.slot_id, side: input.side}
        emits: [demo.slots.Joined]
        payload:
          demo.slots.Joined: {membership_id: input.membership_id}
  - name: demo.slots.Leave
    input:
      - {name: membership_id, type: demo.slots.MembershipId}
    outcomes:
      - name: left
        moves: demo.slots.Membership.remove
        instance: membership_id
        emits: [demo.slots.Left]
        payload:
          demo.slots.Left: {membership_id: input.membership_id}
  - name: demo.slots.Finish
    input:
      - {name: pair_id, type: demo.slots.PairId}
    outcomes:
      - name: finished
        moves: demo.slots.Pair.finish
        instance: pair_id
        emits: [demo.slots.Finished]
        payload:
          demo.slots.Finished: {pair_id: input.pair_id}
  - name: demo.slots.MatchInGroup
    input:
      - {name: pair_id, type: demo.slots.PairId}
      - {name: group_id, type: String}
    outcomes:
      - name: no-match
        when_related:
          entity: demo.slots.Slot
          where: {all: [group_id == input.group_id, state == Open, {exists: {in: {entity: demo.slots.Membership}, as: member, that: {all: [member.slot_id == slot_id, member.side == Left, member.state == Present]}}}]}
          exists: false
        error: demo.slots.NoMatchableSlot
      - name: paired
        creates: demo.slots.Pair
        instance: pair_id
        sets:
          group_id: input.group_id
        emits: [demo.slots.PairMade]
        payload:
          demo.slots.PairMade:
            pair_id: input.pair_id
            slot_id:
              related:
                entity: demo.slots.Slot
                where: {all: [group_id == input.group_id, state == Open, {exists: {in: {entity: demo.slots.Membership}, as: member, that: {all: [member.slot_id == slot_id, member.side == Left, member.state == Present]}}}]}
                order_by: [score desc, slot_id]
                first: slot_id
";

/// The canonical IR.
fn canonical(text: &str) -> String {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new())
        .unwrap_or_else(|error| panic!("{error:?}"))
        .to_canonical_json()
}

/// The canonical IR with every whitespace character removed.
fn compact(text: &str) -> String {
    canonical(text)
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn digest(text: &str) -> String {
    use std::fmt::Write as _;
    Sha256::digest(canonical(text).as_bytes())
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[test]
fn the_selection_carries_the_rows_test_in_its_predicate() {
    let compact = compact(MATCHABLE);
    let test = r#"{"exists":{"as":"member","in":{"entity":"demo.slots.Membership"},"that":{"all":[{"member.slot_id":{"eq":{"fact":"slot_id"}}},"member.side==Left","member.state==Present"]}}}"#;
    let selector =
        format!(r#""where":{{"all":["group_id==input.group_id","state==Open",{test}]}}"#);
    // The head's selector and the guard's, each with the rows test as written.
    assert_eq!(
        compact.matches(&selector).count(),
        2,
        "{selector}\n{compact}"
    );
}

/// The SHA-256 of each fixture's canonical IR, taken on the head before this construct existed.
const PINNED: [(&str, &str, &str); 4] = [
    (
        "ordered-related-head",
        HEAD,
        "e19568fafe85126b8e2020980f84a5c8efbd8d9855a2d2744080915a853e80b1",
    ),
    (
        "filtered-related-reads",
        READS,
        "7285eaf9793c22d81245f49c1445aeff62638f3e2afdce1c23f1c9db425b5b98",
    ),
    (
        "aggregate-list-parameter",
        LIST,
        "f1b6f25c1642e2d1dc117057fc482aafbeaad2566bcec7843fcaf8a707b7c0d5",
    ),
    (
        "explore-preconditions-list",
        PRECONDITIONS,
        "52e815862f7967d1205ec5a30a3d19b0f4be19152109705581bc34ffefb80c45",
    ),
];

#[test]
fn a_model_without_a_rows_test_keeps_its_ir_bytes() {
    let mut moved = Vec::new();
    for (name, fixture, pinned) in PINNED {
        let now = digest(fixture);
        if now != pinned {
            moved.push(format!("{name}: {now}"));
        }
    }
    assert!(
        moved.is_empty(),
        "the IR bytes moved:\n{}",
        moved.join("\n")
    );

    // A selector with an ordinary quantifier over a `List` field of the candidate compiles alike
    // at `ess/26` and at `ess/27`.
    let listed = |format: &str| {
        READS
            .replace("format: ess/22", &format!("format: {format}"))
            .replacen(
                "      - {name: note, type: Optional<String>}\n",
                "      - {name: note, type: Optional<String>}\n      - {name: tags, type: \
                 'Optional<List<String>>'}\n",
                1,
            )
            .replacen(
                "where: {all: [worker_id == input.worker_id, batch_id == input.batch_id]}",
                "where: {all: [worker_id == input.worker_id, batch_id == input.batch_id, \
                 {exists: {in: tags, as: tag, that: tag == input.worker_id}}]}",
                1,
            )
    };
    let older = listed("ess/26");
    assert!(older.contains("in: tags"), "the edit applies:\n{older}");
    assert_eq!(
        canonical(&older).replace("ess/26", "ess/27"),
        canonical(&listed("ess/27"))
    );
}
