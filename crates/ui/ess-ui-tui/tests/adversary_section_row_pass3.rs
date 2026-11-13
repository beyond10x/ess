//! Adversary pass 3 on `section.<name>.row`, after the fix for pass 2: the terminal's value for
//! the documents of `ess-ui-react/tests/adversary_section_row_pass3.rs`. `pick` chooses the order
//! `order` reads by `section.pick.selection.id`; `customer` reads by
//! `section.order.row.customer_id`. `o-2` is picked, and `pick` reads again for the same request
//! (`R`); its second answer is in another order, or fails. The React project holds the record
//! picked (`o-2`, so `customer` reads `c-2`) in both cases.

use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::Duration;

use ess_ui::binding::Answer;
use ess_ui_tui::{App, DataAdapter, FixtureAdapter, Lifecycle, Options, ReadRequest, ReadResult};
use ess_yaml::Value;

const DOCUMENT: &str = r"
format: ess-ui/1
app: shop
model: shop.system
placement_profile: fat
fixtures:
  views:
    orders.All: orders.yaml
    orders.ById: orders.yaml
    customers.ById: customers.yaml
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: order
  sections: [{name: all, pages: [order]}]
pages:
  order:
    kind: detail_page
    title: Order
    sections:
      - {name: pick, component: collection, selection: single, reads: {view: orders.All, key: id}, columns: [id]}
      - {name: order, component: record, reads: {view: orders.ById, key: id, params: {id: section.pick.selection.id}}, fields: [id]}
      - {name: customer, component: record, reads: {view: customers.ById, key: id, params: {id: section.order.row.customer_id}}, fields: [email]}
";

const FIXTURES: &[(&str, &str)] = &[
    (
        "orders.yaml",
        "views:\n  orders.All:\n    rows:\n      - {id: o-1}\n      - {id: o-2}\n  \
         orders.ById:\n    rows:\n      - {id: o-1, customer_id: c-1}\n      \
         - {id: o-2, customer_id: c-2}\n      - {id: o-3, customer_id: c-3}\n",
    ),
    (
        "customers.yaml",
        "views:\n  customers.ById:\n    rows:\n      - {id: c-1, email: ada@shop.test}\n      \
         - {id: c-2, email: grace@shop.test}\n      - {id: c-3, email: alan@shop.test}\n",
    ),
];

/// How `orders.All` answers from now on.
#[derive(Clone, Copy, PartialEq)]
enum Orders {
    Fixture,
    NewFirst,
    Fails,
}

/// The fixtures, with `orders.All` answering as `mode` says.
struct Switch {
    inner: FixtureAdapter,
    mode: Rc<Cell<Orders>>,
}

impl DataAdapter for Switch {
    fn read(&self, request: &ReadRequest) -> Result<ReadResult, String> {
        if request.view != "orders.All" {
            return self.inner.read(request);
        }
        match self.mode.get() {
            Orders::Fixture => self.inner.read(request),
            Orders::NewFirst => {
                let rows: Vec<Value> = ["o-3", "o-1", "o-2"]
                    .iter()
                    .map(|id| ess_yaml::from_str(&format!("{{id: {id}}}")).expect("a row"))
                    .collect();
                Ok(ReadResult {
                    total: Some(rows.len() as u64),
                    rows,
                })
            }
            Orders::Fails => Err("orders are down".to_owned()),
        }
    }

    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Answer {
        self.inner.run(command, input)
    }

    fn load_state(&self, path: &str) -> Option<Value> {
        self.inner.load_state(path)
    }

    fn store_state(&mut self, path: &str, value: Value) {
        self.inner.store_state(path, value);
    }
}

fn open(name: &str) -> (App, Rc<Cell<Orders>>) {
    let dir: PathBuf = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adversary-section-row-pass3")
        .join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale dir is removed");
    }
    std::fs::create_dir_all(&dir).expect("the dir is created");
    for (name, body) in FIXTURES {
        std::fs::write(dir.join(name), body).expect("the fixture is written");
    }
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let (inner, _) = FixtureAdapter::load(&document, &dir, None).expect("the fixtures load");
    let mode = Rc::new(Cell::new(Orders::Fixture));
    let adapter = Switch {
        inner,
        mode: Rc::clone(&mode),
    };
    let mut options = Options::new(dir.join("state"));
    options
        .view_latency
        .insert("orders.All".to_owned(), Duration::from_secs(1));
    let mut app = App::with_adapter(document, Box::new(adapter), vec![], options)
        .unwrap_or_else(|error| panic!("{error}"));
    app.open_page("order", &[]);
    app.advance(Duration::from_secs(1));
    app.focus_section("pick");
    app.keys("j");
    assert_eq!(ids(&app, "order"), ["o-2"]);
    assert_eq!(ids(&app, "customer"), ["c-2"]);
    (app, mode)
}

fn ids(app: &App, section: &str) -> Vec<String> {
    app.rows(section)
        .iter()
        .map(|row| row["id"].as_str().unwrap_or_default().to_owned())
        .collect()
}

/// The second answer has a new order first (`o-3`, as after a command that creates one). The
/// selection was held on `o-2` while `pick` read again; when the answer arrives the cursor keeps
/// its index, not its row, so the selection becomes `o-1` and `order` and `customer` follow it.
#[test]
#[ignore = "story:the-terminals-selection-stays-on-its-record-when-its-collection-reads-again (backlog)"]
fn a_selection_is_kept_on_its_record_when_the_reread_answers_in_another_order() {
    let (mut app, mode) = open("new-first");
    mode.set(Orders::NewFirst);
    app.keys("R");
    assert_eq!(app.section_state("pick"), Lifecycle::Loading);
    assert_eq!(ids(&app, "customer"), ["c-2"]);
    app.advance(Duration::from_secs(1));
    assert_eq!(ids(&app, "pick"), ["o-3", "o-1", "o-2"]);
    assert_eq!(
        ids(&app, "order"),
        ["o-2"],
        "the cursor stays on the row picked when the answer arrives"
    );
    assert_eq!(ids(&app, "customer"), ["c-2"]);
}

/// The second read fails. The terminal then holds no selection, so `order` reads with no id,
/// its first row is `o-1`, and `customer` reads `c-1`, a customer of an order nobody picked;
/// the React project keeps the picked `o-2` and reads `c-2`.
#[test]
#[ignore = "story:the-terminals-selection-stays-on-its-record-when-its-collection-reads-again (backlog)"]
fn a_selection_after_its_collections_reread_fails() {
    let (mut app, mode) = open("fails");
    mode.set(Orders::Fails);
    app.keys("R");
    assert_eq!(ids(&app, "customer"), ["c-2"]);
    app.advance(Duration::from_secs(1));
    assert_eq!(app.section_state("pick"), Lifecycle::Failed);
    assert_eq!(
        ids(&app, "customer"),
        ["c-2"],
        "both renderers bind the same customer after pick's read fails"
    );
}
