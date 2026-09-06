//! Public-entry regression probes for the complete expression binding.

use ess_compiler::{compile, expression, ir::EssIr, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

const SOURCE: &str = include_str!("fixtures/adversary_expression.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    Specification::assemble([(
        Source::new("review.yaml"),
        RawSpecFile::parse(text).expect("regression fixture has valid syntax"),
    )])
}

fn compiled(text: &str) -> EssIr {
    let spec = assemble(text).unwrap_or_else(|errors| panic!("{errors}"));
    let mut sources = SourceMap::new();
    sources.insert("review.yaml", text);
    let plain = compile(&spec, &sources).unwrap();
    let located =
        ess_compiler::resolve::compile_locating(&spec, &sources, &["review.yaml"]).unwrap();
    assert_eq!(plain.to_canonical_json(), located.to_canonical_json());
    plain
}

fn guard(predicate: &str) -> String {
    SOURCE.replace("when: amount > 0", &format!("when: {predicate}"))
}

fn refuses(text: &str, code: ValidationCode, needle: &str) {
    let errors = assemble(text).expect_err("the complete expression must refuse admission");
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| { error.code == code && error.message.contains(needle) }),
        "expected {code:?} naming {needle}: {errors}"
    );
}

#[test]
fn map_values_bind_nested_lists_and_restore_the_outer_binder() {
    let valid = "{forall: {in: groups, as: group, that: {all: [{exists: {in: group, as: group, that: group.amount > 0}}, group.count >= 0]}}}";
    compiled(&guard(valid));
    refuses(
        &guard(&valid.replace("group.amount > 0", "group.count > true")),
        ValidationCode::TypeMismatch,
        "Bool",
    );
    refuses(
        &guard(&valid.replace("group.count >= 0", "group.amount >= 0")),
        ValidationCode::UnobservableFact,
        "group.amount",
    );
}

#[test]
fn canonical_list_ordinals_are_legal_but_map_ordinals_and_wire_names_are_not() {
    for path in [
        "group.0.amount",
        "group.999999999999999999999999999999999999.amount",
    ] {
        compiled(&guard(&format!(
            "{{forall: {{in: groups, as: group, that: {path} >= 0}}}}"
        )));
    }
    for path in ["group.00.amount", "group.01.amount", "group.0.amount_wire"] {
        refuses(
            &guard(&format!(
                "{{forall: {{in: groups, as: group, that: {path} >= 0}}}}"
            )),
            ValidationCode::UnobservableFact,
            path,
        );
    }
    for path in ["groups.0", "groups.Ready", "groups.count.value"] {
        refuses(
            &guard(&format!("{path} > 0")),
            ValidationCode::UnobservableFact,
            path,
        );
    }
}

#[test]
fn parameter_targets_are_resolved_before_shadowing_and_uses_resume_afterward() {
    let text = SOURCE.replace(
        "    filter: amount >= 0",
        "    params:\n      - {name: batches, type: 'Map<String, List<review.data.Entry>>'}\n    filter: {all: [{forall: {in: param.batches, as: param, that: {exists: {in: param, as: param, that: param.amount >= 0}}}}, param.batches.count >= 0]}",
    );
    compiled(&text);
    refuses(
        &text.replace("that: param.amount >= 0", "that: param.batches.count >= 0"),
        ValidationCode::UnobservableFact,
        "param.batches.count",
    );
    refuses(
        &text.replace("in: param.batches", "in: param.absent"),
        ValidationCode::UndeclaredReference,
        "param.absent",
    );
}

#[test]
fn a_parameter_shadowed_everywhere_is_still_unused() {
    let text = SOURCE.replace(
        "    filter: amount >= 0",
        "    params:\n      - {name: limit, type: Decimal}\n    filter: {forall: {in: groups, as: param, that: param.count >= 0}}",
    );
    let errors = assemble(&text).expect_err("the binder is not a free parameter use");
    assert!(errors.to_string().contains("limit"), "{errors}");
    assert!(errors.to_string().contains("read"), "{errors}");
}

