//! A `defined` / `not defined` partition over an Optional input, which the finite proof admits
//! without a default (beyond10x/ess#644), reaches generated target behaviour.
//!
//! The emitted Rust is read as text, not built: the interpreted target runs the same partition in
//! `ess-conformance`'s `when_subject_witness` suite.
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

const MODEL: &str = "format: ess/32
system: demo
version: v1
domain: demo.orders
events:
  - {name: demo.orders.Discounted, fields: []}
  - {name: demo.orders.FullPrice, fields: []}
commands:
  - name: demo.orders.Checkout
    input:
      - {name: coupon, type: Optional<String>}
    outcomes:
      - name: discounted
        when: defined(coupon)
        emits: [demo.orders.Discounted]
      - name: full-price
        when: not defined(coupon)
        emits: [demo.orders.FullPrice]
components:
  - component: orders-service
    owns: {domains: [demo.orders]}
    accepts: {commands: [demo.orders.Checkout]}
    publishes: {events: [demo.orders.Discounted, demo.orders.FullPrice]}
";

const PRESENCE_READ: &str = "Some(&input.coupon).and_then(|value| value.as_ref()).is_some()";

#[test]
fn presence_partition_reaches_generated_targets() {
    let spec = Specification::assemble([(
        Source::new("presence.yaml"),
        RawSpecFile::parse(MODEL).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    for target in [Target::Rust, Target::Go, Target::Web] {
        let synthesis = synthesize_for(&ir, target).unwrap();
        assert!(
            synthesis.plan.is_generated(
                ess_synth::CapabilityKind::CommandBehavior,
                "demo.orders.Checkout"
            ),
            "{} does not generate demo.orders.Checkout",
            target.name()
        );
        if target != Target::Rust {
            continue;
        }
        let behaviour = &synthesis.artifacts["crates/demo-types/src/behaviour.rs"].contents;
        let discounted = behaviour
            .find(&format!("decided(Some({PRESENCE_READ})"))
            .unwrap_or_else(|| panic!("no present-coupon branch:\n{behaviour}"));
        let full_price = behaviour
            .find(&format!(
                "decided((Some({PRESENCE_READ})).map(|value| !value)"
            ))
            .unwrap_or_else(|| panic!("no absent-coupon branch:\n{behaviour}"));
        assert!(discounted < full_price, "{behaviour}");
        let discounted_return =
            behaviour[discounted..full_price].contains("CheckoutOutcome::Discounted");
        assert!(
            discounted_return,
            "present coupon returns no Discounted:\n{behaviour}"
        );
        assert!(
            behaviour[full_price..].contains("CheckoutOutcome::FullPrice"),
            "absent coupon returns no FullPrice:\n{behaviour}"
        );
    }
}
