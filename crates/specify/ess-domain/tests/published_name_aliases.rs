//! The published schema admits the name aliases the parser reads.
//!
//! `story:the-published-schema-admits-the-name-aliases-the-parser-reads`. The parser reads a
//! binding's name under `id:` as well as `name:` (`RawBindingSpec`) and a component's under
//! `component:` as well as `name:` (`RawComponentSpec`). `schemars` does not see a serde alias, so
//! until this story the committed `schemas/generated/ess.schema.json` refused every document that
//! spelt a name that way: an editor marked red what `ess specify validate` reads.
//!
//! # The class
//!
//! Every serde alias on a type the document schema publishes. `ess-domain` carries four
//! `#[serde(alias)]` attributes: `RawBindingSpec.name` (`id`), `RawComponentSpec.name`
//! (`component`), and two on types no published schema reaches (`RawDomainSpec.name` and the
//! test-only `RawSystemSpec.system`). `every_published_alias_is_a_property_of_its_definition`
//! holds the two published ones.
//!
//! # The exceptions
//!
//! `every_committed_document_the_parser_reads_is_admitted_by_the_published_schema` names two
//! refusals and admits them, each for a mechanism other than an alias:
//!
//! - A view's `order_by` key, `Ranking`. The parser reads it as a string (`priority desc`,
//!   `view.rs`), and the schema types it as an object, because the `#[schemars(with = "String")]`
//!   written on the struct is a field attribute that `schemars` 0.8 ignores on a container. Left to
//!   its own story.
//! - An empty document: an empty file, or one holding only comments. The parser reads no document
//!   as the empty specification, and the schema describes a mapping, so a validator handed the
//!   empty document's `null` refuses it. When this story landed, one committed file was empty, a
//!   deployment template that is no specification
//!   (`ess-deployment/tests/workload_kinds/scheduled-ingest/templates/services.yaml`).
//!
//! The corpus is every committed YAML file (`git ls-files`), wherever it sits: `models/`, `docs/`,
//! `fuzz/`, `.engineering/repro/` and `crates/` alike.

use ess_domain::binding::BindingName;
use ess_domain::component::ComponentName;
use ess_domain::spec::RawSpecFile;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const STORY: &str = "story:the-published-schema-admits-the-name-aliases-the-parser-reads";

/// Spellings every lower-case-hyphenated constructor reads.
const READ: &[&str] = &["invoice-service", "s3", "serve-hosted", "a", "x2-y3"];

/// Spellings it refuses, one per rule the constructor applies.
const REFUSED: &[&str] = &[
    "",
    "Invoice",
    "invoice_service",
    "invoice-",
    "-invoice",
    "in--voice",
    "1invoice",
    "invoice service",
    "invoice.service",
    "ïnvoice",
];

/// `(definition, canonical key, alias)` for every alias the document schema reaches.
const ALIASES: &[(&str, &str, &str)] = &[
    ("RawBindingSpec", "name", "id"),
    ("RawComponentSpec", "name", "component"),
];

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn document_schema() -> Value {
    let text = std::fs::read_to_string(root().join("schemas/generated/ess.schema.json"))
        .expect("the generated schema is committed");
    serde_json::from_str(&text).expect("the generated schema is JSON")
}

fn document_validator() -> jsonschema::Validator {
    jsonschema::validator_for(&document_schema()).expect("a usable schema")
}

/// Every committed YAML file in the repository, from `git ls-files`: dot directories
/// (`.engineering/repro/`), `models/`, `docs/`, `fuzz/` and `crates/` alike, and nothing a build or
/// another session left untracked in the checkout.
fn committed_yaml_files() -> Vec<PathBuf> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(root())
        .args(["ls-files", "-z", "--", "*.yaml", "*.yml"])
        .output()
        .expect("git runs in the checkout");
    assert!(
        output.status.success(),
        "git ls-files lists the committed files: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut files: Vec<PathBuf> = output
        .stdout
        .split(|&byte| byte == 0)
        .filter(|path| !path.is_empty())
        .map(|path| root().join(String::from_utf8_lossy(path).as_ref()))
        .collect();
    files.sort();
    files
}

/// A `Ranking` refusal, the first exception the module documentation names.
fn is_ranking_exception(error: &jsonschema::ValidationError<'_>) -> bool {
    error
        .schema_path()
        .to_string()
        .ends_with("/definitions/Ranking/type")
        && error.instance().is_string()
}

