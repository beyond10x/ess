//! A route whose `ess-http/1` binding requires a scheme, served by the generated Rust and Go
//! servers, which answer alike (beyond10x/ess#681,
//! `story:the-rust-http-server-answers-an-unauthenticated-call-with-its-challenge` and its Go
//! story):
//! a call the realization authenticated as no actor, or one carrying no credential where any of the
//! route's schemes puts it, is answered `unauthenticated` before its grant is checked and runs
//! nothing; an open route and an ungranted caller answer as they did.

use std::collections::BTreeMap;

use ess_synth::{
    synthesize_bound, BindingRefusal, OutputLayout, Synthesis, SynthesisFailure, Target,
};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{go_run, rust_run};

const ROUTES: &str = "authentication:
  schemes:
    bearer_auth: {kind: bearer}
    key_auth: {kind: api_key, in: header, name: X-Api-Key}
    session: {kind: api_key, in: cookie, name: session}
  required: [bearer_auth]
routes:
  - view: shop.items.ItemById
    method: get
    path: /items/{item_id}
    parameters: {item_id: param.item_id}
    body: single
    empty: {status: 404}
    authentication: []
  - command: shop.items.PutItem
    method: put
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
    authentication: [bearer_auth, key_auth]
  - command: shop.items.DropItem
    method: delete
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
    authentication: [session]
    refusals:
      unauthenticated:
        status: 401
        body: {error: {literal: login_required}}
        headers:
          WWW-Authenticate: {challenge: {scheme: Cookie, params: {name: {literal: session}}}}
  - command: shop.items.Purge
    method: post
    path: /purge/{item_id}
    parameters: {item_id: input.item_id}
  - command: shop.items.Search
    method: get
    path: /search
    request: {encoding: query}
    authentication: []
";

const LAMP: &str = r#"{"title":"Lamp"}"#;
const BEARER: &str = "authorization: Bearer test-token";

/// `label`, the caller (`y` the granted `Clerk`, `n` no actor), method, path, query, the request's
/// headers (`name: value` joined by ` | `) and body, in the order the harness sends them.
const REQUESTS: &[[&str; 7]] = &[
    ["put-none", "n", "PUT", "/items/a", "", "", LAMP],
    ["put-claimed", "y", "PUT", "/items/a", "", "", LAMP],
    [
        "put-malformed",
        "y",
        "PUT",
        "/items/a",
        "",
        "authorization: Bearer",
        LAMP,
    ],
    [
        "put-two-tokens",
        "y",
        "PUT",
        "/items/a",
        "",
        "authorization: Bearer test token",
        LAMP,
    ],
    [
        "put-wrong-scheme",
        "y",
        "PUT",
        "/items/a",
        "",
        "authorization: Basic dGVzdC10b2tlbg==",
        LAMP,
    ],
    ["put-anonymous", "n", "PUT", "/items/a", "", BEARER, LAMP],
    ["get-not-stored", "n", "GET", "/items/a", "", "", ""],
    ["put-bearer", "y", "PUT", "/items/a", "", BEARER, LAMP],
    [
        "put-key",
        "y",
        "PUT",
        "/items/b",
        "",
        "x-api-key: test-token",
        LAMP,
    ],
    ["get-stored", "n", "GET", "/items/a", "", "", ""],
    ["get-key-stored", "n", "GET", "/items/b", "", "", ""],
    [
        "purge-granted-not",
        "y",
        "POST",
        "/purge/a",
        "",
        "authorization: bearer test-token",
        "",
    ],
    ["purge-none", "n", "POST", "/purge/a", "", "", ""],
    ["drop-no-cookie", "y", "DELETE", "/items/a", "", "", ""],
    [
        "drop-other-cookie",
        "y",
        "DELETE",
        "/items/a",
        "",
        "cookie: theme=dark",
        "",
    ],
    [
        "drop-cookie",
        "y",
        "DELETE",
        "/items/a",
        "",
        "cookie: theme=dark; session=test-token",
        "",
    ],
    ["search-open", "n", "GET", "/search", "limit=1", "", ""],
    [
        "search-open-granted",
        "y",
        "GET",
        "/search",
        "limit=1",
        "",
        "",
    ],
];

/// Both servers' answers to [`REQUESTS`]: `(rust, go)`, once per test binary
/// (`story:the-go-http-server-answers-an-unauthenticated-call-with-its-challenge`).
fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let ir = shared::ir();
        let bound = shared::binding(&ir, ROUTES);
        let synthesis = |target| -> Synthesis {
            synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound))
                .unwrap_or_else(|failure| panic!("synthesizes for {target:?}: {failure}"))
        };
        (
            rust_run("w681auth", &synthesis(Target::Rust), REQUESTS),
            go_run("w681auth", &synthesis(Target::Go), REQUESTS),
        )
    })
}

fn rust() -> &'static BTreeMap<String, Answer> {
    &both().0
}

