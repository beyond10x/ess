//! `ess generate synthesize --target go --http`: the generated Go server serves an `ess-http/1`
//! binding with the Rust server's behaviour (beyond10x/ess#493, stage 2,
//! `docs/design/http-presentation-binding.md`, "Precedence" and "What the servers write (k5, k6)").
//!
//! The model, the binding and the requests are those `tests/http_binding_rust.rs` sends the Rust
//! server (`tests/http_binding/shared.rs`). The Go harness is an in-package test of the generated
//! `server` package: it reads each request as `net/http` reads one off a connection
//! (`http.ReadRequest`), hands it to the surface's own dispatch, and prints the answer in the Rust
//! harness's format; `go test -timeout` bounds it, and it opens no socket. Its realization is the
//! Rust harness's: `Place` unmet for a count of `0` and refused past `5`, and `ItemById` over the
//! store the generated `PutItem` writes.
//!
//! A module of `tests/http_binding_rust.rs` rather than a test binary of its own, so the two
//! servers' cases link once.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use super::shared;

use ess_synth::{
    synthesize_bound, synthesize_laid_out, BindingRefusal, OutputLayout, SynthesisFailure, Target,
};
use shared::{binding, ir, published, Answer, REQUESTS, ROUTES};

/// The Go harness. A request line `net/http` cannot read is answered by `net/http` itself before
/// any handler runs, and is printed as the `400` it is, with the reason as its body.
const HARNESS: &str = r#"package server

import (
	"bufio"
	"fmt"
	"net/http"
	"os"
	"strings"
	"testing"

	"example.invalid/shop/components/shopservice"
	"example.invalid/shop/system"
	"example.invalid/shop/types/behaviour"
	"example.invalid/shop/types/items"
	"example.invalid/shop/types/obligation"
)

type ports struct{ rows []items.ItemSnapshot }

func (p *ports) Get(identity items.ItemId) (items.ItemSnapshot, bool) {
	for _, row := range p.rows {
		if row.Data.ItemId == identity {
			return row, true
		}
	}
	return items.ItemSnapshot{}, false
}

func (p *ports) Put(snapshot items.ItemSnapshot) {
	p.Delete(snapshot.Data.ItemId)
	p.rows = append(p.rows, snapshot)
}

func (p *ports) Delete(identity items.ItemId) {
	kept := p.rows[:0]
	for _, row := range p.rows {
		if row.Data.ItemId != identity {
			kept = append(kept, row)
		}
	}
	p.rows = kept
}

func (p *ports) Place(input items.Place) (items.PlaceOutcome, *obligation.UnmetObligation) {
	if input.Count == 0 {
		return nil, &obligation.UnmetObligation{Capability: "command-behaviour", Source: "shop.items.Place"}
	}
	if input.Count > 5 {
		return items.PlaceOutcomeRefused{}, nil
	}
	return items.PlaceOutcomePlaced{
		Response: items.PlaceResponse{Code: "C-" + input.ItemId.Value()},
		Touched:  items.Touched{ItemId: input.ItemId},
	}, nil
}

func (p *ports) ItemById(itemId items.ItemId) ([]items.ItemById, *obligation.UnmetObligation) {
	found := []items.ItemById{}
	for _, row := range p.rows {
		if row.Data.ItemId == itemId {
			found = append(found, items.ItemById{ItemId: row.Data.ItemId, Title: row.Data.Title})
		}
	}
	return found, nil
}

func clean(text string) string {
	return strings.NewReplacer("\t", "\x1f", "\n", "\x1e", "\r", "\x1d").Replace(text)
}

func TestHTTPBinding(t *testing.T) {
	requests, err := os.ReadFile(os.Getenv("ESS_HTTP_REQUESTS"))
	if err != nil {
		t.Fatal(err)
	}
	store := &ports{}
	s := system.NewSystem(shopservice.New(behaviour.New(behaviour.Ports{ItemStorage: store, Owed: store})))
	clerk := &Caller{Actor: ActorShopItemsClerk}
	for _, line := range strings.Split(string(requests), "\n") {
		fields := strings.Split(line, "\t")
		if len(fields) != 7 {
			t.Fatalf("a request line has seven fields: %q", line)
		}
		label, caller, method, path, query, contentType, body := fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6]
		target := path
		if query != "" {
			target += "?" + query
		}
		raw := fmt.Sprintf("%s %s HTTP/1.1\r\nHost: shop.test\r\nContent-Length: %d\r\n", method, target, len(body))
		if contentType != "" {
			raw += "Content-Type: " + contentType + "\r\n"
		}
		raw += "\r\n" + body
		request, err := http.ReadRequest(bufio.NewReader(strings.NewReader(raw)))
		if err != nil {
			fmt.Printf("answer\t%s\t400\t\t\t%s\n", label, clean("net/http: "+err.Error()))
			continue
		}
		var who *Caller
		if caller == "y" {
			who = clerk
		}
		answer := dispatchShopService(s, who, request)
		headers := make([]string, 0, len(answer.headers))
		for _, header := range answer.headers {
			headers = append(headers, header[0]+": "+header[1])
		}
		fmt.Printf("answer\t%s\t%d\t%s\t%s\t%s\n", label, answer.status, answer.contentType, clean(strings.Join(headers, " | ")), clean(answer.body))
	}
}
"#;

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("http-binding-go-{label}-{}", std::process::id()))
}

fn go(directory: &Path, requests: &Path, arguments: &[&str]) -> Output {
    let mut command = Command::new("go");
    command
        .args(arguments)
        .current_dir(directory)
        .env("ESS_HTTP_REQUESTS", requests)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .env("GOTOOLCHAIN", "local");
    // The caller's `GOCACHE` where it sets one (the tree's `target/go-build`), else one of this
    // run's own beside the generated module. That one is shared by every test of the process that
    // runs `go`, so no test removes it: one test's removal broke another's build mid-run.
    if std::env::var_os("GOCACHE").is_none() {
        command.env("GOCACHE", scratch("gocache"));
    }
    let output = command
        .output()
        .unwrap_or_else(|error| panic!("`go` runs: {error}"));
    eprintln!(
        "go {arguments:?} in {}\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// The Go server's answers by label, once per test binary.
fn go_answers() -> &'static BTreeMap<String, Answer> {
    static ANSWERS: std::sync::OnceLock<BTreeMap<String, Answer>> = std::sync::OnceLock::new();
    ANSWERS.get_or_init(|| {
        let ir = ir();
        let bound = binding(&ir, ROUTES);
        let synthesis = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("the bound model synthesizes for Go: {failure}"));
        let directory = scratch("module");
        shared::write(&synthesis, &directory);
        std::fs::write(directory.join("server/http_binding_test.go"), HARNESS).expect("write");
        let requests = scratch("requests.tsv");
        shared::write_requests(&requests);
        let ran = go(
            &directory,
            &requests,
            &[
                "test",
                "-count=1",
                "-timeout",
                "120s",
                "-v",
                "-run",
                "TestHTTPBinding",
                "./server/",
            ],
        );
        let _ = std::fs::remove_dir_all(&directory);
        let _ = std::fs::remove_file(&requests);
        assert!(ran.status.success(), "the Go harness builds and runs");
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        shared::parsed(
            stdout
                .lines()
                .filter_map(|line| line.strip_prefix("answer\t")),
        )
    })
}

