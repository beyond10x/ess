//! An `ess-http/1` outcome declares response headers, a `WWW-Authenticate` challenge and a
//! redirect, and a standard refusal declares headers of its own
//! (`docs/design/http-presentation-binding.md`, "Response headers", "Redirects" and "Stage 2: keys,
//! precedence and the units that build them").

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{
    compile, compile_refining, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation,
    Refined,
};
use serde_json::{json, Value};

const MODEL: &str = include_str!("fixtures/auth.yaml");

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("auth.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("auth.yaml", MODEL);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The routes ESS derives for `ir`, written out the way `ess_gen::http::routes` derives them for
/// this fixture, as `tests/compile.rs` does.
fn derived(ir: &EssIr) -> Vec<DerivedRoute> {
    let mut table = Vec::new();
    for component in ir.components().values() {
        for handle in &component.accepts {
            table.push(DerivedRoute {
                component: component.name.to_string(),
                operation: Operation::Command(handle.name().to_string()),
                method: Method::Post,
                path: format!("/auth/commands/{}", handle.name().local()),
            });
        }
    }
    table.sort_by(|left, right| {
        (&left.component, &left.path, left.method, &left.operation).cmp(&(
            &right.component,
            &right.path,
            right.method,
            &right.operation,
        ))
    });
    table
}

fn header(ir: &EssIr) -> String {
    format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n",
        ir.source_digest()
    )
}

fn compiled(ir: &EssIr, document: &str) -> Result<HttpIr, HttpDiagnostics> {
    let spec = HttpSpec::from_yaml(&(header(ir) + document))
        .unwrap_or_else(|error| panic!("parses: {error}\n{document}"));
    compile(&spec, ir, &derived(ir))
}

fn located(result: Result<HttpIr, HttpDiagnostics>) -> Vec<(&'static str, String)> {
    match result {
        Ok(_) => Vec::new(),
        Err(diagnostics) => diagnostics
            .0
            .into_iter()
            .map(|diagnostic| (diagnostic.code, diagnostic.path))
            .collect(),
    }
}

fn worded(result: Result<HttpIr, HttpDiagnostics>) -> Vec<String> {
    result
        .expect_err("refused")
        .0
        .into_iter()
        .map(|diagnostic| diagnostic.to_string())
        .collect()
}

fn refused(code: &'static str, path: &str) -> (&'static str, String) {
    (code, path.to_owned())
}

fn ir_json(http: &HttpIr) -> Value {
    serde_json::from_str(&http.to_canonical_json()).expect("canonical JSON")
}

/// The `outcomes` the IR writes for the one route of `http`.
fn outcomes_of(http: &HttpIr) -> Value {
    let json = ir_json(http);
    json["routes"][0]["outcomes"].clone()
}

/// A `ReadPrivateItem` route on `get /private/items/{item_id}` listing `outcomes`.
fn read_private(outcomes: &str) -> String {
    format!(
        "routes:\n  - command: catalog.auth.ReadPrivateItem\n    method: get\n    path: /private/items/{{item_id}}\n    parameters: {{item_id: input.item_id}}\n    outcomes:\n{outcomes}"
    )
}

/// An `ApproveOrder` route on `get /approve`, encoded `query`, listing `outcomes`.
fn approve(outcomes: &str) -> String {
    format!(
        "routes:\n  - command: catalog.auth.ApproveOrder\n    method: get\n    path: /approve\n    request: {{encoding: query}}\n    outcomes:\n{outcomes}"
    )
}

/// The page's `ReadPrivateItem` example: a refused token answers `401` with the error object and a
/// `Bearer` challenge whose `error` attribute is the error's wire name.
const BEARER: &str = "      token-refused:
        status: 401
        body: {error: error}
        headers:
          WWW-Authenticate:
            challenge:
              scheme: Bearer
              params:
                realm: {literal: catalog}
                error: {source: error.wire}
";

/// The page's `ApproveOrder` example, with the `body: none` every redirect requires
/// (`ESS-HTTP-024`): the approval redirects to the caller's URI with the code, and the state only
/// when the request carried one.
const APPROVED: &str = "      approved:
        status: 302
        body: none
        redirect:
          location: input.return_to
          mode: query
          parameters:
            code: response.code
            state: input.state
";

#[test]
fn headers_compile_in_name_order_whatever_the_authored_order() {
    let ir = model();
    let one = compiled(
        &ir,
        &read_private(
            "      read:
        status: 200
        body: response
        headers:
          X-Item: {source: input.item_id}
          Cache-Control: {literal: no-store}
          Pragma: {literal: no-cache}
",
        ),
    )
    .expect("compiles");
    let other = compiled(
        &ir,
        &read_private(
            "      read:
        status: 200
        body: response
        headers:
          Pragma: {literal: no-cache}
          X-Item: {source: input.item_id}
          Cache-Control: {literal: no-store}
",
        ),
    )
    .expect("compiles");
    let text = one.to_canonical_json();
    assert_eq!(other.to_canonical_json(), text);
    assert_eq!(
        outcomes_of(&one),
        json!({"read": {"status": 200, "body": "response", "headers": {
            "Cache-Control": {"literal": "no-store"},
            "Pragma": {"literal": "no-cache"},
            "X-Item": {"source": "input.item_id"},
        }}})
    );
    let cache = text.find("\"Cache-Control\"").expect("written");
    let pragma = text.find("\"Pragma\"").expect("written");
    let item = text.find("\"X-Item\"").expect("written");
    assert!(cache < pragma && pragma < item, "{text}");
    // The canonical bytes read back unchanged; the same headers out of name order do not.
    let read = HttpIr::from_json(&text).expect("the compiled IR reads back");
    assert_eq!(read.to_canonical_json(), text);
    let mut swapped = ir_json(&one);
    let headers = swapped["routes"][0]["outcomes"]["read"]["headers"].clone();
    let reordered = format!(
        "{{\"X-Item\": {}, \"Cache-Control\": {}, \"Pragma\": {}}}",
        headers["X-Item"], headers["Cache-Control"], headers["Pragma"]
    );
    swapped["routes"][0]["outcomes"]["read"]["headers"] = json!("HEADERS");
    let broken = serde_json::to_string(&swapped)
        .unwrap()
        .replace("\"HEADERS\"", &reordered);
    let error = HttpIr::from_json(&broken)
        .expect_err("out of order")
        .to_string();
    assert!(error.contains("headers"), "{error}");
    // An empty `headers` writes nothing: the outcome compiles to the bytes without it.
    let bare = compiled(&ir, &read_private("      read: {status: 200}\n"))
        .expect("compiles")
        .to_canonical_json();
    let empty = compiled(
        &ir,
        &read_private("      read: {status: 200, headers: {}}\n"),
    )
    .expect("compiles")
    .to_canonical_json();
    assert_eq!(empty, bare);
    assert!(!bare.contains("headers"), "{bare}");
}

