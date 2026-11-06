//! Adversary pass 1 on the journey-controls unit of the 0.88.0 wave: the new steps driven against
//! what the story's acceptance and the unit's own reference say they do, and against what the
//! generated app sends for the same control.

use std::path::{Path, PathBuf};

use ess_ui_test::{Outcome, Status};

const DOCUMENT: &str = r"format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
fixtures: {views: {shelf.Items: views.yaml, shelf.Kinds: views.yaml}}
types:
  TimeWindow: {record: {from: timestamp, to: timestamp, preset: {enum: [this_month, last_30d, this_quarter]}}}
  Prefs: {record: {digest: boolean}}
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: items
  sections: [{name: all, pages: [items, settings, defaults, report]}]
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
          - {field: kinds, as: choice, label: Kinds, choice: {component: choice, multiple: true, options: [{value: hardware, label: Hardware}, {value: garden, label: Garden}]}}
        actions:
          - {name: load, does: shelf.Load, label: Load}
  defaults:
    kind: settings_page
    shell: app
    title: Defaults
    sections:
      - name: settings
        component: form
        does: shelf.SavePrefs
        save: on_submit
        fields:
          - name
          - {field: digest, as: toggle, label: Digest}
        draft: {type: Prefs, class: draft, default: {digest: true}}
  items:
    kind: list_page
    shell: app
    title: Items
    state:
      kind: {type: string, class: page_state, store: url}
    header:
      actions:
        - {name: import, upload: {accept: [text/csv], does: shelf.Import}, label: Import}
        - {name: export, export: {reads: shelf.Items, as: csv, params: {kind: state.kind}}, label: Export}
        - {name: export_same, export: {reads: shelf.Items, as: json, params: same_as(list)}, label: Export same}
    sections:
      - name: list
        component: collection
        reads: {view: shelf.Items, params: {kind: state.kind}}
        columns: [name]
        reorder: {does: shelf.Move}
  report:
    kind: report_page
    shell: app
    title: Report
    state:
      window: {type: TimeWindow, class: page_state, default: {preset: this_month}}
      q: {type: string, class: page_state}
      bay: {type: string, class: page_state}
    sections:
      - name: filters
        component: filter_bar
        binds: [state.window, state.q, state.bay]
        search: {binds: state.q}
        window: {binds: state.window}
        choices:
          - {name: bay, component: choice, options: [east, west]}
        actions:
          - {name: apply, does: shelf.Report, bind: {window: state.window, bay: state.bay}, label: Run}
          - {name: load, does: shelf.Load, bind: {bay: state.bay}, label: Load}
      - name: list
        component: collection
        reads: {view: shelf.Items}
        columns: [name]
";

const VIEWS: &str = "views:
  shelf.Items:
    rows:
      - {id: it-1, name: Anvil, kind: tools}
      - {id: it-2, name: Bolt, kind: parts}
      - {id: it-3, name: Chisel, kind: tools}
  shelf.Kinds:
    rows:
      - {code: k1, title: Spanner}
";

const ROWS: &str = "pages/items/sections/list/rows";

fn document(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-test-adversary-w088-u3-pass1")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("the old scratch dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the scratch dir is made");
    std::fs::write(dir.join("views.yaml"), VIEWS).expect("the fixture is written");
    // A CSV saved as "CSV UTF-8" by a spreadsheet: a byte order mark, then UTF-8 text.
    std::fs::write(dir.join("bom.csv"), b"\xEF\xBB\xBFid,name\nit-9,Wrench\n")
        .expect("the BOM file is written");
    // A CSV saved in a single-byte encoding: `é` is the byte 0xE9, not valid UTF-8.
    std::fs::write(dir.join("latin.csv"), b"id,name\nit-9,Caf\xE9\n")
        .expect("the latin file is written");
    std::fs::write(dir.join("ui.yaml"), DOCUMENT).expect("the document is written");
    dir.join("ui.yaml")
}

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

// ── filter bar and form actions whose key is a move key ─────────────────────────────────────

/// The unit made a filter bar run its own actions by the key the hint line offers. An action
/// named `load` is keyed `l` (`h` and `l` are not reserved), and `l` in a filter bar moves to the
/// next item first, so `act` presses the offered key and nothing is sent.
#[test]
fn a_filter_bar_action_keyed_l_sends_its_command() {
    passed(&run(
        "bar_action_l",
        &[
            "open: report",
            "choose: {at: pages/report/sections/filters/choices/bay, option: west}",
            "act: pages/report/sections/filters/actions/load",
            "expect_command: {command: shelf.Load, input: {bay: west}}",
        ],
    ));
}

/// A multiple choice field now takes `l` and `h` to move its option cursor. A form action keyed
/// `l` is then unreachable while the cursor is on that field, which is where `choose` leaves it.
#[test]
fn a_form_action_keyed_l_runs_after_a_multiple_choice() {
    passed(&run(
        "form_action_l",
        &[
            "open: settings",
            "choose: {at: pages/settings/sections/settings/fields/kinds, option: garden}",
            "act: pages/settings/sections/settings/actions/load",
            "expect_command: {command: shelf.Load}",
        ],
    ));
}

// ── upload: the file's text as the browser reads it ─────────────────────────────────────────