fn served(label: &str) -> &'static Answer {
    go_answers()
        .get(label)
        .unwrap_or_else(|| panic!("the Go harness answered `{label}`"))
}

/// The bound server builds and `go vet` finds nothing in it, for the fixture's binding and for one
/// that binds every operation the surface serves, so no derived route is left beside the bound ones
/// and `refusals.not_granted` answers every grant check.
#[test]
fn the_bound_go_server_builds_and_is_vet_clean() {
    let ir = ir();
    let every = format!(
        "{ROUTES}  - command: shop.items.Archive\n    method: post\n    path: /archive/{{item_id}}\n    \
         parameters: {{item_id: input.item_id}}\n"
    );
    for (label, routes) in [("fixture", ROUTES.to_owned()), ("every", every)] {
        let synthesis = synthesize_bound(
            &ir,
            Target::Go,
            OutputLayout::Workspace,
            Some(&binding(&ir, &routes)),
        )
        .unwrap_or_else(|failure| panic!("{label}: synthesizes: {failure}"));
        let directory = scratch(&format!("vet-{label}"));
        shared::write(&synthesis, &directory);
        let requests = scratch(&format!("vet-{label}.tsv"));
        let vetted = go(&directory, &requests, &["vet", "./..."]);
        let _ = std::fs::remove_dir_all(&directory);
        assert!(
            vetted.status.success(),
            "{label}: `go vet` refuses the bound server:\n{}",
            String::from_utf8_lossy(&vetted.stderr)
        );
    }
}

/// One example directory under `examples/`, compiled where it lives, as `ess generate synthesize
/// --path` compiles it.
fn example(name: &str) -> ess_compiler::EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name);
    let mut labels = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                labels.push(
                    path.strip_prefix(&base)
                        .expect("inside the example")
                        .display()
                        .to_string(),
                );
            }
        }
    }
    labels.sort();
    let mut sources = ess_compiler::source::SourceMap::new();
    let mut parsed = Vec::new();
    for label in &labels {
        let text = std::fs::read_to_string(base.join(label)).expect("readable");
        let raw = ess_domain::spec::RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((ess_domain::system::Source::new(label.clone()), raw));
    }
    let specification = ess_domain::Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("`{name}` validates:\n{errors}"));
    ess_compiler::resolve::compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("`{name}` resolves:\n{diagnostics}"))
}

/// Every committed file of `generated/go/<name>` by its path in the tree, the ownership ledger
/// (`.ess-output`) aside.
fn committed_go(name: &str) -> BTreeMap<String, String> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../generated/go")
        .join(name);
    let mut files = BTreeMap::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the tree is readable") {
            let path = entry.expect("an entry").path();
            if path.file_name().is_some_and(|it| it == ".ess-output") {
                continue;
            }
            if path.is_dir() {
                pending.push(path);
            } else {
                let relative = path
                    .strip_prefix(&base)
                    .expect("inside the tree")
                    .display()
                    .to_string();
                files.insert(relative, std::fs::read_to_string(&path).expect("UTF-8"));
            }
        }
    }
    files
}

/// Without a binding every synthesized byte is the unbound synthesis's; with one, nothing outside
/// the server package moves, and the served document is the bound one.
///
/// "The unbound synthesis" is held twice: the committed `generated/go` trees, which `ess generate
/// synthesize` wrote without `--http` and whose `gatepass` holds a server package, are what the
/// bound entry point answers without a binding, file for file and byte for byte; and the fixture's
/// own synthesis, which no committed tree holds, is the laid-out entry point's.
#[test]
fn without_a_binding_the_go_server_keeps_its_bytes() {
    for name in ["billing", "gatepass"] {
        let synthesized =
            synthesize_bound(&example(name), Target::Go, OutputLayout::Workspace, None)
                .unwrap_or_else(|failure| panic!("`{name}` synthesizes: {failure}"));
        let synthesized: BTreeMap<String, String> = synthesized
            .artifacts
            .into_iter()
            .map(|(path, artifact)| (path, artifact.contents))
            .collect();
        let committed = committed_go(name);
        assert_eq!(
            synthesized.keys().collect::<Vec<_>>(),
            committed.keys().collect::<Vec<_>>(),
            "`generated/go/{name}`: the same files"
        );
        for (path, contents) in &committed {
            assert!(
                synthesized[path] == *contents,
                "`generated/go/{name}/{path}` is not what the bound entry point writes without a \
                 binding"
            );
        }
        if name == "gatepass" {
            assert!(
                committed.keys().any(|path| path.starts_with("server/")),
                "the committed tree holds a server"
            );
        }
    }
    let ir = ir();
    let unbound =
        synthesize_laid_out(&ir, Target::Go, OutputLayout::Workspace).expect("synthesizes");
    let none =
        synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, None).expect("synthesizes");
    assert_eq!(
        unbound
            .artifacts
            .iter()
            .map(|(path, artifact)| (path, &artifact.contents))
            .collect::<Vec<_>>(),
        none.artifacts
            .iter()
            .map(|(path, artifact)| (path, &artifact.contents))
            .collect::<Vec<_>>(),
    );
    assert!(
        !unbound
            .artifacts
            .values()
            .any(|artifact| artifact.contents.contains("matched(")),
        "an unbound server dispatches on exact paths"
    );
    let bound = synthesize_bound(
        &ir,
        Target::Go,
        OutputLayout::Workspace,
        Some(&binding(&ir, ROUTES)),
    )
    .expect("synthesizes");
    assert_eq!(
        unbound.artifacts.keys().collect::<Vec<_>>(),
        bound.artifacts.keys().collect::<Vec<_>>()
    );
    let moved: Vec<&String> = bound
        .artifacts
        .iter()
        .filter(|(path, artifact)| unbound.artifacts[*path].contents != artifact.contents)
        .map(|(path, _)| path)
        .collect();
    assert!(
        moved.iter().all(|path| path.starts_with("server/")),
        "only the server package moves with a binding: {moved:?}"
    );
    assert!(
        moved
            .iter()
            .any(|path| path.ends_with("shop-service.openapi.json")),
        "the served document is the bound one: {moved:?}"
    );
}

/// A `json` body is read by one reader in both servers ("Precedence", step 6): where
/// `encoding/json` and the Rust server's `json::parse` disagree on what a document is, the Go
/// server answers as the Rust one. A lone surrogate escape and nesting past 64 are
/// `refusals.malformed`; a raw control character in a string and a number with a leading zero
/// in a member no field declares are read, and the command runs.
#[test]
fn a_json_body_is_read_as_the_rust_server_reads_it() {
    let rust = shared::rust_answers();
    for (label, status) in [
        ("put-lone-surrogate", 400),
        ("put-nested-65", 400),
        ("put-raw-control", 202),
        ("put-leading-zero", 202),
    ] {
        let go = served(label);
        assert_eq!(
            (rust[label].status, go.status),
            (status, status),
            "{label}: rust {:?}\ngo   {go:?}",
            rust[label]
        );
        assert_eq!(go.body, rust[label].body, "{label}");
    }
}

