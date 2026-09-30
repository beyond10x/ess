//! An added grouped view is classified by `view/<view>/added` alone (beyond10x/ess#256).
//!
//! `aggregation` is compared by `compare_aggregations`; left in the residual, a view added with an
//! aggregation also raised `system/<system>/unclassified-changed` and whole obligations.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::diff;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const METRICS: &str = include_str!("../../ess-conformance/tests/fixtures/aggregate-views.yaml");

fn ir(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("metrics.yaml"),
        RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}")),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

const ADDED: &str = "  - name: metrics.session.SessionsByCaller
    source: metrics.session.Session
    consistency: eventual
    group_by: [caller, state]
    fields:
      - {name: caller, type: String}
      - {name: state, type: metrics.session.Session.State}
      - {name: sessions, type: Integer, aggregate: {count: {}}}
";

fn ids(before: &str, after: &str) -> Vec<String> {
    diff(&ir(before), &ir(after))
        .unwrap()
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect()
}

#[test]
fn an_added_grouped_view_is_not_an_unclassified_change() {
    let after = METRICS.replace("views:\n", &format!("views:\n{ADDED}"));
    let ids = ids(METRICS, &after);
    assert_eq!(
        ids,
        ["view/metrics.session.SessionsByCaller/added"],
        "{ids:?}"
    );
}

#[test]
fn a_regrouped_view_is_not_an_unclassified_change() {
    let after = METRICS.replace(
        "    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n",
        "    group_by: [agent_id, state]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: state, type: metrics.session.Session.State}\n",
    );
    let ids = ids(METRICS, &after);
    assert!(
        ids.iter().all(|id| !id.ends_with("/unclassified-changed")),
        "{ids:?}"
    );
    assert!(
        ids.iter()
            .any(|id| id == "view/metrics.session.TalkTimeByAgent/grouping-changed"),
        "{ids:?}"
    );
}
