//! Identities as opaque facts: how a view filter over an identity or a link field is decided
//! (beyond10x/ess#193).
//!
//! The identity of a row a scenario made is the implementation's to assign, so no scenario can
//! spell it. It can still *refer* to it — [`ScenarioValue::Instance`] names the instance an earlier
//! step captured, and [`ScenarioValue::Observed`] the event field the creating branch published it
//! in — and that is enough to decide the one question a filter over an identity usually asks: is
//! this row the one the parameter names? Each such reference is bound as a **token**, a text fact
//! (`instance:<name>`, `observed:<event>.<field>`) that stands for the value without being it.
//!
//! # What a token decides, and what it does not
//!
//! Two tokens meet only in `==` or `!=`, and decide it only where the answer does not depend on the
//! value behind them:
//!
//! - **the same token** is the same value, whatever it is: equal;
//! - **two instances of one type** are two rows of one entity, which never share an identity:
//!   different. An instance is a captured name, or the field a `creates:` branch publishes its new
//!   row's identity in — each scenario refers to the row it creates by that observation and to
//!   every other row by a name, so the two never alias. Any other observed field may carry any
//!   value, and two types may be two entities whose identities coincide, so neither is decided.
//!
//! Everything else a filter could ask of a token — an ordering, a literal, a text test, a length —
//! is a question about the value itself, and the token is left unbound there, so the filter stays
//! `Unknown` and the view is refused by name, exactly as it was before tokens existed.
use super::{
    BTreeMap, BTreeSet, Determined, EntitySpec, EssIr, EventRef, FactPath, FactValue, Node,
    Predicate, ResolvedBody, ResolvedInstance, ResolvedTypeRef, ResolvedView, ScenarioValue,
};
use ess_domain::view::ViewSpec;
use ess_primitives::predicate::{CompareOp, Operand};

/// One identity reference, bound as a fact.
struct Token {
    /// The text the fact holds.
    text: String,
    /// Whether it names an instance: one the scenario captured, or the identity a `creates:` branch
    /// published.
    instance: bool,
    /// The declared type of the path it is bound at, without `Optional`.
    type_ref: Option<ResolvedTypeRef>,
}

/// The token a value is bound as, where the value refers to an identity, and whether it names an
/// instance.
fn text(ir: &EssIr, value: &ScenarioValue) -> Option<(String, bool)> {
    match value {
        ScenarioValue::Instance { instance } => Some((format!("instance:{instance}"), true)),
        ScenarioValue::Observed { event, field } => Some((
            format!("observed:{event}.{field}"),
            published(ir, event, field),
        )),
        _ => None,
    }
}

/// Whether `event.field` is where a `creates:` branch publishes the identity of the row it made.
fn published(ir: &EssIr, event: &EventRef, field: &str) -> bool {
    ir.commands()
        .values()
        .flat_map(|command| &command.outcomes)
        .filter_map(|outcome| outcome.subject.as_ref())
        .any(|subject| {
            matches!(&subject.instance, ResolvedInstance::Observed { event: carried, field: named }
                if EventRef::from(carried) == *event && named.name == field)
        })
}

/// The declared type of a row field of `view`'s source, the identity included, or of a member of
/// one where `field` is a dotted path through structs (beyond10x/ess#428).
fn row_type(ir: &EssIr, view: &ResolvedView, field: &str) -> Option<ResolvedTypeRef> {
    let entity = ir.entity(&view.source);
    let mut segments = field.split('.');
    let root = segments.next()?;
    let mut type_ref = if entity.identity.name == root {
        entity.identity.type_ref.required().clone()
    } else {
        entity
            .fields
            .iter()
            .find(|declared| declared.name == root)
            .map(|declared| declared.type_ref.required().clone())?
    };
    for segment in segments {
        let ResolvedTypeRef::Declared { name } = &type_ref else {
            return None;
        };
        let ResolvedBody::Struct { fields, .. } = &ir.named_type(name).body else {
            return None;
        };
        type_ref = fields
            .iter()
            .find(|declared| declared.name == segment)
            .map(|declared| declared.type_ref.required().clone())?;
    }
    Some(type_ref)
}

/// The member at `path` below the root of a literal struct value, where every segment is there.
fn project<'a>(value: &'a Node, path: &[&str]) -> Option<&'a Node> {
    path.iter().try_fold(value, |held, segment| match held {
        Node::Map(members) => members.get(*segment),
        _ => None,
    })
}

/// The declared type of one of `view`'s parameters.
fn param_type(view: &ResolvedView, name: &str) -> Option<ResolvedTypeRef> {
    view.params
        .iter()
        .find(|param| param.name == name)
        .map(|param| param.type_ref.required().clone())
}

