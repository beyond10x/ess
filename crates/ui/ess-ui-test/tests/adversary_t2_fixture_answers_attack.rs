//! Adversary cases for the per-test answers of a ui test: `by_params` lists, `fails`, `refuses`,
//! a latency per view, and the Playwright spec routing each key on its own.
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
    sections:
      - name: other
        component: record
        reads: {view: example.order.Page, params: {order: params.id}}
        fields: [id, customer]
        actions:
          - {name: cancel, label: Cancel, does: example.order.Cancel, bind: {order: row.id}}
      - {name: lines, component: collection, reads: {view: example.line.Page}, columns: [id]}
      - {name: more, component: collection, reads: {view: example.line.Page}, columns: [id]}
";

fn parse(tests: &str) -> Result<ess_ui_test::TestFile, String> {
    let text = format!("format: ess-ui-test/1\ndocument: ui.yaml\ntests:\n{tests}");
    ess_ui_test::parse_str(&text, Path::new("attack.test.yaml")).map_err(|e| e.to_string())
}

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
    let file = ess_ui_test::parse_str(&text, Path::new("attack.test.yaml")).unwrap();
    let report = ess_ui_test::execute(&path, std::slice::from_ref(&file)).unwrap();
    let spec = ess_ui_test::playwright(&[file], &ess_ui::load_str(DOCUMENT).unwrap());
    (report.tests, spec)
}

fn passes(tests: &str) -> String {
    let (outcomes, spec) = run(tests);
    assert!(!outcomes.is_empty(), "{outcomes:?}");
    for outcome in &outcomes {
        assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
    }
    spec
}

/// A typo in a keyed answer (`by_param`) silently turns it into the fallback answer: every read
/// is then answered by it, and the test exercises the wrong answer without a word. An unknown
/// key under `fixtures` is refused with file, test and key everywhere else.
#[test]
fn an_unknown_key_in_a_list_answer_is_refused() {
    let error = parse(
        "  - name: typo
    fixtures:
      views:
        example.order.Page:
          - {by_param: {order: o-3}, rows: [{id: o-3, customer: Customer 3}]}
    steps: [{open: orders.draft}]
",
    )
    .err()
    .unwrap_or_else(|| panic!("`by_param` in a list answer was accepted"));
    assert!(error.contains("attack.test.yaml"), "{error}");
    assert!(error.contains("typo"), "{error}");
    assert!(error.contains("`by_param`"), "{error}");
}

/// `by_params` that is not a map is read as no `by_params` at all: the keyed answer becomes the
/// answer to every read.
#[test]
fn a_list_answer_whose_by_params_is_not_a_map_is_refused() {
    let result = parse(
        "  - name: scalar
    fixtures:
      views:
        example.order.Page:
          - {by_params: o-3, rows: [{id: o-3, customer: Customer 3}]}
    steps: [{open: orders.draft}]
",
    );
    assert!(
        result.is_err(),
        "`by_params: o-3` was accepted and answers every read"
    );
}

#[test]
fn an_unknown_key_under_a_command_names_file_test_and_key() {
    let error = parse(
        "  - name: zebra
    fixtures:
      commands:
        example.order.Cancel: {refuse: already_cancelled}
    steps: [{open: orders.draft}]
",
    )
    .expect_err("an unknown key under a command is refused");
    assert!(error.contains("attack.test.yaml"), "{error}");
    assert!(error.contains("zebra"), "{error}");
    assert!(error.contains("`refuse`"), "{error}");
}

/// Numbers and strings compare as text; an answer keyed on a param the read does not ask is not
/// chosen; of two answers that both match, the first is.
#[test]
fn by_params_compares_as_text_and_needs_every_param_asked() {
    passes(
        "  - name: number
    fixtures:
      views:
        example.order.Page:
          - {by_params: {order: o-3, status: open}, rows: [{id: o-3, customer: Needs Status}]}
          - {by_params: {order: 7}, rows: [{id: '7', customer: Seven First}]}
          - {by_params: {order: '7'}, rows: [{id: '7', customer: Seven Second}]}
          - {rows: [{id: o-0, customer: Fallback}]}
        example.line.Page: {rows: [{id: l-1}]}
    steps:
      - open: {page: orders.detail, params: {id: '7'}}
      - expect: {at: pages/orders.detail/sections/other, text: Seven First, not_text: Seven Second}
      - open: {page: orders.detail, params: {id: o-3}}
      - expect: {at: pages/orders.detail/sections/other, text: Fallback, not_text: Needs Status}
",
    );
}

