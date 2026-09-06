//! Admission checks exercise authored declarations before any resolved IR exists.

use ess_compiler::{compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const SOURCE: &str = r"
format: ess/1
system: sample
version: v1
domains: [sample.data]
domain: sample.data
types:
  - name: sample.data.Money
    kind: struct
    fields:
      - {name: amount, type: Decimal}
    invariants: [amount >= 0]
  - name: sample.data.Positive
    kind: newtype
    of: Decimal
    invariants: [value >= 0]
  - name: sample.data.Channel
    kind: enum
    variants: [Email, Post]
entities:
  - name: sample.data.Item
    identity: {name: id, type: String}
    fields:
      - {name: amount, type: Decimal}
      - {name: nested, type: sample.data.Money}
      - {name: channel, type: sample.data.Channel}
    invariants: [amount >= 0]
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
commands:
  - name: sample.data.Update
    input:
      - {name: amount, type: Decimal}
      - {name: nested, type: sample.data.Money}
      - {name: flag, type: Boolean}
      - {name: channel, type: sample.data.Channel}
      - {name: lines, type: List<sample.data.Money>}
    outcomes:
      - name: changed
        when: amount > 0
        emits: [sample.data.Changed]
      - name: refused
        error: sample.data.Rejected
events:
  - name: sample.data.Changed
errors:
  - name: sample.data.Rejected
views:
  - name: sample.data.Items
    source: sample.data.Item
    fields:
      - {name: amount, type: Decimal}
    filter: amount >= 0
";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    Specification::assemble([(
        Source::new("expressions.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
}

fn rejected(text: &str, code: ValidationCode, owner: &str, needle: &str) {
    let Err(errors) = assemble(text) else {
        panic!("an ill-typed expression must fail admission: {needle}");
    };
    assert!(
        errors.as_slice().iter().any(|error| error.code == code
            && error.location.contains(owner)
            && error.message.contains(needle)),
        "{errors}"
    );
}

#[test]
fn the_control_assembles_and_compiles() {
    compile(&assemble(SOURCE).unwrap(), &SourceMap::new()).unwrap();
}

#[test]
fn guard_scalar_continuation_fails_at_admission() {
    rejected(
        &SOURCE.replace("when: amount > 0", "when: amount.nonexistent > 0"),
        ValidationCode::UnobservableFact,
        "command.sample.data.Update",
        "amount.nonexistent",
    );
}

#[test]
fn entity_scalar_continuation_fails_at_admission() {
    rejected(
        &SOURCE.replace(
            "    invariants: [amount >= 0]\n    lifecycle:",
            "    invariants: [amount.nonexistent >= 0]\n    lifecycle:",
        ),
        ValidationCode::UnobservableFact,
        "entity sample.data.Item",
        "amount.nonexistent",
    );
}

#[test]
fn struct_scalar_continuation_fails_at_admission() {
    rejected(
        &SOURCE.replacen(
            "invariants: [amount >= 0]",
            "invariants: [amount.nonexistent >= 0]",
            1,
        ),
        ValidationCode::UnobservableFact,
        "types.sample.data.Money",
        "amount.nonexistent",
    );
}

#[test]
fn newtype_scalar_continuation_fails_at_admission() {
    rejected(
        &SOURCE.replace(
            "invariants: [value >= 0]",
            "invariants: [value.nonexistent >= 0]",
        ),
        ValidationCode::UnobservableFact,
        "types.sample.data.Positive",
        "value.nonexistent",
    );
}

#[test]
fn filter_scalar_continuation_fails_at_admission() {
    rejected(
        &SOURCE.replace("filter: amount >= 0", "filter: amount.nonexistent >= 0"),
        ValidationCode::UnobservableFact,
        "view.sample.data.Items",
        "amount.nonexistent",
    );
}

#[test]
fn ordering_boolean_is_a_type_error() {
    rejected(
        &SOURCE.replace("when: amount > 0", "when: flag > true"),
        ValidationCode::TypeMismatch,
        "command.sample.data.Update",
        "Bool",
    );
}

#[test]
fn a_bad_membership_member_is_checked_after_a_valid_one() {
    rejected(
        &SOURCE.replace("when: amount > 0", "when: {amount: {any_of: [1, wrong]}}"),
        ValidationCode::TypeMismatch,
        "command.sample.data.Update",
        "Text",
    );
}

#[test]
fn both_fact_operands_are_resolved() {
    rejected(
        &SOURCE.replace("when: amount > 0", "when: amount > nested.absent"),
        ValidationCode::UnobservableFact,
        "command.sample.data.Update",
        "nested.absent",
    );
}

#[test]
fn enum_literals_are_checked_in_command_guards() {
    rejected(
        &SOURCE.replace("when: amount > 0", "when: channel == Mail"),
        ValidationCode::UndeclaredReference,
        "command.sample.data.Update",
        "Mail",
    );
}

#[test]
fn quantified_bodies_are_checked_in_command_guards() {
    rejected(
        &SOURCE.replace(
            "when: amount > 0",
            "when: {forall: {in: lines, as: line, that: line.absent > 0}}",
        ),
        ValidationCode::UnobservableFact,
        "command.sample.data.Update",
        "line.absent",
    );
}

#[test]
fn both_compiler_entries_preserve_valid_predicates_and_canonical_bytes() {
    let spec = assemble(SOURCE).unwrap();
    let ir = compile(&spec, &SourceMap::new()).unwrap();
    let mut sources = SourceMap::new();
    sources.insert("expressions.yaml", SOURCE);
    let located =
        ess_compiler::resolve::compile_locating(&spec, &sources, &["expressions.yaml"]).unwrap();
    assert_eq!(ir.to_canonical_json(), located.to_canonical_json());
}

#[test]
fn diagnostics_keep_real_owners_and_never_invent_predicate_leaf_spans() {
    let text = SOURCE.replace(
        "when: amount > 0",
        "when: {all: [amount.bad > nested.missing, flag > true]}",
    );
    let errors = assemble(&text).unwrap_err();
    assert_eq!(errors.len(), 3, "{errors}");
    let mut sources = SourceMap::new();
    sources.insert("expressions.yaml", &text);
    let located =
        ess_compiler::resolve::diagnose_locating(&errors, &sources, &["expressions.yaml"]);
    for (error, diagnostic) in errors.as_slice().iter().zip(located.as_slice()) {
        assert_eq!(diagnostic.message, error.message);
        let span = diagnostic.span.as_ref().unwrap();
        assert_eq!(span.source, "expressions.yaml");
        assert_eq!(span.path, error.location);
        if let Some(location) = &span.located {
            assert!(text
                .lines()
                .nth(location.line - 1)
                .unwrap()
                .contains("sample.data.Update"));
        }
    }
    let absent =
        ess_compiler::resolve::diagnose_locating(&errors, &SourceMap::new(), &["expressions.yaml"]);
    assert!(absent.as_slice().iter().all(|diagnostic| diagnostic
        .span
        .as_ref()
        .unwrap()
        .located
        .is_none()));
}

#[test]
fn nested_view_parameters_count_as_used_and_source_only_fields_are_readable() {
    let text=SOURCE.replace("    filter: amount >= 0",
        "    params:\n      - {name: limit, type: sample.data.Money}\n    filter: nested.amount >= param.limit.amount");
    compile(&assemble(&text).unwrap(), &SourceMap::new()).unwrap();
    for (from, to, needle) in [
        ("param.limit.amount", "param.limit.bad", "param.limit.bad"),
        (
            "param.limit.amount",
            "param.absent.amount",
            "param.absent.amount",
        ),
        (
            "name: limit, type: sample.data.Money",
            "name: limit, type: sample.data.Missing",
            "sample.data.Missing",
        ),
    ] {
        let errors = assemble(&text.replace(from, to)).unwrap_err();
        assert!(errors.to_string().contains(needle), "{errors}");
    }
}

#[test]
fn entity_state_has_its_closed_enum_type_and_no_selectors() {
    for (expression, code, needle) in [
        (
            "state.name == Active",
            ValidationCode::UnobservableFact,
            "state.name",
        ),
        (
            "state == Missing",
            ValidationCode::UndeclaredReference,
            "Missing",
        ),
    ] {
        rejected(
            &SOURCE.replace(
                "invariants: [amount >= 0]\n    lifecycle:",
                &format!("invariants: [{expression}]\n    lifecycle:"),
            ),
            code,
            "entity sample.data.Item",
            needle,
        );
    }
    assemble(&SOURCE.replace(
        "invariants: [amount >= 0]\n    lifecycle:",
        "invariants: [state == Active]\n    lifecycle:",
    ))
    .unwrap();
}

#[test]
fn declared_field_names_remain_expression_names_when_wire_names_differ() {
    let text = SOURCE.replace(
        "{name: amount, type: Decimal}",
        "{name: amount, type: Decimal, wire: value}",
    );
    assemble(&text).unwrap();
    rejected(
        &text.replace("when: amount > 0", "when: value > 0"),
        ValidationCode::UnobservableFact,
        "command.sample.data.Update",
        "value",
    );
}

#[test]
fn command_subject_identity_stays_forbidden_as_a_free_read_inside_quantifiers() {
    let text=SOURCE.replace("      - {name: flag, type: Boolean}","      - {name: flag, type: Boolean}\n      - {name: id, type: String}")
        .replace("        when: amount > 0","        when: {forall: {in: lines, as: line, that: id == chosen}}\n        updates: sample.data.Item\n        instance: id");
    rejected(
        &text,
        ValidationCode::UnobservableFact,
        "command.sample.data.Update",
        "id",
    );
    assemble(&text.replace(
        "as: line, that: id == chosen",
        "as: id, that: id.amount > 0",
    ))
    .unwrap();
}

#[test]
fn optional_newtypes_keep_their_representation_without_a_value_selector() {
    let text = SOURCE.replace(
        "    input:\n      - {name: amount, type: Decimal}",
        "    input:\n      - {name: amount, type: Optional<sample.data.Positive>}",
    );
    assemble(&text).unwrap();
    rejected(
        &text.replace("when: amount > 0", "when: amount.value > 0"),
        ValidationCode::UnobservableFact,
        "command.sample.data.Update",
        "amount.value",
    );
}

#[test]
fn integer_fractional_comparison_and_decimal_interval_both_validate() {
    for text in [
        SOURCE
            .replace(
                "    input:\n      - {name: amount, type: Decimal}",
                "    input:\n      - {name: amount, type: Integer}",
            )
            .replace("when: amount > 0", "when: amount == 0.5"),
        SOURCE.replace(
            "when: amount > 0",
            "when: {all: [amount > 0.1, amount < 0.2]}",
        ),
    ] {
        compile(&assemble(&text).unwrap(), &SourceMap::new()).unwrap();
    }
}
