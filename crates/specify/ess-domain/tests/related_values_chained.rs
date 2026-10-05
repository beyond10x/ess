//! `{related: …}` through an Optional reference, and across two references (source format
//! `ess/22`, beyond10x/ess#285, `docs/design/value-expressions.md` E8).
//!
//! The reduction is the issue's: a cost entry names its objective, the objective names an Optional
//! initiative, the initiative names an outcome, and `Book` copies that outcome onto the entry.

const CHAINED: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-values-chained.yaml");

/// The source `Book` fills `outcome_id` from, in the payload and in `sets:` alike.
const TWO_HOPS: &str = "{related: {via: [objective_id, initiative_id], field: outcome_id}}";

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("costs.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn admitted(body: &str) {
    spec(body).unwrap_or_else(|error| panic!("{error}\n{body}"));
}

fn refused(body: &str, expected: &[&str]) {
    let error = spec(body)
        .err()
        .unwrap_or_else(|| panic!("must not validate:\n{body}"));
    for needle in expected {
        assert!(error.contains(needle), "expected {needle:?} in:\n{error}");
    }
}

/// The reduction with `Book`'s source replaced by `source` and the format by `format`.
fn costs(format: &str, source: &str) -> String {
    CHAINED
        .replace("format: ess/22", &format!("format: {format}"))
        .replace(TWO_HOPS, source)
}

/// The issue's own one-hop form: the entry's Optional `initiative_id`, set from an Optional input.
const ONE_OPTIONAL_HOP: &str = "{related: {via: initiative_id, field: outcome_id}}";

#[test]
fn issue_285_the_two_hop_reduction_validates_under_ess_22() {
    admitted(CHAINED);
}

#[test]
fn issue_285_the_filed_one_hop_optional_reduction_validates_under_ess_22() {
    admitted(&costs("ess/22", ONE_OPTIONAL_HOP));
    // Through the input as well as through the field the branch sets from it.
    admitted(&costs(
        "ess/22",
        "{related: {via: input.initiative_id, field: outcome_id}}",
    ));
}

#[test]
fn below_ess_22_an_optional_reference_is_refused_naming_ess_22() {
    for format in ["ess/21", "ess/16"] {
        refused(
            &costs(format, ONE_OPTIONAL_HOP),
            &["type_mismatch", "`initiative_id`", "Optional", "ess/22"],
        );
    }
}

#[test]
fn below_ess_22_a_chained_via_is_refused_naming_ess_22() {
    for format in ["ess/21", "ess/16"] {
        refused(
            &costs(format, TWO_HOPS),
            &["unsupported_format_version", "ess/22"],
        );
    }
}

#[test]
fn a_chained_via_names_exactly_two_references() {
    refused(
        &costs(
            "ess/22",
            "{related: {via: [objective_id], field: outcome_id}}",
        ),
        &["two"],
    );
    refused(
        &costs(
            "ess/22",
            "{related: {via: [objective_id, initiative_id, outcome_id], field: outcome_id}}",
        ),
        &["two"],
    );
}

#[test]
fn the_second_reference_is_a_field_of_the_entity_the_first_names() {
    refused(
        &costs(
            "ess/22",
            "{related: {via: [objective_id, nope], field: outcome_id}}",
        ),
        &[
            "undeclared_reference",
            "`nope` is not a field of `demo.costs.Objective`",
        ],
    );
    refused(
        &costs(
            "ess/22",
            "{related: {via: [objective_id, input.initiative_id], field: outcome_id}}",
        ),
        &["undeclared_reference"],
    );
}

#[test]
fn every_reference_in_a_chain_names_one_entity() {
    // `Initiative.outcome_id` is no entity's identity, so it cannot be followed.
    refused(
        &CHAINED.replace(
            TWO_HOPS,
            "{related: {via: [initiative_id, outcome_id], field: outcome_id}}",
        ),
        &["type_mismatch", "no entity's identity"],
    );
}

#[test]
fn the_field_read_is_a_field_of_the_last_entity() {
    refused(
        &costs(
            "ess/22",
            "{related: {via: [objective_id, initiative_id], field: nope}}",
        ),
        &[
            "undeclared_reference",
            "`nope` is not a field of `demo.costs.Initiative`",
        ],
    );
}

#[test]
fn a_value_that_may_be_absent_is_refused_into_a_required_target() {
    // `CostEntry.outcome_id` made required: an absent initiative would leave it nothing.
    for source in [TWO_HOPS, ONE_OPTIONAL_HOP] {
        let body = costs("ess/22", source).replace(
            "      - {name: outcome_id, type: 'Optional<demo.costs.OutcomeId>'}\n      - {name: cents",
            "      - {name: outcome_id, type: demo.costs.OutcomeId}\n      - {name: cents",
        );
        refused(&body, &["type_mismatch", "absent"]);
    }
}

#[test]
fn a_chain_of_required_references_fills_a_required_target() {
    // Every link required: the value is always there.
    let body = CHAINED
        .replace(
            "      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}\n    lifecycle: {initial: Active",
            "      - {name: initiative_id, type: demo.costs.InitiativeId}\n    lifecycle: {initial: Active",
        )
        .replace(
            "  - name: demo.costs.SetObjective\n    input:\n      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}",
            "  - name: demo.costs.SetObjective\n    input:\n      - {name: initiative_id, type: demo.costs.InitiativeId}",
        );
    admitted(&body);
    let required = body.replace(
        "      - {name: outcome_id, type: 'Optional<demo.costs.OutcomeId>'}\n      - {name: cents",
        "      - {name: outcome_id, type: demo.costs.OutcomeId}\n      - {name: cents",
    );
    admitted(&required);
}

#[test]
fn a_list_reference_is_still_refused() {
    let body = CHAINED.replace(
        "      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}\n    lifecycle: {initial: Active",
        "      - {name: initiative_id, type: 'Optional<demo.costs.InitiativeId>'}\n      - {name: initiatives, type: 'List<demo.costs.InitiativeId>'}\n    lifecycle: {initial: Active",
    );
    refused(
        &body.replace(
            TWO_HOPS,
            "{related: {via: [objective_id, initiatives], field: outcome_id}}",
        ),
        &[
            "type_mismatch",
            "`demo.costs.Objective.initiatives`",
            "List",
        ],
    );
}

#[test]
fn a_list_is_read_only_as_the_via_of_a_related_source() {
    refused(
        &CHAINED.replace(
            "          cents: input.cents",
            "          cents: [input.cents, input.cents]",
        ),
        &["list"],
    );
}

#[test]
fn issue_285_the_published_schema_admits_a_chain_of_exactly_two() {
    let schema: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../schemas/generated/ess.schema.json"
    ))
    .expect("the schema is JSON");
    let chain = schema["definitions"]["RawRelatedVia"]["anyOf"]
        .as_array()
        .expect("`via:` is one reference or a chain")
        .iter()
        .find(|form| form["type"] == "array")
        .expect("a chain is an array");
    assert_eq!(chain["minItems"], 2, "{chain}");
    assert_eq!(chain["maxItems"], 2, "{chain}");
}