/// `PUT`, `DELETE` and the view's `GET` share `/items/{item_id}`; the variable is percent-decoded
/// into the input, a path no template matches is `404` and a method none answers there is `405`.
#[test]
fn a_bound_command_is_dispatched_by_template_and_method() {
    let put = served("put");
    assert_eq!(put.status, 202, "{}", put.body);
    assert_eq!(
        published(put),
        serde_json::json!({"item_id": "a b", "title": "Lamp"})
    );
    let get = served("get");
    assert_eq!(get.status, 200, "{}", get.body);
    assert_eq!(
        get.json(),
        serde_json::json!({"item_id": "a b", "title": "Lamp"})
    );
    let missing = served("get-missing");
    assert_eq!(
        (
            missing.status,
            missing.body.as_str(),
            missing.content_type.as_str()
        ),
        (404, "", "")
    );
    let delete = served("delete");
    assert_eq!(delete.status, 202, "{}", delete.body);
    assert_eq!(published(delete), serde_json::json!({"item_id": "a b"}));
    let post = served("post-on-items");
    assert_eq!(post.status, 405, "{}", post.body);
    assert!(post.body.contains("DELETE, GET, PUT"), "{}", post.body);
    for label in ["nowhere", "empty-variable"] {
        assert_eq!(served(label).status, 404, "{label}: {}", served(label).body);
    }
    let named = served("put-body-names-path");
    assert_eq!(
        (named.status, named.json()),
        (400, serde_json::json!({"error": "invalid_request"}))
    );
    // `net/http` refuses a request line holding a bad percent-escape before any handler runs
    // ("Precedence", step 1); the Rust server reads that line and refuses the path variable.
    let escape = served("put-bad-escape");
    assert_eq!(escape.status, 400, "{}", escape.body);
    assert_eq!(served("docs").status, 200);
    let openapi: serde_json::Value = served("openapi").json();
    assert!(
        openapi["paths"]["/items/{item_id}"]["put"].is_object(),
        "the served document is the bound one: {}",
        openapi["paths"]
    );
}

/// A `form` request decodes into the input; `ignore` admits an unknown parameter, `refuse`
/// answers `malformed`, and so does a scalar key given twice, an empty integer, and a body of
/// another media type.
#[test]
fn a_form_request_decodes_and_an_unknown_parameter_follows_the_binding() {
    let ignored = served("form-ignore");
    assert_eq!(ignored.status, 202, "{}", ignored.body);
    assert_eq!(
        published(ignored),
        serde_json::json!({"item_id": "x 1", "count": 2})
    );
    let declared = served("form-ignore-unlisted");
    assert_eq!(declared.status, 422, "{}", declared.body);
    assert_eq!(declared.json()["outcome"], "out-of-stock");
    let strict = served("form-strict-ok");
    assert_eq!(strict.status, 202, "{}", strict.body);
    assert_eq!(
        published(strict),
        serde_json::json!({"item_id": "é", "count": 3})
    );
    for label in [
        "form-refuse",
        "form-twice",
        "form-empty-integer",
        "form-json-type",
        "form-no-type",
    ] {
        let answer = served(label);
        assert_eq!(
            (answer.status, answer.json()),
            (400, serde_json::json!({"error": "invalid_request"})),
            "{label}"
        );
        assert_eq!(answer.header("Cache-Control"), Some("no-store"), "{label}");
    }
}

/// A `query` command reads its input from the query string, a `List` repeating its key.
#[test]
fn a_query_command_reads_its_inputs_from_the_query_string() {
    let listed = served("query-list");
    assert_eq!(listed.status, 202, "{}", listed.body);
    assert_eq!(
        published(listed),
        serde_json::json!({"words": ["a", "b c"], "limit": 3})
    );
    let none = served("query-list-none");
    assert_eq!(none.status, 202, "{}", none.body);
    assert_eq!(
        published(none),
        serde_json::json!({"words": [], "limit": 1})
    );
    assert_eq!(served("query-scalar-twice").status, 400);
}

/// A listed outcome answers its declared status, body and headers; a header sourced from input
/// that is not a field value is never written, and the committed effect answers `unfinished`.
#[test]
fn a_declared_status_body_and_headers_are_written_per_outcome() {
    let placed = served("placed");
    assert_eq!(placed.status, 201, "{}", placed.body);
    assert_eq!(placed.json(), serde_json::json!({"code": "C-i1"}));
    assert_eq!(placed.header("Cache-Control"), Some("no-store"));
    assert_eq!(placed.header("X-Item"), Some("i1"));
    assert_eq!(placed.header("X-Code"), Some("C-i1"));
    let refused = served("refused");
    assert_eq!(refused.status, 409, "{}", refused.body);
    assert_eq!(refused.json(), serde_json::json!({"error": "out_of_stock"}));
    assert_eq!(refused.header("X-Error"), Some("out_of_stock"));
    let injected = served("placed-header-injection");
    assert_eq!(
        (
            injected.status,
            injected.body.as_str(),
            injected.headers.len()
        ),
        (503, "", 0),
        "a header value holding CR LF is answered as the unfinished refusal the binding declares"
    );
}

/// A `401` carries the declared challenge; each param is a quoted string whose `"` and `\` are
/// escaped, so neither the error's wire name nor an input value can leave it.
#[test]
fn a_bearer_challenge_quotes_the_error_wire_name() {
    let challenged = served("challenge");
    assert_eq!(challenged.status, 401, "{}", challenged.body);
    assert_eq!(
        challenged.header("WWW-Authenticate"),
        Some(r#"Bearer error="invalid \"token\" \\ here", hint="a\"b\\c", realm="shop""#)
    );
    assert_eq!(
        challenged.json(),
        serde_json::json!({"error": "invalid \"token\" \\ here"})
    );
    let read = served("challenge-read");
    assert_eq!(read.status, 202, "{}", read.body);
    assert_eq!(read.header("WWW-Authenticate"), None);
    let control = served("challenge-control");
    assert_eq!(
        (control.status, control.headers.len()),
        (503, 0),
        "a param a quoted string cannot carry is never written"
    );
}

/// `mode: query` appends its parameters to a query the location already has, form-encoded in name
/// order, and omits an absent optional. A location that is not an absolute URI without a fragment
/// is never written.
#[test]
fn a_query_redirect_keeps_an_existing_query_and_omits_an_absent_optional() {
    let query = served("redirect-query");
    assert_eq!(
        (
            query.status,
            query.body.as_str(),
            query.content_type.as_str()
        ),
        (302, "", "")
    );
    assert_eq!(
        query.header("Location"),
        Some("https://client.example/cb?x=1&client=c%26d&state=s+1")
    );
    assert_eq!(
        served("redirect-query-absent").header("Location"),
        Some("https://client.example/cb?client=c")
    );
    for label in ["redirect-relative", "redirect-with-fragment"] {
        let answer = served(label);
        assert_eq!(
            (answer.status, answer.headers.len()),
            (503, 0),
            "{label}: {}",
            answer.body
        );
    }
}

/// `mode: fragment` writes the parameters as the location's fragment, form-encoded in name order,
/// and omits an absent optional.
#[test]
fn a_fragment_redirect_writes_its_parameters_and_omits_an_absent_optional() {
    let fragment = served("redirect-fragment");
    assert_eq!(fragment.status, 303, "{}", fragment.body);
    assert_eq!(
        fragment.header("Location"),
        Some("https://client.example/cb#error=access_denied&state=s")
    );
    assert_eq!(
        served("redirect-fragment-absent").header("Location"),
        Some("https://client.example/cb#error=access_denied")
    );
}

/// A route whose template a request for `/openapi.json` or `/docs` would match is refused at
/// synthesis for Go too, naming it; a longer template is served, and the targets that serve no
/// binding still refuse one by name.
#[test]
fn a_route_claiming_openapi_json_or_docs_is_refused_at_synthesis() {
    let ir = ir();
    for path in ["/openapi.json", "/docs", "/{page}"] {
        let routes = format!(
            "routes:\n  - command: shop.items.Purge\n    method: post\n    path: '{path}'\n{}",
            if path == "/{page}" {
                "    parameters: {page: input.item_id}\n"
            } else {
                ""
            }
        );
        let bound = binding(&ir, &routes);
        let Err(SynthesisFailure::Binding(BindingRefusal::Unservable(refused))) =
            synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
        else {
            panic!("`{path}` is refused at synthesis");
        };
        let text = refused.to_string();
        assert!(
            text.contains("command `shop.items.Purge`")
                && (text.contains("`/openapi.json`") || text.contains("`/docs`")),
            "{path}: {text}"
        );
    }
    let longer = binding(
        &ir,
        "routes:\n  - command: shop.items.Purge\n    method: post\n    path: /docs/{item}\n    \
         parameters: {item: input.item_id}\n",
    );
    assert!(
        synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&longer)).is_ok(),
        "a template of two segments matches neither"
    );
    for target in [Target::Web, Target::Clap] {
        let refused = synthesize_bound(
            &ir,
            target,
            OutputLayout::Workspace,
            Some(&binding(&ir, ROUTES)),
        )
        .err()
        .map(|failure| failure.to_string())
        .unwrap_or_default();
        assert!(
            refused.contains(&format!("the `{}` target does not serve", target.name()))
                && refused.contains("`rust` and `go`"),
            "{refused}"
        );
    }
}

