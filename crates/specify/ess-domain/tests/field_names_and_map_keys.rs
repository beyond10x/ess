//! Two spellings a retrofit needs and the parser used to refuse.
//!
//! * beyond10x/ess#141 — a field whose wire name begins with an underscore (`_url`). The name is
//!   admitted when underscores are followed by a letter; the generators map it to an identifier
//!   of their own language and the wire keeps `_url`.
//! * beyond10x/ess#143 — a map keyed by a newtype of a key primitive (`Map<ItemId, Boolean>`,
//!   `ItemId` a newtype of `String`). A newtype of a string has a string's wire form, so the
//!   reason a structured key is refused does not apply. The key is resolved to its primitive
//!   while the document is read, and the rest of the pipeline sees `Map<String, Boolean>`.

use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_domain::types::{Field, MapKeyNewtypes, Primitive, TypeRef};

const HEADER: &str = "format: ess/14\nsystem: demo\nversion: v1\ndomain: demo.orders\n";

/// Every document read with the key newtypes all of them declare in view, which is what a loader
/// reading a specification's files together does.
fn assemble(documents: &[(&str, &str)]) -> Result<Specification, String> {
    let keys = MapKeyNewtypes::from_documents(documents.iter().map(|(_, text)| *text));
    let mut files = Vec::new();
    for (label, text) in documents {
        let raw = keys
            .scope(|| RawSpecFile::parse(text))
            .map_err(|error| error.to_string())?;
        files.push((Source::new(*label), raw));
    }
    Specification::assemble(files).map_err(|errors| errors.to_string())
}

fn single(text: &str) -> Result<Specification, String> {
    assemble(&[("orders.yaml", text)])
}

// ---- #141 -------------------------------------------------------------------------------------

const ISSUE_141: &str = "types:\n  - name: demo.orders.OrderView\n    kind: struct\n    fields:\n      \
                         - {name: _url, type: Optional<String>}\n      - {name: id, type: String}\n";

#[test]
fn issue_141_a_field_name_may_begin_with_an_underscore() {
    let spec = single(&format!("{HEADER}{ISSUE_141}")).expect("#141's repro validates");
    let declared = spec
        .system()
        .types
        .get(&"demo.orders.OrderView".parse().unwrap())
        .expect("declared");
    let field = declared.field("_url").expect("the field keeps its name");
    assert_eq!(field.naming.wire.as_deref().unwrap_or(&field.name), "_url");
}

#[test]
fn leading_underscores_are_admitted_only_before_a_letter() {
    for spelling in ["_url", "__v", "_Url", "_a1_b"] {
        serde_yaml::from_str::<Field>(&format!("name: {spelling}\ntype: String\n"))
            .unwrap_or_else(|error| panic!("{spelling:?} is a field name: {error}"));
    }
    for spelling in ["_", "__", "_1", "_-a", "1_a"] {
        let error = serde_yaml::from_str::<Field>(&format!("name: {spelling:?}\ntype: String\n"))
            .expect_err(spelling);
        assert!(
            error.to_string().contains("field name"),
            "{spelling:?}: {error}"
        );
    }
}

#[test]
fn the_published_field_pattern_admits_exactly_what_the_parser_admits() {
    let pattern = regex_lite(Field::PATTERN);
    for spelling in ["_url", "__v", "invoice_id", "_", "_1", "1a", "a-b", ""] {
        let parsed =
            serde_yaml::from_str::<Field>(&format!("name: {spelling:?}\ntype: String\n")).is_ok();
        assert_eq!(pattern(spelling), parsed, "{spelling:?}");
    }
}

/// The one pattern this file checks, `^_*[A-Za-z][A-Za-z0-9_]*$`, as a predicate — asserted equal
/// to [`Field::PATTERN`] so a change to either is seen here.
fn regex_lite(pattern: &str) -> impl Fn(&str) -> bool {
    assert_eq!(pattern, "^_*[A-Za-z][A-Za-z0-9_]*$");
    |value: &str| {
        let rest = value.trim_start_matches('_');
        let mut chars = rest.chars();
        matches!(chars.next(), Some(c) if c.is_ascii_alphabetic())
            && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
    }
}

// ---- #143 -------------------------------------------------------------------------------------

const ISSUE_143: &str =
    "types:\n  - {name: demo.orders.ItemId, kind: newtype, of: String}\nevents:\n  \
                         - name: demo.orders.Checked\n    fields:\n      \
                         - {name: results, type: \"Map<demo.orders.ItemId, Boolean>\"}\n";

fn results_type(spec: &Specification) -> TypeRef {
    let event = spec
        .events()
        .get(&"demo.orders.Checked".parse().unwrap())
        .expect("declared");
    event
        .fields
        .iter()
        .find(|field| field.name == "results")
        .expect("the field")
        .type_ref
        .clone()
}

#[test]
fn issue_143_a_map_key_may_be_a_newtype_of_a_key_primitive() {
    let spec = single(&format!("{HEADER}{ISSUE_143}")).expect("#143's repro validates");
    assert_eq!(
        results_type(&spec),
        TypeRef::parse("Map<String, Boolean>").unwrap(),
        "the key resolves to the primitive it is spelled with on the wire"
    );
}

