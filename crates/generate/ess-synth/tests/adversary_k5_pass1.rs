//! Adversary pass 1 for `ess generate synthesize --target rust --http` (beyond10x/ess#493, stage 2,
//! unit k5; `docs/design/http-presentation-binding.md`, "Precedence" and "What the servers write
//! (k5, k6)").
//!
//! The model below is a second shop, served through a **server binding that refines a protocol
//! binding**: the protocol fixes `/token` (a `form` request answering an OAuth-shaped body) and
//! leaves `Purge` open; the server binding fills it in at `/items/current`, beside a `PUT` on
//! `/items/{item_id}`. One harness, built once per test binary, drives the generated `dispatch` and
//! then writes every answer through the generated `http::write` onto a real socket, so the bytes a
//! caller reads are asserted, not only the `Response` value.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::ess_http;
use ess_gen::http::Binding;
use ess_synth::{synthesize_bound, synthesize_laid_out, OutputLayout, Target};

const MODEL: &str = r"
format: ess/22
system: kshop
version: v1
domain: kshop.items
types:
  - {name: kshop.items.ItemId, kind: newtype, of: String}
entities:
  - name: kshop.items.Item
    identity: {name: item_id, type: kshop.items.ItemId}
    fields:
      - {name: title, type: String}
    lifecycle: {initial: Listed, states: [Listed], terminal: [Listed], transitions: []}
errors:
  - name: kshop.items.InvalidGrant
    naming: {wire: invalid_grant}
    fields:
      - {name: error_description, type: String}
  - name: kshop.items.Denied
    naming: {wire: access_denied}
events:
  - name: kshop.items.Put
    fields:
      - {name: item_id, type: kshop.items.ItemId}
      - {name: title, type: String}
  - name: kshop.items.Touched
    fields:
      - {name: item_id, type: kshop.items.ItemId}
  - name: kshop.items.Tokened
    fields:
      - {name: grant_type, type: String}
  - name: kshop.items.Called
    fields:
      - {name: client, type: String}
commands:
  - name: kshop.items.PutItem
    input:
      - {name: item_id, type: kshop.items.ItemId}
      - {name: title, type: String}
    outcomes:
      - name: put
        creates: kshop.items.Item
        instance: item_id
        sets: {title: input.title}
        emits: [kshop.items.Put]
        payload:
          kshop.items.Put: {item_id: input.item_id, title: input.title}
  - name: kshop.items.Purge
    input:
      - {name: item_id, type: kshop.items.ItemId}
    outcomes:
      - name: purged
        emits: [kshop.items.Touched]
        payload:
          kshop.items.Touched: {item_id: input.item_id}
  - name: kshop.items.Secret
    input:
      - {name: item_id, type: kshop.items.ItemId}
    outcomes:
      - name: told
        emits: [kshop.items.Touched]
        payload:
          kshop.items.Touched: {item_id: input.item_id}
  - name: kshop.items.Token
    input:
      - {name: grant_type, type: String}
      - {name: code, type: String}
      - {name: level, type: Integer}
      - {name: note, type: Optional<String>}
      - {name: scopes, type: List<String>}
    response:
      - {name: access_token, type: String}
    outcomes:
      - name: invalid
        when: level < 1
        error: kshop.items.InvalidGrant
      - name: issued
        returns: true
        emits: [kshop.items.Tokened]
        payload:
          kshop.items.Tokened: {grant_type: input.grant_type}
  - name: kshop.items.Callback
    input:
      - {name: return_to, type: String}
      - {name: state, type: Optional<String>}
      - {name: client, type: String}
      - {name: level, type: Integer}
    outcomes:
      - name: denied
        when: level < 1
        error: kshop.items.Denied
      - name: approved
        emits: [kshop.items.Called]
        payload:
          kshop.items.Called: {client: input.client}
actors:
  - name: kshop.items.Clerk
    may:
      - kshop.items.PutItem
      - kshop.items.Purge
      - kshop.items.Token
      - kshop.items.Callback