/// The reference: the upload sends "the file's `name` and its text as `content`". The generated
/// app reads it with `File.text()`, a UTF-8 decode that drops a leading byte order mark; the
/// terminal reads it with `read_to_string`, which keeps it.
#[test]
fn an_upload_sends_the_text_without_a_byte_order_mark() {
    passed(&run(
        "upload_bom",
        &[
            "open: items",
            "upload: {at: pages/items/header/actions/import, file: bom.csv}",
            "expect_command: {command: shelf.Import, input: {name: bom.csv, content: \"id,name\\nit-9,Wrench\\n\"}}",
        ],
    ));
}

/// The generated app sends a file that is not valid UTF-8 with replacement characters; the
/// terminal notifies a read error and sends no command at all.
#[test]
fn an_upload_of_a_file_that_is_not_utf8_sends_the_command() {
    passed(&run(
        "upload_latin",
        &[
            "open: items",
            "upload: {at: pages/items/header/actions/import, file: latin.csv}",
            "expect_command: {command: shelf.Import, input: {name: latin.csv}}",
        ],
    ));
}

// ── reorder: what the command carries ──────────────────────────────────────────────────────

/// The unit's `a_step_reorders_a_row` asserts only `id`, so a terminal that sent `down` for `up`
/// passes it. This pins the direction the terminal sends, as the generated app's `delta`: the
/// specification here declares no input for the reorder command, so the terminal sends what the
/// move buttons send (the story's reorder decision).
#[test]
fn a_move_up_is_sent_as_up() {
    let up = format!("move: {{at: {ROWS}/it-2, to: up}}");
    let down = format!("move: {{at: {ROWS}/it-3, to: down}}");
    passed(&run(
        "move_direction",
        &[
            "open: items",
            &up,
            "expect_command: {command: shelf.Move, input: {id: it-2, delta: -1}}",
            &down,
            "expect_command: {command: shelf.Move, input: {id: it-3, delta: 1}}",
        ],
    ));
}

/// The generated app's move buttons send `{id, delta: -1 | 1}`
/// (`collection.tsx.tmpl`, `move(row, -1)`). A journey that pins what the reorder command
/// carries cannot hold in both renderers while the terminal sends `{id, direction}`.
#[test]
fn a_move_up_sends_what_the_generated_app_sends() {
    let up = format!("move: {{at: {ROWS}/it-2, to: up}}");
    passed(&run(
        "move_delta",
        &[
            "open: items",
            &up,
            "expect_command: {command: shelf.Move, input: {id: it-2, delta: -1}}",
        ],
    ));
}

// ── open with URL page state ─────────────────────────────────────────────────────────────────

/// `open: {page, state}` is "a link carrying page state in its query"; a plain `open` is the
/// page's route with no query, as the spec's `page.goto(route)` loads it. The terminal keeps the
/// earlier link's URL state, so the list stays narrowed.
#[test]
fn a_plain_open_after_a_link_with_state_has_no_url_state() {
    passed(&run(
        "url_state_dropped",
        &[
            "open: {page: items, state: {kind: tools}}",
            "expect: {at: pages/items/sections/list, rows: [it-1, it-3]}",
            "open: items",
            "expect: {at: pages/items/sections/list, rows: [it-1, it-2, it-3]}",
        ],
    ));
}

// ── guards the unit's suite does not exercise ───────────────────────────────────────────────

/// An untouched toggle is sent as its declared default (the draft's), not `false`.
#[test]
fn an_untouched_toggle_is_sent_as_the_drafts_default() {
    passed(&run(
        "toggle_default",
        &[
            "open: defaults",
            "type: {at: pages/defaults/sections/settings/fields/name, text: Ada}",
            "act: pages/defaults/sections/settings",
            "expect_command: {command: shelf.SavePrefs, input: {name: Ada, digest: true}}",
        ],
    ));
}

/// `same_as(<section>)` export params are the params of that section's read.
#[test]
fn an_export_with_same_as_params_reads_with_the_sections_params() {
    passed(&run(
        "export_same_as",
        &[
            "open: {page: items, state: {kind: tools}}",
            "act: pages/items/header/actions/export_same",
            "expect_export: {view: shelf.Items, as: json, params: {kind: tools}}",
        ],
    ));
}

/// `as` narrows `expect_export`: a csv export does not satisfy `as: json`.
#[test]
fn expect_export_fails_on_another_format() {
    let outcome = run(
        "export_format",
        &[
            "open: {page: items, state: {kind: tools}}",
            "act: pages/items/header/actions/export",
            "expect_export: {view: shelf.Items, as: json}",
        ],
    );
    assert_eq!(outcome.step, Some(3), "{outcome:#?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("never exported as json"), "{message}");
}

/// A window is chosen in a filter bar that also has a search before it.
#[test]
fn a_window_is_chosen_in_a_bar_with_a_search() {
    passed(&run(
        "window_after_search",
        &[
            "open: report",
            "choose: {at: pages/report/sections/filters, option: this_quarter}",
            "act: pages/report/sections/filters/actions/apply",
            "expect_command: {command: shelf.Report, input: {window: {preset: this_quarter}}}",
        ],
    ));
}

/// Opening with state the page does not declare fails the step and names the state.
#[test]
fn open_with_undeclared_state_is_refused() {
    let outcome = run(
        "url_state_undeclared",
        &["open: {page: items, state: {tier: gold}}"],
    );
    assert_eq!(outcome.step, Some(1), "{outcome:#?}");
    let message = outcome.message.unwrap_or_default();
    assert!(message.contains("declares no state tier"), "{message}");
}
