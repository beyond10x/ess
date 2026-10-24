//! `<field> in <list>` and `<field> not in <list>` are not compact predicates; the refusal names the
//! quantifier that asks whether the field is an element of the list
//! (story:guard-tests-list-membership).
use ess_primitives::predicate::Predicate;

#[test]
fn membership_spelling_names_the_exists_quantifier() {
    for (expression, hint) in [
        (
            "action in input.frontier_actions",
            "exists: {in: input.frontier_actions, as: x, that: action == x}",
        ),
        (
            "action not in input.frontier_actions",
            "not: {exists: {in: input.frontier_actions, as: x, that: action == x}}",
        ),
        (
            "  step.action   in   allowed ",
            "exists: {in: allowed, as: x, that: step.action == x}",
        ),
    ] {
        let diagnostic = Predicate::parse_expression(expression)
            .expect_err(expression)
            .to_string();
        assert!(
            diagnostic.contains("a predicate is either a comparison"),
            "{diagnostic}"
        );
        assert!(diagnostic.contains(hint), "{expression}: {diagnostic}");
    }
}

#[test]
fn other_unreadable_text_names_no_quantifier() {
    for expression in [
        "action within input.frontier_actions",
        "contains(input.frontier_actions, action)",
        "in input.frontier_actions",
        "action in",
    ] {
        let diagnostic = Predicate::parse_expression(expression)
            .expect_err(expression)
            .to_string();
        assert!(
            !diagnostic.contains("exists: {in:"),
            "{expression}: {diagnostic}"
        );
    }
}