components:
  - component: kshop-service
    reached_by: network
    owns: {domains: [kshop.items]}
    accepts:
      commands:
        - kshop.items.PutItem
        - kshop.items.Purge
        - kshop.items.Secret
        - kshop.items.Token
        - kshop.items.Callback
    publishes:
      events: [kshop.items.Put, kshop.items.Touched, kshop.items.Tokened, kshop.items.Called]
";

/// The standard refusals both bindings declare; a refinement restates them exactly.
const REFUSALS: &str = r#"refusals:
  not_granted:
    status: 401
    body: {error: {literal: invalid_token}}
    headers:
      WWW-Authenticate:
        challenge:
          scheme: Bearer
          params:
            error: {literal: invalid_token}
            realm: {literal: 'a "q" \ b'}
"#;

/// The protocol binding: `/token` fixed, `Purge` open.
const PROTOCOL_ROUTES: &str = r"routes:
  - command: kshop.items.Token
    method: post
    path: /token
    request: {encoding: form, unknown: ignore}
    outcomes:
      issued:
        status: 200
        body: response
        headers:
          Cache-Control: {literal: no-store}
          X-Note: {source: input.note}
      invalid:
        status: 400
        body: {error: error}
        headers:
          Cache-Control: {literal: no-store}
  - command: kshop.items.Purge
    method: post
    outcomes:
      purged: {status: 204, body: none}
";

/// The server binding's own routes, beside the protocol's restated with `Purge`'s path filled in.
const SERVER_ROUTES: &str = r"routes:
  - command: kshop.items.Token
    method: post
    path: /token
    request: {encoding: form, unknown: ignore}
    outcomes:
      issued:
        status: 200
        body: response
        headers:
          Cache-Control: {literal: no-store}
          X-Note: {source: input.note}
      invalid:
        status: 400
        body: {error: error}
        headers:
          Cache-Control: {literal: no-store}
  - command: kshop.items.Purge
    method: post
    path: /items/current
    outcomes:
      purged: {status: 204, body: none}
  - command: kshop.items.PutItem
    method: put
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
  - command: kshop.items.Secret
    method: post
    path: /secret
  - command: kshop.items.Callback
    method: get
    path: /cb
    request: {encoding: query}
    outcomes:
      approved:
        status: 302
        body: none
        redirect:
          location: input.return_to
          mode: query
          parameters:
            client: input.client
            state: input.state
      denied:
        status: 302
        body: none
        redirect:
          location: input.return_to
          mode: fragment
          parameters:
            error: error.wire
            state: input.state
";

const FORM: &str = "application/x-www-form-urlencoded";

