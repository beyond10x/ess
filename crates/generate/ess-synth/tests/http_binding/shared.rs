//! What `tests/http_binding_rust.rs` and its module `tests/http_binding/go.rs` share: the model
//! and the `ess-http/1` binding both servers serve, every request both are sent, and the Rust
//! server's answers to them (beyond10x/ess#493, stage 2,
//! `docs/design/http-presentation-binding.md`, "Precedence" and "What the servers write (k5,
//! k6)").
//!
//! One harness per server drives the generated surface's own dispatch with every request in
//! [`REQUESTS`] and prints each answer as `label<TAB>status<TAB>content type<TAB>headers<TAB>body`,
//! the headers `name: value` joined by ` | `, and a tab, a line feed and a carriage return in
//! either written as the unit, record and group separators, which [`restored`] reads back.

#![allow(dead_code)]

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::ess_http;
use ess_gen::http::Binding;
use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

pub const MODEL: &str = include_str!("../fixtures/http-binding-shop.yaml");
pub const ROUTES: &str = include_str!("../fixtures/http-binding-shop.routes.yaml");

/// Every request the harness sends: `label`, whether the caller is the granted `Clerk` (`y`) or
/// no actor (`n`), method, path, query, `Content-Type` (empty for none) and body.
pub const REQUESTS: &[[&str; 7]] = &[
    // Two methods and a view on one templated path.
    [
        "put",
        "y",
        "PUT",
        "/items/a%20b",
        "",
        "",
        r#"{"title":"Lamp"}"#,
    ],
    ["get", "y", "GET", "/items/a%20b", "", "", ""],
    ["get-missing", "y", "GET", "/items/zz", "", "", ""],
    ["delete", "y", "DELETE", "/items/a%20b", "", "", ""],
    ["post-on-items", "y", "POST", "/items/a", "", "", "{}"],
    ["nowhere", "y", "GET", "/items/a/b", "", "", ""],
    [
        "empty-variable",
        "y",
        "PUT",
        "/items/",
        "",
        "",
        r#"{"title":"Lamp"}"#,
    ],
    [
        "put-body-names-path",
        "y",
        "PUT",
        "/items/a",
        "",
        "",
        r#"{"item_id":"b","title":"Lamp"}"#,
    ],
    [
        "put-bad-escape",
        "y",
        "PUT",
        "/items/a%zz",
        "",
        "",
        r#"{"title":"Lamp"}"#,
    ],
    // A `json` body read as the Rust server's `json::parse` reads one, where `encoding/json`
    // reads it otherwise: a lone surrogate escape and nesting past 64 are refused, a raw control
    // character in a string and a leading zero are read. A member no field declares is not read.
    [
        "put-lone-surrogate",
        "y",
        "PUT",
        "/items/j1",
        "",
        "",
        r#"{"title":"\ud800"}"#,
    ],
    [
        "put-raw-control",
        "y",
        "PUT",
        "/items/j2",
        "",
        "",
        "{\"title\":\"a\u{1}b\"}",
    ],
    [
        "put-nested-65",
        "y",
        "PUT",
        "/items/j3",
        "",
        "",
        concat!(
            r#"{"title":"Lamp","extra":"#,
            "[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[[",
            "]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]]",
            "}"
        ),
    ],
    [
        "put-leading-zero",
        "y",
        "PUT",
        "/items/j4",
        "",
        "",
        r#"{"title":"Lamp","extra":-01}"#,
    ],
    ["docs", "y", "GET", "/docs", "", "", ""],
    ["openapi", "y", "GET", "/openapi.json", "", "", ""],
    // `form`, `ignore` and `refuse`.
    [
        "form-ignore",
        "y",
        "POST",
        "/orders",
        "",
        "application/x-www-form-urlencoded; charset=UTF-8",
        "item_id=x+1&count=2&utm_source=mail",
    ],
    [
        "form-ignore-unlisted",
        "y",
        "POST",
        "/orders",
        "",
        "application/x-www-form-urlencoded",
        "item_id=x&count=9",
    ],
    [
        "form-refuse",
        "y",
        "POST",
        "/strict-orders",
        "",
        "application/x-www-form-urlencoded",
        "item_id=x&count=2&utm_source=mail",
    ],
    [
        "form-twice",
        "y",
        "POST",
        "/strict-orders",
        "",
        "application/x-www-form-urlencoded",
        "item_id=x&item_id=y&count=2",
    ],
    [
        "form-empty-integer",
        "y",
        "POST",
        "/strict-orders",
        "",
        "application/x-www-form-urlencoded",
        "item_id=x&count=",
    ],
    [
        "form-json-type",
        "y",
        "POST",
        "/strict-orders",
        "",
        "application/json",
        "item_id=x&count=2",
    ],
    [
        "form-no-type",
        "y",
        "POST",
        "/strict-orders",
        "",
        "",
        "item_id=x&count=2",
    ],
    [
        "form-strict-ok",
        "y",
        "POST",
        "/strict-orders",
        "",
        "application/x-www-form-urlencoded",
        "count=3&item_id=%C3%A9",
    ],
    // `query`, with a `List`.
    [
        "query-list",
        "y",
        "GET",
        "/search",
        "words=a&words=b+c&limit=3",
        "",
        "",
    ],
    ["query-list-none", "y", "GET", "/search", "limit=1", "", ""],
    [
        "query-scalar-twice",
        "y",
        "GET",
        "/search",
        "limit=1&limit=2",
        "",
        "",
    ],
    // Declared status, body and headers per outcome.
    [
        "placed",
        "y",
        "POST",
        "/places",
        "",
        "",
        r#"{"item_id":"i1","count":1}"#,
    ],
    [
        "refused",
        "y",
        "POST",
        "/places",
        "",
        "",
        r#"{"item_id":"i1","count":9}"#,
    ],
    [
        "placed-header-injection",
        "y",
        "POST",
        "/places",
        "",
        "",
        "{\"item_id\":\"i1\\r\\nSet-Cookie: x=1\",\"count\":1}",
    ],
    // A bearer challenge.
    [
        "challenge",
        "y",
        "GET",
        "/private/p1",
        "level=0&hint=a%22b%5Cc",
        "",
        "",
    ],
    [
        "challenge-read",
        "y",
        "GET",
        "/private/p1",
        "level=2&hint=h",
        "",
        "",
    ],
    [
        "challenge-control",
        "y",
        "GET",
        "/private/p1",
        "level=0&hint=a%0Db",
        "",
        "",
    ],
    // Redirects.
    [
        "redirect-query",
        "y",
        "GET",
        "/approve",
        "return_to=https%3A%2F%2Fclient.example%2Fcb%3Fx%3D1&state=s+1&client=c%26d&level=2",
        "",
        "",
    ],
    [
        "redirect-query-absent",
        "y",
        "GET",
        "/approve",
        "return_to=https%3A%2F%2Fclient.example%2Fcb&client=c&level=2",
        "",
        "",
    ],
    [
        "redirect-fragment",
        "y",
        "GET",
        "/approve",
        "return_to=https%3A%2F%2Fclient.example%2Fcb&state=s&client=c&level=0",
        "",
        "",
    ],
    [
        "redirect-fragment-absent",
        "y",
        "GET",
        "/approve",
        "return_to=https%3A%2F%2Fclient.example%2Fcb&client=c&level=0",
        "",
        "",
    ],
    [
        "redirect-relative",
        "y",
        "GET",
        "/approve",
        "return_to=%2Fcb&client=c&level=2",
        "",
        "",
    ],
    [
        "redirect-with-fragment",
        "y",
        "GET",
        "/approve",
        "return_to=https%3A%2F%2Fclient.example%2Fcb%23top&client=c&level=2",
        "",
        "",
    ],
    // Standard refusals: overridden on the routes the binding names, today's elsewhere.
    ["not-granted", "y", "POST", "/purge/p1", "", "", ""],
    [
        "not-granted-anonymous",
        "n",
        "POST",
        "/purge/p1",
        "",
        "",
        "",
    ],
    [
        "not-granted-derived",
        "y",
        "POST",
        "/items/commands/Archive",
        "",
        "",
        r#"{"item_id":"p1"}"#,
    ],
    [
        "unfinished",
        "y",
        "POST",
        "/places",
        "",
        "",
        r#"{"item_id":"i1","count":0}"#,
    ],
    ["malformed-json", "y", "POST", "/places", "", "", "{"],
];