/// Every committed YAML document that `RawSpecFile::parse` reads validates against the committed
/// schema as a whole document, not only at the positions one story tightened.
#[test]
fn every_committed_document_the_parser_reads_is_admitted_by_the_published_schema() {
    let validator = document_validator();
    let files = committed_yaml_files();

    let mut read = 0;
    let mut rankings = 0;
    let mut empty = Vec::new();
    let mut refusals = Vec::new();
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        if RawSpecFile::parse(&text).is_err() {
            continue;
        }
        let Ok(value) = ess_yaml::from_str::<Value>(&text) else {
            continue;
        };
        read += 1;
        if value.is_null() {
            empty.push(
                file.strip_prefix(root())
                    .unwrap_or(file)
                    .display()
                    .to_string(),
            );
            continue;
        }
        for error in validator.iter_errors(&value) {
            if is_ranking_exception(&error) {
                rankings += 1;
                continue;
            }
            refusals.push(format!(
                "{}: {} at {} ({})",
                file.strip_prefix(root()).unwrap_or(file).display(),
                error,
                error.instance_path(),
                error.schema_path()
            ));
        }
    }
    eprintln!(
        "{read} committed documents parsed; named exceptions: {rankings} `Ranking` refusal(s), \
         empty file(s) {empty:?}"
    );
    assert!(read > 50, "the walk found the committed documents: {read}");
    assert!(
        refusals.is_empty(),
        "{STORY}: the published schema refuses {} place(s) in documents the parser reads:\n{}",
        refusals.len(),
        refusals.join("\n")
    );
}

/// Each alias is a property of its definition, with the canonical key's schema: the same
/// charset, so an editor refuses under `id:` exactly what it refuses under `name:`.
#[test]
fn every_published_alias_is_a_property_of_its_definition() {
    let schema = document_schema();
    for &(definition, canonical, alias) in ALIASES {
        let properties = &schema["definitions"][definition]["properties"];
        assert!(
            !properties[alias].is_null(),
            "{STORY}: `{definition}` publishes no `{alias}` property"
        );
        assert_eq!(
            properties[alias]["pattern"], properties[canonical]["pattern"],
            "{STORY}: `{definition}.{alias}` carries another charset than `{canonical}`"
        );
    }
}

/// The alias spelling holds the constructor's charset in both directions, read off the whole
/// document schema the way an editor reads it.
#[test]
fn an_alias_spelling_carries_the_charset_of_its_canonical_key() {
    let validator = document_validator();
    for &written in READ.iter().chain(REFUSED) {
        let binding_reads = BindingName::new(written).is_ok();
        let component_reads = ComponentName::new(written).is_ok();
        assert_eq!(
            binding_reads,
            READ.contains(&written),
            "corpus: {written:?}"
        );
        assert_eq!(
            component_reads,
            READ.contains(&written),
            "corpus: {written:?}"
        );

        let component = json!({ "components": [{ "component": written }] });
        assert_eq!(
            validator.is_valid(&component),
            component_reads,
            "{STORY}: `component: {written:?}` and `ComponentName::new` disagree"
        );

        let binding = json!({ "bindings": [{
            "id": written,
            "when": { "event": "billing.InvoiceIssued" },
            "invoke": { "command": "billing.SendInvoice" },
            "delivery": "at_most_once",
            "on_failure": "drop",
        }] });
        let canonical = json!({ "bindings": [{
            "name": written,
            "when": { "event": "billing.InvoiceIssued" },
            "invoke": { "command": "billing.SendInvoice" },
            "delivery": "at_most_once",
            "on_failure": "drop",
        }] });
        assert_eq!(
            validator.is_valid(&canonical),
            binding_reads,
            "the binding fixture is admitted under `name:` iff its name reads: {:?}",
            validator
                .iter_errors(&canonical)
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            validator.is_valid(&binding),
            binding_reads,
            "{STORY}: `id: {written:?}` and `BindingName::new` disagree"
        );
    }
}

/// A component written with `component:` is read and admitted, as the binding with `id:` is.
#[test]
fn a_component_written_with_its_component_alias_is_admitted_by_the_schema() {
    let text = "components:\n  - component: invoice-service\n";
    RawSpecFile::parse(text).expect("the parser reads `component:`");
    let document: Value = ess_yaml::from_str(text).expect("YAML");
    let refusals: Vec<String> = document_validator()
        .iter_errors(&document)
        .map(|e| e.to_string())
        .collect();
    assert!(
        refusals.is_empty(),
        "{STORY}: the parser reads a component written with `component:` and the published \
         schema refuses it: {refusals:?}"
    );
}

/// The alias is one spelling of one key: written under both spellings, or under neither, the
/// parser refuses the declaration, and so does the schema.
#[test]
fn a_name_written_under_both_spellings_or_neither_is_refused_by_both() {
    let validator = document_validator();
    for text in [
        "components:\n  - name: invoice-service\n    component: invoice-service\n",
        "components:\n  - owns: {}\n",
    ] {
        assert!(
            RawSpecFile::parse(text).is_err(),
            "the parser refuses {text:?}"
        );
        let document: Value = ess_yaml::from_str(text).expect("YAML");
        assert!(
            !validator.is_valid(&document),
            "{STORY}: the parser refuses {text:?} and the published schema admits it"
        );
    }
}
