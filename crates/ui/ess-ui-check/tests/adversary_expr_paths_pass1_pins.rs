//! Adversarial pass 1: cases the unit's suite leaves to no test. Each holds behaviour the unit
//! has, at a branch of `sites.rs` or `expr::read` that no case of `expression_paths.rs` or
//! `checks.rs` reaches, so a mutant there stays green: a map key typed `expr` (an action's `sets`,
//! a guard's `then.set`), a `one_of` type (a guard's `then`), and a number written as a string.

use std::path::PathBuf;

use ess_ui_check::{check_source, Finding, Options, Report};

fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn run(text: &str) -> Report {
    check_source(text, "test.yaml", &crate_dir(), None, &Options::default())
}

fn of<'a>(report: &'a Report, check: &str) -> Vec<&'a Finding> {
    report
        .findings
        .iter()
        .filter(|finding| finding.check == check)
        .collect()
}

/// Page `p` with one row action `action` on its collection, and a shell with `guards`.
fn document(action: &str, guards: &str, read: &str) -> String {
    format!(
        "format: ess-ui/1\napp: t\nmodel: shop\nplacement_profile: fat\n\
         shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}, \
         state: {{theme: {{type: string, class: preference, store: local_storage}}}}, \
         guards: [{guards}]}}}}\n\
         navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}\n\
         pages:\n  p:\n    kind: detail_page\n    title: P\n    \
         state: {{search: {{type: string, class: page_state}}}}\n    sections:\n      \
         - {{name: deals, component: collection, reads: {{view: t.Deals, params: {{q: {read}}}}}, \
         columns: [id], row_actions: [{action}]}}\n"
    )
}

const FINE_ACTION: &str = "{name: star, label: Star, sets: {state.search: toggle}}";
const FINE_GUARD: &str = "{name: g, when: url.query.x, then: {set: {shell.theme: url.query.x}}}";

#[test]
fn the_document_is_clean() {
    let report = run(&document(FINE_ACTION, FINE_GUARD, "state.search"));
    assert!(
        of(&report, "expr_paths").is_empty(),
        "{:#?}",
        report.findings
    );
    assert!(
        of(&report, "expr_roots").is_empty(),
        "{:#?}",
        report.findings
    );
}

/// `sets` is `{map: {key: expr, …}}`: the key is the state written.
#[test]
fn an_undeclared_state_a_sets_key_writes_is_an_error() {
    let report = run(&document(
        "{name: star, label: Star, sets: {state.nosuch: toggle}}",
        FINE_GUARD,
        "state.search",
    ));
    let found = of(&report, "expr_paths");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "pages/p/sections/deals/row_actions/star");
    assert!(found[0].message.contains("`state.nosuch`"), "{found:#?}");
}

/// A guard's `then` is a `one_of`, and its `set` a map keyed by `expr`.
#[test]
fn an_undeclared_shell_state_a_guard_sets_is_an_error() {
    let report = run(&document(
        FINE_ACTION,
        "{name: g, when: url.query.x, then: {set: {shell.nosuch: url.query.x}}}",
        "state.search",
    ));
    let found = of(&report, "expr_paths");
    assert_eq!(found.len(), 1, "{:#?}", report.findings);
    assert_eq!(found[0].path, "shells/app/guards/g");
    assert!(found[0].message.contains("`shell.nosuch`"), "{found:#?}");
}

/// The unit's own case for "a number draws nothing" writes `3` and `1.5` unquoted in a flow
/// mapping: YAML reads them as numbers, so `expr::read` never sees them. As strings they reach it.
#[test]
fn a_number_written_as_a_string_draws_no_warning() {
    let parsed: ess_yaml::Value = ess_yaml::from_str("{a: 1.5, b: 3}").expect("yaml");
    assert!(
        parsed["a"].as_str().is_none() && parsed["b"].as_str().is_none(),
        "unquoted, they are no strings: {parsed:?}"
    );
    for number in ["'1.5'", "'-2.5'", "'10.0.0.1'", "'3'"] {
        let report = run(&document(FINE_ACTION, FINE_GUARD, number));
        assert!(
            of(&report, "expr_roots").is_empty(),
            "{number}: {:#?}",
            report.findings
        );
    }
    let report = run(&document(FINE_ACTION, FINE_GUARD, "'v1.5'"));
    assert_eq!(
        of(&report, "expr_roots").len(),
        1,
        "control, a dotted word that is no number: {:#?}",
        report.findings
    );
}
