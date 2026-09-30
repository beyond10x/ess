//! One verdict in both renderers: wherever the terminal refuses a step, the Playwright spec of
//! the same test is `test.fixme` with the same reason.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

fn document() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/partner-portal/ui.yaml")
        .canonicalize()
        .expect("the example exists")
}

fn file(steps: &[&str]) -> ess_ui_test::TestFile {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: parity\n  steps:\n",
        document().display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    ess_ui_test::parse_str(&text, Path::new("parity.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn run(steps: &[&str]) -> Outcome {
    let report =
        ess_ui_test::execute(&document(), &[file(steps)]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn spec(steps: &[&str]) -> String {
    ess_ui_test::playwright(
        &[file(steps)],
        &ess_ui::load_path(&document()).expect("the document loads"),
    )
}

/// The reasons the spec's `test.fixme` lines give.
fn fixme_reasons(spec: &str) -> Vec<String> {
    spec.lines()
        .filter_map(|line| line.trim().strip_prefix("test.fixme(true, "))
        .filter_map(|rest| rest.strip_suffix(");"))
        .map(|literal| serde_json::from_str::<String>(literal).expect("a JSON string literal"))
        .collect()
}

/// Each case is refused by the terminal, and the spec marks the test `test.fixme` with the reason
/// the terminal gave, stopping there.
#[test]
fn a_step_the_terminal_refuses_is_fixme_in_the_spec_with_the_same_reason() {
    let cases: [(&str, &[&str]); 6] = [
        // An icon action shows no label in the browser.
        (
            "icon action",
            &[
                "open: partners.list",
                "expect: {at: pages/partners.list/sections/list/rows/pt-003/row_actions/edit, text: edit}",
            ],
        ),
        // A cards collection has no column header in the browser.
        (
            "cards header",
            &[
                "open: partners.list",
                "expect: {at: pages/partners.list/sections/list/columns/tier, text: tier}",
            ],
        ),
        // Any step at a cell React does not render, not only `text`.
        (
            "act at an unrendered cell",
            &[
                "open: partners.list",
                "act: pages/partners.list/sections/list/rows/pt-003/columns/name",
            ],
        ),
        // A node inside a row's item has no cells of its own in the terminal.
        (
            "item node",
            &[
                "open: partners.list",
                "expect: {at: pages/partners.list/sections/list/rows/pt-003/item/card, text: Cedar Partners}",
            ],
        ),
        // Past the end of the clock.
        (
            "end of time",
            &[
                "open: overview",
                "advance: 18446744073709551615s",
                "advance: 18446744073709551615s",
            ],
        ),
        // More cycles of a looping script than one move plays.
        ("cycles", &["open: overview", "advance: 180018s"]),
    ];
    for (name, steps) in cases {
        let outcome = run(steps);
        assert_eq!(outcome.status, Status::Failed, "{name}: {outcome:?}");
        let message = outcome.message.clone().unwrap_or_default();
        let spec = spec(steps);
        let reasons = fixme_reasons(&spec);
        assert!(
            reasons
                .iter()
                .any(|reason| message.contains(reason.as_str())),
            "{name}: the terminal refused with {message:?}; the spec's fixme reasons are \
             {reasons:?}:\n{spec}"
        );
        let body = spec
            .split("test.fixme(true, ")
            .nth(1)
            .expect("a fixme line");
        let after = body.split_once('\n').map_or("", |(_, rest)| rest);
        assert!(
            after.trim_start().starts_with("});"),
            "{name}: the spec stops at the refused step:\n{spec}"
        );
    }
}

/// A path that names no node fails in both renderers; it is not a refusal.
#[test]
fn a_path_naming_no_node_is_not_fixme() {
    let steps = [
        "open: partners.list",
        "expect: {at: pages/partners.list/sections/nowhere, text: x}",
    ];
    assert_eq!(run(&steps).status, Status::Failed);
    assert!(fixme_reasons(&spec(&steps)).is_empty());
}

/// Controls: what both renderers show is read, not refused.
#[test]
fn what_both_renderers_show_is_read() {
    for step in [
        // The row action of a row without the cursor reads the label its button shows.
        "expect: {at: pages/tickets.list/sections/list/rows/tk-02/row_actions/open, text: Open}",
        // A table's column header reads its label.
        "expect: {at: pages/tickets.list/sections/list/columns/subject, text: subject}",
        // A row reads its cells.
        "expect: {at: pages/tickets.list/sections/list/rows/tk-01, text: SSO login fails}",
    ] {
        let steps = ["open: tickets.list", step];
        let outcome = run(&steps);
        assert_eq!(outcome.status, Status::Passed, "{step}: {outcome:?}");
        assert!(
            fixme_reasons(&spec(&steps)).is_empty(),
            "{step}: {}",
            spec(&steps)
        );
    }
}

#[test]
fn the_reference_page_states_the_rule() {
    let page = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../website/docs/reference/ess-ui-test.md"),
    )
    .expect("the page is read");
    assert!(
        page.contains(
            "Wherever the terminal refuses a\nstep, the Playwright spec for that test is `test.fixme` with the same reason"
        ),
        "the reference page states the rule"
    );
}
