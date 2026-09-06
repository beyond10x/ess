//! Authored admission must classify semantic and producer limits separately.

use ess_compiler::{compile, ir::EssIr, source::SourceMap};
use ess_conformance::authored::{compile as compile_authored, Source};
use ess_domain::{spec::RawSpecFile, system::Source as SpecSource, Specification};

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/adversary_expression.yaml");

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        SpecSource::new("review.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

fn authored(ir: &EssIr, predicate: &str) -> ess_conformance::authored::Authoring {
    let text = format!("type: ess-scenario/1\ndomain: review.data\nscenario: expression-review\nsummary: Check an authored row expression.\ntimeline:\n  - at: 2026-01-05T09:00:00Z\n    command: review.data.ZObserve\n    outcome: observed\nassert:\n  - view: review.data.Items\n    satisfies: {predicate}\n");
    compile_authored(ir, &[Source::new("review-scenario.yaml", text)])
}

fn refused(ir: &EssIr, predicate: &str, code: &str) {
    let result = authored(ir, predicate);
    assert!(result.scenarios.is_empty(), "{predicate}");
    assert_eq!(
        result.refusals.len(),
        1,
        "{predicate}: {:?}",
        result.refusals
    );
    assert_eq!(
        result.refusals[0].code().to_string(),
        code,
        "{predicate}: {}",
        result.refusals[0]
    );
}

#[test]
fn malformed_bound_operands_refuse_before_the_collection_projection_gap() {
    let ir = ir();
    refused(&ir, "{forall: {in: groups, as: group, that: {exists: {in: group, as: entry, that: entry.count > true}}}}", "ESS-AUTHOR-035");
    refused(&ir, "{forall: {in: groups, as: group, that: {exists: {in: group, as: entry, that: entry.amount > 0}}}}", "ESS-AUTHOR-026");
    refused(
        &ir,
        "{forall: {in: groups.count, as: count, that: true}}",
        "ESS-AUTHOR-035",
    );
}

#[test]
fn authored_rows_preserve_scalar_controls_and_reject_aggregate_and_unprojected_reads() {
    let ir = ir();
    for predicate in [
        "entry.amount >= 0",
        "phase == 'false'",
        "{phase: {none_of: []}}",
        "flag == true",
    ] {
        let result = authored(&ir, predicate);
        assert!(
            result.refusals.is_empty(),
            "{predicate}: {:?}",
            result.refusals
        );
        assert_eq!(result.scenarios.len(), 1);
    }
    for predicate in [
        "{entry: {exists: true}}",
        "loop",
        "param.limit > 0",
        "groups.count > 0",
        "entry.amount_wire > 0",
    ] {
        refused(&ir, predicate, "ESS-AUTHOR-026");
    }
}

#[test]
fn authored_optional_enum_membership_rejects_later_invalid_values() {
    let ir = ir();
    for predicate in [
        "phase == false",
        "{phase: {none_of: [Ready, Unknown]}}",
        "{phase: {any_of: [Ready, false]}}",
        "{all: [true, flag > true]}",
    ] {
        refused(&ir, predicate, "ESS-AUTHOR-035");
    }
}

#[test]
fn authored_semantic_depth_is_distinct_from_the_projection_limit() {
    let ir = ir();
    let path = format!("entry.{}amount", "next.".repeat(40));
    let fields = &ir.views().values().next().unwrap().fields;
    let resolved = ess_compiler::expression::resolve_path(
        &ir,
        fields,
        &ess_primitives::facts::FactPath::new(&path).unwrap(),
        "authored row",
    )
    .unwrap();
    assert!(resolved.optional && resolved.access.depth > 32);
    refused(&ir, &format!("{path} > 0"), "ESS-AUTHOR-026");
}
