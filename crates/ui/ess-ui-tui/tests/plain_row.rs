//! `field: .` names the row itself: a read answering plain strings draws one row per string,
//! sorts by them and keys each row by its string (story:a-column-may-show-a-plain-string-row).
use ess_ui_tui::{App, DataAdapter, Options, ReadRequest, ReadResult};
use ess_yaml::Value;
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

struct Scratch(PathBuf);

impl Scratch {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&path);
        Self(path)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

struct Plain;
impl DataAdapter for Plain {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        let rows = match request.view.as_str() {
            "t.Names" => "[beta, alpha]",
            _ => "[{id: i-1, label: An object}]",
        };
        Ok(ReadResult {
            rows: ess_yaml::from_str(rows).unwrap(),
            total: None,
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

fn app(section: &str, state: &Scratch) -> App {
    let document = format!(
        "format: ess-ui/1
app: plain
model: t
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: detail_page
    title: Plain
    sections:
      - {section}
"
    );
    let mut app = App::with_adapter(
        ess_ui::load_str(&document).unwrap(),
        Box::new(Plain),
        vec![],
        Options::new(state.0.clone()),
    )
    .unwrap();
    app.advance(Duration::from_secs(1));
    app.focus_section("list");
    app
}

/// The keys of the rows drawn for `list`, in screen order.
fn row_keys(app: &mut App) -> Vec<String> {
    app.render_text(100, 30);
    app.regions()
        .into_iter()
        .filter_map(|region| {
            region
                .path
                .strip_prefix("pages/p/sections/list/rows/")
                .filter(|rest| !rest.contains('/'))
                .map(str::to_owned)
        })
        .collect()
}

#[test]
fn a_column_of_the_row_itself_draws_sorts_and_keys_plain_strings() {
    let state = Scratch::new("ess-tui-plain-row");
    let mut app = app(
        "{name: list, component: collection, reads: t.Names, columns: [{field: ., name: name, label: Name, sortable: true}], sort: {by: ., allowed: [.]}}",
        &state,
    );
    let screen = app.render_text(100, 30);
    let alpha = screen.find("alpha");
    let beta = screen.find("beta");
    assert!(alpha.is_some() && beta.is_some(), "{screen}");
    assert!(alpha < beta, "sorted by the row itself: {screen}");
    assert_eq!(
        row_keys(&mut app),
        ["alpha", "beta"],
        "a plain row is keyed by itself"
    );
}

#[test]
fn the_row_itself_is_absent_on_an_object_row() {
    let state = Scratch::new("ess-tui-plain-row-object");
    let app = app(
        "{name: list, component: collection, reads: t.Item, columns: [label, {field: ., name: whole}]}",
        &state,
    );
    let screen = app.render_text(100, 30);
    assert!(screen.contains("An object"), "{screen}");
    assert!(
        !screen.contains("i-1"),
        "an object row reads as absent under `field: .`: {screen}"
    );
}