#[test]
fn a_key_newtype_resolves_through_a_chain_of_newtypes() {
    let text = format!(
        "{HEADER}types:\n  - {{name: demo.orders.Sku, kind: newtype, of: Integer}}\n  - {{name: \
         demo.orders.ItemId, kind: newtype, of: demo.orders.Sku}}\nevents:\n  - name: \
         demo.orders.Checked\n    fields:\n      - {{name: results, type: \"Map<demo.orders.ItemId, \
         Boolean>\"}}\n"
    );
    let spec = single(&text).expect("a newtype of a newtype of Integer is a key");
    assert_eq!(
        results_type(&spec),
        TypeRef::parse("Map<Integer, Boolean>").unwrap()
    );
}

#[test]
fn a_key_newtype_declared_in_another_file_resolves_when_the_files_are_read_together() {
    let types =
        format!("{HEADER}types:\n  - {{name: demo.orders.ItemId, kind: newtype, of: String}}\n");
    let events =
        "domain: demo.orders\nevents:\n  - name: demo.orders.Checked\n    fields:\n      - \
                  {name: results, type: \"Map<demo.orders.ItemId, Boolean>\"}\n";
    let spec = assemble(&[("types.yaml", &types), ("events.yaml", events)])
        .expect("a newtype declared in a sibling file is a key");
    assert_eq!(
        results_type(&spec),
        TypeRef::parse("Map<String, Boolean>").unwrap()
    );
}

#[test]
fn a_key_that_is_not_a_newtype_of_a_key_primitive_is_still_refused() {
    for (declaration, why) in [
        (
            "  - name: demo.orders.ItemId\n    kind: struct\n    fields: [{name: id, type: String}]\n",
            "a struct has no stable wire form as a key",
        ),
        (
            "  - {name: demo.orders.ItemId, kind: newtype, of: List<String>}\n",
            "a list is not a key",
        ),
        (
            "  - {name: demo.orders.ItemId, kind: newtype, of: Binary64}\n",
            "Binary64 keys have no admitted wire spelling",
        ),
    ] {
        let text = format!(
            "{HEADER}types:\n{declaration}events:\n  - name: demo.orders.Checked\n    fields:\n      \
             - {{name: results, type: \"Map<demo.orders.ItemId, Boolean>\"}}\n"
        );
        let error = single(&text).expect_err(why);
        assert!(error.contains("map key"), "{why}: {error}");
    }

    let undeclared = format!(
        "{HEADER}events:\n  - name: demo.orders.Checked\n    fields:\n      - {{name: results, \
         type: \"Map<demo.orders.ItemId, Boolean>\"}}\n"
    );
    let error = single(&undeclared).expect_err("an undeclared key is not a key");
    assert!(error.contains("map key"), "{error}");
}

#[test]
fn a_newtype_cycle_does_not_hang_the_key_resolution() {
    let text = format!(
        "{HEADER}types:\n  - {{name: demo.orders.A, kind: newtype, of: demo.orders.B}}\n  - {{name: \
         demo.orders.B, kind: newtype, of: demo.orders.A}}\nevents:\n  - name: demo.orders.Checked\n    \
         fields:\n      - {{name: results, type: \"Map<demo.orders.A, Boolean>\"}}\n"
    );
    single(&text).expect_err("a cycle resolves to nothing");
}

#[test]
fn outside_a_scope_a_named_key_is_refused_as_before_and_a_scope_does_not_leak() {
    let spelling = "Map<demo.orders.ItemId, Boolean>";
    TypeRef::parse(spelling).expect_err("no declarations in view");

    let keys = MapKeyNewtypes::from_documents([format!("{HEADER}{ISSUE_143}").as_str()]);
    assert_eq!(
        keys.scope(|| TypeRef::parse(spelling)),
        Ok(TypeRef::parse("Map<String, Boolean>").unwrap())
    );

    let nested = keys.scope(|| MapKeyNewtypes::default().scope(|| TypeRef::parse(spelling)));
    nested.expect_err("an inner scope replaces the outer one");
    assert!(
        keys.scope(|| TypeRef::parse(spelling)).is_ok(),
        "the outer scope is restored"
    );
    TypeRef::parse(spelling).expect_err("the scope ended with its closure");
}

#[test]
fn a_reader_of_one_file_adds_its_own_declarations_to_those_in_view() {
    let item: ess_domain::name::QualifiedName = "demo.orders.ItemId".parse().unwrap();
    let sibling = MapKeyNewtypes::from_documents([
        "types:\n  - {name: demo.orders.Sku, kind: newtype, of: String}\n",
    ]);
    let file: serde_yaml::Value = serde_yaml::from_str(
        "types:\n  - {name: demo.orders.ItemId, kind: newtype, of: demo.orders.Sku}\n",
    )
    .unwrap();

    assert_eq!(MapKeyNewtypes::current(), MapKeyNewtypes::default());
    let merged = sibling.scope(|| MapKeyNewtypes::current().with_value(&file));
    assert_eq!(
        merged.get(&item),
        Some(Primitive::String),
        "the chain runs through the sibling's declaration"
    );
    assert_eq!(MapKeyNewtypes::default().with_value(&file).get(&item), None);

    let conflicting = MapKeyNewtypes::from_documents([
        "types:\n  - {name: demo.orders.ItemId, kind: newtype, of: String}\n",
        "types:\n  - {name: demo.orders.ItemId, kind: newtype, of: Integer}\n",
    ]);
    assert_eq!(
        conflicting.get(&item),
        None,
        "a duplicate is not guessed at"
    );
}
