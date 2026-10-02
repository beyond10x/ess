//! Steps the example tests do not use: header actions, form fields in an overlay, read latency,
//! closing an overlay, and a beat the scripts never play.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

fn document() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/partner-portal/ui.yaml")
        .canonicalize()
        .expect("the example document exists")
}

fn run(tests: &str) -> Outcome {
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n{tests}",
        document().display()
    );
    let file = ess_ui_test::parse_str(&text, Path::new("steps.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report =
        ess_ui_test::execute(&document(), &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn passed(outcome: &Outcome) {
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn a_header_action_opens_a_form_whose_fields_are_typed_and_submitted() {
    passed(&run(
        "- name: create\n  steps:\n\
         \x20 - open: partners.list\n\
         \x20 - act: pages/partners.list/header/actions/create\n\
         \x20 - expect: {at: pages/partners.list/overlays/create, text: New partner}\n\
         \x20 - type: {at: pages/partners.list/overlays/create/fields/name, text: Gum Tree Ltd}\n\
         \x20 - type: {at: pages/partners.list/overlays/create/fields/website, text: https://gum.example.com}\n\
         \x20 - act: pages/partners.list/overlays/create\n\
         \x20 - expect_command: {command: partners.CreatePartner, input: {name: Gum Tree Ltd, website: https://gum.example.com}}\n",
    ));
}

#[test]
fn a_choice_field_is_chosen_not_typed() {
    let outcome = run("- name: tier\n  steps:\n\
         \x20 - open: partners.list\n\
         \x20 - act: pages/partners.list/header/actions/create\n\
         \x20 - type: {at: pages/partners.list/overlays/create/fields/tier, text: gold}\n");
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
    assert!(
        outcome
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("chosen, not typed"),
        "{outcome:?}"
    );
}

#[test]
fn a_form_choice_field_is_chosen_by_value_or_label() {
    // PartnerTier offers registered, silver, gold, platinum; the field and its choice node both
    // address the field, and a second choice moves from the first.
    passed(&run("- name: tier\n  steps:\n\
         \x20 - open: partners.list\n\
         \x20 - act: pages/partners.list/header/actions/create\n\
         \x20 - type: {at: pages/partners.list/overlays/create/fields/name, text: Gum Tree Ltd}\n\
         \x20 - choose: {at: pages/partners.list/overlays/create/fields/tier, option: platinum}\n\
         \x20 - choose: {at: pages/partners.list/overlays/create/fields/tier/choice, option: silver}\n\
         \x20 - act: pages/partners.list/overlays/create\n\
         \x20 - expect_command: {command: partners.CreatePartner, input: {name: Gum Tree Ltd, tier: silver}}\n"));
    let outcome = run("- name: no such tier\n  steps:\n\
         \x20 - open: partners.list\n\
         \x20 - act: pages/partners.list/header/actions/create\n\
         \x20 - choose: {at: pages/partners.list/overlays/create/fields/tier, option: copper}\n");
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("no option \"copper\""), "{message}");
}

#[test]
fn latency_shows_loading_until_the_clock_passes_it() {
    passed(&run(
        "- name: loading\n  latency: 1s\n  steps:\n\
         \x20 - open: tickets.list\n\
         \x20 - expect: {at: pages/tickets.list/sections/list, state: loading}\n\
         \x20 - advance: 1s\n\
         \x20 - expect: {at: pages/tickets.list/sections/list, state: ready}\n",
    ));
}

#[test]
fn a_replaced_view_without_rows_shows_empty() {
    passed(&run(
        "- name: empty\n  fixtures:\n    views:\n      tickets.Page: {rows: []}\n  steps:\n\
         \x20 - open: tickets.list\n\
         \x20 - expect: {at: pages/tickets.list/sections/list, state: empty}\n",
    ));
}

#[test]
fn selecting_the_page_closes_an_open_overlay_and_steps_behind_it_are_refused() {
    let outcome = run("- name: behind\n  steps:\n\
         \x20 - open: partners.list\n\
         \x20 - act: pages/partners.list/sections/list/rows/pt-003/row_actions/delete\n\
         \x20 - select: pages/partners.list/sections/list\n");
    assert_eq!(outcome.step, Some(3), "{outcome:?}");
    assert!(
        outcome
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("an overlay is open"),
        "{outcome:?}"
    );
    passed(&run(
        "- name: closed\n  steps:\n\
         \x20 - open: partners.list\n\
         \x20 - act: pages/partners.list/sections/list/rows/pt-003/row_actions/delete\n\
         \x20 - select: pages/partners.list\n\
         \x20 - expect: {at: pages/partners.list, not_text: Delete partner}\n\
         \x20 - act: pages/partners.list/sections/list/rows/pt-001/row_actions/open\n",
    ));
}

#[test]
fn a_beat_no_script_plays_fails_naming_it() {
    let outcome = run("- name: never\n  steps:\n\
         \x20 - open: overview\n\
         \x20 - play: {channel: metrics, lifecycle: closed}\n");
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    assert!(
        outcome
            .message
            .as_deref()
            .unwrap_or_default()
            .contains("play no lifecycle closed on metrics"),
        "{outcome:?}"
    );
}

#[test]
fn a_path_on_another_page_names_both_pages() {
    let outcome = run("- name: elsewhere\n  steps:\n\
         \x20 - open: overview\n\
         \x20 - expect: {at: pages/partners.list/sections/list, rows: 6}\n");
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(
        message.contains("is on page partners.list; the page shown is overview"),
        "{message}"
    );
}
