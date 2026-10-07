//! Client row filtering over a shared raw adapter result.
use ess_ui_tui::{App, DataAdapter, Lifecycle, Options, ReadRequest, ReadResult};
use serde_yaml::Value;
use std::{cell::Cell, collections::BTreeMap, rc::Rc, time::Duration};

struct Rows(Rc<Cell<usize>>);
impl DataAdapter for Rows {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        assert!(
            request.params.is_empty(),
            "the predicate never goes to the adapter"
        );
        self.0.set(self.0.get() + 1);
        Ok(ReadResult {
            rows: serde_yaml::from_str(
                "[{id: a, group: one, label: Alpha}, {id: b, group: two, label: Beta}]",
            )
            .unwrap(),
            total: Some(99),
        })
    }
    fn run(&mut self, _: &str, _: &BTreeMap<String, Value>) -> ess_ui::binding::Answer {
        panic!("unexpected command")
    }
    fn load_state(&self, _: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _: &str, _: Value) {}
}

fn document() -> String {
    r"format: ess-ui/1
app: filters
model: t
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}, nav: {kind: navigation}}}}
navigation:
  home: p
  sections:
    - name: entries
      pages: {from_view: t.All, page: p, param: id, label: row.label, filter: row.group == one}
pages:
  p:
    kind: detail_page
    title: Filters
    params: {id: string}
    header: {total: first}
    sections:
      - {name: first, component: collection, reads: {view: t.All, filter: row.group == one}, columns: [label]}
      - {name: second, component: collection, reads: {view: t.All, filter: row.group == two}, columns: [label]}
      - {name: empty, component: collection, reads: {view: t.All, filter: row.group == missing}, columns: [label], states: {empty: {message: Nobody}}}
".to_owned()
}

#[test]
fn read_filters_keep_distinct_rows_share_one_request_and_decide_empty() {
    let calls = Rc::new(Cell::new(0));
    let mut app = App::with_adapter(
        ess_ui::load_str(&document()).unwrap(),
        Box::new(Rows(calls.clone())),
        vec![],
        Options::new(std::env::temp_dir().join("ess-read-filter-distinct")),
    )
    .unwrap();
    app.advance(Duration::from_secs(1));
    assert_eq!(
        app.rows("first")
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["a"]
    );
    assert_eq!(
        app.rows("second")
            .iter()
            .map(|row| row["id"].as_str().unwrap())
            .collect::<Vec<_>>(),
        ["b"]
    );
    assert_eq!(app.section_state("empty"), Lifecycle::Empty);
    assert_eq!(calls.get(), 1, "different visibility must share raw reads");
    let screen = app.render_text(100, 40);
    assert!(screen.contains("Nobody"), "{screen}");
    assert!(!screen.contains("99"), "totals use kept rows: {screen}");
}

#[test]
fn unchecked_invalid_filter_fails_closed() {
    for filter in [
        "row.group",
        "matches(params)",
        "row.group ==",
        "actor.id == row.id",
    ] {
        let source = document().replace("row.group == one", filter);
        let mut app = App::with_adapter(
            ess_ui::load_str(&source).unwrap(),
            Box::new(Rows(Rc::new(Cell::new(0)))),
            vec![],
            Options::new(std::env::temp_dir().join("ess-read-filter-invalid")),
        )
        .unwrap();
        app.advance(Duration::from_secs(1));
        assert!(
            app.rows("first").is_empty(),
            "invalid filter leaked rows: {filter}"
        );
    }
}

fn live_app(name: &str, filter: &str, events: &str) -> App {
    let source = document()
        .replace("pages:\n  p:", "channels: {updates: {carries: {events: [t.Changed]}, direction: server_to_client, delivery: every_event, resume: refetch}}\npages:\n  p:")
        .replace("filter: row.group == one}, columns: [label]}", &format!("filter: '{filter}'}}, columns: [label], live: {{channel: updates, effect: insert_or_patch, only_if: row.label != blocked, when_paged_away: count_new}}}}"));
    let script = ess_ui_tui::live::Script::parse(
        "updates",
        &serde_yaml::from_str(&format!("{{events: [{events}]}}")).unwrap(),
    )
    .unwrap();
    let mut options = Options::new(std::env::temp_dir().join(name));
    options.page_size = 1;
    App::with_adapter(
        ess_ui::load_str(&source).unwrap(),
        Box::new(Rows(Rc::new(Cell::new(0)))),
        vec![script],
        options,
    )
    .unwrap()
}

