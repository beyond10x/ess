//! Exact ESS binding, canonicalization, refusals, and the generated run-mode projection.

use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_compiler::{compile as compile_ess, EssIr};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_realization::{compile, RealizationCode, RealizationSpec};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/billing")
}

fn compiled_ess() -> EssIr {
    let base = fixture();
    let mut found = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("fixture directory is readable") {
            let path = entry.expect("fixture entry is readable").path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .is_some_and(|extension| extension == "yaml")
            {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("source is beneath fixture root")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("fixture source is readable");
        let raw = RawSpecFile::parse(&text).expect("fixture source parses");
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("billing fixture validates:\n{errors}"));
    compile_ess(&specification, &sources)
        .unwrap_or_else(|errors| panic!("billing fixture resolves:\n{errors}"))
}

fn source(ess: &EssIr) -> String {
    format!(
        r#"type: ess-realization/1
id: billing-local
specification:
  system: billing
  version: v3
  source_digest: sha256:{}
synthesis:
  target: rust-linux-x86_64/1
  generator: ess/0.8.0
components: [invoice-service]
actors: [billing.invoice.Customer, billing.invoice.Auditor]
implementations:
  - id: billing-binary
    components: [invoice-service]
    artifact:
      kind: binary
      locator: https://example.invalid/billing.tar.gz
      identity: sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
entrypoints:
  - id: local-tui
    title: Local TUI
    summary: Run the interactive terminal interface.
    primary: true
    interaction: agent_loop
    attachment: in_process
    availability: public
    support: preview
    implementation: billing-binary
    actors: [billing.invoice.Customer]
    surfaces:
      - kind: command
        name: billing.invoice.CreateInvoice
      - kind: view
        name: billing.invoice.InvoiceById
    invocation:
      kind: argv
      argv: [billing, tui, --model, "${{MODEL}}"]
    requires:
      - kind: operating_system
        name: linux
        summary: Linux host.
      - kind: environment_variable
        name: MODEL
        summary: Model identifier, not a credential.
"#,
        ess.source_digest()
    )
}

#[test]
fn exact_realization_compiles_to_canonical_ir_and_markdown() {
    let ess = compiled_ess();
    let specification = RealizationSpec::from_yaml(&source(&ess)).expect("source parses");
    let realization = compile(&specification, &ess).expect("exact source resolves");
    let json = realization.to_canonical_json();
    assert!(json.starts_with("{\n  \"type\": \"ess-realization-ir/1\""));
    assert!(json.contains("\"realization_digest\": \"sha256:"));
    assert_eq!(json, realization.to_canonical_json());

    let markdown = realization.to_markdown();
    assert!(markdown.contains("Local TUI (recommended)"));
    assert!(markdown.contains("`billing tui --model ${MODEL}`"));
    assert!(!markdown.contains("Approval required"));
}

#[test]
fn a_different_ess_digest_is_refused() {
    let ess = compiled_ess();
    let source = source(&ess).replace(
        &format!("sha256:{}", ess.source_digest()),
        "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    );
    let specification = RealizationSpec::from_yaml(&source).expect("source parses");
    let diagnostics = compile(&specification, &ess).expect_err("a stale lock must be refused");
    assert!(diagnostics.contains(RealizationCode::SpecificationMismatch));
}

#[test]
fn unresolved_surfaces_and_inline_secret_flags_are_refused_together() {
    let ess = compiled_ess();
    let source = source(&ess)
        .replace(
            "billing.invoice.CreateInvoice",
            "billing.invoice.DoesNotExist",
        )
        .replace(
            "argv: [billing, tui, --model, \"${MODEL}\"]",
            "argv: [billing, tui, --api-key, actual-secret]",
        );
    let specification = RealizationSpec::from_yaml(&source).expect("source parses");
    let diagnostics = compile(&specification, &ess).expect_err("both defects must be refused");
    assert!(diagnostics.contains(RealizationCode::UnresolvedReference));
    assert!(diagnostics.contains(RealizationCode::SecretValue));
}

#[test]
fn unknown_fields_cannot_smuggle_credential_values_into_the_document() {
    let ess = compiled_ess();
    let source = source(&ess).replace(
        "name: MODEL\n        summary:",
        "name: MODEL\n        value: actual-secret\n        summary:",
    );
    let error = RealizationSpec::from_yaml(&source).expect_err("value has no place in the type");
    assert!(error.to_string().contains("unknown field `value`"));
}
