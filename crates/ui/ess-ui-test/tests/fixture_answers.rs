//! A test declares how each read and command is answered: rows per param set, a failed read, a
//! refused command, a slow read, a placeholder replaced like a view.
use std::path::Path;

use ess_ui_test::Status;

const DOCUMENT: &str = r"format: ess-ui/1
app: answers
model: t
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: orders.draft, sections: [{name: all, pages: [orders.draft]}]}
pages:
  orders.draft:
    kind: detail_page
    title: Drafts
    sections:
      - {name: list, component: collection, reads: {placeholder: example.draft.Page, fixture: drafts.yaml}, columns: [id, name]}
  orders.detail:
    kind: detail_page
    title: Order
    params: {id: string}
    header:
      actions:
        - {name: cancel, label: Cancel, does: example.order.Cancel, bind: {order: params.id}}
    sections:
      - {name: other, component: record, reads: {view: example.order.Page, params: {order: params.id}}, fields: [id, customer]}
      - {name: lines, component: collection, reads: {view: example.line.Page}, columns: [id]}
";

fn run(tests: &str) -> (Vec<ess_ui_test::Outcome>, String) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("ui.yaml");
    std::fs::write(&path, DOCUMENT).unwrap();
    std::fs::write(
        dir.path().join("drafts.yaml"),
        "rows:\n  - {id: d-1, name: One}\n  - {id: d-2, name: Two}\n",
    )
    .unwrap();
    let text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n{tests}",
        path.display()
    );
    let file = ess_ui_test::parse_str(&text, Path::new("answers.test.yaml")).unwrap();
    let report = ess_ui_test::execute(&path, std::slice::from_ref(&file)).unwrap();
    let spec = ess_ui_test::playwright(&[file], &ess_ui::load_str(DOCUMENT).unwrap());
    (report.tests, spec)
}

fn passes(tests: &str) -> String {
    let (outcomes, spec) = run(tests);
    for outcome in &outcomes {
        assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
    }
    spec
}

const ORDERS: &str = "        example.order.Page:
          - {by_params: {order: o-3}, rows: [{id: o-3, customer: Customer 3}]}
          - {by_params: {order: o-4}, rows: [{id: o-4, customer: Customer 4}]}
          - {rows: [{id: o-0, customer: Fallback}]}
        example.line.Page: {rows: [{id: l-1}]}
";

#[test]
fn a_list_of_answers_is_chosen_by_the_params_asked() {
    passes(&format!(
        "  - name: by params
    fixtures:
      views:
{ORDERS}    steps:
      - open: {{page: orders.detail, params: {{id: o-3}}}}
      - expect: {{at: pages/orders.detail/sections/other, text: Customer 3, not_text: Customer 4}}
      - open: {{page: orders.detail, params: {{id: o-4}}}}
      - expect: {{at: pages/orders.detail/sections/other, text: Customer 4}}
      - open: {{page: orders.detail, params: {{id: o-9}}}}
      - expect: {{at: pages/orders.detail/sections/other, text: Fallback}}
"
    ));
}

#[test]
fn no_answer_matching_and_none_without_params_is_no_rows() {
    passes(
        "  - name: nothing
    fixtures:
      views:
        example.order.Page:
          - {by_params: {order: o-3}, rows: [{id: o-3, customer: Customer 3}]}
        example.line.Page: {rows: [{id: l-1}]}
    steps:
      - open: {page: orders.detail, params: {id: o-9}}
      - expect: {at: pages/orders.detail/sections/other, not_text: Customer 3}
",
    );
}

#[test]
fn a_failing_view_shows_failed_and_its_message() {
    passes(
        "  - name: fails
    fixtures:
      views:
        example.order.Page: {fails: backend is down}
        example.line.Page: {rows: [{id: l-1}]}
    steps:
      - open: {page: orders.detail, params: {id: o-3}}
      - expect: {at: pages/orders.detail/sections/other, state: failed, text: backend is down}
      - expect: {at: pages/orders.detail/sections/lines, state: ready}
",
    );
}

#[test]
fn a_refused_command_is_logged_and_shown_where_the_user_acted() {
    passes(&format!(
        "  - name: refuses
    fixtures:
      views:
{ORDERS}      commands:
        example.order.Cancel: {{refuses: already_cancelled, payload: {{at: today}}}}
    steps:
      - open: {{page: orders.detail, params: {{id: o-3}}}}
      - act: pages/orders.detail/header/actions/cancel
      - expect_command: {{command: example.order.Cancel, input: {{order: o-3}}}}
      - expect: {{at: pages/orders.detail, text: already_cancelled}}
"
    ));
}

#[test]
fn latency_by_view_leaves_the_other_sections_ready() {
    passes(&format!(
        "  - name: latency
    latency: {{example.order.Page: 2s}}
    fixtures:
      views:
{ORDERS}    steps:
      - open: {{page: orders.detail, params: {{id: o-3}}}}
      - expect: {{at: pages/orders.detail/sections/other, state: loading}}
      - expect: {{at: pages/orders.detail/sections/lines, state: ready}}
      - advance: 2s
      - expect: {{at: pages/orders.detail/sections/other, state: ready, text: Customer 3}}
"
    ));
}