#[test]
fn a_bearer_challenge_with_an_error_source_compiles() {
    let ir = model();
    let http = compiled(&ir, &read_private(BEARER)).expect("compiles");
    assert_eq!(
        outcomes_of(&http),
        json!({"token-refused": {"status": 401, "body": {"error": "error"}, "headers": {
            "WWW-Authenticate": {"challenge": {"scheme": "Bearer", "params": {
                "error": {"source": "error.wire"},
                "realm": {"literal": "catalog"},
            }}},
        }}})
    );
    let text = http.to_canonical_json();
    let read = HttpIr::from_json(&text).expect("the compiled IR reads back");
    assert_eq!(read.to_canonical_json(), text);
    // A param read from one of the error's fields, the header named in another case, and a
    // challenge without params compile too; the IR keeps the name as authored.
    let http = compiled(
        &ir,
        &read_private(
            "      token-refused:
        status: 401
        headers:
          www-authenticate:
            challenge:
              scheme: Bearer
              params:
                error: {source: error.wire}
                error_description: {source: error.description}
",
        ),
    )
    .expect("compiles");
    assert_eq!(
        outcomes_of(&http)["token-refused"]["headers"],
        json!({"www-authenticate": {"challenge": {"scheme": "Bearer", "params": {
            "error": {"source": "error.wire"},
            "error_description": {"source": "error.description"},
        }}}})
    );
    let http = compiled(
        &ir,
        &read_private(
            "      token-refused:
        status: 401
        headers: {WWW-Authenticate: {challenge: {scheme: Basic}}}
",
        ),
    )
    .expect("compiles");
    assert_eq!(
        outcomes_of(&http)["token-refused"]["headers"],
        json!({"WWW-Authenticate": {"challenge": {"scheme": "Basic"}}})
    );
    // A standard refusal answers `401` with a challenge whose params are all literals.
    let http = compiled(
        &ir,
        "refusals:
  not_granted:
    status: 401
    body: none
    headers:
      WWW-Authenticate: {challenge: {scheme: Bearer, params: {realm: {literal: catalog}}}}
      Cache-Control: {literal: no-store}
routes: []
",
    )
    .expect("compiles");
    assert_eq!(
        ir_json(&http)["refusals"],
        json!({"not_granted": {"status": 401, "body": "none", "headers": {
            "Cache-Control": {"literal": "no-store"},
            "WWW-Authenticate": {"challenge": {"scheme": "Bearer", "params": {
                "realm": {"literal": "catalog"},
            }}},
        }}})
    );
    let text = http.to_canonical_json();
    assert_eq!(
        HttpIr::from_json(&text)
            .expect("reads back")
            .to_canonical_json(),
        text
    );
}

#[test]
fn ess_http_026_a_bad_or_reserved_or_duplicate_header_is_refused() {
    let ir = model();
    let at = |outcome: &str, name: &str| format!("routes[0].outcomes.{outcome}.headers.{name}");
    for (outcome, headers, name, named) in [
        // A name that is not an RFC 9110 token.
        ("read", "{'X(Item)': {literal: a}}", "X(Item)", "token"),
        ("read", "{'': {literal: a}}", "", "token"),
        // A header the encoding or `redirect` owns, in any case.
        (
            "read",
            "{Content-Type: {literal: a}}",
            "Content-Type",
            "encoding",
        ),
        (
            "read",
            "{content-length: {literal: '1'}}",
            "content-length",
            "encoding",
        ),
        (
            "read",
            "{Transfer-Encoding: {literal: chunked}}",
            "Transfer-Encoding",
            "encoding",
        ),
        (
            "read",
            "{Location: {literal: 'https://a'}}",
            "Location",
            "redirect",
        ),
        // A header declared twice, ignoring case.
        (
            "read",
            "{Cache-Control: {literal: no-store}, cache-control: {literal: no-cache}}",
            "cache-control",
            "`Cache-Control`",
        ),
        // A source that names no field, or a field of no scalar type.
        ("read", "{X-Item: {source: input.nope}}", "X-Item", "`nope`"),
        (
            "token-refused",
            "{X-Grant: {source: error.grant}}",
            "X-Grant",
            "Grant",
        ),
        // A source the outcome cannot supply.
        (
            "read",
            "{X-Error: {source: error.description}}",
            "X-Error",
            "no error",
        ),
        (
            "token-refused",
            "{X-Item: {source: response.item_id}}",
            "X-Item",
            "returns",
        ),
        (
            "read",
            "{X-Error: {source: error.wire}}",
            "X-Error",
            "no error",
        ),
        // A source outside the four forms.
        (
            "read",
            "{X-Item: {source: param.item_id}}",
            "X-Item",
            "source",
        ),
        ("read", "{X-Item: {source: input}}", "X-Item", "source"),
        // A literal that is not a valid field value.
        (
            "read",
            "{X-Item: {literal: \"a\\nb\"}}",
            "X-Item",
            "field value",
        ),
        ("read", "{X-Item: {literal: ' a'}}", "X-Item", "field value"),
    ] {
        let status = if outcome == "read" { 200 } else { 403 };
        let document = read_private(&format!(
            "      {outcome}: {{status: {status}, headers: {headers}}}\n"
        ));
        let result = compiled(&ir, &document);
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-026", &at(outcome, name))],
            "{headers}"
        );
        let shown = worded(result);
        assert!(shown[0].contains(named), "{headers}: {shown:?}");
    }
}

