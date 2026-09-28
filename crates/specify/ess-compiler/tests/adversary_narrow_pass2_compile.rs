//! Adversary pass 2 for `story:optional-input-narrowed-after-refusal` (beyond10x/ess#169): an
//! `Optional` of an aggregate narrows in validation, and the compiler must agree or the model fails
//! as `ESS-COMMAND-002`.
use ess_compiler::ir::{EssIr, ResolvedPayloadValue, ResolvedTypeRef};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const NOTES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/optional-input-narrowed.yaml");

const PAYLOAD: &str = "{generated: true}, account_id: input.account_id, text: input.text}";

fn edited(base: &str, before: &str, after: &str) -> String {
    assert!(base.contains(before), "fixture holds {before:?}");
    base.replacen(before, after, 1)
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new())
        .unwrap_or_else(|errors| panic!("the model compiles: {errors}"))
}

/// The repro with an extra optional input `extra: Optional<of>`, refused when absent, copied into
/// an event field `extra: of`.
fn with_extra(of: &str, types: &str) -> String {
    let text = edited(
        NOTES,
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n",
        &format!("  - {{name: demo.notes.AccountId, kind: newtype, of: Uuid}}\n{types}"),
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\n    outcomes:\n",
        &format!("      - {{name: text, type: String}}\n      - {{name: extra, type: \"Optional<{of}>\"}}\n    outcomes:\n"),
    );
    let text = edited(
        &text,
        "        error: demo.notes.AccountMissing\n",
        "        error: demo.notes.AccountMissing\n      - name: extra-missing\n        when: missing(extra)\n        error: demo.notes.AccountMissing\n",
    );
    let text = edited(
        &text,
        PAYLOAD,
        "{generated: true}, account_id: input.account_id, text: input.text, extra: input.extra}",
    );
    edited(
        &text,
        "      - {name: text, type: String}\nviews:",
        &format!(
            "      - {{name: text, type: String}}\n      - {{name: extra, type: \"{of}\"}}\nviews:"
        ),
    )
}

fn extra_read(ir: &EssIr) -> ResolvedTypeRef {
    let fields = &ir.commands()[&"demo.notes.SubmitNote".parse().unwrap()]
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "submitted")
        .unwrap()
        .payload[0]
        .fields;
    let extra = fields
        .iter()
        .find(|field| field.target == "extra")
        .expect("extra is filled");
    assert!(extra.conversion.is_none());
    match &extra.value {
        ResolvedPayloadValue::InputField { type_ref, .. } => type_ref.clone(),
        other => panic!("an input read, not {other:?}"),
    }
}

#[test]
fn an_optional_list_input_narrows_and_compiles() {
    let read = extra_read(&ir(&with_extra("List<String>", "")));
    assert!(matches!(read, ResolvedTypeRef::List { .. }), "{read:?}");
}

#[test]
fn an_optional_struct_input_narrows_and_compiles() {
    let read = extra_read(&ir(&with_extra(
        "demo.notes.Owner",
        "  - name: demo.notes.Owner\n    kind: struct\n    fields:\n      - {name: account_id, type: demo.notes.AccountId}\n",
    )));
    assert!(
        !matches!(read, ResolvedTypeRef::Optional { .. }),
        "{read:?}"
    );
}

#[test]
fn an_optional_map_input_narrows_and_compiles() {
    let read = extra_read(&ir(&with_extra("Map<String, Integer>", "")));
    assert!(matches!(read, ResolvedTypeRef::Map { .. }), "{read:?}");
}