/// Every request: `label`, caller (`y` the granted `Clerk`, `n` no actor), method, path, query,
/// `Content-Type` (empty for none) and body.
const REQUESTS: &[[&str; 7]] = &[
    // Path segments.
    [
        "put-plus",
        "y",
        "PUT",
        "/items/a+b",
        "",
        "",
        r#"{"title":"T"}"#,
    ],
    [
        "put-slash",
        "y",
        "PUT",
        "/items/a%2Fb",
        "",
        "",
        r#"{"title":"T"}"#,
    ],
    [
        "put-reserved",
        "y",
        "PUT",
        "/items/%3F%23%26%3D",
        "",
        "",
        r#"{"title":"T"}"#,
    ],
    [
        "put-bad-utf8",
        "y",
        "PUT",
        "/items/%FF",
        "",
        "",
        r#"{"title":"T"}"#,
    ],
    [
        "put-current",
        "y",
        "PUT",
        "/items/current",
        "",
        "",
        r#"{"title":"T"}"#,
    ],
    // A literal on one method beside a variable on another.
    [
        "purge",
        "y",
        "POST",
        "/items/current",
        "",
        "",
        r#"{"item_id":"x"}"#,
    ],
    ["post-variable", "y", "POST", "/items/zzz", "", "", "{}"],
    // `form`, `ignore`.
    [
        "token-plus",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=a+b&code=c%2Bd&level=1",
    ],
    [
        "token-scope-one",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=g&code=c&level=1&scopes=x",
    ],
    [
        "token-scope-many",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "scopes=x&grant_type=g&scopes=y%20z&code=c&level=1&scopes=",
    ],
    [
        "token-note-absent",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=g&code=c&level=1",
    ],
    [
        "token-note",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=g&code=c&level=1&note=hi+there",
    ],
    [
        "token-note-crlf",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=g&code=c&level=1&note=a%0D%0ASet-Cookie:+x=1",
    ],
    [
        "token-note-del",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=g&code=c&level=1&note=a%7Fb",
    ],
    [
        "token-note-nul",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=g&code=c&level=1&note=a%00b",
    ],
    [
        "token-type-case",
        "y",
        "POST",
        "/token",
        "",
        "Application/X-WWW-Form-URLEncoded ; charset=UTF-8",
        "grant_type=g&code=c&level=1",
    ],
    [
        "token-type-text",
        "y",
        "POST",
        "/token",
        "",
        "text/plain",
        "grant_type=g&code=c&level=1",
    ],
    [
        "token-type-prefix",
        "y",
        "POST",
        "/token",
        "",
        "application/x-www-form-urlencoded-not",
        "grant_type=g&code=c&level=1",
    ],
    [
        "token-ignore-bad-value",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "utm=%ZZ&grant_type=g&code=c&level=1",
    ],
    [
        "token-ignore-bad-key",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "utm%ZZ=1&grant_type=g&code=c&level=1",
    ],
    [
        "token-scalar-twice",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=g&code=a&code=b&level=1",
    ],
    [
        "token-bad-utf8-value",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=g&code=%FF&level=1",
    ],
    [
        "token-invalid",
        "y",
        "POST",
        "/token",
        "",
        FORM,
        "grant_type=g&code=c&level=0",
    ],
    // Redirects.
    [
        "cb-query",
        "y",
        "GET",
        "/cb",
        "return_to=https%3A%2F%2Fc.example%2Fcb%3Fx%3D1&state=s+p&client=a%26b%2F%C3%A9%3D&level=1",
        "",
        "",
    ],
    [
        "cb-bare-question",
        "y",
        "GET",
        "/cb",
        "return_to=https%3A%2F%2Fc.example%2Fcb%3F&client=c&level=1",
        "",
        "",
    ],
    [
        "cb-fragment-keeps-query",
        "y",
        "GET",
        "/cb",
        "return_to=https%3A%2F%2Fc.example%2Fcb%3Fx%3D1&state=s&client=c&level=0",
        "",
        "",
    ],
    // Refusal override with a challenge; `not_granted` precedes `malformed`.
    [
        "secret",
        "y",
        "POST",
        "/secret",
        "",
        "",
        r#"{"item_id":"x"}"#,
    ],
    ["secret-malformed", "n", "POST", "/secret", "", "", "{"],
];

const HARNESS: &str = r#"use std::io::Read as _;

use kshop_server::http;
use kshop_server::kshop_service as surface;
use kshop_types::actor::{Actor, Caller};
use kshop_types::behaviour::{Generated, ItemStorage};
use kshop_types::items;
use kshop_types::items::obligations::TokenBehavior;
use kshop_types::obligation::UnmetObligation;

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

impl TokenBehavior for Ports {
    fn token(&mut self, input: items::Token) -> Result<items::TokenOutcome, UnmetObligation> {
        if input.level < 1 {
            return Ok(items::TokenOutcome::Invalid {
                error: items::InvalidGrant { error_description: format!("bad {}", input.grant_type) },
            });
        }
        Ok(items::TokenOutcome::Issued {
            response: items::TokenResponse {
                access_token: format!("{}|{}|{}", input.grant_type, input.code, input.scopes.join(",")),
            },
            tokened: items::Tokened { grant_type: input.grant_type },
        })
    }
}

fn clean(text: &str) -> String {
    text.replace('\t', "\u{1f}").replace('\n', "\u{1e}").replace('\r', "\u{1d}")
}

