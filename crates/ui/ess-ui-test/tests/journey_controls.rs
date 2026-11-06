//! A journey step reaches every control a page declares (beyond10x/ess#722 items 1 to 11, and
//! three more): a form on a page section, a form that saves on change, a form toggle, a multiple
//! choice and a long label, row selection for a bulk action, a filter window, a row's expand, a
//! column-header sort, a reorder, an upload, URL page state, an export's params, a filter-bar
//! choice and a choice that is a section of its own. Each test drives the control and checks what
//! it sends.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {shelf.Items: views.yaml, shelf.ItemById: views.yaml, shelf.Kinds: views.yaml}}
types:
  Sort: {record: {by: string, dir: {enum: [asc, desc]}}}
  TimeWindow: {record: {from: timestamp, to: timestamp, preset: {enum: [this_month, last_30d, this_quarter]}}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: items
  sections: [{name: all, pages: [items, settings, prefs, report]}]
pages:
  settings:
    kind: settings_page
    shell: app
    title: Settings
    sections:
      - name: settings
        component: form
        does: shelf.SaveProfile
        save: on_submit
        fields:
          - name
          - {field: notify, as: toggle, label: Notify}
          - {field: digest, as: toggle, label: Digest}
          - {field: shelf, as: choice, label: Preferred storage location, choice: {component: choice, options: [{value: north, label: North wing upper shelving}, {value: south, label: South wing lower shelving}]}}
          - {field: kinds, as: choice, label: Item categories, choice: {component: choice, multiple: true, options: [{value: hardware, label: Hardware and fittings}, {value: garden, label: Garden tools and seeds}, {value: paint, label: Paint and finishes}]}}
        actions:
          - {name: invite, does: shelf.Invite, label: Invite}
  prefs:
    kind: settings_page
    shell: app
    title: Preferences
    sections:
      - name: settings
        component: form
        does: shelf.SetPrefs
        save: on_change
        fields: [nickname, {field: dark, as: toggle, label: Dark}]
  items:
    kind: list_page
    shell: app
    title: Items
    state:
      kind: {type: string, class: page_state, store: url, pinned: true}
      sort: {type: Sort, class: page_state}
      selected: {type: {list: string}, class: selection}
    header:
      actions:
        - {name: import, upload: {accept: [text/csv], does: shelf.Import}, label: Import}
        - {name: export, export: {reads: shelf.Items, as: csv, params: {kind: state.kind}}, label: Export}
        - {name: apply, does: shelf.Apply, bind: {kind: state.kind, sort: state.sort}, label: Apply}
    sections:
      - name: list
        component: collection
        reads: {view: shelf.Items, params: {kind: state.kind}}
        selection: multiple
        columns: [{field: name, sortable: true}, {field: size, sortable: true}]
        sort: {by: name, allowed: [name, size], mode: server}
        reorder: {does: shelf.Move}
        expand: {component: record, reads: {view: shelf.ItemById, params: {id: row.id}}, fields: [note]}
        bulk_actions:
          - {name: archive, does: shelf.Archive, bind: {ids: state.selected}, label: Archive, confirm: {title: Archive items}}
  report:
    kind: report_page
    shell: app
    title: Report
    state:
      window: {type: TimeWindow, class: page_state, default: {preset: this_month}}
      kind: {type: string, class: page_state}
      bay: {type: string, class: page_state}
      level: {type: string, class: page_state}
    header:
      actions:
        - {name: level_report, does: shelf.Level, bind: {level: state.level}, label: Level report}
    sections:
      - name: filters
        component: filter_bar
        binds: [state.window, state.kind, state.bay]
        window: {binds: state.window}
        choices:
          - {name: kind, component: choice, reads: {view: shelf.Kinds, filter: row.active == true}, value: code, label: title}
          - {name: bay, component: choice, options: [east, west]}
        actions:
          - {name: apply, does: shelf.Report, bind: {window: state.window, kind: state.kind, bay: state.bay}, label: Run}
          - {name: hold, does: shelf.Hold, bind: {bay: state.bay}, label: Hold}
      - name: level
        component: choice
        binds: state.level
        options: [low, high]
      - name: list
        component: collection
        reads: {view: shelf.Items}
        columns: [name]
";

const VIEWS: &str = "views:
  shelf.Items:
    rows:
      - {id: it-1, name: Anvil, size: 3, kind: tools, note: Anvil note}
      - {id: it-2, name: Bolt, size: 1, kind: parts, note: Bolt note}
      - {id: it-3, name: Chisel, size: 2, kind: tools, note: Chisel note}
  shelf.ItemById:
    rows:
      - {id: it-1, note: Anvil note}
      - {id: it-2, note: Bolt note}
      - {id: it-3, note: Chisel note}
  shelf.Kinds:
    rows:
      - {code: k1, title: Spanner, active: false}
      - {code: k2, title: Hammer, active: true}
      - {code: k3, title: Saw, active: true}
";

const CSV: &str = "id,name\nit-9,Wrench\n";

const ROWS: &str = "pages/items/sections/list/rows";

/// Each test writes its own copy of the document, so no test reads another's half-written file.
fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-journey-controls")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is made");
    std::fs::write(dir.join("views.yaml"), VIEWS).expect("the fixture is written");
    std::fs::write(dir.join("stock.csv"), CSV).expect("the upload file is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

/// Runs one test of `steps` (one YAML step per entry) from a test file next to the document.
fn run(name: &str, steps: &[&str]) -> Outcome {
    let document = document(name);
    let mut text =
        format!("format: ess-ui-test/1\ndocument: ui.yaml\ntests:\n- name: {name}\n  steps:\n");
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let source = document.with_file_name("journey.yaml");
    let file =
        ess_ui_test::parse_str(&text, &source).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let report = ess_ui_test::execute(&document, &[file]).unwrap_or_else(|error| panic!("{error}"));
    report.tests.into_iter().next().expect("one outcome")
}

fn passed(outcome: &Outcome) {
    assert_eq!(outcome.status, Status::Passed, "{outcome:#?}");
}

#[test]
fn a_section_form_submits() {
    passed(&run(
        "section_form",
        &[
            "open: settings",
            "type: {at: pages/settings/sections/settings/fields/name, text: Ada}",
            "act: pages/settings/sections/settings",
            "expect_command: {command: shelf.SaveProfile, input: {name: Ada}}",
        ],
    ));
}

#[test]
fn an_on_change_section_form_sends_each_change() {
    passed(&run(
        "on_change",
        &[
            "open: prefs",
            "type: {at: pages/prefs/sections/settings/fields/nickname, text: Ada}",
            "expect_command: {command: shelf.SetPrefs, input: {nickname: Ada}}",
            "choose: {at: pages/prefs/sections/settings/fields/dark, option: 'true'}",
            "expect_command: {command: shelf.SetPrefs, input: {nickname: Ada, dark: true}}",
        ],
    ));
}

#[test]
fn a_step_sets_a_form_toggle() {
    // `digest` is never touched: it is sent as the unchecked box it shows, not left out.
    passed(&run(
        "toggle",
        &[
            "open: settings",
            "choose: {at: pages/settings/sections/settings/fields/notify, option: 'true'}",
            "act: pages/settings/sections/settings",
            "expect_command: {command: shelf.SaveProfile, input: {notify: true, digest: false}}",
        ],
    ));
}

#[test]
fn choose_takes_a_multiple_choice_and_long_labels() {
    passed(&run(
        "multiple",
        &[
            "open: settings",
            "choose: {at: pages/settings/sections/settings/fields/shelf, option: South wing lower shelving}",
            "choose: {at: pages/settings/sections/settings/fields/kinds, option: hardware}",
            "choose: {at: pages/settings/sections/settings/fields/kinds, option: Garden tools and seeds}",
            "act: pages/settings/sections/settings",
            "expect_command: {command: shelf.SaveProfile, input: {shelf: south, kinds: [hardware, garden]}}",
        ],
    ));
}

#[test]
fn a_step_selects_rows_for_a_bulk_action() {
    let mark = format!("mark: {ROWS}/it-1");
    let mark_third = format!("mark: {ROWS}/it-3");
    passed(&run(
        "bulk",
        &[
            "open: items",
            &mark,
            &mark_third,
            "act: pages/items/sections/list/bulk_actions/archive",
            "expect: {at: pages/items/sections/list/bulk_actions/archive/confirm/overlay, text: Archive items}",
            "act: pages/items/sections/list/bulk_actions/archive/confirm/overlay",
            "expect_command: {command: shelf.Archive, input: {ids: [it-1, it-3]}}",
        ],
    ));
}

#[test]
fn a_step_sets_a_filter_window() {
    passed(&run(
        "window",
        &[
            "open: report",
            "choose: {at: pages/report/sections/filters, option: last_30d}",
            "act: pages/report/sections/filters/actions/apply",
            "expect_command: {command: shelf.Report, input: {window: {preset: last_30d}}}",
        ],
    ));
}

#[test]
fn a_step_expands_a_row() {
    let expand = format!("select: {ROWS}/it-2/expand");
    let shows = format!("expect: {{at: {ROWS}/it-2/expand, text: Bolt note}}");
    passed(&run(
        "expand",
        &[
            "open: items",
            "expect: {at: pages/items/sections/list, not_text: Bolt note}",
            &expand,
            &shows,
            "expect: {at: pages/items/sections/list, not_text: Anvil note}",
        ],
    ));
}

#[test]
fn a_step_sorts_by_a_column_header() {
    passed(&run(
        "sort",
        &[
            "open: items",
            "expect: {at: pages/items/sections/list, rows: [it-1, it-2, it-3]}",
            "act: pages/items/sections/list/columns/size",
            "expect: {at: pages/items/sections/list, rows: [it-2, it-3, it-1]}",
            "act: pages/items/header/actions/apply",
            "expect_command: {command: shelf.Apply, input: {sort: {by: size, dir: asc}}}",
            "act: pages/items/sections/list/columns/size",
            "expect: {at: pages/items/sections/list, rows: [it-1, it-3, it-2]}",
            "act: pages/items/header/actions/apply",
            "expect_command: {command: shelf.Apply, input: {sort: {by: size, dir: desc}}}",
        ],
    ));
}

#[test]
fn a_step_reorders_a_row() {
    let up = format!("move: {{at: {ROWS}/it-2, to: up}}");
    passed(&run(
        "reorder",
        &[
            "open: items",
            &up,
            "expect_command: {command: shelf.Move, input: {id: it-2, delta: -1}}",
        ],
    ));
}

#[test]
fn a_step_uploads_a_file() {
    passed(&run(
        "upload",
        &[
            "open: items",
            "upload: {at: pages/items/header/actions/import, file: stock.csv}",
            "expect_command: {command: shelf.Import, input: {name: stock.csv, content: \"id,name\\nit-9,Wrench\\n\"}}",
        ],
    ));
}

#[test]
fn open_takes_url_page_state() {
    passed(&run(
        "url_state",
        &[
            "open: {page: items, state: {kind: tools}}",
            "expect: {at: pages/items/sections/list, rows: [it-1, it-3]}",
            "act: pages/items/header/actions/apply",
            "expect_command: {command: shelf.Apply, input: {kind: tools}}",
        ],
    ));
    // Only a state the page keeps in its URL can be opened with.
    let outcome = run(
        "not_url_state",
        &["open: {page: items, state: {selected: [it-1]}}"],
    );
    assert_eq!(outcome.step, Some(1), "{outcome:#?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("not kept in the URL"), "{message}");
}

#[test]
fn export_params_are_assertable() {
    passed(&run(
        "export",
        &[
            "open: {page: items, state: {kind: tools}}",
            "act: pages/items/header/actions/export",
            "expect_export: {view: shelf.Items, as: csv, params: {kind: tools}}",
        ],
    ));
    let outcome = run(
        "export_mismatch",
        &[
            "open: {page: items, state: {kind: tools}}",
            "act: pages/items/header/actions/export",
            "expect_export: {view: shelf.Items, params: {kind: parts}}",
        ],
    );
    assert_eq!(outcome.step, Some(3), "{outcome:#?}");
}

#[test]
fn a_step_sets_a_filter_bar_choice() {
    // The choice lists only the active kinds (`filter`): Saw is its second option, not its third.
    passed(&run(
        "bar_choice",
        &[
            "open: report",
            "choose: {at: pages/report/sections/filters/choices/kind, option: Saw}",
            "choose: {at: pages/report/sections/filters/choices/bay, option: west}",
            "act: pages/report/sections/filters/actions/apply",
            "expect_command: {command: shelf.Report, input: {kind: k3, bay: west}}",
        ],
    ));
}

#[test]
fn a_step_chooses_in_a_section_level_choice() {
    passed(&run(
        "section_choice",
        &[
            "open: report",
            "choose: {at: pages/report/sections/level, option: high}",
            "act: pages/report/header/actions/level_report",
            "expect_command: {command: shelf.Level, input: {level: high}}",
        ],
    ));
}

/// A letter a view takes before its actions keys none of them: `i` edits a form's field and `h`
/// moves a filter bar to its previous item, so a form action `invite` and a bar action `hold` are
/// offered under other letters, and `act` reaches both.
#[test]
fn a_view_key_never_keys_an_action_of_the_view() {
    passed(&run(
        "view_keys",
        &[
            "open: settings",
            "act: pages/settings/sections/settings/actions/invite",
            "expect_command: {command: shelf.Invite}",
            "open: report",
            "choose: {at: pages/report/sections/filters/choices/bay, option: east}",
            "act: pages/report/sections/filters/actions/hold",
            "expect_command: {command: shelf.Hold, input: {bay: east}}",
        ],
    ));
}

/// The Playwright spec drives the same controls in the generated app; an export's read is not
/// captured there, so `expect_export` is `test.fixme`, never a silent pass.
#[test]
fn the_playwright_spec_drives_each_control() {
    let document = document("playwright");
    let steps: [&str; 12] = [
        "open: {page: items, state: {kind: tools}}",
        &format!("mark: {ROWS}/it-1"),
        &format!("move: {{at: {ROWS}/it-2, to: down}}"),
        &format!("select: {ROWS}/it-2/expand"),
        "act: pages/items/sections/list/columns/size",
        "upload: {at: pages/items/header/actions/import, file: stock.csv}",
        "open: settings",
        "choose: {at: pages/settings/sections/settings/fields/notify, option: 'true'}",
        "act: pages/settings/sections/settings",
        "open: report",
        "choose: {at: pages/report/sections/filters, option: last_30d}",
        "expect_export: {view: shelf.Items}",
    ];
    let mut text =
        "format: ess-ui-test/1\ndocument: ui.yaml\ntests:\n- name: spec\n  steps:\n".to_owned();
    for step in steps {
        text.push_str("  - ");
        text.push_str(step);
        text.push('\n');
    }
    let file = ess_ui_test::parse_str(&text, &document.with_file_name("journey.yaml"))
        .unwrap_or_else(|error| panic!("{error}"));
    let loaded = ess_ui::load_path(&document).expect("the document loads");
    let spec = ess_ui_test::playwright(&[file], &loaded);
    let file_path = document.with_file_name("stock.csv").display().to_string();
    for wanted in [
        "new URLSearchParams({\"kind\":\"tools\"})".to_owned(),
        "rows/it-1\"]').locator('input[aria-label=\"Select row\"]').first().click();".to_owned(),
        "rows/it-2\"]').locator(\".ui-move button\").last().click();".to_owned(),
        "rows/it-2/expand\"]')).toBeVisible();".to_owned(),
        "columns/size\"] button.ui-sort').click();".to_owned(),
        format!(".setInputFiles({});", serde_json::to_string(&file_path).expect("a string")),
        "fields/notify\"]').locator('input[type=\"checkbox\"]').setChecked(true);".to_owned(),
        "sections/settings\"] button[type=\"submit\"]').click();".to_owned(),
        "select[aria-label=\"Time window\"]').selectOption(\"last_30d\");".to_owned(),
        "test.fixme(true, \"expect_export shelf.Items: the spec does not capture the read an export makes\");".to_owned(),
    ] {
        assert!(spec.contains(&wanted), "{wanted}\n{spec}");
    }
}
