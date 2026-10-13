//! The compiled IR carries the head of an ordered selection as its own payload value
//! (`docs/design/ordered-related-head.md`, "Format and compatibility"):
//! `ResolvedPayloadValue::RelatedHead` (`kind: related_head`) with the selector a filtered read
//! carries, the ranking keys written as a view writes them, the field read and its type. A model
//! without the source carries none, and a nested mapping that only looks like the source keeps the
//! IR it compiled to before `ess/25`.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const HEAD: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/ordered-related-head.yaml");
const READS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/filtered-related-reads.yaml");

/// The canonical IR with every whitespace character removed.
fn ir(text: &str) -> String {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("ranked.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new())
        .unwrap_or_else(|error| panic!("{error:?}"))
        .to_canonical_json()
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replace(from, to)
}

const SELECTION: &str = r#""selection":{"entity":"demo.ranked.Membership","where":{"all":["slot_id==input.slot_id","state==Present"]}}"#;

#[test]
fn the_head_compiles_to_related_head_with_its_rankings() {
    let compact = ir(HEAD);
    let key = format!(
        r#"{{"kind":"related_head",{SELECTION},"order_by":["element_score desc","membership_id asc"],"field":"element_key","type_ref":{{"kind":"primitive","name":"string"}}}}"#
    )
    .replace(' ', "");
    assert!(compact.contains(&key), "{key}\n{compact}");
    // The identity read at the identity's declared type.
    assert!(
        compact.contains(
            r#""order_by":["element_scoredesc","membership_idasc"],"field":"membership_id","type_ref":{"kind":"declared","name":"demo.ranked.MembershipId"}"#
        ),
        "{compact}"
    );
    // Never the exactly-one read.
    assert!(!compact.contains("related_selection"), "{compact}");
}

/// A model without the head carries no `related_head`, and the filtered read keeps its own kind.
#[test]
fn a_model_without_the_head_keeps_its_ir_bytes() {
    let reads = ir(READS);
    assert!(!reads.contains("related_head"), "{reads}");
    assert!(reads.contains("related_selection"), "{reads}");

    let minted = ir(&HEAD
        .replace(
            "            element_key:\n              related:\n                entity: demo.ranked.Membership\n                where: {all: [slot_id == input.slot_id, state == Present]}\n                order_by: [element_score desc, membership_id]\n                first: element_key\n",
            "            element_key: {generated: true}\n",
        )
        .replace(
            "            membership_id:\n              related:\n                entity: demo.ranked.Membership\n                where: {all: [slot_id == input.slot_id, state == Present]}\n                order_by: [element_score desc, membership_id]\n                first: membership_id\n",
            "            membership_id: {generated: true}\n",
        )
        .replace(
            "          element_key:\n            related:\n              entity: demo.ranked.Membership\n              where: {all: [slot_id == input.slot_id, state == Present]}\n              order_by: [element_score desc, membership_id]\n              first: element_key\n",
            "          element_key: {generated: true}\n",
        ));
    assert!(!minted.contains("related_head"), "{minted}");
    assert!(!minted.contains("order_by"), "{minted}");
}

/// A struct target whose fields are `related: {entity, where, order_by, first}`, all texts, is the
/// nested mapping it always was: it compiles to the same IR at `ess/24` and at `ess/25`, the
/// recorded format aside.
#[test]
fn a_text_order_by_compiles_to_the_same_nested_mapping_at_ess_24_and_ess_25() {
    let nested = edit(
        &edit(
            HEAD,
            "  - {name: demo.ranked.Score, kind: newtype, of: Integer}\n",
            "  - {name: demo.ranked.Score, kind: newtype, of: Integer}\n  - name: demo.ranked.Selector\n    kind: struct\n    fields:\n      - {name: entity, type: String}\n      - {name: where, type: String}\n      - {name: order_by, type: String}\n      - {name: first, type: String}\n  - name: demo.ranked.Reference\n    kind: struct\n    fields:\n      - {name: related, type: demo.ranked.Selector}\n",
        ),
        "      - {name: element_key, type: String}\n    lifecycle:\n      initial: Open",
        "      - {name: element_key, type: String}\n      - {name: reference, type: Optional<demo.ranked.Reference>}\n    lifecycle:\n      initial: Open",
    );
    let nested = edit(
        &nested,
        "          slot_id: input.slot_id\n          element_key:\n            related:",
        "          slot_id: input.slot_id\n          reference:\n            related:\n              entity: demo.ranked.Membership\n              where: slot_id\n              order_by: element_score desc\n              first: element_key\n          element_key:\n            related:",
    );
    // Below `ess/25` the head itself is refused, so both formats compile the model without it.
    let plain = |format: &str| {
        let text = edit(&nested, "format: ess/25", &format!("format: {format}"));
        let text = text
            .replace(
                "            related:\n              entity: demo.ranked.Membership\n              where: {all: [slot_id == input.slot_id, state == Present]}\n              order_by: [element_score desc, membership_id]\n              first: element_key\n",
                "            {generated: true}\n",
            )
            .replace(
                "              related:\n                entity: demo.ranked.Membership\n                where: {all: [slot_id == input.slot_id, state == Present]}\n                order_by: [element_score desc, membership_id]\n                first: element_key\n",
                "              {generated: true}\n",
            )
            .replace(
                "              related:\n                entity: demo.ranked.Membership\n                where: {all: [slot_id == input.slot_id, state == Present]}\n                order_by: [element_score desc, membership_id]\n                first: membership_id\n",
                "              {generated: true}\n",
            );
        assert!(!text.contains("order_by: ["), "{text}");
        ir(&text).replace(&format!(r#""format":"{format}""#), "")
    };
    let at_24 = plain("ess/24");
    let at_25 = plain("ess/25");
    assert!(
        at_25.contains(r#""target":"order_by""#),
        "the text order_by is a nested leaf: {at_25}"
    );
    assert_eq!(at_24, at_25);
}

/// `ties: unordered` is recorded beside the keys, and only there: a total order's head keeps the
/// bytes it compiled to before the flag existed.
#[test]
fn ties_unordered_is_recorded_beside_the_rankings() {
    let total = ir(HEAD);
    assert!(!total.contains(r#""ties""#), "{total}");

    let text = HEAD.replace(
        "                order_by: [element_score desc, membership_id]\n                first: element_key\n",
        "                order_by: [element_score desc]\n                ties: unordered\n                first: element_key\n",
    );
    assert_ne!(text, HEAD, "fixture edit did not land");
    let compact = ir(&text);
    let key = format!(
        r#"{{"kind":"related_head",{SELECTION},"order_by":["element_score desc"],"ties":"unordered","field":"element_key","type_ref":{{"kind":"primitive","name":"string"}}}}"#
    )
    .replace(' ', "");
    assert!(compact.contains(&key), "{key}\n{compact}");
}
