//! A variant may declare what it is called on the wire, and only from `ess/5`.
//!
//! `story:an-enum-variant-carries-its-own-wire-spelling`. The case that asked for it:
//! `RecordingAction` is one command over three route shapes, where `Stop` carries the empty path
//! suffix and `Flag` carries `/flag`. Without a declared spelling the alternatives are splitting
//! the command to satisfy a path, or writing the table again in every target.

use ess_domain::types::EnumVariant;
use ess_domain::{spec::RawSpecFile, system::Source, Specification, TypeBody};
use ess_primitives::error::ValidationCode;

fn model(format: &str, variants: &str) -> String {
    format!(
        "format: {format}\nsystem: recordings\nversion: v1\ndomain: recordings.backend\ntypes:\n  \
         - name: recordings.backend.RecordingAction\n    kind: enum\n    variants:\n{variants}"
    )
}

fn assemble(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| error.to_string())?;
    Specification::assemble([(Source::new("recordings.yaml"), raw)]).map_err(|e| e.to_string())
}

const DECLARED: &str = "      - name: Stop\n        wire: \"\"\n      - name: Flag\n        wire: \
                        flag\n      - name: Tags\n        wire: tags\n";
const BARE: &str = "      - Stop\n      - Flag\n      - Tags\n";

fn variants_of(spec: &Specification) -> Vec<EnumVariant> {
    let declared = spec
        .system()
        .types
        .get(&"recordings.backend.RecordingAction".parse().unwrap())
        .expect("the enum is declared");
    match &declared.body {
        TypeBody::Enum { variants } => variants.clone(),
        other => panic!("declared as an enum, read back as {other:?}"),
    }
}

#[test]
fn a_variant_declares_its_own_wire_spelling_under_ess_5() {
    let spec = assemble(&model("ess/5", DECLARED)).expect("ess/5 admits declared variant naming");
    let variants = variants_of(&spec);

    assert_eq!(
        variants
            .iter()
            .map(|variant| (variant.name(), variant.wire()))
            .collect::<Vec<_>>(),
        vec![("Stop", ""), ("Flag", "flag"), ("Tags", "tags")],
        "the declared spelling is what comes back, including the empty one"
    );
}

#[test]
fn an_empty_wire_spelling_is_a_spelling_and_not_an_absent_one() {
    let spec = assemble(&model("ess/5", DECLARED)).expect("assembles");
    let stop = variants_of(&spec)
        .into_iter()
        .next()
        .expect("a first variant");

    assert_eq!(stop.name(), "Stop");
    assert_eq!(
        stop.wire(),
        "",
        "`Stop` is the route with no suffix; falling back to its name would spell `/Stop`"
    );
    assert!(!stop.is_bare(), "it declared something");
}

#[test]
fn a_bare_variant_falls_back_to_its_name() {
    let spec = assemble(&model("ess/5", BARE)).expect("assembles");

    for variant in variants_of(&spec) {
        assert_eq!(variant.wire(), variant.name());
        assert!(variant.is_bare());
    }
}

#[test]
fn declared_variant_naming_is_refused_before_ess_5() {
    for format in ["ess/1", "ess/2", "ess/3", "ess/4"] {
        let errors = assemble(&model(format, DECLARED))
            .expect_err("declared variant naming is not admitted here");

        assert!(
            errors.contains(&ValidationCode::UnsupportedFormatVersion.to_string()),
            "{format}: {errors}"
        );
        assert!(
            errors.contains("ess/5"),
            "{format}: the refusal says which format admits it: {errors}"
        );
        assert!(
            errors.contains("Stop"),
            "{format}: the refusal names the variant: {errors}"
        );
    }
}

#[test]
fn a_bare_variant_list_is_admitted_by_every_format() {
    for format in ["ess/1", "ess/2", "ess/3", "ess/4", "ess/5"] {
        assemble(&model(format, BARE)).unwrap_or_else(|error| {
            panic!("{format} has always admitted a bare variant list: {error}")
        });
    }
}

#[test]
fn a_bare_variant_serializes_back_as_a_bare_name() {
    let variants = EnumVariant::bare(["Stop", "Flag"]);
    assert_eq!(
        serde_json::to_string(&variants).expect("serializes"),
        r#"["Stop","Flag"]"#,
        "a document written before declared naming existed keeps its bytes"
    );
}

#[test]
fn a_declared_variant_serializes_as_a_mapping() {
    let spec = assemble(&model("ess/5", DECLARED)).expect("assembles");
    let stop = variants_of(&spec)
        .into_iter()
        .next()
        .expect("a first variant");

    assert_eq!(
        serde_json::to_string(&stop).expect("serializes"),
        r#"{"name":"Stop","wire":""}"#
    );
}

#[test]
fn a_variant_round_trips_through_both_authored_forms() {
    for text in [r#""Flag""#, r#"{"name":"Flag"}"#] {
        let variant: EnumVariant = serde_json::from_str(text).expect("both forms read");
        assert_eq!(variant, EnumVariant::new("Flag"));
        assert_eq!(
            serde_json::to_string(&variant).expect("serializes"),
            r#""Flag""#,
            "a variant that declares nothing is written back bare, whichever form wrote it"
        );
    }
}

#[test]
fn a_key_the_variant_does_not_know_is_refused_by_name() {
    let error = assemble(&model("ess/5", "      - name: Stop\n        wyre: \"\"\n"))
        .expect_err("a misspelt key is not dropped in silence");

    assert!(
        error.contains("wyre"),
        "the refusal names the key rather than reporting that no form matched: {error}"
    );
}

#[test]
fn the_published_schema_accepts_what_the_parser_accepts() {
    // A variant name has never been checked against a pattern on the way in. Publishing one would
    // make an author's editor refuse a document this repository assembles.
    let schema = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../schemas/generated/ess.schema.json"),
    )
    .expect("the generated schema is committed");
    let schema: serde_json::Value = serde_json::from_str(&schema).expect("valid JSON");
    let variant = &schema["definitions"]["EnumVariant"];

    assert!(!variant.is_null(), "the schema carries the variant shape");
    assert!(
        !serde_json::to_string(variant)
            .expect("serializes")
            .contains("pattern"),
        "no pattern the parser does not enforce: {variant}"
    );
    assert!(
        !serde_json::to_string(&schema["definitions"]["NamedEnumVariant"])
            .expect("serializes")
            .contains("pattern"),
        "the mapping form carries none either"
    );
}