/// The bytes `http::write` puts on a socket for `answer`.
fn wire(answer: &http::Response) -> String {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("bind");
    let mut client = std::net::TcpStream::connect(listener.local_addr().expect("addr")).expect("connect");
    let (mut server, _) = listener.accept().expect("accept");
    http::write(&mut server, answer).expect("write");
    drop(server);
    let mut bytes = Vec::new();
    client.read_to_end(&mut bytes).expect("read");
    String::from_utf8_lossy(&bytes).into_owned()
}

fn main() {
    let requests = std::fs::read_to_string(std::env::args().nth(1).expect("the requests")).expect("read");
    let mut system = kshop_system::System::new(kshop_service::KshopService::new(Generated::new(Ports::default())));
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
            "{label}\t{}\t{}\t{}\t{}\t{}",
            answered.status,
            answered.content_type,
            clean(&headers.join(" | ")),
            clean(&answered.body),
            clean(&wire(&answered))
        );
    }
}
"#;

fn ir_of(model: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("kshop.yaml"),
        RawSpecFile::parse(model).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("kshop.yaml", model);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn ir() -> EssIr {
    ir_of(MODEL)
}

fn header(ir: &EssIr) -> String {
    format!(
        "type: ess-http/1\nspecification:\n  system: kshop\n  version: v1\n  source_digest: \
         sha256:{}\n",
        ir.source_digest()
    )
}

/// The protocol binding's exact bytes.
fn protocol(ir: &EssIr) -> String {
    format!("{}{REFUSALS}{PROTOCOL_ROUTES}", header(ir))
}

/// The server binding, refining [`protocol`] by its digest, compiled as `--http … --refines …`
/// compiles it.
fn binding(ir: &EssIr) -> Binding {
    let protocol = protocol(ir);
    let refined = ess_http::Refined::from_yaml(protocol.as_bytes()).expect("the protocol reads");
    let server = format!(
        "{}refines: {}\n{REFUSALS}{SERVER_ROUTES}",
        header(ir),
        ess_http::digest(protocol.as_bytes())
    );
    let spec = ess_http::HttpSpec::from_yaml(&server).expect("well formed");
    let compiled =
        ess_http::compile_refining(&spec, Some(&refined), ir, &ess_gen::http::derived(ir))
            .unwrap_or_else(|refused| panic!("the server binding compiles: {refused}"));
    Binding::new(compiled).unwrap_or_else(|refused| panic!("every route has a path: {refused}"))
}

fn write_tree(artifacts: &BTreeMap<String, ess_gen::Artifact>, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn cargo(directory: &Path, arguments: &[&str], deny: bool) -> Output {
    let mut command =
        Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"));
    command
        .args(arguments)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("RUSTC_WRAPPER")
        .env("CARGO_INCREMENTAL", "0");
    if deny {
        command.env("RUSTFLAGS", "-D warnings");
    } else {
        command.env_remove("RUSTFLAGS");
    }
    let output = command.output().expect("cargo runs");
    eprintln!(
        "cargo {arguments:?} in {}\n{}",
        directory.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

struct Answer {
    status: u16,
    content_type: String,
    headers: Vec<(String, String)>,
    body: String,
    /// What `http::write` put on the socket, head and body.
    wire: String,
}

impl Answer {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(named, _)| named == name)
            .map(|(_, value)| value.as_str())
    }

    fn json(&self) -> serde_json::Value {
        serde_json::from_str(&self.body)
            .unwrap_or_else(|error| panic!("the body is JSON ({error}): {}", self.body))
    }

    /// The header lines of the written head, without the status line.
    fn wire_headers(&self) -> Vec<String> {
        let head = self.wire.split("\r\n\r\n").next().unwrap_or_default();
        head.split("\r\n").skip(1).map(str::to_owned).collect()
    }
}

fn restored(text: &str) -> String {
    text.replace('\u{1f}', "\t")
        .replace('\u{1e}', "\n")
        .replace('\u{1d}', "\r")
}

fn answers() -> &'static BTreeMap<String, Answer> {
    static ANSWERS: std::sync::OnceLock<BTreeMap<String, Answer>> = std::sync::OnceLock::new();
    ANSWERS.get_or_init(|| {
        let ir = ir();
        let bound = binding(&ir);
        let synthesis = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("the bound model synthesizes: {failure}"));
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("adversary-k5-pass1-{}", std::process::id()));
        write_tree(&synthesis.artifacts, &root.join("kshop"));
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).expect("mkdir");
        let dependencies = [
            "kshop-server",
            "kshop-system",
            "kshop-service",
            "kshop-types",
        ]
        .iter()
        .fold(String::new(), |mut out, name| {
            let _ = writeln!(out, "{name} = {{ path = \"../kshop/crates/{name}\" }}");
            out
        });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"adversary-k5-harness\"\nversion = \"0.0.0\"\n\
                 edition = \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/main.rs"), HARNESS).expect("write");
        let requests = root.join("requests.tsv");
        std::fs::write(
            &requests,
            REQUESTS
                .iter()
                .map(|request| request.join("\t"))
                .collect::<Vec<_>>()
                .join("\n"),
        )
        .expect("write");
        let target = root.join("target");
        let ran = cargo(
            &harness,
            &[
                "run",
                "--offline",
                "--quiet",
                "--target-dir",
                target.to_str().expect("UTF-8"),
                "--",
                requests.to_str().expect("UTF-8"),
            ],
            false,
        );
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        let _ = std::fs::remove_dir_all(&root);
        assert!(ran.status.success(), "the harness builds and runs");
        eprintln!("{stdout}");
        stdout
            .lines()
            .map(|line| {
                let fields: Vec<&str> = line.split('\t').collect();
                let headers = if fields[3].is_empty() {
                    Vec::new()
                } else {
                    fields[3]
                        .split(" | ")
                        .map(|header| {
                            let (name, value) = header.split_once(": ").unwrap_or((header, ""));
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
                        wire: restored(fields[5]),
                    },
                )
            })
            .collect()
    })
}

