//! The predicate grammar is shared, and `infra-spec/1` does not gain the case-insensitive text
//! operators (beyond10x/ess#140) by accident: admitting them is that format's own decision.

use infra_domain::code::InfraCode;
use infra_spec::read_spec;

fn document(predicate: &str) -> String {
    format!(
        "format: infra-spec/1\nname: fixture\nexpectations:\n  - id: a\n    expect:\n      workload_predicate: {predicate}\n"
    )
}

#[test]
fn a_case_insensitive_operator_is_refused_because_infra_spec_1_does_not_carry_it() {
    for predicate in [
        "{workload.name: {equals_ignore_case: api}}",
        "{not: {workload.name: {in_ignore_case: [api, web]}}}",
        "{all: [workload.replicas >= 2, {workload.name: {equals_ignore_case: web}}]}",
    ] {
        let refused = read_spec(&document(predicate)).expect_err(predicate);
        assert!(
            refused.contains(InfraCode::SpecInvalidExpectation),
            "{predicate}: {refused}"
        );
    }
}

#[test]
fn the_same_expectation_with_an_equality_is_admitted() {
    read_spec(&document("{workload.name: {eq: api}}")).expect("an equality is infra-spec/1");
}

/// `trimmed:` (beyond10x/ess#532, ess/25) is not carried either, wrapping a fold or not, and the
/// refusal names it rather than the operator it wraps.
#[test]
fn a_trimmed_operator_is_refused_by_name_because_infra_spec_1_does_not_carry_it() {
    for predicate in [
        "{workload.name: {trimmed: {equals: api}}}",
        "{not: {workload.name: {trimmed: {in_ignore_case: [api, web]}}}}",
        "{all: [workload.replicas >= 2, {workload.name: {trimmed: {starts_with: w}}}]}",
    ] {
        let refused = read_spec(&document(predicate)).expect_err(predicate);
        assert!(
            refused.contains(InfraCode::SpecInvalidExpectation)
                && refused
                    .to_string()
                    .contains("`trimmed` is not part of infra-spec/1"),
            "{predicate}: {refused}"
        );
    }
}