#[test]
fn read_filter_hides_patched_rows_and_a_later_partial_patch_restores_them() {
    let mut app = live_app("ess-filter-patch", "row.group == one", "{at: 2s, event: t.Changed, payload: {id: a, group: two}}, {at: 3s, event: t.Changed, payload: {id: a, group: one}}, {at: 4s, event: t.Changed, payload: {id: c, group: two}}, {at: 5s, event: t.Changed, payload: {id: d, group: one, label: blocked}}");
    app.advance(Duration::from_secs(1));
    assert_eq!(app.rows("first").len(), 1);
    app.advance(Duration::from_secs(1));
    assert_eq!(app.rows("first").len(), 0);
    app.advance(Duration::from_secs(1));
    assert_eq!(
        app.rows("first")[0]["label"].as_str(),
        Some("Alpha"),
        "raw row fields survive hiding"
    );
    app.advance(Duration::from_secs(2));
    assert_eq!(
        app.rows("first").len(),
        1,
        "both the read filter and only_if exclude new rows"
    );
}

#[test]
fn read_filter_paged_away_counts_only_post_event_matching_rows() {
    let mut app = live_app("ess-filter-paged", "row.group != bad", "{at: 2s, event: t.Changed, payload: {id: c, group: bad}}, {at: 3s, event: t.Changed, payload: {id: d, group: one}}, {at: 4s, event: t.Changed, payload: {id: d, group: bad}}, {at: 5s, event: t.Changed, payload: {id: d, group: one}}");
    app.advance(Duration::from_secs(1));
    app.focus_section("first");
    app.keys("n");
    app.advance(Duration::from_secs(1));
    assert!(!app.render_text(100, 40).contains("1 new"));
    app.advance(Duration::from_secs(1));
    assert!(app.render_text(100, 40).contains("1 new"));
    app.advance(Duration::from_secs(1));
    assert!(!app.render_text(100, 40).contains("1 new"));
    app.advance(Duration::from_secs(1));
    assert!(app.render_text(100, 40).contains("1 new"));
    app.keys("p");
    assert!(app.rows("first").iter().any(|row| row["id"] == "d"));
    assert!(!app.rows("first").iter().any(|row| row["id"] == "c"));
}

#[test]
fn read_filter_state_change_reuses_rows_and_prunes_bulk_selection() {
    let source = document()
        .replace("header: {total: first}", "state: {pick: {type: string, class: component_state, default: one}, selected: {type: {list: string}, class: selection}}\n    header: {total: first, actions: [{name: flip, label: Flip, sets: {state.pick: two}}]}")
        .replace("filter: row.group == one}, columns: [label]}", "filter: row.group == state.pick}, columns: [label], selection: multiple, bulk_actions: [{name: apply, does: t.Bulk, bind: {ids: state.selected}}]}");
    let calls = Rc::new(Cell::new(0));
    let mut app = App::with_adapter(
        ess_ui::load_str(&source).unwrap(),
        Box::new(Rows(calls.clone())),
        vec![],
        Options::new(std::env::temp_dir().join("ess-filter-selection")),
    )
    .unwrap();
    app.advance(Duration::from_secs(1));
    app.focus_section("first");
    app.keys("<space>");
    assert!(app.render_text(100, 40).contains("1 selected"));
    app.keys(":flip<enter>");
    assert_eq!(app.rows("first")[0]["id"].as_str(), Some("b"));
    assert_eq!(calls.get(), 1);
    app.focus_section("first");
    app.keys("A"); // Rows::run panics if a hidden selection reaches the bulk command.
}