fn served(label: &str) -> &'static Answer {
    answers()
        .get(label)
        .unwrap_or_else(|| panic!("the harness answered `{label}`"))
}

fn published(answer: &Answer) -> serde_json::Value {
    let body = answer.json();
    let events = body["published"]
        .as_array()
        .unwrap_or_else(|| panic!("an envelope lists what it published: {body}"));
    assert_eq!(events.len(), 1, "{body}");
    events[0]["payload"].clone()
}

/// A path variable is percent-decoded and nothing else: `+` is itself in a path, `%2F` is a `/`
/// inside one segment, reserved characters survive, and bytes that are not UTF-8 are malformed.
#[test]
fn adv_path_segments_decode_percent_only_and_refuse_invalid_utf8() {
    for (label, expected) in [
        ("put-plus", "a+b"),
        ("put-slash", "a/b"),
        ("put-reserved", "?#&="),
        ("put-current", "current"),
    ] {
        let answer = served(label);
        assert_eq!(answer.status, 202, "{label}: {}", answer.body);
        assert_eq!(
            published(answer),
            serde_json::json!({"item_id": expected, "title": "T"}),
            "{label}"
        );
    }
    assert_eq!(
        served("put-bad-utf8").status,
        400,
        "{}",
        served("put-bad-utf8").body
    );
}

/// `POST /items/current` (a literal) and `PUT /items/{item_id}` (a variable) do not shadow each
/// other: the literal answers its method, and a `POST` the literal does not match is `405`.
#[test]
fn adv_a_literal_on_one_method_beside_a_variable_on_another() {
    assert_eq!(served("purge").status, 204, "{}", served("purge").body);
    let other = served("post-variable");
    assert_eq!(other.status, 405, "{}", other.body);
    assert!(other.body.contains("PUT"), "{}", other.body);
}

