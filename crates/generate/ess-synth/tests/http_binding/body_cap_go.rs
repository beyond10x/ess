//! The generated Go server reads at most the request body its binding declares for the matched
//! route, and answers a larger one with the refusal the binding declares, before it reads the body
//! (story:the-generated-go-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route).
//!
//! The model and the binding are the pair of `tests/fixtures/body-cap-rust/{model,http}.yaml.in`,
//! read as they are: nothing in them is Rust-specific, and one pair keeps the parity case honest.
//! `task body-cap-go-server` generates and builds the Go server once (`ESS_BODY_CAP_GO_SERVER`);
//! each socket case starts its own process of it. The parity case also starts the Rust server of
//! the same pair (`ESS_BODY_CAP_RUST_SERVER`). A module of `tests/http_binding_rust.rs`, spawning
//! no cargo.

use std::io::{BufRead as _, Read as _, Write as _};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::ess_http;
use ess_gen::http::Binding;
use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

use super::shared;

const MODEL: &str = include_str!("../fixtures/body-cap-rust/model.yaml.in");
const BINDING: &str = include_str!("../fixtures/body-cap-rust/http.yaml.in");

/// The digest of the shop fixture's bound Go synthesis, joined as `path`, a line feed, `contents`
/// in path order, captured on the tree before this change: a binding that declares no limit and no
/// `too_large` refusal writes the bytes it wrote before.
const SHOP_WORKSPACE_DIGEST: &str =
    "sha256:7f2e3e2f1d3cf5aeed0e7b39d258476a105c01ee4ea10c5a342e4f71d580b9b1";

fn fixture_ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("parcel.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("parcel.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The binding compiled against `ir`; it pins the model's source digest, so the server
/// `task body-cap-go-server` built was generated from this model and this binding.
fn fixture_binding(ir: &EssIr) -> Binding {
    assert!(
        BINDING.contains(&format!("  source_digest: sha256:{}\n", ir.source_digest())),
        "fixtures/body-cap-rust/http.yaml.in pins the model's source digest, sha256:{}; it is \
         the Rust server's fixture: run `task body-cap-go-server` again after it is updated there",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(BINDING).expect("well formed");
    let compiled = ess_http::compile(&spec, ir, &ess_gen::http::derived(ir))
        .unwrap_or_else(|refused| panic!("the binding compiles: {refused}"));
    Binding::new(compiled).unwrap_or_else(|refused| panic!("every route has a path: {refused}"))
}

fn fixture_synthesis() -> Synthesis {
    let ir = fixture_ir();
    synthesize_bound(
        &ir,
        Target::Go,
        OutputLayout::Workspace,
        Some(&fixture_binding(&ir)),
    )
    .expect("synthesizes")
}

fn shop() -> Synthesis {
    let ir = shared::ir();
    synthesize_bound(
        &ir,
        Target::Go,
        OutputLayout::Workspace,
        Some(&shared::binding(&ir, shared::ROUTES)),
    )
    .expect("synthesizes")
}

fn joined(synthesis: &Synthesis) -> String {
    let mut joined = String::new();
    for (path, artifact) in &synthesis.artifacts {
        joined.push_str(path);
        joined.push('\n');
        joined.push_str(&artifact.contents);
    }
    joined
}

/// The one artifact whose text holds `needle`.
fn artifact_holding<'a>(synthesis: &'a Synthesis, needle: &str) -> &'a str {
    let found: Vec<&str> = synthesis
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .filter(|contents| contents.contains(needle))
        .collect();
    assert_eq!(found.len(), 1, "one artifact holds `{needle}`");
    found[0]
}

/// A second served component, appended to the fixture's `components:` list, that the fixture
/// binding routes nothing of.
const SECOND_COMPONENT: &str = "  - component: desk
    owns: {domains: [parcel.desk]}
    accepts:
      commands:
        - parcel.desk.Note
    publishes: {events: [parcel.desk.Noted]}
    reached_by: network
";

/// The domain the second component owns: one command that reads a body.
const SECOND_DOMAIN: &str = "domain: parcel.desk
types:
  - {name: parcel.desk.NoteId, kind: newtype, of: Integer}
