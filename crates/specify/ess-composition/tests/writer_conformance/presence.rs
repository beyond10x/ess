//! The presence and optionality half of a writer assertion, cell for cell.

use crate::common::Pair;

const TYPES: &str = "
types:
  - name: PREFIX.Required
    kind: struct
    fields:
      - { name: codec, type: String }
  - name: PREFIX.NullWhenAbsent
    kind: struct
    fields:
      - name: codec
        type: Optional<String>
        presence: null_when_absent
  - name: PREFIX.OmittedWhenAbsent
    kind: struct
    fields:
      - name: codec
        type: Optional<String>
        presence: omitted_when_absent
  - name: PREFIX.OmittedOrNull
    kind: struct
    fields:
      - name: codec
        type: Optional<String>
        presence: omitted_or_null
  - name: PREFIX.Unstated
    kind: struct
    fields:
      - { name: codec, type: Optional<String> }
  - name: PREFIX.Items
    kind: struct
    fields:
      - { name: items, type: List<String> }
  - name: PREFIX.ItemsOfOptional
    kind: struct
    fields:
      - { name: items, type: List<Optional<String>> }
";

/// The domain text of `<system>.session`, holding the structs both sides declare.
pub fn domain(system: &str) -> String {
    let prefix = format!("{system}.session");
    format!("domain: {prefix}\n{}", TYPES.replace("PREFIX", &prefix))
}

/// The two models of this file: the same structs under `relay.session` and `wire.session`.
pub fn pair() -> Pair {
    Pair::new(&domain("relay"), &domain("wire"))
}

pub fn writer_entry(local: &str, imported: &str) -> String {
    Pair::entry(
        &format!("type: relay.session.{local}"),
        &format!("type: wire.session.{imported}"),
        "    writer: true\n",
    )
}

pub fn plain_entry(local: &str, imported: &str) -> String {
    Pair::entry(
        &format!("type: relay.session.{local}"),
        &format!("type: wire.session.{imported}"),
        "",
    )
}

const FORMS: [&str; 5] = [
    "Required",
    "NullWhenAbsent",
    "OmittedWhenAbsent",
    "OmittedOrNull",
    "Unstated",
];

const fn cell(row: usize, column: usize, conforms: bool) -> (&'static str, &'static str, bool) {
    (FORMS[row], FORMS[column], conforms)
}

/// (local struct, imported struct, conforms): the specification's presence table, row by row.
const CELLS: [(&str, &str, bool); 25] = [
    cell(0, 0, true),
    cell(0, 1, true),
    cell(0, 2, true),
    cell(0, 3, true),
    cell(0, 4, true),
    cell(1, 0, false),
    cell(1, 1, true),
    cell(1, 2, false),
    cell(1, 3, true),
    cell(1, 4, false),
    cell(2, 0, false),
    cell(2, 1, false),
    cell(2, 2, true),
    cell(2, 3, true),
    cell(2, 4, true),
    cell(3, 0, false),
    cell(3, 1, false),
    cell(3, 2, false),
    cell(3, 3, true),
    cell(3, 4, false),
    cell(4, 0, false),
    cell(4, 1, false),
    cell(4, 2, true),
    cell(4, 3, true),
    cell(4, 4, true),
];

fn presence_spelling(form: &str) -> &'static str {
    match form {
        "NullWhenAbsent" => "`null_when_absent`",
        "OmittedWhenAbsent" => "`omitted_when_absent`",
        "OmittedOrNull" => "`omitted_or_null`",
        _ => "unstated",
    }
}

