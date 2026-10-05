//! A refusal that declares its compensating change (`compensates: true`, ess/22,
//! beyond10x/ess#197): its codes, and the IR it compiles to (`docs/design/refusal-with-effect.md`).

use ess_compiler::resolve::{compile, diagnose_locating};
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/refusal-with-effect.yaml");

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// Every diagnostic `text` earns, as `(code, path)`.
fn diagnosed(text: &str) -> Vec<(String, String)> {
    let raw = RawSpecFile::parse(text).unwrap();
    let errors = Specification::assemble([(Source::new("order.yaml"), raw)])
        .expect_err("the document must be refused");
    let mut sources = SourceMap::new();
    sources.insert("order.yaml".to_owned(), text.to_owned());
    diagnose_locating(&errors, &sources, &["order.yaml"])
        .as_slice()
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.code.to_string(),
                diagnostic.span.as_ref().expect("a span").path.clone(),
            )
        })
        .collect()
}

fn ir_json(text: &str) -> serde_json::Value {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("order.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    let ir = compile(&spec, &SourceMap::new()).expect("compiles");
    serde_json::to_value(&ir).unwrap()
}

/// The outcomes of the declaration named `command`, wherever the IR document holds it.
fn outcomes(ir: &serde_json::Value, command: &str) -> Vec<serde_json::Value> {
    fn find<'v>(value: &'v serde_json::Value, command: &str) -> Option<&'v serde_json::Value> {
        match value {
            serde_json::Value::Object(map) => {
                if map.get("name").is_some_and(|name| name == command)
                    && map.contains_key("outcomes")
                {
                    return Some(value);
                }
                map.values().find_map(|inner| find(inner, command))
            }
            serde_json::Value::Array(items) => items.iter().find_map(|inner| find(inner, command)),
            _ => None,
        }
    }
    find(ir, command).unwrap_or_else(|| panic!("{command} in {ir:#}"))["outcomes"]
        .as_array()
        .unwrap()
        .clone()
}

#[test]
fn the_compensating_branch_carries_the_marker_its_subject_and_its_error() {
    let ir = ir_json(MODEL);
    let outcomes = outcomes(&ir, "shop.order.JoinOrder");
    let failed = outcomes
        .iter()
        .find(|outcome| outcome["name"] == "failed")
        .unwrap();
    assert_eq!(failed["compensates"], true, "{failed:#}");
    assert!(failed.get("subject").is_some(), "{failed:#}");
    assert!(failed.get("error").is_some(), "{failed:#}");
    assert!(failed.get("sets").is_some(), "{failed:#}");
    for other in outcomes
        .iter()
        .filter(|outcome| outcome["name"] != "failed")
    {
        assert!(other.get("compensates").is_none(), "{other:#}");
    }
}

#[test]
fn an_unmarked_refusal_with_a_move_is_still_ess_command_004() {
    assert_eq!(
        diagnosed(&replaced(MODEL, "        compensates: true\n", "")),
        vec![(
            "ESS-COMMAND-004".to_owned(),
            "command.shop.order.JoinOrder.outcomes.failed".to_owned()
        )]
    );
}

#[test]
fn the_marker_below_ess_22_is_ess_command_009() {
    assert_eq!(
        diagnosed(&replaced(MODEL, "format: ess/22\n", "format: ess/21\n")),
        vec![(
            "ESS-COMMAND-009".to_owned(),
            "command.shop.order.JoinOrder.outcomes.failed.compensates".to_owned()
        )]
    );
}

#[test]
fn the_marker_on_a_guarded_refusal_is_ess_command_004() {
    assert_eq!(
        diagnosed(&replaced(
            MODEL,
            "        external: the upstream refuses the join\n",
            "        when: reason == \"refused\"\n",
        )),
        vec![(
            "ESS-COMMAND-004".to_owned(),
            "command.shop.order.JoinOrder.outcomes.failed".to_owned()
        )]
    );
}
