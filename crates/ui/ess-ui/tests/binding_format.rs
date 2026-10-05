//! The binding JSON a renderer reads (beyond10x/ess#328, #330): a view's `identity` and the
//! model `enums` a document's options name are additions. A binding without them reads as it did
//! and writes back the same bytes, so every binding computed before them is unchanged.

use ess_ui::binding::{Binding, EnumVariant};

/// A binding as `ess_ui_check::binding` wrote one before either addition.
const OLD: &str = r#"{
  "system": "gatepass",
  "components": {
    "pass-service": {
      "views": {
        "gatepass.visit.ExpectedVisits": {
          "path": "/visits/views/expected",
          "params": []
        }
      },
      "commands": {}
    }
  },
  "names": {
    "visit.ExpectedVisits": "gatepass.visit.ExpectedVisits"
  }
}"#;

#[test]
fn a_binding_without_identity_or_enums_reads_and_writes_its_old_bytes() {
    let binding: Binding = serde_json::from_str(OLD).expect("the old binding reads");
    assert!(binding.enums.is_empty());
    let view = binding
        .view("visit.ExpectedVisits")
        .expect("the view is found by the name the document writes");
    assert_eq!(view.identity, None);
    assert_eq!(
        serde_json::to_string_pretty(&binding).expect("serialises"),
        OLD
    );
}

#[test]
fn a_views_identity_and_a_model_enum_round_trip() {
    let mut binding: Binding = serde_json::from_str(OLD).expect("the old binding reads");
    binding
        .components
        .get_mut("pass-service")
        .and_then(|served| served.views.get_mut("gatepass.visit.ExpectedVisits"))
        .expect("the route")
        .identity = Some("visit_id".to_owned());
    binding.enums.insert(
        "gatepass.visit.Gate".to_owned(),
        vec![EnumVariant {
            value: "north-gate".to_owned(),
            label: "North".to_owned(),
        }],
    );
    let text = serde_json::to_string_pretty(&binding).expect("serialises");
    assert!(text.contains(r#""identity": "visit_id""#), "{text}");
    assert!(
        text.contains(r#""value": "north-gate""#) && text.contains(r#""label": "North""#),
        "{text}"
    );
    let read: Binding = serde_json::from_str(&text).expect("reads back");
    assert_eq!(read, binding);
    assert_eq!(
        read.view("gatepass.visit.ExpectedVisits")
            .and_then(|view| view.identity.as_deref()),
        Some("visit_id"),
        "a qualified name finds the view too"
    );
    assert_eq!(read.view("visit.Other"), None);
}
