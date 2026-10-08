//! A domain added to or removed from `system.yaml`'s `domains:` list is
//! `domain/<domain>/added|removed` (beyond10x/ess#469), never `system/<system>/unclassified-changed`.
//!
//! The constructs a new domain declares were already `<family>/<name>/added`, but the residual
//! still saw the domain's own entry, so a purely additive revision failed
//! `--fail-on breaking-or-unknown`. A domain that declares nothing is named by no other change, so
//! the domain gets a change of its own (design §60 reserves `domain` right after `system`) rather
//! than merely leaving the residual. The kinds are `ess-diff/15` vocabulary.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{classified, diff, Compatibility, EssDelta, FailOn, Gate, RawEssDelta};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const ONE: &str = "format: ess/14
system: warehouse
version: v1
domains:
  - warehouse.shipment
";

const TWO: &str = "format: ess/14
system: warehouse
version: v1
domains:
  - warehouse.shipment
  - warehouse.billing
";

const REORDERED: &str = "format: ess/14
system: warehouse
version: v1
domains:
  - warehouse.billing
  - warehouse.shipment
";

const SHIPMENT: &str = "domain: warehouse.shipment
entities:
  - name: warehouse.shipment.Shipment
    identity: {name: shipment_id, type: Uuid}
    fields:
      - {name: destination, type: String}
    lifecycle: {initial: Draft, states: [Draft], terminal: [Draft]}
";

const BILLING: &str = "domain: warehouse.billing
entities:
  - name: warehouse.billing.Invoice
    identity: {name: invoice_id, type: Uuid}
    fields:
      - {name: amount_cents, type: Integer}
    lifecycle: {initial: Draft, states: [Draft], terminal: [Draft]}
";

/// A domain that declares nothing: no other change can name its arrival.
const EMPTY_BILLING: &str = "domain: warehouse.billing\n";

const ADDED: &str = "domain/warehouse.billing/added";
const REMOVED: &str = "domain/warehouse.billing/removed";
const UNCLASSIFIED: &str = "system/warehouse/unclassified-changed";

fn ir(documents: &[(&str, &str)]) -> EssIr {
    let spec = Specification::assemble(documents.iter().map(|(path, text)| {
        (
            Source::new(*path),
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}")),
        )
    }))
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn shipment_only() -> EssIr {
    ir(&[("system.yaml", ONE), ("domains/shipment.yaml", SHIPMENT)])
}

fn with_billing(system: &str, billing: &str) -> EssIr {
    ir(&[
        ("system.yaml", system),
        ("domains/shipment.yaml", SHIPMENT),
        ("domains/billing.yaml", billing),
    ])
}

fn ids(delta: &EssDelta) -> Vec<String> {
    delta
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect()
}

fn verdict_of(delta: &EssDelta, id: &str) -> (Compatibility, Compatibility, Compatibility) {
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == id)
        .unwrap_or_else(|| panic!("`{id}` in {}", delta.to_canonical_json()));
    let compatibility = delta.compatibility_of(&change.id()).expect("classified");
    (
        compatibility.callers(),
        compatibility.readers(),
        compatibility.history(),
    )
}

#[test]
fn an_added_domain_is_named_compatible_and_the_gate_passes() {
    let delta = classified(&shipment_only(), &with_billing(TWO, BILLING)).unwrap();
    let json = delta.to_canonical_json();
    // Design §60: `domain` sorts after `system` and before `type`, not by the alphabet.
    assert_eq!(
        ids(&delta),
        [
            ADDED,
            "type/warehouse.billing.Invoice.State/added",
            "entity/warehouse.billing.Invoice/added",
        ],
        "{json}"
    );
    let compatible = Compatibility::Compatible;
    assert_eq!(
        verdict_of(&delta, ADDED),
        (compatible, compatible, compatible),
        "{json}"
    );
    let outcome = Gate::new(FailOn::BreakingOrUnknown)
        .judge(&delta, None)
        .unwrap();
    assert!(outcome.passed(), "{:?}\n{json}", outcome.failing());
    let text = ess_diff::render::text(&delta);
    assert!(text.contains(ADDED), "{text}");
}

