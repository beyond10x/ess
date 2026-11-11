//! `as: bytes` on a field (story:a-value-renders-in-the-form-its-field-declares): the
//! schema lists the metric's existing word `bytes` among a field's `as` values, and a document
//! that writes it loads with it.
//!
//! `as: duration` with `unit:` (story:a-duration-renders-as-hours-minutes-seconds): the schema
//! declares `unit: ms | s` on a field, defaulting to `ms`; the model reads it, refuses any other
//! unit at the field's path, and refuses a unit on a field that is no duration.

use ess_ui::{DurationUnit, Field, MetricFormat, NodeRef};
use ess_yaml::Value;

const DOCUMENT: &str = r"format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation: {home: p, sections: [{name: all, pages: [p]}]}
pages:
  p:
    kind: list_page
    title: Files
    sections:
      - {name: used, component: metric, reads: t.Usage, format: bytes}
      - name: files
        component: collection
        reads: t.Files
        columns:
          - {field: size, as: bytes}
";

/// The values the schema lists for `Field.as`.
fn field_as_values() -> Vec<String> {
    let schema: Value = ess_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML");
    let field_as = &schema["constructs"]["Field"]["fields"]["as"]["type"]["enum"];
    field_as
        .as_sequence()
        .unwrap_or_else(|| panic!("Field.as is an enum: {field_as:?}"))
        .iter()
        .map(|value| value.as_str().expect("an enum value is text").to_owned())
        .collect()
}

#[test]
fn the_schema_lists_bytes_among_the_field_as_values() {
    let values = field_as_values();
    assert!(
        values.iter().any(|value| value == "bytes"),
        "Field.as lists bytes: {values:?}"
    );
}

#[test]
fn a_bytes_field_and_a_bytes_metric_load() {
    let document = ess_ui::load_str(DOCUMENT).unwrap_or_else(|error| panic!("{error}"));
    let mut field = None;
    let mut metric = None;
    for located in document.nodes() {
        match located.node {
            NodeRef::Field(found) if found.name == "size" => field = Some(found.clone()),
            NodeRef::Section(section) => {
                if let ess_ui::Body::Composite(ess_ui::Composite::Metric(found)) = &section.body {
                    metric = Some(found.clone());
                }
            }
            _ => {}
        }
    }
    let field = field.expect("the size column loads");
    assert_eq!(field.field_as.as_deref(), Some("bytes"));
    let metric = metric.expect("the metric loads");
    assert_eq!(metric.format, Some(MetricFormat::Bytes));
}

/// A collection of `columns`, written as flow mappings.
fn durations(columns: &str) -> String {
    format!(
        r"format: ess-ui/1
app: t
model: t.system
placement_profile: fat
shells:
  app: {{regions: {{main: {{kind: page_outlet}}}}}}
navigation: {{home: p, sections: [{{name: all, pages: [p]}}]}}
pages:
  p:
    kind: list_page
    title: Calls
    sections:
      - name: calls
        component: collection
        reads: t.Calls
        columns:
{columns}"
    )
}

fn field(document: &ess_ui::Document, name: &str) -> Field {
    document
        .nodes()
        .into_iter()
        .find_map(|located| match located.node {
            NodeRef::Field(found) if found.name == name => Some(found.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the column {name} loads"))
}

#[test]
fn the_schema_declares_a_duration_unit_of_ms_s_m_or_h_defaulting_to_ms() {
    let schema: Value = ess_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML");
    let unit = &schema["constructs"]["Field"]["fields"]["unit"];
    let values: Vec<&str> = unit["type"]["enum"]
        .as_sequence()
        .unwrap_or_else(|| panic!("Field.unit is an enum: {unit:?}"))
        .iter()
        .map(|value| value.as_str().expect("an enum value is text"))
        .collect();
    assert_eq!(values, ["ms", "s", "m", "h"]);
    assert_eq!(unit["default"].as_str(), Some("ms"), "{unit:?}");
    let note = unit["note"].as_str().expect("Field.unit has a note");
    assert!(note.contains("h:mm:ss"), "the note says the form: {note}");
}

#[test]
fn a_duration_field_reads_its_unit_and_defaults_to_milliseconds() {
    let document = ess_ui::load_str(&durations(
        "          - {field: talk, as: duration}\n          \
         - {field: hold, as: duration, unit: s}\n          \
         - {field: wrap, as: duration, unit: ms}\n",
    ))
    .unwrap_or_else(|error| panic!("{error}"));
    let talk = field(&document, "talk");
    assert_eq!(talk.unit, None);
    assert_eq!(talk.duration_unit(), DurationUnit::Ms, "the default unit");
    assert_eq!(field(&document, "hold").unit, Some(DurationUnit::S));
    assert_eq!(field(&document, "hold").duration_unit(), DurationUnit::S);
    assert_eq!(field(&document, "wrap").unit, Some(DurationUnit::Ms));
}

#[test]
fn a_duration_unit_other_than_ms_s_m_or_h_is_refused_at_the_field() {
    for unit in ["min", "hours", "seconds", "MS", "1"] {
        let error = ess_ui::load_str(&durations(&format!(
            "          - {{field: talk, as: duration, unit: {unit}}}\n"
        )))
        .err()
        .unwrap_or_else(|| panic!("unit {unit} is refused"));
        assert_eq!(
            error.path().to_string(),
            "pages/p/sections/calls/columns/talk",
            "unit {unit}: {error}"
        );
        assert!(
            error.message().contains("unit") && error.message().contains(unit),
            "the refusal names the unit {unit}: {error}"
        );
    }
}

#[test]
fn a_unit_on_a_field_that_is_no_duration_is_refused_at_the_field() {
    for column in [
        "{field: size, as: number, unit: s}",
        "{field: size, unit: ms}",
    ] {
        let error = ess_ui::load_str(&durations(&format!("          - {column}\n")))
            .err()
            .unwrap_or_else(|| panic!("{column} is refused"));
        assert_eq!(
            error.path().to_string(),
            "pages/p/sections/calls/columns/size",
            "{column}: {error}"
        );
        assert!(
            error.message().contains("as: duration"),
            "the refusal says where a unit belongs: {error}"
        );
    }
}

#[test]
fn an_unmapped_duration_unit_is_accepted() {
    let document = ess_ui::load_str(&durations(
        "          - {field: talk, as: duration, unit: 'UNMAPPED: the old page never said'}\n",
    ))
    .unwrap_or_else(|error| panic!("{error}"));
    assert!(matches!(
        field(&document, "talk").unit,
        Some(DurationUnit::Unmapped(_))
    ));
    assert_eq!(field(&document, "talk").duration_unit(), DurationUnit::Ms);
}
