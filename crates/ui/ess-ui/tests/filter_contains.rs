//! `contains` in a client row filter: a search over the loaded rows, as a list page whose server
//! read takes no search value narrows what it shows.

fn keeps(text: &str, scope: &str) -> bool {
    let scope: ess_yaml::Value = ess_yaml::from_str(scope).unwrap();
    ess_ui::filter::keeps(text, &|path| {
        let mut value = &scope;
        for segment in path {
            value = &value[segment.as_str()];
        }
        value.clone()
    })
}

const FILTER: &str = "row.email contains state.search";

#[test]
fn contains_keeps_a_row_whose_text_holds_every_word_of_the_search_in_any_case() {
    let row = "row: {email: Alex.Example@Example.com}";
    assert!(keeps(
        FILTER,
        &format!("{{{row}, state: {{search: alex}}}}")
    ));
    assert!(keeps(
        FILTER,
        &format!("{{{row}, state: {{search: 'EXAMPLE.COM alex'}}}}")
    ));
    assert!(!keeps(
        FILTER,
        &format!("{{{row}, state: {{search: 'alex sam'}}}}")
    ));
    assert!(!keeps(
        FILTER,
        &format!("{{{row}, state: {{search: sam}}}}")
    ));
}

#[test]
fn contains_keeps_every_row_while_the_search_is_empty() {
    let row = "row: {email: alex@example.com}";
    for search in ["''", "'   '", "null"] {
        assert!(
            keeps(FILTER, &format!("{{{row}, state: {{search: {search}}}}}")),
            "an empty search ({search}) hides no row"
        );
    }
    assert!(keeps(FILTER, &format!("{{{row}, state: {{}}}}")));
}

#[test]
fn contains_reads_a_missing_field_as_empty_text() {
    assert!(!keeps(FILTER, "{row: {}, state: {search: alex}}"));
    assert!(keeps(FILTER, "{row: {}, state: {search: ''}}"));
}

#[test]
fn contains_is_a_comparison_that_combines_and_negates() {
    let scope = "{row: {email: alex@example.com, enabled: true}, state: {search: alex}}";
    assert!(keeps(
        "row.enabled == true and row.email contains state.search",
        scope
    ));
    assert!(!keeps("not (row.email contains state.search)", scope));
    let parsed = ess_ui::filter::parse(FILTER).expect("contains parses");
    assert!(parsed.boolean() && parsed.allowed_roots());
    assert_eq!(
        parsed.paths(),
        vec![
            &["row".to_owned(), "email".to_owned()][..],
            &["state".to_owned(), "search".to_owned()][..]
        ]
    );
}

#[test]
fn contains_needs_both_sides() {
    assert!(ess_ui::filter::parse("row.email contains").is_none());
    assert!(ess_ui::filter::parse("contains state.search").is_none());
}