/// `body: none` on `204`: no body, no `Content-Type`, and — RFC 9110 §8.6, "A server MUST NOT send
/// a Content-Length header field in any response with a status code of … 204 (No Content)" — no
/// `Content-Length` either, on the bytes `http::write` puts on the socket.
#[test]
fn adv_a_204_writes_no_body_no_content_type_and_no_content_length() {
    let purge = served("purge");
    assert_eq!(
        (
            purge.status,
            purge.body.as_str(),
            purge.content_type.as_str()
        ),
        (204, "", "")
    );
    let lines = purge.wire_headers();
    assert!(
        !lines
            .iter()
            .any(|line| line.to_ascii_lowercase().starts_with("content-type")),
        "{lines:?}"
    );
    assert!(
        !lines
            .iter()
            .any(|line| line.to_ascii_lowercase().starts_with("content-length")),
        "a 204 carries no Content-Length (RFC 9110 §8.6); written: {lines:?}"
    );
}

/// A `405` on a bound route names its methods in `Allow` (RFC 9110 §15.5.6: "The origin server
/// MUST generate an Allow header field in a 405 response"); the bound answer can now carry
/// headers.
#[test]
fn adv_a_405_on_a_bound_path_writes_allow() {
    let other = served("post-variable");
    assert_eq!(other.status, 405, "{}", other.body);
    assert!(
        other
            .wire_headers()
            .iter()
            .any(|line| line.eq_ignore_ascii_case("allow: put")),
        "written: {:?}",
        other.wire_headers()
    );
}

/// In a `form` body `+` is a space and `%2B` a plus.
#[test]
fn adv_form_plus_is_a_space_and_percent_2b_is_a_plus() {
    let answer = served("token-plus");
    assert_eq!(answer.status, 200, "{}", answer.body);
    assert_eq!(
        answer.json(),
        serde_json::json!({"access_token": "a b|c+d|"})
    );
}

/// A `List` field repeats its key: none, one and many values, an empty value as the empty text.
#[test]
fn adv_a_list_field_takes_zero_one_and_many_values() {
    for (label, expected) in [
        ("token-plus", "a b|c+d|"),
        ("token-scope-one", "g|c|x"),
        ("token-scope-many", "g|c|x,y z,"),
    ] {
        let answer = served(label);
        assert_eq!(answer.status, 200, "{label}: {}", answer.body);
        assert_eq!(answer.json()["access_token"], expected, "{label}");
    }
}

/// An absent `Optional` source omits its header — not written empty — and a present one is
/// written exactly once, on the wire too.
#[test]
fn adv_an_absent_optional_omits_its_header_and_a_present_one_is_written_once() {
    let absent = served("token-note-absent");
    assert_eq!(absent.status, 200, "{}", absent.body);
    assert_eq!(absent.header("X-Note"), None);
    assert!(
        !absent
            .wire_headers()
            .iter()
            .any(|line| line.starts_with("X-Note")),
        "{:?}",
        absent.wire_headers()
    );
    let present = served("token-note");
    assert_eq!(present.header("X-Note"), Some("hi there"));
    assert_eq!(
        present
            .wire_headers()
            .iter()
            .filter(|line| *line == "X-Note: hi there" || *line == "Cache-Control: no-store")
            .count(),
        2,
        "{:?}",
        present.wire_headers()
    );
}

/// CR LF, DEL and NUL from input never reach a header line: the effect stands and the answer is
/// `unfinished` with `committed: true` (no override on this binding).
#[test]
fn adv_a_control_character_in_a_sourced_header_is_never_written() {
    for label in ["token-note-crlf", "token-note-del", "token-note-nul"] {
        let answer = served(label);
        assert_eq!(answer.status, 501, "{label}: {}", answer.body);
        assert_eq!(answer.json()["committed"], true, "{label}");
        assert!(answer.headers.is_empty(), "{label}: {:?}", answer.headers);
        assert!(
            !answer.wire.contains("Set-Cookie"),
            "{label}: {}",
            answer.wire
        );
    }
}

/// The `form` media type is matched ignoring case and parameters; another type, or one it is only a
/// prefix of, is malformed.
#[test]
fn adv_the_form_media_type_is_matched_without_case_or_parameters() {
    assert_eq!(
        served("token-type-case").status,
        200,
        "{}",
        served("token-type-case").body
    );
    for label in ["token-type-text", "token-type-prefix"] {
        assert_eq!(served(label).status, 400, "{label}: {}", served(label).body);
    }
}

