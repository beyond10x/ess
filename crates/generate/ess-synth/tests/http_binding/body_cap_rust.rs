//! The generated Rust server reads at most the request body its binding declares for the matched
//! route, and answers a larger one with the refusal the binding declares, before it reads the body
//! (story:the-generated-rust-server-reads-at-most-the-request-body-its-binding-declares-for-the-matched-route).
//!
//! The model and the binding are `tests/fixtures/body-cap-rust/{model,http}.yaml.in`: a binding
//! `limits` of 64 bytes with a `too_large` refusal of its own, a route with its own limit and
//! refusal, a route whose limit is above the fixed bound, a view, and a command left to its
//! derived route. `task body-cap-rust-server` generates and builds the server once; each socket
//! case starts its own process of it (`ESS_BODY_CAP_RUST_SERVER`). A module of
//! `tests/http_binding_rust.rs`, spawning no cargo.

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

/// The digests of the shop fixture's bound synthesis, joined as `path`, a line feed, `contents`
/// in path order, captured on the tree before this change: a binding that declares no limit and no
/// `too_large` refusal writes the bytes it wrote before. Re-recorded when the served document of
/// the shop's `form` route with `unknown: ignore` (`Order`) wrote its input with
/// `additionalProperties: true` (beyond10x/ess#752). The crate row was re-recorded when the crate
/// layout wrote its `wire` and `json` modules at the root, the server re-exporting them, and those
/// modules gained the entity snapshot codecs
/// (story `a-synthesized-rust-crate-carries-typed-decoders-and-encoders-without-a-server-…`).
const SHOP_WORKSPACE_DIGEST: &str =
    "sha256:e4e1c90021f86090e3fe91b39c238be4802a00e40cae55755ac8301e09b0930b";
const SHOP_CRATE_DIGEST: &str =
    "sha256:d07390df455dfb8f76658806a6d80f513388ce9e6282939dce6b59bb37adae24";

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

/// [`BINDING`] compiled against `ir`; it pins the model's source digest, so the server
/// `task body-cap-rust-server` built was generated from this model and this binding.
fn fixture_binding(ir: &EssIr) -> Binding {
    assert!(
        BINDING.contains(&format!("  source_digest: sha256:{}\n", ir.source_digest())),
        "fixtures/body-cap-rust/http.yaml.in pins the model's source digest, sha256:{}; write it \
         there and run `task body-cap-rust-server` again",
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
        Target::Rust,
        OutputLayout::Crate,
        Some(&fixture_binding(&ir)),
    )
    .expect("synthesizes")
}

fn shop(layout: OutputLayout) -> Synthesis {
    let ir = shared::ir();
    synthesize_bound(
        &ir,
        Target::Rust,
        layout,
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

fn artifact<'a>(synthesis: &'a Synthesis, suffix: &str) -> &'a str {
    let found: Vec<&str> = synthesis
        .artifacts
        .iter()
        .filter(|(path, _)| path.ends_with(suffix))
        .map(|(_, artifact)| artifact.contents.as_str())
        .collect();
    assert_eq!(found.len(), 1, "one artifact ends in `{suffix}`");
    found[0]
}

/// A binding that declares no limit and no `too_large` refusal writes the bytes it wrote before
/// this change, in both layouts.
#[test]
fn a_binding_that_declares_no_limit_keeps_the_rust_server_bytes() {
    for (layout, expected) in [
        (OutputLayout::Workspace, SHOP_WORKSPACE_DIGEST),
        (OutputLayout::Crate, SHOP_CRATE_DIGEST),
    ] {
        let digest = ess_http::digest(joined(&shop(layout)).as_bytes());
        assert_eq!(digest, expected, "{layout:?}");
    }
}

/// The fixed bound the generated `http` module reads is the one the binding crate exports.
#[test]
fn the_fixed_bound_is_the_one_the_binding_crate_exports() {
    let shop = shop(OutputLayout::Crate);
    let http = artifact(&shop, "src/server/http.rs");
    assert!(
        http.contains("pub const MAX_BODY: usize = 1_048_576;"),
        "the uncapped read bound"
    );
    assert_eq!(ess_http::FIXED_REQUEST_BYTES, 1_048_576);
}

