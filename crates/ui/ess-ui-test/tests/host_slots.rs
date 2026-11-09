//! A host slot in `ess-ui-test/1`. The terminal draws a slot as a placeholder region at the
//! slot's path labelled `<host>: <reason>`, so a `text` check reads that label there. The host
//! draws what is inside the region, in both renderers, so a step that acts inside it — at the
//! slot's path or a path below it — is refused by one rule, by name: in the terminal run, and as
//! `test.fixme` in the Playwright spec for the generated React project.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
      overlay: {kind: overlay_outlet}
navigation: {home: p, sections: [{name: all, pages: [p, grid]}]}
pages:
  p:
    kind: static_page
    title: P
    header: {actions: [{name: pick, opens: picker, label: Pick}]}
    sections:
      - {name: map, host: RouteMap, reason: a map widget, stands_in_for: chart}
      - name: form
        component: form
        does: t.Save
        fields:
          - {field: title, label: Title}
          - {field: route, label: Route, host: RoutePicker, reason: a vendor route picker, stands_in_for: input}
        children:
          - {name: preview, host: RoutePreview, reason: a vendor preview, stands_in_for: none}
    overlays:
      picker: {kind: drawer, title: Pick, host: TruckPicker, reason: a fleet picker, stands_in_for: collection}
  grid: {kind: static_page, title: Grid, host: VendorGrid, reason: a vendor grid, stands_in_for: collection}
";

const MAP: &str = "pages/p/sections/map";
const FORM: &str = "pages/p/sections/form";

fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-host-slots")
        .join(name);
    std::fs::create_dir_all(&dir).expect("the dir is made");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    std::fs::write(dir.join("route.json"), "{}\n").expect("the upload file is written");
    dir.join("ui.yaml")
}

fn file(name: &str, steps: &[&str]) -> ess_ui_test::TestFile {
    let mut text = format!(
        "format: ess-ui-test/1\ndocument: {}\ntests:\n- name: slot\n  steps:\n",
        document(name).display()
    );
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    ess_ui_test::parse_str(&text, Path::new("host_slots.yaml"))
        .unwrap_or_else(|error| panic!("{error}\n{text}"))
}