/// `refusals:` answers `not_granted`, `unfinished` and `malformed` as declared on the routes the
/// binding names; a route it does not name keeps today's answer.
#[test]
fn a_declared_refusal_override_answers_not_granted_unfinished_and_malformed() {
    for label in ["not-granted", "not-granted-anonymous"] {
        let answer = served(label);
        assert_eq!(
            (answer.status, answer.json()),
            (403, serde_json::json!({"error": "access_denied"})),
            "{label}"
        );
    }
    let derived = served("not-granted-derived");
    assert_eq!(derived.status, 403, "{}", derived.body);
    assert_eq!(
        derived.json(),
        serde_json::json!({"refused": "not granted", "actor": "shop.items.Clerk"})
    );
    let unfinished = served("unfinished");
    assert_eq!(
        (
            unfinished.status,
            unfinished.body.as_str(),
            unfinished.content_type.as_str()
        ),
        (503, "", "")
    );
    let malformed = served("malformed-json");
    assert_eq!(
        (malformed.status, malformed.json()),
        (400, serde_json::json!({"error": "invalid_request"}))
    );
    assert_eq!(malformed.header("Cache-Control"), Some("no-store"));
}

/// Whether `body` is an answer the binding does not write: the envelope of an outcome it does not
/// list, a derived refusal, a view's `rows`. Each server renders those as it did before any
/// binding, and the two order an object's members differently there (the Rust envelope in
/// declaration order, Go's `encoding/json` by name); the binding's own answers are byte for byte.
fn unbound_rendering(body: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(body).is_ok_and(|value| {
        ["outcome", "refused", "rows"]
            .iter()
            .any(|member| value.get(member).is_some())
    })
}

/// One binding, the same requests: both servers answer each with the same status, content type,
/// headers and body bytes. A body the binding does not write is compared as JSON
/// ([`unbound_rendering`]); `put-bad-escape` is the one request `net/http` refuses before the Go
/// surface sees it ("Precedence", step 1), and both answer it `400`.
#[test]
fn the_go_and_rust_servers_answer_one_binding_alike() {
    let rust = shared::rust_answers();
    let go = go_answers();
    let mut differing = String::new();
    let mut compared = 0;
    for request in REQUESTS {
        let label = request[0];
        let (Some(rust), Some(go)) = (rust.get(label), go.get(label)) else {
            let _ = writeln!(differing, "{label}: not answered by both servers");
            continue;
        };
        compared += 1;
        if label == "put-bad-escape" {
            if (rust.status, go.status) != (400, 400) {
                let _ = writeln!(differing, "{label}: {} and {}", rust.status, go.status);
            }
            continue;
        }
        let bodies = rust.body == go.body
            || (unbound_rendering(&rust.body)
                && serde_json::from_str::<serde_json::Value>(&rust.body).ok()
                    == serde_json::from_str::<serde_json::Value>(&go.body).ok());
        if rust.status != go.status
            || rust.content_type != go.content_type
            || rust.headers != go.headers
            || !bodies
        {
            let _ = writeln!(differing, "{label}:\n  rust {rust:?}\n  go   {go:?}");
        }
    }
    assert_eq!(compared, REQUESTS.len(), "{differing}");
    assert!(differing.is_empty(), "the servers disagree:\n{differing}");
}

