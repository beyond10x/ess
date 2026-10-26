//! A lifecycle state's wire name (`ess/32`, beyond10x/ess#639) reaches the IR as the naming of the
//! state enum's variant; the lifecycle keeps the declared name, and a lifecycle that declares no
//! spelling keeps its IR bytes.

use ess_compiler::ir::ResolvedBody;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use serde_json::Value;

use super::MODEL;

fn compiled(text: &str) -> ess_compiler::ir::EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("state-change.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("state-change.yaml", text.to_owned());
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("{errors}"))
}

/// Every JSON object under `value` holding the key `key`.
fn holding<'a>(value: &'a Value, key: &str, found: &mut Vec<&'a Value>) {
    match value {
        Value::Object(map) => {
            if map.contains_key(key) {
                found.push(value);
            }
            map.values().for_each(|child| holding(child, key, found));
        }
        Value::Array(items) => items.iter().for_each(|child| holding(child, key, found)),
        _ => {}
    }
}

/// The variants of `demo.room.Member.State`, as `(name, wire)`.
fn state_variants(ir: &ess_compiler::ir::EssIr) -> Vec<(String, String)> {
    let state = &ir.types()[&"demo.room.Member.State".parse().unwrap()];
    let ResolvedBody::Enum { variants } = &state.body else {
        panic!("the state type is an enum: {:?}", state.body);
    };
    variants
        .iter()
        .map(|variant| (variant.name().to_owned(), variant.wire().to_owned()))
        .collect()
}

#[test]
fn a_bare_lifecycle_keeps_its_ir_bytes() {
    for format in ["ess/4", "ess/32"] {
        let ir = compiled(&MODEL.replace("format: ess/4", &format!("format: {format}")));
        let json: Value = serde_json::from_str(&ir.to_canonical_json()).expect("the IR is JSON");
        let mut lifecycles = Vec::new();
        holding(&json, "transitions", &mut lifecycles);
        assert_eq!(lifecycles.len(), 1, "{format}: one lifecycle");
        let keys: std::collections::BTreeSet<&str> = lifecycles[0]
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            std::collections::BTreeSet::from(["initial", "states", "terminal", "transitions"]),
            "{format}: no key is added to a lifecycle that spells no state"
        );
        let mut enums = Vec::new();
        holding(&json, "variants", &mut enums);
        let state = enums
            .iter()
            .find(|candidate| {
                candidate["variants"] == serde_json::json!(["Added", "Gone", "Pending"])
            })
            .copied();
        assert!(
            state.is_some(),
            "{format}: the state enum's variants stay bare names: {enums:?}"
        );
    }
}

#[test]
fn a_state_wire_name_reaches_the_state_enum_variant() {
    let text = MODEL.replace("format: ess/4", "format: ess/32").replace(
        "states: [Pending, Added, Gone]",
        "states: [{name: Pending, wire: pending}, Added, Gone]",
    );
    let ir = compiled(&text);
    assert_eq!(
        state_variants(&ir),
        [("Added", "Added"), ("Gone", "Gone"), ("Pending", "pending")]
            .map(|(name, wire)| (name.to_owned(), wire.to_owned()))
            .to_vec(),
    );
    let member = &ir.entities()[&"demo.room.Member".parse().unwrap()];
    assert_eq!(member.lifecycle.initial.to_string(), "Pending");
    assert!(
        member
            .lifecycle
            .states
            .iter()
            .any(|state| state.to_string() == "Pending"),
        "the lifecycle keeps the declared name"
    );
}
