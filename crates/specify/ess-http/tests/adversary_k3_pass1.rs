//! Adversarial cases for the response headers, challenges and redirects of an `ess-http/1`
//! outcome, held to `docs/design/http-presentation-binding.md`, "Response headers" and
//! "Redirects", and to `compile`'s own table of refusals (`ESS-HTTP-026`, `-027`).

use ess_compiler::resolve::compile as compile_model;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_http::{compile, DerivedRoute, HttpDiagnostics, HttpIr, HttpSpec, Method, Operation};

const MODEL: &str = include_str!("fixtures/auth.yaml");

/// The header fixture with `TokenRefused`'s wire name replaced by `wire`, a YAML double-quoted
/// scalar.
fn model_with_wire(wire: &str) -> EssIr {
    let text = MODEL.replacen(
        "naming: {wire: invalid_token}",
        &format!("naming: {{wire: \"{wire}\"}}"),
        1,
    );
    assert_ne!(text, MODEL);
    let raw = RawSpecFile::parse(&text).expect("well formed");
    let spec = Specification::assemble([(Source::new("auth.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("auth.yaml", &text);
    compile_model(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

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

fn compiled(ir: &EssIr, document: &str) -> Result<HttpIr, HttpDiagnostics> {
    let text = format!(
        "type: ess-http/1\nspecification:\n  system: catalog\n  version: v1\n  source_digest: sha256:{}\n{document}",
        ir.source_digest()
    );
    let spec = HttpSpec::from_yaml(&text).unwrap_or_else(|error| panic!("parses: {error}\n{text}"));
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

/// A `ReadPrivateItem` route listing `outcomes`.
fn read_private(outcomes: &str) -> String {
    format!(
        "routes:\n  - command: catalog.auth.ReadPrivateItem\n    method: get\n    path: /private/items/{{item_id}}\n    parameters: {{item_id: input.item_id}}\n    outcomes:\n{outcomes}"
    )
}

/// The error's wire name is known when the binding compiles, and `error.wire` always names it
/// (answer 19). A literal header holding the same text is refused as no field value
/// (`ESS-HTTP-026`), so a header reading it must be too: a CR LF in it ends the header line and
/// starts another, the injection the field-value rule exists to stop.
#[test]
fn ess_http_026_a_header_reading_an_error_wire_name_that_is_no_field_value_is_refused() {
    let ir = model_with_wire("invalid_token\\r\\nSet-Cookie: session=attacker");
    // The same text as a literal is refused.
    let literal = compiled(
        &ir,
        &read_private(
            "      token-refused:
        status: 400
        headers:
          X-Error: {literal: \"invalid_token\\r\\nSet-Cookie: session=attacker\"}
",
        ),
    );
    assert_eq!(
        located(literal),
        [(
            "ESS-HTTP-026",
            "routes[0].outcomes.token-refused.headers.X-Error".to_owned()
        )]
    );
    // Read through `error.wire`, it compiles.
    let sourced = compiled(
        &ir,
        &read_private(
            "      token-refused:
        status: 400
        headers:
          X-Error: {source: error.wire}
",
        ),
    );
    assert_eq!(
        located(sourced),
        [(
            "ESS-HTTP-026",
            "routes[0].outcomes.token-refused.headers.X-Error".to_owned()
        )]
    );
}

/// The page says "the generator writes the quoting, so an error wire name cannot break out of the
/// challenge". A quoted string carries no CR, LF or non-ASCII text (RFC 9110 §5.6.4), so no quoting
/// can hold such a wire name, and a literal param holding it is refused (`ESS-HTTP-027`); the
/// `error` param of the page's own `Bearer` example reading it must be too.
#[test]
fn ess_http_027_a_challenge_param_reading_an_error_wire_name_no_quoted_string_carries_is_refused() {
    for wire in [
        "invalid_token\\r\\nSet-Cookie: session=attacker",
        "jeton_refusé",
    ] {
        let ir = model_with_wire(wire);
        let bearer = |error: &str| {
            format!(
                "      token-refused:
        status: 401
        body: {{error: error}}
        headers:
          WWW-Authenticate:
            challenge:
              scheme: Bearer
              params:
                realm: {{literal: catalog}}
                error: {error}
"
            )
        };
        let at = "routes[0].outcomes.token-refused.headers.WWW-Authenticate.challenge.params.error"
            .to_owned();
        // The same text as a literal param is refused.
        assert_eq!(
            located(compiled(
                &ir,
                &read_private(&bearer(&format!("{{literal: \"{wire}\"}}")))
            )),
            [("ESS-HTTP-027", at.clone())],
            "{wire}"
        );
        // Read through `error.wire`, it compiles. Either code a param source is refused under
        // (`-026` for its source, `-027` for its text) would do; refused it must be.
        let sourced = located(compiled(
            &ir,
            &read_private(&bearer("{source: error.wire}")),
        ));
        assert!(
            matches!(
                sourced.as_slice(),
                [(code, path)] if ["ESS-HTTP-026", "ESS-HTTP-027"].contains(code) && *path == at
            ),
            "{wire}: {sourced:?}"
        );
    }
}