#[test]
fn the_default_delta_carries_it_as_ess_diff_15_and_earlier_formats_refuse_it() {
    let delta = diff(&shipment_only(), &with_billing(TWO, BILLING)).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/15", "{json}");
    assert!(ids(&delta).contains(&ADDED.to_owned()), "{json}");
    assert!(!ids(&delta).contains(&UNCLASSIFIED.to_owned()), "{json}");

    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), delta, "it reads back");

    assert!(
        delta
            .to_canonical_json_for("ess-diff/14".parse().unwrap())
            .is_err(),
        "ess-diff/14 refuses `{ADDED}`"
    );
    let mut older: serde_json::Value = serde_json::from_str(&json).unwrap();
    older["format"] = serde_json::Value::from("ess-diff/14");
    let raw: RawEssDelta = serde_json::from_value(older).unwrap();
    let refused = EssDelta::try_from(raw).expect_err("an `ess-diff/14` reader refuses it");
    assert!(
        refused.to_string().contains("outside the vocabulary"),
        "{refused}"
    );
}

#[test]
fn a_domain_added_with_no_declaration_is_named_by_its_own_change() {
    let delta = classified(&shipment_only(), &with_billing(TWO, EMPTY_BILLING)).unwrap();
    assert_eq!(ids(&delta), [ADDED], "{}", delta.to_canonical_json());
}

#[test]
fn a_removed_domain_is_named_and_breaking() {
    let delta = classified(&with_billing(TWO, BILLING), &shipment_only()).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(
        ids(&delta),
        [
            REMOVED,
            "type/warehouse.billing.Invoice.State/removed",
            "entity/warehouse.billing.Invoice/removed",
        ],
        "{json}"
    );
    let (callers, _, _) = verdict_of(&delta, REMOVED);
    assert_eq!(callers, Compatibility::Breaking, "{json}");
    let outcome = Gate::new(FailOn::Breaking).judge(&delta, None).unwrap();
    let failing: Vec<String> = outcome.failing().iter().map(ToString::to_string).collect();
    assert!(failing.contains(&REMOVED.to_owned()), "{failing:?}");
}

/// Removing a domain that declared nothing is breaking on its own: no member removal says so.
#[test]
fn a_removed_empty_domain_still_fails_the_breaking_gate() {
    let delta = classified(&with_billing(TWO, EMPTY_BILLING), &shipment_only()).unwrap();
    assert_eq!(ids(&delta), [REMOVED], "{}", delta.to_canonical_json());
    let outcome = Gate::new(FailOn::Breaking).judge(&delta, None).unwrap();
    assert!(!outcome.passed(), "{}", delta.to_canonical_json());
}

#[test]
fn reordering_the_domains_list_is_no_change() {
    let delta = diff(
        &with_billing(TWO, BILLING),
        &with_billing(REORDERED, BILLING),
    )
    .unwrap();
    assert!(delta.is_empty(), "{}", delta.to_canonical_json());
}

/// Only the one-sided entry leaves the residual: a domain both revisions declare is still
/// compared there, so its naming edit stays `unclassified-changed` beside the added domain.
#[test]
fn a_naming_edit_on_a_domain_both_sides_declare_stays_unclassified_beside_an_added_domain() {
    let renamed = SHIPMENT.replacen(
        "domain: warehouse.shipment\n",
        "domain: warehouse.shipment\nnaming:\n  display: Outbound shipments\n",
        1,
    );
    assert_ne!(renamed, SHIPMENT);
    let after = ir(&[
        ("system.yaml", TWO),
        ("domains/shipment.yaml", &renamed),
        ("domains/billing.yaml", EMPTY_BILLING),
    ]);
    let delta = diff(&shipment_only(), &after).unwrap();
    assert_eq!(
        ids(&delta),
        [UNCLASSIFIED, ADDED],
        "{}",
        delta.to_canonical_json()
    );
}
