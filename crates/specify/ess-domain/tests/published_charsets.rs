//! A raw field constrained by a charset publishes that charset.
//!
//! `story:a-field-constrained-by-a-charset-publishes-that-charset`. `schemas/generated/ess.schema.json`
//! is what a schema-aware editor validates an authored document against. A `Raw*` field that is a
//! bare `type: string` in that schema, while its constructor refuses most strings, hands an author a
//! green editor and a red `ess validate`.
//!
//! Each case holds one field to its constructor in both directions the story names: the pattern the
//! type derives is the constructor's `PATTERN`, and the committed schema refuses the spellings the
//! constructor refuses — and admits the ones it reads, so a pattern stricter than the parser would
//! be caught too.
//!
//! # The class
//!
//! Every position in the document schema whose text `ess-domain` parses or validates against a
//! charset, whatever the Rust type is called. Enumerated from the checking side: every charset
//! check in `ess-domain` — the `PATTERN` constructors, `QualifiedName::new` and its single-segment
//! `local_name`, and `types::field_name` with its callers — traced to the document position it
//! reads, as of this story:
//!
//! | position | checked by | published |
//! |---|---|---|
//! | `RawComponentSpec.name` | `ComponentName::new` | here |
//! | `RawCommandLineSurface.binary` | `CliName::new` | here |
//! | `RawCommandGroup.name` | `CliName::new` | here |
//! | `RawBindingSpec.name` | `BindingName::new` | here |
//! | `RawTopology.workloads` keys | `ComponentName::new` | here, as `propertyNames` |
//! | `Transition.name` | `local_name` | here, `Transition::NAME_PATTERN` |
//! | `SelectionInput.name`, `Selection.name` | `field_name` in `SelectionPlan::resolve` | here |
//! | `SelectionMapping.selection`, `.path[]` | `field_name` in `BindingSpec::validate` | here |
//! | `RawComponentSetting.name` | `CliName::new` | `component_settings.rs` |
//! | `Field`, `InputField`, `RawViewField`, `RelationSpec` names; `.via` | `field_name` | already |
//! | outcome, outcome-group, state, qualified names, versions | their newtypes | already |
//! | `RawRelated.via`, `.field` | `is_field_name`, from `ess/16` | no: see below |
//!
//! `RawRelated` is not published because a pattern there would refuse nothing: its shape is also a
//! valid `RawNestedSources`, which a document below `ess/16` means. The rest of the schema's bare
//! strings are checked by no charset: free text (`summary`, `external`, `alphabet`, `prefix`,
//! `Naming`), a name resolved by lookup among declarations (`instance`, `group_by`,
//! `Ranking.field`, `RawSubjectField`, aggregates, `First.in`, `excluding`, `first_present`, union
//! `tag`, map keys other than workloads), a small grammar that is parsed whole
//! (`SelectionInput.from`, mapping and payload source strings, `TypeRef`, predicates), or a bare
//! enum variant name, which no parser checks (`EnumVariant`, see its schema impl).

use ess_domain::binding::{BindingName, RawBindingSpec};
use ess_domain::component::{
    CliName, ComponentName, RawCommandGroup, RawCommandLineSurface, RawComponentSpec,
};
use serde_json::{json, Value};

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

fn document_schema() -> Value {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../schemas/generated/ess.schema.json"),
    )
    .expect("the generated schema is committed");
    serde_json::from_str(&text).expect("the generated schema is JSON")
}

/// A validator for one property of one definition in the committed document schema.
fn property_validator(definition: &str, property: &str) -> jsonschema::Validator {
    let schema = document_schema();
    let property_schema = &schema["definitions"][definition]["properties"][property];
    assert!(
        !property_schema.is_null(),
        "the document schema carries `{definition}.{property}`"
    );
    jsonschema::validator_for(&json!({
        "$schema": schema["$schema"],
        "definitions": schema["definitions"],
        "allOf": [property_schema],
    }))
    .expect("a usable schema")
}

/// The pattern the type derives for `property`, read from its own generated schema.
fn derived_pattern(root: &schemars::schema::RootSchema, property: &str) -> Value {
    serde_json::to_value(root).expect("serialises")["properties"][property]["pattern"].clone()
}

/// The committed schema and `parses` agree on every spelling in [`READ`] and [`REFUSED`].
fn assert_parity(definition: &str, property: &str, parses: impl Fn(&str) -> bool) {
    let validator = property_validator(definition, property);
    for &written in READ.iter().chain(REFUSED) {
        let reads = parses(written);
        assert_eq!(
            reads,
            READ.contains(&written),
            "the corpus is wrong about {written:?}"
        );
        let admitted = validator.is_valid(&json!(written));
        assert_eq!(
            admitted,
            reads,
            "{definition}.{property} {written:?}: the schema {} it and the parser {} it",
            if admitted { "admits" } else { "refuses" },
            if reads { "reads" } else { "refuses" },
        );
    }
}