entities:
  - name: parcel.desk.Memo
    identity: {name: note_id, type: parcel.desk.NoteId}
    fields:
      - {name: text, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: parcel.desk.Noted
    fields:
      - {name: note_id, type: parcel.desk.NoteId}
commands:
  - name: parcel.desk.Note
    input:
      - {name: note_id, type: parcel.desk.NoteId}
      - {name: text, type: String}
    outcomes:
      - name: noted
        creates: parcel.desk.Memo
        instance: note_id
        sets: {text: input.text}
        emits: [parcel.desk.Noted]
        payload:
          parcel.desk.Noted: {note_id: input.note_id}
";

/// The fixture model with a second served component beside `parcel`, compiled from two documents.
fn two_component_ir() -> EssIr {
    let first = format!("{MODEL}{SECOND_COMPONENT}");
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in [
        ("parcel.yaml", first.as_str()),
        ("desk.yaml", SECOND_DOMAIN),
    ] {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        sources.insert(label, text);
        parsed.push((Source::new(label), raw));
    }
    let spec =
        Specification::assemble(parsed).unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The fixture binding, pinned to `ir`'s source digest: it bounds routes of `parcel` only.
fn binding_for(ir: &EssIr) -> Binding {
    let pinned = BINDING
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("source_digest: sha256:") {
                format!("  source_digest: sha256:{}", ir.source_digest())
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let spec = ess_http::HttpSpec::from_yaml(&pinned).expect("well formed");
    let compiled = ess_http::compile(&spec, ir, &ess_gen::http::derived(ir))
        .unwrap_or_else(|refused| panic!("the binding compiles: {refused}"));
    Binding::new(compiled).unwrap_or_else(|refused| panic!("every route has a path: {refused}"))
}

/// Two components the specification serves, and a binding that bounds a route of one of them: a
/// valid pair, which the Rust target synthesizes. The Go target builds one server package for
/// every served component and asserts that the components agree about capping a request
/// (`src/go/http.rs`, `helpers_file`), so the second component, whose routes the binding leaves
/// derived, makes the generator panic instead of writing the package.
#[test]
fn a_bound_route_beside_a_second_served_component_synthesizes_the_go_server() {
    let ir = two_component_ir();
    let binding = binding_for(&ir);
    assert_eq!(
        ir.components()
            .values()
            .filter(|component| component.reached_by == ess_domain::component::Reach::Network)
            .count(),
        2,
        "the model serves two components"
    );
    synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&binding))
        .expect("the Rust target synthesizes the pair");
    let go = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&binding))
    }));
    match go {
        Ok(result) => {
            result.expect("the Go target synthesizes the pair");
        }
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| {
                    payload
                        .downcast_ref::<&str>()
                        .map(|text| (*text).to_owned())
                })
                .unwrap_or_default();
            panic!("the Go target panicked on a valid model and binding: {message}");
        }
    }
}

/// The component the binding bounds nothing of, beside one it does, reads with the fixed 1 MiB
/// bound (`maxBody`) and the plain 413, through the package's one `readBody`; the capped component
/// reads through its route's `bodyLimit`.
#[test]
fn an_uncapped_component_beside_a_capped_one_reads_the_fixed_bound() {
    let ir = two_component_ir();
    let binding = binding_for(&ir);
    let go = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&binding))
        .expect("the Go target synthesizes the pair");
    let readers: Vec<&str> = go
        .artifacts
        .values()
        .map(|artifact| artifact.contents.as_str())
        .filter(|contents| contents.contains(":= readBody(request"))
        .collect();
    assert_eq!(readers.len(), 2, "one surface file per served component");
    let uncapped: Vec<&str> = readers
        .iter()
        .copied()
        .filter(|contents| !contents.contains("bodyLimit"))
        .collect();
    assert_eq!(uncapped.len(), 1, "one component reads the fixed bound");
    assert!(
        uncapped[0]
            .contains(":= readBody(request, bodyCap{limit: maxBody, tooLarge: bodyTooLarge})"),
        "the uncapped component passes the fixed bound"
    );
    let helpers = artifact_holding(&go, "func readBody(request *http.Request, reading bodyCap)");
    assert!(helpers.contains("const maxBody = "));
}

/// A binding that declares no limit and no `too_large` refusal writes the bytes it wrote before.
#[test]
fn a_binding_that_declares_no_limit_keeps_the_go_server_bytes() {
    let digest = ess_http::digest(joined(&shop()).as_bytes());
    assert_eq!(digest, SHOP_WORKSPACE_DIGEST);
}

