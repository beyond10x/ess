//! `uniqueItems` decides the same way at every array length, for every validator this crate builds.
//!
//! Schema equality treats `0` and `-0.0` as one number, so an array holding both is never unique,
//! whatever its length and whatever order the pair appears in. The decision must not depend on the
//! size threshold a validator uses to switch from pairwise comparison to hashing.

use std::collections::BTreeSet;
use std::path::Path;

use schema_contract::bundle::{self, Bundle, Dialect, SCHEMA_DIALECT};
use schema_contract::validate::{self, IssueCode, JsonDocument};
use serde_json::{json, Value};

/// A document-rooted bundle whose root is an array with the supplied `uniqueItems` claim.
fn array_bundle(unique: bool) -> Bundle {
    let source = json!({"$schema": SCHEMA_DIALECT, "type": "array", "uniqueItems": unique});
    let bytes = format!("{}\n", serde_json::to_string_pretty(&source).unwrap());
    bundle::import_document(&bytes, "Root", &BTreeSet::new(), Dialect::Draft202012).unwrap()
}

/// `items` followed by however many distinct fillers reach `length`.
fn padded(items: &[Value], length: usize) -> Value {
    let mut values = items.to_vec();
    let mut filler = 100_i64;
    while values.len() < length {
        values.push(json!(filler));
        filler += 1;
    }
    Value::Array(values)
}

/// The lengths that straddle the pairwise/hashing threshold, plus one well beyond it.
const LENGTHS: [usize; 5] = [2, 15, 16, 17, 64];

#[track_caller]
fn refuses(bundle: &Bundle, instance: &Value) {
    let found = bundle.validate("Root", instance).unwrap();
    assert_eq!(found.len(), 1, "{instance}");
    assert_eq!(found[0].pointer, "", "{instance}");
    assert_eq!(
        found[0].message,
        format!("{instance} has non-unique elements"),
        "{instance}"
    );
}

#[track_caller]
fn accepts(bundle: &Bundle, instance: &Value) {
    assert_eq!(
        bundle.validate("Root", instance).unwrap(),
        Vec::new(),
        "{instance}"
    );
}

#[test]
fn equal_signed_zeros_are_never_unique_at_any_length() {
    let bundle = array_bundle(true);
    for pair in [
        [json!(0), json!(-0.0)],
        [json!(-0.0), json!(0)],
        [json!(0.0), json!(-0.0)],
        [json!(-0.0), json!(-0.0)],
        [json!({"z": 0}), json!({"z": -0.0})],
        [json!([0, "a"]), json!([-0.0, "a"])],
        [json!({"z": [{"y": -0.0}]}), json!({"z": [{"y": 0}]})],
    ] {
        for length in LENGTHS {
            refuses(&bundle, &padded(&pair, length));
            let reversed = [pair[1].clone(), pair[0].clone()];
            refuses(&bundle, &padded(&reversed, length));
            let mut trailing = padded(&[], length - 2);
            let values = trailing.as_array_mut().unwrap();
            values.extend(pair.clone());
            refuses(&bundle, &trailing);
        }
    }
}

#[test]
fn exact_numeric_equality_and_distinctness_are_unchanged() {
    let bundle = array_bundle(true);
    for duplicate in [
        [json!(1), json!(1.0)],
        [json!(-1), json!(-1.0)],
        [
            json!(9_007_199_254_740_992_i64),
            json!(9_007_199_254_740_992.0),
        ],
    ] {
        for length in LENGTHS {
            refuses(&bundle, &padded(&duplicate, length));
        }
    }
    for distinct in [
        [
            json!(9_007_199_254_740_992_i64),
            json!(9_007_199_254_740_993_i64),
        ],
        [json!(i64::MIN), json!(u64::MAX)],
        [json!(0), json!(0.5)],
        [json!(0), json!("0")],
        [json!(0), json!(false)],
        [json!(0), json!(null)],
        [json!({"z": 0}), json!({"z": 1})],
        [json!({"z": 0}), json!({"y": 0})],
    ] {
        for length in LENGTHS {
            accepts(&bundle, &padded(&distinct, length));
        }
    }
}

