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
                reading: None,
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

#[test]
fn clock_reading_provenance_cannot_be_erased_by_generic_comparison_or_wrappers() {
    let mut types = registry();
    types
        .insert(NamedType {
            name: "sample.Clock".parse().unwrap(),
            naming: Naming::default(),
            body: TypeBody::Newtype {
                of: TypeRef::parse("String").unwrap(),
                invariants: vec![],
            },
            reading: Some(ess_domain::reading::ReadingContract {
                encoding: ess_domain::reading::ReadingEncoding::OffsetDateTimeText,
                origins: vec![ess_domain::reading::ReadingOrigin {
                    role: ess_domain::reading::ReadingRole::ProducerProcess,
                    offset: ess_domain::reading::OffsetAuthority::EncodedOffset,
                }],
            }),
        })
        .unwrap();
    types
        .insert(NamedType {
            name: "sample.ClockWrapper".parse().unwrap(),
            naming: Naming::default(),
            body: TypeBody::Newtype {
                of: TypeRef::parse("sample.Clock").unwrap(),
                invariants: vec![],
            },
            reading: None,
        })
        .unwrap();
    let fields = [
        field("direct", "sample.Clock"),
        field("wrapped", "sample.ClockWrapper"),
        field("legacy", "Timestamp"),
    ];
    let environment = DomainEnvironment::new(&types, &fields);
    for name in ["direct", "wrapped"] {
        assert!(
            resolve_path(&environment, &path(name), "clock witness").is_err(),
            "{name}"
        );
    }
    assert!(resolve_path(&environment, &path("legacy"), "clock witness").is_ok());
}

/// Every form of `Predicate`, and whether that form can put a literal against a fact at all.
///
/// **This list is hand-written. Rust cannot enumerate an enum's variants, so something has to be.**
/// What is not hand-written is the obligation to extend it: `form_of` below is an exhaustive match
/// over `Predicate`, so a form added upstream stops this file compiling, and the only thing the
/// author can write in the new arm is a `Form` — which is either a new variant here, and then
/// `every_literal_carrying_form_has_a_case` fails until `enum_literal_cases` covers it, or an
/// existing variant, which is a wrong line written on purpose rather than an omission.
///
/// That is the whole of what is enforced, stated so nobody reads more into it. The compiler forces
/// the new form to be *named*; the test below forces a named literal-carrying form to be *covered*.
/// Neither can force the name to be the right one.
macro_rules! predicate_forms {
    ($($form:ident => $carries:expr,)+) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        enum Form { $($form,)+ }

        impl Form {
            /// Generated from the same token list as the variants, so it cannot be short of one.
            const ALL: &'static [Form] = &[$(Form::$form,)+];

            /// Whether *some* predicate of this form can carry a literal.
            ///
            /// A form-level answer, unlike [`carries_a_literal`], which answers for a value:
            /// `Predicate::All(vec![])` carries none, but `Predicate::All` is a form that can.
            fn may_carry_a_literal(self) -> bool {
                match self { $(Form::$form => $carries,)+ }
            }
        }
    };
}

predicate_forms! {
    Always => false,
    Never => false,
    Truthy => false,
    Defined => false,
    Compare => true,
    AnyOf => true,
    NoneOf => true,
    All => true,
    Any => true,
    Not => true,
    Forall => true,
    Exists => true,
    // It carries a literal, but never against an enum: a string operator over an enum is refused
    // as a type mismatch whatever its literal, declared variant or not, so the variant rule this
    // file checks has no case of it. `tests/string_operators.rs` asserts that refusal.
    TextMatch => false,
}