/// The fixed bound the generated package reads is the one the binding crate exports.
#[test]
fn the_fixed_bound_is_the_one_the_binding_crate_exports() {
    let shop = shop();
    let helpers = artifact_holding(&shop, "const maxBody = ");
    assert!(
        helpers.contains(&format!(
            "const maxBody = {}\n",
            ess_http::FIXED_REQUEST_BYTES
        )),
        "the uncapped read bound"
    );
}

/// A surface whose binding bounds a request reads through its body limit; one whose binding does
/// not has none of it.
#[test]
fn a_capped_go_surface_reads_through_its_body_limit() {
    let capped = fixture_synthesis();
    let helpers = artifact_holding(&capped, "type bodyCap struct");
    assert!(helpers.contains("func bodyTooLarge("));
    assert!(helpers.contains("func readBody(request *http.Request, reading bodyCap)"));
    assert!(!helpers.contains("func readBody(request *http.Request) ("));
    assert!(helpers.contains("func refusalTooLarge("));
    let own = helpers
        .lines()
        .filter(|line| {
            line.starts_with("func refusalTooLarge") && !line.contains("refusalTooLarge(")
        })
        .count();
    assert_eq!(
        own, 1,
        "the route with its own too_large has a function of its own"
    );
    let surface = artifact_holding(&capped, "func bodyLimitParcel(");
    assert!(surface.contains("limit: 64") && surface.contains("limit: 256"));
    assert!(surface.contains("refusalTooLarge"));
    for text in shop().artifacts.values().map(|artifact| &artifact.contents) {
        for absent in ["bodyCap", "bodyLimit", "refusalTooLarge"] {
            assert!(
                !text.contains(absent),
                "the shop synthesis holds `{absent}`"
            );
        }
    }
    let shop = shop();
    let helpers = artifact_holding(&shop, "const maxBody = ");
    assert!(helpers.contains("func readBody(request *http.Request) ([]byte, *response) {"));
}

/// One row of the generated `bodyLimitParcel`: its path, its method and what it returns.
struct Row {
    path: String,
    method: String,
    result: String,
}

fn rows(surface: &str) -> Vec<Row> {
    let start = surface
        .find("func bodyLimitParcel(")
        .expect("the surface has a body limit");
    let function = &surface[start..];
    let function = &function[..function.find("\n}\n").expect("the function ends")];
    let quoted = |line: &str| {
        let open = line.find('"').expect("a quoted text") + 1;
        let close = open + line[open..].find('"').expect("closed");
        line[open..close].to_owned()
    };
    let mut rows = Vec::new();
    let mut path = String::new();
    let mut method = String::new();
    for line in function.lines() {
        if line.starts_with("\tif ") {
            path = quoted(line);
        } else if line.starts_with("\t\tif method ==") {
            method = quoted(line);
        } else if let Some(result) = line.strip_prefix("\t\t\treturn ") {
            rows.push(Row {
                path: path.clone(),
                method: method.clone(),
                result: result.to_owned(),
            });
        }
    }
    rows
}

/// The lookup as a table: a route's cap, the binding's cap where the route has none, the fixed
/// bound for a route that reads no body, a derived route or a document.
#[test]
fn the_cap_lookup_of_a_route_above_the_fixed_bound_is_that_route_cap() {
    let capped = fixture_synthesis();
    let rows = rows(artifact_holding(&capped, "func bodyLimitParcel("));
    let row = |method: &str, path: &str| {
        rows.iter()
            .find(|row| row.method == method && row.path == path)
            .unwrap_or_else(|| panic!("a row for `{method} {path}`"))
    };
    let fixed = "bodyCap{limit: maxBody, tooLarge: bodyTooLarge}";
    let bulk = row("POST", "/bulk");
    assert!(
        bulk.result
            .starts_with("bodyCap{limit: 2097152, tooLarge: refusalTooLarge")
            && 2_097_152 > ess_http::FIXED_REQUEST_BYTES,
        "{}",
        bulk.result
    );
    assert!(row("POST", "/parcels")
        .result
        .starts_with("bodyCap{limit: 64, tooLarge: refusalTooLarge"));
    assert!(row("POST", "/labels")
        .result
        .starts_with("bodyCap{limit: 256, tooLarge: refusalTooLarge"));
    assert_eq!(row("GET", "/parcels").result, fixed);
    assert_eq!(row("GET", "/openapi.json").result, fixed);
    assert_eq!(row("GET", "/docs").result, fixed);
    let ir = fixture_ir();
    let derived = ess_gen::http::derived(&ir);
    assert!(!derived.is_empty(), "the model has derived routes");
    let mut kept = 0;
    for derived in derived {
        for row in rows.iter().filter(|row| row.path == derived.path) {
            assert_eq!(row.result, fixed, "{} {}", row.method, row.path);
            kept += 1;
        }
    }
    assert!(kept >= 1, "a derived route stays beside the declared ones");
}

