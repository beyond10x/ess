//! Adversary, pass 2, for a branch guarded by a row of another entity (`when_related:`, ess/18,
//! beyond10x/ess#211), domain half: several predicate branches over one related row.
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const SIGN_IN: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-sign-in.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(ess_domain::system::Source::new("sign-in.yaml"), raw)])
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// The fixture with the configuration on a plan and in a region, and the predicate refusal
/// replaced by `refusals`.
fn with_refusals(refusals: &str) -> String {
    let text = replaced(
        SIGN_IN,
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.signin.SignInId, kind: newtype, of: Uuid}\n  - {name: demo.signin.Plan, kind: enum, variants: [Basic, Premium]}\n  - {name: demo.signin.Region, kind: enum, variants: [North, South]}\n",
    );
    let text = replaced(
        &text,
        "      - {name: redirect_client, type: demo.signin.ClientId}\n    lifecycle",
        "      - {name: redirect_client, type: demo.signin.ClientId}\n      - {name: plan, type: demo.signin.Plan}\n      - {name: region, type: demo.signin.Region}\n    lifecycle",
    );
    let text = replaced(
        &text,
        "  - {name: demo.signin.NoRedirectEntry, summary: The configuration does not register the client., fields: []}\n",
        "  - {name: demo.signin.NoRedirectEntry, summary: The configuration does not register the client., fields: []}\n  - {name: demo.signin.Northern, summary: Northern tenants sign in elsewhere., fields: []}\n",
    );
    replaced(
        &text,
        "      - name: no-redirect-entry\n        when_related: {via: input.tenant, predicate: redirect_client != input.client}\n        error: demo.signin.NoRedirectEntry\n",
        refusals,
    )
}

/// Two predicate refusals over one related row, both true for a basic configuration in the north:
/// on that row two branches answer and nothing states which. Over enum fields alone the joint
/// partition is finite, so the domain must see the overlap.
#[test]
fn adversary_pass2_two_related_predicate_refusals_true_on_one_row_are_refused() {
    let text = with_refusals(
        "      - name: basic
        when_related: {via: input.tenant, predicate: plan == Basic}
        error: demo.signin.NoRedirectEntry
      - name: northern
        when_related: {via: input.tenant, predicate: region == North}
        error: demo.signin.Northern
",
    );
    match assemble(&text) {
        Ok(_) => panic!(
            "admitted: a Basic configuration in the North selects both `basic` and `northern`"
        ),
        Err(errors) => assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::ConflictingDeclaration),
            "{errors}"
        ),
    }
}

/// The same two refusals made disjoint are admitted: the partition check above is about overlap,
/// not about two predicate branches as such.
#[test]
fn adversary_pass2_two_disjoint_related_predicate_refusals_are_admitted() {
    let text = with_refusals(
        "      - name: basic
        when_related: {via: input.tenant, predicate: plan == Basic}
        error: demo.signin.NoRedirectEntry
      - name: northern
        when_related: {via: input.tenant, predicate: {all: [plan == Premium, region == North]}}
        error: demo.signin.Northern
",
    );
    if let Err(errors) = assemble(&text) {
        panic!("disjoint predicate refusals are admitted:\n{errors}");
    }
}
