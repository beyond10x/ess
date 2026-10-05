//! A measure's condition (`docs/design/conditional-aggregate-measures.md`, beyond10x/ess#363): the
//! source-format gate, the reader's refusals of an empty condition, and the private structural
//! key that orders and hashes an [`Aggregate`](super::Aggregate) whose condition a resolved
//! [`Predicate`] holds.
//!
//! # Why a key and not a trait on `Predicate`
//!
//! [`Aggregate`](super::Aggregate) has always been `Ord` and `Hash`, and a predicate is neither.
//! Adding either trait to [`Predicate`] would make every predicate of the language orderable and
//! hashable by whatever its derive happened to do. The key is private to this module: it walks a
//! resolved predicate exhaustively — every variant has its own tag, no wildcard and no rendered
//! text — in the order its fields are declared, and compares numbers as numbers
//! ([`Number`]'s own order), so `2^53` and `2^53 + 1` are two keys and `1.5` and `1.50` one. It
//! is structural, not logical: children are never sorted, deduplicated or simplified.

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use ess_primitives::facts::{FactPath, FactValue, Number};
use ess_primitives::predicate::{
    CompareKind, CompareOp, Derived, FoldOp, OffsetDirection, OffsetMagnitude, Operand, Predicate,
    Quantified, TextOp, TextOperand,
};

/// The key under which `where` is written in an aggregate map.
pub(crate) const WHERE: &str = "where";

/// One token of a predicate's structural key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Token {
    /// Which variant, or which kind of operand, follows.
    Tag(u8),
    /// A collection's length, before its members.
    Len(usize),
    /// A path, a binder or a text literal.
    Text(String),
    /// A boolean literal.
    Bool(bool),
    /// A numeric literal: ordered by value, hashed by its exact spelling.
    Number(Number),
    /// An operator or a comparison kind.
    Op(u8),
}

impl Hash for Token {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Tag(tag) => (0_u8, tag).hash(state),
            Self::Len(len) => (1_u8, len).hash(state),
            Self::Text(text) => (2_u8, text).hash(state),
            Self::Bool(value) => (3_u8, value).hash(state),
            // `exact_text` is one spelling per value: equal numbers hash alike, signed zero
            // included, and no binary64 rounding merges two large integers.
            Self::Number(number) => (4_u8, number.exact_text()).hash(state),
            Self::Op(op) => (5_u8, op).hash(state),
        }
    }
}

/// The structural key of `predicate`, in pre-order.
fn key(predicate: &Predicate) -> Vec<Token> {
    let mut out = Vec::new();
    walk(predicate, &mut out);
    out
}

fn path(path: &FactPath, out: &mut Vec<Token>) {
    out.push(Token::Text(path.to_string()));
}

fn value(value: &FactValue, out: &mut Vec<Token>) {
    match value {
        FactValue::Bool(held) => {
            out.push(Token::Tag(0));
            out.push(Token::Bool(*held));
        }
        FactValue::Number(number) => {
            out.push(Token::Tag(1));
            out.push(Token::Number(*number));
        }
        FactValue::Text(text) => {
            out.push(Token::Tag(2));
            out.push(Token::Text(text.clone()));
        }
    }
}

fn values(held: &[FactValue], out: &mut Vec<Token>) {
    out.push(Token::Len(held.len()));
    for one in held {
        value(one, out);
    }
}

fn operand(operand: &Operand, out: &mut Vec<Token>) {
    match operand {
        Operand::Fact(fact) => {
            out.push(Token::Tag(0));
            path(fact, out);
        }
        Operand::Literal(literal) => {
            out.push(Token::Tag(1));
            value(literal, out);
        }
        Operand::Offset(offset) => {
            out.push(Token::Tag(2));
            path(&offset.base, out);
            out.push(Token::Op(match offset.direction {
                OffsetDirection::Add => 0,
                OffsetDirection::Subtract => 1,
            }));
            match &offset.magnitude {
                OffsetMagnitude::Integer(number) => {
                    out.push(Token::Tag(0));
                    out.push(Token::Number(*number));
                }
                OffsetMagnitude::ElapsedSeconds {
                    seconds,
                    written_unit,
                } => {
                    out.push(Token::Tag(1));
                    out.push(Token::Text(format!("{seconds}{}", written_unit.letter())));
                }
            }
        }
        Operand::Derived(derived) => {
            out.push(Token::Tag(3));
            match derived {
                Derived::Utf8Bytes(parent) => {
                    out.push(Token::Tag(0));
                    path(parent, out);
                }
            }
        }
    }
}

const fn compare_op(op: CompareOp) -> u8 {
    match op {
        CompareOp::Eq => 0,
        CompareOp::Ne => 1,
        CompareOp::Lt => 2,
        CompareOp::Le => 3,
        CompareOp::Gt => 4,
        CompareOp::Ge => 5,
    }
}

const fn compare_kind(kind: CompareKind) -> u8 {
    match kind {
        CompareKind::Value => 0,
        CompareKind::Instant => 1,
    }
}