#[test]
fn all_owners_reject_wrong_enum_literals_through_optional_newtypes() {
    for expression in ["phase == Unknown", "{phase: {none_of: [Ready, Unknown]}}"] {
        refuses(
            &guard(expression),
            ValidationCode::UndeclaredReference,
            "Unknown",
        );
        refuses(
            &SOURCE.replace("filter: amount >= 0", &format!("filter: {expression}")),
            ValidationCode::UndeclaredReference,
            "Unknown",
        );
        refuses(
            &SOURCE.replace(
                "invariants: [amount >= 0]",
                &format!("invariants: [{expression}]"),
            ),
            ValidationCode::UndeclaredReference,
            "Unknown",
        );
    }
    for expression in ["phase == false", "phase == 1"] {
        refuses(&guard(expression), ValidationCode::TypeMismatch, "Text");
    }
    compiled(&guard("phase == 'false'"));
}

#[test]
fn reverse_enum_literals_and_fact_operands_follow_the_same_representation_rules() {
    let ir = compiled(SOURCE);
    let fields = &ir.commands().values().next().unwrap().input;
    for (literal, expected) in [
        (FactValue::text("Ready"), None),
        (
            FactValue::text("Unknown"),
            Some(ValidationCode::UndeclaredReference),
        ),
        (FactValue::Bool(false), Some(ValidationCode::TypeMismatch)),
    ] {
        let predicate = Predicate::Compare {
            left: Operand::Literal(literal),
            op: CompareOp::Eq,
            right: Operand::Fact(FactPath::new("phase").unwrap()),
        };
        let result = expression::check_predicate(&ir, fields, &predicate, "public row adapter");
        assert_eq!(result.errors.first().map(|error| error.code), expected);
    }
    let predicate = Predicate::Compare {
        left: Operand::Fact(FactPath::new("entry.phase").unwrap()),
        op: CompareOp::Eq,
        right: Operand::Fact(FactPath::new("phase").unwrap()),
    };
    assert!(
        expression::check_predicate(&ir, fields, &predicate, "public row adapter")
            .errors
            .is_empty()
    );
}

#[test]
fn unrelated_failures_survive_invalid_quantifier_targets_in_source_order() {
    let text = guard("{all: [{forall: {in: amount, as: item, that: {all: [item.absent > 0, entry.absent > 0]}}}, phase.name == Ready, flag > true]}");
    let errors = assemble(&text).unwrap_err();
    let errors = errors.as_slice();
    assert_eq!(errors.len(), 4, "{errors:?}");
    for (error, needle) in
        errors
            .iter()
            .zip(["not a collection", "entry.absent", "phase.name", "Bool"])
    {
        assert!(error.message.contains(needle), "{error:?}");
        assert!(error.location.contains("command.review.data.Update"));
    }
}

#[test]
fn finite_recursive_selectors_consume_segments_but_transparent_cycles_do_not() {
    let long = format!("entry.{}amount", "next.".repeat(90));
    compiled(&guard(&format!("{long} >= 0")));
    let bad = format!("entry.{}missing", "next.".repeat(90));
    refuses(
        &guard(&format!("{bad} >= 0")),
        ValidationCode::UnobservableFact,
        &bad,
    );
    refuses(
        &guard("loop == 0"),
        ValidationCode::SelfReference,
        "review.data.Loop",
    );
}

#[test]
fn empty_membership_remains_typed_and_does_not_hide_aggregate_reads() {
    compiled(&guard("{phase: {none_of: []}}"));
    compiled(&guard("{amount: {any_of: []}}"));
    for predicate in [
        "{entry: {any_of: []}}",
        "{groups: {none_of: []}}",
        "{entry: {exists: true}}",
    ] {
        refuses(&guard(predicate), ValidationCode::TypeMismatch, "aggregate");
    }
}

#[test]
fn membership_reports_every_invalid_member_even_after_a_valid_enum_literal() {
    let errors = assemble(&guard("{phase: {none_of: [Ready, Unknown, false, 1]}}"))
        .expect_err("all three incompatible membership entries must be reported");
    let errors = errors.as_slice();
    assert_eq!(errors.len(), 3, "{errors:?}");
    assert_eq!(errors[0].code, ValidationCode::UndeclaredReference);
    assert!(errors[0].message.contains("Unknown"));
    assert_eq!(errors[1].code, ValidationCode::TypeMismatch);
    assert!(errors[1].message.contains("Bool"));
    assert_eq!(errors[2].code, ValidationCode::TypeMismatch);
    assert!(errors[2].message.contains("Number"));
}