#[test]
fn ess_http_026_a_source_the_outcome_cannot_supply_or_a_refusal_reads_is_refused() {
    let ir = model();
    // `error.wire` on an outcome whose error declares a field named `wire` is ambiguous.
    let result = compiled(
        &ir,
        &approve("      wired: {status: 400, headers: {X-Error: {source: error.wire}}}\n"),
    );
    assert_eq!(
        located(result.clone()),
        [refused(
            "ESS-HTTP-026",
            "routes[0].outcomes.wired.headers.X-Error"
        )]
    );
    assert!(worded(result)[0].contains("`wire`"));
    // A `List` is not a scalar, and `response.` needs `returns: true`.
    assert_eq!(
        located(compiled(
            &ir,
            &approve(
                "      noted: {status: 200, headers: {X-Scopes: {source: input.scopes}, X-Code: {source: response.code}}}\n"
            )
        )),
        [
            refused("ESS-HTTP-026", "routes[0].outcomes.noted.headers.X-Scopes"),
            refused("ESS-HTTP-026", "routes[0].outcomes.noted.headers.X-Code"),
        ]
    );
    // An `Optional` scalar, a newtype of one and an enum-free scalar are sources a header writes.
    assert_eq!(
        located(compiled(
            &ir,
            &approve(
                "      noted: {status: 200, headers: {X-State: {source: input.state}, X-Count: {source: input.count}, X-Uri: {source: input.maybe_uri}}}\n"
            )
        )),
        []
    );
    // The same header written twice exactly.
    let text = header(&ir)
        + "routes:\n  - command: catalog.auth.ReadPrivateItem\n    method: get\n    path: /private/items/{item_id}\n    parameters: {item_id: input.item_id}\n    outcomes:\n      read:\n        status: 200\n        headers:\n          Pragma: {literal: no-cache}\n";
    let mut json: Value = ess_yaml::from_str(&text).expect("YAML");
    json["routes"][0]["outcomes"]["read"]["headers"] = json!("HEADERS");
    let json = serde_json::to_string(&json).unwrap().replace(
        "\"HEADERS\"",
        r#"{"Pragma": {"literal": "no-cache"}, "Pragma": {"literal": "no-store"}}"#,
    );
    let spec = HttpSpec::from_json(&json).expect("parses");
    assert_eq!(
        located(compile(&spec, &ir, &derived(&ir))),
        [refused(
            "ESS-HTTP-026",
            "routes[0].outcomes.read.headers.Pragma"
        )]
    );
    // Any `source` in a standard refusal's header: a refusal has no outcome to read one from.
    assert_eq!(
        located(compiled(
            &ir,
            "refusals:
  malformed:
    status: 400
    body: none
    headers:
      X-Item: {source: input.item_id}
      WWW-Authenticate: {challenge: {scheme: Bearer, params: {error: {source: error.wire}}}}
routes: []
"
        )),
        [
            refused("ESS-HTTP-026", "refusals.malformed.headers.X-Item"),
            refused(
                "ESS-HTTP-026",
                "refusals.malformed.headers.WWW-Authenticate.challenge.params.error"
            ),
        ]
    );
    // A refusal's header is held to the same name rules.
    assert_eq!(
        located(compiled(
            &ir,
            "refusals:\n  malformed: {status: 400, body: none, headers: {Content-Type: {literal: a}}}\nroutes: []\n"
        )),
        [refused("ESS-HTTP-026", "refusals.malformed.headers.Content-Type")]
    );
}

#[test]
fn ess_http_027_a_challenge_with_a_bad_scheme_or_param_is_refused() {
    let ir = model();
    let at = "routes[0].outcomes.token-refused.headers.WWW-Authenticate";
    for (challenge, path, named) in [
        // A scheme that is not a token.
        (
            "{scheme: 'Bear er'}",
            format!("{at}.challenge.scheme"),
            "token",
        ),
        ("{scheme: ''}", format!("{at}.challenge.scheme"), "token"),
        // A param name that is not a token.
        (
            "{scheme: Bearer, params: {'re alm': {literal: catalog}}}",
            format!("{at}.challenge.params.re alm"),
            "token",
        ),
        // A param declared twice, ignoring case.
        (
            "{scheme: Bearer, params: {realm: {literal: a}, Realm: {literal: b}}}",
            format!("{at}.challenge.params.Realm"),
            "`realm`",
        ),
        // A param literal a quoted string cannot carry.
        (
            "{scheme: Bearer, params: {realm: {literal: \"a\\u0007\"}}}",
            format!("{at}.challenge.params.realm"),
            "quoted",
        ),
    ] {
        let document = read_private(&format!(
            "      token-refused: {{status: 401, headers: {{WWW-Authenticate: {{challenge: {challenge}}}}}}}\n"
        ));
        let result = compiled(&ir, &document);
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-027", &path)],
            "{challenge}"
        );
        let shown = worded(result);
        assert!(shown[0].contains(named), "{challenge}: {shown:?}");
    }
    // A challenge under any header but `WWW-Authenticate`.
    let result = compiled(
        &ir,
        &read_private(
            "      token-refused: {status: 403, headers: {Proxy-Authenticate: {challenge: {scheme: Bearer}}}}\n",
        ),
    );
    assert_eq!(
        located(result.clone()),
        [refused(
            "ESS-HTTP-027",
            "routes[0].outcomes.token-refused.headers.Proxy-Authenticate"
        )]
    );
    assert!(worded(result)[0].contains("WWW-Authenticate"));
    // A declared `401` without a challenge: no header, or `WWW-Authenticate` as a literal.
    for headers in ["", ", headers: {WWW-Authenticate: {literal: Bearer}}"] {
        let result = compiled(
            &ir,
            &read_private(&format!("      token-refused: {{status: 401{headers}}}\n")),
        );
        assert_eq!(
            located(result.clone()),
            [refused(
                "ESS-HTTP-027",
                "routes[0].outcomes.token-refused.headers"
            )],
            "{headers}"
        );
        assert!(worded(result)[0].contains("401"));
    }
    // And a standard refusal on `401` without one.
    assert_eq!(
        located(compiled(
            &ir,
            "refusals:\n  not_granted: {status: 401, body: none}\nroutes: []\n"
        )),
        [refused("ESS-HTTP-027", "refusals.not_granted.headers")]
    );
    // A challenge refused for its scheme is not refused again as missing.
    assert_eq!(
        located(compiled(
            &ir,
            "refusals:\n  not_granted: {status: 401, body: none, headers: {WWW-Authenticate: {challenge: {scheme: 'a b'}}}}\nroutes: []\n"
        )),
        [refused(
            "ESS-HTTP-027",
            "refusals.not_granted.headers.WWW-Authenticate.challenge.scheme"
        )]
    );
    // A param written twice exactly.
    let text = header(&ir) + &read_private(BEARER);
    let mut json: Value = ess_yaml::from_str(&text).expect("YAML");
    json["routes"][0]["outcomes"]["token-refused"]["headers"]["WWW-Authenticate"]["challenge"]
        ["params"] = json!("PARAMS");
    let json = serde_json::to_string(&json).unwrap().replace(
        "\"PARAMS\"",
        r#"{"realm": {"literal": "a"}, "realm": {"literal": "b"}}"#,
    );
    let spec = HttpSpec::from_json(&json).expect("parses");
    assert_eq!(
        located(compile(&spec, &ir, &derived(&ir))),
        [refused(
            "ESS-HTTP-027",
            &format!("{at}.challenge.params.realm")
        )]
    );
}