/// The form a predicate is, as an exhaustive match.
///
/// The single place upstream's variant list is read. Adding a `Predicate` variant stops this
/// compiling here.
fn form_of(predicate: &Predicate) -> Form {
    match predicate {
        Predicate::Always => Form::Always,
        Predicate::Never => Form::Never,
        Predicate::Truthy(_) => Form::Truthy,
        Predicate::Defined(_) => Form::Defined,
        Predicate::Compare { .. } => Form::Compare,
        Predicate::AnyOf { .. } => Form::AnyOf,
        Predicate::NoneOf { .. } => Form::NoneOf,
        Predicate::All(_) => Form::All,
        Predicate::Any(_) => Form::Any,
        Predicate::Not(_) => Form::Not,
        Predicate::Forall(_) => Form::Forall,
        Predicate::Exists(_) => Form::Exists,
        Predicate::TextMatch { .. } => Form::TextMatch,
    }
}

/// Whether a given predicate *does* put a literal against a fact.
///
/// Exhaustive on purpose, and a value-level answer: `Predicate::AnyOf` with no values carries no
/// literal even though its form can. Used below to assert each case is an instance of the rule
/// being checked, which a form-level answer cannot do.
///
/// `Predicate` is not `#[non_exhaustive]`, so a form added upstream stops this file compiling
/// rather than arriving quietly with a right-hand side nothing resolves — which is the defect
/// `story:enum-variant-in-an-entity-invariant` reports, one form at a time.
fn carries_a_literal(predicate: &Predicate) -> bool {
    match predicate {
        Predicate::Compare { left, right, .. } => {
            matches!(left, Operand::Literal(_)) || matches!(right, Operand::Literal(_))
        }
        Predicate::AnyOf { values, .. } | Predicate::NoneOf { values, .. } => !values.is_empty(),
        Predicate::TextMatch { .. } => true,
        Predicate::All(children) | Predicate::Any(children) => {
            children.iter().any(carries_a_literal)
        }
        Predicate::Not(inner) => carries_a_literal(inner),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            carries_a_literal(&quantified.body)
        }
        // Nothing to compare: these read a fact, or nothing at all.
        Predicate::Always | Predicate::Never | Predicate::Truthy(_) | Predicate::Defined(_) => {
            false
        }
    }
}

/// Every shape that puts a literal beside an enum-typed fact.
///
/// Hand-written, and `every_literal_carrying_form_has_a_case` is what keeps it from going stale:
/// it reads `form_of` off each case and requires one for every `Form::ALL` entry that
/// `may_carry_a_literal`. Some forms appear more than once — `Compare` carries the literal on
/// either side and under either operator — and that surplus is deliberate; the check is a floor,
/// not a census.
fn enum_literal_cases(variant: &str) -> Vec<(String, Predicate)> {
    let literal = || Operand::Literal(FactValue::text(variant));
    let eq = |left: Operand, right: Operand| Predicate::Compare {
        left,
        op: CompareOp::Eq,
        right,
    };
    let against = |fact: &str| eq(Operand::Fact(path(fact)), literal());
    vec![
        ("a literal on the right".to_owned(), against("state")),
        (
            "a literal on the left".to_owned(),
            eq(literal(), Operand::Fact(path("state"))),
        ),
        (
            "inequality".to_owned(),
            Predicate::Compare {
                left: Operand::Fact(path("state")),
                op: CompareOp::Ne,
                right: literal(),
            },
        ),
        ("through a newtype".to_owned(), against("wrapped_state")),
        (
            "through a struct member".to_owned(),
            against("lines.0.state"),
        ),
        (
            "any_of".to_owned(),
            Predicate::AnyOf {
                path: path("state"),
                values: vec![FactValue::text(variant)],
            },
        ),
        (
            "none_of".to_owned(),
            Predicate::NoneOf {
                path: path("state"),
                values: vec![FactValue::text(variant)],
            },
        ),
        (
            "under all".to_owned(),
            Predicate::All(vec![against("state")]),
        ),
        (
            "under any".to_owned(),
            Predicate::Any(vec![against("state")]),
        ),
        (
            "under not".to_owned(),
            Predicate::Not(Box::new(against("state"))),
        ),
        (
            "under forall, over a list".to_owned(),
            quantified("lines", "line", against("line.state")),
        ),
        (
            "under exists, over a map".to_owned(),
            Predicate::Exists(Box::new(Quantified {
                over: path("mapping"),
                bind: "entry".into(),
                body: against("entry.state"),
            })),
        ),
    ]
}

