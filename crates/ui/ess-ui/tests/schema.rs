//! `schemas/ui/ess-ui.schema.yaml` says what the story says it must: every construct documented,
//! every type structural, every ordered collection a list, the composite union one union.

use std::collections::BTreeSet;
use std::path::Path;

use serde_yaml::{Mapping, Value};

fn schema() -> Value {
    serde_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML")
}

fn constructs(schema: &Value) -> &Mapping {
    schema["constructs"]
        .as_mapping()
        .expect("constructs is a map")
}

#[test]
fn the_embedded_schema_is_the_file_in_the_tree() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../schemas/ui/ess-ui.schema.yaml");
    let on_disk = std::fs::read_to_string(file).expect("the schema file reads");
    assert_eq!(on_disk, ess_ui::SCHEMA);
    assert_eq!(schema()["format"], Value::from(ess_ui::FORMAT));
}

#[test]
fn no_type_is_written_as_a_string() {
    let example = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../examples/partner-portal/ui.yaml"),
    )
    .expect("the example reads");
    for (name, text) in [("schema", ess_ui::SCHEMA), ("example", example.as_str())] {
        assert_eq!(text.matches("type: \"").count(), 0, "{name} quotes a type");
    }
}

#[test]
fn every_construct_carries_summary_doc_notes_example_group_and_order() {
    let schema = schema();
    let groups: BTreeSet<&str> = schema["groups"]
        .as_sequence()
        .expect("groups is a list")
        .iter()
        .map(|group| group["name"].as_str().expect("a group name"))
        .collect();
    let mut problems = Vec::new();
    for (name, construct) in constructs(&schema) {
        let name = name.as_str().expect("a construct name");
        for key in ["summary", "doc"] {
            if !construct[key].is_string() {
                problems.push(format!("{name} has no `{key}`"));
            }
        }
        if construct.get("example").is_none() {
            problems.push(format!("{name} has no `example`"));
        }
        if !construct["group"]
            .as_str()
            .is_some_and(|group| groups.contains(group))
        {
            problems.push(format!("{name} names no declared `group`"));
        }
        if !construct["order"].is_u64() {
            problems.push(format!("{name} has no `order`"));
        }
        let Some(fields) = construct["fields"].as_mapping() else {
            problems.push(format!("{name} has no `fields`"));
            continue;
        };
        for (field, spec) in fields {
            let field = field.as_str().expect("a field name");
            if !spec["note"].is_string() {
                problems.push(format!("{name}.{field} has no `note`"));
            }
            if spec.get("type").is_none() {
                problems.push(format!("{name}.{field} has no `type`"));
            }
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

/// Whether a type expression is a list, or a `one_of` whose first alternative is one.
fn is_list(ty: &Value) -> bool {
    ty.get("list").is_some()
        || ty["one_of"]
            .as_sequence()
            .and_then(|alternatives| alternatives.first())
            .is_some_and(is_list)
}

#[test]
fn every_ordered_collection_is_a_list_of_named_nodes() {
    const ORDERED: &[&str] = &[
        "sections",
        "columns",
        "fields",
        "inputs",
        "actions",
        "row_actions",
        "bulk_actions",
        "item_actions",
        "node_actions",
        "edge_actions",
        "alternatives",
        "item",
        "choices",
        "parts",
        "metrics",
        "toolbar",
        "body",
        "children",
        "tabs",
        "groups",
        "guards",
    ];
    // A channel's `fields` names the payload fields readable as `channel.<name>.<field>`; they are
    // looked up, never laid out, so that map carries no order. A confirm's `body` is its
    // explanation text, not a widget body.
    const UNORDERED: &[(&str, &str)] = &[("Channel", "fields"), ("confirm", "body")];
    let schema = schema();
    let mut checked = 0;
    let mut maps = Vec::new();
    for (name, construct) in constructs(&schema) {
        let name = name.as_str().expect("a construct name");
        let Some(fields) = construct["fields"].as_mapping() else {
            continue;
        };
        for (field, spec) in fields {
            let field = field.as_str().expect("a field name");
            if !ORDERED.contains(&field) || UNORDERED.contains(&(name, field)) {
                continue;
            }
            checked += 1;
            if !is_list(&spec["type"]) {
                maps.push(format!("{name}.{field}"));
            }
        }
    }
    assert!(
        maps.is_empty(),
        "ordered collections written as maps: {maps:?}"
    );
    assert!(
        checked >= 30,
        "only {checked} ordered collections were checked"
    );
}

#[test]
fn the_composite_kinds_are_one_union_discriminated_by_component() {
    let schema = schema();
    let union = &schema["constructs"]["Composite"]["union"];
    assert_eq!(union["tag"], Value::from("component"));
    let members: Vec<&str> = union["members"]
        .as_sequence()
        .expect("the union lists its members")
        .iter()
        .map(|member| member.as_str().expect("a member name"))
        .collect();
    assert_eq!(members, ess_ui::COMPOSITE_KINDS);
    for member in members {
        assert_eq!(
            schema["constructs"][member]["group"],
            Value::from("composites"),
            "{member} is a construct of the composites group"
        );
    }
}

#[test]
fn every_indexed_shorthand_is_also_declared_on_its_construct() {
    let schema = schema();
    let mut missing = Vec::new();
    for entry in schema["shorthands"]["index"]
        .as_sequence()
        .expect("the shorthand index is a list")
    {
        let mut local = entry.as_mapping().expect("an index entry is a map").clone();
        let construct = local
            .remove("construct")
            .and_then(|name| name.as_str().map(str::to_owned))
            .expect("an index entry names its construct");
        let declared = schema["constructs"][construct.as_str()]["shorthand"]
            .as_sequence()
            .is_some_and(|entries| entries.contains(&Value::Mapping(local.clone())));
        if !declared {
            missing.push(format!("{construct}: {local:?}"));
        }
    }
    assert!(missing.is_empty(), "{missing:#?}");
}

#[test]
fn the_loader_holds_names_to_the_schemas_segment_pattern() {
    assert_eq!(
        schema()["constructs"]["NodePath"]["syntax"]["segment_pattern"],
        Value::from(ess_ui::SEGMENT_PATTERN)
    );
}

/// What the loader accepts the schema declares (beyond10x/ess#305): an overlay's `visible` and
/// `degrades`, a primitive's `state` and `degrades`, and a sort written without `allowed`.
#[test]
fn the_keys_the_loader_accepts_on_an_overlay_a_primitive_and_a_sort_are_declared() {
    let loaded = ess_ui::load_str(
        "format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: p
  sections: [{name: all, pages: [p]}]
pages:
  p:
    kind: detail_page
    title: P
    overlays:
      o: {kind: dialog, component: confirm, does: t.X, visible: state.on, degrades: {no_drawer: dialog}}
    sections:
      - name: rows
        component: collection
        reads: t.Rows
        sort: {by: due}
      - name: info
        component: record
        reads: t.ById
        children:
          - {name: hint, primitive: text, text: T, state: {open: {type: boolean, class: component_state}}, degrades: {no_charts: table}}
",
    );
    if let Err(error) = loaded {
        panic!("the loader refuses: {error}");
    }
    let schema = schema();
    let fields = |construct: &str| -> BTreeSet<String> {
        schema["constructs"][construct]["fields"]
            .as_mapping()
            .expect("the construct has fields")
            .keys()
            .filter_map(Value::as_str)
            .map(str::to_owned)
            .collect()
    };
    for (construct, key) in [
        ("overlay", "visible"),
        ("overlay", "degrades"),
        ("Primitive", "state"),
        ("Primitive", "degrades"),
    ] {
        assert!(
            fields(construct).contains(key),
            "the loader accepts `{key}` on {construct}, and the schema does not declare it"
        );
    }
    let allowed =
        &schema["constructs"]["collection"]["fields"]["sort"]["type"]["record"]["allowed"];
    assert!(
        allowed.get("optional").is_some(),
        "the loader defaults `sort.allowed`, and the schema requires it: {allowed:?}"
    );
}

#[test]
fn a_copy_action_derives_no_name() {
    let schema = schema();
    let sources = &schema["shorthands"]["index"]
        .as_sequence()
        .expect("the shorthand index is a list")
        .iter()
        .find(|entry| entry["construct"] == "Action" && entry["at"] == "name")
        .expect("Action derives its name")["expands_to"]["first_present"];
    let sources: Vec<&str> = sources
        .as_sequence()
        .expect("a list of sources")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        !sources.iter().any(|source| source.starts_with("copy")),
        "{sources:?}"
    );
}

fn strings(value: &Value) -> Vec<&str> {
    value
        .as_sequence()
        .unwrap_or_else(|| panic!("not a list: {value:?}"))
        .iter()
        .map(|item| item.as_str().expect("a string"))
        .collect()
}

fn keys(value: &Value) -> Vec<&str> {
    value
        .as_mapping()
        .unwrap_or_else(|| panic!("not a map: {value:?}"))
        .keys()
        .map(|key| key.as_str().expect("a string key"))
        .collect()
}

#[test]
fn the_type_and_tone_token_names_are_the_text_styles_and_the_tones() {
    let schema = schema();
    let constructs = &schema["constructs"];
    let tokens = &constructs["Tokens"]["fields"];
    let styles = strings(&constructs["text"]["fields"]["style"]["type"]["enum"]);
    let tones = strings(&constructs["Primitive"]["tone"]["enum"]);
    assert_eq!(
        strings(&tokens["type"]["type"]["map"]["key"]["enum"]),
        styles
    );
    assert_eq!(
        strings(&tokens["tone"]["type"]["map"]["key"]["enum"]),
        tones
    );
    let builtins = &constructs["Tokens"]["builtins"];
    let mut builtin_styles = keys(&builtins["type"]);
    builtin_styles.sort_unstable();
    let mut sorted_styles = styles.clone();
    sorted_styles.sort_unstable();
    assert_eq!(
        builtin_styles, sorted_styles,
        "every text style has a built-in entry"
    );
    let mut builtin_tones = keys(&builtins["tone"]);
    builtin_tones.sort_unstable();
    let mut sorted_tones = tones.clone();
    sorted_tones.sort_unstable();
    assert_eq!(
        builtin_tones, sorted_tones,
        "every tone has a built-in entry"
    );
}

#[test]
fn a_badge_and_an_icon_take_one_tone_by_naming_a_tone_map() {
    let schema = schema();
    let constructs = &schema["constructs"];
    let badge = &constructs["badge"]["fields"]["tone_by"]["type"];
    assert_eq!(badge, &constructs["icon"]["fields"]["tone_by"]["type"]);
    assert_eq!(
        badge["record"]["tones"],
        serde_yaml::from_str::<Value>("{optional: {ref: tone_map}}").expect("YAML")
    );
    assert!(
        strings(&schema["type_rule"]["constructors"]["ref"]["kinds"]).contains(&"tone_map"),
        "`{{ref: tone_map}}` is a kind of reference"
    );
}

#[test]
fn the_reference_states_the_button_and_terminal_emphasis_tables() {
    let schema = schema();
    let constructs = &schema["constructs"];
    let colors = keys(&constructs["Tokens"]["builtins"]["color"]);

    let emphasis = &constructs["button"]["emphasis"];
    assert_eq!(
        keys(emphasis),
        strings(&constructs["button"]["fields"]["tone"]["type"]["enum"])
    );
    for (tone, row) in emphasis.as_mapping().expect("a table") {
        assert_eq!(keys(row), ["fill", "text", "border"], "{tone:?}");
        for color in row.as_mapping().expect("a row").values() {
            let color = color.as_str().expect("a color name");
            assert!(
                color == "none" || colors.contains(&color),
                "{tone:?}: `{color}` is no built-in color"
            );
        }
    }
    assert_eq!(emphasis["primary"]["fill"], Value::from("accent"));
    assert_eq!(emphasis["ghost"]["border"], Value::from("none"));

    let terminal = &constructs["Tokens"]["terminal_emphasis"];
    assert_eq!(
        keys(terminal),
        strings(&constructs["Primitive"]["tone"]["enum"])
    );
    for (tone, modifiers) in [
        ("neutral", "bold"),
        ("info", "italic"),
        ("success", "bold"),
        ("warning", "underlined"),
        ("danger", "bold and underlined"),
    ] {
        assert_eq!(terminal[tone], Value::from(modifiers), "{tone}");
    }
}