#[test]
fn ess_http_028_a_redirect_without_3xx_or_3xx_without_redirect_is_refused() {
    let ir = model();
    let redirect = |outcome: &str| format!("routes[0].outcomes.{outcome}.redirect");
    for (outcome, written, path, named) in [
        // A redirect on a status that is not a redirect.
        (
            "approved",
            "{status: 200, redirect: {location: input.return_to, mode: query}}",
            redirect("approved"),
            "`200`",
        ),
        // A redirect status without a redirect.
        (
            "noted",
            "{status: 303, body: none}",
            redirect("noted"),
            "`303`",
        ),
        // A location that is not a `String` or a newtype of one, or is `Optional`.
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.count, mode: query}}",
            format!("{}.location", redirect("approved")),
            "String",
        ),
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.state, mode: query}}",
            format!("{}.location", redirect("approved")),
            "Optional",
        ),
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.maybe_uri, mode: query}}",
            format!("{}.location", redirect("approved")),
            "Optional",
        ),
        // A location that names no field, or that the outcome cannot supply.
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.nope, mode: query}}",
            format!("{}.location", redirect("approved")),
            "`nope`",
        ),
        (
            "noted",
            "{status: 302, body: none, redirect: {location: response.code, mode: query}}",
            format!("{}.location", redirect("noted")),
            "returns",
        ),
        (
            "approved",
            "{status: 302, body: none, redirect: {location: error.wire, mode: query}}",
            format!("{}.location", redirect("approved")),
            "location",
        ),
        // A mode outside the set, or none.
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.return_to, mode: hash}}",
            format!("{}.mode", redirect("approved")),
            "`hash`",
        ),
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.return_to}}",
            format!("{}.mode", redirect("approved")),
            "mode",
        ),
        // A parameter that names no field, names a non-scalar, or that the outcome cannot supply.
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.return_to, mode: query, parameters: {scope: input.scopes}}}",
            format!("{}.parameters.scope", redirect("approved")),
            "List",
        ),
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.return_to, mode: query, parameters: {x: input.nope}}}",
            format!("{}.parameters.x", redirect("approved")),
            "`nope`",
        ),
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.return_to, mode: query, parameters: {error: error.wire}}}",
            format!("{}.parameters.error", redirect("approved")),
            "no error",
        ),
        (
            "approved",
            "{status: 302, body: none, redirect: {location: input.return_to, mode: query, parameters: {x: 'input'}}}",
            format!("{}.parameters.x", redirect("approved")),
            "source",
        ),
    ] {
        let document = approve(&format!("      {outcome}: {written}\n"));
        let result = compiled(&ir, &document);
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-028", &path)],
            "{written}"
        );
        let shown = worded(result);
        assert!(shown[0].contains(named), "{written}: {shown:?}");
    }
}

#[test]
fn ess_http_028_a_parameter_twice_and_a_body_beside_a_redirect_are_refused() {
    let ir = model();
    // A parameter written twice.
    let text = header(&ir) + &approve(APPROVED);
    let mut json: Value = ess_yaml::from_str(&text).expect("YAML");
    json["routes"][0]["outcomes"]["approved"]["redirect"]["parameters"] = json!("PARAMETERS");
    let json = serde_json::to_string(&json).unwrap().replace(
        "\"PARAMETERS\"",
        r#"{"state": "input.state", "state": "input.client"}"#,
    );
    let spec = HttpSpec::from_json(&json).expect("parses");
    assert_eq!(
        located(compile(&spec, &ir, &derived(&ir))),
        [refused(
            "ESS-HTTP-028",
            "routes[0].outcomes.approved.redirect.parameters.state"
        )]
    );
    // A body other than `none` beside a redirect is `ESS-HTTP-024`'s, the envelope by default:
    // the page's example as written, without `body: none`, is refused.
    for body in ["", "body: envelope, ", "body: response, "] {
        let result = compiled(
            &ir,
            &approve(&format!(
                "      approved: {{status: 302, {body}redirect: {{location: input.return_to, mode: query}}}}\n"
            )),
        );
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-024", "routes[0].outcomes.approved.body")],
            "{body}"
        );
        assert!(worded(result)[0].contains("redirect"));
    }
}

