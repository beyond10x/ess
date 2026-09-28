//! Adversarial cases for `story:a-field-constrained-by-a-charset-publishes-that-charset`, pass 1.
//!
//! The story's rule: *a raw field constrained by a charset publishes that charset, or the schema
//! accepts what the parser refuses.* These cases hold the committed
//! `schemas/generated/ess.schema.json` to the parser on three fronts: every committed document the
//! parser reads is still admitted at the newly published charsets; boundary spellings the unit's
//! corpus does not try; and document fields outside the unit's `Raw*`-prefix enumeration that the
//! parser holds to a charset while the schema publishes a bare string.

use ess_domain::component::{CliName, ComponentName};
use ess_domain::entity::Transition;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn document_schema() -> Value {
    let text = std::fs::read_to_string(root().join("schemas/generated/ess.schema.json"))
        .expect("the generated schema is committed");
    serde_json::from_str(&text).expect("the generated schema is JSON")
}

fn definition_validator(definition: &str) -> jsonschema::Validator {
    let schema = document_schema();
    assert!(
        !schema["definitions"][definition].is_null(),
        "the document schema carries `{definition}`"
    );
    jsonschema::validator_for(&json!({
        "$schema": schema["$schema"],
        "definitions": schema["definitions"],
        "allOf": [{ "$ref": format!("#/definitions/{definition}") }],
    }))
    .expect("a usable schema")
}

fn yaml_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if path.is_dir() {
            if name == "target" || name.starts_with('.') || name == "node_modules" {
                continue;
            }
            yaml_files(&path, out);
        } else if name.ends_with(".yaml") || name.ends_with(".yml") {
            out.push(path);
        }
    }
}

/// Every committed YAML document that `RawSpecFile::parse` reads is admitted by the committed
/// schema at the five places this unit tightened. A refusal there is a document an editor now
/// marks red that `ess validate` reads — the one direction the story's safety fact forbids.
#[test]
fn every_committed_document_the_parser_reads_is_admitted_at_the_new_charsets() {
    let schema = document_schema();
    let validator = jsonschema::validator_for(&schema).expect("a usable schema");
    let mut files = Vec::new();
    yaml_files(&root().join("examples"), &mut files);
    yaml_files(&root().join("crates"), &mut files);
    files.sort();

    let tightened = [
        "/RawComponentSpec/properties/name",
        "/RawCommandLineSurface/properties/binary",
        "/RawCommandGroup/properties/name",
        "/RawBindingSpec/properties/name",
        "/RawTopology/properties/workloads/propertyNames",
        "/Transition/properties/name",
        "/Selection/properties/name",
        "/SelectionInput/properties/name",
        "/SelectionMapping/properties/selection",
        "/SelectionMapping/properties/path",
    ];
    let mut read = 0;
    let mut regressions = Vec::new();
    let mut other = 0;
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        if RawSpecFile::parse(&text).is_err() {
            continue;
        }
        let Ok(value) = serde_yaml::from_str::<Value>(&text) else {
            continue;
        };
        read += 1;
        for error in validator.iter_errors(&value) {
            let at = error.schema_path().to_string();
            if tightened.iter().any(|t| at.contains(t)) {
                regressions.push(format!(
                    "{}: {} at {}",
                    file.display(),
                    error,
                    error.instance_path()
                ));
            } else {
                other += 1;
            }
        }
    }
    eprintln!("{read} committed documents parsed; {other} refusals outside the tightened fields");
    assert!(read > 50, "the walk found the committed documents: {read}");
    assert!(
        regressions.is_empty(),
        "the tightened schema refuses documents the parser reads:\n{}",
        regressions.join("\n")
    );
}