// ---- the socket cases -----------------------------------------------------------------------

struct Served(Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// The server binary `variable` names, made executable.
fn server(variable: &str, task: &str) -> PathBuf {
    let Some(path) = std::env::var_os(variable).map(PathBuf::from) else {
        panic!(
            "{variable} is not set: run `task {task}` and set it to the binary that task names, \
             or run the tests through `task test`, which does both"
        );
    };
    let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
        panic!(
            "{variable}={} names no server ({error}): run `task {task}`",
            path.display()
        )
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            permissions.set_mode(permissions.mode() | 0o755);
            std::fs::set_permissions(&path, permissions).expect("the server is made executable");
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    path
}

/// Starts `binary` on an ephemeral port of `127.0.0.1`; returns it and its address.
fn start(binary: PathBuf) -> (Served, String) {
    let mut child = Command::new(binary)
        .args(["--listen", "127.0.0.1:0"])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the server starts");
    let stdout = child.stdout.take().expect("piped");
    let served = Served(child);
    let mut lines = std::io::BufReader::new(stdout).lines();
    let address = loop {
        let line = lines
            .next()
            .expect("a ready record before the server stops")
            .expect("text");
        let Ok(record) = serde_json::from_str::<serde_json::Value>(&line) else {
            continue;
        };
        if let Some(address) = record["runtime"]["address"].as_str() {
            break address.to_owned();
        }
    };
    std::thread::spawn(move || lines.for_each(drop));
    (served, address)
}

fn serve() -> (Served, String) {
    start(server("ESS_BODY_CAP_GO_SERVER", "body-cap-go-server"))
}

struct Reply {
    status: u16,
    headers: Vec<(String, String)>,
    body: String,
}

impl Reply {
    fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    }
}

/// Writes `head` and `body` to a new connection and reads the answer by its `Content-Length`.
fn exchange(address: &str, head: &str, body: &[u8]) -> Reply {
    let mut stream = TcpStream::connect(address).expect("connects");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("timeout");
    stream.write_all(head.as_bytes()).expect("writes the head");
    // A server that refuses before it reads the body may close with the body unread.
    let _ = stream.write_all(body);
    let mut reader = std::io::BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).expect("reads the status line");
    let status = line
        .split(' ')
        .nth(1)
        .and_then(|status| status.parse().ok())
        .unwrap_or_else(|| panic!("a status line: {line:?}"));
    let mut headers = Vec::new();
    loop {
        line.clear();
        reader.read_line(&mut line).expect("reads a header");
        let text = line.trim_end();
        if text.is_empty() {
            break;
        }
        let (name, value) = text.split_once(':').expect("a header line");
        headers.push((name.to_owned(), value.trim().to_owned()));
    }
    let length: usize = headers
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case("content-length"))
        .map_or(0, |(_, value)| value.parse().expect("a length"));
    let mut bytes = vec![0_u8; length];
    reader.read_exact(&mut bytes).expect("reads the body");
    Reply {
        status,
        headers,
        body: String::from_utf8(bytes).expect("text"),
    }
}

/// Sends `method path` announcing `announced` bytes, with `extra` header lines, and writes `body`.
fn send_with(
    address: &str,
    method: &str,
    path: &str,
    announced: usize,
    extra: &str,
    body: &[u8],
) -> Reply {
    let head = format!(
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\n\
         Content-Length: {announced}\r\n{extra}\r\n"
    );
    exchange(address, &head, body)
}

fn send(address: &str, method: &str, path: &str, announced: usize, body: &[u8]) -> Reply {
    send_with(address, method, path, announced, "", body)
}

/// Sends a `POST` of unannounced length: `body` in chunks of `chunk` bytes and the last chunk.
fn send_chunked(address: &str, path: &str, body: &[u8], chunk: usize) -> Reply {
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: application/json\r\n\
         Transfer-Encoding: chunked\r\n\r\n"
    );
    let mut bytes = Vec::new();
    for piece in body.chunks(chunk) {
        bytes.extend_from_slice(format!("{:x}\r\n", piece.len()).as_bytes());
        bytes.extend_from_slice(piece);
        bytes.extend_from_slice(b"\r\n");
    }
    bytes.extend_from_slice(b"0\r\n\r\n");
    exchange(address, &head, &bytes)
}