#[test]
fn ess_http_028_a_redirect_on_a_refused_status_is_not_judged_and_parse_refusals_name_the_key() {
    let ir = model();
    // A redirect on a status `ESS-HTTP-023` refused is not judged again.
    assert_eq!(
        located(compiled(
            &ir,
            &approve(
                "      approved: {status: 300, body: none, redirect: {location: input.return_to, mode: query}}\n"
            )
        )),
        [refused("ESS-HTTP-023", "routes[0].outcomes.approved.status")]
    );
    // A missing `location` is a parse refusal naming the key, as a missing `status` is; an
    // unknown key in a redirect too.
    for (redirect, named) in [
        ("{mode: query}", "missing field `location`"),
        (
            "{location: input.return_to, mode: query, fragment: x}",
            "unknown field `fragment`",
        ),
    ] {
        let text = header(&ir)
            + &approve(&format!(
                "      approved: {{status: 302, body: none, redirect: {redirect}}}\n"
            ));
        let error = HttpSpec::from_yaml(&text).expect_err(named).to_string();
        assert!(error.contains(named), "{error}");
    }
}

#[test]
fn a_query_redirect_with_an_optional_state_parameter_compiles() {
    let ir = model();
    let http = compiled(
        &ir,
        &approve(&format!(
            "{APPROVED}      denied:
        status: 302
        body: none
        redirect:
          location: input.return_to
          mode: fragment
          parameters: {{state: input.state, error: error.wire}}
"
        )),
    )
    .expect("compiles");
    assert_eq!(
        outcomes_of(&http),
        json!({
            "approved": {"status": 302, "body": "none", "redirect": {
                "location": "input.return_to",
                "mode": "query",
                "parameters": {"code": "response.code", "state": "input.state"},
            }},
            "denied": {"status": 302, "body": "none", "redirect": {
                "location": "input.return_to",
                "mode": "fragment",
                "parameters": {"error": "error.wire", "state": "input.state"},
            }},
        })
    );
    let text = http.to_canonical_json();
    let read = HttpIr::from_json(&text).expect("the compiled IR reads back");
    assert_eq!(read.to_canonical_json(), text);
    // A location from a `String` field, and a redirect without parameters, compile; an empty
    // `parameters` writes nothing.
    let bare = compiled(
        &ir,
        &approve(
            "      approved: {status: 303, body: none, redirect: {location: input.client, mode: query}}\n",
        ),
    )
    .expect("compiles")
    .to_canonical_json();
    let empty = compiled(
        &ir,
        &approve(
            "      approved: {status: 303, body: none, redirect: {location: input.client, mode: query, parameters: {}}}\n",
        ),
    )
    .expect("compiles")
    .to_canonical_json();
    assert_eq!(empty, bare);
    assert!(!bare.contains("parameters"), "{bare}");
    // Every redirect status takes a redirect.
    for status in [301, 302, 303, 307, 308] {
        assert_eq!(
            located(compiled(
                &ir,
                &approve(&format!(
                    "      approved: {{status: {status}, body: none, redirect: {{location: input.return_to, mode: query}}}}\n"
                ))
            )),
            [],
            "{status}"
        );
    }
}

#[test]
fn ess_http_025_two_redirects_on_one_status_are_told_apart_by_the_parameters_they_always_write() {
    let ir = model();
    // Two redirects on one status are told apart by the parameters they always write (RFC 6749
    // §4.1.2 and §4.1.2.1): `approved` always writes `code`, which `denied` never writes. The
    // same holds for a redirect with no parameter beside one that always writes `code`.
    let bare = compiled(
        &ir,
        &approve(&format!(
            "{APPROVED}      denied: {{status: 302, body: none, redirect: {{location: input.return_to, mode: fragment}}}}\n"
        )),
    );
    assert_eq!(located(bare), []);
    // Always writing one name the other also always writes, both on `302`, tells nothing: the
    // required parameter names are equal, whatever the mode and the optional ones.
    let equal = compiled(
        &ir,
        &approve(&format!(
            "{APPROVED}      denied: {{status: 302, body: none, redirect: {{location: input.return_to, mode: fragment, parameters: {{code: error.wire, scope: input.maybe_uri}}}}}}\n"
        )),
    );
    assert_eq!(
        located(equal),
        [refused("ESS-HTTP-025", "routes[0].outcomes.approved")]
    );
    // Required names that differ only by a name the other may also write tell nothing either:
    // `denied` always writes `code` and `state`, and `approved` writes both when the request
    // carried `state`.
    let optional = compiled(
        &ir,
        &approve(&format!(
            "{APPROVED}      denied: {{status: 302, body: none, redirect: {{location: input.return_to, mode: query, parameters: {{code: error.wire, state: input.client}}}}}}\n"
        )),
    );
    assert_eq!(
        worded(optional),
        [
            "error[ESS-HTTP-025]: routes[0].outcomes.approved: outcome `approved` of command \
          `catalog.auth.ApproveOrder` answers `302` with `none`, and so does `denied` (`none`), \
          with a body a reader cannot tell apart; two outcomes on one status are told apart by \
          two `envelope` bodies, by `{<member>: error}` on one member with distinct error wire \
          names, or by two redirects, one always writing a parameter the other never \
          writes or both writing one parameter from `error.wire` with distinct error wire \
          names"
        ]
    );
    // A required name the other never writes tells them apart, though the other's are a subset.
    let superset = compiled(
        &ir,
        &approve(&format!(
            "{APPROVED}      denied: {{status: 302, body: none, redirect: {{location: input.return_to, mode: query, parameters: {{code: error.wire, error: error.wire}}}}}}\n"
        )),
    );
    assert_eq!(located(superset), []);
    // A redirect beside an outcome on the same status that declares none is still refused, by
    // `ESS-HTTP-028`, and that outcome is not counted again under `ESS-HTTP-025`.
    let missing = compiled(
        &ir,
        &approve(&format!(
            "{APPROVED}      denied: {{status: 302, body: none}}\n"
        )),
    );
    assert_eq!(
        located(missing),
        [refused(
            "ESS-HTTP-028",
            "routes[0].outcomes.denied.redirect"
        )]
    );
}

