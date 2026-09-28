//! Adversary pass 1 against the `retry:` block reader (ess/16, beyond10x/ess#165).
//!
//! At the base `on_failure: {retry: <anything>}` was refused while the document was read: only
//! `escalate` took a block. The new reader reads `retry:`'s value as `Option<RetryBound>`, so an
//! empty value (`retry:` followed by nothing, YAML null) is read as "no bound" and admitted as a
//! bare `retry` — while the published schema's third spelling requires an object under `retry`.

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/bounded-retry.yaml");
const BLOCK: &str = "retry: {attempts: 3, final: [demo.ledger.Unknown]}";

fn with_policy(policy: &str) -> String {
    let model = MODEL.replace(BLOCK, policy);
    assert_ne!(model, MODEL);
    model
}

fn parse(body: &str) -> Result<ess_domain::spec::RawSpecFile, String> {
    ess_domain::spec::RawSpecFile::parse(body).map_err(|error| error.to_string())
}

fn assemble(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = parse(body)?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("ledger.yaml"), raw)])
        .map_err(|errors| errors.to_string())
}

/// `retry:` with an empty value is neither the bare word nor a bound; the base refused it and the
/// schema still does.
#[test]
fn adversary_retry_an_empty_retry_block_is_refused_as_it_was() {
    for empty in ["retry: null", "retry: ~", "retry:"] {
        // The fixture writes the block as a mapping under `on_failure:`, so this is
        // `on_failure: {retry: null}` and friends.
        let model = with_policy(empty);
        assert!(
            parse(&model).is_err(),
            "`on_failure: {{{empty}}}` is refused while the document is read, as at the base and \
             as the published schema says (the third spelling requires an object under `retry`)"
        );
    }
}

/// Boundary: the smallest bound is admitted (the refusal is `< 2`, not `<= 2`).
#[test]
fn adversary_retry_two_attempts_is_the_smallest_admitted_bound() {
    assemble(&with_policy("retry: {attempts: 2}"))
        .unwrap_or_else(|errors| panic!("`attempts: 2` is admitted: {errors}"));
}

/// An empty `final:` list states no final refusal, and serializes as a bound without one.
#[test]
fn adversary_retry_an_empty_final_list_is_a_bound_without_finals() {
    let with = assemble(&with_policy("retry: {attempts: 3, final: []}")).expect("admitted");
    let without = assemble(&with_policy("retry: {attempts: 3}")).expect("admitted");
    let name = ess_domain::binding::BindingName::new("notify-ledger").unwrap();
    assert_eq!(
        serde_json::to_value(&with.bindings()[&name]).unwrap(),
        serde_json::to_value(&without.bindings()[&name]).unwrap()
    );
}
