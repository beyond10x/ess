//! Adversarial cases for `story:a-field-constrained-by-a-charset-publishes-that-charset`, pass 2.
//!
//! Pass 1 walked `examples/` and `crates/` and matched refusals by schema path. That misses a
//! refusal raised inside an `anyOf` (a `SelectionMapping` sits under `AuthoredMappingSchema.anyOf`,
//! so a bad selector is reported at the `anyOf`, not at `SelectionMapping`), and it misses every
//! document outside those two trees and every spec written in a Markdown code block. These cases
//! compare the committed schema with the same schema minus the ten charsets this unit published,
//! over every YAML file and every fenced YAML block in the repository that `RawSpecFile::parse`
//! reads: an instance location refused by the first and admitted by the second is a document the
//! unit newly marks red.

use ess_domain::spec::RawSpecFile;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root exists")
}

fn committed_schema() -> Value {
    let text = std::fs::read_to_string(root().join("schemas/generated/ess.schema.json"))
        .expect("the generated schema is committed");
    serde_json::from_str(&text).expect("the generated schema is JSON")
}

/// The positions this unit published a charset at: `(definition, property, on items)`.
const PUBLISHED: &[(&str, &str, bool)] = &[
    ("RawComponentSpec", "name", false),
    ("RawCommandLineSurface", "binary", false),
    ("RawCommandGroup", "name", false),
    ("RawBindingSpec", "name", false),
    ("Transition", "name", false),
    ("Selection", "name", false),
    ("SelectionInput", "name", false),
    ("SelectionMapping", "selection", false),
    ("SelectionMapping", "path", true),
];

