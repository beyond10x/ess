//! A guard comparing a fact with one constant offset of another (`ess/22`, beyond10x/ess#233, #244,
//! `docs/design/expression-family-source22.md` A2) at Entity Runtime lowering.
//!
//! entity-core has no operand that moves a value by a constant, so a guard reading `lower + 5` or
//! `posted_at + 1h` is refused by name (`OffsetUnsupported`) rather than lowered to a comparison
//! with the text it is spelled like, or with its base alone — either of which decides a different
//! rule. The same command with a guard comparing the two facts directly still lowers.
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

/// `CancelInvoice` under `ess/22` with Integer inputs `lower` and `upper` and Timestamp inputs
/// `posted_at` and `due_at`, `posted` carrying `condition` verbatim.
fn cancel(condition: &str) -> EssIr {
    let replacement = format!("      - name: invoice_id\n        type: billing.invoice.InvoiceId\n      - {{name: lower, type: Integer}}\n      - {{name: upper, type: Integer}}\n      - {{name: posted_at, type: Timestamp}}\n      - {{name: due_at, type: Timestamp}}\n\n    outcomes:\n      - name: posted\n{condition}        error: billing.invoice.InvoiceStateConflict\n      - name: cancelled\n");
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

fn refused_by_offset(condition: &str, base: &str) {
    let ir = cancel(condition);
    let refused = lower_billing(&ir).expect_err(
        "entity-core has no operand that moves a value by a constant, so the lowering must refuse \
         the guard rather than compare with the text or the base alone",
    );
    assert!(
        refused.iter().any(|(code, path, message)| {
            *code == LoweringCode::OffsetUnsupported
                && path.contains("CancelInvoice")
                && message.contains(base)
                && message.contains("offset")
        }),
        "{refused:?}"
    );
}

#[test]
fn a_guard_reading_an_integer_offset_is_refused_by_name() {
    refused_by_offset("        when: upper > lower + 5\n", "lower");
}

#[test]
fn a_guard_reading_a_timestamp_offset_is_refused_by_name() {
    refused_by_offset("        when: due_at <= posted_at - 24h\n", "posted_at");
}

#[test]
fn the_same_guard_without_an_offset_still_lowers() {
    let ir = cancel("        when: upper > lower\n");
    lower_billing(&ir).expect("two Integer facts compare in entity-core");
}
