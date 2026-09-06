//! The common expression policy, independently of any producer's projection abilities.

use ess_domain::expression::{check_predicate, resolve_path, DomainEnvironment, ScalarKind};
use ess_domain::{Field, NamedType, Naming, TypeBody, TypeRef, TypeRegistry};
use ess_primitives::error::ValidationCode;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, Operand, Predicate, Quantified};

fn field(name: &str, kind: &str) -> Field {
    Field::new(name, TypeRef::parse(kind).unwrap())
}
fn registry() -> TypeRegistry {
    let mut registry = TypeRegistry::new();
    for (name, body) in [
        (
            "sample.Money",
            TypeBody::Struct {
                fields: vec![
                    field("amount", "Decimal"),
                    field("count", "String"),
                    field("state", "sample.State"),
                ],
                invariants: vec![],
            },
        ),
        (
            "sample.State",
            TypeBody::Enum {
                variants: vec!["Ready".into(), "false".into(), "1".into()],
            },
        ),
        (
            "sample.OtherState",
            TypeBody::Enum {
                variants: vec!["Absent".into()],
            },
        ),
        (
            "sample.Email",
            TypeBody::Newtype {
                of: TypeRef::parse("String").unwrap(),
                invariants: vec![],
            },
        ),
        (
            "sample.Wrapped",
            TypeBody::Newtype {
                of: TypeRef::parse("Optional<sample.Money>").unwrap(),
                invariants: vec![],
            },
        ),
        (
            "sample.WrappedState",
            TypeBody::Newtype {
                of: TypeRef::parse("sample.State").unwrap(),
                invariants: vec![],
            },
        ),
        (
            "sample.Node",
            TypeBody::Struct {
                fields: vec![
                    field("next", "Optional<sample.Node>"),
                    field("amount", "Decimal"),
                ],
                invariants: vec![],
            },
        ),
        (
            "sample.Loop",
            TypeBody::Newtype {
                of: TypeRef::parse("Optional<sample.Loop>").unwrap(),
                invariants: vec![],
            },
        ),
        (
            "sample.Union",
            TypeBody::Union {
                tag: "kind".into(),
                variants: [("email".into(), TypeRef::parse("sample.Email").unwrap())].into(),
            },
        ),
    ] {
        registry
            .insert(NamedType {
                name: name.parse().unwrap(),
                body,
                naming: Naming::default(),
            })
            .unwrap();
    }
    registry
}
fn fields() -> Vec<Field> {
    vec![
        field("amount", "Decimal"),
        field("quantity", "Integer"),
        field("flag", "Boolean"),
        field("text", "String"),
        field("note", "Optional<String>"),
        field("email", "sample.Email"),
        field("money", "sample.Wrapped"),
        field("state", "sample.State"),
        field("other", "sample.OtherState"),
        field("wrapped_state", "sample.WrappedState"),
        field("lines", "List<sample.Money>"),
        field("mapping", "Map<String, sample.Money>"),
        field("nested", "List<List<sample.Money>>"),
        field("node", "sample.Node"),
        field("loop", "sample.Loop"),
        field("union", "sample.Union"),
        field("broken", "sample.Missing"),
    ]
}
fn predicate(text: &str) -> Predicate {
    serde_yaml::from_str(text).unwrap()
}
fn path(text: &str) -> FactPath {
    FactPath::new(text).unwrap()
}
fn compare(left: &str, right: &str) -> Predicate {
    Predicate::Compare {
        left: Operand::Fact(path(left)),
        op: CompareOp::Eq,
        right: Operand::Fact(path(right)),
    }
}
fn quantified(over: &str, bind: &str, body: Predicate) -> Predicate {
    Predicate::Forall(Box::new(Quantified {
        over: path(over),
        bind: bind.into(),
        body,
    }))
}