struct KeyRows(Rc<Cell<usize>>);
impl DataAdapter for KeyRows {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        let mut result = Rows(self.0.clone()).read(request)?;
        for row in &mut result.rows {
            let id = row
                .as_mapping_mut()
                .unwrap()
                .remove(Value::String("id".into()))
                .unwrap();
            row["item_id"] = id;
        }
        Ok(result)
    }
    fn run(&mut self, _: &str, _: &BTreeMap<String, Value>) -> ess_ui::binding::Answer {
        panic!("hidden key-only selection reached a bulk command")
    }
    fn load_state(&self, _: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _: &str, _: Value) {}
}

#[test]
fn read_filter_prunes_key_only_rows_without_collapsing_them_to_null_ids() {
    let source = document()
        .replace("header: {total: first}", "state: {pick: {type: string, class: component_state, default: one}, selected: {type: {list: string}, class: selection}}\n    header: {total: first, actions: [{name: flip, label: Flip, sets: {state.pick: two}}]}")
        .replace("filter: row.group == one}, columns: [label]}", "filter: row.group == state.pick, key: item_id}, columns: [label], selection: multiple, bulk_actions: [{name: apply, does: t.Bulk, bind: {ids: state.selected}}]}");
    let mut app = App::with_adapter(
        ess_ui::load_str(&source).unwrap(),
        Box::new(KeyRows(Rc::new(Cell::new(0)))),
        vec![],
        Options::new(std::env::temp_dir().join("ess-filter-selection-key")),
    )
    .unwrap();
    app.advance(Duration::from_secs(1));
    app.focus_section("first");
    app.keys("<space>");
    assert!(app.render_text(100, 40).contains("1 selected"));
    app.keys(":flip<enter>");
    assert_eq!(app.rows("first")[0]["item_id"].as_str(), Some("b"));
    assert!(!app.render_text(100, 40).contains("1 selected"));
    app.focus_section("first");
    app.keys("A");
}

#[test]
fn read_filter_page_params_narrow_rows_without_becoming_request_params() {
    let source = document().replace(
        "filter: row.group == one}, columns",
        "filter: row.group == params.id}, columns",
    );
    let calls = Rc::new(Cell::new(0));
    let mut app = App::with_adapter(
        ess_ui::load_str(&source).unwrap(),
        Box::new(Rows(calls.clone())),
        vec![],
        Options::new(std::env::temp_dir().join("ess-filter-page-param")),
    )
    .unwrap();
    app.open_page("p", &[("id", "one")]);
    assert_eq!(app.rows("first")[0]["id"].as_str(), Some("a"));
    app.open_page("p", &[("id", "two")]);
    assert_eq!(app.rows("first")[0]["id"].as_str(), Some("b"));
    // Each navigation unmounts the page's memory cache. Rows::read also asserts that the
    // request params remain empty: the page parameter belongs only to the client predicate.
    assert_eq!(calls.get(), 3);
}

#[test]
fn dynamic_menu_keeps_matching_rows_and_refuses_invalid_predicates() {
    let source = document()
        .replace(
            "filter: row.group == one}, columns",
            "filter: row.group == missing}, columns",
        )
        .replace(
            "filter: row.group == two}, columns",
            "filter: row.group == missing}, columns",
        );
    for (filter, shown) in [
        ("row.group == one", true),
        ("row.group", false),
        ("actor.id == row.id", false),
    ] {
        let source = source.replace(
            "label: row.label, filter: row.group == one",
            &format!("label: row.label, filter: {filter}"),
        );
        let mut app = App::with_adapter(
            ess_ui::load_str(&source).unwrap(),
            Box::new(Rows(Rc::new(Cell::new(0)))),
            vec![],
            Options::new(std::env::temp_dir().join("ess-filter-menu")),
        )
        .unwrap();
        app.advance(Duration::from_secs(1));
        let screen = app.render_text(100, 40);
        assert_eq!(screen.contains("Alpha"), shown, "{screen}");
        assert!(!screen.contains("Beta"), "{screen}");
    }
}
