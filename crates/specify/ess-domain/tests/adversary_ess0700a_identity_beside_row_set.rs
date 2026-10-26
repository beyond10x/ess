//! Adversary pass 1, ESS 0.70.0 unit A (beyond10x/ess#554): an identity-addressed `when_related:`
//! beside a row set, admitted from `ess/32`.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const REPRO: &str = include_str!("fixtures/identity-beside-row-set.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("wiring.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn edited(text: &str, before: &str, after: &str) -> String {
    let changed = text.replacen(before, after, 1);
    assert_ne!(changed, text, "`{before}` is in the model");
    changed
}

const CONNECTED: &str = "      - name: connected\n        creates: wiring.board.Wire\n";

/// A `when_subject` guard beside the row set and the named rows keeps its refusal at `ess/32`,
/// and that refusal names the `when_subject` guard, the construct `ess/32` still refuses. Naming
/// the identity-addressed `when_related:`, which `ess/32` admits, tells the author to remove the
/// guard the format allows.
#[test]
fn adversary_a_when_subject_beside_identity_and_row_set_is_named_at_ess_32() {
    let subject = "      - name: odd
        when_subject:
          predicate: from_port == input.from_port
        error: wiring.board.SourceTaken
";
    let text = edited(REPRO, CONNECTED, &format!("{subject}{CONNECTED}"));
    let errors = assemble(&text).expect_err("a `when_subject` beside a row set is refused");
    let refused: Vec<_> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::UnsupportedConstruct)
        .collect();
    assert!(!refused.is_empty(), "{}", listed(&errors));
    for error in &refused {
        assert!(
            error.message.contains("a `when_subject` guard"),
            "the refusal names the guard ess/32 refuses: {}\n{}",
            error.message,
            listed(&errors)
        );
    }
}

/// An accepting row-set branch declared above a refusal over a present named row: one request (a
/// wrong-facing source, no wire at the sink) satisfies both, the rule answers the refusal at
/// step 4 and the accepting branch after it, so from `ess/29` the declaration is refused as
/// `conflicting_declaration`, as an accepting `when:` declared there is.
#[test]
fn adversary_a_accepting_row_set_above_a_named_row_refusal_is_conflicting() {
    let fresh = "      - name: connected-fresh
        when_related:
          entity: wiring.board.Wire
          where: to_port == input.to_port
          exists: false
        creates: wiring.board.Wire
        instance: wire_id
        emits: [wiring.board.Connected]
        payload: {wiring.board.Connected: {wire_id: {generated: true}}}
        sets: {from_port: input.from_port, to_port: input.to_port}
";
    let anchor = "      - name: source-not-output\n";
    // The analogue `ess/29` already refuses: an accepting `when:` there.
    let when = "      - name: connected-apart
        when: input.from_port != input.to_port
        creates: wiring.board.Wire
        instance: wire_id
        emits: [wiring.board.Connected]
        payload: {wiring.board.Connected: {wire_id: {generated: true}}}
        sets: {from_port: input.from_port, to_port: input.to_port}
";
    let mut outcomes = Vec::new();
    for (label, branch) in [("when", when), ("row set", fresh)] {
        let text = edited(REPRO, anchor, &format!("{branch}{anchor}"));
        let verdict = match assemble(&text) {
            Ok(_) => "validates".to_owned(),
            Err(errors) => listed(&errors),
        };
        outcomes.push(format!("{label}: {verdict}"));
    }
    for outcome in &outcomes {
        assert!(
            outcome.contains("ConflictingDeclaration") && outcome.contains("source-not-output"),
            "{}",
            outcomes.join("\n")
        );
    }
}
