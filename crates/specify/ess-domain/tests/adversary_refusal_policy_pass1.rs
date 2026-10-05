//! Adversary pass 1 against the refusal-selected failure policy unit (ess/22, beyond10x/ess#269):
//! `final` reached through an alias, which the unit's own tests name only by outcome.
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/refusal-policy.yaml");
const POLICY: &str = "    on_failure:
      drop: [wrong-state]
      retry: {outcomes: [demo.ledger.Unavailable, rejected], attempts: 3, final: [rejected]}
      escalate:
        emits: demo.ledger.RecordEscalated
        except: [wrong-state, demo.ledger.Unavailable, rejected]
";

fn assemble(policy: &str) -> Result<Specification, ValidationErrors> {
    let body = MODEL.replace(POLICY, &format!("    on_failure:\n{policy}"));
    assert_ne!(body, MODEL);
    let raw = ess_domain::spec::RawSpecFile::parse(&body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("ledger.yaml"), raw)])
}

/// `final: [demo.ledger.Unavailable]` expands to `unavailable` and `busy`; the retry selects only
/// `unavailable`, `busy` is dropped. "Final aliases must resolve wholly inside that retry policy's
/// selected set; otherwise validation refuses them."
#[test]
fn adversary_a_final_alias_reaching_outside_the_retry_selection_is_refused() {
    let errors = assemble(
        "      drop: [busy, wrong-state]\n      retry: {outcomes: [unavailable, rejected], attempts: 3, final: [demo.ledger.Unavailable]}\n      escalate: {emits: demo.ledger.RecordEscalated, except: [busy, wrong-state, unavailable, rejected]}\n",
    )
    .expect_err("`busy` is final for a retry that does not select it");
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::ConflictingDeclaration
                && error.to_string().contains("`busy`")
                && error.to_string().contains("final")
        }),
        "{errors}"
    );
}

/// The same alias wholly inside the selection, with the retry as the fallback: admitted.
#[test]
fn adversary_a_final_alias_inside_a_fallback_retry_selection_is_admitted() {
    assemble(
        "      drop: [wrong-state, at-limit]\n      retry: {except: [wrong-state, at-limit], attempts: 3, final: [demo.ledger.Unavailable]}\n",
    )
    .unwrap_or_else(|errors| panic!("admitted: {errors}"));
}