#[test]
fn empty_singleton_and_unique_arrays_still_pass_and_false_stays_nonrestrictive() {
    let unique = array_bundle(true);
    accepts(&unique, &json!([]));
    accepts(&unique, &json!([0]));
    accepts(&unique, &json!([-0.0]));
    for length in LENGTHS {
        accepts(&unique, &padded(&[], length));
    }
    let permissive = array_bundle(false);
    for length in LENGTHS {
        accepts(&permissive, &padded(&[json!(0), json!(-0.0)], length));
        accepts(&permissive, &padded(&[json!(1), json!(1.0)], length));
    }
}

#[test]
fn instance_document_validation_refuses_signed_zero_duplicates_at_any_length() {
    let schema = json!({"$schema": SCHEMA_DIALECT, "$id": "https://ess.invalid/unique",
        "type": "object", "properties": {"values": {"type": "array", "uniqueItems": true}}});
    for length in LENGTHS {
        let instance = json!({
            "schema": "https://ess.invalid/unique",
            "values": padded(&[json!(0), json!(-0.0)], length),
        });
        let report = validate::validate(
            &[JsonDocument::new("unique.schema.json", &schema)],
            &[JsonDocument::new("instance.json", &instance)],
        );
        assert!(report.valid.is_empty(), "{instance}");
        assert_eq!(report.issues.len(), 1, "{instance}");
        assert_eq!(report.issues[0].code, IssueCode::InvalidInstance);
        assert_eq!(report.issues[0].instance_path, "/values");
    }
}

/// The dependency path `tail` names, never spelled literally so no scan matches this file.
fn dependency(tail: &str) -> String {
    format!("jsonschema{}{tail}", "::")
}

/// Every path into the validator dependency that `text` names, in source order and deduplicated.
///
/// A file is read whatever it is called: the Rust runtime this crate emits into a consumer's
/// crate is `rust_runtime.rs.txt`, and it builds a validator like any other source file.
fn dependency_paths(text: &str) -> Vec<String> {
    let needle = dependency("");
    let mut found: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find(&needle) {
        rest = &rest[at + needle.len()..];
        let tail: String = rest
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == ':')
            .collect();
        let path = dependency(tail.trim_end_matches(':'));
        // A brace import names no single path; record it as one so the list stays exhaustive.
        let path = if path == needle {
            dependency("{}")
        } else {
            path
        };
        if !found.contains(&path) {
            found.push(path);
        }
    }
    found.sort();
    found
}

/// Every path naming a construction that compiles a validator with the dependency's own
/// `uniqueItems`, and therefore decides it by array length.
fn builds_a_validator(path: &str) -> bool {
    const CONSTRUCTIONS: [&str; 9] = [
        "options",
        "options_for",
        "async_options",
        "async_options_for",
        "validator_for",
        "validator_map_for",
        "is_valid",
        "validate",
        "evaluate",
    ];
    path.split("::")
        .skip(1)
        .any(|segment| CONSTRUCTIONS.contains(&segment))
}

/// Every file under `root` that names a dependency path, with the paths it names.
///
/// Every file is read, whatever it is called: the Rust runtime this crate emits into a consumer's
/// crate is `rust_runtime.rs.txt`, and it builds a validator like any other source file.
fn dependency_sites(root: &Path, keep: impl Fn(&Path) -> bool) -> Vec<(String, Vec<String>)> {
    let mut sites = Vec::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if keep(&path) {
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                let paths = dependency_paths(&text);
                if !paths.is_empty() {
                    sites.push((
                        path.strip_prefix(root).unwrap().display().to_string(),
                        paths,
                    ));
                }
            }
        }
    }
    sites.sort();
    sites
}

/// A site rendered as one comparable line.
fn rendered(site: &(String, Vec<String>)) -> String {
    format!("{}: {}", site.0, site.1.join(", "))
}

