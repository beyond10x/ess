//! Adversary pass 1 for beyond10x/ess#219: a newtype's `prefix:` change, attacked from outside.
//!
//! `ess-diff/10` shipped in 0.41.0 without the prefix kinds, so they are `ess-diff/11` vocabulary
//! (coordinator decision, 2026-09-29): a delta carrying one is `ess-diff/11`, and both an
//! `ess-diff/10` writer and an `ess-diff/10` reader refuse it. The rest drives transitions the
//! unit's own suite does not: a prefix moved beside an alphabet, a representation, a reading
//! contract or a kind; case-only and unrelated replacements; and the dependency closure a prefix
//! change seeds through a wrapping newtype.
#![allow(clippy::too_many_lines)]

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{diff, impact, EssDelta, RawEssDelta, SemanticRelation};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const HEADER: &str = "format: ess/15\nsystem: demo\nversion: v1\ndomain: demo.msgs\n";

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("msgs.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors:?}\n{text}"))
}

fn delta(before: &str, after: &str) -> EssDelta {
    diff(&ir(before), &ir(after)).unwrap()
}

/// One newtype `demo.msgs.Code` whose body lines are given verbatim.
fn one(body: &str) -> String {
    format!("{HEADER}types:\n  - name: demo.msgs.Code\n{body}")
}

fn ids(delta: &EssDelta) -> Vec<String> {
    delta.changes().iter().map(|c| c.id().to_string()).collect()
}

fn relation_of(delta: &EssDelta, id: &str) -> SemanticRelation {
    delta
        .changes()
        .iter()
        .find(|c| c.id().to_string() == id)
        .unwrap_or_else(|| panic!("{id} in {:#?}", ids(delta)))
        .relation()
}

#[test]
fn a_prefix_change_is_ess_diff_11_vocabulary_and_ess_diff_10_refuses_it() {
    for (before, after) in [
        (
            "    kind: newtype\n    of: String\n",
            "    kind: newtype\n    of: String\n    prefix: \"ord_\"\n",
        ),
        (
            "    kind: newtype\n    of: String\n    prefix: \"ord_\"\n",
            "    kind: newtype\n    of: String\n",
        ),
        (
            "    kind: newtype\n    of: String\n    prefix: \"ord_\"\n",
            "    kind: newtype\n    of: String\n    prefix: \"inv_\"\n",
        ),
    ] {
        let delta = delta(&one(before), &one(after));
        let json = delta.to_canonical_json();
        assert_eq!(delta.changes().len(), 1, "{json}");
        assert_eq!(
            delta.changes()[0].minimum_format(),
            11,
            "a prefix kind is not in the released ess-diff/10: {json}"
        );
        assert_eq!(delta.format.to_string(), "ess-diff/11", "{json}");
        assert!(
            delta
                .to_canonical_json_for("ess-diff/10".parse().unwrap())
                .is_err(),
            "an ess-diff/10 writer must refuse a prefix kind: {json}"
        );
        let as_ten = json.replace("\"ess-diff/11\"", "\"ess-diff/10\"");
        let raw: RawEssDelta = serde_json::from_str(&as_ten).unwrap();
        let refused = format!(
            "{:?}",
            EssDelta::try_from(raw).expect_err("an ess-diff/10 reader refuses a prefix kind")
        );
        assert!(refused.contains("UnsupportedFormatVersion"), "{refused}");
    }
}

#[test]
fn case_only_unrelated_and_one_character_replacements_take_the_comparison_relation() {
    for (was, is, relation) in [
        // Case differs: neither starts with the other.
        ("Ord_", "ord_", SemanticRelation::Changed),
        // Same characters, different order.
        ("ab", "ba", SemanticRelation::Changed),
        // One character extended to many, and back.
        ("o", "ord_", SemanticRelation::Narrowed),
        ("ord_", "o", SemanticRelation::Expanded),
        // Shared stem, neither extends the other.
        ("ord_a", "ord_b", SemanticRelation::Changed),
    ] {
        let before = one(&format!(
            "    kind: newtype\n    of: String\n    prefix: \"{was}\"\n"
        ));
        let after = one(&format!(
            "    kind: newtype\n    of: String\n    prefix: \"{is}\"\n"
        ));
        let delta = delta(&before, &after);
        assert_eq!(
            ids(&delta),
            ["type/demo.msgs.Code/prefix-changed"],
            "{was} → {is}"
        );
        assert_eq!(
            relation_of(&delta, "type/demo.msgs.Code/prefix-changed"),
            relation,
            "{was} → {is}"
        );
    }
}

#[test]
fn a_prefix_moved_beside_an_alphabet_reports_both_each_with_its_own_relation() {
    // Alphabet widened (a superset) while the prefix is extended: one expands, one narrows.
    let before =
        one("    kind: newtype\n    of: String\n    alphabet: \"abcdo_\"\n    prefix: \"o\"\n");
    let after =
        one("    kind: newtype\n    of: String\n    alphabet: \"abcdor_\"\n    prefix: \"or\"\n");
    let delta = delta(&before, &after);
    let json = delta.to_canonical_json();
    assert_eq!(
        ids(&delta),
        [
            "type/demo.msgs.Code/alphabet-changed",
            "type/demo.msgs.Code/prefix-changed",
        ],
        "{json}"
    );
    assert_eq!(
        relation_of(&delta, "type/demo.msgs.Code/prefix-changed"),
        SemanticRelation::Narrowed
    );
    assert!(!json.contains("unclassified"), "{json}");
}