fn run(name: &str, steps: &[&str]) -> Outcome {
    let report = ess_ui_test::execute(&document(name), &[file(name, steps)])
        .unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn spec(name: &str, steps: &[&str]) -> String {
    ess_ui_test::playwright(
        &[file(name, steps)],
        &ess_ui::load_path(&document(name)).expect("the document loads"),
    )
}

/// The refusal of a step acting at `path`, inside host slot `host`.
fn inside(path: &str, host: &str) -> String {
    format!("`{path}` is host slot `{host}`: the host draws it, so a test cannot act inside it")
}

/// The test failed at step 2, the step after `open`, with exactly `wanted` as the reason.
fn assert_refused_at_step_two(outcome: &Outcome, wanted: &str) {
    assert_eq!(outcome.status, Status::Failed, "{outcome:?}");
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let message = outcome.message.as_deref().unwrap_or_default();
    assert!(
        message.ends_with(&format!(": {wanted}")),
        "expected the refusal\n  {wanted}\nfound\n  {message}"
    );
}

// ── text ────────────────────────────────────────────────────────────────────────────────────

#[test]
fn a_text_check_of_the_label_at_the_slots_path_passes() {
    let outcome = run(
        "text",
        &[
            "open: p",
            &format!("expect: {{at: {MAP}, text: \"RouteMap: a map widget\"}}"),
            &format!(
                "expect: {{at: {FORM}/children/preview, text: \"RoutePreview: a vendor preview\"}}"
            ),
            &format!(
                "expect: {{at: {FORM}/fields/route, text: \"RoutePicker: a vendor route picker\"}}"
            ),
            "act: pages/p/header/actions/pick",
            "expect: {at: pages/p/overlays/picker, text: \"TruckPicker: a fleet picker\"}",
            "open: grid",
            "expect: {at: pages/grid, text: \"VendorGrid: a vendor grid\"}",
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

#[test]
fn a_not_text_check_of_the_label_at_the_slots_path_fails() {
    let outcome = run(
        "not_text",
        &[
            "open: p",
            &format!("expect: {{at: {MAP}, not_text: \"RouteMap: a map widget\"}}"),
        ],
    );
    assert_eq!(outcome.status, Status::Failed, "{outcome:?}");
    assert_eq!(outcome.step, Some(2), "{outcome:?}");
    let message = outcome.message.unwrap_or_default();
    assert!(
        message.contains("expected no \"RouteMap: a map widget\"") && message.contains(MAP),
        "{message}"
    );
}

// ── steps inside the region: one per kind ──────────────────────────────────────────────────

#[test]
fn act_inside_a_slot_is_refused() {
    let outcome = run("act", &["open: p", &format!("act: {MAP}")]);
    assert_refused_at_step_two(&outcome, &inside(MAP, "RouteMap"));
}

#[test]
fn select_below_a_slot_is_refused() {
    let below = format!("{MAP}/rows/r-1");
    let outcome = run("select", &["open: p", &format!("select: {below}")]);
    assert_refused_at_step_two(&outcome, &inside(&below, "RouteMap"));
}

#[test]
fn mark_inside_a_slot_is_refused() {
    let below = format!("{MAP}/rows/r-1");
    let outcome = run("mark", &["open: p", &format!("mark: {below}")]);
    assert_refused_at_step_two(&outcome, &inside(&below, "RouteMap"));
}

#[test]
fn move_inside_a_slot_is_refused() {
    let below = format!("{MAP}/rows/r-1");
    let outcome = run(
        "move",
        &["open: p", &format!("move: {{at: {below}, to: up}}")],
    );
    assert_refused_at_step_two(&outcome, &inside(&below, "RouteMap"));
}

#[test]
fn upload_at_a_slot_is_refused() {
    let at = format!("{FORM}/children/preview");
    let outcome = run(
        "upload",
        &[
            "open: p",
            &format!("upload: {{at: {at}, file: route.json}}"),
        ],
    );
    assert_refused_at_step_two(&outcome, &inside(&at, "RoutePreview"));
}

#[test]
fn type_at_a_field_slot_is_refused() {
    let at = format!("{FORM}/fields/route");
    let outcome = run(
        "type",
        &["open: p", &format!("type: {{at: {at}, text: A12}}")],
    );
    assert_refused_at_step_two(&outcome, &inside(&at, "RoutePicker"));
}

#[test]
fn choose_at_a_field_slot_is_refused() {
    let at = format!("{FORM}/fields/route");
    let outcome = run(
        "choose",
        &["open: p", &format!("choose: {{at: {at}, option: north}}")],
    );
    assert_refused_at_step_two(&outcome, &inside(&at, "RoutePicker"));
}

/// A page that is a slot is refused at its own path; a header action of the page that holds a
/// slot, and a field beside a field slot, are the document's and are driven as before.
#[test]
fn only_the_slot_and_what_is_below_it_is_refused() {
    let outcome = run("page", &["open: grid", "act: pages/grid"]);
    assert_refused_at_step_two(&outcome, &inside("pages/grid", "VendorGrid"));
    let outcome = run(
        "beside",
        &[
            "open: p",
            &format!("type: {{at: {FORM}/fields/title, text: Ada}}"),
            "act: pages/p/header/actions/pick",
        ],
    );
    assert_eq!(outcome.status, Status::Passed, "{outcome:?}");
}

// ── the Playwright spec ─────────────────────────────────────────────────────────────────────

#[test]
fn the_playwright_spec_marks_a_step_inside_a_slot_fixme_with_the_same_reason() {
    let spec = spec("playwright", &["open: p", &format!("act: {MAP}")]);
    let reason = inside(MAP, "RouteMap");
    let wanted = format!(
        "test.fixme(true, {});",
        serde_json::to_string(&reason).expect("a string")
    );
    assert!(spec.contains(&wanted), "{wanted}\n\n{spec}");
    assert!(
        !spec.contains(&format!("data-ui-path=\\\"{MAP}\\\"\"]').click()")),
        "{spec}"
    );
}

/// The generated app draws the host's own component at the slot, not the terminal's label, so a
/// text check there is `test.fixme` in the spec while the terminal reads its label.
#[test]
fn the_playwright_spec_marks_a_text_check_at_a_slot_fixme() {
    let spec = spec(
        "playwright_text",
        &[
            "open: p",
            &format!("expect: {{at: {MAP}, text: \"RouteMap: a map widget\"}}"),
        ],
    );
    let reason = format!(
        "`{MAP}` is host slot `RouteMap`: the generated app draws the host's component there, \
         not the terminal's label, so its text is the host's"
    );
    let wanted = format!(
        "test.fixme(true, {});",
        serde_json::to_string(&reason).expect("a string")
    );
    assert!(spec.contains(&wanted), "{wanted}\n\n{spec}");
}
