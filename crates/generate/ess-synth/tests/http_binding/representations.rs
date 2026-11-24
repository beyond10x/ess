//! A view route's CSV representation, sent to both generated servers (beyond10x/ess#753,
//! `story:generated-servers-answer-a-view-as-csv-when-its-query-value-selects-it`): `Listing`
//! answers its rows as JSON, as `;`-separated CSV with a header line and `Content-Disposition`
//! under `?format=csv`, and as comma-separated CSV without a header line under `?format=plain`;
//! `ItemsTitled` reads its own `title` parameter beside the selector.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run_with, ir_of, rust_run_with};
use super::answer_forms::{go_harness, rust_harness, MODEL_EDITS};

/// The item gains an `Optional` note no command sets and an `Optional` rank `PutItem` sets from its
/// input, and the model a view of every item and one of every item with its rank.
const EDITS: &[(&str, &str)] = &[
    (
        "      - {name: title, type: String}\n    lifecycle:",
        "      - {name: title, type: String}\n      - {name: note, type: Optional<String>}\n      - {name: rank, type: Optional<Integer>}\n    lifecycle:",
    ),
    (
        "  - name: shop.items.PutItem\n    input:\n      - {name: item_id, type: shop.items.ItemId}\n      - {name: title, type: String}\n",
        "  - name: shop.items.PutItem\n    input:\n      - {name: item_id, type: shop.items.ItemId}\n      - {name: title, type: String}\n      - {name: rank, type: Optional<Integer>}\n",
    ),
    (
        "        sets: {title: input.title}\n",
        "        sets: {title: input.title, rank: input.rank}\n",
    ),
    (
        "actors:\n",
        "  - name: shop.items.Listing
    source: shop.items.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: shop.items.ItemId}
      - {name: title, type: String}
      - {name: note, type: Optional<String>}
  - name: shop.items.Ranked
    source: shop.items.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: shop.items.ItemId}
      - {name: title, type: String}
      - {name: rank, type: Optional<Integer>}
actors:
",
    ),
];