/// Under `unknown: ignore` a parameter no field names is ignored whatever it holds: a key that is
/// not percent-encoded UTF-8 names no field just as a value that is not does, so the two answer
/// alike (RFC 6749 §3.1, "MUST ignore unrecognized request parameters").
#[test]
fn adv_ignore_treats_an_undecodable_unknown_key_as_an_undecodable_unknown_value() {
    let value = served("token-ignore-bad-value");
    let key = served("token-ignore-bad-key");
    assert_eq!(value.status, 200, "{}", value.body);
    assert_eq!(
        (key.status, key.body.as_str()),
        (value.status, value.body.as_str()),
        "an unknown key with a bad escape is refused while an unknown value with one is ignored"
    );
}

/// A scalar key twice, and a value that is not UTF-8, are malformed.
#[test]
fn adv_a_scalar_twice_or_a_non_utf8_value_is_malformed() {
    for label in ["token-scalar-twice", "token-bad-utf8-value"] {
        assert_eq!(served(label).status, 400, "{label}: {}", served(label).body);
    }
}

/// `{error: error}` writes the error's wire name beside the error's own fields under their wire
/// names (decided for stage 2, answer 9), with the declared header.
#[test]
fn adv_an_error_body_carries_the_error_fields() {
    let answer = served("token-invalid");
    assert_eq!(answer.status, 400, "{}", answer.body);
    assert_eq!(
        answer.json(),
        serde_json::json!({"error": "invalid_grant", "error_description": "bad g"})
    );
    assert_eq!(answer.header("Cache-Control"), Some("no-store"));
}

/// `mode: query` keeps the location's query, appends after a bare `?` without a second separator,
/// and form-encodes every value; `mode: fragment` keeps the query and writes the fragment.
#[test]
fn adv_redirects_keep_the_query_and_encode_values() {
    let query = served("cb-query");
    assert_eq!((query.status, query.body.as_str()), (302, ""));
    assert_eq!(
        query.header("Location"),
        Some("https://c.example/cb?x=1&client=a%26b%2F%C3%A9%3D&state=s+p")
    );
    assert_eq!(
        served("cb-bare-question").header("Location"),
        Some("https://c.example/cb?client=c")
    );
    let fragment = served("cb-fragment-keeps-query");
    assert_eq!(fragment.status, 302, "{}", fragment.body);
    assert_eq!(
        fragment.header("Location"),
        Some("https://c.example/cb?x=1#error=access_denied&state=s")
    );
}