/// Every form that can carry a literal has a case in [`enum_literal_cases`].
///
/// The defect this closes: `carries_a_literal` is exhaustive, so a `Predicate` variant added
/// upstream breaks it and gets an arm — and then nothing at all requires a *case*. The new form
/// would be classified, compiled and silently untested, which is precisely the shape of the defect
/// `story:enum-variant-in-an-entity-invariant` reports.
///
/// What it can still miss is stated in [`Form`]'s own doc: an author who answers a new `Predicate`
/// variant with an existing `Form` satisfies both this check and the compiler. That is a wrong line
/// written deliberately, which is a different failure from a list quietly going short, and no test
/// in this file can tell it from a correct one.
#[test]
fn every_literal_carrying_form_has_a_case() {
    let covered: Vec<Form> = enum_literal_cases("Fax")
        .iter()
        .map(|(_, predicate)| form_of(predicate))
        .collect();

    let missing: Vec<&Form> = Form::ALL
        .iter()
        .filter(|form| form.may_carry_a_literal() && !covered.contains(form))
        .collect();
    assert!(
        missing.is_empty(),
        "{missing:?} can put a literal against a fact and `enum_literal_cases` has no case of it; \
         covered: {covered:?}"
    );

    let spurious: Vec<&Form> = covered
        .iter()
        .filter(|form| !form.may_carry_a_literal())
        .collect();
    assert!(
        spurious.is_empty(),
        "{spurious:?} is a case here and is classified as carrying no literal; one of the two is \
         wrong"
    );
}

#[test]
fn every_predicate_form_that_carries_a_literal_refuses_an_undeclared_enum_variant() {
    // The story's own "what this does not cover": equality against a variant is the reported
    // instance, and the question it leaves open is whether the other forms resolve their
    // right-hand side at all. They are checked here rather than assumed, in the one place that
    // decides it for every owner — an entity invariant, a view filter, a command guard, a value
    // object's invariant and a selection predicate all reach this checker.
    let registry = registry();
    let fields = fields();
    let env = DomainEnvironment::new(&registry, &fields);

    for (label, predicate) in enum_literal_cases("Fax") {
        assert!(
            carries_a_literal(&predicate),
            "{label} is not a case of the rule being checked"
        );
        let checked = check_predicate(&env, &predicate, "owner");
        let refused = checked
            .errors
            .iter()
            .find(|error| error.code == ValidationCode::UndeclaredReference)
            .unwrap_or_else(|| panic!("{label}: `Fax` was admitted by `{predicate}`"));
        // The enum that declares the variants, the variant that was written, and the variants
        // that exist. `sample.State` unconditionally, including the newtype case: the wrapper
        // `sample.WrappedState` is what the field declares, but the enum is the declaration the
        // author has to edit, and acceptance clause 2 asks for it by name. A prefix assertion on
        // `"sample."` is satisfied by the wrapper and cannot tell the two apart.
        assert!(
            refused.message.contains("`sample.State`")
                && refused.message.contains("`Ready`")
                && refused.message.contains("Fax"),
            "{label}: the refusal names the enum, the variant and what is declared: {}",
            refused.message
        );
    }

    // The same forms, with a variant the enum does declare. A checker that refused these would
    // pass the loop above and be worthless.
    for (label, predicate) in enum_literal_cases("Ready") {
        let checked = check_predicate(&env, &predicate, "owner");
        assert!(
            checked.errors.is_empty(),
            "{label}: `Ready` is declared, and `{predicate}` was refused: {}",
            checked.validation_errors()
        );
    }
}