/// A protocol binding fixing the challenge and the redirect.
fn protocol(ir: &EssIr) -> String {
    header(ir)
        + &approve(&format!(
            "{APPROVED}      denied:
        status: 403
        headers: {{Cache-Control: {{literal: no-store}}}}
"
        ))
}

#[test]
fn ess_http_012_a_refinement_restates_headers_and_redirects_exactly() {
    let ir = model();
    let refined = Refined::from_yaml(protocol(&ir).as_bytes()).expect("parses");
    let refining = |document: String| {
        let text = document.replacen(
            "routes:",
            &format!("refines: {}\nroutes:", refined.digest()),
            1,
        );
        let spec = HttpSpec::from_yaml(&text).expect("parses");
        compile_refining(&spec, Some(&refined), &ir, &derived(&ir))
    };
    // Restated with parameters in another order and an empty `headers` written out, compiles.
    let restated = protocol(&ir)
        .replace(
            "            code: response.code\n            state: input.state\n",
            "            state: input.state\n            code: response.code\n",
        )
        .replace(
            "        body: none\n        redirect:",
            "        body: none\n        headers: {}\n        redirect:",
        );
    assert_ne!(restated, protocol(&ir));
    assert_eq!(located(refining(restated)), []);
    // A header changed, a redirect parameter dropped, a mode changed.
    let changed = protocol(&ir)
        .replace("{literal: no-store}", "{literal: no-cache}")
        .replace("            state: input.state\n", "")
        .replace("mode: query", "mode: fragment");
    let result = refining(changed);
    assert_eq!(
        located(result.clone()),
        [
            refused("ESS-HTTP-012", "routes[0].outcomes.approved.redirect"),
            refused("ESS-HTTP-012", "routes[0].outcomes.denied.headers"),
        ]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("fragment") && shown[0].contains("query"),
        "{shown:?}"
    );
    assert!(
        shown[1].contains("no-cache") && shown[1].contains("no-store"),
        "{shown:?}"
    );
    // Headers added where the refined binding had none.
    let added = protocol(&ir).replace(
        "        body: none\n        redirect:",
        "        body: none\n        headers: {Pragma: {literal: no-cache}}\n        redirect:",
    );
    assert_eq!(
        located(refining(added)),
        [refused(
            "ESS-HTTP-012",
            "routes[0].outcomes.approved.headers"
        )]
    );
}

#[test]
fn ess_http_012_headers_and_redirects_compare_as_values_not_as_text() {
    let ir = model();
    // Values are compared as values, not as text: one literal whose text spells two headers is
    // not the two headers, and a redirect parameter whose text spells two is not the two.
    let spelled = |headers: &str, parameters: &str| {
        header(&ir)
            + &approve(&format!(
                "      approved:
        status: 302
        body: none
        redirect: {{location: input.return_to, mode: query, parameters: {parameters}}}
      denied:
        status: 403
        headers: {headers}
"
            ))
    };
    let one = spelled(
        "{Cache-Control: {literal: 'a}, Pragma: {literal: b'}}",
        "{code: 'response.code, state: input.state'}",
    );
    let refined_one = Refined::from_yaml(one.as_bytes()).expect("parses");
    let two = spelled(
        "{Cache-Control: {literal: a}, Pragma: {literal: b}}",
        "{code: response.code, state: input.state}",
    )
    .replacen(
        "routes:",
        &format!("refines: {}\nroutes:", refined_one.digest()),
        1,
    );
    let spec = HttpSpec::from_yaml(&two).expect("parses");
    let shown: Vec<(&str, String)> = located(compile_refining(
        &spec,
        Some(&refined_one),
        &ir,
        &derived(&ir),
    ))
    .into_iter()
    .filter(|(code, _)| *code == "ESS-HTTP-012")
    .collect();
    assert_eq!(
        shown,
        [
            refused("ESS-HTTP-012", "routes[0].outcomes.approved.redirect"),
            refused("ESS-HTTP-012", "routes[0].outcomes.denied.headers"),
        ]
    );
}

#[test]
fn ess_http_012_a_refinement_restates_a_refusals_headers_exactly() {
    let ir = model();
    // A standard refusal's headers.
    let with_refusal = |headers: &str| {
        header(&ir)
            + &format!("refusals:\n  malformed: {{status: 400, body: none{headers}}}\nroutes: []\n")
    };
    let refused_binding = Refined::from_yaml(
        with_refusal(", headers: {Cache-Control: {literal: no-store}}").as_bytes(),
    )
    .expect("parses");
    let text = with_refusal("").replacen(
        "refusals:",
        &format!("refines: {}\nrefusals:", refused_binding.digest()),
        1,
    );
    let spec = HttpSpec::from_yaml(&text).expect("parses");
    assert_eq!(
        located(compile_refining(
            &spec,
            Some(&refused_binding),
            &ir,
            &derived(&ir)
        )),
        [refused("ESS-HTTP-012", "refusals.malformed.headers")]
    );
}