/// `fails` fails every read of the view: both sections reading it, and a placeholder of that
/// name, without touching its fixture file.
#[test]
fn fails_fails_every_section_and_a_placeholder() {
    passes(
        "  - name: fails everywhere
    fixtures:
      views:
        example.order.Page: {rows: [{id: o-3, customer: Customer 3}]}
        example.line.Page: {fails: lines are down}
        example.draft.Page: {fails: drafts are down}
    steps:
      - open: {page: orders.detail, params: {id: o-3}}
      - expect: {at: pages/orders.detail/sections/lines, state: failed, text: lines are down}
      - expect: {at: pages/orders.detail/sections/more, state: failed, text: lines are down}
      - expect: {at: pages/orders.detail/sections/other, state: ready, text: Customer 3}
      - open: orders.draft
      - expect: {at: pages/orders.draft/sections/list, state: failed, text: drafts are down}
",
    );
}

/// The refusal shows in the record the user acted from, with its payload, and the command is
/// still logged with the row's key.
#[test]
fn a_refusal_shows_in_the_record_the_user_acted_from() {
    passes(
        "  - name: refused in place
    fixtures:
      views:
        example.order.Page: {rows: [{id: o-3, customer: Customer 3}]}
        example.line.Page: {rows: [{id: l-1}]}
      commands:
        example.order.Cancel: {refuses: already_cancelled, payload: {at: today}}
    steps:
      - open: {page: orders.detail, params: {id: o-3}}
      - act: pages/orders.detail/sections/other/actions/cancel
      - expect_command: {command: example.order.Cancel, input: {order: o-3}}
      - expect: {at: pages/orders.detail/sections/other, text: already_cancelled}
      - expect: {at: pages/orders.detail/sections/other, text: today}
      - expect: {at: pages/orders.detail/sections/lines, not_text: already_cancelled}
",
    );
}

/// A latency per view holds that view `loading` through a partial advance; a view not named
/// answers at once.
#[test]
fn latency_by_view_holds_through_a_partial_advance() {
    passes(
        "  - name: partial
    latency: {example.order.Page: 2s, example.draft.Page: 1s}
    fixtures:
      views:
        example.order.Page: {rows: [{id: o-3, customer: Customer 3}]}
        example.line.Page: {rows: [{id: l-1}]}
    steps:
      - open: {page: orders.detail, params: {id: o-3}}
      - expect: {at: pages/orders.detail/sections/other, state: loading}
      - expect: {at: pages/orders.detail/sections/more, state: ready}
      - advance: 1500ms
      - expect: {at: pages/orders.detail/sections/other, state: loading}
      - advance: 500ms
      - expect: {at: pages/orders.detail/sections/other, state: ready, text: Customer 3}
      - open: orders.draft
      - expect: {at: pages/orders.draft/sections/list, state: loading}
      - advance: 1s
      - expect: {at: pages/orders.draft/sections/list, state: ready, rows: [d-1, d-2]}
",
    );
}

fn routed_once(tests: &str, wanted: &str) {
    let spec = passes(tests);
    assert_eq!(spec.matches("test.fixme").count(), 0, "{spec}");
    assert!(spec.contains(wanted), "{spec}");
}

/// Each key alone is routed, never `test.fixme` (the unit's own case combines a list and a
/// command, so dropping any single detection survives it).
#[test]
fn each_key_alone_is_routed_not_fixme() {
    routed_once(
        "  - name: fails only
    fixtures:
      views:
        example.order.Page: {fails: down}
    steps:
      - open: orders.draft
",
        "status: 500",
    );
    routed_once(
        "  - name: list only
    fixtures:
      views:
        example.order.Page:
          - {rows: [{id: o-0, customer: Fallback}]}
    steps:
      - open: orders.draft
",
        "answers.find",
    );
    routed_once(
        "  - name: latency only
    latency: {example.order.Page: 2s}
    steps:
      - open: orders.draft
",
        "setTimeout(resolve, ms)",
    );
    routed_once(
        "  - name: command only
    fixtures:
      commands:
        example.order.Cancel: {refuses: nope}
    steps:
      - open: orders.draft
",
        "\"error\":\"nope\"",
    );
}