/// The Rust server's answer to `label`, the Go server's held to its status, `Content-Type`,
/// headers and body: the same JSON value, as the existing tests hold the standard `403`, whose
/// members the two servers write in different orders, and the same bytes for every other body.
fn answered(label: &str) -> &'static Answer {
    let (rust, go) = both();
    let rust = rust
        .get(label)
        .unwrap_or_else(|| panic!("the Rust server answered `{label}`"));
    let go = go
        .get(label)
        .unwrap_or_else(|| panic!("the Go server answered `{label}`"));
    let body = |answer: &Answer| {
        serde_json::from_str::<serde_json::Value>(&answer.body)
            .ok()
            .filter(|_| answer.status == 403)
            .unwrap_or_else(|| serde_json::Value::String(answer.body.clone()))
    };
    assert_eq!(
        (
            rust.status,
            rust.content_type.as_str(),
            body(rust),
            &rust.headers
        ),
        (go.status, go.content_type.as_str(), body(go), &go.headers),
        "{label}: the servers disagree:\n  rust {rust:?}\n  go   {go:?}"
    );
    rust
}

#[test]
fn both_servers_answer_every_request_alike() {
    for request in REQUESTS {
        answered(request[0]);
    }
    for answer in both().1.values() {
        assert!(
            !answer.body.contains("test-token")
                && !format!("{:?}", answer.headers).contains("test-token"),
            "go: {answer:?}"
        );
    }
}

/// `label` answered the default `unauthenticated` refusal with the `Bearer` challenge.
fn challenged(label: &str) {
    let answer = answered(label);
    assert_eq!(answer.status, 401, "{label}: {answer:?}");
    assert_eq!(
        answer.body, r#"{"refused":"unauthenticated"}"#,
        "{label}: {answer:?}"
    );
    assert_eq!(
        answer.headers,
        [("WWW-Authenticate".to_owned(), "Bearer".to_owned())],
        "{label}: {answer:?}"
    );
}

#[test]
fn a_call_without_the_credential_gets_the_challenge_and_runs_nothing() {
    for label in [
        "put-none",
        "put-claimed",
        "put-malformed",
        "put-two-tokens",
        "put-wrong-scheme",
        "put-anonymous",
        "purge-none",
    ] {
        challenged(label);
    }
    // Nothing those calls sent was stored.
    assert_eq!(answered("get-not-stored").status, 404);
    // No answer echoes the credential.
    for answer in rust().values() {
        assert!(
            !answer.body.contains("test-token")
                && !format!("{:?}", answer.headers).contains("test-token"),
            "{answer:?}"
        );
    }
}

#[test]
fn a_credential_of_any_listed_scheme_admits_a_call_the_realization_authenticated() {
    for label in ["put-bearer", "put-key"] {
        let answer = answered(label);
        assert!(answer.status < 300, "{label}: {answer:?}");
    }
    assert_eq!(answered("get-stored").status, 200);
    assert_eq!(answered("get-key-stored").status, 200);
    let dropped = answered("drop-cookie");
    assert!(dropped.status < 300, "{dropped:?}");
}

#[test]
fn an_open_route_and_an_ungranted_caller_answer_as_today() {
    let ungranted = answered("purge-granted-not");
    assert_eq!(ungranted.status, 403, "{ungranted:?}");
    assert!(
        ungranted.body.contains(r#""actor":"shop.items.Clerk""#),
        "{ungranted:?}"
    );
    let open = answered("search-open");
    assert_eq!(open.status, 403, "{open:?}");
    assert!(open.body.contains(r#""actor":null"#), "{open:?}");
    assert!(answered("search-open-granted").status < 300);
}

#[test]
fn a_route_override_answers_a_call_without_its_cookie() {
    for label in ["drop-no-cookie", "drop-other-cookie"] {
        let answer = answered(label);
        assert_eq!(answer.status, 401, "{label}: {answer:?}");
        assert_eq!(answer.body, r#"{"error":"login_required"}"#, "{label}");
        assert_eq!(
            answer.header("WWW-Authenticate"),
            Some(r#"Cookie name="session""#),
            "{label}: {answer:?}"
        );
    }
}

#[test]
fn a_route_requiring_only_a_query_key_is_refused_at_synthesis() {
    let ir = shared::ir();
    let routes = ROUTES.replace(
        "    session: {kind: api_key, in: cookie, name: session}",
        "    session: {kind: api_key, in: query, name: session}",
    );
    assert_ne!(routes, ROUTES);
    let routes = routes.replace(
        "    refusals:\n      unauthenticated:\n        status: 401\n        body: {error: {literal: login_required}}\n        headers:\n          WWW-Authenticate: {challenge: {scheme: Cookie, params: {name: {literal: session}}}}\n",
        "    refusals:\n      unauthenticated: {status: 403, body: none}\n",
    );
    let bound = shared::binding(&ir, &routes);
    for target in [Target::Rust, Target::Go] {
        match synthesize_bound(&ir, target, OutputLayout::Workspace, Some(&bound)) {
            Err(SynthesisFailure::Binding(BindingRefusal::Unservable(refused))) => {
                let text = refused.to_string();
                assert!(
                    text.contains("shop.items.DropItem") && text.contains("query"),
                    "{text}"
                );
            }
            Err(other) => panic!("{target:?}: refused otherwise: {other}"),
            Ok(_) => panic!("{target:?}: a query key alone is served"),
        }
    }
}
