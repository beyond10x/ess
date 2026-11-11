//! Adversary cases for beyond10x/ess#728: a candidate `Unknown` only for an `Optional` input it
//! left out is passed over while arranging a row. Every scenario that skip lets through passes on
//! the interpreted target; where no candidate supplies the input, the refusal stays the
//! ESS-SYNTH-002 naming it.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const HEAD: &str = r#"format: ess/34
system: example
version: v1
domain: example.review
summary: A decision is recorded, refused on some kinds, and annotated only when accepted.
types:
  - {name: example.review.Kind, kind: enum, variants: [Accept, Erratum]}
  - {name: example.review.Check, kind: enum, variants: [Reported, Verified]}
  - {name: example.review.Sign, kind: enum, variants: [Unsigned, Signed]}
  - name: example.review.Info
    kind: struct
    fields:
      - {name: check, type: example.review.Check}
entities:
  - name: example.review.Decision
    identity: {name: key, type: String}
    fields:
      - {name: kind, type: example.review.Kind}
      - {name: note, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
views:
  - name: example.review.Decisions
    source: example.review.Decision
    consistency: read_your_writes
    fields:
      - {name: key, type: String}
      - {name: kind, type: example.review.Kind}
      - {name: note, type: String}
actors:
  - name: example.review.Reviewer
    may: [example.review.Record, example.review.Annotate]
errors:
  - {name: example.review.Refused, summary: The decision is refused., fields: []}
  - {name: example.review.Other, summary: The decision is refused otherwise., fields: []}
  - {name: example.review.AlreadyExists, summary: The decision exists., fields: []}
  - {name: example.review.NotAccepted, summary: Only an accepted decision is annotated., fields: []}
events:
  - name: example.review.Recorded
    fields: [{name: key, type: String}]
  - name: example.review.Annotated
    fields: [{name: key, type: String}]
commands:
  - name: example.review.Record
    input:
      - {name: key, type: String}
      - {name: kind, type: example.review.Kind}
      - {name: check, type: "Optional<example.review.Check>", presence: omitted_when_absent}
      - {name: sign, type: "Optional<example.review.Sign>", presence: omitted_when_absent}
      - {name: info, type: "Optional<example.review.Info>", presence: omitted_when_absent}
      - {name: note, type: String}
    outcomes:
"#;

const TAIL: &str = r#"      - name: already-exists
        existing_instance: true
        error: example.review.AlreadyExists
      - name: recorded
        creates: example.review.Decision
        instance: key
        sets: {kind: input.kind, note: input.note}
        emits: [example.review.Recorded]
        payload: {example.review.Recorded: {key: input.key}}
  - name: example.review.Annotate
    input:
      - {name: key, type: String}
      - {name: note, type: String}
    outcomes:
      - name: not-accepted
        when_subject:
          predicate: kind != Accept
        error: example.review.NotAccepted
      - name: annotated
        updates: example.review.Decision
        instance: key
        sets: {note: input.note}
        emits: [example.review.Annotated]
        payload: {example.review.Annotated: {key: input.key}}
"#;

const NOT_ACCEPTED: &str = "example.review.Annotate/outcome/not-accepted";
const ANNOTATED: &str = "example.review.Annotate/outcome/annotated";

fn model(refusals: &str) -> String {
    format!("{HEAD}{refusals}{TAIL}")
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("absent-optional.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Every refusal by the scenario it names: `(code, rendered cause)`.
fn refused(result: &Synthesis) -> BTreeMap<String, (String, String)> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            (
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string),
                (refusal.cause.code().to_string(), refusal.cause.to_string()),
            )
        })
        .collect()
}