/// Adversary pass 1 for the Go server (unit k6, beyond10x/ess#493). Each case states what the
/// design page says the Go server answers ("What the servers write (k5, k6)": the Rust server's
/// status, headers and body bytes wherever the binding writes the answer) and holds the Go server
/// to it, against the Rust server where both are asked.
mod adversary_k6_pass1 {
    use std::collections::BTreeMap;
    use std::fmt::Write as _;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
    use ess_domain::{spec::RawSpecFile, system::Source, Specification};
    use ess_gen::ess_http;
    use ess_gen::http::Binding;
    use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};

    use super::shared::{self, Answer};

    /// The view's fields as the fixture declares them, and declared `title` first: an order that
    /// is not the alphabetical one.
    const VIEW_FIELDS: (&str, &str) = (
        "    fields:\n      - {name: item_id, type: shop.items.ItemId}\n      - {name: title, type: \
         String}\nactors:",
        "    fields:\n      - {name: title, type: String}\n      - {name: item_id, type: \
         shop.items.ItemId}\nactors:",
    );

    /// Every request both servers are sent, in the format of [`shared::REQUESTS`].
    const REQUESTS: &[[&str; 7]] = &[
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
        [
            "placed-u2028",
            "y",
            "POST",
            "/places",
            "",
            "",
            r#"{"item_id":"i ","count":1}"#,
        ],
        ["nowhere-escaped", "y", "GET", "/no%20where", "", "", ""],
        ["openapi-escaped", "y", "GET", "/openapi%2Ejson", "", "", ""],
        ["head-on-get", "y", "HEAD", "/items/a%20b", "", "", ""],
        ["get-with-body", "y", "GET", "/search", "limit=1", "", "x"],
        [
            "form-ignore-bad-key",
            "y",
            "POST",
            "/orders",
            "",
            "application/x-www-form-urlencoded",
            "item_id=x&count=2&%zz=1",
        ],
        [
            "form-refuse-bad-key",
            "y",
            "POST",
            "/strict-orders",
            "",
            "application/x-www-form-urlencoded",
            "item_id=x&count=2&%zz=1",
        ],
        [
            "form-leading-zero",
            "y",
            "POST",
            "/strict-orders",
            "",
            "application/x-www-form-urlencoded",
            "item_id=x&count=007",
        ],
        [
            "view-query-names-path",
            "y",
            "GET",
            "/items/a%20b",
            "item_id=zz",
            "",
            "",
        ],
        [
            "query-unknown",
            "y",
            "GET",
            "/search",
            "limit=1&x=1",
            "",
            "",
        ],
        [
            "redirect-unicode-param",
            "y",
            "GET",
            "/approve",
            "return_to=https%3A%2F%2Fc.example%2Fcb&client=%C3%A9%E2%80%A8&level=2",
            "",
            "",
        ],
        [
            "lowercase-escape-path",
            "y",
            "GET",
            "/items/%c3%a9",
            "",
            "",
            "",
        ],
        [
            "put-trailing",
            "y",
            "PUT",
            "/items/t1",
            "",
            "",
            r#"{"title":"Lamp"} x"#,
        ],
        [
            "put-repeated-member",
            "y",
            "PUT",
            "/items/t2",
            "",
            "",
            r#"{"title":"A","title":"B"}"#,
        ],
    ];

    pub(super) fn ir_of(model: &str) -> EssIr {
        let spec = Specification::assemble([(
            Source::new("shop.yaml"),
            RawSpecFile::parse(model).expect("well formed"),
        )])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
        let mut sources = SourceMap::new();
        sources.insert("shop.yaml", model);
        compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
    }

    /// The fixture model with its view's fields declared `title` first.
    fn reordered() -> EssIr {
        assert!(
            shared::MODEL.contains(VIEW_FIELDS.0),
            "the fixture's view moved"
        );
        ir_of(&shared::MODEL.replace(VIEW_FIELDS.0, VIEW_FIELDS.1))
    }

    fn scratch(label: &str) -> PathBuf {
        super::scratch(&format!("k6adv-{label}"))
    }

    fn write_requests(path: &Path, requests: &[[&str; 7]]) {
        let lines: Vec<String> = requests.iter().map(|request| request.join("\t")).collect();
        std::fs::write(path, lines.join("\n")).expect("write");
    }

    /// The Go server's answers to `requests`, through the module's own harness.
    pub(super) fn go_run(
        label: &str,
        synthesis: &Synthesis,
        requests: &[[&str; 7]],
    ) -> BTreeMap<String, Answer> {
        let directory = scratch(&format!("{label}-module"));
        shared::write(synthesis, &directory);
        std::fs::write(
            directory.join("server/http_binding_test.go"),
            super::HARNESS,
        )
        .expect("write");
        let file = scratch(&format!("{label}.tsv"));
        write_requests(&file, requests);
        let ran = super::go(
            &directory,
            &file,
            &[
                "test",
                "-count=1",
                "-timeout",
                "120s",
                "-v",
                "-run",
                "TestHTTPBinding",
                "./server/",
            ],
        );
        let _ = std::fs::remove_dir_all(&directory);
        let _ = std::fs::remove_file(&file);
        assert!(
            ran.status.success(),
            "{label}: the Go harness builds and runs"
        );
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        shared::parsed(
            stdout
                .lines()
                .filter_map(|line| line.strip_prefix("answer\t")),
        )
    }

    /// The Rust server's answers to `requests`, through [`shared::RUST_HARNESS`].
    pub(super) fn rust_run(
        label: &str,
        synthesis: &Synthesis,
        requests: &[[&str; 7]],
    ) -> BTreeMap<String, Answer> {
        rust_run_with(label, synthesis, requests, shared::RUST_HARNESS)
    }

    /// The Rust server's answers to `requests`, through `main` as the harness's `src/main.rs`.
    pub(super) fn rust_run_with(
        label: &str,
        synthesis: &Synthesis,
        requests: &[[&str; 7]],
        main: &str,
    ) -> BTreeMap<String, Answer> {
        let root = scratch(&format!("{label}-rust"));
        shared::write(synthesis, &root.join("shop"));
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
                "[package]\nname = \"k6adv-rust-harness\"\nversion = \"0.0.0\"\n\
                 edition = \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/main.rs"), main).expect("write");
        let file = root.join("requests.tsv");
        write_requests(&file, requests);
        let target = root.join("target");
        let ran = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
            .args(["run", "--offline", "--quiet", "--target-dir"])
            .arg(&target)
            .arg("--")
            .arg(&file)
            .current_dir(&harness)
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env("CARGO_INCREMENTAL", "0")
            .output()
            .expect("cargo runs");
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            ran.status.success(),
            "{label}: the Rust harness builds and runs:\n{}",
            String::from_utf8_lossy(&ran.stderr)
        );
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        shared::parsed(stdout.lines())
    }

    /// Both servers' answers to [`REQUESTS`] for the reordered model and the fixture's binding:
    /// `(rust, go)`, once per test binary.
    fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
        static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
            std::sync::OnceLock::new();
        BOTH.get_or_init(|| {
            let ir = reordered();
            let bound = shared::binding(&ir, shared::ROUTES);
            let rust = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for Rust: {failure}"));
            let go = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for Go: {failure}"));
            (
                rust_run("both", &rust, REQUESTS),
                go_run("both", &go, REQUESTS),
            )
        })
    }

    fn answered(label: &str) -> (&'static Answer, &'static Answer) {
        let (rust, go) = both();
        (
            rust.get(label)
                .unwrap_or_else(|| panic!("the Rust server answered `{label}`")),
            go.get(label)
                .unwrap_or_else(|| panic!("the Go server answered `{label}`")),
        )
    }

    /// `body: single` is an answer the binding writes ("the one row, bare"), so the design page
    /// holds both servers to one byte sequence for it. The Rust server writes the row's members in
    /// the order the view declares them; the Go server encodes the row as a `map[string]any`
    /// (`record_encoder`) and `bare` writes it with `encoding/json`, which sorts the keys.
    #[test]
    fn adv_a_single_view_row_is_the_same_bytes_from_both_servers() {
        let (rust_put, go_put) = answered("put");
        assert_eq!((rust_put.status, go_put.status), (202, 202));
        let (rust, go) = answered("get");
        assert_eq!(rust.status, 200, "{}", rust.body);
        assert_eq!(
            rust.body, r#"{"title":"Lamp","item_id":"a b"}"#,
            "the Rust server writes the row in declaration order"
        );
        assert_eq!(
            go.body, rust.body,
            "a `single` row the binding writes is the same bytes from the Go server"
        );
    }

    /// `body: response` is an answer the binding writes. A response text holding U+2028 is written
    /// as itself by the Rust server (`json::push_text` escapes only `"`, `\` and controls), and as
    /// ` ` by Go's `encoding/json`, which escapes U+2028 and U+2029 even with HTML escaping
    /// off.
    #[test]
    fn adv_a_response_body_holding_u2028_is_the_same_bytes_from_both_servers() {
        let (rust, go) = answered("placed-u2028");
        assert_eq!(
            (rust.status, go.status),
            (201, 201),
            "{} / {}",
            rust.body,
            go.body
        );
        assert_eq!(rust.body, "{\"code\":\"C-i\u{2028}\"}");
        assert_eq!(
            go.body, rust.body,
            "a `response` body the binding writes is the same bytes from the Go server"
        );
    }

    /// A path no route matches, spelled with a percent-escape: the `404` is a derived refusal, so
    /// the comparison test reads it as JSON. The Rust server names the path as the request spelled
    /// it; the Go server names `request.URL.Path`, already decoded.
    #[test]
    fn adv_a_percent_encoded_path_no_route_matches_is_refused_alike() {
        let (rust, go) = answered("nowhere-escaped");
        assert_eq!((rust.status, go.status), (404, 404));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&go.body).ok(),
            serde_json::from_str::<serde_json::Value>(&rust.body).ok(),
            "rust {}\ngo   {}",
            rust.body,
            go.body
        );
    }

    /// A document path spelled with a percent-escape (`/openapi%2Ejson`): the Rust server matches
    /// the two documents and every derived route on the path as the request spelled it, the Go
    /// server on the decoded `request.URL.Path`.
    #[test]
    fn adv_a_percent_encoded_document_path_is_answered_alike() {
        let (rust, go) = answered("openapi-escaped");
        assert_eq!(
            go.status, rust.status,
            "rust {} {}\ngo   {} {}",
            rust.status, rust.body, go.status, go.body
        );
    }

    /// What `net/http` reads before the surface does, and the request shapes the k5 pass found on
    /// the Rust server: `HEAD` on a `GET` route, a body on a `GET`, an undecodable key under
    /// `ignore` and `refuse`, a leading zero, a query key the path carries on a view, an unknown
    /// query key, a non-ASCII redirect parameter, a lower-case escape in a path.
    #[test]
    fn adv_the_go_server_answers_what_http_reads_first_as_the_rust_server_does() {
        let mut differing = String::new();
        for label in [
            "head-on-get",
            "get-with-body",
            "form-ignore-bad-key",
            "form-refuse-bad-key",
            "form-leading-zero",
            "view-query-names-path",
            "query-unknown",
            "redirect-unicode-param",
            "lowercase-escape-path",
        ] {
            let (rust, go) = answered(label);
            // The view row's member order is the single-row case's finding, not this one's.
            let bodies = rust.body == go.body
                || ((super::unbound_rendering(&rust.body) || label == "view-query-names-path")
                    && serde_json::from_str::<serde_json::Value>(&rust.body).ok()
                        == serde_json::from_str::<serde_json::Value>(&go.body).ok());
            if rust.status != go.status
                || rust.content_type != go.content_type
                || rust.headers != go.headers
                || !bodies
            {
                let _ = writeln!(differing, "{label}:\n  rust {rust:?}\n  go   {go:?}");
            }
        }
        assert!(differing.is_empty(), "the servers disagree:\n{differing}");
    }

    /// A `json` body is one JSON document ("Precedence", step 6: input that cannot be read per
    /// `request` is `refusals.malformed`). The Rust server's `json::parse` refuses text after the
    /// document and keeps a repeated member as it arrived; the Go server's `jsonBody` decodes the
    /// first value with `json.Decoder` and ignores what follows it, into a map where the last
    /// repeated member wins. The answers to both are the binding's, so they are compared whole.
    #[test]
    fn adv_a_json_body_with_trailing_text_or_a_repeated_member_is_read_alike() {
        let mut differing = String::new();
        for label in ["put-trailing", "put-repeated-member"] {
            let (rust, go) = answered(label);
            let bodies = rust.body == go.body
                || (super::unbound_rendering(&rust.body)
                    && serde_json::from_str::<serde_json::Value>(&rust.body).ok()
                        == serde_json::from_str::<serde_json::Value>(&go.body).ok());
            if rust.status != go.status || rust.headers != go.headers || !bodies {
                let _ = writeln!(differing, "{label}:\n  rust {rust:?}\n  go   {go:?}");
            }
        }
        assert!(differing.is_empty(), "the servers disagree:\n{differing}");
    }

    /// A standard refusal's literal body is any text (`ESS-HTTP-029` checks only that its member
    /// has a name), so a French message with a no-break space before `!` compiles. The Go server
    /// writes the text into Go source with Rust's `{:?}`, whose `\u{a0}` is no Go escape; the
    /// package's own `go_string` (`src/go/selection.rs`) is the Go spelling.
    #[test]
    fn adv_a_refusal_literal_holding_a_no_break_space_builds_in_go() {
        let ir = shared::ir();
        let from = "body: {error: {literal: invalid_request}}";
        assert!(shared::ROUTES.contains(from), "the fixture's refusal moved");
        let routes = shared::ROUTES.replace(
            from,
            "body: {error: {literal: \"requ\\u00eate invalide\\u00a0!\"}}",
        );
        let synthesis = synthesize_bound(
            &ir,
            Target::Go,
            OutputLayout::Workspace,
            Some(&shared::binding(&ir, &routes)),
        )
        .unwrap_or_else(|failure| panic!("synthesizes: {failure}"));
        let directory = scratch("nbsp-vet");
        shared::write(&synthesis, &directory);
        let vetted = super::go(&directory, &scratch("nbsp-vet.tsv"), &["vet", "./..."]);
        let _ = std::fs::remove_dir_all(&directory);
        assert!(
            vetted.status.success(),
            "`go vet` refuses the bound server:\n{}",
            String::from_utf8_lossy(&vetted.stderr)
        );
    }

    /// A server binding that refines a protocol binding leaving `Purge` open is served by the Go
    /// server at the path the server binding fills in, with the protocol's refusal overrides.
    #[test]
    fn adv_a_refining_binding_is_served_by_go_at_the_path_it_fills_in() {
        let ir = shared::ir();
        let split = shared::ROUTES
            .find("routes:\n")
            .expect("the fixture has routes");
        let refusals = &shared::ROUTES[..split];
        let protocol = shared::document(
            &ir,
            &format!("{refusals}routes:\n  - command: shop.items.Purge\n    method: post\n"),
        );
        let refined =
            ess_http::Refined::from_yaml(protocol.as_bytes()).expect("the protocol reads");
        let server = shared::document(
            &ir,
            &format!(
                "refines: {}\n{}",
                ess_http::digest(protocol.as_bytes()),
                shared::ROUTES
            ),
        );
        let spec = ess_http::HttpSpec::from_yaml(&server).expect("well formed");
        let compiled =
            ess_http::compile_refining(&spec, Some(&refined), &ir, &ess_gen::http::derived(&ir))
                .unwrap_or_else(|refused| panic!("the server binding compiles: {refused}"));
        let bound = Binding::new(compiled).expect("every route has a path");
        let synthesis = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes: {failure}"));
        let answers = go_run(
            "refined",
            &synthesis,
            &[
                ["purge", "y", "POST", "/purge/p1", "", "", ""],
                ["purge-open", "y", "POST", "/purge", "", "", ""],
            ],
        );
        let purge = &answers["purge"];
        assert_eq!(
            (purge.status, purge.json()),
            (403, serde_json::json!({"error": "access_denied"})),
            "{purge:?}"
        );
        assert_eq!(
            answers["purge-open"].status, 404,
            "{:?}",
            answers["purge-open"]
        );
    }

    /// Each insertion into [`shared::MODEL`] that gives it a command whose input holds `Json`: an
    /// anchor in the fixture, starting at a line end, and the text written after that line end.
    const JSON_MODEL: &[(&str, &str)] = &[
        (
            "\nentities:\n",
            "  - name: shop.items.Note\n    kind: struct\n    fields:\n      - {name: label, \
             type: String}\n      - {name: data, type: Json}\n",
        ),
        (
            "\ncommands:\n",
            "  - name: shop.items.Noted\n    fields:\n      - {name: data, type: Json}\n      - \
             {name: note, type: shop.items.Note}\n",
        ),
        (
            "\nviews:\n",
            "  - name: shop.items.Annotate\n    input:\n      - {name: item_id, type: \
             shop.items.ItemId}\n      - {name: data, type: Json}\n      - {name: note, type: \
             shop.items.Note}\n    outcomes:\n      - name: annotated\n        emits: \
             [shop.items.Noted]\n        payload:\n          shop.items.Noted: {data: input.data, \
             note: input.note}\n",
        ),
        ("\ncomponents:\n", "      - shop.items.Annotate\n"),
        (
            "\n        - shop.items.Purge\n",
            "        - shop.items.Annotate\n",
        ),
    ];

    /// The fixture model with [`JSON_MODEL`] in it, synthesized for Rust and for Go with the
    /// fixture's routes and one more, `POST /notes/{item_id}` for `Annotate`: `(rust, go)`.
    fn json_syntheses() -> (Synthesis, Synthesis) {
        let mut model = shared::MODEL.to_owned();
        for (anchor, inserted) in JSON_MODEL {
            assert_eq!(
                model.matches(anchor).count(),
                1,
                "the fixture moved: {anchor}"
            );
            let at = model.find(anchor).expect("present") + 1;
            model.insert_str(at, inserted);
        }
        let publishes = "shop.items.Searched, shop.items.Approved]";
        assert_eq!(model.matches(publishes).count(), 1, "the fixture moved");
        let model = model.replace(
            publishes,
            "shop.items.Searched, shop.items.Approved, shop.items.Noted]",
        );
        let ir = ir_of(&model);
        let routes = format!(
            "{}  - command: shop.items.Annotate\n    method: post\n    path: /notes/{{item_id}}\n    \
             parameters: {{item_id: input.item_id}}\n",
            shared::ROUTES
        );
        let bound = shared::binding(&ir, &routes);
        let rust = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for Rust: {failure}"));
        let go = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for Go: {failure}"));
        (rust, go)
    }

    /// A `Json` input field, at the top of a `json` body and inside a body member, holding an
    /// object (members not in alphabetical order, nested), an array and a scalar. The Rust server
    /// hands the command the value as it arrived (`json::parse`); the Go server's `jsonBody` and
    /// `withPath` must hand the generated decoder a value its `jsonAt` and `objectAt` read, so the
    /// command publishes what it was sent and both servers write the same bytes. The model's
    /// other readers are held alike once it uses `Json`.
    #[test]
    fn adv_a_json_input_reaches_the_command_as_the_rust_server_delivers_it() {
        let (rust, go) = json_syntheses();
        let sent: &[(&str, &str, &str)] = &[
            (
                "json-object",
                r#"{"b":1,"a":[true,null,"x"],"c":{"z":{"y":[]},"d":"e"}}"#,
                r#"{"z":{"y":[]},"c":"d"}"#,
            ),
            ("json-array", r#"[1,{"z":2,"y":3},[],"s"]"#, "[]"),
            ("json-scalar", r#""text""#, "1.50"),
            ("json-false", "false", "12"),
        ];
        let bodies: Vec<String> = sent
            .iter()
            .map(|(_, data, nested)| {
                format!(r#"{{"data":{data},"note":{{"label":"l","data":{nested}}}}}"#)
            })
            .collect();
        let mut requests: Vec<[&str; 7]> = sent
            .iter()
            .zip(&bodies)
            .map(|((label, _, _), body)| [*label, "y", "POST", "/notes/n1", "", "", body.as_str()])
            .collect();
        // The model's other readers, now that it uses `Json`: a view's path, a `form` body, a
        // `query` string and a command with no body, each handed to a generated decoder.
        let others: &[[&str; 7]] = &[
            ["json-model-view", "y", "GET", "/items/zz", "", "", ""],
            ["json-model-delete", "y", "DELETE", "/items/zz", "", "", ""],
            [
                "json-model-form",
                "y",
                "POST",
                "/strict-orders",
                "",
                "application/x-www-form-urlencoded",
                "item_id=x&count=2",
            ],
            [
                "json-model-query",
                "y",
                "GET",
                "/search",
                "words=a&limit=1",
                "",
                "",
            ],
        ];
        requests.extend_from_slice(others);
        let rust_answers = rust_run("json-input", &rust, &requests);
        let go_answers = go_run("json-input", &go, &requests);
        let mut differing = String::new();
        for [label, ..] in others {
            let (rust, go) = (&rust_answers[*label], &go_answers[*label]);
            let bodies = rust.body == go.body
                || (super::unbound_rendering(&rust.body)
                    && serde_json::from_str::<serde_json::Value>(&rust.body).ok()
                        == serde_json::from_str::<serde_json::Value>(&go.body).ok());
            if rust.status == 400
                || rust.status != go.status
                || rust.headers != go.headers
                || !bodies
            {
                let _ = writeln!(differing, "{label}:\n  rust {rust:?}\n  go   {go:?}");
            }
        }
        for (label, data, nested) in sent {
            let rust = &rust_answers[*label];
            let go = &go_answers[*label];
            if rust.status != go.status || rust.headers != go.headers || rust.body != go.body {
                let _ = writeln!(differing, "{label}:\n  rust {rust:?}\n  go   {go:?}");
                continue;
            }
            assert!(
                (200..300).contains(&rust.status),
                "{label}: the command ran: {rust:?}"
            );
            let payload = shared::published(rust);
            let expected: serde_json::Value = serde_json::from_str(data).expect("JSON");
            let nested_expected: serde_json::Value = serde_json::from_str(nested).expect("JSON");
            assert_eq!(payload["data"], expected, "{label}: the command saw `data`");
            assert_eq!(
                payload["note"],
                serde_json::json!({"label": "l", "data": nested_expected}),
                "{label}: the command saw `note`"
            );
            assert!(
                rust.body.contains(data) && rust.body.contains(nested),
                "{label}: each value is written as it arrived: {}",
                rust.body
            );
        }
        assert!(differing.is_empty(), "the servers disagree:\n{differing}");
    }
}

/// Two adopter reports on one variant of the fixture, each server built once for both.
///
/// - `DropItem` gains an `Optional<String>` input and is bound `delete` with
///   `request: {encoding: query}`: both servers read it from the query string, never from a body
///   (`story:a-delete-command-may-take-its-input-from-the-query-string`).
/// - `ItemById`, whose parameter `item_id` and field `title` declare `wire:`, is bound
///   `get /lookup` with its parameter unmapped: both servers read the parameter from the query
///   string under its wire name and write the field under its own
///   (`story:a-view-parameter-carries-its-wire-name-on-every-http-reader`).
mod adopter_wire_reports {
    use std::collections::BTreeMap;

    use ess_synth::{synthesize_bound, OutputLayout, Target};

    use super::adversary_k6_pass1::{go_run, ir_of, rust_run};
    use super::shared::{self, Answer};

    /// `(anchor, replacement)`, each anchor once in the fixture.
    const EDITS: &[(&str, &str)] = &[
        (
            "  - name: shop.items.DropItem
    input:
      - {name: item_id, type: shop.items.ItemId}
    outcomes:
      - name: dropped
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
",
            "  - name: shop.items.DropItem
    input:
      - {name: item_id, type: shop.items.ItemId}
      - {name: reason, type: Optional<String>}
    outcomes:
      - name: dropped
        emits: [shop.items.Dropped]
        payload:
          shop.items.Dropped: {item_id: input.item_id, reason: input.reason}
",
        ),
        (
            "  - name: shop.items.Approved\n",
            "  - name: shop.items.Dropped
    fields:
      - {name: item_id, type: shop.items.ItemId}
      - {name: reason, type: Optional<String>}
  - name: shop.items.Approved\n",
        ),
        (
            "shop.items.Searched, shop.items.Approved]",
            "shop.items.Searched, shop.items.Approved, shop.items.Dropped]",
        ),
        (
            "      - {name: item_id, type: shop.items.ItemId}
    filter: item_id == param.item_id
    fields:
      - {name: item_id, type: shop.items.ItemId}
      - {name: title, type: String}
",
            "      - {name: item_id, type: shop.items.ItemId, wire: itemKey}
    filter: item_id == param.item_id
    fields:
      - {name: item_id, type: shop.items.ItemId}
      - {name: title, type: String, wire: itemTitle}
",
        ),
    ];

    /// `(anchor, replacement)` on the fixture's routes, each anchor once.
    const ROUTE_EDITS: &[(&str, &str)] = &[
        (
            "  - view: shop.items.ItemById
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
",
            "  - view: shop.items.ItemById
    method: get
    path: /lookup
",
        ),
        (
            "  - command: shop.items.DropItem
    method: delete
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
",
            "  - command: shop.items.DropItem
    method: delete
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: query}
",
        ),
    ];

    /// Each `drop-…` request is preceded by a `put` of the item it drops.
    const REQUESTS: &[[&str; 7]] = &[
        [
            "put",
            "y",
            "PUT",
            "/items/w1",
            "",
            "",
            r#"{"title":"Lamp"}"#,
        ],
        ["lookup", "y", "GET", "/lookup", "itemKey=w1", "", ""],
        ["lookup-none", "y", "GET", "/lookup", "itemKey=zz", "", ""],
        [
            "lookup-model-name",
            "y",
            "GET",
            "/lookup",
            "item_id=w1",
            "",
            "",
        ],
        [
            "drop-query",
            "y",
            "DELETE",
            "/items/w1",
            "reason=gone+away",
            "",
            "",
        ],
        [
            "put-2",
            "y",
            "PUT",
            "/items/w1",
            "",
            "",
            r#"{"title":"Lamp"}"#,
        ],
        ["drop-no-reason", "y", "DELETE", "/items/w1", "", "", ""],
        [
            "put-3",
            "y",
            "PUT",
            "/items/w1",
            "",
            "",
            r#"{"title":"Lamp"}"#,
        ],
        [
            "drop-body-not-read",
            "y",
            "DELETE",
            "/items/w1",
            "",
            "application/json",
            r#"{"reason":"from the body"}"#,
        ],
        [
            "put-4",
            "y",
            "PUT",
            "/items/w1",
            "",
            "",
            r#"{"title":"Lamp"}"#,
        ],
        [
            "drop-unknown-key",
            "y",
            "DELETE",
            "/items/w1",
            "why=x",
            "",
            "",
        ],
    ];

    /// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary.
    fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
        static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
            std::sync::OnceLock::new();
        BOTH.get_or_init(|| {
            let mut model = shared::MODEL.to_owned();
            for (anchor, replacement) in EDITS {
                assert_eq!(
                    model.matches(anchor).count(),
                    1,
                    "the fixture moved: {anchor}"
                );
                model = model.replace(anchor, replacement);
            }
            let ir = ir_of(&model);
            let mut routes = shared::ROUTES.to_owned();
            for (anchor, replacement) in ROUTE_EDITS {
                assert_eq!(
                    routes.matches(anchor).count(),
                    1,
                    "the routes moved: {anchor}"
                );
                routes = routes.replace(anchor, replacement);
            }
            let bound = shared::binding(&ir, &routes);
            let rust = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for Rust: {failure}"));
            let go = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for Go: {failure}"));
            (
                rust_run("adopter-wire", &rust, REQUESTS),
                go_run("adopter-wire", &go, REQUESTS),
            )
        })
    }

    /// Each server's answer to `label`, both held to the same status and JSON body.
    fn answered(label: &str) -> [&'static Answer; 2] {
        let (rust, go) = both();
        let rust = rust
            .get(label)
            .unwrap_or_else(|| panic!("the Rust server answered `{label}`"));
        let go = go
            .get(label)
            .unwrap_or_else(|| panic!("the Go server answered `{label}`"));
        assert_eq!(
            (
                rust.status,
                serde_json::from_str::<serde_json::Value>(&rust.body).ok()
            ),
            (
                go.status,
                serde_json::from_str::<serde_json::Value>(&go.body).ok()
            ),
            "{label}: the servers disagree:\n  rust {rust:?}\n  go   {go:?}"
        );
        [rust, go]
    }

    /// The `reason` the command saw, as the event it published carries it.
    fn reason(answer: &Answer) -> Option<String> {
        let payload = shared::published(answer);
        payload
            .get("reason")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    }

    #[test]
    fn a_delete_command_encoded_query_reads_its_input_from_the_query_string() {
        for answer in answered("drop-query") {
            assert_eq!(answer.status, 202, "{answer:?}");
            assert_eq!(reason(answer).as_deref(), Some("gone away"), "{answer:?}");
        }
        for answer in answered("drop-no-reason") {
            assert_eq!(answer.status, 202, "{answer:?}");
            assert_eq!(reason(answer), None, "{answer:?}");
        }
        // A body is not where the input is read from.
        for answer in answered("drop-body-not-read") {
            assert_eq!(answer.status, 202, "{answer:?}");
            assert_eq!(reason(answer), None, "{answer:?}");
        }
        // `unknown: refuse`, the default, holds for a query parameter no field names.
        for answer in answered("drop-unknown-key") {
            assert_eq!(answer.status, 400, "{answer:?}");
        }
    }

    #[test]
    fn a_bound_view_reads_its_parameter_and_writes_its_field_under_their_wire_names() {
        for answer in answered("put") {
            assert_eq!(answer.status, 202, "{answer:?}");
        }
        for answer in answered("lookup") {
            assert_eq!(answer.status, 200, "{answer:?}");
            assert_eq!(
                answer.json(),
                serde_json::json!({"item_id": "w1", "itemTitle": "Lamp"}),
                "{answer:?}"
            );
        }
        for answer in answered("lookup-none") {
            assert_eq!(answer.status, 404, "{answer:?}");
        }
        // The model's name is not the query key: `itemKey` is missing and `item_id` is unknown.
        for answer in answered("lookup-model-name") {
            assert_eq!(answer.status, 400, "{answer:?}");
        }
    }
}

#[path = "adversary_u9c_pass1.rs"]
mod adversary_u9c_pass1;

#[path = "bare_field_and_errors.rs"]
mod bare_field_and_errors;

#[path = "adversary_u9d_pass1.rs"]
mod adversary_u9d_pass1;
