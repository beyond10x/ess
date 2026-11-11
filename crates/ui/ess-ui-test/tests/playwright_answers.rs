//! The Playwright spec routes each declared fixture answer: a list of view answers by params, a
//! failing view, a refused command and a latency per view are answered by `page.route`, and no
//! test using them is `test.fixme`. The spec is generated text; no browser runs here.
use std::path::Path;

const DOCUMENT: &str = r"format: ess-ui/1
app: answers
model: t
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: orders.detail, sections: [{name: all, pages: [orders.detail]}]}
pages:
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

fn spec(tests: &str) -> String {
    let text = format!("format: ess-ui-test/1\ndocument: ui.yaml\ntests:\n{tests}");
    let file = ess_ui_test::parse_str(&text, Path::new("answers.test.yaml")).unwrap();
    ess_ui_test::playwright(&[file], &ess_ui::load_str(DOCUMENT).unwrap())
}

const STEPS: &str = "    steps:
      - open: {page: orders.detail, params: {id: o-3}}
";

fn no_fixme(spec: &str) {
    assert!(!spec.contains("test.fixme"), "{spec}");
}

#[test]
fn a_list_of_answers_is_one_handler_choosing_by_params() {
    let spec = spec(&format!(
        "  - name: by params
    fixtures:
      views:
        example.order.Page:
          - {{by_params: {{order: o-3}}, rows: [{{id: o-3}}]}}
          - {{rows: [{{id: o-0}}]}}
{STEPS}"
    ));
    no_fixme(&spec);
    assert_eq!(spec.matches("page.route(").count(), 1, "{spec}");
    assert!(spec.contains("\"by_params\":{\"order\":\"o-3\"}"), "{spec}");
    assert!(spec.contains("searchParams.get(\"params\")"), "{spec}");
    assert!(
        spec.contains("answers.find((answer) => answer.by_params"),
        "{spec}"
    );
    assert!(
        spec.contains("?? answers.find((answer) => !answer.by_params)"),
        "{spec}"
    );
    assert!(spec.contains("{ rows: [] }"), "{spec}");
}

#[test]
fn a_failing_view_is_status_500_with_its_message() {
    let spec = spec(&format!(
        "  - name: fails
    fixtures:
      views:
        example.order.Page: {{fails: backend is down}}
{STEPS}"
    ));
    no_fixme(&spec);
    assert!(
        spec.contains("route.fulfill({ status: 500, contentType: \"text/plain\", body: \"backend is down\" })"),
        "{spec}"
    );
}

#[test]
fn a_refused_command_is_the_refusal_body_the_http_adapter_reads() {
    let spec = spec(
        "  - name: refuses
    fixtures:
      commands:
        example.order.Cancel: {refuses: already_cancelled, payload: {at: today}}
    steps:
      - open: {page: orders.detail, params: {id: o-3}}
      - act: pages/orders.detail/header/actions/cancel
      - expect_command: {command: example.order.Cancel, input: {order: o-3}}
",
    );
    no_fixme(&spec);
    assert!(spec.contains("commands.push("), "{spec}");
    assert!(
        spec.contains("\"outcome\":\"refused\"")
            && spec.contains("\"error\":\"already_cancelled\""),
        "{spec}"
    );
    assert!(spec.contains("\"payload\":{\"at\":\"today\"}"), "{spec}");
    assert!(spec.contains("status: 422"), "{spec}");
}

#[test]
fn a_refused_command_without_expect_command_is_still_routed() {
    let spec = spec(&format!(
        "  - name: refuses
    fixtures:
      commands:
        example.order.Cancel: {{refuses: gone}}
{STEPS}"
    ));
    no_fixme(&spec);
    assert!(spec.contains("\"error\":\"gone\""), "{spec}");
}

#[test]
fn a_latency_map_delays_each_named_views_route_on_the_page_clock() {
    let spec = spec(&format!(
        "  - name: latency
    latency: {{example.order.Page: 2s, example.line.Page: 500ms}}
    fixtures:
      views:
        example.order.Page: {{rows: [{{id: o-3}}]}}
{STEPS}"
    ));
    no_fixme(&spec);
    let delay = |ms: u32| format!("setTimeout(resolve, ms)), {ms});");
    assert!(spec.contains(&delay(2000)), "{spec}");
    assert!(spec.contains(&delay(500)), "{spec}");
    assert!(spec.contains("route.continue()"), "{spec}");
    assert!(
        spec.find("page.route").unwrap() < spec.find("page.clock.install").unwrap(),
        "{spec}"
    );
}

#[test]
fn a_test_using_no_new_key_emits_what_it_emitted_before() {
    let spec = spec(&format!(
        "  - name: plain
    fixtures:
      views:
        example.order.Page: {{rows: [{{id: o-3}}]}}
{STEPS}"
    ));
    assert_eq!(spec.matches("page.route(").count(), 1, "{spec}");
    assert!(
        spec.contains("await page.route((url) => url.pathname.endsWith(\"/views/example.order.Page\"), (route) => route.fulfill({ json: {\"rows\":[{\"id\":\"o-3\"}]} }));"),
        "{spec}"
    );
    no_fixme(&spec);
}