#[test]
fn compiled_ir_with_a_header_or_redirect_compiling_refuses_is_refused() {
    type Mutation = fn(&mut Value);
    let ir = model();
    let http = compiled(&ir, &approve(APPROVED)).expect("compiles");
    let canonical = ir_json(&http);
    let cases: [(&str, Mutation); 13] = [
        ("`headers` is empty", |json| {
            json["routes"][0]["outcomes"]["approved"]["headers"] = json!({});
        }),
        ("`parameters` is empty", |json| {
            json["routes"][0]["outcomes"]["approved"]["redirect"]["parameters"] = json!({});
        }),
        ("token", |json| {
            json["routes"][0]["outcomes"]["approved"]["headers"] = json!({"X Y": {"literal": "a"}});
        }),
        ("Location", |json| {
            json["routes"][0]["outcomes"]["approved"]["headers"] =
                json!({"Location": {"literal": "a"}});
        }),
        ("twice", |json| {
            json["routes"][0]["outcomes"]["approved"]["headers"] =
                json!({"Pragma": {"literal": "a"}, "pragma": {"literal": "b"}});
        }),
        ("field value", |json| {
            json["routes"][0]["outcomes"]["approved"]["headers"] =
                json!({"Pragma": {"literal": "a\nb"}});
        }),
        ("WWW-Authenticate", |json| {
            json["routes"][0]["outcomes"]["approved"]["headers"] =
                json!({"Pragma": {"challenge": {"scheme": "Bearer"}}});
        }),
        ("`redirect`", |json| {
            json["routes"][0]["outcomes"]["approved"]["status"] = json!(200);
        }),
        ("`redirect`", |json| {
            let outcome = json["routes"][0]["outcomes"]["approved"]
                .as_object_mut()
                .unwrap();
            outcome.remove("redirect");
        }),
        ("`none`", |json| {
            let outcome = json["routes"][0]["outcomes"]["approved"]
                .as_object_mut()
                .unwrap();
            outcome.remove("body");
        }),
        ("hash", |json| {
            json["routes"][0]["outcomes"]["approved"]["redirect"]["mode"] = json!("hash");
        }),
        ("source", |json| {
            json["routes"][0]["outcomes"]["approved"]["redirect"]["location"] = json!("param.x");
        }),
        ("401", |json| {
            json["routes"][0]["outcomes"]["approved"] = json!({"status": 401});
        }),
    ];
    for (named, mutate) in cases {
        let mut json = canonical.clone();
        mutate(&mut json);
        let text = serde_json::to_string(&json).expect("serializes");
        let error = HttpIr::from_json(&text).expect_err(named).to_string();
        assert!(error.contains(named), "{named}: {error}");
    }
    // A refusal's header reads no source.
    let http = compiled(
        &ir,
        "refusals:\n  malformed: {status: 400, body: none, headers: {Pragma: {literal: no-cache}}}\nroutes: []\n",
    )
    .expect("compiles");
    let mut json = ir_json(&http);
    json["refusals"]["malformed"]["headers"]["Pragma"] = json!({"source": "input.item_id"});
    let error = HttpIr::from_json(&json.to_string())
        .expect_err("a refusal's source")
        .to_string();
    assert!(error.contains("source"), "{error}");
}

