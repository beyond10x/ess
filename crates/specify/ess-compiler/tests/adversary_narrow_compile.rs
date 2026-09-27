//! Adversary cases for `story:optional-input-narrowed-after-refusal` (beyond10x/ess#169): every
//! model `ess-domain` admits through narrowing must also compile, or it fails as `ESS-COMMAND-002`.
use ess_compiler::ir::{EssIr, ResolvedPayloadField, ResolvedPayloadValue};
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const NOTES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/optional-input-narrowed.yaml");

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

fn submitted_payload(ir: &EssIr) -> &[ResolvedPayloadField] {
    &ir.commands()[&"demo.notes.SubmitNote".parse().unwrap()]
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "submitted")
        .unwrap()
        .payload[0]
        .fields
}

#[test]
fn a_narrowed_leaf_beside_an_input_or_generated_leaf_compiles() {
    let text = edited(
        NOTES,
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n  - name: demo.notes.Owner\n    kind: struct\n    fields:\n      - {name: account_id, type: demo.notes.AccountId}\n      - {name: delegate_id, type: demo.notes.AccountId}\n",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\n    outcomes:",
        "      - {name: text, type: String}\n      - {name: delegate_id, type: Optional<demo.notes.AccountId>}\n    outcomes:",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\nviews:",
        "      - {name: text, type: String}\n      - {name: owner, type: demo.notes.Owner}\nviews:",
    );
    let text = edited(
        &text,
        "account_id: input.account_id, text: input.text}\nevents:",
        "account_id: input.account_id, text: input.text, owner: {account_id: input.account_id, delegate_id: {input: delegate_id, else: {generated: true}}}}\nevents:",
    );
    let ir = ir(&text);
    let owner = submitted_payload(&ir)
        .iter()
        .find(|field| field.target == "owner")
        .expect("owner");
    let ResolvedPayloadValue::Struct { fields } = &owner.value else {
        panic!("{owner:?}");
    };
    let leaf = fields
        .iter()
        .find(|field| field.target == "account_id")
        .unwrap();
    let ResolvedPayloadValue::InputField { type_ref, .. } = &leaf.value else {
        panic!("{leaf:?}");
    };
    assert!(!type_ref.is_optional(), "{leaf:?}");
    assert_eq!(leaf.conversion, None);
}

#[test]
fn an_enum_and_a_decimal_narrow_compile() {
    let text = edited(
        NOTES,
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.notes.AccountId, kind: newtype, of: Uuid}\n  - {name: demo.notes.Kind, kind: enum, variants: [Plain, Urgent]}\n",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\n    outcomes:",
        "      - {name: text, type: String}\n      - {name: kind, type: Optional<demo.notes.Kind>}\n      - {name: weight, type: Optional<Decimal>}\n    outcomes:",
    );
    let text = edited(
        &text,
        "      - name: submitted\n",
        "      - name: kind-missing\n        when: missing(kind)\n        error: demo.notes.AccountMissing\n      - name: weight-missing\n        when: not defined(weight)\n        error: demo.notes.AccountMissing\n      - name: submitted\n",
    );
    let text = edited(
        &text,
        "text: input.text}\nevents:",
        "text: input.text, kind: input.kind, weight: input.weight}\nevents:",
    );
    let text = edited(
        &text,
        "      - {name: text, type: String}\nviews:",
        "      - {name: text, type: String}\n      - {name: kind, type: demo.notes.Kind}\n      - {name: weight, type: Decimal}\nviews:",
    );
    let ir = ir(&text);
    for name in ["kind", "weight"] {
        let field = submitted_payload(&ir)
            .iter()
            .find(|field| field.target == name)
            .unwrap();
        let ResolvedPayloadValue::InputField { type_ref, .. } = &field.value else {
            panic!("{field:?}");
        };
        assert!(!type_ref.is_optional(), "{field:?}");
    }
}

#[test]
fn a_subject_state_branch_guarded_by_defined_compiles() {
    let text = edited(
        NOTES,
        "    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}\n",
        "    lifecycle:\n      initial: Open\n      states: [Open, Filed]\n      terminal: [Filed]\n      transitions:\n        - {name: file, from: [Open, Filed], to: Filed}\n",
    );
    let text = edited(
        &text,
        "  - {name: demo.notes.Service, may: [demo.notes.SubmitNote]}\n",
        "  - {name: demo.notes.Service, may: [demo.notes.SubmitNote, demo.notes.FileNote]}\n",
    );
    let text = edited(
        &text,
        "events:\n",
        "  - name: demo.notes.FileNote\n    input:\n      - {name: note_id, type: demo.notes.NoteId}\n      - {name: account_id, type: Optional<demo.notes.AccountId>}\n    outcomes:\n      - name: filed\n        moves: demo.notes.Note.file\n        instance: note_id\n        when_subject_state: Open\n        when: defined(account_id)\n        emits: [demo.notes.NoteFiled]\n        payload:\n          demo.notes.NoteFiled: {note_id: input.note_id, account_id: input.account_id}\n      - name: no-account\n        moves: demo.notes.Note.file\n        instance: note_id\n        emits: [demo.notes.NoteFiledAlone]\n        payload:\n          demo.notes.NoteFiledAlone: {note_id: input.note_id}\nevents:\n  - name: demo.notes.NoteFiledAlone\n    fields:\n      - {name: note_id, type: demo.notes.NoteId}\n  - name: demo.notes.NoteFiled\n    fields:\n      - {name: note_id, type: demo.notes.NoteId}\n      - {name: account_id, type: demo.notes.AccountId}\n",
    );
    let _ = ir(&text);
}

#[test]
fn a_declared_crossing_from_the_optional_type_still_compiles_under_ess_16() {
    // The #169 workaround for an input whose declared type is not the target's: narrowing must
    // not turn the declared `Optional<Uuid> -> AccountId` crossing into an undeclared
    // `Uuid -> AccountId` one, in validation or in the compiler's re-check.
    let text = edited(
        NOTES,
        "      - {name: account_id, type: Optional<demo.notes.AccountId>}\n",
        "      - {name: account_id, type: Optional<Uuid>}\n",
    );
    let text = edited(
        &text,
        "commands:\n",
        "conversions:\n  - {from: Optional<Uuid>, to: demo.notes.AccountId, because: the account parameter is the account's identifier}\ncommands:\n",
    );
    let ir = ir(&text);
    let field = submitted_payload(&ir)
        .iter()
        .find(|field| field.target == "account_id")
        .unwrap();
    assert!(field.conversion.is_some(), "{field:?}");
}