#[test]
fn a_command_line_binary_publishes_the_cli_name_charset() {
    assert_eq!(
        derived_pattern(&schemars::schema_for!(RawCommandLineSurface), "binary"),
        json!(CliName::PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    assert_parity("RawCommandLineSurface", "binary", |written| {
        CliName::new(written).is_ok()
    });
}

#[test]
fn a_command_group_name_publishes_the_cli_name_charset() {
    assert_eq!(
        derived_pattern(&schemars::schema_for!(RawCommandGroup), "name"),
        json!(CliName::PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    assert_parity("RawCommandGroup", "name", |written| {
        CliName::new(written).is_ok()
    });
}

#[test]
fn a_component_name_publishes_the_component_name_charset() {
    assert_eq!(
        derived_pattern(&schemars::schema_for!(RawComponentSpec), "name"),
        json!(ComponentName::PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    assert_parity("RawComponentSpec", "name", |written| {
        ComponentName::new(written).is_ok()
    });
}

#[test]
fn a_binding_name_publishes_the_binding_name_charset() {
    assert_eq!(
        derived_pattern(&schemars::schema_for!(RawBindingSpec), "name"),
        json!(BindingName::PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    assert_parity("RawBindingSpec", "name", |written| {
        BindingName::new(written).is_ok()
    });
}
#[test]
fn a_workload_key_publishes_the_component_name_charset() {
    use ess_domain::topology::RawTopology;

    let derived = serde_json::to_value(schemars::schema_for!(RawTopology)).expect("serialises");
    assert_eq!(
        derived["properties"]["workloads"]["propertyNames"]["$ref"],
        json!("#/definitions/ComponentName"),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    let validator = property_validator("RawTopology", "workloads");
    for &written in READ.iter().chain(REFUSED) {
        let reads = ComponentName::new(written).is_ok();
        let admitted = validator.is_valid(&json!({ written: {} }));
        assert_eq!(
            admitted, reads,
            "RawTopology.workloads key {written:?}: the schema and the parser disagree"
        );
    }
}

/// Spellings on both sides of the field-name and transition-name charsets: underscores, hyphens,
/// digits, case, dots, spaces, a trailing newline and a non-ASCII letter.
const IDENTIFIERS: &[&str] = &[
    "invoice_id",
    "_url",
    "IssueInvoice",
    "a",
    "Z9",
    "issue-invoice",
    "a-",
    "_a-b",
    "",
    "_",
    "__1",
    "1abc",
    "-a",
    "a.b",
    "a b",
    "a!",
    "a\n",
    "ïd",
];

/// The committed schema at `definition.property` and `parses` agree on every spelling in
/// [`IDENTIFIERS`], of which some must be read and some refused. `wrap` places the spelling where
/// the property holds it: itself, or inside a list.
fn assert_identifier_parity(
    definition: &str,
    property: &str,
    wrap: impl Fn(&str) -> Value,
    parses: impl Fn(&str) -> bool,
) {
    let validator = property_validator(definition, property);
    let (mut read, mut refused) = (0, 0);
    for &written in IDENTIFIERS {
        let reads = parses(written);
        if reads {
            read += 1;
        } else {
            refused += 1;
        }
        let admitted = validator.is_valid(&wrap(written));
        assert_eq!(
            admitted,
            reads,
            "{definition}.{property} {written:?}: the schema {} it and the parser {} it",
            if admitted { "admits" } else { "refuses" },
            if reads { "reads" } else { "refuses" },
        );
    }
    assert!(
        read > 0 && refused > 0,
        "the corpus tries both sides: {read} read, {refused} refused"
    );
}

/// The pattern the committed schema publishes at `definition.property`, or at its items.
fn committed_pattern(definition: &str, property: &str) -> Value {
    let schema = document_schema();
    let at = &schema["definitions"][definition]["properties"][property];
    if at["pattern"].is_null() {
        at["items"]["pattern"].clone()
    } else {
        at["pattern"].clone()
    }
}

#[test]
fn a_transition_name_publishes_the_single_segment_charset() {
    use ess_domain::entity::{StateName, Transition};

    assert_eq!(
        derived_pattern(&schemars::schema_for!(Transition), "name"),
        json!(Transition::NAME_PATTERN),
        "a schema that accepts what the parser refuses is worse than no schema"
    );
    let to = StateName::new("Paid").expect("a state name");
    assert_identifier_parity(
        "Transition",
        "name",
        |w| json!(w),
        |written| Transition::new(written, [], to.clone()).is_ok(),
    );
}

#[test]
fn a_selection_input_name_publishes_the_field_name_charset() {
    use ess_domain::selection::SelectionInput;
    use ess_domain::types::{is_field_name, Field};

    assert_eq!(
        derived_pattern(&schemars::schema_for!(SelectionInput), "name"),
        json!(Field::PATTERN),
        "a schema that accepts what the validator refuses is worse than no schema"
    );
    assert_identifier_parity("SelectionInput", "name", |w| json!(w), is_field_name);
}

#[test]
fn a_selector_name_publishes_the_field_name_charset() {
    use ess_domain::selection::Selection;
    use ess_domain::types::{is_field_name, Field};

    assert_eq!(
        derived_pattern(&schemars::schema_for!(Selection), "name"),
        json!(Field::PATTERN),
        "a schema that accepts what the validator refuses is worse than no schema"
    );
    assert_identifier_parity("Selection", "name", |w| json!(w), is_field_name);
}

/// `SelectionMapping` is private to `binding.rs`, so the committed schema is read for its literal.
#[test]
fn a_selection_mapping_publishes_the_field_name_charset_on_its_selector_and_path() {
    use ess_domain::types::{is_field_name, Field};

    for property in ["selection", "path"] {
        assert_eq!(
            committed_pattern("SelectionMapping", property),
            json!(Field::PATTERN),
            "SelectionMapping.{property}: a schema that accepts what the validator refuses is \
             worse than no schema"
        );
    }
    assert_identifier_parity("SelectionMapping", "selection", |w| json!(w), is_field_name);
    assert_identifier_parity("SelectionMapping", "path", |w| json!([w]), is_field_name);
}
