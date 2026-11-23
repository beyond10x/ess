//! The narrowing half of a writer assertion, one group of tests per row of the table.

use std::fmt::Write;

use crate::common::Pair;

/// A block field of a struct, at the indent of a `fields:` list item.
fn field(name: &str, type_ref: &str, extra: &str) -> String {
    format!("      - name: {name}\n        type: {type_ref}\n{extra}")
}

/// The domain text of `<system>.session`: `types` (each `@` reads as the domain name) and, when
/// `holder` is not empty, the struct `Holder` with those fields.
fn domain(system: &str, types: &str, holder: &str) -> String {
    let prefix = format!("{system}.session");
    let mut text = format!("domain: {prefix}\n\ntypes:\n{types}");
    if !holder.is_empty() {
        let _ = write!(
            text,
            "  - name: @.Holder\n    kind: struct\n    fields:\n{holder}"
        );
    }
    text.replace('@', &prefix)
}

/// Two models and the two ends an entry of them compares.
struct Models {
    pair: Pair,
    local: String,
    imported: String,
}

impl Models {
    /// `Holder` against `Holder`, each with its types and its one field.
    fn holders(
        (local_types, local_field): (&str, String),
        (wire_types, wire_field): (&str, String),
    ) -> Self {
        Self::types(
            (local_types, local_field),
            (wire_types, wire_field),
            "Holder",
            "Holder",
        )
    }

    /// The named type of each side as the ends.
    fn types(
        (local_types, local_field): (&str, String),
        (wire_types, wire_field): (&str, String),
        local: &str,
        imported: &str,
    ) -> Self {
        Self {
            pair: Pair::new(
                &domain("relay", local_types, &local_field),
                &domain("wire", wire_types, &wire_field),
            ),
            local: format!("type: relay.session.{local}"),
            imported: format!("type: wire.session.{imported}"),
        }
    }

    fn document(&self, format: &str, extra: &str) -> String {
        self.pair
            .document(format, &Pair::entry(&self.local, &self.imported, extra))
    }

    fn writer(&self) -> String {
        self.document("ess-composition/6", "    writer: true\n")
    }

    fn plain(&self) -> String {
        self.document("ess-composition/6", "")
    }

    /// Composes as a writer, and is drift as a plain entry.
    fn admitted(&self) {
        self.pair.composes(&self.writer());
        self.pair.drift(&self.plain());
    }

    /// The detail of the drift a writer entry is refused with.
    fn refused(&self) -> String {
        self.pair.drift(&self.writer())
    }
}

const LOCAL_NEWTYPES: &str = "
  - name: @.Inner
    kind: newtype
    of: String
    alphabet: \"ab\"
  - name: @.Outer
    kind: newtype
    of: relay.session.Inner
  - name: @.Count
    kind: newtype
    of: Integer
  - name: @.Status
    kind: enum
    variants:
      - { name: Open, wire: open }
";

const WIRE_REF: &str = "
  - name: @.Ref
    kind: newtype
    of: String
";

#[test]
fn a_newtype_chain_is_admitted_over_what_the_imported_reference_is() {
    let chain = Models::holders(
        (LOCAL_NEWTYPES, field("code", "relay.session.Outer", "")),
        ("", field("code", "String", "")),
    );
    chain.admitted();

    let count = Models::holders(
        (LOCAL_NEWTYPES, field("code", "relay.session.Count", "")),
        ("", field("code", "String", "")),
    );
    let detail = count.refused();
    for part in ["field `code`", "Integer", "String"] {
        assert!(detail.contains(part), "{part}: {detail}");
    }

    let string = Models::holders(
        (LOCAL_NEWTYPES, field("code", "String", "")),
        (WIRE_REF, field("code", "wire.session.Ref", "")),
    );
    let detail = string.refused();
    assert!(detail.contains("field `code`"), "{detail}");

    let enumeration = Models::holders(
        (LOCAL_NEWTYPES, field("code", "relay.session.Status", "")),
        (WIRE_REF, field("code", "wire.session.Ref", "")),
    );
    let detail = enumeration.refused();
    assert!(detail.contains("field `code`"), "{detail}");
}

const WIRE_STATUS: &str = "
  - name: @.Status
    kind: enum
    variants:
      - { name: Open, wire: open }
";