/// The Rust server's harness, printing one line per request as the module documentation says.
pub const RUST_HARNESS: &str = r#"use shop_server::http;
use shop_server::shop_service as surface;
use shop_types::actor::{Actor, Caller};
use shop_types::behaviour::{Generated, ItemStorage};
use shop_types::items;
use shop_types::items::obligations::{ItemByIdQuery, PlaceBehavior};
use shop_types::obligation::UnmetObligation;

#[derive(Default)]
struct Ports {
    items: Vec<items::ItemSnapshot>,
}

impl ItemStorage for Ports {
    fn get(&self, identity: &items::ItemId) -> Option<items::ItemSnapshot> {
        self.items.iter().find(|row| &row.data.item_id == identity).cloned()
    }
    fn put(&mut self, snapshot: items::ItemSnapshot) {
        self.delete(&snapshot.data.item_id);
        self.items.push(snapshot);
    }
    fn delete(&mut self, identity: &items::ItemId) {
        self.items.retain(|row| &row.data.item_id != identity);
    }
}

impl PlaceBehavior for Ports {
    fn place(&mut self, input: items::Place) -> Result<items::PlaceOutcome, UnmetObligation> {
        if input.count == 0 {
            return Err(UnmetObligation { capability: "command-behaviour", source: "shop.items.Place" });
        }
        if input.count > 5 {
            return Ok(items::PlaceOutcome::Refused { error: items::OutOfStock });
        }
        Ok(items::PlaceOutcome::Placed {
            response: items::PlaceResponse { code: format!("C-{}", input.item_id.0) },
            touched: items::Touched { item_id: input.item_id },
        })
    }
}