#[test]
fn a_prefix_moved_beside_a_representation_reports_both() {
    let before = format!(
        "{HEADER}types:\n  - name: demo.msgs.Base\n    kind: newtype\n    of: String\n  \
         - name: demo.msgs.Code\n    kind: newtype\n    of: String\n    prefix: \"x/\"\n"
    );
    let after = format!(
        "{HEADER}types:\n  - name: demo.msgs.Base\n    kind: newtype\n    of: String\n  \
         - name: demo.msgs.Code\n    kind: newtype\n    of: demo.msgs.Base\n"
    );
    let delta = delta(&before, &after);
    let json = delta.to_canonical_json();
    assert_eq!(
        ids(&delta),
        [
            "type/demo.msgs.Code/prefix-removed",
            "type/demo.msgs.Code/representation-changed",
        ],
        "{json}"
    );
    assert!(!json.contains("unclassified"), "{json}");
}

#[test]
fn a_prefix_dropped_by_a_kind_change_is_the_kind_change_alone() {
    let before = one("    kind: newtype\n    of: String\n    prefix: \"x/\"\n");
    let after = one("    kind: struct\n    fields:\n      - {name: text, type: String}\n");
    let delta = delta(&before, &after);
    let json = delta.to_canonical_json();
    assert_eq!(ids(&delta), ["type/demo.msgs.Code/kind-changed"], "{json}");
}

#[test]
fn a_reading_change_beside_a_prefix_change_reports_both_and_nothing_else() {
    let reading = |role: &str| {
        format!(
            "    reading:\n      encoding: offset_date_time_text\n      \
             origins: [{{role: {role}, offset: encoded_offset}}]\n"
        )
    };
    let before = one(&format!(
        "    kind: newtype\n    of: String\n{}",
        reading("producer_process")
    ));
    let after = one(&format!(
        "    kind: newtype\n    of: String\n    prefix: \"2\"\n{}",
        reading("consumer_process")
    ));
    let delta = delta(&before, &after);
    let json = delta.to_canonical_json();
    assert_eq!(
        ids(&delta),
        [
            "type/demo.msgs.Code/prefix-added",
            "type/demo.msgs.Code/reading-contract-changed",
        ],
        "{json}"
    );
}

#[test]
fn a_prefix_change_seeds_an_attributed_closure_through_a_wrapping_newtype() {
    let model = |prefix: &str| {
        format!(
            "{HEADER}types:\n  - name: demo.msgs.Inner\n    kind: newtype\n    of: String\n\
             {prefix}  - name: demo.msgs.Outer\n    kind: newtype\n    of: demo.msgs.Inner\n\
             events:\n  - name: demo.msgs.Sent\n    fields:\n      \
             - {{name: code, type: demo.msgs.Outer}}\n\
             components:\n  - component: msgs-service\n    owns: {{domains: [demo.msgs]}}\n    \
             publishes: {{events: [demo.msgs.Sent]}}\n"
        )
    };
    let before = ir(&model(""));
    let after = ir(&model("    prefix: \"m-\"\n"));
    let report = impact(&before, &after, None, None).unwrap();
    let delta_ids = ids(&report.delta);
    assert_eq!(delta_ids, ["type/demo.msgs.Inner/prefix-added"]);
    let change = report.delta.changes()[0].id();
    let reached: Vec<String> = report
        .impacts_of(&change)
        .map(|path| format!("{:?}", path.target))
        .collect();
    assert!(
        reached
            .iter()
            .any(|target| target.contains("demo.msgs.Outer")),
        "{reached:#?}"
    );
    assert!(
        reached
            .iter()
            .any(|target| target.contains("demo.msgs.Sent")),
        "{reached:#?}"
    );
}

/// Evidence for a judgement note, green by design: the relation is read from the declared prefix,
/// so an outer newtype declaring the prefix its inner layer already imposes is called `narrowed`
/// although the values it admits are the same on both sides (the effective prefix is `m-` both
/// times). The raw reader re-derives the relation from the change's own content, so a relation that
/// looked at the effective prefix could not be checked by it.
#[test]
fn an_outer_prefix_equal_to_the_inner_one_is_called_narrowed_though_nothing_narrows() {
    let model = |outer: &str| {
        format!(
            "{HEADER}types:\n  - name: demo.msgs.Inner\n    kind: newtype\n    of: String\n    \
             prefix: \"m-\"\n  - name: demo.msgs.Outer\n    kind: newtype\n    of: demo.msgs.Inner\n\
             {outer}"
        )
    };
    let delta = delta(&model(""), &model("    prefix: \"m-\"\n"));
    assert_eq!(ids(&delta), ["type/demo.msgs.Outer/prefix-added"]);
    assert_eq!(
        relation_of(&delta, "type/demo.msgs.Outer/prefix-added"),
        SemanticRelation::Narrowed
    );
}
