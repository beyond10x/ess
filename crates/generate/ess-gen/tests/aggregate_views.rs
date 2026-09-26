//! The published contract says what an aggregate view computes (beyond10x/ess#96).
//!
//! `docs/design/aggregate-views.md`, "Projections": the `OpenAPI` response description and the
//! documentation page gain the grouping sentence and one clause per aggregate field; the row schema
//! is unchanged, because the aggregate fields carry their declared result types.
use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const METRICS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/aggregate-views.yaml");

const COMPONENT: &str = "components:\n  - component: metrics-service\n    reached_by: network\n    owns:\n      domains: [metrics.session]\n    accepts:\n      commands: [metrics.session.Record, metrics.session.Complete]\n    publishes:\n      events: [metrics.session.Recorded, metrics.session.Completed]\n";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(&format!("{METRICS}{COMPONENT}")).unwrap();
    let spec = Specification::assemble([(Source::new("metrics.yaml"), raw)]).unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn artifacts(ir: &EssIr, matches: impl Fn(&str) -> bool) -> String {
    ess_gen::generate_all(ir)
        .unwrap()
        .into_iter()
        .filter(|(path, _)| matches(path))
        .map(|(path, artifact)| format!("== {path}\n{}", artifact.contents))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_openapi_description_says_how_rows_are_grouped_and_what_each_field_computes() {
    let openapi = artifacts(&ir(), |path| path.contains("openapi"));
    assert!(
        openapi.contains(
            "Contains the instances where `state == Completed` holds. Grouped by `agent_id`; one \
             row per group holding at least one instance. `distinct_callers` = count of distinct \
             `caller` values; `longest_wait` = greatest `wait_seconds`; `mean_talk` = average of \
             `talk_seconds`, rounded to 6 places half-even; `sessions` = count of instances; \
             `talk_seconds` = sum of `talk_seconds`."
        ),
        "{openapi}"
    );
    assert!(
        openapi.contains("holds. Always exactly one row. `longest_wait` = greatest `wait_seconds`"),
        "{openapi}"
    );
}

#[test]
fn the_documentation_says_how_rows_are_grouped_and_what_each_field_computes() {
    let docs = artifacts(&ir(), |path| {
        std::path::Path::new(path)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("md"))
    });
    assert!(
        docs.contains(
            "Grouped by `agent_id`, `channel`; one row per group holding at least one instance."
        ),
        "{docs}"
    );
    assert!(docs.contains("Always exactly one row."), "{docs}");
    assert!(
        docs.contains("`sessions` — `Integer`, the count of instances in the group"),
        "{docs}"
    );
    assert!(
        docs.contains(
            "`mean_talk` — `Optional<Decimal>`, which may be absent, the average of \
             `talk_seconds`, rounded to 6 places half-even in the group"
        ),
        "{docs}"
    );
    // A group key is a field like any other.
    assert!(docs.contains("`agent_id` — `String`\n"), "{docs}");
}

#[test]
fn a_model_without_an_aggregate_view_keeps_its_description() {
    let text =
        include_str!("../../../verify/ess-conformance/tests/fixtures/stored-field-guards.yaml");
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("parcels.yaml"), raw)]).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let all = artifacts(&ir, |_| true);
    assert!(!all.contains("Grouped by"), "{all}");
    assert!(!all.contains("exactly one row"), "{all}");
}