/// The three view routes, before `Purge`'s.
const ROUTE_EDITS: &[(&str, &str)] = &[(
    "  - command: shop.items.Purge\n",
    "  - view: shop.items.Listing
    method: get
    path: /listing
    body: array
    representations:
      - query: {format: csv}
        encoding: {csv: {separator: \";\", header: true}}
        headers:
          Content-Disposition: {literal: attachment}
      - query: {format: plain}
        encoding: {csv: {header: false}}
  - view: shop.items.ItemsTitled
    method: get
    path: /titled
    representations:
      - query: {format: csv}
        encoding: {csv: {}}
  - view: shop.items.Ranked
    method: get
    path: /ranked
    body: array
    representations:
      - query: {format: csv}
        encoding: {csv: {}}
  - command: shop.items.Purge\n",
)];

const REQUESTS: &[[&str; 7]] = &[
    [
        "put-north",
        "y",
        "PUT",
        "/items/r-1",
        "",
        "",
        r#"{"title":"north"}"#,
    ],
    [
        "put-separated",
        "y",
        "PUT",
        "/items/r-2",
        "",
        "",
        r#"{"title":"a;b"}"#,
    ],
    [
        "put-quoted",
        "y",
        "PUT",
        "/items/r-3",
        "",
        "",
        r#"{"title":"say \"hi\"\nnow"}"#,
    ],
    ["json", "y", "GET", "/listing", "", "", ""],
    ["csv", "y", "GET", "/listing", "format=csv", "", ""],
    ["plain", "y", "GET", "/listing", "format=plain", "", ""],
    ["encoded", "y", "GET", "/listing", "format=%63sv", "", ""],
    ["unknown", "y", "GET", "/listing", "format=xml", "", ""],
    [
        "twice",
        "y",
        "GET",
        "/listing",
        "format=csv&format=csv",
        "",
        "",
    ],
    ["titled", "y", "GET", "/titled", "title=north", "", ""],
    [
        "titled-csv",
        "y",
        "GET",
        "/titled",
        "format=csv&title=a%3Bb",
        "",
        "",
    ],
    // After every request above, so `Listing` answers its three rows: six titles a spreadsheet
    // would read as a formula, each opening with one of `=`, `+`, `-`, `@`, a tab and a CR.
    ["put-equals", "y", "PUT", "/items/r-4", "", "", r#"{"title":"=1+2","rank":-7}"#],
    ["put-plus", "y", "PUT", "/items/r-5", "", "", r#"{"title":"+1","rank":-7}"#],
    ["put-minus", "y", "PUT", "/items/r-6", "", "", r#"{"title":"-1","rank":-7}"#],
    ["put-at", "y", "PUT", "/items/r-7", "", "", r#"{"title":"@a","rank":-7}"#],
    ["put-tab", "y", "PUT", "/items/r-8", "", "", r#"{"title":"\tt","rank":-7}"#],
    ["put-return", "y", "PUT", "/items/r-9", "", "", r#"{"title":"\rr","rank":-7}"#],
    ["ranked", "y", "GET", "/ranked", "format=csv", "", ""],
];

/// `text` with every `(anchor, replacement)` applied, each anchor found once.
fn edited(text: &str, edits: &[(&str, &str)]) -> String {
    let mut text = text.to_owned();
    for (anchor, replacement) in edits {
        assert_eq!(
            text.matches(anchor).count(),
            1,
            "the fixture moved: {anchor}"
        );
        text = text.replace(anchor, replacement);
    }
    text
}

/// The Rust harness answering `Listing` and `Ranked` over the stored items, too.
fn rust() -> String {
    edited(
        &rust_harness(),
        &[
            ("ItemsTitledQuery, ", "ItemsTitledQuery, ListingQuery, RankedQuery, "),
            (
                "fn clean(text: &str) -> String {",
                "impl ListingQuery for Ports {
    fn listing(&self) -> Result<Vec<items::Listing>, UnmetObligation> {
        Ok(self
            .items
            .iter()
            .map(|row| items::Listing {
                item_id: row.data.item_id.clone(),
                title: row.data.title.clone(),
                note: row.data.note.clone(),
            })
            .collect())
    }
}

impl RankedQuery for Ports {
    fn ranked(&self) -> Result<Vec<items::Ranked>, UnmetObligation> {
        Ok(self
            .items
            .iter()
            .map(|row| items::Ranked {
                item_id: row.data.item_id.clone(),
                title: row.data.title.clone(),
                rank: row.data.rank,
            })
            .collect())
    }
}

fn clean(text: &str) -> String {",
            ),
        ],
    )
}

/// The Go harness answering `Listing` and `Ranked` over the stored items, too.
fn go() -> String {
    edited(
        &go_harness(),
        &[(
            "func clean(text string) string {",
            "func (p *ports) Listing() ([]items.Listing, *obligation.UnmetObligation) {
	found := []items.Listing{}
	for _, row := range p.rows {
		found = append(found, items.Listing{ItemId: row.Data.ItemId, Title: row.Data.Title, Note: row.Data.Note})
	}
	return found, nil
}

func (p *ports) Ranked() ([]items.Ranked, *obligation.UnmetObligation) {
	found := []items.Ranked{}
	for _, row := range p.rows {
		found = append(found, items.Ranked{ItemId: row.Data.ItemId, Title: row.Data.Title, Rank: row.Data.Rank})
	}
	return found, nil
}

func clean(text string) string {",
        )],
    )
}

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let model = edited(&edited(shared::MODEL, MODEL_EDITS), EDITS);
        let ir = ir_of(&model);
        let bound = shared::binding(&ir, &edited(shared::ROUTES, ROUTE_EDITS));
        let synthesis = |target| -> Synthesis {
            synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for {target:?}: {failure}"))
        };
        (
            rust_run_with("w753", &synthesis(Target::Rust), REQUESTS, &rust()),
            go_run_with("w753", &synthesis(Target::Go), REQUESTS, &go()),
        )
    })
}

/// Each server's answer to `label`, held to the same status, `Content-Type`, body bytes and
/// `Content-Disposition`.
fn answered(label: &str) -> &'static Answer {
    let (rust, go) = both();
    let rust = rust
        .get(label)
        .unwrap_or_else(|| panic!("the Rust server answered `{label}`"));
    let go = go
        .get(label)
        .unwrap_or_else(|| panic!("the Go server answered `{label}`"));
    assert_eq!(
        (rust.status, rust.content_type.as_str(), rust.body.as_str()),
        (go.status, go.content_type.as_str(), go.body.as_str()),
        "{label}: the servers disagree:\n  rust {rust:?}\n  go   {go:?}"
    );
    assert_eq!(
        rust.header("Content-Disposition"),
        go.header("Content-Disposition"),
        "{label}"
    );
    rust
}

const CSV: &str = "text/csv; charset=utf-8";

#[test]
fn both_servers_answer_a_view_as_csv_when_the_query_selects_it() {
    let json = answered("json");
    assert_eq!(json.status, 200, "{json:?}");
    assert_eq!(json.content_type, "application/json", "{json:?}");
    assert_eq!(
        json.json(),
        serde_json::json!([
            {"item_id": "r-1", "title": "north"},
            {"item_id": "r-2", "title": "a;b"},
            {"item_id": "r-3", "title": "say \"hi\"\nnow"},
        ]),
        "{json:?}"
    );
    assert_eq!(json.header("Content-Disposition"), None, "{json:?}");

    let csv = answered("csv");
    assert_eq!(
        (csv.status, csv.content_type.as_str()),
        (200, CSV),
        "{csv:?}"
    );
    assert_eq!(
        csv.body,
        "item_id;title;note\r\nr-1;north;\r\nr-2;\"a;b\";\r\nr-3;\"say \"\"hi\"\"\nnow\";\r\n",
        "{csv:?}"
    );
    assert_eq!(csv.header("Content-Disposition"), Some("attachment"));
    assert_eq!(
        answered("encoded").body,
        csv.body,
        "a percent-encoded value"
    );

    let plain = answered("plain");
    assert_eq!((plain.status, plain.content_type.as_str()), (200, CSV));
    assert_eq!(
        plain.body, "r-1,north,\r\nr-2,a;b,\r\nr-3,\"say \"\"hi\"\"\nnow\",\r\n",
        "{plain:?}"
    );
    assert_eq!(plain.header("Content-Disposition"), None, "{plain:?}");
}

/// Answered as the binding's `refusals.malformed` declares it: `400`, `{"error": "invalid_request"}`.
#[test]
fn a_value_no_representation_names_and_a_repeated_selector_are_malformed() {
    for label in ["unknown", "twice"] {
        let answer = answered(label);
        assert_eq!(answer.status, 400, "{label}: {answer:?}");
        assert_eq!(
            answer.json(),
            serde_json::json!({"error": "invalid_request"}),
            "{label}"
        );
    }
}

#[test]
fn a_view_reads_its_own_parameters_beside_the_selector() {
    let titled = answered("titled");
    assert_eq!(
        titled.json(),
        serde_json::json!({"rows": [{"item_id": "r-1", "title": "north"}]}),
        "{titled:?}"
    );
    let csv = answered("titled-csv");
    assert_eq!((csv.status, csv.content_type.as_str()), (200, CSV));
    assert_eq!(csv.body, "item_id,title\r\nr-2,a;b\r\n", "{csv:?}");
}

/// A text cell opening with `=`, `+`, `-`, `@`, a tab or a CR is written behind `'`, before the
/// quoting, so a spreadsheet does not read it as a formula; the negative `Integer` rank is written
/// as it is, an absent one as an empty field, and both servers write the same bytes.
#[test]
fn a_text_cell_a_spreadsheet_reads_as_a_formula_is_written_behind_an_apostrophe() {
    let csv = answered("ranked");
    assert_eq!((csv.status, csv.content_type.as_str()), (200, CSV));
    assert_eq!(
        csv.body,
        "item_id,title,rank\r\nr-1,north,\r\nr-2,a;b,\r\nr-3,\"say \"\"hi\"\"\nnow\",\r\n\
         r-4,'=1+2,-7\r\nr-5,'+1,-7\r\nr-6,'-1,-7\r\nr-7,'@a,-7\r\nr-8,'\tt,-7\r\n\
         r-9,\"'\rr\",-7\r\n",
        "{csv:?}"
    );
}