/// Every identity reference one row and one read bind, by path: the row's own identity, each field
/// the arrangement filled with an instance (a link to its owner), and each parameter the read sends
/// as one.
///
/// Only the facts the view's filter decides are returned; see [`decided`].
pub(super) fn facts(
    ir: &EssIr,
    view: &ResolvedView,
    settled: &BTreeMap<String, Determined>,
    identity: Option<&ScenarioValue>,
    params: &BTreeMap<String, ScenarioValue>,
) -> Vec<(FactPath, FactValue)> {
    let Some(filter) = &view.filter else {
        return Vec::new();
    };
    let mut tokens: BTreeMap<FactPath, Token> = BTreeMap::new();
    let mut add = |path: Option<FactPath>, value: &ScenarioValue, type_ref| {
        if let (Some(path), Some((text, instance))) = (path, text(ir, value)) {
            tokens.insert(
                path,
                Token {
                    text,
                    instance,
                    type_ref,
                },
            );
        }
    };
    let named = &ir.entity(&view.source).identity.name;
    if let Some(value) = identity {
        add(FactPath::new(named).ok(), value, row_type(ir, view, named));
    }
    for (name, determined) in settled {
        if matches!(determined.value, ScenarioValue::Instance { .. }) && name != named {
            add(
                FactPath::new(name).ok(),
                &determined.value,
                row_type(ir, view, name),
            );
        }
    }
    for (name, value) in params {
        add(
            FactPath::new(format!("{}.{name}", ViewSpec::PARAM)).ok(),
            value,
            param_type(view, name),
        );
    }
    let kept = decided(filter, &tokens);
    tokens
        .into_iter()
        .filter(|(path, _)| kept.contains(path))
        .map(|(path, token)| (path, FactValue::text(token.text)))
        .collect()
}

/// The token paths `filter` reads only where they decide: in an `==` or `!=` against another token
/// the two are [`comparable`] by, or in `defined()`.
///
/// A path read anywhere else is dropped, and dropping it can leave a comparison it took part in
/// without its other side, so this runs until nothing more is dropped.
fn decided(filter: &Predicate, tokens: &BTreeMap<FactPath, Token>) -> BTreeSet<FactPath> {
    let mut kept: BTreeSet<FactPath> = tokens.keys().cloned().collect();
    loop {
        let mut dropped = BTreeSet::new();
        visit(filter, &[], tokens, &kept, &mut dropped);
        if dropped.is_empty() {
            return kept;
        }
        kept.retain(|path| !dropped.contains(path));
    }
}

/// Whether an `==` or `!=` between two tokens is decided without the values behind them.
fn comparable(left: &Token, right: &Token) -> bool {
    left.text == right.text
        || (left.instance
            && right.instance
            && left.type_ref.is_some()
            && left.type_ref == right.type_ref)
}

fn visit(
    predicate: &Predicate,
    binders: &[&str],
    tokens: &BTreeMap<FactPath, Token>,
    kept: &BTreeSet<FactPath>,
    dropped: &mut BTreeSet<FactPath>,
) {
    // A path rooted at a quantifier's binder is an element, never a token.
    let free = |path: &FactPath| !binders.contains(&path.namespace());
    // A read of `path` that decides nothing about a token: the token itself, or one it is a prefix
    // of — `id.count` reads the length of the token's text.
    let mut refuse = |path: &FactPath, exact: bool| {
        if !free(path) {
            return;
        }
        for token in kept {
            let prefix = path.segments().len() > token.segments().len()
                && path.segments().starts_with(token.segments());
            if prefix || (exact && token == path) {
                dropped.insert(token.clone());
            }
        }
    };
    match predicate {
        Predicate::Always | Predicate::Never => {}
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                visit(child, binders, tokens, kept, dropped);
            }
        }
        Predicate::Not(inner) => visit(inner, binders, tokens, kept, dropped),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            refuse(&quantified.over, true);
            let mut inner = binders.to_vec();
            inner.push(quantified.bind.as_str());
            visit(&quantified.body, &inner, tokens, kept, dropped);
        }
        Predicate::Compare {
            left, op, right, ..
        } => {
            let (left, right) = (fact(left, binders), fact(right, binders));
            let decides = matches!(op, CompareOp::Eq | CompareOp::Ne)
                && match (left, right) {
                    (Some(left), Some(right)) => {
                        kept.contains(left)
                            && kept.contains(right)
                            && comparable(&tokens[left], &tokens[right])
                    }
                    _ => false,
                };
            for path in [left, right].into_iter().flatten() {
                refuse(path, !decides);
            }
        }
        Predicate::Defined(path) => refuse(path, false),
        leaf => {
            for path in leaf.fact_paths() {
                refuse(path, true);
            }
        }
    }
}

