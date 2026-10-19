//! A guard reads a fact through a digest or encoding function (`ess/27`,
//! `story:digest-and-encoding-functions-in-expressions`, beyond10x/ess#584):
//! `challenge != base64url(sha256(ascii(input.verifier)))`, RFC 7636 PKCE `S256`.
//!
//! The reproduction validates below `ess/27` only with the check left to an `external:` cause; from
//! `ess/27` the transform is a held-row predicate. Each function is typed: `sha256` takes a String
//! (its UTF-8 bytes) or Bytes, `base64url` Bytes, `ascii` a String, and a misuse is refused naming
//! the type it takes. A function of a fact is a command guard's operand; elsewhere it is refused by
//! name.

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/digest-guard.yaml");

const TRANSFORM: &str = "challenge != base64url(sha256(ascii(input.verifier)))";

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("pkce.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

fn at(format: &str, body: &str) -> String {
    body.replacen("format: ess/27", &format!("format: {format}"), 1)
}

/// The reproduction with the check left to the implementation: the `mismatch` branch an
/// `external:` refusal, as the starting release has to write it.
fn with_external_cause(body: &str) -> String {
    body.replace(
        "      - name: mismatch
        when_subject:
          predicate:
            all:
              - method == S256
              - challenge != base64url(sha256(ascii(input.verifier)))
        error: demo.pkce.InvalidVerifier
",
        "      - name: mismatch
        external: the S256 transform of the verifier differs from the stored challenge
        error: demo.pkce.InvalidVerifier
",
    )
    // The reproduction is the S256 check alone: beside a held-state refusal declared after it, the
    // stand-in is refused for the order it would answer in (beyond10x/ess#584, attempt 2).
    .replace(
        "      - name: redirect-mismatch
        when_subject: {predicate: redirect_uri != input.redirect_uri}
        error: demo.pkce.RedirectMismatch
",
        "",
    )
}

#[test]
fn the_reproduction_validates_below_ess_27_only_with_an_external_cause() {
    spec(&with_external_cause(&at("ess/26", MODEL))).unwrap_or_else(|errors| panic!("{errors}"));
    let errors = refused(&at("ess/26", MODEL));
    assert!(
        errors.contains("unsupported_format_version") && errors.contains("ess/27"),
        "{errors}"
    );
}

#[test]
fn the_transform_validates_in_when_subject_from_ess_27() {
    spec(MODEL).unwrap_or_else(|errors| panic!("{errors}"));
    // Without `ascii`, `sha256` reads the String's UTF-8 bytes.
    spec(&MODEL.replace(TRANSFORM, "challenge != base64url(sha256(input.verifier))"))
        .unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn a_misused_function_is_refused_naming_the_type_it_takes() {
    for (written, says) in [
        (
            "challenge != base64url(input.verifier)",
            "`base64url` takes Bytes",
        ),
        (
            "challenge != base64url(ascii(sha256(input.verifier)))",
            "`ascii` takes a String",
        ),
        ("challenge != sha256(input.verifier)", "Bytes"),
    ] {
        let errors = refused(&MODEL.replace(TRANSFORM, written));
        assert!(
            errors.contains("type_mismatch") && errors.contains(says),
            "{written}: {errors}"
        );
    }
    let integer = MODEL.replace(
        "      - {name: verifier, type: String}",
        "      - {name: verifier, type: Integer}",
    );
    let errors = refused(&integer);
    assert!(errors.contains("`ascii` takes a String"), "{errors}");
}

#[test]
fn a_function_outside_a_command_guard_is_refused_by_name() {
    let invariant = MODEL.replace(
        "      - {name: redirect_uri, type: String}
    lifecycle:",
        "      - {name: redirect_uri, type: String}
    invariants:
      - challenge != base64url(sha256(ascii(redirect_uri)))
    lifecycle:",
    );
    let errors = refused(&invariant);
    assert!(
        errors.contains("unsupported_construct") && errors.contains("command outcome's guards"),
        "{errors}"
    );
}

#[test]
fn a_quoted_call_is_the_text_it_spells_in_every_format() {
    let quoted = MODEL.replace(
        TRANSFORM,
        "challenge != \"base64url(sha256(ascii(input.verifier)))\"",
    );
    spec(&quoted).unwrap_or_else(|errors| panic!("{errors}"));
    spec(&at("ess/26", &quoted)).unwrap_or_else(|errors| panic!("{errors}"));
}