#[test]
fn an_enum_is_admitted_where_the_imported_reference_is_string() {
    Models::holders(
        (LOCAL_NEWTYPES, field("status", "relay.session.Status", "")),
        ("", field("status", "String", "")),
    )
    .admitted();

    let reverse = Models::holders(
        ("", field("status", "String", "")),
        (WIRE_STATUS, field("status", "wire.session.Status", "")),
    );
    assert!(reverse.refused().contains("field `status`"));

    let newtype = Models::holders(
        (LOCAL_NEWTYPES, field("status", "relay.session.Status", "")),
        (WIRE_REF, field("status", "wire.session.Ref", "")),
    );
    assert!(newtype.refused().contains("field `status`"));
}

fn status(variants: &[(&str, &str)]) -> String {
    let mut text = "\n  - name: @.Status\n    kind: enum\n    variants:\n".to_owned();
    for (name, wire) in variants {
        let _ = writeln!(text, "      - {{ name: {name}, wire: {wire} }}");
    }
    text
}

#[test]
fn a_subset_of_the_enum_variants_is_admitted_by_wire_name() {
    let imported = status(&[
        ("Open", "open"),
        ("Closed", "closed"),
        ("Pending", "pending"),
    ]);
    let models = |local: &[(&str, &str)]| {
        Models::types(
            (&status(local), String::new()),
            (&imported, String::new()),
            "Status",
            "Status",
        )
    };

    let subset = models(&[("Open", "open"), ("Closed", "closed")]);
    subset.admitted();
    let reader = subset.document("ess-composition/6", "    reader: true\n");
    let detail = subset.pair.drift(&reader);
    assert!(detail.contains("lacks wire names"), "{detail}");

    let extra = models(&[("Open", "open"), ("Extra", "extra")]);
    let detail = extra.refused();
    assert!(
        detail.contains("wire names") && detail.contains("extra"),
        "{detail}"
    );

    let live = models(&[("Live", "open")]);
    live.pair.composes(&live.writer());
}

const WIRE_ITEM: &str = "
  - name: @.Item
    kind: union
    tag: kind
    variants:
      text: String
      count: Integer
      gone:
";

fn item(tag: &str, variants: &str) -> String {
    format!("\n  - name: @.Item\n    kind: union\n    tag: {tag}\n    variants:\n{variants}")
}

#[test]
fn a_subset_of_the_union_variants_is_admitted_under_the_same_tag() {
    let models = |local: String| {
        Models::types(
            (&local, String::new()),
            (WIRE_ITEM, String::new()),
            "Item",
            "Item",
        )
    };
    models(item("kind", "      text: String\n      gone:\n")).admitted();

    let detail = models(item("kind", "      text: String\n      extra: String\n")).refused();
    assert!(detail.contains("variants"), "{detail}");

    let detail = models(item("type", "      text: String\n      gone:\n")).refused();
    assert!(detail.contains("tag "), "{detail}");

    let detail = models(item("kind", "      text: Optional<String>\n      gone:\n")).refused();
    assert!(detail.contains("text"), "{detail}");

    let detail = models(item("kind", "      text: String\n      gone: String\n")).refused();
    assert!(detail.contains("a payload where"), "{detail}");
}

const LOCAL_OBJECTS: &str = "
  - name: @.Meta
    kind: struct
    fields:
      - { name: a, type: String }
  - name: @.MetaBox
    kind: newtype
    of: relay.session.Meta
  - name: @.Status
    kind: enum
    variants:
      - { name: Open, wire: open }
  - name: @.Item
    kind: union
    tag: kind
    variants:
      text: String
";

#[test]
fn a_struct_or_a_string_keyed_map_is_admitted_where_the_imported_reference_is_any_object() {
    let against = |local: &str| {
        Models::holders(
            (LOCAL_OBJECTS, field("extra", local, "")),
            ("", field("extra", "Map<String, Json>", "")),
        )
    };
    for admitted in [
        "relay.session.Meta",
        "Map<String, String>",
        "relay.session.MetaBox",
    ] {
        let models = against(admitted);
        models.pair.composes(&models.writer());
        models.pair.drift(&models.plain());
    }
    for refused in [
        "Json",
        "List<String>",
        "Optional<relay.session.Meta>",
        "relay.session.Status",
        "relay.session.Item",
        "Map<Integer, String>",
    ] {
        let detail = against(refused).refused();
        assert!(detail.contains("field `extra`"), "{refused}: {detail}");
    }
    let bare = Models::holders(
        (LOCAL_OBJECTS, field("extra", "relay.session.Meta", "")),
        ("", field("extra", "Json", "")),
    );
    assert!(bare.refused().contains("field `extra`"));
}