#[test]
fn a_latency_of_one_duration_still_delays_every_read() {
    passes(
        "  - name: latency
    latency: 2s
    fixtures:
      views:
        example.order.Page: {rows: [{id: o-3, customer: Customer 3}]}
        example.line.Page: {rows: [{id: l-1}]}
    steps:
      - open: {page: orders.detail, params: {id: o-3}}
      - expect: {at: pages/orders.detail/sections/lines, state: loading}
      - advance: 2s
      - expect: {at: pages/orders.detail/sections/lines, state: ready}
",
    );
}

#[test]
fn a_view_fixture_replaces_a_placeholder_read_of_that_name() {
    passes(
        "  - name: placeholder
    fixtures:
      views:
        example.draft.Page: {rows: [{id: d-9, name: Nine}]}
    steps:
      - open: orders.draft
      - expect: {at: pages/orders.draft/sections/list, rows: [d-9]}
",
    );
}

#[test]
fn an_unknown_fixture_key_is_refused_with_file_test_and_key() {
    let text = "format: ess-ui-test/1\ndocument: ui.yaml\ntests:\n  - name: strange\n    fixtures: {refuse: x}\n    steps: [{open: orders.draft}]\n";
    let error = ess_ui_test::parse_str(text, Path::new("odd.test.yaml")).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("odd.test.yaml"), "{message}");
    assert!(message.contains("(strange)"), "{message}");
    assert!(message.contains("`refuse`"), "{message}");
}

fn fixme_count(fixtures_and_latency: &str) -> usize {
    let spec = passes(&format!(
        "  - name: one key
{fixtures_and_latency}    steps:
      - open: {{page: orders.detail, params: {{id: o-3}}}}
"
    ));
    spec.matches("test.fixme").count()
}

#[test]
fn each_new_key_alone_leaves_the_playwright_test_routed() {
    let lines = "    fixtures:
      views:
        example.order.Page:
          - {by_params: {order: o-3}, rows: [{id: o-3, customer: Customer 3}]}
        example.line.Page: {rows: [{id: l-1}]}
";
    assert_eq!(fixme_count(lines), 0, "a list of answers");
    let fails = "    fixtures:
      views:
        example.order.Page: {fails: down}
        example.line.Page: {rows: [{id: l-1}]}
";
    assert_eq!(fixme_count(fails), 0, "fails");
    let commands = "    fixtures:
      views:
        example.order.Page: {rows: [{id: o-3, customer: Customer 3}]}
        example.line.Page: {rows: [{id: l-1}]}
      commands:
        example.order.Cancel: {refuses: gone}
";
    assert_eq!(fixme_count(commands), 0, "commands");
    let latency = "    latency: {example.order.Page: 2s}
    fixtures:
      views:
        example.order.Page: {rows: [{id: o-3, customer: Customer 3}]}
        example.line.Page: {rows: [{id: l-1}]}
";
    assert_eq!(fixme_count(latency), 0, "a latency per view");
    let none = "    fixtures:
      views:
        example.order.Page: {rows: [{id: o-3, customer: Customer 3}]}
        example.line.Page: {rows: [{id: l-1}]}
";
    assert_eq!(fixme_count(none), 0, "the map form");
}

#[test]
fn a_list_answer_refuses_unknown_keys_and_a_by_params_that_is_no_map() {
    for (answer, key) in [
        ("{by_param: {order: o-3}, rows: []}", "`by_param`"),
        ("{by_params: o-3, rows: []}", "`by_params`"),
    ] {
        let text = format!(
            "format: ess-ui-test/1\ndocument: ui.yaml\ntests:\n  - name: typo\n    fixtures: {{views: {{v.Page: [{answer}]}}}}\n    steps: [{{open: orders.draft}}]\n"
        );
        let message = ess_ui_test::parse_str(&text, Path::new("typo.test.yaml"))
            .unwrap_err()
            .to_string();
        assert!(message.contains("typo.test.yaml"), "{message}");
        assert!(message.contains("(typo)"), "{message}");
        assert!(message.contains(key), "{message}");
    }
}

#[test]
fn the_playwright_spec_marks_no_test_with_these_keys_fixme() {
    let spec = passes(&format!(
        "  - name: refuses
    fixtures:
      views:
{ORDERS}      commands:
        example.order.Cancel: {{refuses: already_cancelled}}
    steps:
      - open: {{page: orders.detail, params: {{id: o-3}}}}
  - name: plain
    fixtures:
      views:
        example.draft.Page: {{rows: [{{id: d-9, name: Nine}}]}}
    steps:
      - open: orders.draft
"
    ));
    assert_eq!(spec.matches("test.fixme").count(), 0, "{spec}");
}
