//! Adversary, wave D350 unit U4, pass 1.
//!
//! `story:the-published-schema-admits-the-name-aliases-the-parser-reads`. Its acceptance reads
//! "every committed YAML document that `RawSpecFile::parse` reads validates against the published
//! schema, or the exception is named with its reason". The unit's corpus case
//! (`published_name_aliases.rs`) walks `crates/` and `examples/` only, and skips every directory
//! whose name starts with a dot. The repository commits specification documents elsewhere too:
//! its own model (`models/`), design examples (`docs/design/`), fuzz seeds (`fuzz/`) and issue
//! reproductions (`.engineering/repro/`, which spell names `id:` and `component:`).
//!
//! These cases hold the acceptance over every committed file (`git ls-files`), with the same two
//! named exceptions the unit's case admits, and hold the "both or neither" refusal for a binding,
//! which the unit's case asserts for a component only.

use ess_domain::spec::RawSpecFile;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const STORY: &str = "story:the-published-schema-admits-the-name-aliases-the-parser-reads";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn document_validator() -> jsonschema::Validator {
    let text = std::fs::read_to_string(root().join("schemas/generated/ess.schema.json"))
        .expect("the generated schema is committed");
    let schema: Value = serde_json::from_str(&text).expect("the generated schema is JSON");
    jsonschema::validator_for(&schema).expect("a usable schema")
}

/// Every committed `.yaml`/`.yml` path, relative to the repository root.
fn committed_yaml() -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root())
        .args(["ls-files", "-z"])
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git ls-files succeeds");
    let mut files: Vec<String> = String::from_utf8(output.stdout)
        .expect("UTF-8 paths")
        .split('\0')
        .filter(|path| {
            std::path::Path::new(path)
                .extension()
                .is_some_and(|extension| extension == "yaml" || extension == "yml")
        })
        .map(str::to_owned)
        .collect();
    files.sort();
    files
}

/// The unit's first named exception: a `Ranking` written as a string (`published_name_aliases.rs`).
fn is_ranking_exception(error: &jsonschema::ValidationError<'_>) -> bool {
    error
        .schema_path()
        .to_string()
        .ends_with("/definitions/Ranking/type")
        && error.instance().is_string()
}

/// The acceptance over the whole committed corpus, not two of its directories.
#[test]
fn every_committed_document_anywhere_the_parser_reads_is_admitted_by_the_published_schema() {
    let validator = document_validator();
    let mut read = 0;
    let mut outside_the_units_walk = 0;
    let mut refusals = Vec::new();
    for file in committed_yaml() {
        let Ok(text) = std::fs::read_to_string(root().join(&file)) else {
            continue;
        };
        if RawSpecFile::parse(&text).is_err() {
            continue;
        }
        let Ok(value) = ess_yaml::from_str::<Value>(&text) else {
            continue;
        };
        read += 1;
        let walked_by_the_unit = (file.starts_with("crates/") || file.starts_with("examples/"))
            && !file.split('/').any(|segment| segment.starts_with('.'));
        if !walked_by_the_unit {
            outside_the_units_walk += 1;
        }
        // The unit's second named exception: the empty file, read as the empty specification.
        if value.is_null() {
            continue;
        }
        for error in validator.iter_errors(&value) {
            if is_ranking_exception(&error) {
                continue;
            }
            refusals.push(format!(
                "{file}: {error} at {} ({})",
                error.instance_path(),
                error.schema_path()
            ));
        }
    }
    eprintln!(
        "{read} committed documents parsed, {outside_the_units_walk} outside `crates/` and \
         `examples/` (or under a dot directory)"
    );
    assert!(
        outside_the_units_walk > 0,
        "the corpus has documents outside the unit's walk"
    );
    assert!(
        refusals.is_empty(),
        "{STORY}: the published schema refuses {} place(s) in committed documents the parser \
         reads, none of them a named exception:\n{}",
        refusals.len(),
        refusals.join("\n")
    );
}

/// `aliased_name_schema` (`name.rs`) repeats `additionalProperties: false` because, its comment
/// says, `schemars` 0.8 drops the parent's under `flatten` otherwise. Nothing in the unit's cases
/// would notice that line gone: every case writes known keys only. An unknown key beside either
/// spelling is refused by the parser (`deny_unknown_fields`), and must be by the schema.
#[test]
fn an_unknown_key_beside_either_name_spelling_is_refused_by_both() {
    let validator = document_validator();
    let rest = "    when:\n      event: billing.InvoiceIssued\n    invoke:\n      command: \
                billing.SendInvoice\n    delivery: at_most_once\n    on_failure: drop\n";
    for text in [
        format!("bindings:\n  - name: send-invoice\n    nmae: send-invoice\n{rest}"),
        format!("bindings:\n  - id: send-invoice\n    nmae: send-invoice\n{rest}"),
        "components:\n  - name: invoice-service\n    nmae: invoice-service\n".to_owned(),
        "components:\n  - component: invoice-service\n    nmae: invoice-service\n".to_owned(),
    ] {
        assert!(
            RawSpecFile::parse(&text).is_err(),
            "the parser refuses {text:?}"
        );
        let document: Value = ess_yaml::from_str(&text).expect("YAML");
        assert!(
            !validator.is_valid(&document),
            "{STORY}: the parser refuses an unknown key in {text:?} and the published schema \
             admits it"
        );
    }
}

/// A binding written under both spellings, or under neither, is refused by the parser and by the
/// schema. The rest of the binding is admitted, so only the name decides.
#[test]
fn a_binding_name_written_under_both_spellings_or_neither_is_refused_by_both() {
    let validator = document_validator();
    let rest = "    when:\n      event: billing.InvoiceIssued\n    invoke:\n      command: \
                billing.SendInvoice\n    delivery: at_most_once\n    on_failure: drop\n";
    let one = format!("bindings:\n  - name: send-invoice\n{rest}");
    RawSpecFile::parse(&one).expect("control: the parser reads the binding with `name:` alone");
    let one_value: Value = ess_yaml::from_str(&one).expect("YAML");
    assert!(
        validator.is_valid(&one_value),
        "control: the binding with `name:` alone is admitted: {:?}",
        validator
            .iter_errors(&one_value)
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
    );
    for text in [
        format!("bindings:\n  - name: send-invoice\n    id: send-invoice\n{rest}"),
        format!("bindings:\n  - name: send-invoice\n    id: other-name\n{rest}"),
        format!("bindings:\n  -\n{rest}"),
    ] {
        assert!(
            RawSpecFile::parse(&text).is_err(),
            "the parser refuses {text:?}"
        );
        let document: Value = ess_yaml::from_str(&text).expect("YAML");
        assert!(
            !validator.is_valid(&document),
            "{STORY}: the parser refuses {text:?} and the published schema admits it"
        );
    }
}
