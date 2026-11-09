//! The terminal draws a host slot as a labelled placeholder region at the slot's path: a
//! bordered region whose text is `<host>: <reason>`. It cannot draw the host's own component, so
//! it marks where the component stands, at each of the scopes a slot takes: a page, a section, an
//! overlay, a section's child and a form field. `ess generate ui --target tui` and `ess ui run
//! --tui` hold the document to the terminal's profile first, and a slot no longer fails it.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use ess_ui::binding::Answer;
use ess_ui_tui::{App, DataAdapter, Options, ReadRequest, ReadResult, Region};
use ess_yaml::Value;

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

const FORM: &str = "pages/p/sections/form";

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-host-slots")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

/// Answers no read with rows and records every command it is sent.
struct Recording(Sent);

impl DataAdapter for Recording {
    fn read(&self, _request: &ReadRequest) -> Result<ReadResult, String> {
        Ok(ReadResult::default())
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Answer {
        self.0
            .borrow_mut()
            .push((command.to_owned(), input.clone()));
        Answer::Accepted
    }
    fn load_state(&self, _path: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _path: &str, _value: Value) {}
}

type Sent = Rc<RefCell<Vec<(String, BTreeMap<String, Value>)>>>;

fn open(test: &str) -> (App, Sent) {
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let sent: Sent = Rc::default();
    let app = App::with_adapter(
        document,
        Box::new(Recording(Rc::clone(&sent))),
        Vec::new(),
        Options::new(state_dir(test)),
    )
    .unwrap_or_else(|error| panic!("a document with host slots opens: {error}"));
    (app, sent)
}

/// The text inside `region` of a screen rendered as text.
fn text_in(screen: &str, region: &Region) -> String {
    screen
        .lines()
        .skip(usize::from(region.area.y))
        .take(usize::from(region.area.height))
        .map(|line| {
            line.chars()
                .skip(usize::from(region.area.x))
                .take(usize::from(region.area.width))
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// The text drawn at `path` on the last frame, failing when nothing was recorded there.
fn drawn_at(app: &App, screen: &str, path: &str) -> String {
    let regions = app.regions();
    let region = regions
        .iter()
        .find(|region| region.path == path)
        .unwrap_or_else(|| panic!("a region is recorded at {path}: {regions:#?}\n{screen}"));
    text_in(screen, region)
}

/// A bordered region: a box drawn with its four corners around the label.
fn assert_boxed(drawn: &str, label: &str) {
    let lines: Vec<&str> = drawn.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.contains(label))
        .unwrap_or_else(|| panic!("the region shows {label:?}:\n{drawn}"));
    let row = lines[at];
    let (left, right) = (row.find(label).unwrap_or(0), row.find(label).unwrap_or(0));
    assert!(
        row[..left].contains('│') && row[right + label.len()..].contains('│'),
        "the label is bordered left and right:\n{drawn}"
    );
    assert!(
        at > 0 && lines[at - 1].contains('┌') && lines[at - 1].contains('┐'),
        "a top border is drawn above the label:\n{drawn}"
    );
    assert!(
        lines
            .get(at + 1)
            .is_some_and(|line| line.contains('└') && line.contains('┘')),
        "a bottom border is drawn below the label:\n{drawn}"
    );
}

#[test]
fn the_terminal_profile_admits_a_document_with_host_slots() {
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(document.host_slots().len(), 5);
    ess_ui_tui::profile::check(&document, &ess_ui_tui::TUI)
        .unwrap_or_else(|refusal| panic!("a host slot is drawn, not refused: {refusal}"));
}

#[test]
fn generating_the_terminal_crate_admits_a_document_with_host_slots() {
    let binding: ess_ui::binding::Binding = serde_json::from_str(
        r#"{"system": "t", "components": {"svc": {"views": {},
            "commands": {"t.Save": {"path": "/save", "body_required": true, "errors": {}}}}},
            "names": {"t.Save": "t.Save"}}"#,
    )
    .expect("the binding reads");
    let files = ess_ui_tui::generate::render(DOCUMENT, &binding)
        .unwrap_or_else(|error| panic!("a document with host slots generates: {error}"));
    assert!(files["src/ui.yaml"].contains("host: RouteMap"));
}

#[test]
fn a_section_slot_is_a_bordered_region_labelled_with_its_host_and_reason() {
    let (app, _) = open("section");
    let screen = app.render_text(160, 80);
    let drawn = drawn_at(&app, &screen, "pages/p/sections/map");
    assert_boxed(&drawn, "RouteMap: a map widget");
}

#[test]
fn a_section_childs_slot_is_a_bordered_region_at_the_childs_path() {
    let (app, _) = open("child");
    let screen = app.render_text(160, 80);
    let drawn = drawn_at(&app, &screen, &format!("{FORM}/children/preview"));
    assert_boxed(&drawn, "RoutePreview: a vendor preview");
}

#[test]
fn a_form_fields_slot_is_labelled_on_the_fields_own_line() {
    let (app, _) = open("field");
    let screen = app.render_text(160, 80);
    let drawn = drawn_at(&app, &screen, &format!("{FORM}/fields/route"));
    assert!(
        drawn.contains("Route") && drawn.contains("│RoutePicker: a vendor route picker│"),
        "the field keeps its label and draws the slot bordered in place of its input:\n{drawn}"
    );
}

#[test]
fn an_overlay_slot_is_a_bordered_region_in_the_open_overlay() {
    let (mut app, _) = open("overlay");
    app.open_overlay("picker");
    let screen = app.render_text(160, 80);
    let drawn = drawn_at(&app, &screen, "pages/p/overlays/picker");
    assert_boxed(&drawn, "TruckPicker: a fleet picker");
}

#[test]
fn a_page_slot_is_a_bordered_region_below_the_pages_title() {
    let (mut app, _) = open("page");
    app.open_page("grid", &[]);
    let screen = app.render_text(160, 80);
    let drawn = drawn_at(&app, &screen, "pages/grid");
    assert_boxed(&drawn, "VendorGrid: a vendor grid");
    assert!(screen.contains("Grid"), "{screen}");
}

/// The host draws a field slot's input, so the terminal edits nothing there: keys typed at it do
/// not reach the draft the form sends.
#[test]
fn keys_at_a_field_slot_edit_nothing() {
    let (mut app, sent) = open("keys");
    app.focus_section("form");
    app.keys("<enter>Ada<esc>j<enter>xyz<esc><c-s>");
    let sent = sent.borrow();
    let (command, input) = sent
        .iter()
        .find(|(command, _)| command == "t.Save")
        .unwrap_or_else(|| panic!("the form is sent: {sent:?}"));
    assert_eq!(command, "t.Save");
    assert_eq!(
        input.get("title").and_then(Value::as_str),
        Some("Ada"),
        "{input:?}"
    );
    assert!(
        !input.contains_key("route"),
        "a field the host draws is not edited by the terminal: {input:?}"
    );
}