#[test]
fn every_validator_this_crate_builds_or_emits_comes_from_the_checked_options() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // The positive form: every path into the dependency, named. A construction spelled any other
    // way — through the draft module, `validator_for`, `options_for`, `is_valid` — changes this
    // list and fails here, because only the whole list is pinned, not one spelling.
    let expected: Vec<String> = [
        // A type, a draft selector and the meta-schema check, which compiles the 2020-12
        // meta-schema, whose only `uniqueItems` arrays are arrays of strings.
        format!(
            "bundle.rs: {}, {}, {}",
            dependency("Draft::Draft202012"),
            dependency("Validator"),
            dependency("meta::validate")
        ),
        // The emitted runtimes build their options from the type and attach the checked keyword
        // from the `unique_items` module they are emitted beside.
        format!(
            "realize/normalize/legacy_v1_v3/rust_runtime.rs.txt: {}, {}, {}",
            dependency("Draft::Draft202012"),
            dependency("ValidationOptions::default"),
            dependency("Validator")
        ),
        format!(
            "realize/normalize/legacy_v4/rust_runtime.rs.txt: {}, {}, {}",
            dependency("Draft::Draft202012"),
            dependency("ValidationOptions::default"),
            dependency("Validator")
        ),
        format!(
            "realize/normalize/legacy_v5/rust_runtime.rs.txt: {}, {}, {}",
            dependency("Draft::Draft202012"),
            dependency("ValidationOptions::default"),
            dependency("Validator")
        ),
        format!(
            "realize/normalize/rust_runtime.rs.txt: {}, {}, {}",
            dependency("Draft::Draft202012"),
            dependency("ValidationOptions::default"),
            dependency("Validator")
        ),
        format!(
            "realize/normalize/source.rs: {}",
            dependency("Draft::Draft202012")
        ),
        // The keyword itself, compiled here and emitted verbatim: it builds no validator.
        format!(
            "realize/normalize/unique_items.rs: {}, {}, {}",
            dependency("json::cmp"),
            dependency("paths::Location"),
            dependency("{}")
        ),
        // The one call to the dependency's own constructor in the crate.
        format!(
            "uniqueness.rs: {}, {}",
            dependency("ValidationOptions"),
            dependency("options")
        ),
        format!(
            "validate.rs: {}, {}, {}, {}, {}",
            dependency("Registry"),
            dependency("Registry::new"),
            dependency("ValidationError"),
            dependency("Validator"),
            dependency("meta::validate")
        ),
    ]
    .into_iter()
    .collect();
    let sources = dependency_sites(&crate_root.join("src"), |_| true);
    let actual: Vec<String> = sources.iter().map(rendered).collect();
    assert_eq!(
        actual, expected,
        "every validator this crate compiles or emits must be built from `uniqueness::options()`, \
         which decides `uniqueItems` consistently at every array length"
    );
    // Said as a rule rather than as a list: no file this crate compiles or emits reaches any of
    // the dependency's own validator constructions, whatever it is called and however it is
    // spelled. The one exception is the meta-schema check, which has no options-based form and
    // whose only `uniqueItems` arrays are arrays of strings.
    let constructions: Vec<String> = sources
        .iter()
        .filter(|site| site.0 != "uniqueness.rs")
        .filter(|site| {
            site.1
                .iter()
                .any(|path| builds_a_validator(path) && path != &dependency("meta::validate"))
        })
        .map(rendered)
        .collect();
    assert_eq!(
        constructions,
        Vec::<String>::new(),
        "a validator built by the dependency's own construction decides `uniqueItems` by array \
         length; build it from `uniqueness::options()` instead"
    );

    // The crate's own tests build validators too, and one of them is the oracle a bundle's verdict
    // is asserted equal to. Only the adversary's file may build the dependency's own construction,
    // because measuring what that construction does is what the file is for.
    let unchecked: Vec<String> = dependency_sites(&crate_root.join("tests"), |path| {
        path.extension().is_some_and(|extension| extension == "rs")
            && path.file_name().and_then(std::ffi::OsStr::to_str)
                != Some("schema_unique_items_adversary.rs")
    })
    .iter()
    .filter(|site| site.1.iter().any(|path| builds_a_validator(path)))
    .map(rendered)
    .collect();
    assert_eq!(
        unchecked,
        Vec::<String>::new(),
        "a test that builds its own validator decides `uniqueItems` by array length while the \
         code it checks does not"
    );
}

#[path = "fixtures/normalization_v1.rs"]
#[allow(dead_code)]
mod fixture_v1;

#[test]
fn the_emitted_rust_runtime_carries_this_crates_own_keyword_verbatim() {
    let realization = fixture_v1::plan().rust("uniqueness-probe").unwrap();
    assert_eq!(
        realization.files["src/unique_items.rs"],
        include_str!("../src/realize/normalize/unique_items.rs"),
        "the emitted runtime must carry the reference's own keyword, not a copy of it that can \
         drift away from the behaviour it was generated from"
    );
}
