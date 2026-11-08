//! Adversary pass on story:a-client-paged-collection-offers-sizes-and-a-range (terminal).
//!
//! 1. "Every paged collection, client or server, shows a from-to of total label" and "the
//!    terminal draws the same label": on a server-paged read the range must be the server's
//!    page of the server's total, as React draws it (`31-34 of 34` on page 4 of 34 rows, the
//!    case `ess-ui-react/tests/client_pager.rs` pins), not the rows the answer happens to hold;
//!    and a server pager without a `size` draws none, as React draws none (`server-unsized`).
//! 2. `+` chooses the next of a client pager's sizes, so every listed size must be reachable:
//!    `ess ui check` refuses a `sizes` list that repeats a size (`pager_sizes`), and `+` reaches
//!    every size of a list it accepts.
use ess_ui_tui::{App, DataAdapter, Options, ReadRequest, ReadResult};
use ess_yaml::Value;
use std::{collections::BTreeMap, path::Path, path::PathBuf, time::Duration};

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

/// A server of 34 rows, `row-0` to `row-33`, paged by its `page` and `size` params (size 10
/// when `size` is absent); it answers one page and the total.
struct Server;
impl DataAdapter for Server {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        let number = |name: &str, default: u64| {
            request
                .params
                .get(name)
                .and_then(|value| match value {
                    Value::Number(number) => number.as_u64(),
                    Value::String(text) => text.parse().ok(),
                    _ => None,
                })
                .unwrap_or(default)
        };
        // Without a `page` param (a client-paged read) it answers every row.
        let size = if request.params.contains_key("page") {
            number("size", 10)
        } else {
            34
        };
        let page = number("page", 1).max(1);
        let rows = ((page - 1) * size..(page * size).min(34))
            .map(|index| ess_yaml::from_str(&format!("{{id: r-{index}, name: row-{index}}}")).unwrap())
            .collect();
        Ok(ReadResult {
            rows,
            total: Some(34),
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

fn document(reads: &str) -> String {
    format!(
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
    state:
      p: {{type: integer, class: page_state, store: url, default: 4}}
      n: {{type: integer, class: page_state, store: url, default: 10}}
    sections:
      - {{name: list, component: collection, reads: {reads}, columns: [name]}}
"
    )
}

fn app(reads: &str, state: &Scratch) -> App {
    let mut app = App::with_adapter(
        ess_ui::load_str(&document(reads)).unwrap(),
        Box::new(Server),
        vec![],
        Options::new(state.0.clone()),
    )
    .unwrap();
    app.advance(Duration::from_secs(1));
    app.focus_section("list");
    app
}

#[test]
fn a_server_pager_range_is_the_servers_page_of_the_servers_total() {
    let state = Scratch::new("ess-tui-adv-w091-server-range");
    let app = app(
        "{view: t.All, params: {page: state.p, size: state.n}, paging: server, pager: {page: page, size: size, sizes: [10, 25]}}",
        &state,
    );
    let screen = app.render_text(120, 60);
    assert!(
        screen.contains("row-31") && screen.contains("row-33"),
        "the server answers its page 4: {screen}"
    );
    // React draws `31-34 of 34` for this read (ess-ui-react/tests/client_pager.rs,
    // `a_server_pager_with_a_known_size_shows_its_range`); the terminal must draw the same label.
    assert!(
        screen.contains("31-34 of 34"),
        "the terminal's range must be the server's page of the server's total: {screen}"
    );
}

#[test]
fn a_server_pager_without_a_size_draws_no_range() {
    let state = Scratch::new("ess-tui-adv-w091-server-unsized");
    let app = app(
        "{view: t.All, params: {page: state.p}, paging: server, pager: {page: page}}",
        &state,
    );
    let screen = app.render_text(120, 60);
    assert!(
        screen.contains("row-31") && screen.contains("row-33"),
        "the server answers its page 4: {screen}"
    );
    // React's `server-unsized` case: without a size the range is unknown, so none is drawn.
    assert!(
        !screen.contains(" of 34") && !screen.contains(" of 4"),
        "a server pager without a size has no range to draw: {screen}"
    );
}

#[test]
fn plus_reaches_every_size_a_client_pager_lists() {
    // A repeated size would hold `+` on it, and React would key two options alike:
    // `ess ui check` refuses the list.
    let report = ess_ui_check::check_source(
        &document("{view: t.All, paging: client, pager: {sizes: [10, 10, 25]}}"),
        "pager.yaml",
        Path::new("."),
        None,
        &ess_ui_check::Options::default(),
    );
    assert!(
        report.findings.iter().any(|finding| {
            finding.check == "pager_sizes"
                && finding.path == "pages/p/sections/list/reads/pager/sizes"
                && finding.severity == ess_ui_check::Severity::Error
        }),
        "`ess ui check` must refuse a client pager's repeated size: {:#?}",
        report.findings
    );
    // A list it accepts: `+` reaches every size, in order, and comes back round.
    let state = Scratch::new("ess-tui-adv-w091-every-size");
    let mut app = app(
        "{view: t.All, paging: client, pager: {sizes: [10, 25, 50]}}",
        &state,
    );
    let first = app.render_text(120, 60);
    assert!(first.contains("+ 10 per page"), "{first}");
    let mut seen = Vec::new();
    for _ in 0..3 {
        app.keys("+");
        let screen = app.render_text(120, 60);
        seen.push(
            screen
                .lines()
                .find(|line| line.contains("per page"))
                .unwrap_or_default()
                .trim()
                .to_owned(),
        );
    }
    for size in [25, 50, 10] {
        assert!(
            seen.iter()
                .any(|line| line.contains(&format!("+ {size} per page"))),
            "pressing `+` three times never offers {size}, a size the pager lists: {seen:#?}"
        );
    }
}