/// Boundaries the unit's corpus does not try: a trailing newline (a `$` that also matches before
/// a final line break would admit it), a non-ASCII letter and digit, a single hyphenated digit
/// segment, and a long name (no constructor bounds the length, so neither may the schema).
#[test]
fn the_published_charset_agrees_with_the_constructor_at_the_boundaries() {
    let spellings = [
        "a\n",
        "a-1",
        "a-1-2",
        "z9-9z",
        "\u{0430}",
        "a\u{0661}",
        "\u{ff41}",
        "a\u{00a0}",
        "A",
        "a-B",
        "a--",
        "-",
    ];
    let long = "a".repeat(4096);
    let group = definition_validator("RawCommandGroup");
    let topology = definition_validator("RawTopology");
    for written in spellings.iter().copied().chain([long.as_str()]) {
        let reads = CliName::new(written).is_ok();
        let admitted = group.is_valid(&json!({ "name": written }));
        assert_eq!(admitted, reads, "RawCommandGroup.name {written:?}");
        let reads = ComponentName::new(written).is_ok();
        let admitted = topology.is_valid(&json!({ "workloads": { written: {} } }));
        assert_eq!(admitted, reads, "RawTopology.workloads key {written:?}");
    }
}

/// `Transition.name` is read from the document (`entities[].lifecycle.transitions[].name`) through
/// `deserialize_local_name`, which holds it to a single `QualifiedName` segment — a charset,
/// refused while the document is read. The committed schema publishes it as a bare string, so an
/// editor admits a transition name `ess validate` refuses: the story's rule, broken on a field the
/// `Raw*`-prefix enumeration in `tests/published_charsets.rs` never looked at.
#[test]
fn a_transition_name_the_parser_refuses_the_schema_refuses() {
    let validator = definition_validator("Transition");
    for written in ["settle now", "1settle", "billing.settle", "", "settle!"] {
        let parsed = serde_yaml::from_str::<Transition>(&format!(
            "name: {written:?}\nfrom: [Draft]\nto: Paid\n"
        ));
        assert!(
            parsed.is_err(),
            "the parser refuses the transition {written:?}"
        );
        assert!(
            !validator.is_valid(&json!({ "name": written, "from": ["Draft"], "to": "Paid" })),
            "Transition.name {written:?}: the parser refuses it and the published schema admits it"
        );
    }
}

/// `RawBindingSpec.name` is read under the alias `id` as well (`binding.rs:206`), and 21 committed
/// documents write it that way, `binding-selection.yaml` among them. The charset this unit
/// published sits on `name` only: the schema has no `id` property at all, so a binding written
/// with `id:` is refused whole by `additionalProperties` and `required`, whatever its spelling.
///
/// That is today's state, pinned here. Publishing the aliases is
/// `story:the-published-schema-admits-the-name-aliases-the-parser-reads`, and this case flips
/// when that story lands.
#[test]
fn a_binding_written_with_its_id_alias_is_refused_by_the_schema_today() {
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml");
    RawSpecFile::parse(text).expect("the parser reads `id:`");
    let document: Value = serde_yaml::from_str(text).expect("YAML");
    let binding = &document["bindings"][0];
    assert_eq!(binding["id"], json!("choose"));
    let validator = definition_validator("RawBindingSpec");
    let refusals: Vec<String> = validator
        .iter_errors(binding)
        .map(|e| e.to_string())
        .collect();
    assert!(
        !refusals.is_empty(),
        "the schema now admits a binding written with `id:`: \
         story:the-published-schema-admits-the-name-aliases-the-parser-reads has landed, so turn \
         this case into the assertion that it is admitted"
    );
}

/// `Selection.name` (`bindings[].selections[].name`) is held to the field-name charset by
/// `SelectionPlan` construction (`selection.rs:236`), so `ess validate` refuses a hyphenated
/// selector. The committed schema publishes a bare string.
#[test]
fn a_selector_name_the_validator_refuses_the_schema_refuses() {
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml")
        .replace("first_identified", "first-identified");
    let raw = RawSpecFile::parse(&text).expect("the document parses");
    let assembled = Specification::assemble([(Source::new("selection.yaml"), raw)]);
    let refused = match assembled {
        Err(errors) => errors.to_string(),
        Ok(spec) => spec.validate().to_string(),
    };
    assert!(
        refused.contains("first-identified"),
        "ess validate refuses the hyphenated selector: {refused}"
    );
    let validator = definition_validator("Selection");
    assert!(
        !validator.is_valid(&json!({ "name": "first-identified", "first_present": ["a"] })),
        "Selection.name `first-identified`: the validator refuses it and the published schema \
         admits it"
    );
}