/// The free fact path an operand reads, if any.
fn fact<'a>(operand: &'a Operand, binders: &[&str]) -> Option<&'a FactPath> {
    match operand {
        Operand::Fact(path) if !binders.contains(&path.namespace()) => Some(path),
        _ => None,
    }
}

/// The value one of `view`'s parameters is sent as, read off the filter: what the row holds in the
/// one field, or the one member of a struct field, an `==` or `!=` compares the parameter with.
///
/// Read off the filter, never off a name (beyond10x/ess#428): `owner == param.who` sends `who` the
/// owner the scenario stored, and a parameter named after one field but compared with another is
/// sent the other. The parameter is declared at that field's type. The row's identity is sent as
/// the identity the scenario refers to it by; any other field as the literal or instance the
/// arrangement settled there; a member as the member of the literal struct settled at its root —
/// the row's identity included, which the caller settles under its name where the scenario sent it
/// as a literal. A struct the scenario holds only as an instance or an observed value has no member
/// projection, so its members are not bound here. A parameter compared with anything else, or with
/// two different fields, is left to whatever else binds it.
pub(super) fn param(
    ir: &EssIr,
    view: &ResolvedView,
    name: &str,
    settled: &BTreeMap<String, Determined>,
    identity: Option<&ScenarioValue>,
) -> Option<ScenarioValue> {
    let field = compared_with(ir, view, name)?;
    if field == ir.entity(&view.source).identity.name {
        return identity.cloned();
    }
    let segments: Vec<&str> = field.split('.').collect();
    let (root, members) = segments.split_first()?;
    match &settled.get(*root)?.value {
        ScenarioValue::Literal { value } => {
            project(value, members).map(|member| ScenarioValue::literal(member.clone()))
        }
        instance @ ScenarioValue::Instance { .. } if members.is_empty() => Some(instance.clone()),
        _ => None,
    }
}

/// The one row field, or dotted struct member, `view`'s filter compares the parameter `name` with
/// by `==` or `!=`, where it is declared at that field's type: what [`param`] sends it.
pub(super) fn compared_with(ir: &EssIr, view: &ResolvedView, name: &str) -> Option<String> {
    let filter = view.filter.as_ref()?;
    let wanted = FactPath::new(format!("{}.{name}", ViewSpec::PARAM)).ok()?;
    let mut fields = BTreeSet::new();
    compared(filter, &wanted, &mut fields);
    // A dotted path reads a struct member only where the declared types say it is one; any other
    // (`name.count`, a length) is not a field the parameter could be sent the value of.
    fields.retain(|field| !field.contains('.') || row_type(ir, view, field).is_some());
    let mut fields = fields.into_iter();
    let field = fields.next()?;
    if fields.next().is_some() || field == EntitySpec::STATE {
        return None;
    }
    let declared = row_type(ir, view, &field)?;
    (param_type(view, name).as_ref() == Some(&declared)).then_some(field)
}

/// The row paths below a root the scenario holds only by reference — an instance it captured or a
/// value it observed — that `filter` reads: a member of such a value has no projection, so nothing
/// can bind it (beyond10x/ess#428).
pub(super) fn unprojected(
    ir: &EssIr,
    view: &ResolvedView,
    settled: &BTreeMap<String, Determined>,
    unbound: &[FactPath],
) -> Vec<FactPath> {
    let identity = &ir.entity(&view.source).identity.name;
    unbound
        .iter()
        .filter(|path| path.segments().len() > 1 && path.namespace() != ViewSpec::PARAM)
        // A struct member, by the declared types: a length or any other read of a scalar is not.
        .filter(|path| row_type(ir, view, &path.segments().join(".")).is_some())
        .filter(|path| match settled.get(path.namespace()) {
            Some(determined) => !matches!(determined.value, ScenarioValue::Literal { .. }),
            None => path.namespace() == identity,
        })
        .cloned()
        .collect()
}

/// Every row path an `==` or `!=` compares with `param`: a field, or a dotted member of one.
fn compared(predicate: &Predicate, param: &FactPath, out: &mut BTreeSet<String>) {
    match predicate {
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                compared(child, param, out);
            }
        }
        Predicate::Not(inner) => compared(inner, param, out),
        Predicate::Compare {
            left: Operand::Fact(left),
            op: CompareOp::Eq | CompareOp::Ne,
            right: Operand::Fact(right),
            ..
        } => {
            for (one, other) in [(left, right), (right, left)] {
                if one == param && other.namespace() != ViewSpec::PARAM {
                    out.insert(other.segments().join("."));
                }
            }
        }
        _ => {}
    }
}
