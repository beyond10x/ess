//! `undeclared_fields` moving between `refused` and `ignored` (`ess/24`, beyond10x/ess#500) is a
//! typed change, never `system/<name>/unclassified-changed`.
//!
//! - On a command it is `command/<name>/response-undeclared-fields-changed`; on a struct type it
//!   is `type/<name>/undeclared-fields-changed`. Both kinds are `ess-diff/18` vocabulary.
//! - Opening a record (`refused` → `ignored`) widens the set of values it admits: `expanded`. A
//!   reader written against the closed record may now be sent a field it refuses, so it is
//!   breaking for readers of an output use and compatible for callers and history.
//! - Closing it narrows: `narrowed`. A caller still sending an extra member to an input use is
//!   refused, and so may be a stored value carrying one; a reader is sent less, which it already
//!   admitted. A command response is an output only, so closing it breaks nobody.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::{
    classified, diff, Compatibility, EssDelta, FailOn, Gate, RawEssDelta, SemanticRelation,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// `{command}`, `{record}` and `{input}` are the keys written on `PlaceOrder`, on `Extension`
/// (reached from the response) and on `Note` (reached from the input).
const MODEL: &str = "\
format: ess/24
system: catalog
version: v1
domain: catalog.orders
types:
  - name: catalog.orders.Extension
    kind: struct
{record}    fields:
      - name: vendor
        type: String
  - name: catalog.orders.Note
    kind: struct
{input}    fields:
      - name: text
        type: String
actors:
  - name: catalog.orders.Buyer
    may:
      - catalog.orders.PlaceOrder
commands:
  - name: catalog.orders.PlaceOrder
{command}    input:
      - name: item
        type: String
      - name: note
        type: catalog.orders.Note
    response:
      - name: order_ref
        type: String
      - name: extension
        type: catalog.orders.Extension
    outcomes:
      - name: placed
        returns: true
";

const RESPONSE: &str = "command/catalog.orders.PlaceOrder/response-undeclared-fields-changed";
const RECORD: &str = "type/catalog.orders.Extension/undeclared-fields-changed";
const INPUT: &str = "type/catalog.orders.Note/undeclared-fields-changed";
const UNCLASSIFIED: &str = "system/catalog/unclassified-changed";

#[derive(Clone, Copy, Default)]
struct Keys {
    command: Option<&'static str>,
    record: Option<&'static str>,
    input: Option<&'static str>,
}

