//! `as: bytes` on a field (story:a-value-renders-in-the-form-its-field-declares, B7-1): the
//! schema lists the metric's existing word `bytes` among a field's `as` values, and a document
//! that writes it loads with it.

use ess_ui::{MetricFormat, NodeRef};
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