/// A surface whose binding bounds a request reads through its body limit; one whose binding does
/// not has none of it.
#[test]
fn a_capped_surface_reads_through_its_body_limit() {
    let capped = fixture_synthesis();
    let http = artifact(&capped, "src/server/http.rs");
    assert!(
        http.contains("limit: &dyn Fn(&str, &str)"),
        "the capped http module"
    );
    assert!(
        http.contains("pub fn body_too_large"),
        "the capped http module"
    );
    let surface = artifact(&capped, "src/server/parcel.rs");
    assert!(
        surface.contains("fn body_limit("),
        "the surface holds the lookup and its refusal"
    );
    assert!(
        surface.contains("fn refusal_too_large"),
        "the surface holds the lookup and its refusal"
    );
    assert!(
        surface.contains("Some((64, refusal_too_large))")
            && surface.contains("Some((256, refusal_too_large_"),
        "the surface holds its body limits"
    );
    let shop = shop(OutputLayout::Crate);
    for text in shop.artifacts.values().map(|artifact| &artifact.contents) {
        for absent in ["body_limit", "body_too_large", "limit: &dyn"] {
            assert!(
                !text.contains(absent),
                "the shop synthesis holds `{absent}`"
            );
        }
    }
}

/// One row of the generated `body_limit`: its template, its method and what it returns.
struct Row {
    path: String,
    method: String,
    result: String,
}

fn rows(surface: &str) -> Vec<Row> {
    let start = surface
        .find("fn body_limit(")
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
        if line.starts_with("    if ") {
            path = quoted(line);
        } else if line.starts_with("        if method ==") {
            method = quoted(line);
        } else if let Some(result) = line.trim().strip_prefix("return ") {
            rows.push(Row {
                path: path.clone(),
                method: method.clone(),
                result: result.trim_end_matches(';').to_owned(),
            });
        }
    }
    rows
}

