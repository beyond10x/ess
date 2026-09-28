//! A narrowed `Optional` input reaches the IR at its present type (beyond10x/ess#169, `ess/16`,
//! `docs/design/optional-input-narrowing.md`).
//!
//! The compiler re-checks every payload and `sets:` read against its target. It narrows exactly
//! where `ess-domain` does, or the #169 repro is admitted by validation and then refused by the
//! compiler as `ESS-COMMAND-002`. The read records the type it was checked at, `T`, with no
//! conversion, so a consumer lowering the branch copies a value it knows is there.
use ess_compiler::ir::{EssIr, ResolvedOutcome, ResolvedPayloadField, ResolvedPayloadValue};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const NOTES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/optional-input-narrowed.yaml");

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("notes.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new())
        .unwrap_or_else(|errors| panic!("the model compiles: {errors}"))
}

fn outcome<'a>(ir: &'a EssIr, name: &str) -> &'a ResolvedOutcome {
    ir.commands()
        .get(&"demo.notes.SubmitNote".parse().unwrap())
        .expect("the command")
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == name)
        .unwrap_or_else(|| panic!("{name}"))
}

fn read_of<'a>(fields: &'a [ResolvedPayloadField], target: &str) -> &'a ResolvedPayloadField {
    fields
        .iter()
        .find(|field| field.target == target)
        .unwrap_or_else(|| panic!("{target}"))
}

fn assert_present_read(field: &ResolvedPayloadField) {
    let ResolvedPayloadValue::InputField {
        field: name,
        type_ref,
    } = &field.value
    else {
        panic!("an input read: {field:?}");
    };
    assert_eq!(name, "account_id");
    assert!(
        !type_ref.is_optional(),
        "read at its present type: {type_ref:?}"
    );
    assert_eq!(type_ref, &field.target_type);
    assert_eq!(field.conversion, None, "no crossing is declared or needed");
}

#[test]
fn issue_169_compiles_and_the_default_reads_the_account_as_present() {
    let ir = ir(NOTES);
    let submitted = outcome(&ir, "submitted");
    assert_present_read(read_of(&submitted.payload[0].fields, "account_id"));
    assert_present_read(read_of(&submitted.sets, "account_id"));
}

#[test]
fn the_declared_input_keeps_its_optional_type() {
    let ir = ir(NOTES);
    let command = &ir.commands()[&"demo.notes.SubmitNote".parse().unwrap()];
    let input = command
        .input
        .iter()
        .find(|field| field.name == "account_id")
        .unwrap();
    assert!(input.type_ref.is_optional());
}

#[test]
fn a_branch_guarded_by_defined_compiles_with_the_same_read() {
    let text = NOTES
        .replacen(
            "      - name: account-missing\n        when: not defined(account_id)\n",
            "      - name: account-missing\n",
            1,
        )
        .replacen(
            "      - name: submitted\n",
            "      - name: submitted\n        when: defined(account_id)\n",
            1,
        );
    assert_ne!(text, NOTES);
    let ir = ir(&text);
    let submitted = outcome(&ir, "submitted");
    assert_present_read(read_of(&submitted.payload[0].fields, "account_id"));
    assert_present_read(read_of(&submitted.sets, "account_id"));
}

#[test]
fn a_declared_crossing_from_the_optional_type_is_still_the_one_recorded() {
    // Declared type first: the author's `Optional<Uuid> -> AccountId` crossing admits the copy and
    // stays on the IR, on the payload and on `sets:`, with the read at its declared type.
    let text = NOTES
        .replacen(
            "      - {name: account_id, type: Optional<demo.notes.AccountId>}\n",
            "      - {name: account_id, type: Optional<Uuid>}\n",
            1,
        )
        .replacen(
            "commands:\n",
            "conversions:\n  - {from: Optional<Uuid>, to: demo.notes.AccountId, because: the parameter is the account's identifier}\ncommands:\n",
            1,
        );
    let ir = ir(&text);
    let submitted = outcome(&ir, "submitted");
    for field in [
        read_of(&submitted.payload[0].fields, "account_id"),
        read_of(&submitted.sets, "account_id"),
    ] {
        let ResolvedPayloadValue::InputField { type_ref, .. } = &field.value else {
            panic!("an input read: {field:?}");
        };
        assert!(type_ref.is_optional(), "{field:?}");
        assert_eq!(
            field.conversion.as_deref(),
            Some("the parameter is the account's identifier")
        );
    }
}
