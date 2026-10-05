//! A guard comparing the UTF-8 byte length of a String (`ess/22`, beyond10x/ess#233,
//! `docs/design/expression-family-source22.md` "String `.utf8_bytes`") at Entity Runtime lowering.
//!
//! entity-core has no address for a text's byte length — `count` resolves on arrays and maps only —
//! so a guard reading `{utf8_bytes: label}` is refused by name (`Utf8BytesUnsupported`) rather than
//! lowered to a read of the text itself, or of a field spelled `label.utf8_bytes`, either of which
//! decides a different rule. The same command comparing the text directly still lowers.
use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile_locating;
use ess_compiler::source::SourceMap;
use ess_domain::component::ComponentName;
use ess_domain::name::QualifiedName;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_entity_runtime::{lower, LoweringCode, LoweringOptions};
use ess_service_contract::extract;
use ess_synth::SynthesisPlan;

fn example(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
}

fn compile_changes(base: &Path, changes: &[(&str, &str, &str)]) -> EssIr {
    let mut pending = vec![base.to_path_buf()];
    let mut paths = Vec::new();
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory is readable") {
            let path = entry.expect("fixture entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|e| e == "yaml") {
                paths.push(path);
            }
        }
    }
    paths.sort();
    let mut parsed = Vec::new();
    let mut labels = Vec::new();
    let mut sources = SourceMap::new();
    for path in paths {
        let label = path.strip_prefix(base).unwrap().display().to_string();
        let mut text = std::fs::read_to_string(&path).unwrap();
        for (target, before, after) in changes {
            if label == *target {
                assert!(text.contains(before), "{target}: {before}");
                text = text.replacen(before, after, 1);
            }
        }
        sources.insert(label.clone(), text.clone());
        parsed.push((
            Source::new(label.clone()),
            RawSpecFile::parse(&text).unwrap(),
        ));
        labels.push(label);
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates: {errors}"));
    compile_locating(&specification, &sources, &labels).expect("fixture compiles")
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).unwrap()
}

fn lower_billing(ir: &EssIr) -> Result<(), Vec<(LoweringCode, String, String)>> {
    let plan = SynthesisPlan::of(ir);
    let service = extract(ir, &plan, &ComponentName::new("invoice-service").unwrap()).unwrap();
    let options = LoweringOptions {
        definition_versions: ["billing.invoice.Account", "billing.invoice.Invoice"]
            .iter()
            .map(|entry| (name(entry), NonZeroU32::new(1).unwrap()))
            .collect(),
        scales: BTreeMap::new(),
    };
    lower(&service, &options)
        .map(|_| ())
        .map_err(|diagnostics| {
            diagnostics
                .into_vec()
                .into_iter()
                .map(|d| (d.code, d.path.clone(), d.message.clone()))
                .collect()
        })
}

const KEPT_INPUT: &str = "      - name: invoice_id\n        type: billing.invoice.InvoiceId\n\n    outcomes:\n      - name: cancelled\n";

/// `CancelInvoice` under `ess/22` with String inputs `label` and `code`, `posted` carrying
/// `condition` verbatim.
fn cancel(condition: &str) -> EssIr {
    let replacement = format!("      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - {{name: label, type: String}}\n      - {{name: code, type: String}}\n\n    outcomes:\n      - name: posted\n{condition}        error: billing.invoice.InvoiceStateConflict\n      - name: cancelled\n");
    compile_changes(
        &example("billing"),
        &[
            ("system.yaml", "format: ess/1\n", "format: ess/22\n"),
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
            ("domains/invoice.yaml", KEPT_INPUT, replacement.as_str()),
        ],
    )
}

fn refused_by_byte_length(condition: &str) {
    let ir = cancel(condition);
    let refused = lower_billing(&ir).expect_err(
        "entity-core has no address for a text's byte length, so the lowering must refuse the \
         guard rather than read the text or a field of that name",
    );
    assert!(
        refused.iter().any(|(code, path, message)| {
            *code == LoweringCode::Utf8BytesUnsupported
                && path.contains("CancelInvoice")
                && message.contains("label")
                && message.contains("UTF-8")
        }),
        "{refused:?}"
    );
}

#[test]
fn a_guard_reading_a_byte_length_is_refused_by_name() {
    refused_by_byte_length("        when: label.utf8_bytes > 8\n");
}

#[test]
fn a_guard_comparing_two_byte_lengths_is_refused_by_name() {
    refused_by_byte_length("        when: label.utf8_bytes != code.utf8_bytes\n");
}

#[test]
fn the_same_guard_without_a_byte_length_still_lowers() {
    let ir = cancel("        when: label != code\n");
    lower_billing(&ir).expect("two String facts compare in entity-core");
}
