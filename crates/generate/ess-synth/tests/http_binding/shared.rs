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
    // character in a string and a leading zero are read. A member no field declares is malformed
    // (story:a-json-command-body-refuses-an-unknown-member), so `put-leading-zero` is refused
    // for its member and `place-leading-zero` reads the leading zero in a declared field.
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
    [
        "place-leading-zero",
        "y",
        "POST",
        "/places",
        "",
        "",
        r#"{"item_id":"j5","count":-01}"#,
    ],
    // A JSON command body with a member no input field names, under the default `unknown:
    // refuse`: `refusals.malformed` (story:a-json-command-body-refuses-an-unknown-member), the
    // design's "an unknown key under `refuse`" (`docs/design/http-presentation-binding.md`).
    [
        "place-unknown-member",
        "y",
        "POST",
        "/places",
        "",
        "",
        r#"{"item_id":"i1","count":1,"x":1}"#,
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
    fn list(&self) -> Vec<items::ItemSnapshot> {
        self.items.clone()
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
            // The content type, or `name: value` headers joined by ` | ` (beyond10x/ess#681).
            headers: content_type
                .split(" | ")
                .filter(|header| !header.is_empty())
                .map(|header| match header.split_once(": ") {
                    Some((name, value)) => (name.to_ascii_lowercase(), value.to_owned()),
                    None => ("content-type".to_owned(), header.to_owned()),
                })
                .collect(),
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

/// What a [`Server`] is built into: the generated Rust workspace with a harness crate beside it,
/// run as a program; the generated Go module with an in-package test harness, run as its compiled
/// test binary; or a Go module that is only vetted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Rust,
    Go,
    GoVet,
}

impl Kind {
    /// The word `task http-binding-servers` reads from a server's `kind` file.
    pub fn name(self) -> &'static str {
        match self {
            Kind::Rust => "rust",
            Kind::Go => "go",
            Kind::GoVet => "go-vet",
        }
    }
}

/// The crate name of every Rust harness, so its binary is `target/debug/<this>`.
pub const RUST_HARNESS_CRATE: &str = "http-binding-harness";

/// One server a test sends requests to, as the sources it is built from.
///
/// A test never builds one: `task http-binding-servers` writes the sources of every server
/// `servers()` in `tests/http_binding_rust.rs` lists and builds each once, before the tests, into
/// `ESS_HTTP_BINDING_SERVERS`, under its [`Server::digest`]. A test synthesizes the same sources
/// again, which is cheap, and runs the program built from exactly those bytes, so a change to the
/// synthesis, a harness or a fixture is never answered by a stale build.
pub struct Server {
    pub label: String,
    pub kind: Kind,
    files: BTreeMap<String, String>,
}