const fn text_op(op: TextOp) -> u8 {
    match op {
        TextOp::StartsWith => 0,
        TextOp::EndsWith => 1,
        TextOp::Contains => 2,
    }
}

const fn fold_op(op: FoldOp) -> u8 {
    match op {
        FoldOp::EqualsIgnoreCase => 0,
        FoldOp::InIgnoreCase => 1,
    }
}

fn quantified(quantified: &Quantified, out: &mut Vec<Token>) {
    path(&quantified.over, out);
    out.push(Token::Text(quantified.bind.clone()));
    walk(&quantified.body, out);
}

/// Every variant, by its own tag: adding one to [`Predicate`] fails to compile here.
#[allow(clippy::too_many_lines)]
fn walk(predicate: &Predicate, out: &mut Vec<Token>) {
    match predicate {
        Predicate::Always => out.push(Token::Tag(0)),
        Predicate::Never => out.push(Token::Tag(1)),
        Predicate::All(children) => {
            out.push(Token::Tag(2));
            out.push(Token::Len(children.len()));
            for child in children {
                walk(child, out);
            }
        }
        Predicate::Any(children) => {
            out.push(Token::Tag(3));
            out.push(Token::Len(children.len()));
            for child in children {
                walk(child, out);
            }
        }
        Predicate::Not(child) => {
            out.push(Token::Tag(4));
            walk(child, out);
        }
        Predicate::Compare {
            left,
            op,
            right,
            kind,
        } => {
            out.push(Token::Tag(5));
            operand(left, out);
            out.push(Token::Op(compare_op(*op)));
            operand(right, out);
            out.push(Token::Op(compare_kind(*kind)));
        }
        Predicate::Truthy(fact) => {
            out.push(Token::Tag(6));
            path(fact, out);
        }
        Predicate::Defined(fact) => {
            out.push(Token::Tag(7));
            path(fact, out);
        }
        Predicate::AnyOf {
            path: fact,
            values: held,
        } => {
            out.push(Token::Tag(8));
            path(fact, out);
            values(held, out);
        }
        Predicate::NoneOf {
            path: fact,
            values: held,
        } => {
            out.push(Token::Tag(9));
            path(fact, out);
            values(held, out);
        }
        Predicate::TextMatch {
            path: fact,
            op,
            value: literal,
        } => {
            out.push(Token::Tag(10));
            path(fact, out);
            out.push(Token::Op(text_op(*op)));
            match literal {
                TextOperand::Literal(literal) => value(literal, out),
                // After every literal tag, so a parameter is never keyed as the text spelling it.
                TextOperand::Fact {
                    namespace, name, ..
                } => {
                    out.push(Token::Tag(3));
                    out.push(Token::Text(namespace.keyword().to_owned()));
                    out.push(Token::Text(name.clone()));
                }
            }
        }
        Predicate::FoldMatch {
            path: fact,
            op,
            values: held,
        } => {
            out.push(Token::Tag(11));
            path(fact, out);
            out.push(Token::Op(fold_op(*op)));
            values(held, out);
        }
        Predicate::Forall(body) => {
            out.push(Token::Tag(12));
            quantified(body, out);
        }
        Predicate::Exists(body) => {
            out.push(Token::Tag(13));
            quantified(body, out);
        }
        Predicate::Distinct(distinct) => {
            out.push(Token::Tag(14));
            path(&distinct.over, out);
            out.push(Token::Text(distinct.bind.clone()));
            match &distinct.key {
                Some(key) => {
                    out.push(Token::Tag(1));
                    path(key, out);
                }
                None => out.push(Token::Tag(0)),
            }
            out.push(Token::Text(
                distinct
                    .key_kind
                    .map_or("", |kind| kind.keyword())
                    .to_owned(),
            ));
        }
    }
}

/// The order of two optional conditions: none first, then the structural keys.
pub(super) fn cmp(left: Option<&Predicate>, right: Option<&Predicate>) -> Ordering {
    match (left, right) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Less,
        (Some(_), None) => Ordering::Greater,
        (Some(left), Some(right)) => key(left).cmp(&key(right)),
    }
}

/// Feeds a present condition to `state`: a presence discriminator, then the structural key with
/// its length. An absent condition feeds nothing, which keeps every unconditioned aggregate's
/// hash the one it had.
pub(super) fn hash<H: Hasher>(condition: Option<&Predicate>, state: &mut H) {
    if let Some(predicate) = condition {
        true.hash(state);
        key(predicate).hash(state);
    }
}

/// Whether a condition as written is empty: `null`, empty text, an empty list or an empty map.
/// Each would otherwise read as `Always`, a condition nobody wrote.
pub(super) fn empty(node: &ess_primitives::node::Node) -> bool {
    use ess_primitives::node::Node;
    match node {
        Node::Null => true,
        Node::Text(text) => text.trim().is_empty(),
        Node::Seq(items) => items.is_empty(),
        Node::Map(entries) => entries.is_empty(),
        Node::Bool(_) | Node::Number(_) => false,
    }
}