#[test]
fn the_presence_table_cell_by_cell() {
    let models = pair();
    let mut wrong = Vec::new();
    let mut conforming = 0;
    let mut drifting = 0;
    for (local, imported, conforms) in CELLS {
        let name = format!("{local} -> {imported}");
        let text = models.document("ess-composition/6", &writer_entry(local, imported));
        let result = models.compose(&text);
        if conforms {
            conforming += 1;
            if let Err(diagnostics) = result {
                wrong.push(format!("{name}: expected to compose, got {diagnostics}"));
            }
            continue;
        }
        drifting += 1;
        let Err(diagnostics) = result else {
            wrong.push(format!("{name}: expected drift, composed"));
            continue;
        };
        let all = diagnostics.as_slice();
        if all.len() != 1 || all[0].code().to_string() != "type_conformance_drift" {
            wrong.push(format!("{name}: expected one drift, got {diagnostics}"));
            continue;
        }
        let detail = all[0].detail();
        let expected = if imported == "Required" {
            "field `codec`: `Optional<String>` where the imported type has `String`".to_owned()
        } else {
            format!(
                "field `codec`: presence {} where `wire.session.{imported}` has {}",
                presence_spelling(local),
                presence_spelling(imported)
            )
        };
        if detail.matches("field `codec`").count() != 1 || !detail.contains(&expected) {
            wrong.push(format!("{name}: expected {expected:?} in {detail:?}"));
        }
    }
    assert_eq!((conforming, drifting), (14, 11));
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

#[test]
fn a_sender_that_always_writes_the_key_composes_where_a_plain_or_reader_entry_refuses() {
    let models = pair();
    models.composes(&models.document(
        "ess-composition/6",
        &writer_entry("NullWhenAbsent", "OmittedOrNull"),
    ));
    let wanted =
        "presence `null_when_absent` where `wire.session.OmittedOrNull` has `omitted_or_null`";
    let plain = models.drift(&models.document(
        "ess-composition/6",
        &plain_entry("NullWhenAbsent", "OmittedOrNull"),
    ));
    assert!(plain.contains(wanted), "{plain}");
    let reader = models.drift(&models.document(
        "ess-composition/6",
        &Pair::entry(
            "type: relay.session.NullWhenAbsent",
            "type: wire.session.OmittedOrNull",
            "    reader: true\n",
        ),
    ));
    assert!(reader.contains(wanted), "{reader}");
}

#[test]
fn a_local_optional_against_an_imported_required_field_is_drift_under_writer_and_admitted_plain() {
    let models = pair();
    models.composes(&models.document("ess-composition/6", &plain_entry("Unstated", "Required")));
    let detail =
        models.drift(&models.document("ess-composition/6", &writer_entry("Unstated", "Required")));
    assert!(
        detail.contains("field `codec`: `Optional<String>` where the imported type has `String`"),
        "{detail}"
    );
    models.composes(&models.document("ess-composition/6", &writer_entry("Required", "Unstated")));
    let detail =
        models.drift(&models.document("ess-composition/6", &plain_entry("Required", "Unstated")));
    assert!(
        detail.contains("field `codec`: `String` where the imported type has `Optional<String>`"),
        "{detail}"
    );
}

#[test]
fn a_field_with_no_presence_writes_only_a_missing_key_as_its_published_schema_reads_it() {
    let models = pair();
    for imported in ["OmittedWhenAbsent", "Unstated", "OmittedOrNull"] {
        models.composes(&models.document("ess-composition/6", &writer_entry("Unstated", imported)));
    }
    let detail = models.drift(&models.document(
        "ess-composition/6",
        &writer_entry("Unstated", "NullWhenAbsent"),
    ));
    assert!(
        detail.contains(
            "presence unstated where `wire.session.NullWhenAbsent` has `null_when_absent`"
        ),
        "{detail}"
    );
    for local in ["NullWhenAbsent", "OmittedOrNull"] {
        let detail =
            models.drift(&models.document("ess-composition/6", &writer_entry(local, "Unstated")));
        assert!(detail.contains("has unstated"), "{detail}");
    }
}

#[test]
fn optionality_holds_at_any_position() {
    let models = pair();
    models.composes(&models.document(
        "ess-composition/6",
        &writer_entry("Items", "ItemsOfOptional"),
    ));
    models.drift(&models.document(
        "ess-composition/6",
        &plain_entry("Items", "ItemsOfOptional"),
    ));
    let detail = models.drift(&models.document(
        "ess-composition/6",
        &writer_entry("ItemsOfOptional", "Items"),
    ));
    assert!(
        detail.contains("field `items`: `Optional<String>` where the imported type has `String`"),
        "{detail}"
    );
}
