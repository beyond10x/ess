//! The predicate grammar is shared, and `infra-spec/1` does not gain distinct list members
//! (`ess/22`, beyond10x/ess#237) by accident: admitting them is that format's own decision.

use infra_domain::code::InfraCode;
use infra_spec::read_spec;

fn document(predicate: &str) -> String {
    format!(
        "format: infra-spec/1\nname: fixture\nexpectations:\n  - id: a\n    expect:\n      workload_predicate: {predicate}\n"
    )
}

#[test]
fn distinct_is_refused_because_infra_spec_1_does_not_carry_it() {
    for predicate in [
        "{distinct: {in: workload.containers, as: container, kind: string}}",
        "{all: [workload.replicas >= 2, {not: {distinct: {in: workload.labels, as: label}}}]}",
    ] {
        let refused = read_spec(&document(predicate)).expect_err(predicate);
        assert!(
            refused.contains(InfraCode::SpecInvalidExpectation)
                && refused.to_string().contains("distinct"),
            "{predicate}: {refused}"
        );
    }
}

#[test]
fn a_workload_fact_named_distinct_keeps_its_meaning() {
    let refused = read_spec(&document("{distinct: {eq: api}}")).expect_err("not a workload fact");
    assert!(
        !refused.contains(InfraCode::SpecInvalidExpectation),
        "refused as an unknown fact, as before: {refused}"
    );
}