/// The header fixture with `edit` applied to its text, compiled.
fn model_edited(edit: impl Fn(&str) -> String) -> EssIr {
    let text = edit(MODEL);
    assert_ne!(text, MODEL, "the edit changes the fixture");
    let raw = RawSpecFile::parse(&text).expect("well formed");
    let spec = Specification::assemble([(Source::new("auth.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("auth.yaml", &text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The fixture with `TokenRefused`'s wire name `wire`, a YAML double-quoted scalar.
fn model_with_token_wire(wire: &str) -> EssIr {
    model_edited(|text| {
        text.replacen(
            "naming: {wire: invalid_token}",
            &format!("naming: {{wire: \"{wire}\"}}"),
            1,
        )
    })
}

/// The error's wire name is known when the binding compiles, so a header reading `error.wire` is
/// held to the field-value rule a literal holding the same text is (`ESS-HTTP-026`), naming the
/// wire name; a param reading it, to the quoted-string rule (`ESS-HTTP-027`).
#[test]
fn ess_http_026_a_header_reading_an_error_wire_name_is_held_to_the_field_value_rule() {
    for (wire, named) in [
        ("invalid_token\\r\\nSet-Cookie: a=b", "'\\r'"),
        ("invalid_token ", "space"),
        ("jeton_refusé", "'é'"),
    ] {
        let ir = model_with_token_wire(wire);
        let result = compiled(
            &ir,
            &read_private(&format!(
                "{BEARER}          X-Error: {{source: error.wire}}\n"
            )),
        );
        let located_at = located(result.clone());
        assert!(
            located_at.contains(&refused(
                "ESS-HTTP-026",
                "routes[0].outcomes.token-refused.headers.X-Error"
            )),
            "{wire}: {located_at:?}"
        );
        let shown = worded(result);
        let header = shown
            .iter()
            .find(|line| line.contains("headers.X-Error:"))
            .unwrap_or_else(|| panic!("{wire}: {shown:?}"));
        assert!(
            header.contains("wire name") && header.contains(named),
            "{wire}: {header}"
        );
    }
    // A wire name that is a field value compiles, the page's example.
    let ir = model();
    assert_eq!(
        located(compiled(
            &ir,
            &read_private(&format!(
                "{BEARER}          X-Error: {{source: error.wire}}\n"
            ))
        )),
        []
    );
}

#[test]
fn ess_http_027_a_challenge_param_reading_an_error_wire_name_no_quoted_string_carries_is_refused_naming_it(
) {
    let at = "routes[0].outcomes.token-refused.headers.WWW-Authenticate.challenge.params.error";
    for wire in ["jeton_refusé", "invalid_token\\nX: y"] {
        let ir = model_with_token_wire(wire);
        let result = compiled(&ir, &read_private(BEARER));
        assert_eq!(
            located(result.clone()),
            [refused("ESS-HTTP-027", at)],
            "{wire}"
        );
        let shown = worded(result);
        assert!(
            shown[0].contains("wire name") && shown[0].contains("quoted string"),
            "{wire}: {shown:?}"
        );
    }
    // `"` and `\` are visible US-ASCII, which the generator escapes: a quoted string carries them.
    let ir = model_with_token_wire("invalid \\\"token\\\\");
    assert_eq!(located(compiled(&ir, &read_private(BEARER))), []);
}

/// The fixture with an `invalid_scope` error and two more error outcomes on `ApproveOrder`:
/// `scope-refused` reporting it and `denied-again` reporting `access_denied` as `denied` does.
fn model_with_scope_refused() -> EssIr {
    model_edited(|text| {
        text.replacen(
            "    naming: {wire: access_denied}\n",
            "    naming: {wire: access_denied}\n  - name: catalog.auth.InvalidScope\n    naming: {wire: invalid_scope}\n",
            1,
        )
        .replacen(
            "      - name: wired\n",
            "      - name: scope-refused\n        when: count == 0\n        error: catalog.auth.InvalidScope\n      - name: denied-again\n        when: count == 1\n        error: catalog.auth.AccessDenied\n      - name: wired\n",
            1,
        )
    })
}

/// Every authorization error redirects to the client with its own `error` (RFC 6749 §4.1.2.1), so
/// two redirects on one status that both always write one parameter from `error.wire` are told
/// apart when their error wire names differ, as two `{<member>: error}` bodies are
/// (`ESS-HTTP-025`).
#[test]
fn ess_http_025_two_redirects_writing_the_error_wire_name_under_one_parameter_are_told_apart_by_it()
{
    let ir = model_with_scope_refused();
    let redirect = |outcome: &str, parameters: &str| {
        format!(
            "      {outcome}: {{status: 302, body: none, redirect: {{location: input.return_to, mode: query, parameters: {parameters}}}}}\n"
        )
    };
    // `access_denied` and `invalid_scope` on one `302`, both writing `error` from `error.wire`.
    let apart = compiled(
        &ir,
        &approve(&format!(
            "{}{}",
            redirect("denied", "{error: error.wire}"),
            redirect("scope-refused", "{error: error.wire, state: input.state}")
        )),
    );
    assert_eq!(located(apart), []);
    // One error, so one wire name: the two cannot be told apart.
    let same = compiled(
        &ir,
        &approve(&format!(
            "{}{}",
            redirect("denied", "{error: error.wire}"),
            redirect("denied-again", "{error: error.wire}")
        )),
    );
    assert_eq!(
        located(same),
        [refused("ESS-HTTP-025", "routes[0].outcomes.denied")]
    );
    // The name is written from `error.wire` by both, or it tells nothing: `input.client` may hold
    // either wire name.
    let one_sided = compiled(
        &ir,
        &approve(&format!(
            "{}{}",
            redirect("denied", "{error: error.wire}"),
            redirect("scope-refused", "{error: input.client}")
        )),
    );
    assert_eq!(
        located(one_sided),
        [refused("ESS-HTTP-025", "routes[0].outcomes.denied")]
    );
}

/// A redirect parameter's name is written into the location's query or fragment as it is, so it is
/// one or more RFC 3986 `unreserved` characters, which no encoding changes (`ESS-HTTP-028`).
#[test]
fn ess_http_028_a_redirect_parameter_name_outside_rfc_3986_unreserved_is_refused() {
    let ir = model();
    for name in [
        "''", "'a b'", "'a&b'", "'a=b'", "'a#b'", "'a%20'", "'a+b'", "état",
    ] {
        let result = compiled(
            &ir,
            &approve(&format!(
                "      approved: {{status: 302, body: none, redirect: {{location: input.return_to, mode: query, parameters: {{{name}: input.client}}}}}}\n"
            )),
        );
        let bare = name.trim_matches('\'');
        assert_eq!(
            located(result.clone()),
            [refused(
                "ESS-HTTP-028",
                &format!("routes[0].outcomes.approved.redirect.parameters.{bare}")
            )],
            "{name}"
        );
        let shown = worded(result);
        assert!(shown[0].contains("unreserved"), "{name}: {shown:?}");
    }
    let http = compiled(
        &ir,
        &approve(
            "      approved: {status: 302, body: none, redirect: {location: input.return_to, mode: query, parameters: {Az-09._~: input.client}}}\n",
        ),
    )
    .expect("an unreserved name compiles");
    // The IR reader refuses the same name without the specification.
    let mut json = ir_json(&http);
    json["routes"][0]["outcomes"]["approved"]["redirect"]["parameters"] =
        json!({"a b": "input.client"});
    let error = HttpIr::from_json(&json.to_string())
        .expect_err("a parameter name outside unreserved")
        .to_string();
    assert!(error.contains("unreserved"), "{error}");
}

/// `error.wire` is the error's wire name, never the absolute URI a `location` holds
/// (`ESS-HTTP-028`), though it is a `String`.
#[test]
fn ess_http_028_a_location_reading_the_error_wire_name_is_refused() {
    let ir = model();
    let result = compiled(
        &ir,
        &approve(
            "      denied: {status: 302, body: none, redirect: {location: error.wire, mode: query}}\n",
        ),
    );
    assert_eq!(
        located(result.clone()),
        [refused(
            "ESS-HTTP-028",
            "routes[0].outcomes.denied.redirect.location"
        )]
    );
    let shown = worded(result);
    assert!(
        shown[0].contains("wire name") && shown[0].contains("absolute URI"),
        "{shown:?}"
    );
    // The IR reader refuses it without the specification.
    let http = compiled(&ir, &approve(APPROVED)).expect("compiles");
    let mut json = ir_json(&http);
    json["routes"][0]["outcomes"]["approved"]["redirect"]["location"] = json!("error.wire");
    let error = HttpIr::from_json(&json.to_string())
        .expect_err("a location reading error.wire")
        .to_string();
    assert!(error.contains("error.wire"), "{error}");
}

/// `Content-Encoding` and `Content-Length` describe the encoded content and `Connection` the
/// connection, which the server writes; a binding declares none of them, in any case
/// (`ESS-HTTP-026`), and the IR reader refuses each.
#[test]
fn ess_http_026_content_encoding_content_length_and_connection_are_reserved() {
    let ir = model();
    let http = compiled(&ir, &approve(APPROVED)).expect("compiles");
    for name in [
        "Content-Encoding",
        "content-encoding",
        "Content-Length",
        "Connection",
        "CONNECTION",
    ] {
        let result = compiled(
            &ir,
            &read_private(&format!(
                "      read: {{status: 200, headers: {{{name}: {{literal: close}}}}}}\n"
            )),
        );
        assert_eq!(
            located(result.clone()),
            [refused(
                "ESS-HTTP-026",
                &format!("routes[0].outcomes.read.headers.{name}")
            )],
            "{name}"
        );
        let mut json = ir_json(&http);
        json["routes"][0]["outcomes"]["approved"]["headers"] = json!({name: {"literal": "close"}});
        HttpIr::from_json(&json.to_string()).expect_err(name);
    }
}
