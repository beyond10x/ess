//! A synthesized suite carries each field's presence policy on its payload leaf
//! (beyond10x/ess#139), and is written as suite/24.
//!
//! Needs the `describe` change in `synthesize.rs` (`mark_presence`).

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::scenario::SuiteFormat;
use ess_conformance::ScenarioStep;
use ess_domain::types::Presence;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const MODEL: &str = "format: ess/15
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Receipt
    kind: struct
    fields:
      - {name: code, type: Optional<String>, presence: null_when_absent}
events:
  - name: demo.orders.Placed
    fields:
      - {name: partner_ref, type: Optional<String>, presence: null_when_absent}
      - {name: discount_code, type: Optional<String>, presence: omitted_when_absent}
      - {name: note, type: Optional<String>}
      - {name: receipt, type: demo.orders.Receipt}
      - {name: later, type: Optional<demo.orders.Receipt>}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.Place]}
commands:
  - name: demo.orders.Place
    input:
      - {name: partner_ref, type: Optional<String>}
      - {name: discount_code, type: Optional<String>}
      - {name: note, type: Optional<String>}
      - {name: receipt, type: demo.orders.Receipt}
      - {name: later, type: Optional<demo.orders.Receipt>}
    outcomes:
      - name: placed
        emits: [demo.orders.Placed]
        payload:
          demo.orders.Placed:
            partner_ref: input.partner_ref
            discount_code: input.discount_code
            note: input.note
            receipt: input.receipt
            later: input.later
";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

#[test]
fn issue_139_the_synthesized_shape_carries_both_policies() {
    let result = ess_conformance::synthesize::synthesize(&ir(MODEL));
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    assert_eq!(
        result.suite.provenance.suite_version,
        SuiteFormat::parse("ess-conformance/24").unwrap()
    );
    let shape = result
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .find_map(|step| match step {
            ScenarioStep::ExpectEvent { shape, .. } => Some(shape),
            _ => None,
        })
        .expect("the suite expects the event");
    let presence = |path: &str| {
        shape
            .leaves()
            .get(path)
            .unwrap_or_else(|| panic!("no leaf {path}"))
            .presence
    };
    assert_eq!(presence("partner_ref"), Some(Presence::NullWhenAbsent));
    assert_eq!(presence("discount_code"), Some(Presence::OmittedWhenAbsent));
    assert_eq!(presence("note"), None);
    // Inside a struct that is always there, the policy is carried; inside one that may itself be
    // absent it is not, because a runner cannot tell the two absences apart.
    assert_eq!(presence("receipt.code"), Some(Presence::NullWhenAbsent));
    assert_eq!(presence("later.code"), None);
}
