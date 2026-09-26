//! The native targets say what an aggregate view computes (beyond10x/ess#96).
//!
//! `docs/design/aggregate-views.md`, "Projections": the plan's `ViewQuery` obligation, the Rust and
//! Go row types' doc comments, and the web catalog and page render the grouping and each
//! aggregate. The row structs are unchanged — the owed query computes the aggregate.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, SynthesisPlan, Target};

const METRICS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/aggregate-views.yaml");

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("metrics.yaml"),
        RawSpecFile::parse(METRICS).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn emitted(target: Target) -> String {
    let synthesis = synthesize_for(&ir(), target).unwrap_or_else(|error| panic!("{error:?}"));
    synthesis
        .artifacts
        .iter()
        .map(|(path, artifact)| format!("== {path}\n{}", artifact.contents))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn the_plan_contract_names_the_grouping_and_every_aggregate() {
    let plan = SynthesisPlan::of(&ir()).to_markdown();
    assert!(
        plan.contains(
            "containing instances where `state == Completed`, grouped by `agent_id`, computing \
             `distinct_callers = count_distinct(caller)`, `longest_wait = max(wait_seconds)`, \
             `mean_talk = avg(talk_seconds)`, `sessions = count()`, `talk_seconds = \
             sum(talk_seconds)`"
        ),
        "{plan}"
    );
    assert!(
        plan.contains(
            ", one row, computing `longest_wait = max(wait_seconds)`, `sessions = count()`"
        ),
        "{plan}"
    );
}

#[test]
fn the_rust_and_go_row_types_carry_the_grouping_line() {
    let rust = emitted(Target::Rust);
    assert!(
        rust.contains(
            "/// Grouped by `agent_id`, `channel`; one row per group holding at least one instance.\n/// Serving it"
        ),
        "{rust}"
    );
    assert!(
        rust.contains("/// Always exactly one row.\n/// Serving it"),
        "{rust}"
    );
    let go = emitted(Target::Go);
    assert!(
        go.contains("// Grouped by `agent_id`; one row per group holding at least one instance.\n// Serving it"),
        "{go}"
    );
}

#[test]
fn the_web_catalog_and_page_carry_the_grouping_and_each_aggregate() {
    let web = emitted(Target::Web);
    assert!(
        web.contains(r#""group_by":["agent_id","channel"]"#) || web.contains(r#""group_by": ["#),
        "{web}"
    );
    assert!(
        web.contains(r#""aggregate":"sum(talk_seconds)""#)
            || web.contains(r#""aggregate": "sum(talk_seconds)""#),
        "{web}"
    );
    assert!(web.contains("count()"), "{web}");
    assert!(web.contains("grouped by ${view.group_by.join"), "{web}");
}