impl ItemByIdQuery for Ports {
    fn item_by_id(&self, item_id: items::ItemId) -> Result<Vec<items::ItemById>, UnmetObligation> {
        Ok(self
            .items
            .iter()
            .filter(|row| row.data.item_id == item_id)
            .map(|row| items::ItemById { item_id: row.data.item_id.clone(), title: row.data.title.clone() })
            .collect())
    }
}

fn clean(text: &str) -> String {
    text.replace('\t', "\u{1f}").replace('\n', "\u{1e}").replace('\r', "\u{1d}")
}

fn main() {
    let requests = std::fs::read_to_string(std::env::args().nth(1).expect("the requests")).expect("read");
    let mut system = shop_system::System::new(shop_service::ShopService::new(Generated::new(Ports::default())));
    let clerk = Caller { actor: Actor::ALL[0] };
    for line in requests.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let [label, caller, method, path, query, content_type, body] = fields[..] else {
            panic!("a request line has seven fields: {line}");
        };
        let request = http::Request {
            method: method.to_owned(),
            path: path.to_owned(),
            query: query.to_owned(),
            headers: if content_type.is_empty() {
                Vec::new()
            } else {
                vec![("content-type".to_owned(), content_type.to_owned())]
            },
            body: body.as_bytes().to_vec(),
        };
        let caller = (caller == "y").then_some(&clerk);
        let answered = surface::dispatch(&mut system, caller, &request);
        let headers: Vec<String> = answered
            .headers
            .iter()
            .map(|(name, value)| format!("{name}: {value}"))
            .collect();
        println!(
            "{label}\t{}\t{}\t{}\t{}",
            answered.status,
            answered.content_type,
            clean(&headers.join(" | ")),
            clean(&answered.body)
        );
    }
}
"#;

pub fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("shop.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("shop.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// `routes` as a whole `ess-http/1` document for `ir`.
pub fn document(ir: &EssIr, routes: &str) -> String {
    format!(
        "type: ess-http/1\nspecification:\n  system: shop\n  version: v1\n  source_digest: \
         sha256:{}\n{routes}",
        ir.source_digest()
    )
}

/// `routes` compiled against `ir` and held to a path on every route, as `--http` reads them.
pub fn binding(ir: &EssIr, routes: &str) -> Binding {
    let spec = ess_http::HttpSpec::from_yaml(&document(ir, routes)).expect("well formed");
    let compiled = ess_http::compile(&spec, ir, &ess_gen::http::derived(ir))
        .unwrap_or_else(|refused| panic!("the binding compiles: {refused}"));
    Binding::new(compiled).unwrap_or_else(|refused| panic!("every route has a path: {refused}"))
}

pub fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

/// Every request of [`REQUESTS`], one tab-separated line each, written to `path`.
pub fn write_requests(path: &Path) {
    std::fs::write(
        path,
        REQUESTS
            .iter()
            .map(|request| request.join("\t"))
            .collect::<Vec<_>>()
            .join("\n"),
    )
    .expect("write");
}

fn cargo(directory: &Path, arguments: &[String]) -> Output {
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(arguments)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    eprintln!(
        "cargo {arguments:?} in {}\n{}",
        directory.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// One answer a harness printed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answer {
    pub status: u16,
    pub content_type: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

impl Answer {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(named, _)| named == name)
            .map(|(_, value)| value.as_str())
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body)
            .unwrap_or_else(|error| panic!("the body is JSON ({error}): {}", self.body))
    }
}