impl Server {
    /// The generated Rust workspace under `shop/` and a harness crate under `harness/` whose
    /// `src/main.rs` is `main`.
    pub fn rust(label: &str, synthesis: &Synthesis, main: &str) -> Self {
        let mut files: BTreeMap<String, String> = synthesis
            .artifacts
            .iter()
            .map(|(relative, artifact)| (format!("shop/{relative}"), artifact.contents.clone()))
            .collect();
        let dependencies = ["shop-server", "shop-system", "shop-service", "shop-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../shop/crates/{name}\" }}");
                out
            });
        files.insert(
            "harness/Cargo.toml".to_owned(),
            format!(
                "[package]\nname = \"{RUST_HARNESS_CRATE}\"\nversion = \"0.0.0\"\n\
                 edition = \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        );
        files.insert("harness/src/main.rs".to_owned(), main.to_owned());
        Self {
            label: label.to_owned(),
            kind: Kind::Rust,
            files,
        }
    }

    /// The generated Go module with `harness` as `server/http_binding_test.go`.
    pub fn go(label: &str, synthesis: &Synthesis, harness: &str) -> Self {
        let mut server = Self::go_vet(label, synthesis);
        server.kind = Kind::Go;
        server
            .files
            .insert("server/http_binding_test.go".to_owned(), harness.to_owned());
        server
    }

    /// The generated Go module, which `go vet ./...` must find nothing in.
    pub fn go_vet(label: &str, synthesis: &Synthesis) -> Self {
        Self {
            label: label.to_owned(),
            kind: Kind::GoVet,
            files: synthesis
                .artifacts
                .iter()
                .map(|(relative, artifact)| (relative.clone(), artifact.contents.clone()))
                .collect(),
        }
    }

    /// The SHA-256 of the kind and every file, path and bytes, as lowercase hex: the name the
    /// built server goes under.
    pub fn digest(&self) -> String {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(self.kind.name().as_bytes());
        bytes.push(0);
        for (path, contents) in &self.files {
            bytes.extend_from_slice(path.as_bytes());
            bytes.push(0);
            bytes.extend_from_slice(&(contents.len() as u64).to_le_bytes());
            bytes.extend_from_slice(contents.as_bytes());
        }
        let digest = ess_http::digest(&bytes);
        digest.strip_prefix("sha256:").unwrap_or(&digest).to_owned()
    }

    /// Writes the sources under `root/<digest>/`: the tree under `src/`, and the files `kind`
    /// and `label` that `task http-binding-servers` reads. A server listed twice is written once.
    pub fn write_sources(&self, root: &Path) {
        let directory = root.join(self.digest());
        if directory.exists() {
            return;
        }
        for (relative, contents) in &self.files {
            let destination = directory.join("src").join(relative);
            std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
            std::fs::write(&destination, contents).expect("write");
        }
        std::fs::write(directory.join("kind"), self.kind.name()).expect("write");
        std::fs::write(directory.join("label"), &self.label).expect("write");
    }

    /// The program `task http-binding-servers` built from these sources, or a panic saying
    /// what to run.
    fn built(&self) -> PathBuf {
        let (label, kind) = (&self.label, self.kind.name());
        let Some(directory) = std::env::var_os("ESS_HTTP_BINDING_SERVERS") else {
            panic!(
                "ESS_HTTP_BINDING_SERVERS is not set: run `task http-binding-servers` and set it \
                 to the directory that task names, or run the tests through `task test`, which \
                 does both"
            )
        };
        let digest = self.digest();
        let built = Path::new(&directory).join(&digest);
        if built.is_file() {
            return built;
        }
        if let Ok(log) =
            std::fs::read_to_string(Path::new(&directory).join(format!("{digest}.failed")))
        {
            panic!("{label}: the {kind} server does not build:\n{log}");
        }
        panic!(
            "{label}: no {kind} server {digest} in ESS_HTTP_BINDING_SERVERS={}: run `task \
             http-binding-servers` again, since its sources changed; a server that task does not \
             build is missing from `servers()` in tests/http_binding_rust.rs",
            Path::new(&directory).display()
        )
    }

    /// Asserts that `go vet ./...` found nothing in a [`Kind::GoVet`] module.
    pub fn vetted(&self) {
        assert_eq!(
            self.kind,
            Kind::GoVet,
            "{}: only a vetted module",
            self.label
        );
        self.built();
    }

    /// What the server printed to `requests`, one tab-separated line each.
    pub fn run(&self, requests: &[[&str; 7]]) -> String {
        let binary = self.built();
        std::fs::create_dir_all(env!("CARGO_TARGET_TMPDIR")).expect("mkdir");
        let file = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "http-binding-{}-{}-{}.tsv",
            self.label,
            self.kind.name(),
            std::process::id()
        ));
        let lines: Vec<String> = requests.iter().map(|request| request.join("\t")).collect();
        std::fs::write(&file, lines.join("\n")).expect("write");
        let mut command = Command::new(&binary);
        match self.kind {
            Kind::Rust => {
                command.arg(&file);
            }
            Kind::Go => {
                command
                    .args([
                        "-test.count=1",
                        "-test.timeout=120s",
                        "-test.v",
                        "-test.run=TestHTTPBinding",
                    ])
                    .env("ESS_HTTP_REQUESTS", &file);
            }
            Kind::GoVet => panic!("{}: a vetted module is not run", self.label),
        }
        let ran: Output = command
            .output()
            .unwrap_or_else(|error| panic!("{} runs: {error}", binary.display()));
        let _ = std::fs::remove_file(&file);
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        assert!(
            ran.status.success(),
            "{}: the {} harness runs:\n{stdout}\n{}",
            self.label,
            self.kind.name(),
            String::from_utf8_lossy(&ran.stderr)
        );
        stdout
    }

    /// The server's answers to `requests`, by label.
    pub fn answers(&self, requests: &[[&str; 7]]) -> BTreeMap<String, Answer> {
        self.answers_in(&self.run(requests))
    }

    /// The answers in what [`Server::run`] printed, by label.
    pub fn answers_in(&self, stdout: &str) -> BTreeMap<String, Answer> {
        match self.kind {
            Kind::Go => parsed(
                stdout
                    .lines()
                    .filter_map(|line| line.strip_prefix("answer\t")),
            ),
            _ => parsed(stdout.lines().filter(|line| !line.starts_with("events\t"))),
        }
    }
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

/// The fixture's model and binding synthesized for Rust, driven by [`rust_events_harness`].
pub fn rust_server() -> Server {
    let ir = ir();
    let bound = binding(&ir, ROUTES);
    let synthesis = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
        .unwrap_or_else(|failure| panic!("the bound model synthesizes: {failure}"));
    Server::rust("fixture", &synthesis, &rust_events_harness())
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
        let server = rust_server();
        let stdout = server.run(REQUESTS);
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
        (server.answers_in(&stdout), events)
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
