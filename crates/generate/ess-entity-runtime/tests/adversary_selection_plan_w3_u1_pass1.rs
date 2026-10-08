//! Adversary pass 1 against `story:entity-runtime-lowering-reads-selection-plan` (wave 3, U1).
//!
//! The wave's invariant: no behaviour change for any model that validates, with one byte exception
//! (a held-state branch declared after an accepting or external branch whose input guard the
//! finite prover shows disjoint). The story's acceptance: no branch-selection-order rule in
//! `lower_command` other than the two entity-core target rules.
//!
//! A refusal written `when: true` + `error:` declared before an input-guarded refusal was the one
//! shape the lowering's move to the precedence plan answered differently: the base lowering put it
//! first and answered it for every request; the plan puts the input-guarded refusal first. The
//! interpreter answered as the plan does, the Rust emitter and the generated explorer as the base.
//!
//! Decided in <https://github.com/beyond10x/ess/issues/489>: validation refuses the shape
//! (<https://github.com/beyond10x/ess/pull/490>), so no model that validates reaches the lowering
//! with it. The case below checks that refusal on the billing example.
use std::path::Path;

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const CANCEL: &str = "    input:\n      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n        moves: billing.invoice.Invoice.cancel\n";

/// `CancelInvoice` with `paused: when: true` + `error:` declared first, then the input-guarded
/// refusal `rush-refused: when: rush == true`, then `cancelled` guarded by `rush == false`, so
/// `paused` is the command's one unconditional branch; the example's `wrong-state` follows.
const PAUSED_FIRST: &str = "    input:\n      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - name: rush\n        type: Boolean\n\n    outcomes:\n      - name: paused\n        when: true\n        error: billing.invoice.InvoiceStateConflict\n        summary: Cancelling is paused.\n      - name: rush-refused\n        when: rush == true\n        error: billing.invoice.InvalidAmount\n        summary: A rushed cancellation is refused.\n      - name: cancelled\n        when: rush == false\n        moves: billing.invoice.Invoice.cancel\n";

/// The billing example at `ess/20`, with `CancelInvoice`'s input and first branch replaced by
/// `cancel`, assembled.
fn assembled(cancel: &str) -> Result<Specification, String> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/billing");
    let changes: [(&str, &str, &str); 4] = [
        ("system.yaml", "format: ess/1\n", "format: ess/20\n"),
        (
            "domains/email.yaml",
            "            recipient: input.recipient\n",
            "            recipient: input.recipient\n            message_id: {generated: true}\n",
        ),
        (
            "domains/invoice.yaml",
            "          billing.invoice.InvoiceCreated:\n",
            "          billing.invoice.InvoiceCreated:\n            invoice_id: {generated: true}\n",
        ),
        ("domains/invoice.yaml", CANCEL, cancel),
    ];
    let mut pending = vec![base.clone()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory is readable") {
            let path = entry.expect("fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let mut parsed = Vec::new();
    for path in paths {
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        for (target, before, after) in &changes {
            if label == *target {
                assert!(text.contains(before), "{label}: `{before}` is present");
                text = text.replacen(before, after, 1);
            }
        }
        parsed.push((Source::new(label), RawSpecFile::parse(&text).unwrap()));
    }
    Specification::assemble(parsed).map_err(|errors| errors.to_string())
}

/// The unchanged example validates, so the refusal below is the `when: true` branch's.
#[test]
fn adv_w3u1_p1_control_the_example_validates() {
    assembled(CANCEL).unwrap_or_else(|errors| panic!("the example validates: {errors}"));
}

/// A `when: true` refusal declared before an input-guarded refusal is refused by validation
/// (<https://github.com/beyond10x/ess/issues/489>).
#[test]
fn adv_w3u1_p1_a_when_true_refusal_declared_before_an_input_refusal_is_refused() {
    let Err(errors) = assembled(PAUSED_FIRST) else {
        panic!("a `when: true` refusal is refused (#489)");
    };
    assert!(
        errors.contains("`paused` is a refusal whose `when:` always holds"),
        "{errors}"
    );
}