/// The answers a harness printed, one line each, by label.
pub fn parsed<'a>(lines: impl Iterator<Item = &'a str>) -> BTreeMap<String, Answer> {
    lines
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            let headers = if fields[3].is_empty() {
                Vec::new()
            } else {
                fields[3]
                    .split(" | ")
                    .map(|header| {
                        let (name, value) = header.split_once(": ").expect("name: value");
                        (name.to_owned(), restored(value))
                    })
                    .collect()
            };
            (
                fields[0].to_owned(),
                Answer {
                    status: fields[1].parse().expect("a status"),
                    content_type: fields[2].to_owned(),
                    headers,
                    body: restored(fields[4]),
                },
            )
        })
        .collect()
}

/// The Rust server's answers by label, once per test binary.
pub fn rust_answers() -> &'static BTreeMap<String, Answer> {
    &rust_run().0
}

/// The events `dispatch_with_events` handed back with each answer of [`rust_answers`], by label,
/// as a JSON array of the envelopes `encode_system_event` writes.
pub fn rust_events() -> &'static BTreeMap<String, serde_json::Value> {
    &rust_run().1
}

/// [`RUST_HARNESS`] answering through `dispatch_with_events`, which also prints the events each
/// answer handed back, as `events<TAB>label<TAB>[envelope, …]`.
fn rust_events_harness() -> String {
    let dispatch = "let answered = surface::dispatch(&mut system, caller, &request);";
    assert!(RUST_HARNESS.contains(dispatch), "the harness dispatches");
    RUST_HARNESS.replace(
        dispatch,
        "let (answered, events) = surface::dispatch_with_events(&mut system, caller, &request);\n        \
         let events: Vec<String> = events.iter().map(shop_server::wire::encode_system_event).collect();\n        \
         println!(\"events\\t{label}\\t[{}]\", events.join(\",\"));",
    )
}

/// [`rust_answers`] and [`rust_events`], from one run of the harness.
fn rust_run() -> &'static (
    BTreeMap<String, Answer>,
    BTreeMap<String, serde_json::Value>,
) {
    static ANSWERS: std::sync::OnceLock<(
        BTreeMap<String, Answer>,
        BTreeMap<String, serde_json::Value>,
    )> = std::sync::OnceLock::new();
    ANSWERS.get_or_init(|| {
        let ir = ir();
        let bound = binding(&ir, ROUTES);
        let synthesis = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("the bound model synthesizes: {failure}"));
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("http-binding-rust-{}", std::process::id()));
        write(&synthesis, &root.join("shop"));
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).expect("mkdir");
        let dependencies = ["shop-server", "shop-system", "shop-service", "shop-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../shop/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"http-binding-rust-harness\"\nversion = \"0.0.0\"\n\
                 edition = \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/main.rs"), rust_events_harness()).expect("write");
        let requests = root.join("requests.tsv");
        write_requests(&requests);
        let target = root.join("target");
        let arguments: Vec<String> = [
            "run",
            "--offline",
            "--quiet",
            "--target-dir",
            target.to_str().expect("UTF-8"),
            "--",
            requests.to_str().expect("UTF-8"),
        ]
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect();
        let ran = cargo(&harness, &arguments);
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        let _ = std::fs::remove_dir_all(&root);
        assert!(ran.status.success(), "the harness builds and runs");
        eprintln!("{stdout}");
        let events = stdout
            .lines()
            .filter_map(|line| line.strip_prefix("events\t"))
            .map(|line| {
                let (label, listed) = line.split_once('\t').expect("label<TAB>events");
                let listed = serde_json::from_str(listed)
                    .unwrap_or_else(|error| panic!("`{label}`'s events are JSON ({error})"));
                (label.to_owned(), listed)
            })
            .collect();
        (
            parsed(stdout.lines().filter(|line| !line.starts_with("events\t"))),
            events,
        )
    })
}

/// A harness field with its tabs, line feeds and carriage returns put back.
pub fn restored(text: &str) -> String {
    text.replace('\u{1f}', "\t")
        .replace('\u{1e}', "\n")
        .replace('\u{1d}', "\r")
}

/// The one event a command answer published, as its payload.
pub fn published(answer: &Answer) -> serde_json::Value {
    let body = answer.json();
    let events = body["published"]
        .as_array()
        .unwrap_or_else(|| panic!("an envelope lists what it published: {body}"));
    assert_eq!(events.len(), 1, "{body}");
    events[0]["payload"].clone()
}