fn ir(keys: Keys) -> EssIr {
    let key = |value: Option<&str>| {
        value.map_or_else(String::new, |value| {
            format!("    undeclared_fields: {value}\n")
        })
    };
    let text = MODEL
        .replace("{command}", &key(keys.command))
        .replace("{record}", &key(keys.record))
        .replace("{input}", &key(keys.input));
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn closed() -> EssIr {
    ir(Keys::default())
}

fn ids(delta: &EssDelta) -> Vec<String> {
    delta
        .changes()
        .iter()
        .map(|change| change.id().to_string())
        .collect()
}

fn answers(delta: &EssDelta, id: &str) -> (SemanticRelation, [Compatibility; 3]) {
    let change = delta
        .changes()
        .iter()
        .find(|change| change.id().to_string() == id)
        .unwrap_or_else(|| panic!("`{id}` in {}", delta.to_canonical_json()));
    let compatibility = delta.compatibility_of(&change.id()).expect("classified");
    (
        change.relation(),
        [
            compatibility.callers(),
            compatibility.readers(),
            compatibility.history(),
        ],
    )
}

use Compatibility::{Breaking as B, Compatible as C};

#[test]
fn opening_a_command_response_is_expanded_and_breaking_for_readers_only() {
    let opened = ir(Keys {
        command: Some("ignored"),
        ..Keys::default()
    });
    let delta = classified(&closed(), &opened).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(ids(&delta), [RESPONSE], "{json}");
    assert_eq!(
        answers(&delta, RESPONSE),
        (SemanticRelation::Expanded, [C, B, C]),
        "{json}"
    );
    let text = ess_diff::render::text(&delta);
    assert!(text.contains("refused → ignored"), "{text}");
    let outcome = Gate::new(FailOn::Breaking).judge(&delta, None).unwrap();
    assert!(!outcome.passed(), "{json}");
}

#[test]
fn closing_a_command_response_is_narrowed_and_compatible() {
    let opened = ir(Keys {
        command: Some("ignored"),
        ..Keys::default()
    });
    let delta = classified(&opened, &closed()).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(ids(&delta), [RESPONSE], "{json}");
    assert_eq!(
        answers(&delta, RESPONSE),
        (SemanticRelation::Narrowed, [C, C, C]),
        "{json}"
    );
    let outcome = Gate::new(FailOn::BreakingOrUnknown)
        .judge(&delta, None)
        .unwrap();
    assert!(outcome.passed(), "{json}");
}

#[test]
fn opening_a_struct_is_read_through_where_it_is_used() {
    let opened = ir(Keys {
        record: Some("ignored"),
        input: Some("ignored"),
        ..Keys::default()
    });
    let delta = classified(&closed(), &opened).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(ids(&delta), [RECORD, INPUT], "{json}");
    // Reached from the response: a reader of the closed record may now be sent a field it refuses.
    assert_eq!(
        answers(&delta, RECORD),
        (SemanticRelation::Expanded, [C, B, C]),
        "{json}"
    );
    // Reached from the input: the system admits more of what a caller sends.
    assert_eq!(
        answers(&delta, INPUT),
        (SemanticRelation::Expanded, [C, C, C]),
        "{json}"
    );
}

#[test]
fn closing_a_struct_breaks_the_callers_of_its_input_use() {
    let opened = ir(Keys {
        record: Some("ignored"),
        input: Some("ignored"),
        ..Keys::default()
    });
    let delta = classified(&opened, &closed()).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(ids(&delta), [RECORD, INPUT], "{json}");
    assert_eq!(
        answers(&delta, RECORD),
        (SemanticRelation::Narrowed, [C, C, C]),
        "{json}"
    );
    assert_eq!(
        answers(&delta, INPUT),
        (SemanticRelation::Narrowed, [B, C, C]),
        "{json}"
    );
}

#[test]
fn the_default_delta_carries_it_as_ess_diff_18_and_earlier_formats_refuse_it() {
    let opened = ir(Keys {
        command: Some("ignored"),
        record: Some("ignored"),
        ..Keys::default()
    });
    let delta = diff(&closed(), &opened).unwrap();
    let json = delta.to_canonical_json();
    assert_eq!(delta.format.to_string(), "ess-diff/18", "{json}");
    assert_eq!(ids(&delta), [RECORD, RESPONSE], "{json}");
    assert!(!ids(&delta).contains(&UNCLASSIFIED.to_owned()), "{json}");
    assert!(
        delta.compatibility().is_some(),
        "an ess-diff/18 delta is classified: {json}"
    );

    let raw: RawEssDelta = serde_json::from_str(&json).unwrap();
    assert_eq!(EssDelta::try_from(raw).unwrap(), delta, "it reads back");

    for id in [RECORD, RESPONSE] {
        assert!(json.contains(id), "{json}");
    }
    let refused = delta
        .to_canonical_json_for("ess-diff/17".parse().unwrap())
        .expect_err("ess-diff/17 cannot write it");
    assert!(
        refused.to_string().contains("not representable"),
        "{refused}"
    );

    let mut older: serde_json::Value = serde_json::from_str(&json).unwrap();
    older["format"] = serde_json::Value::from("ess-diff/17");
    let raw: RawEssDelta = serde_json::from_value(older).unwrap();
    let refused = EssDelta::try_from(raw).expect_err("an `ess-diff/17` reader refuses it");
    assert!(
        refused.to_string().contains("outside the vocabulary"),
        "{refused}"
    );
}

#[test]
fn writing_the_default_is_no_change() {
    let refused = ir(Keys {
        command: Some("refused"),
        record: Some("refused"),
        input: Some("refused"),
    });
    let delta = diff(&closed(), &refused).unwrap();
    assert!(delta.is_empty(), "{}", delta.to_canonical_json());
}

/// A struct declared open in the later revision only is `type/<name>/added`, which stands for all
/// of its content: no undeclared-fields change beside it, and no residual.
#[test]
fn an_added_open_struct_is_only_added() {
    let text = MODEL
        .replace("{command}", "")
        .replace("{record}", "")
        .replace("{input}", "")
        .replace(
            "actors:\n",
            "  - name: catalog.orders.Spare\n    kind: struct\n    undeclared_fields: ignored\n    \
             fields:\n      - name: label\n        type: String\nactors:\n",
        );
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("catalog.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let after = compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"));
    let delta = diff(&closed(), &after).unwrap();
    assert_eq!(
        ids(&delta),
        ["type/catalog.orders.Spare/added"],
        "{}",
        delta.to_canonical_json()
    );
}

/// A struct both revisions declare open, and a command both open: no change, and no residual.
#[test]
fn an_unmoved_open_struct_raises_nothing() {
    let opened = ir(Keys {
        command: Some("ignored"),
        record: Some("ignored"),
        input: Some("ignored"),
    });
    let delta = diff(&opened, &opened).unwrap();
    assert!(delta.is_empty(), "{}", delta.to_canonical_json());
}