fn statuses(result: &Synthesis, target: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(target.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

/// Synthesizes `refusals` and asserts that every emitted scenario passes on the interpreted target.
fn synthesized(refusals: &str) -> (BTreeMap<String, (String, String)>, BTreeMap<String, Status>) {
    let model = ir(&model(refusals));
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refused(&result);
    let ran = statuses(&result, &model);
    let failed: Vec<_> = ran
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}\nrefused: {refused:#?}");
    (refused, ran)
}

fn assert_arranged(refusals: &str) {
    let (refused, ran) = synthesized(refusals);
    for id in [NOT_ACCEPTED, ANNOTATED] {
        assert!(
            ran.contains_key(id),
            "{id} not emitted\nemitted: {:#?}\nrefused: {refused:#?}",
            ran.keys()
        );
    }
}

/// Two `Optional` inputs, each read by its own sibling refusal beside `kind == Erratum`: a row
/// with `kind: Erratum` needs both supplied.
#[test]
fn two_optional_inputs_each_read_by_a_refusal_are_arranged() {
    assert_arranged(
        r#"      - name: not-verified
        when:
          all: ["kind == Erratum", "check != Verified"]
        error: example.review.Refused
      - name: unsigned
        when:
          all: ["kind == Erratum", "sign != Signed"]
        error: example.review.Other
"#,
    );
}

/// Two `Optional` inputs read by one refusal: a row with `kind: Erratum` needs both supplied.
#[test]
fn two_optional_inputs_in_one_refusal_are_arranged() {
    assert_arranged(
        r#"      - name: not-verified
        when:
          all: ["kind == Erratum", {any: ["check != Verified", "sign != Signed"]}]
        error: example.review.Refused
"#,
    );
}

/// A member of an `Optional` struct read by the refusal: the candidate leaving the struct out is
/// `Unknown` only for it, and one supplying `info.check: Verified` creates the `Erratum` row.
#[test]
fn a_member_of_an_optional_struct_is_arranged() {
    assert_arranged(
        r#"      - name: not-verified
        when:
          all: ["kind == Erratum", "info.check != Verified"]
        error: example.review.Refused
"#,
    );
}

/// An `Erratum` row is created by no input that supplies `check` (both values are refused), and
/// every candidate leaving it out is `Unknown`. `not-accepted` has no row: its refusal is the
/// ESS-SYNTH-002 naming the left-out `check`, as before #728, not a cause that hides it.
#[test]
#[ignore = "backlog: story:a-left-out-optional-stays-named-when-another-row-is-built"]
fn a_row_only_a_left_out_optional_could_create_refuses_naming_it() {
    let (refused, ran) = synthesized(
        r#"      - name: verified-erratum
        when:
          all: ["kind == Erratum", "check == Verified"]
        error: example.review.Refused
      - name: reported-erratum
        when:
          all: ["kind == Erratum", "check == Reported"]
        error: example.review.Other
"#,
    );
    assert!(!ran.contains_key(NOT_ACCEPTED), "{:#?}", ran.keys());
    let Some((code, cause)) = refused.get(NOT_ACCEPTED) else {
        panic!("{NOT_ACCEPTED} neither emitted nor refused: {refused:#?}");
    };
    assert_eq!(code, "ESS-SYNTH-002", "{cause}");
    assert!(cause.contains("supplied nothing for it"), "{cause}");
}

const INVOICE: &str = r#"format: ess/34
system: example
version: v1
domain: example.billing
summary: An invoice is issued, refused for a gold tier without an approved discount, and paid only on the basic tier.
types:
  - {name: example.billing.Tier, kind: enum, variants: [Basic, Gold]}
  - {name: example.billing.Discount, kind: enum, variants: [Pending, Approved]}
entities:
  - name: example.billing.Invoice
    identity: {name: key, type: String}
    fields:
      - {name: tier, type: example.billing.Tier}
    lifecycle:
      initial: Open
      states: [Open, Paid]
      terminal: [Paid]
      transitions:
        - {name: pay, from: [Open], to: Paid}
views:
  - name: example.billing.Invoices
    source: example.billing.Invoice
    consistency: read_your_writes
    fields:
      - {name: key, type: String}
      - {name: state, type: example.billing.Invoice.State}
      - {name: tier, type: example.billing.Tier}
actors:
  - name: example.billing.Clerk
    may: [example.billing.Issue, example.billing.Pay]
errors:
  - {name: example.billing.Unapproved, summary: A gold invoice needs an approved discount., fields: []}
  - {name: example.billing.AlreadyExists, summary: The invoice exists., fields: []}
  - {name: example.billing.Premium, summary: Only a basic invoice is paid here., fields: []}
  - {name: example.billing.Conflict, summary: The invoice is paid., fields: []}
events:
  - name: example.billing.Issued
    fields: [{name: key, type: String}]
  - name: example.billing.InvoicePaid
    fields: [{name: key, type: String}]
commands:
  - name: example.billing.Issue
    input:
      - {name: key, type: String}
      - {name: tier, type: example.billing.Tier}
      - {name: discount, type: "Optional<example.billing.Discount>", presence: omitted_when_absent}
    outcomes:
      - name: unapproved
        when:
          all: ["tier == Gold", "discount != Approved"]
        error: example.billing.Unapproved
      - name: already-exists
        existing_instance: true
        error: example.billing.AlreadyExists
      - name: issued
        creates: example.billing.Invoice
        instance: key
        sets: {tier: input.tier}
        emits: [example.billing.Issued]
        payload: {example.billing.Issued: {key: input.key}}
  - name: example.billing.Pay
    input:
      - {name: key, type: String}
    outcomes:
      - name: premium
        when_subject:
          predicate: tier != Basic
        error: example.billing.Premium
      - name: paid
        moves: example.billing.Invoice.pay
        instance: key
        emits: [example.billing.InvoicePaid]
        payload: {example.billing.InvoicePaid: {key: input.key}}
      - name: wrong-state
        wrong_state: true
        error: example.billing.Conflict
"#;

/// The second adopter of #728: a move guarded on a stored field the creator copies, beside a
/// refusal over an `Optional` input. Every Pay outcome is written and passes.
#[test]
fn a_guarded_move_beside_an_optional_refusal_is_arranged() {
    let model = ir(INVOICE);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refused(&result);
    assert!(refused.is_empty(), "{refused:#?}");
    let ran = statuses(&result, &model);
    for id in [
        "example.billing.Pay/outcome/premium",
        "example.billing.Pay/outcome/paid",
    ] {
        assert!(ran.contains_key(id), "{id}: {:#?}", ran.keys());
    }
    let failed: Vec<_> = ran
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}