/// The committed schema with every charset in [`PUBLISHED`] and the workload `propertyNames`
/// removed — the base's shape at those positions.
fn relaxed_schema() -> Value {
    let mut schema = committed_schema();
    for &(definition, property, items) in PUBLISHED {
        let at = &mut schema["definitions"][definition]["properties"][property];
        let at = if items { &mut at["items"] } else { at };
        let removed = at
            .as_object_mut()
            .and_then(|object| object.remove("pattern"));
        assert!(
            removed.is_some(),
            "{definition}.{property}: the committed schema publishes a pattern here"
        );
    }
    let removed = schema["definitions"]["RawTopology"]["properties"]["workloads"]
        .as_object_mut()
        .and_then(|object| object.remove("propertyNames"));
    assert!(
        removed.is_some(),
        "RawTopology.workloads carries propertyNames"
    );
    schema
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.is_dir() {
            if matches!(
                name.as_str(),
                "target" | "node_modules" | ".git" | ".engineering"
            ) {
                continue;
            }
            walk(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Every YAML text in the repository: whole `.yaml`/`.yml` files, and each fenced `yaml`/`yml`
/// block of a `.md`/`.mdx` file, labelled with its file and opening line.
fn yaml_texts() -> Vec<(String, String)> {
    let mut files = Vec::new();
    walk(&root(), &mut files);
    files.sort();
    let mut texts = Vec::new();
    for file in files {
        let name = file
            .strip_prefix(root())
            .unwrap_or(&file)
            .to_string_lossy()
            .into_owned();
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        let extension = file
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        if matches!(extension.as_str(), "yaml" | "yml") {
            texts.push((name, text));
        } else if matches!(extension.as_str(), "md" | "mdx") {
            let mut block: Option<(usize, String)> = None;
            for (number, line) in text.lines().enumerate() {
                let trimmed = line.trim_start();
                match &mut block {
                    None => {
                        if trimmed.starts_with("```yaml") || trimmed.starts_with("```yml") {
                            block = Some((number + 1, String::new()));
                        }
                    }
                    Some((opened, body)) => {
                        if trimmed.starts_with("```") {
                            texts.push((format!("{name}:{opened}"), std::mem::take(body)));
                            block = None;
                        } else {
                            body.push_str(line);
                            body.push('\n');
                        }
                    }
                }
            }
        }
    }
    texts
}

fn refused_at(validator: &jsonschema::Validator, value: &Value) -> BTreeSet<String> {
    validator
        .iter_errors(value)
        .map(|error| error.instance_path().to_string())
        .collect()
}

/// No YAML the parser reads, anywhere in the repository, is refused by the committed schema at a
/// location the base schema admits.
#[test]
fn no_document_in_the_repository_the_parser_reads_is_newly_refused() {
    let strict = jsonschema::validator_for(&committed_schema()).expect("a usable schema");
    let relaxed = jsonschema::validator_for(&relaxed_schema()).expect("a usable schema");
    let (mut read, mut outside) = (0, 0);
    let mut regressions = Vec::new();
    for (label, text) in yaml_texts() {
        if RawSpecFile::parse(&text).is_err() {
            continue;
        }
        let Ok(value) = serde_yaml::from_str::<Value>(&text) else {
            continue;
        };
        read += 1;
        if !label.starts_with("examples/") && !label.starts_with("crates/") {
            outside += 1;
        }
        let before = refused_at(&relaxed, &value);
        for at in refused_at(&strict, &value).difference(&before) {
            regressions.push(format!("{label} at {at}"));
        }
    }
    eprintln!("{read} texts parsed, {outside} of them outside examples/ and crates/");
    assert!(read > 100, "the walk found the committed documents: {read}");
    assert!(
        regressions.is_empty(),
        "the published charsets refuse documents the parser reads:\n{}",
        regressions.join("\n")
    );
}

/// The comparison above can see a refusal inside `anyOf`: a selection mapping whose selector is
/// hyphenated is refused by the committed schema and admitted by the relaxed one, at the mapping
/// entry. Without this, an empty regression list could mean the comparison is blind there.
#[test]
fn the_comparison_sees_a_selection_mapping_refused_inside_any_of() {
    let text = include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml");
    let mut value: Value = serde_yaml::from_str(text).expect("YAML");
    let mapping = value["bindings"][0]["mapping"]
        .as_object_mut()
        .expect("the fixture maps its binding");
    let (target, _) = mapping
        .iter()
        .find(|(_, source)| source.is_object())
        .map(|(target, source)| (target.clone(), source.clone()))
        .expect("the fixture maps one input from a selection");
    mapping[&target]["selection"] = Value::String("first-identified".to_owned());
    let strict = jsonschema::validator_for(&committed_schema()).expect("a usable schema");
    let relaxed = jsonschema::validator_for(&relaxed_schema()).expect("a usable schema");
    let before = refused_at(&relaxed, &value);
    let new: Vec<_> = refused_at(&strict, &value)
        .difference(&before)
        .cloned()
        .collect();
    assert_eq!(new, vec![format!("/bindings/0/mapping/{target}")]);
}

/// End to end, for the three positions the unit's own parity tests hold only to `is_field_name`:
/// each hyphenated spelling is refused by `ess validate` (assemble, then validate) and by the
/// committed schema. A parity test against `is_field_name` stays green if the validator stops
/// calling it; this one does not.
#[test]
fn the_validator_and_the_schema_both_refuse_a_hyphenated_input_selector_and_path() {
    use ess_domain::system::Source;
    use ess_domain::Specification;

    let text = include_str!("../../../generate/ess-synth/tests/fixtures/binding-selection.yaml");
    let clean = Specification::assemble([(
        Source::new("selection.yaml"),
        RawSpecFile::parse(text).expect("parses"),
    )])
    .expect("the fixture assembles");
    assert!(
        clean.validate().is_empty(),
        "the unmodified fixture validates clean: {}",
        clean.validate()
    );
    let strict = jsonschema::validator_for(&committed_schema()).expect("a usable schema");
    let cases = [
        (
            "an input name",
            text.replace("name: legs", "name: le-gs")
                .replace("in: legs", "in: le-gs"),
        ),
        (
            "a mapping selector",
            text.replace("{selection: agent,", "{selection: ag-ent,"),
        ),
        (
            "a mapping path",
            text.replace(
                "{selection: agent, path: [id]}",
                "{selection: agent, path: [i-d]}",
            ),
        ),
    ];
    for (what, written) in cases {
        assert_ne!(written, text, "{what}: the fixture was changed");
        let raw = RawSpecFile::parse(&written).expect("the document parses");
        let refused = match Specification::assemble([(Source::new("selection.yaml"), raw)]) {
            Err(errors) => !errors.to_string().is_empty(),
            Ok(spec) => !spec.validate().is_empty(),
        };
        let value: Value = serde_yaml::from_str(&written).expect("YAML");
        assert!(refused, "{what}: ess validate refuses it");
        assert!(
            !strict.is_valid(&value),
            "{what}: the committed schema refuses it"
        );
    }
}