static FRESH: AtomicU64 = AtomicU64::new(1000);

/// A `Receive`-shaped body of exactly `size` bytes, with a fresh `parcel_id`.
fn body_of(size: usize) -> Vec<u8> {
    let id = FRESH.fetch_add(1, Ordering::Relaxed);
    let empty = format!("{{\"parcel_id\":{id},\"note\":\"\"}}");
    assert!(size >= empty.len(), "{size} holds a command body");
    let body = format!(
        "{{\"parcel_id\":{id},\"note\":\"{}\"}}",
        "n".repeat(size - empty.len())
    );
    assert_eq!(body.len(), size);
    body.into_bytes()
}

fn post(address: &str, path: &str, size: usize) -> Reply {
    let body = body_of(size);
    send(address, "POST", path, body.len(), &body)
}

fn baseline(address: &str) -> u16 {
    let reply = post(address, "/parcels", 40);
    assert!(
        (200..300).contains(&reply.status),
        "the baseline command is accepted: {} {}",
        reply.status,
        reply.body
    );
    reply.status
}

#[test]
fn a_body_of_the_cap_is_read_and_one_byte_more_is_refused() {
    let (_served, address) = serve();
    let baseline = baseline(&address);
    let accepted = post(&address, "/parcels", 64);
    assert_eq!(accepted.status, baseline, "{}", accepted.body);
    assert_ne!(accepted.status, 422, "{}", accepted.body);
    let refused = post(&address, "/parcels", 65);
    assert_eq!(refused.status, 422, "{}", refused.body);
    assert_eq!(refused.body, r#"{"error":"too_big"}"#);
    assert_eq!(refused.header("X-Limit"), Some("64"));
}

/// The announced length is checked before one body byte is read: the answer to a request that
/// sends no body arrives before the socket read timeout expires.
#[test]
fn a_refusal_needs_no_body_read() {
    let (_served, address) = serve();
    let refused = send(&address, "POST", "/parcels", 65, b"");
    assert_eq!(refused.status, 422, "{}", refused.body);
    assert_eq!(refused.body, r#"{"error":"too_big"}"#);
}

#[test]
fn an_expect_continue_request_is_refused_without_a_continue() {
    let (_served, address) = serve();
    let refused = send_with(
        &address,
        "POST",
        "/parcels",
        65,
        "Expect: 100-continue\r\n",
        b"",
    );
    assert_eq!(
        refused.status, 422,
        "the first status line: {}",
        refused.body
    );
    assert_eq!(refused.body, r#"{"error":"too_big"}"#);
}

#[test]
fn a_route_limit_stands_in_place_of_the_binding_limit() {
    let (_served, address) = serve();
    let baseline = baseline(&address);
    assert_eq!(post(&address, "/labels", 256).status, baseline);
    assert_eq!(post(&address, "/labels", 65).status, baseline);
    let refused = post(&address, "/labels", 257);
    assert_eq!(refused.status, 413, "{}", refused.body);
    assert_eq!(refused.body, "");
    assert_eq!(refused.header("Content-Length"), Some("0"));
    assert_eq!(refused.header("Content-Type"), None);
}

#[test]
fn a_body_of_unannounced_length_is_cut_at_the_cap() {
    let (_served, address) = serve();
    let baseline = baseline(&address);
    let accepted = send_chunked(&address, "/parcels", &body_of(64), 32);
    assert_eq!(accepted.status, baseline, "{}", accepted.body);
    let refused = send_chunked(&address, "/parcels", &body_of(65), 33);
    assert_eq!(refused.status, 422, "{}", refused.body);
    assert_eq!(refused.header("X-Limit"), Some("64"));
}

const GO_DEFAULT: &str =
    r#"{"refused":"the body is longer than 1048576 bytes, which is all this surface reads"}"#;

#[test]
fn a_route_cap_above_the_fixed_bound_reads_a_body_the_default_refuses() {
    let (_served, address) = serve();
    let baseline = baseline(&address);
    assert!(1_200_000 > usize::try_from(ess_http::FIXED_REQUEST_BYTES).expect("fits"));
    let read = post(&address, "/bulk", 1_200_000);
    assert_eq!(read.status, baseline, "{}", read.body);
    let derived = derived_path(&address, "parcel.intake.Archive");
    let refused = post(&address, &derived, 1_200_000);
    assert_eq!(refused.status, 413, "{}", refused.body);
    assert_eq!(refused.body, GO_DEFAULT);
    let over = send(&address, "POST", "/bulk", 2_097_153, b"");
    assert_eq!(over.status, 422, "{}", over.body);
    assert_eq!(over.body, r#"{"error":"too_big"}"#);
}

#[test]
fn a_route_that_reads_no_body_and_a_derived_route_keep_the_fixed_bound() {
    let (_served, address) = serve();
    let view = send(&address, "GET", "/parcels", 65, &[b'x'; 65]);
    assert_eq!(view.status, 200, "{}", view.body);
    let derived = derived_path(&address, "parcel.intake.Archive");
    for path in [derived.as_str(), "/nowhere"] {
        let refused = send(&address, "POST", path, 1_048_577, b"");
        assert_eq!(refused.status, 413, "{path}: {}", refused.body);
        let document: serde_json::Value = serde_json::from_str(&refused.body).expect("JSON");
        assert_eq!(
            document["refused"],
            "the body is longer than 1048576 bytes, which is all this surface reads",
            "{path}"
        );
    }
}

/// The path of the derived operation `operation`, read from the server's own document.
fn derived_path(address: &str, operation: &str) -> String {
    let reply = send(address, "GET", "/openapi.json", 0, b"");
    assert_eq!(reply.status, 200);
    let document: serde_json::Value = serde_json::from_str(&reply.body).expect("JSON");
    document["paths"]
        .as_object()
        .expect("paths")
        .iter()
        .find(|(_, item)| {
            item.as_object().is_some_and(|methods| {
                methods
                    .values()
                    .any(|method| method["operationId"] == operation)
            })
        })
        .map_or_else(
            || panic!("a path for `{operation}`"),
            |(path, _)| path.clone(),
        )
}

/// One request, as both servers are sent it: method, path, announced length, body.
struct Request {
    label: &'static str,
    method: &'static str,
    path: String,
    announced: usize,
    body: Vec<u8>,
    declared: bool,
}

/// The Go and the Rust server of one binding answer the same status to the same request, and the
/// same body and `X-Limit` header where the binding declares the refusal.
#[test]
fn the_go_and_rust_servers_answer_a_capped_binding_alike() {
    let (_go, go_address) = serve();
    let (_rust, rust_address) = start(server("ESS_BODY_CAP_RUST_SERVER", "body-cap-rust-server"));
    let derived = derived_path(&go_address, "parcel.intake.Archive");
    assert_eq!(
        derived,
        derived_path(&rust_address, "parcel.intake.Archive")
    );
    let command = |label, path: &str, size, declared| {
        let body = body_of(size);
        Request {
            label,
            method: "POST",
            path: path.to_owned(),
            announced: body.len(),
            body,
            declared,
        }
    };
    let announced = |label, path: &str, announced, declared| Request {
        label,
        method: "POST",
        path: path.to_owned(),
        announced,
        body: Vec::new(),
        declared,
    };
    let requests = [
        command("Receive 64", "/parcels", 64, true),
        command("Receive 65", "/parcels", 65, true),
        command("Label 256", "/labels", 256, true),
        command("Label 257", "/labels", 257, true),
        announced("Receive 65 announced", "/parcels", 65, true),
        command("bulk 1200000", "/bulk", 1_200_000, true),
        announced("bulk 2097153 announced", "/bulk", 2_097_153, true),
        Request {
            label: "view with a body",
            method: "GET",
            path: "/parcels".to_owned(),
            announced: 65,
            body: vec![b'x'; 65],
            declared: false,
        },
        announced("Archive 1048577 announced", &derived, 1_048_577, false),
    ];
    for request in requests {
        let ask = |address: &str| {
            send(
                address,
                request.method,
                &request.path,
                request.announced,
                &request.body,
            )
        };
        let (go, rust) = (ask(&go_address), ask(&rust_address));
        assert_eq!(go.status, rust.status, "{}", request.label);
        if request.declared && go.status == 422 {
            assert_eq!(go.body, rust.body, "{}", request.label);
            assert_eq!(
                go.header("X-Limit"),
                rust.header("X-Limit"),
                "{}",
                request.label
            );
        }
        if request.declared && go.status == 413 {
            assert_eq!(go.body, rust.body, "{}", request.label);
        }
    }
}
