//! A refusal-selected failure policy (ess/22, beyond10x/ess#269) is compared as its complete
//! resolved table: `binding/<name>/refusal-policy-changed`, relation Changed, carrying the typed
//! normalized table and fallback on each side. It is `ess-diff/14` vocabulary: no earlier format
//! can carry it, written or read. A universal policy change keeps `failure-changed` and its
//! minimum format.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_diff::change::{BindingChange, SemanticChange, SemanticRelation};
use ess_diff::{classified, diff};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("../../ess-conformance/tests/fixtures/refusal-policy.yaml");
const POLICY: &str = "    on_failure:
      drop: [wrong-state]
      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [wrong-state, demo.ledger.Unavailable, rejected]
";

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("refusal-policy.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn revised(policy: &str) -> String {
    let revised = MODEL.replace(POLICY, policy);
    assert_ne!(revised, MODEL);
    revised
}

fn changes(before: &str, after: &str) -> Vec<SemanticChange> {
    diff(&compiled(before), &compiled(after))
        .expect("one system")
        .changes()
        .to_vec()
}

fn binding_changes(before: &str, after: &str) -> Vec<BindingChange> {
    changes(before, after)
        .into_iter()
        .map(|change| match change {
            SemanticChange::Binding { changed, .. } => changed,
            other => panic!("a binding change: {other:?}"),
        })
        .collect()
}

#[test]
fn a_changed_table_is_one_refusal_policy_change_carrying_both_complete_tables() {
    let after = revised(
        "    on_failure:
      drop: [wrong-state, at-limit]
      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 4, final: [rejected]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [wrong-state, at-limit, demo.ledger.Unavailable, rejected]
",
    );
    let changes = changes(MODEL, &after);
    assert_eq!(changes.len(), 1, "{changes:#?}");
    let change = &changes[0];
    assert_eq!(
        change.id().to_string(),
        "binding/notify-ledger/refusal-policy-changed"
    );
    assert_eq!(change.relation(), SemanticRelation::Changed);
    assert_eq!(change.minimum_format(), 14);
    let json = serde_json::to_value(change).unwrap();
    let before = &json["changed"]["before"];
    let after = &json["changed"]["after"];
    assert_eq!(
        json["changed"]["kind"], "refusal-policy-changed",
        "{json:#}"
    );
    assert_eq!(
        before["fallback"],
        serde_json::json!({"policy": "escalate", "emits": "demo.ledger.RecordEscalated"}),
        "{json:#}"
    );
    // Keyed by outcome, in name order: at-limit, busy, rejected, unavailable, wrong-state.
    assert_eq!(before["refusals"].as_array().unwrap().len(), 5, "{json:#}");
    assert_eq!(
        after["refusals"][3],
        serde_json::json!({"outcome": "unavailable", "policy": "retry", "attempts": 4, "final": ["rejected"]}),
        "{json:#}"
    );
    assert_eq!(
        after["refusals"][0],
        serde_json::json!({"outcome": "at-limit", "policy": "drop"}),
        "{json:#}"
    );
}

#[test]
fn rewriting_a_table_with_aliases_that_resolve_identically_is_no_change() {
    let after = revised(
        "    on_failure:
      drop: [demo.ledger.WrongState]
      retry: {outcomes: [unavailable, busy, demo.ledger.Unknown], attempts: 3, final: [demo.ledger.Unknown]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [demo.ledger.WrongState, unavailable, busy, rejected]
",
    );
    assert_eq!(changes(MODEL, &after).len(), 0);
}

#[test]
fn a_universal_policy_becoming_selected_reports_both_the_word_and_the_table() {
    let universal = revised("    on_failure: retry\n");
    let changes = binding_changes(&universal, MODEL);
    assert_eq!(changes.len(), 2, "{changes:#?}");
    assert!(
        matches!(&changes[0], BindingChange::FailureChanged { before, after }
            if before == "retry" && after.contains("per refusal")),
        "{changes:#?}"
    );
    assert!(
        matches!(&changes[1], BindingChange::RefusalPolicyChanged { before, after }
            if before.is_none() && after.is_some()),
        "{changes:#?}"
    );
}

#[test]
fn a_universal_change_keeps_failure_changed_and_its_minimum_format() {
    let before = revised("    on_failure: retry\n");
    let after = revised("    on_failure: drop\n");
    let changes = changes(&before, &after);
    assert_eq!(changes.len(), 1, "{changes:#?}");
    assert_eq!(
        changes[0].id().to_string(),
        "binding/notify-ledger/failure-changed"
    );
    assert!(
        changes[0].minimum_format() < 14,
        "{}",
        changes[0].minimum_format()
    );
}

#[test]
fn no_format_before_14_carries_the_change_written_or_read() {
    let after = revised(
        "    on_failure:\n      escalate: {emits: demo.ledger.RecordEscalated, except: []}\n",
    );
    let delta = classified(&compiled(MODEL), &compiled(&after)).expect("one system");
    let written = delta.to_canonical_json();
    assert!(written.contains("\"ess-diff/14\""), "{written}");
    let raw: ess_diff::RawEssDelta = serde_json::from_str(&written).unwrap();
    ess_diff::EssDelta::try_from(raw).unwrap_or_else(|errors| panic!("{errors}"));
    let unclassified = diff(&compiled(MODEL), &compiled(&after)).expect("one system");
    for major in [10, 13] {
        let format = ess_diff::DeltaFormat::parse(&format!("ess-diff/{major}")).unwrap();
        assert!(
            unclassified.to_canonical_json_for(format).is_err(),
            "ess-diff/{major} cannot write it"
        );
    }
    // Read back under a format that cannot carry it, with the classification removed so that only
    // the vocabulary is wrong.
    let mut document: serde_json::Value = serde_json::from_str(&written).unwrap();
    document["format"] = "ess-diff/13".into();
    for change in document["changes"].as_array_mut().unwrap() {
        change.as_object_mut().unwrap().remove("compatibility");
    }
    let raw: ess_diff::RawEssDelta = serde_json::from_value(document).unwrap();
    let errors = ess_diff::EssDelta::try_from(raw).expect_err("ess-diff/13 cannot carry it");
    assert!(
        errors.to_string().contains("refusal-policy-changed"),
        "{errors}"
    );
}