/// The lookup as a table: a route's cap, the binding's cap where the route has none, the fixed
/// bound's absence for a route that reads no body, a derived route or a document.
#[test]
fn the_cap_lookup_of_a_route_above_the_fixed_bound_is_that_route_cap() {
    let capped = fixture_synthesis();
    let rows = rows(artifact(&capped, "src/server/parcel.rs"));
    let row = |method: &str, path: &str| {
        rows.iter()
            .find(|row| row.method == method && row.path == path)
            .unwrap_or_else(|| panic!("a row for `{method} {path}`"))
    };
    let bulk = row("POST", "/bulk");
    assert!(
        bulk.result.starts_with("Some((2097152, refusal_too_large")
            && 2_097_152 > ess_http::FIXED_REQUEST_BYTES,
        "{}",
        bulk.result
    );
    assert!(row("POST", "/parcels")
        .result
        .starts_with("Some((64, refusal_too_large"));
    assert!(row("POST", "/labels")
        .result
        .starts_with("Some((256, refusal_too_large_"));
    // A `multipart` route is capped as every route reading a body (beyond10x/ess#754).
    assert!(row("POST", "/attachments")
        .result
        .starts_with("Some((256, refusal_too_large)"));
    assert_eq!(row("GET", "/parcels").result, "None");
    assert_eq!(row("GET", "/openapi.json").result, "None");
    assert_eq!(row("GET", "/docs").result, "None");
    let ir = fixture_ir();
    let derived = ess_gen::http::derived(&ir);
    assert!(!derived.is_empty(), "the model has derived routes");
    // A derived route the binding replaces has no row of its own (its declared route answers);
    // every one that stays has a row, and it bounds nothing.
    let mut kept = 0;
    for derived in derived {
        for row in rows.iter().filter(|row| row.path == derived.path) {
            assert_eq!(row.result, "None", "{} {}", row.method, row.path);
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

fn server() -> PathBuf {
    let Some(path) = std::env::var_os("ESS_BODY_CAP_RUST_SERVER").map(PathBuf::from) else {
        panic!(
            "ESS_BODY_CAP_RUST_SERVER is not set: run `task body-cap-rust-server` and set it to \
             the binary that task names, or run the tests through `task test`, which does both"
        );
    };
    let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
        panic!(
            "ESS_BODY_CAP_RUST_SERVER={} names no server ({error}): run `task \
             body-cap-rust-server`",
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

/// Starts the server on an ephemeral port of `127.0.0.1`; returns it and its address.
fn serve() -> (Served, String) {
    let mut child = Command::new(server())
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
        if record["event"] == "system.ready" {
            break record["runtime"]["address"]
                .as_str()
                .unwrap_or_else(|| panic!("the ready record names the address: {line}"))
                .to_owned();
        }
    };
    std::thread::spawn(move || lines.for_each(drop));
    (served, address)
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

/// Sends `method path` of `application/json` announcing `announced` bytes and writing `body`.
fn send(address: &str, method: &str, path: &str, announced: usize, body: &[u8]) -> Reply {
    send_typed(address, method, path, "application/json", announced, body)
}

/// Sends `method path` of `media` announcing `announced` bytes and writing `body`; reads the
/// answer by its `Content-Length`.
fn send_typed(
    address: &str,
    method: &str,
    path: &str,
    media: &str,
    announced: usize,
    body: &[u8],
) -> Reply {
    let mut stream = TcpStream::connect(address).expect("connects");
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .expect("timeout");
    let head = format!(
        "{method} {path} HTTP/1.1\r\nHost: {address}\r\nContent-Type: {media}\r\n\
         Content-Length: {announced}\r\n\r\n"
    );
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

/// The media type of [`multipart_of`]'s bodies.
const MULTIPART: &str = "multipart/form-data; boundary=B";

/// An `Attach` `multipart` body of exactly `size` bytes, with a fresh `parcel_id`
/// (beyond10x/ess#754).
fn multipart_of(size: usize) -> Vec<u8> {
    let id = FRESH.fetch_add(1, Ordering::Relaxed);
    let head = format!(
        "--B\r\nContent-Disposition: form-data; name=\"parcel_id\"\r\n\r\n{id}\r\n\
         --B\r\nContent-Disposition: form-data; name=\"note\"\r\n\r\n"
    );
    let tail = "\r\n--B--";
    assert!(size >= head.len() + tail.len(), "{size} holds a part");
    let body = format!("{head}{}{tail}", "n".repeat(size - head.len() - tail.len()));
    assert_eq!(body.len(), size);
    body.into_bytes()
}

/// A `multipart` body is read within its route's cap like every body, and one byte more is the
/// binding's `too_large`.
#[test]
fn a_multipart_body_of_the_cap_is_read_and_one_byte_more_is_refused() {
    let (_served, address) = serve();
    let baseline = baseline(&address);
    let body = multipart_of(256);
    let accepted = send_typed(&address, "POST", "/attachments", MULTIPART, 256, &body);
    assert_eq!(accepted.status, baseline, "{}", accepted.body);
    let body = multipart_of(257);
    let refused = send_typed(&address, "POST", "/attachments", MULTIPART, 257, &body);
    assert_eq!(refused.status, 422, "{}", refused.body);
    assert_eq!(refused.body, r#"{"error":"too_big"}"#);
    assert_eq!(refused.header("X-Limit"), Some("64"));
}

#[test]
fn a_refusal_needs_no_body_read() {
    let (_served, address) = serve();
    let refused = send(&address, "POST", "/parcels", 65, b"");
    assert_eq!(refused.status, 422, "{}", refused.body);
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
fn a_route_cap_above_the_fixed_bound_reads_a_body_the_default_refuses() {
    let (_served, address) = serve();
    let baseline = baseline(&address);
    assert!(1_200_000 > usize::try_from(ess_http::FIXED_REQUEST_BYTES).expect("fits"));
    let read = post(&address, "/bulk", 1_200_000);
    assert_eq!(read.status, baseline, "{}", read.body);
    let derived = derived_path(&address, "parcel.intake.Archive");
    let refused = post(&address, &derived, 1_200_000);
    assert_eq!(refused.status, 413, "{}", refused.body);
    assert_eq!(
        refused.body,
        r#"{"refused":"the body is 1200000 bytes and this surface reads at most 1048576"}"#
    );
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
        assert_eq!(
            refused.body,
            r#"{"refused":"the body is 1048577 bytes and this surface reads at most 1048576"}"#,
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
