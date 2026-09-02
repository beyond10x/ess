//! What a declared relation looks like in the synthesised Rust.
//!
//! `docs/design/ess-entity-relations-design-v0.1.md` §4.3 decides this is a doc attribute and not a
//! typed field: a typed `Account` on `InvoiceData` would make the child hold the parent, which is a
//! navigation decision the runtime owns and which nothing generated here has a store to make good
//! on. What the projection owes is that a reader of the generated field is told what it means,
//! which is exactly what the specification says and what no id type can say by itself.

use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// The repository root.
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// The billing example, compiled where it lives.
fn billing() -> EssIr {
    let base = root().join("examples/billing");
    let mut labels = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                labels.push(
                    path.strip_prefix(&base)
                        .expect("inside the example")
                        .display()
                        .to_string(),
                );
            }
        }
    }
    labels.sort();

    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for label in &labels {
        let text = std::fs::read_to_string(base.join(label))
            .unwrap_or_else(|error| panic!("{label} is readable: {error}"));
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label.clone()), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the billing specification validates:\n{errors}"));
    compile_locating(&specification, &sources, &labels)
        .unwrap_or_else(|diagnostics| panic!("the billing specification resolves:\n{diagnostics}"))
}

/// The synthesised Rust module holding the invoice domain's types.
fn invoice_module() -> String {
    let synthesis = ess_synth::synthesize(&billing());
    synthesis
        .artifacts
        .get("crates/billing-types/src/invoice.rs")
        .unwrap_or_else(|| {
            panic!(
                "no invoice module; the synthesis wrote {:?}",
                synthesis.artifacts.keys().collect::<Vec<_>>()
            )
        })
        .contents
        .clone()
}

#[test]
fn the_generated_data_struct_says_what_the_field_carrying_a_relation_means() {
    let module = invoice_module();
    assert!(
        module.contains(
            "    /// Carries `invoices`: `billing.invoice.Account` owns many \
             `billing.invoice.Invoice`.\n"
        ),
        "the carrying field names both ends of the relation:\n{module}"
    );
    assert!(
        !module.contains("pub account: Account"),
        "the relation is a doc attribute, not a typed field: nothing here navigates"
    );
}

#[test]
fn the_committed_rust_module_is_byte_for_byte_what_the_projection_writes() {
    // The committed tree is what `examples/billing-realization` compiles against, so a projection
    // that changed what it emits without moving these bytes has left the hand-written half
    // satisfying an interface nothing publishes any more.
    let path = root().join("generated/rust/billing/crates/billing-types/src/invoice.rs");
    let recorded = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} is committed ({error})", path.display()));
    assert_eq!(
        invoice_module(),
        recorded,
        "`generated/rust/billing/crates/billing-types/src/invoice.rs` is not what `ess synthesize` \
         writes for the billing example; regenerate it in the commit that changed the projection"
    );
}
