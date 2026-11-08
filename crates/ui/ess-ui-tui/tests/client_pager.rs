//! A client-paged collection pages by its pager's sizes, draws the from-to range and takes a
//! size choice in the terminal (story:a-client-paged-collection-offers-sizes-and-a-range).
use ess_ui_tui::{App, DataAdapter, Options, ReadRequest, ReadResult};
use ess_yaml::Value;
use std::{collections::BTreeMap, path::PathBuf, time::Duration};

/// A state directory `<TMPDIR>/<name>-<pid>`, removed when dropped, panics included.
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

/// Thirty rows: `keep-0` to `keep-3`, then `other-4` to `other-29`.
struct Rows;
impl DataAdapter for Rows {
    fn read(&self, _: &ReadRequest) -> Result<ReadResult, String> {
        let rows = (0..30)
            .map(|index| {
                let name = if index < 4 {
                    format!("keep-{index}")
                } else {
                    format!("other-{index}")
                };
                ess_yaml::from_str(&format!("{{id: r-{index}, name: {name}}}")).unwrap()
            })
            .collect();
        Ok(ReadResult {
            rows,
            total: Some(30),
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

fn app(reads: &str, state: &Scratch) -> App {
    let document = format!(
        "format: ess-ui/1
app: pager
model: t
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: detail_page
    title: Paged
    sections:
      - {{name: list, component: collection, reads: {reads}, columns: [name]}}
"
    );
    let mut app = App::with_adapter(
        ess_ui::load_str(&document).unwrap(),
        Box::new(Rows),
        vec![],
        Options::new(state.0.clone()),
    )
    .unwrap();
    app.advance(Duration::from_secs(1));
    app.focus_section("list");
    app
}

/// The drawn rows, by their names.
fn shown(screen: &str) -> Vec<String> {
    screen
        .split(|character: char| !(character.is_ascii_alphanumeric() || character == '-'))
        .filter(|word| word.starts_with("keep-") || word.starts_with("other-"))
        .map(str::to_owned)
        .collect()
}

#[test]
fn a_client_pager_pages_by_its_sizes_and_draws_the_range() {
    let state = Scratch::new("ess-tui-client-pager-sizes");
    let mut app = app(
        "{view: t.All, paging: client, pager: {sizes: [10, 25]}}",
        &state,
    );
    let first = app.render_text(120, 60);
    assert_eq!(shown(&first).len(), 10, "{first}");
    assert!(first.contains("page 1/3 · 30 rows · 1-10 of 30"), "{first}");
    // `+` chooses the next size, as the React per-page select does.
    app.keys("+");
    let sized = app.render_text(120, 60);
    assert_eq!(shown(&sized).len(), 25, "{sized}");
    assert!(sized.contains("page 1/2 · 30 rows · 1-25 of 30"), "{sized}");
    app.keys("n");
    let last = app.render_text(120, 60);
    assert_eq!(shown(&last).len(), 5, "{last}");
    assert!(last.contains("page 2/2 · 30 rows · 26-30 of 30"), "{last}");
    // Choosing a size returns to page 1; past the last size it comes back to the first.
    app.keys("+");
    let back = app.render_text(120, 60);
    assert!(back.contains("page 1/3 · 30 rows · 1-10 of 30"), "{back}");
}

#[test]
fn a_search_returns_the_client_pager_to_page_one() {
    let state = Scratch::new("ess-tui-client-pager-search");
    let mut app = app(
        "{view: t.All, paging: client, pager: {sizes: [10, 25]}}",
        &state,
    );
    app.keys("nn");
    let third = app.render_text(120, 60);
    assert!(
        third.contains("page 3/3 · 30 rows · 21-30 of 30"),
        "{third}"
    );
    app.keys("/keep<enter>");
    let found = app.render_text(120, 60);
    assert_eq!(
        shown(&found),
        ["keep-0", "keep-1", "keep-2", "keep-3"],
        "{found}"
    );
    assert!(found.contains("page 1/1 · 4 rows · 1-4 of 4"), "{found}");
}

#[test]
fn an_unpaged_collection_keeps_its_footer() {
    let state = Scratch::new("ess-tui-client-pager-unpaged");
    let mut app = app("{view: t.All}", &state);
    let screen = app.render_text(120, 60);
    assert!(screen.contains("page 1/3 · 30 rows"), "{screen}");
    assert!(!screen.contains(" of 30"), "{screen}");
    app.keys("+");
    assert!(
        app.render_text(120, 60).contains("page 1/3 · 30 rows"),
        "a collection without a pager has one page size"
    );
}
