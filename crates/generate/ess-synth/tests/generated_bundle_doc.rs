//! The doc comment on the generated `Generated<P>` bundle names only the context traits the same
//! synthesis emits (`story:generated-bundle-doc-names-only-emitted-traits`).
//!
//! `ASKS_CONTEXT` fills a payload field with `{generated: true}`, so its behaviour asks the context
//! for the value and the module emits `TryContext` and its legacy `Context` adapter: the bundle's doc
//! names both, byte for byte as before. `ASKS_NOTHING` fills every field from the input and owes the
//! one command with an `external:` outcome (a `when_related:` guard beside it keeps it owed), so no
//! generated behaviour asks the context anything and neither trait is emitted: the bundle's doc
//! names neither.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};

const ASKS_CONTEXT: &str = r"
format: ess/23
system: demo
version: v1
domain: demo.rates
events:
  - name: demo.rates.RateQuoted
    fields:
      - {name: amount, type: Decimal}
      - {name: rate, type: Optional<Decimal>}
commands:
  - name: demo.rates.Quote
    input:
      - {name: amount, type: Decimal}
    outcomes:
      - name: quoted
        emits: [demo.rates.RateQuoted]
        payload:
          demo.rates.RateQuoted:
            amount: input.amount
            rate: {generated: true}
components:
  - component: rates-service
    owns: {domains: [demo.rates]}
    accepts: {commands: [demo.rates.Quote]}
    publishes: {events: [demo.rates.RateQuoted]}
    reached_by: network
";

const ASKS_NOTHING: &str = r"
format: ess/23
system: demo
version: v1
domain: demo.rates
types:
  - {name: demo.rates.DeskId, kind: newtype, of: String}
entities:
  - name: demo.rates.Desk
    identity: {name: desk_id, type: demo.rates.DeskId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
events:
  - name: demo.rates.RateQuoted
    fields:
      - {name: amount, type: Decimal}
      - {name: rate, type: Optional<Decimal>}
  - name: demo.rates.RateConfirmed
    fields:
      - {name: desk_id, type: demo.rates.DeskId}
errors:
  - name: demo.rates.NoDesk
  - name: demo.rates.QuoteRefused
commands:
  - name: demo.rates.Quote
    input:
      - {name: amount, type: Decimal}
      - {name: rate, type: Optional<Decimal>}
    outcomes:
      - name: quoted
        emits: [demo.rates.RateQuoted]
        payload:
          demo.rates.RateQuoted:
            amount: input.amount
            rate: input.rate
  - name: demo.rates.Confirm
    input:
      - {name: desk_id, type: demo.rates.DeskId}
    outcomes:
      - name: no-desk
        when_related: {via: input.desk_id, exists: false}
        error: demo.rates.NoDesk
      - {name: refused, external: the rate desk refuses the quote, error: demo.rates.QuoteRefused}
      - name: confirmed
        emits: [demo.rates.RateConfirmed]
        payload:
          demo.rates.RateConfirmed: {desk_id: input.desk_id}
components:
  - component: rates-service
    owns: {domains: [demo.rates]}
    accepts: {commands: [demo.rates.Quote, demo.rates.Confirm]}
    publishes: {events: [demo.rates.RateQuoted, demo.rates.RateConfirmed]}
    reached_by: network
";

/// The `Generated<P>` doc and struct as written while the module emits `TryContext`: the bytes
/// before this story, unchanged.
const WITH_CONTEXT: &str = "
/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes,
/// `TryContext` (or its legacy `Context` blanket adapter) where one asks it anything, and every `…Behavior` and `…Query` trait the plan still
/// owes; `Generated<P>` forwards those to it.
pub struct Generated<P> {
    /// The storage and context ports, and every behaviour or query still owed.
    pub ports: P,
}
";

/// The same, where the module emits no context trait.
const WITHOUT_CONTEXT: &str = "
/// Every generated behaviour of this workspace, over the ports `P` supplies.
///
/// `P` implements the storage trait of each entity a generated behaviour reads or writes, and
/// every `…Behavior` and `…Query` trait the plan still owes; `Generated<P>` forwards those to it.
pub struct Generated<P> {
    /// The storage ports, and every behaviour or query still owed.
    pub ports: P,
}
";

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("rates.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The Rust behaviour module of `text`, with `Quote`'s behaviour generated.
fn behaviour(text: &str) -> (ess_synth::SynthesisPlan, String) {
    let synthesis = synthesize_for(&model(text), Target::Rust).expect("a realizable target");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.rates.Quote"),
        Some(&SynthesisDisposition::Generated),
        "the behaviour is generated"
    );
    let module = synthesis
        .artifacts
        .get("crates/demo-types/src/behaviour.rs")
        .unwrap_or_else(|| {
            panic!(
                "no behaviour module among {:?}",
                synthesis.artifacts.keys().collect::<Vec<_>>()
            )
        })
        .contents
        .clone();
    (synthesis.plan, module)
}

/// The `Generated<P>` doc comment and struct, from the blank line before it to its closing brace.
fn bundle(module: &str) -> &str {
    let end = module
        .find("pub struct Generated<P> {")
        .expect("the bundle is written");
    let start = module[..end]
        .rfind("\n\n")
        .expect("a blank line precedes the doc");
    let close = end + module[end..].find("\n}\n").expect("the struct closes") + 3;
    &module[start + 1..close]
}

#[test]
fn a_module_that_emits_try_context_keeps_the_bundle_doc_byte_for_byte() {
    let (_, module) = behaviour(ASKS_CONTEXT);
    assert!(
        module.contains("pub trait TryContext {")
            && module.contains("impl<T: Context + ?Sized> TryContext for T {"),
        "the generated payload field asks the context, so both traits are emitted:\n{module}"
    );
    assert_eq!(bundle(&module), WITH_CONTEXT);
}

#[test]
fn a_module_that_emits_no_context_trait_names_neither_in_the_bundle_doc() {
    let (plan, module) = behaviour(ASKS_NOTHING);
    assert!(
        plan.obligation_of(CapabilityKind::CommandBehavior, "demo.rates.Confirm")
            .is_some(),
        "the `external:` command is owed by the implementor"
    );
    assert!(
        !module.contains("pub trait TryContext") && !module.contains("pub trait Context"),
        "no generated behaviour asks the context, so neither trait is emitted:\n{module}"
    );
    let doc = bundle(&module);
    assert!(
        !doc.contains("TryContext") && !doc.contains("`Context`"),
        "the bundle doc names a trait the module does not emit:\n{doc}"
    );
    assert_eq!(doc, WITHOUT_CONTEXT);
}