/// `refusals.not_granted` answers its status, literal body and challenge, each param a quoted
/// string with `"` and `\` escaped, and precedes `malformed` (Precedence 4 before 6).
#[test]
fn adv_a_refusal_override_writes_its_challenge_and_precedes_malformed() {
    for label in ["secret", "secret-malformed"] {
        let answer = served(label);
        assert_eq!(answer.status, 401, "{label}: {}", answer.body);
        assert_eq!(
            answer.json(),
            serde_json::json!({"error": "invalid_token"}),
            "{label}"
        );
        assert_eq!(
            answer.header("WWW-Authenticate"),
            Some(r#"Bearer error="invalid_token", realm="a \"q\" \\ b""#),
            "{label}"
        );
    }
}

/// The served document is the refined binding's: `Purge` at the path the server binding filled in,
/// marked as a bound route.
#[test]
fn adv_a_refining_binding_is_served_at_the_paths_it_fills_in() {
    let ir = ir();
    let synthesis = synthesize_bound(
        &ir,
        Target::Rust,
        OutputLayout::Workspace,
        Some(&binding(&ir)),
    )
    .expect("synthesizes");
    let document = synthesis
        .artifacts
        .iter()
        .find(|(path, _)| path.ends_with("kshop-service.openapi.json"))
        .map(|(_, artifact)| artifact.contents.clone())
        .expect("a served document");
    let document: serde_json::Value = serde_json::from_str(&document).expect("JSON");
    assert_eq!(
        document["paths"]["/items/current"]["post"]["x-ess-route-source"], "binding",
        "{}",
        document["paths"]
    );
}

/// What the server answers on a bound route is a response the document it serves declares for that
/// operation: every status, and on `204` no content. The document and the server read one binding.
#[test]
fn adv_every_status_served_on_a_bound_route_is_one_the_served_document_declares() {
    let ir = ir();
    let synthesis = synthesize_bound(
        &ir,
        Target::Rust,
        OutputLayout::Workspace,
        Some(&binding(&ir)),
    )
    .expect("synthesizes");
    let document: serde_json::Value = serde_json::from_str(
        &synthesis
            .artifacts
            .iter()
            .find(|(path, _)| path.ends_with("kshop-service.openapi.json"))
            .map(|(_, artifact)| artifact.contents.clone())
            .expect("a served document"),
    )
    .expect("JSON");
    let mut undeclared = Vec::new();
    for [label, _, method, path, ..] in REQUESTS {
        let answer = served(label);
        // `404` and `405` are facts about the surface, not about an operation, and so is the `400`
        // of a `malformed` this binding does not override: "today's `400`", which no document
        // declares. A `400` an outcome declares is still checked.
        let malformed = answer.status == 400 && answer.json().get("refused").is_some();
        if matches!(answer.status, 404 | 405) || malformed {
            continue;
        }
        let template = if path.starts_with("/items/") && *method == "PUT" {
            "/items/{item_id}"
        } else {
            path
        };
        let responses = &document["paths"][template][method.to_ascii_lowercase()]["responses"];
        let declared = &responses[answer.status.to_string()];
        if declared.is_null() {
            undeclared.push(format!("{label}: {method} {template} {}", answer.status));
        } else if answer.body.is_empty() && declared.get("content").is_some() {
            undeclared.push(format!(
                "{label}: {} has content in the document",
                answer.status
            ));
        }
    }
    assert!(
        undeclared.is_empty(),
        "served but not declared: {undeclared:#?}"
    );
}

/// Without a binding this model keeps its bytes in both layouts; with one, the generated workspace
/// and the single crate (`--layout crate`, `server` feature) build with warnings denied, as every
/// unbound layout is held to (`tests/single_crate_layout.rs`).
#[test]
fn adv_a_bound_server_builds_with_warnings_denied_in_both_layouts() {
    let ir = ir();
    for layout in [OutputLayout::Workspace, OutputLayout::Crate] {
        let unbound = synthesize_laid_out(&ir, Target::Rust, layout).expect("synthesizes");
        let none = synthesize_bound(&ir, Target::Rust, layout, None).expect("synthesizes");
        assert!(
            unbound
                .artifacts
                .iter()
                .zip(&none.artifacts)
                .all(|((a, x), (b, y))| a == b && x.contents == y.contents),
            "{layout:?}"
        );
    }
    let bound = binding(&ir);
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("adversary-k5-pass1-build-{}", std::process::id()));
    let target = root.join("target");
    let mut failed = Vec::new();
    for (layout, features) in [
        (OutputLayout::Workspace, None),
        (OutputLayout::Crate, Some("server")),
    ] {
        let synthesis = synthesize_bound(&ir, Target::Rust, layout, Some(&bound))
            .unwrap_or_else(|failure| panic!("{layout:?}: {failure}"));
        let tree = root.join(format!("{layout:?}"));
        write_tree(&synthesis.artifacts, &tree);
        let mut arguments = vec![
            "check",
            "--offline",
            "--all-targets",
            "--target-dir",
            target.to_str().expect("UTF-8"),
        ];
        if let Some(features) = features {
            arguments.extend(["--features", features]);
        }
        let checked = cargo(&tree, &arguments, true);
        if !checked.status.success() {
            failed.push(format!(
                "{layout:?}:\n{}",
                String::from_utf8_lossy(&checked.stderr)
            ));
        }
    }
    let _ = std::fs::remove_dir_all(&root);
    assert!(failed.is_empty(), "{}", failed.join("\n"));
}