const OMISSION_FIELDS: [(&str, &str, &str); 5] = [
    ("req", "String", ""),
    (
        "nul",
        "Optional<String>",
        "        presence: null_when_absent\n",
    ),
    (
        "omit",
        "Optional<String>",
        "        presence: omitted_when_absent\n",
    ),
    (
        "either",
        "Optional<String>",
        "        presence: omitted_or_null\n",
    ),
    ("none", "Optional<String>", ""),
];

fn fields_struct(without: Option<&str>) -> String {
    let mut text = "\n  - name: @.Fields\n    kind: struct\n    fields:\n".to_owned();
    for (name, type_ref, extra) in OMISSION_FIELDS {
        if Some(name) != without {
            text.push_str(&field(name, type_ref, extra));
        }
    }
    text
}

#[test]
fn an_imported_field_may_be_left_out_only_where_its_presence_admits_omission() {
    let mut wrong = Vec::new();
    for (name, _, _) in OMISSION_FIELDS {
        let models = Models::types(
            (&fields_struct(Some(name)), String::new()),
            (&fields_struct(None), String::new()),
            "Fields",
            "Fields",
        );
        let admitted = matches!(name, "omit" | "either" | "none");
        match (admitted, models.pair.compose(&models.writer())) {
            (true, Ok(_)) => {}
            (false, Err(diagnostics))
                if diagnostics.to_string().contains(&format!("field `{name}`")) => {}
            (admitted, outcome) => wrong.push(format!(
                "{name}: expected admitted={admitted}, got {:?}",
                outcome.map(|_| ()).map_err(|error| error.to_string())
            )),
        }
        if models.pair.compose(&models.plain()).is_ok() {
            wrong.push(format!("{name}: a plain entry composed"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

const OPEN_RECORD: &str = "
  - name: @.OpenRecord
    kind: struct
    undeclared_fields: ignored
    fields:
      - { name: id, type: String }
  - name: @.ClosedRecord
    kind: struct
    fields:
      - { name: id, type: String }
";

const RECORD: &str = "
  - name: @.Record
    kind: struct
    fields:
      - { name: id, type: String }
      - { name: memo, type: Integer }
  - name: @.JsonRecord
    kind: struct
    fields:
      - { name: id, type: String }
      - { name: memo, type: Json }
";

#[test]
fn an_extra_field_is_admitted_only_against_a_struct_that_ignores_undeclared_fields() {
    let holder = |local: &str, imported: &str| {
        Models::holders(
            (
                RECORD,
                field("record", &format!("relay.session.{local}"), ""),
            ),
            (
                OPEN_RECORD,
                field("record", &format!("wire.session.{imported}"), ""),
            ),
        )
    };
    holder("Record", "OpenRecord").admitted();
    holder("JsonRecord", "OpenRecord").admitted();
    let detail = holder("Record", "ClosedRecord").refused();
    assert!(detail.contains("memo"), "{detail}");

    let end = |local: &str, imported: &str| {
        Models::types(
            (RECORD, String::new()),
            (OPEN_RECORD, String::new()),
            local,
            imported,
        )
    };
    end("Record", "OpenRecord").admitted();
    let detail = end("Record", "ClosedRecord").refused();
    assert!(detail.contains("field `memo`"), "{detail}");
}

fn command(name: &str, ignores: bool, response: &str) -> String {
    format!(
        "  - name: {name}\n{}    input:\n      - {{ name: channel, type: String }}\n    \
         response:\n{response}    outcomes:\n      - name: done\n        returns: true\n",
        if ignores {
            "    undeclared_fields: ignored\n"
        } else {
            ""
        }
    )
}

fn event(name: &str, fields: &str) -> String {
    format!("  - name: {name}\n    fields:\n{fields}")
}

#[test]
fn an_extra_field_against_a_command_response_follows_the_commands_own_key() {
    let id = field("id", "String", "");
    let both = format!("{id}{}", field("memo", "Integer", ""));
    let relay = format!(
        "domain: relay.session\n\nevents:\n{}\ncommands:\n{}{}",
        event("relay.session.Opened", &both),
        command("relay.session.Open", false, &both),
        command("relay.session.Close", false, &both),
    );
    let wire = format!(
        "domain: wire.session\n\nevents:\n{}\ncommands:\n{}{}",
        event("wire.session.Opened", &id),
        command("wire.session.Open", true, &id),
        command("wire.session.Close", false, &id),
    );
    let pair = Pair::with_surface(
        &relay,
        (
            &["relay.session.Open", "relay.session.Close"],
            &["relay.session.Opened"],
        ),
        &wire,
        (
            &["wire.session.Open", "wire.session.Close"],
            &["wire.session.Opened"],
        ),
    );
    let document = |local: &str, imported: &str| {
        pair.document(
            "ess-composition/6",
            &Pair::entry(local, imported, "    writer: true\n"),
        )
    };
    pair.composes(&document(
        "command: relay.session.Open, part: response",
        "command: wire.session.Open, part: response",
    ));
    let detail = pair.drift(&document(
        "command: relay.session.Close, part: response",
        "command: wire.session.Close, part: response",
    ));
    assert!(detail.contains("field `memo`"), "{detail}");
    let detail = pair.drift(&document(
        "event: relay.session.Opened",
        "event: wire.session.Opened",
    ));
    assert!(detail.contains("field `memo`"), "{detail}");
}

const WIRE_NOTE: &str = "
  - name: @.OpenRecord
    kind: struct
    undeclared_fields: ignored
    fields:
      - { name: id, type: String }
      - name: note
        type: Optional<String>
        presence: omitted_or_null
";

fn keyed_record(memo_type: &str) -> String {
    format!(
        "\n  - name: @.Record\n    kind: struct\n    fields:\n      - {{ name: id, type: String }}\n\
         \x20     - name: memo\n        type: {memo_type}\n        wire: note\n"
    )
}

#[test]
fn an_extra_field_that_travels_as_an_imported_key_is_compared_with_that_field() {
    let models = |memo_type: &str| {
        Models::types(
            (&keyed_record(memo_type), String::new()),
            (WIRE_NOTE, String::new()),
            "Record",
            "OpenRecord",
        )
    };
    let detail = models("Integer").refused();
    assert!(detail.contains("field `memo`"), "{detail}");
    models("String").admitted();
}

fn newtype(
    name: &str,
    alphabet: Option<&str>,
    prefix: Option<&str>,
    invariants: &[&str],
) -> String {
    let mut text = format!("\n  - name: @.{name}\n    kind: newtype\n    of: String\n");
    if let Some(alphabet) = alphabet {
        let _ = writeln!(text, "    alphabet: \"{alphabet}\"");
    }
    if let Some(prefix) = prefix {
        let _ = writeln!(text, "    prefix: \"{prefix}\"");
    }
    if !invariants.is_empty() {
        text.push_str("    invariants:\n");
        for invariant in invariants {
            let _ = writeln!(text, "      - '{invariant}'");
        }
    }
    text
}

/// (local constraints, imported constraints, whether a writer conforms, a word the drift holds).
type Cell<'a> = (
    (Option<&'a str>, Option<&'a str>, &'a [&'a str]),
    (Option<&'a str>, Option<&'a str>, &'a [&'a str]),
    bool,
);

const X: &str = "value != \"x\"";
const Y: &str = "value != \"y\"";

fn cells() -> Vec<Cell<'static>> {
    let none: &[&str] = &[];
    vec![
        (
            (Some("abr-"), None, none),
            (Some("abcr-"), None, none),
            true,
        ),
        (
            (Some("abcdr-"), None, none),
            (Some("abcr-"), None, none),
            false,
        ),
        ((None, None, none), (Some("abcr-"), None, none), false),
        ((Some("ab"), None, none), (None, None, none), true),
        ((None, Some("r-x"), none), (None, Some("r-"), none), true),
        ((None, Some("r"), none), (None, Some("r-"), none), false),
        ((None, Some("q-"), none), (None, Some("r-"), none), false),
        ((None, None, none), (None, Some("r-"), none), false),
        ((None, None, &[X, Y]), (None, None, &[X]), true),
        ((None, None, &[Y]), (None, None, &[X]), false),
        ((None, None, none), (None, None, &[X]), false),
        ((None, None, &[X]), (None, None, none), true),
    ]
}

#[test]
fn value_constraints_narrow_for_a_writer() {
    let mut wrong = Vec::new();
    for (index, (local, imported, conforms)) in cells().into_iter().enumerate() {
        let models = Models::holders(
            (
                &newtype("Token", local.0, local.1, local.2),
                field("token", "@.Token", ""),
            ),
            (
                &newtype("Token", imported.0, imported.1, imported.2),
                field("token", "@.Token", ""),
            ),
        );
        let writer = models.pair.compose(&models.writer());
        if writer.is_ok() != conforms {
            wrong.push(format!("cell {index}: writer composes {}", writer.is_ok()));
        }
        if !conforms
            && !models
                .pair
                .drift(&models.writer())
                .contains("field `token`")
        {
            wrong.push(format!("cell {index}: drift names no field"));
        }
        if models.pair.compose(&models.plain()).is_ok() {
            wrong.push(format!("cell {index}: a plain entry composed"));
        }
        let five = models.document("ess-composition/5", "");
        if models.pair.compose(&five).is_ok() {
            wrong.push(format!("cell {index}: /5 composed unequal constraints"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));

    let equal = Models::holders(
        (
            &newtype("Token", Some("ab"), Some("a"), &[X]),
            field("token", "@.Token", ""),
        ),
        (
            &newtype("Token", Some("ab"), Some("a"), &[X]),
            field("token", "@.Token", ""),
        ),
    );
    equal
        .pair
        .composes(&equal.document("ess-composition/5", ""));
}

fn window(invariants: &[&str]) -> String {
    let mut text = "\n  - name: @.Window\n    kind: struct\n    fields:\n      - { name: low, type: Integer }\n      - { name: high, type: Integer }\n".to_owned();
    if !invariants.is_empty() {
        text.push_str("    invariants:\n");
        for invariant in invariants {
            let _ = writeln!(text, "      - '{invariant}'");
        }
    }
    text
}

#[test]
fn struct_invariants_narrow_for_a_writer() {
    let models = |local: &[&str]| {
        Models::types(
            (&window(local), String::new()),
            (&window(&["low <= high"]), String::new()),
            "Window",
            "Window",
        )
    };
    models(&["low <= high", "low >= 0"]).admitted();
    let detail = models(&[]).refused();
    assert!(detail.contains("invariant"), "{detail}");
}

fn kind(attributes: &str, variants: &[(&str, Option<&str>)]) -> String {
    let mut text = format!("\n  - name: @.Kind\n    kind: enum\n{attributes}    variants:\n");
    for (name, terminal) in variants {
        match terminal {
            Some(value) => {
                let _ = writeln!(
                    text,
                    "      - {{ name: {name}, attributes: {{ terminal: {value} }} }}"
                );
            }
            None => {
                let _ = writeln!(text, "      - {{ name: {name} }}");
            }
        }
    }
    text
}

const TERMINAL: &str = "    attributes:\n      - { name: terminal, type: Boolean }\n";

#[test]
fn variant_attributes_stay_exact_for_a_writer() {
    let imported = kind(
        TERMINAL,
        &[("Open", Some("false")), ("Closed", Some("true"))],
    );
    let models = |local: String| {
        Models::types(
            (&local, String::new()),
            (&imported, String::new()),
            "Kind",
            "Kind",
        )
    };
    models(kind(TERMINAL, &[("Open", Some("false"))])).admitted();

    let detail = models(kind("", &[("Open", None)])).refused();
    assert!(detail.contains("attributes"), "{detail}");

    let detail = models(kind(TERMINAL, &[("Open", Some("true"))])).refused();
    assert!(
        detail.contains("variant") && detail.contains("terminal"),
        "{detail}"
    );

    let more = format!("{TERMINAL}      - {{ name: handled, type: Optional<String> }}\n");
    let detail = models(kind(&more, &[("Open", Some("false"))])).refused();
    assert!(detail.contains("attributes"), "{detail}");
}

#[test]
fn a_plain_v6_entry_keeps_its_verdicts() {
    let optional = Models::holders(
        ("", field("code", "Optional<String>", "")),
        ("", field("code", "String", "")),
    );
    optional.pair.composes(&optional.plain());
    optional.pair.drift(&optional.writer());

    let same = Models::holders(
        ("", field("code", "String", "")),
        ("", field("code", "String", "")),
    );
    same.pair.composes(&same.plain());
    same.pair.composes(&same.writer());

    let equal = Models::holders(
        (LOCAL_NEWTYPES, field("status", "relay.session.Status", "")),
        (WIRE_STATUS, field("status", "wire.session.Status", "")),
    );
    equal.pair.composes(&equal.plain());
    equal.pair.composes(&equal.writer());
}
