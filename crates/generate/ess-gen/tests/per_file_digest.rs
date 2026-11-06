//! Each generated file carries the digest of its own inputs, and nothing that moves with the whole
//! model.
//!
//! A file that stamped the whole model's digest beside its own slice was rewritten by every change
//! anywhere in the specification: one field added to one entity moved a header line in every
//! generated file, and a review of the change had to find the few files that changed for a reason
//! among hundreds that changed for none. The model's digest belongs to the record of the run, not
//! to each file it wrote.
//!
//! Asserted against a pair of models that differ in one entity of one context: every file of the
//! other context stands byte for byte, the changed context's page moves, and across every
//! projection a file whose contract digest stood is a file whose bytes stood.

use std::collections::BTreeMap;

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::{run, Artifact};
use ess_gen::docs::Docs;
use ess_gen::Provenance;

const SYSTEM: &str = r"
format: ess/1
system: shelf
version: v1

domains:
  - shelf.orders
  - shelf.billing
";

const BILLING: &str = r"
domain: shelf.billing

types:
  - name: shelf.billing.InvoiceId
    kind: newtype
    of: Uuid

entities:
  - name: shelf.billing.Invoice
    identity:
      name: invoice_id
      type: shelf.billing.InvoiceId
    fields:
      - name: total
        type: Integer
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
";

/// The orders context, with one field of its one entity typed as given.
fn orders(note: &str) -> String {
    format!(
        r"
domain: shelf.orders

types:
  - name: shelf.orders.OrderId
    kind: newtype
    of: Uuid

entities:
  - name: shelf.orders.Order
    identity:
      name: order_id
      type: shelf.orders.OrderId
    fields:
      - name: note
        type: {note}
    lifecycle:
      initial: Placed
      states: [Placed]
      terminal: [Placed]
"
    )
}

/// The shelf system, its orders entity's `note` field typed as given.
fn shelf(note: &str) -> EssIr {
    let orders = orders(note);
    let files = [
        ("system.yaml", SYSTEM),
        ("orders.yaml", orders.as_str()),
        ("billing.yaml", BILLING),
    ];
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        let raw = RawSpecFile::parse(text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.to_owned(), text.to_owned());
        parsed.push((Source::new(label.to_owned()), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn docs(ir: &EssIr) -> BTreeMap<String, Artifact> {
    run(&Docs, ir).expect("no two pages claim one path")
}

#[test]
fn a_change_in_one_context_leaves_every_docs_file_of_another_byte_identical() {
    let before = shelf("String");
    let after = shelf("Integer");
    assert_ne!(
        before.source_digest(),
        after.source_digest(),
        "the fixture pair must be two different models"
    );

    let (old, new) = (docs(&before), docs(&after));
    assert_eq!(
        old.keys().collect::<Vec<_>>(),
        new.keys().collect::<Vec<_>>(),
        "the change adds or removes no page"
    );

    let billing: Vec<&String> = old.keys().filter(|path| path.contains("billing")).collect();
    assert!(
        !billing.is_empty(),
        "the billing context has no page of its own; there are {:?}",
        old.keys().collect::<Vec<_>>()
    );
    for path in billing {
        assert_eq!(
            old[path].contents, new[path].contents,
            "`{path}` documents only the billing context, and a change to an orders entity \
             rewrote it"
        );
    }

    let orders: Vec<&String> = old.keys().filter(|path| path.contains("orders")).collect();
    assert!(
        !orders.is_empty(),
        "the orders context has no page of its own"
    );
    for path in orders {
        assert_ne!(
            old[path].contents, new[path].contents,
            "`{path}` documents the entity that changed, and it stood"
        );
    }
}

#[test]
fn across_every_projection_a_file_whose_contract_digest_stood_is_byte_identical() {
    let old = ess_gen::generate_all(&shelf("String")).expect("paths are unique");
    let new = ess_gen::generate_all(&shelf("Integer")).expect("paths are unique");
    let mut stood = 0;
    for (path, artifact) in &old {
        let Some(regenerated) = new.get(path) else {
            continue;
        };
        let before = Provenance::read_digests(&artifact.contents)
            .unwrap_or_else(|| panic!("`{path}` carries readable provenance"));
        let after = Provenance::read_digests(&regenerated.contents)
            .unwrap_or_else(|| panic!("`{path}` carries readable provenance"));
        if before.contract_digest == after.contract_digest {
            stood += 1;
            assert_eq!(
                artifact.contents, regenerated.contents,
                "`{path}` claims its inputs did not move, and its bytes did"
            );
        } else {
            assert_ne!(artifact.contents, regenerated.contents, "`{path}`");
        }
    }
    assert!(
        stood > 0,
        "no file's slice excluded the change, so the property was never exercised"
    );
}

#[test]
fn a_change_inside_a_files_slice_still_changes_that_file() {
    // The other half: dropping the whole-model digest must not leave a file that stands while
    // its own inputs move, which is what the drift check (`ess generate --check`, a byte
    // comparison) would then miss.
    let (old, new) = (docs(&shelf("String")), docs(&shelf("Integer")));
    let path = old
        .keys()
        .find(|path| path.contains("orders"))
        .expect("an orders page");
    let (before, after) = (&old[path].contents, &new[path].contents);
    assert_ne!(before, after, "`{path}` stood while its entity moved");
    assert_ne!(
        Provenance::read_digests(before).map(|read| read.contract_digest),
        Provenance::read_digests(after).map(|read| read.contract_digest),
        "`{path}`'s stamp stood while its entity moved"
    );
}