#[test]
fn complete_paths_keep_nominal_types_optionality_and_access_requirements() {
    let registry = registry();
    let fields = fields();
    let env = DomainEnvironment::new(&registry, &fields);
    for (text, kind, optional, collection) in [
        ("amount", ScalarKind::Number, false, false),
        ("money.amount", ScalarKind::Number, true, false),
        ("money.count", ScalarKind::Text, true, false),
        ("lines.count", ScalarKind::Number, false, true),
        ("lines.0.amount", ScalarKind::Number, false, true),
        ("mapping.count", ScalarKind::Number, false, true),
        (
            "lines.999999999999999999999999999999999999.amount",
            ScalarKind::Number,
            false,
            true,
        ),
    ] {
        let resolved = resolve_path(&env, &path(text), "test").unwrap();
        assert_eq!(resolved.scalar, Some(kind), "{text}");
        assert_eq!(resolved.optional, optional, "{text}");
        assert_eq!(resolved.access.collection, collection, "{text}");
        if text.rsplit('.').next() == Some("count") && collection {
            assert_eq!(resolved.terminal.to_string(), "Integer");
        }
    }
}
#[test]
fn scalar_enum_collection_and_union_selector_errors_name_the_first_bad_segment() {
    let registry = registry();
    let fields = fields();
    let env = DomainEnvironment::new(&registry, &fields);
    for (text, segment) in [
        ("amount.no.more", "no"),
        ("state.name", "name"),
        ("money.absent.x", "absent"),
        ("lines.bad.amount", "bad"),
        ("lines.01.amount", "01"),
        ("lines.count.next", "next"),
        ("mapping.0.amount", "0"),
        ("mapping.key", "key"),
        ("union.kind", "kind"),
    ] {
        let error = resolve_path(&env, &path(text), "owner").unwrap_err();
        assert_eq!(error.code, ValidationCode::UnobservableFact, "{text}");
        assert_eq!(error.segment.as_deref(), Some(segment), "{text}");
        assert!(
            error.message.contains(text) && error.message.contains(segment),
            "{error:?}"
        );
    }
}
#[test]
fn scalar_operand_contract_is_distinct_from_nominal_assignment_and_satisfiability() {
    let registry = registry();
    let fields = fields();
    let env = DomainEnvironment::new(&registry, &fields);
    for p in [
        compare("amount", "quantity"),
        compare("email", "text"),
        compare("state", "other"),
        predicate("quantity == 0.5"),
        predicate("text > later"),
        predicate("flag"),
        predicate("amount"),
        predicate("text"),
        predicate("{note: {exists: true}}"),
        predicate("{amount: {any_of: []}}"),
        Predicate::Compare {
            left: Operand::Literal(FactValue::Bool(true)),
            op: CompareOp::Eq,
            right: Operand::Literal(FactValue::Bool(false)),
        },
    ] {
        let checked = check_predicate(&env, &p, "owner");
        assert!(checked.errors.is_empty(), "{p}: {:?}", checked.errors);
    }
    assert!(!ess_domain::types::is_assignable(
        &fields[5].type_ref,
        &fields[3].type_ref
    ));
}
#[test]
fn every_operand_membership_item_and_dead_ast_child_is_checked() {
    let registry = registry();
    let fields = fields();
    let env = DomainEnvironment::new(&registry, &fields);
    for p in [
        predicate("flag > true"),
        compare("amount", "text"),
        predicate("{amount: {any_of: [1, bad]}}"),
        predicate("{amount: {none_of: [1, bad]}}"),
        Predicate::Truthy(path("money")),
        Predicate::Defined(path("lines")),
        compare("money", "money"),
        Predicate::All(vec![Predicate::Never, compare("amount", "flag")]),
        Predicate::Any(vec![
            Predicate::Always,
            Predicate::Not(Box::new(compare("flag", "text"))),
        ]),
        Predicate::Compare {
            left: Operand::Literal(FactValue::Bool(true)),
            op: CompareOp::Eq,
            right: Operand::Literal(FactValue::text("true")),
        },
    ] {
        let checked = check_predicate(&env, &p, "owner");
        assert!(
            checked
                .errors
                .iter()
                .any(|e| e.code == ValidationCode::TypeMismatch),
            "{p}: {:?}",
            checked.errors
        );
    }
}
#[test]
fn enum_literal_checks_apply_through_wrappers_membership_and_bound_paths() {
    let registry = registry();
    let fields = fields();
    let env = DomainEnvironment::new(&registry, &fields);
    for p in [
        predicate("money.state == Bad"),
        predicate("wrapped_state == Bad"),
        predicate("{state: {none_of: [Ready, Bad]}}"),
        quantified("lines", "line", predicate("line.state == Bad")),
    ] {
        let checked = check_predicate(&env, &p, "owner");
        assert!(
            checked
                .errors
                .iter()
                .any(|e| e.code == ValidationCode::UndeclaredReference),
            "{p}"
        );
    }
    for p in [predicate("state == false"), predicate("state == 1")] {
        assert!(check_predicate(&env, &p, "owner")
            .errors
            .iter()
            .any(|e| e.code == ValidationCode::TypeMismatch));
    }
    for p in [
        predicate("state == 'false'"),
        predicate("wrapped_state == Ready"),
        quantified("mapping", "value", predicate("value.state == Ready")),
    ] {
        assert!(check_predicate(&env, &p, "owner").errors.is_empty(), "{p}");
    }
}
#[test]
fn quantifiers_resolve_targets_before_pushing_lexical_binders() {
    let registry = registry();
    let fields = fields();
    let env = DomainEnvironment::new(&registry, &fields);
    for p in [
        quantified("lines", "lines", predicate("lines.amount > 0")),
        quantified(
            "nested",
            "row",
            quantified("row", "row", predicate("row.amount > 0")),
        ),
        quantified("mapping", "entry", compare("entry.amount", "amount")),
        quantified(
            "lines",
            "line",
            quantified("mapping", "entry", compare("entry.amount", "line.amount")),
        ),
    ] {
        assert!(check_predicate(&env, &p, "owner").errors.is_empty(), "{p}");
    }
    for p in [
        quantified("amount", "line", Predicate::Always),
        quantified("money", "line", Predicate::Always),
        quantified("union", "line", Predicate::Always),
        quantified("lines", "line", predicate("line.absent > 0")),
        quantified(
            "lines",
            "line",
            quantified("line.amount", "item", Predicate::Always),
        ),
    ] {
        assert!(!check_predicate(&env, &p, "owner").errors.is_empty(), "{p}");
    }
    let p = Predicate::All(vec![
        quantified("lines", "line", Predicate::Always),
        predicate("line.amount > 0"),
    ]);
    assert!(!check_predicate(&env, &p, "owner").errors.is_empty());
}
#[test]
fn parameters_are_typed_at_depth_and_shadowed_lexically() {
    let registry = registry();
    let fields = fields();
    let params = vec![
        field("limit", "sample.Money"),
        field("broken", "sample.Missing"),
    ];
    let env = DomainEnvironment::new(&registry, &fields).with_params(&params);
    let p = quantified(
        "lines",
        "line",
        compare("line.amount", "param.limit.amount"),
    );
    let checked = check_predicate(&env, &p, "owner");
    assert!(checked.errors.is_empty(), "{:?}", checked.errors);
    assert!(checked.parameters.contains("limit"));
    let checked = check_predicate(
        &env,
        &quantified("lines", "param", predicate("param.amount > 0")),
        "owner",
    );
    assert!(checked.errors.is_empty());
    assert!(checked.parameters.is_empty());
    for text in [
        "param.absent.amount > 0",
        "param.limit.absent > 0",
        "param.broken > 0",
    ] {
        assert!(
            !check_predicate(&env, &predicate(text), "owner")
                .errors
                .is_empty(),
            "{text}"
        );
    }
}
#[test]
fn recursive_reads_need_progress_but_have_no_projection_depth_limit() {
    let registry = registry();
    let fields = fields();
    let env = DomainEnvironment::new(&registry, &fields);
    let text = format!("node.{}amount", "next.".repeat(80));
    let resolved = resolve_path(&env, &path(&text), "owner").unwrap();
    assert_eq!(resolved.scalar, Some(ScalarKind::Number));
    assert!(resolved.access.depth > 32);
    let error = resolve_path(&env, &path("loop"), "owner").unwrap_err();
    assert_eq!(error.code, ValidationCode::SelfReference);
    assert!(error.message.contains("sample.Loop"));
}
#[test]
fn failed_operands_suppress_cascades_and_keep_stable_independent_errors() {
    let registry = registry();
    let fields = fields();
    let env = DomainEnvironment::new(&registry, &fields);
    let p = Predicate::All(vec![
        compare("amount.absent", "money.absent"),
        compare("flag", "text"),
    ]);
    let checked = check_predicate(&env, &p, "owner");
    assert_eq!(checked.errors.len(), 3);
    assert_eq!(checked.errors[0].segment.as_deref(), Some("absent"));
    assert!(checked.errors[0].message.contains("amount.absent"));
    assert!(checked.errors[1].message.contains("money.absent"));
    assert_eq!(checked.errors[2].code, ValidationCode::TypeMismatch);
    assert!(checked.errors.iter().all(|e| e.owner == "owner"));
    let missing = resolve_path(&env, &path("broken"), "owner").unwrap_err();
    assert_eq!(missing.code, ValidationCode::UndeclaredReference);
    assert!(missing.message.contains("sample.Missing"));
}
