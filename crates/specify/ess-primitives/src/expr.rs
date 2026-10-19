//! The typed expression language (G1, `docs/design/generic-language-features.md` § G1).
//!
//! One grammar, one set of typing rules and one evaluator, shared by every position that computes a
//! value: a `{expr: "<expression>"}` source in `sets:` and `payload:`, a computed view field, and an
//! operand of a predicate. This module owns the syntax tree ([`Expr`]), its compact text form
//! ([`Expr::parse`] and [`fmt::Display`], which give each other back), the rules that type it
//! ([`Expr::infer`], over the [`Kind`]s a reader declares for each path it reads), and its
//! evaluation ([`Expr::evaluate`]). What a path *is* — which roots a position admits and which
//! declared type a path has — is decided where the declarations are, in `ess-domain`.
//!
//! # Grammar
//!
//! ```text
//! expr    = term (("+" | "-") term)*
//! term    = unary (("*" | "/") unary)*
//! unary   = "-" unary | primary
//! primary = number | duration | text | "true" | "false" | "now"
//!         | path | call | "(" expr ")"
//! call    = name "(" expr ("," expr)* ")"
//! ```
//!
//! A `number` is digits with an optional fraction (`12`, `1.5`); a `duration` is digits followed by
//! `s`, `m` or `h` (`60s`, `5m`, `1h`), the elapsed-time grammar `now - 60s` already reads; a `text`
//! is double-quoted with `\"` and `\\` escapes. A `path` is dotted segments of letters, digits and
//! `_`. The functions are `round(x, digits, half_even | half_up | down)`, `abs(x)`, `min(a, b)`,
//! `max(a, b)`, `coalesce(a, b)`, and the digest and encoding functions `sha256(x)`, `base64url(x)`
//! and `ascii(x)` (beyond10x/ess#584).
//!
//! # Values
//!
//! * **Exact numbers.** Integer arithmetic is exact inside signed 128 bits, and a result outside it
//!   is unknown, never wrapped. Decimal arithmetic is exact; `/` divides to six fractional digits,
//!   half-even, unless `round(…)` wraps it, in which case the exact quotient is rounded once to that
//!   call's digits and mode. A zero divisor makes the quotient absent, so `/` is `Optional`.
//! * **Time.** Elapsed seconds only: `Timestamp - Timestamp` is a `Duration`, `Timestamp ±
//!   Duration` a `Timestamp`, `Duration ± Duration` and `Duration * Integer` a `Duration`. A
//!   `Duration` is read from and written as ISO 8601 `PT…S` text; days, months and fractions of a
//!   second are no elapsed seconds and read as unknown, as does a negative duration written out.
//! * **Unknown and absent.** An unobserved read, an invalid instant, a non-ASCII text under `ascii`
//!   and an out-of-bound intermediate make the result [`Evaluated::Unknown`]; an absent `Optional`
//!   and a zero divisor make it [`Evaluated::Absent`], which `coalesce` replaces and every other
//!   form passes on.

pub mod digest;

use std::fmt;

use crate::facts::{FactPath, FactValue, Number};
use crate::time::{ElapsedUnit, Rfc3339Instant};

/// The value types an expression computes over: the declared primitives it can read, compute and
/// write. Every other declared type — an enum, a `Uuid`, a `Binary64`, a struct — is no operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    /// `Integer`, exact.
    Integer,
    /// `Decimal`, exact.
    Decimal,
    /// `Timestamp`: an RFC 3339 instant.
    Timestamp,
    /// `Duration`: whole elapsed seconds.
    Duration,
    /// `String`.
    String,
    /// `Bytes`, which travel as padded standard base64.
    Bytes,
    /// `Boolean`.
    Boolean,
}

impl Kind {
    /// The declared type's name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Integer => "Integer",
            Self::Decimal => "Decimal",
            Self::Timestamp => "Timestamp",
            Self::Duration => "Duration",
            Self::String => "String",
            Self::Bytes => "Bytes",
            Self::Boolean => "Boolean",
        }
    }

    fn numeric(self) -> bool {
        matches!(self, Self::Integer | Self::Decimal)
    }

    fn ordered(self) -> bool {
        matches!(
            self,
            Self::Integer | Self::Decimal | Self::Timestamp | Self::Duration | Self::String
        )
    }
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// The static type of an expression: a [`Kind`], and whether the value may be absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExprType {
    /// What the value is.
    pub kind: Kind,
    /// Whether the value may be absent: an `Optional` read, a quotient, or a form over either.
    pub optional: bool,
}

impl fmt::Display for ExprType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.optional {
            write!(f, "Optional<{}>", self.kind)
        } else {
            write!(f, "{}", self.kind)
        }
    }
}

/// How `round(…)` settles the digits it drops.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rounding {
    /// To the nearest, a tie to the even neighbour.
    HalfEven,
    /// To the nearest, a tie away from zero.
    HalfUp,
    /// Toward zero: the dropped digits are discarded.
    Down,
}

impl Rounding {
    /// The word it is written as.
    pub fn keyword(self) -> &'static str {
        match self {
            Self::HalfEven => "half_even",
            Self::HalfUp => "half_up",
            Self::Down => "down",
        }
    }

    fn parse(word: &str) -> Option<Self> {
        match word {
            "half_even" => Some(Self::HalfEven),
            "half_up" => Some(Self::HalfUp),
            "down" => Some(Self::Down),
            _ => None,
        }
    }
}

/// The most fractional digits `round(…)` keeps.
pub const MAX_ROUND_DIGITS: u8 = 18;

/// The fractional digits an unrounded `/` keeps, half-even, as an `avg` measure does.
pub const DIVISION_DIGITS: u8 = 6;

/// A function an expression calls, other than `round`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Function {
    /// `abs(x)`.
    Abs,
    /// `min(a, b)`.
    Min,
    /// `max(a, b)`.
    Max,
    /// `coalesce(a, b)`: `a` where it is present, otherwise `b`.
    Coalesce,
    /// `sha256(x)`: the SHA-256 digest of a `Bytes`, or of a `String`'s UTF-8 bytes (FIPS 180-4).
    Sha256,
    /// `base64url(x)`: a `Bytes` in base64url without padding (RFC 4648 § 5).
    Base64Url,
    /// `ascii(x)`: a `String`'s bytes where every character is ASCII; unknown otherwise.
    Ascii,
}

impl Function {
    /// Every function, in the order a refusal lists them.
    pub const ALL: [Self; 7] = [
        Self::Abs,
        Self::Min,
        Self::Max,
        Self::Coalesce,
        Self::Sha256,
        Self::Base64Url,
        Self::Ascii,
    ];

    /// The name it is called by.
    pub fn name(self) -> &'static str {
        match self {
            Self::Abs => "abs",
            Self::Min => "min",
            Self::Max => "max",
            Self::Coalesce => "coalesce",
            Self::Sha256 => "sha256",
            Self::Base64Url => "base64url",
            Self::Ascii => "ascii",
        }
    }

    /// How many arguments it takes.
    pub fn arity(self) -> usize {
        match self {
            Self::Abs | Self::Sha256 | Self::Base64Url | Self::Ascii => 1,
            Self::Min | Self::Max | Self::Coalesce => 2,
        }
    }

    fn named(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|function| function.name() == name)
    }

    /// Whether it is one of the digest and encoding functions (beyond10x/ess#584).
    pub fn is_digest(self) -> bool {
        matches!(self, Self::Sha256 | Self::Base64Url | Self::Ascii)
    }
}

/// A binary arithmetic operator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArithOp {
    /// `+`: a sum, a moved instant, or (over two `String`s) a concatenation.
    Add,
    /// `-`: a difference, or the `Duration` between two instants.
    Subtract,
    /// `*`.
    Multiply,
    /// `/`: always a `Decimal`, absent where the divisor is zero.
    Divide,
}

impl ArithOp {
    /// The symbol it is written with.
    pub fn symbol(self) -> char {
        match self {
            Self::Add => '+',
            Self::Subtract => '-',
            Self::Multiply => '*',
            Self::Divide => '/',
        }
    }

    fn precedence(self) -> u8 {
        match self {
            Self::Add | Self::Subtract => 1,
            Self::Multiply | Self::Divide => 2,
        }
    }
}

/// One expression.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expr {
    /// A whole-number literal.
    Integer(i128),
    /// A decimal literal with a fraction, held exactly.
    Decimal(Decimal),
    /// An elapsed-time literal, `60s`, with the unit it was written in.
    Duration {
        /// The seconds it names.
        seconds: i64,
        /// `s`, `m` or `h`, as written.
        unit: ElapsedUnit,
    },
    /// A double-quoted text literal.
    Text(String),
    /// `true` or `false`.
    Boolean(bool),
    /// The current time, `now`.
    Now,
    /// A fact read at a path. `kind` is what the declaration says it holds; the parser leaves it
    /// `None`, and a reader that knows the declarations fills it ([`Expr::with_kinds`]).
    Read {
        /// The path read.
        path: FactPath,
        /// What the path holds, where known.
        kind: Option<Kind>,
    },
    /// Unary minus.
    Negate(Box<Expr>),
    /// A binary arithmetic form.
    Arith {
        /// The operator.
        op: ArithOp,
        /// The left operand.
        left: Box<Expr>,
        /// The right operand.
        right: Box<Expr>,
    },
    /// `round(value, digits, mode)`.
    Round {
        /// What is rounded: an `Integer` or a `Decimal`, a quotient taken exactly.
        value: Box<Expr>,
        /// How many fractional digits are kept.
        digits: u8,
        /// How a tie and the dropped digits are settled.
        mode: Rounding,
    },
    /// Any other function call.
    Call {
        /// The function.
        function: Function,
        /// Its arguments, as many as [`Function::arity`].
        arguments: Vec<Expr>,
    },
}

/// Why a text is no expression, or an expression no well-typed one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExprError {
    /// What is wrong, in a sentence a document author reads.
    pub message: String,
}

impl ExprError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ExprError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ExprError {}

impl serde::Serialize for Expr {
    /// The canonical text, which [`Expr::parse`] reads back to the same tree.
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl Expr {
    /// Reads the compact text form.
    pub fn parse(text: &str) -> Result<Self, ExprError> {
        let tokens = lex(text)?;
        let mut parser = Parser {
            tokens: &tokens,
            at: 0,
        };
        let expr = parser.expr()?;
        if let Some(token) = parser.peek() {
            return Err(ExprError::new(format!(
                "`{text}`: `{token}` follows a complete expression; an operator (`+`, `-`, `*`, \
                 `/`) joins two"
            )));
        }
        Ok(expr)
    }

    /// Every path this expression reads, in the order written, each once.
    pub fn reads(&self) -> Vec<&FactPath> {
        let mut found = Vec::new();
        self.walk(&mut |expr| {
            if let Self::Read { path, .. } = expr {
                if !found.contains(&path) {
                    found.push(path);
                }
            }
        });
        found
    }

    /// Whether `now` is read anywhere in it.
    pub fn reads_now(&self) -> bool {
        let mut found = false;
        self.walk(&mut |expr| found |= matches!(expr, Self::Now));
        found
    }

    /// Every function this expression calls, `round` aside, in the order written.
    pub fn functions(&self) -> Vec<Function> {
        let mut found = Vec::new();
        self.walk(&mut |expr| {
            if let Self::Call { function, .. } = expr {
                found.push(*function);
            }
        });
        found
    }

    /// Whether it divides anywhere, and so may be absent where a divisor is zero.
    pub fn divides(&self) -> bool {
        let mut found = false;
        self.walk(&mut |expr| {
            found |= matches!(
                expr,
                Self::Arith {
                    op: ArithOp::Divide,
                    ..
                }
            );
        });
        found
    }

    fn walk<'a>(&'a self, visit: &mut dyn FnMut(&'a Self)) {
        visit(self);
        match self {
            Self::Negate(inner) | Self::Round { value: inner, .. } => inner.walk(visit),
            Self::Arith { left, right, .. } => {
                left.walk(visit);
                right.walk(visit);
            }
            Self::Call { arguments, .. } => {
                for argument in arguments {
                    argument.walk(visit);
                }
            }
            Self::Integer(_)
            | Self::Decimal(_)
            | Self::Duration { .. }
            | Self::Text(_)
            | Self::Boolean(_)
            | Self::Now
            | Self::Read { .. } => {}
        }
    }

    /// The same expression with every read's path rewritten by `rewrite`, and its kind kept.
    #[must_use]
    pub fn map_paths(&self, rewrite: &dyn Fn(&FactPath) -> FactPath) -> Self {
        self.map_reads(&|path, kind| Self::Read {
            path: rewrite(path),
            kind,
        })
    }

    /// The same expression with every read's kind set by `kind_of`.
    #[must_use]
    pub fn with_kinds(&self, kind_of: &dyn Fn(&FactPath) -> Option<Kind>) -> Self {
        self.map_reads(&|path, _| Self::Read {
            path: path.clone(),
            kind: kind_of(path),
        })
    }

    fn map_reads(&self, read: &dyn Fn(&FactPath, Option<Kind>) -> Self) -> Self {
        match self {
            Self::Read { path, kind } => read(path, *kind),
            Self::Negate(inner) => Self::Negate(Box::new(inner.map_reads(read))),
            Self::Arith { op, left, right } => Self::Arith {
                op: *op,
                left: Box::new(left.map_reads(read)),
                right: Box::new(right.map_reads(read)),
            },
            Self::Round {
                value,
                digits,
                mode,
            } => Self::Round {
                value: Box::new(value.map_reads(read)),
                digits: *digits,
                mode: *mode,
            },
            Self::Call {
                function,
                arguments,
            } => Self::Call {
                function: *function,
                arguments: arguments
                    .iter()
                    .map(|argument| argument.map_reads(read))
                    .collect(),
            },
            Self::Integer(_)
            | Self::Decimal(_)
            | Self::Duration { .. }
            | Self::Text(_)
            | Self::Boolean(_)
            | Self::Now => self.clone(),
        }
    }

    /// The type of this expression, where every path it reads has the type `read` gives it; or why
    /// it has none, naming the form and the types it met.
    pub fn infer(
        &self,
        read: &dyn Fn(&FactPath) -> Result<ExprType, ExprError>,
    ) -> Result<ExprType, ExprError> {
        let plain = |kind| ExprType {
            kind,
            optional: false,
        };
        match self {
            Self::Integer(_) => Ok(plain(Kind::Integer)),
            Self::Decimal(_) => Ok(plain(Kind::Decimal)),
            Self::Duration { .. } => Ok(plain(Kind::Duration)),
            Self::Text(_) => Ok(plain(Kind::String)),
            Self::Boolean(_) => Ok(plain(Kind::Boolean)),
            Self::Now => Ok(plain(Kind::Timestamp)),
            Self::Read { path, .. } => read(path),
            Self::Negate(inner) => {
                let inner_type = inner.infer(read)?;
                if matches!(
                    inner_type.kind,
                    Kind::Integer | Kind::Decimal | Kind::Duration
                ) {
                    Ok(inner_type)
                } else {
                    Err(ExprError::new(format!(
                        "`{self}` negates a {}; unary `-` applies to an Integer, a Decimal or a \
                         Duration",
                        inner_type.kind
                    )))
                }
            }
            Self::Arith { op, left, right } => {
                let (left_type, right_type) = (left.infer(read)?, right.infer(read)?);
                let kind = arith_kind(*op, left_type.kind, right_type.kind).ok_or_else(|| {
                    ExprError::new(format!(
                        "`{self}` applies `{}` to a {} and a {}; {}",
                        op.symbol(),
                        left_type.kind,
                        right_type.kind,
                        arith_rule(*op)
                    ))
                })?;
                Ok(ExprType {
                    kind,
                    optional: left_type.optional || right_type.optional || *op == ArithOp::Divide,
                })
            }
            Self::Round { value, .. } => {
                let value_type = value.infer(read)?;
                if !value_type.kind.numeric() {
                    return Err(ExprError::new(format!(
                        "`{self}` rounds a {}; `round` applies to an Integer or a Decimal",
                        value_type.kind
                    )));
                }
                Ok(ExprType {
                    kind: Kind::Decimal,
                    optional: value_type.optional,
                })
            }
            Self::Call {
                function,
                arguments,
            } => {
                let types = arguments
                    .iter()
                    .map(|argument| argument.infer(read))
                    .collect::<Result<Vec<_>, _>>()?;
                call_type(*function, &types).ok_or_else(|| {
                    ExprError::new(format!(
                        "`{self}` calls `{}` with {}; {}",
                        function.name(),
                        types
                            .iter()
                            .map(|typed| format!("a {}", typed.kind))
                            .collect::<Vec<_>>()
                            .join(" and "),
                        call_rule(*function)
                    ))
                })
            }
        }
    }

    /// Evaluates this expression over `facts`.
    pub fn evaluate(&self, facts: &dyn Reads) -> Evaluated {
        match self.eval(facts) {
            Ok(Some(value)) => Evaluated::Value(value),
            Ok(None) => Evaluated::Absent,
            Err(Unknown) => Evaluated::Unknown,
        }
    }

    fn eval(&self, facts: &dyn Reads) -> Result<Option<Value>, Unknown> {
        Ok(Some(match self {
            Self::Integer(value) => Value::Integer(*value),
            Self::Decimal(value) => Value::Decimal(*value),
            Self::Duration { seconds, .. } => Value::Duration(i128::from(*seconds)),
            Self::Text(text) => Value::String(text.clone()),
            Self::Boolean(value) => Value::Boolean(*value),
            Self::Now => Value::Timestamp(facts.now().ok_or(Unknown)?),
            Self::Read { path, kind } => match facts.read(path) {
                Reading::Value(value) => Value::decode(&value, *kind).ok_or(Unknown)?,
                Reading::Absent => return Ok(None),
                Reading::Unknown => return Err(Unknown),
            },
            Self::Negate(inner) => {
                let Some(value) = inner.eval(facts)? else {
                    return Ok(None);
                };
                match value {
                    Value::Integer(value) => Value::Integer(value.checked_neg().ok_or(Unknown)?),
                    Value::Decimal(value) => Value::Decimal(value.negate().ok_or(Unknown)?),
                    Value::Duration(value) => Value::Duration(value.checked_neg().ok_or(Unknown)?),
                    _ => return Err(Unknown),
                }
            }
            Self::Arith { op, left, right } => {
                let (left, right) = (left.eval(facts)?, right.eval(facts)?);
                let (Some(left), Some(right)) = (left, right) else {
                    return Ok(None);
                };
                return arith(*op, left, right);
            }
            Self::Round {
                value,
                digits,
                mode,
            } => {
                // A quotient is rounded once, exactly, at the call's own digits and mode.
                if let Self::Arith {
                    op: ArithOp::Divide,
                    left,
                    right,
                } = value.as_ref()
                {
                    let (left, right) = (left.eval(facts)?, right.eval(facts)?);
                    let (Some(left), Some(right)) = (left, right) else {
                        return Ok(None);
                    };
                    return divide(&left, &right, *digits, *mode);
                }
                let Some(value) = value.eval(facts)? else {
                    return Ok(None);
                };
                let exact = value.as_decimal().ok_or(Unknown)?;
                Value::Decimal(exact.round(*digits, *mode).ok_or(Unknown)?)
            }
            Self::Call {
                function,
                arguments,
            } => return call(*function, arguments, facts),
        }))
    }

    /// The binding strength of the form at the top of this expression, for the printer.
    fn precedence(&self) -> u8 {
        match self {
            Self::Arith { op, .. } => op.precedence(),
            Self::Negate(_) => 3,
            _ => 4,
        }
    }
}

/// Why a fold over a fact stopped: something it read was unknown or out of bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Unknown;

/// What a path reads as, for an evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reading {
    /// A value is bound there.
    Value(FactValue),
    /// The path is an absent `Optional`.
    Absent,
    /// Nothing was observed there.
    Unknown,
}

impl Reading {
    /// A value as a document carries it: `null` is absent, a scalar is its value, and a mapping or
    /// a sequence is no operand of an expression, so unknown.
    pub fn of_node(node: &crate::node::Node) -> Self {
        use crate::node::Node;
        match node {
            Node::Null => Self::Absent,
            Node::Bool(value) => Self::Value(FactValue::Bool(*value)),
            Node::Number(number) => Self::Value(FactValue::Number(*number)),
            Node::Text(text) => Self::Value(FactValue::Text(text.clone())),
            Node::Seq(_) | Node::Map(_) => Self::Unknown,
        }
    }
}

/// Where an evaluation reads its facts and its clock.
pub trait Reads {
    /// The fact at `path`.
    fn read(&self, path: &FactPath) -> Reading;
    /// The current time, where a clock is set.
    fn now(&self) -> Option<Rfc3339Instant> {
        None
    }
}

impl<F: Fn(&FactPath) -> Reading> Reads for F {
    fn read(&self, path: &FactPath) -> Reading {
        self(path)
    }
}

/// The facts of a [`crate::facts::FactSource`], read as an expression reads them: a bound value, an
/// observed absence, or nothing observed.
pub struct SourceReads<'a>(pub &'a dyn crate::facts::FactSource);

impl Reads for SourceReads<'_> {
    fn read(&self, path: &FactPath) -> Reading {
        match self.0.fact(path) {
            Some(value) => Reading::Value(value),
            None if self.0.observed_presence(path) == Some(false) => Reading::Absent,
            None => Reading::Unknown,
        }
    }

    fn now(&self) -> Option<Rfc3339Instant> {
        self.0.now()
    }
}

/// The result of evaluating an expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Evaluated {
    /// A value.
    Value(Value),
    /// No value: an absent `Optional`, or a zero divisor.
    Absent,
    /// Not decidable from what was observed, or out of bounds.
    Unknown,
}

impl Evaluated {
    /// The value as the fact it is written as, `None` where there is none or it has no spelling
    /// (a negative `Duration`, an Integer past `i64`, a Decimal with more places than a written
    /// number keeps).
    pub fn to_fact(&self) -> Option<FactValue> {
        match self {
            Self::Value(value) => value.to_fact(),
            Self::Absent | Self::Unknown => None,
        }
    }

    /// The value as a document carries it: absent is `null`; `None` where it is unknown or has
    /// no spelling ([`Self::to_fact`]).
    pub fn to_node(&self) -> Option<crate::node::Node> {
        use crate::node::Node;
        match self {
            Self::Absent => Some(Node::Null),
            Self::Unknown => None,
            Self::Value(value) => Some(match value.to_fact()? {
                FactValue::Bool(value) => Node::Bool(value),
                FactValue::Number(number) => Node::Number(number),
                FactValue::Text(text) => Node::Text(text),
            }),
        }
    }
}

/// One computed value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    /// An exact integer.
    Integer(i128),
    /// An exact decimal.
    Decimal(Decimal),
    /// An instant.
    Timestamp(Rfc3339Instant),
    /// Elapsed seconds, possibly negative inside a computation.
    Duration(i128),
    /// Text.
    String(String),
    /// Bytes.
    Bytes(Vec<u8>),
    /// A truth value.
    Boolean(bool),
}

impl Value {
    /// Reads a fact as a value of `kind`; with no kind known, a whole number is an Integer, any
    /// other number a Decimal, a text a String and a boolean a Boolean.
    pub fn decode(fact: &FactValue, kind: Option<Kind>) -> Option<Self> {
        match (kind, fact) {
            (Some(Kind::Integer), FactValue::Number(number)) => {
                Some(Self::Integer(i128::from(number.as_i64()?)))
            }
            (Some(Kind::Integer), FactValue::Text(text)) => text
                .parse::<i64>()
                .ok()
                .map(|value| Self::Integer(value.into())),
            (Some(Kind::Decimal), FactValue::Number(number)) => {
                Decimal::of_number(*number).map(Self::Decimal)
            }
            (Some(Kind::Decimal), FactValue::Text(text)) => {
                Decimal::of_number(Number::decimal_literal(text)?).map(Self::Decimal)
            }
            (Some(Kind::Timestamp), FactValue::Text(text)) => {
                Rfc3339Instant::parse_rfc3339(text).map(Self::Timestamp)
            }
            (Some(Kind::Duration), FactValue::Text(text)) => {
                parse_elapsed(text).map(|seconds| Self::Duration(seconds.into()))
            }
            (Some(Kind::String) | None, FactValue::Text(text)) => Some(Self::String(text.clone())),
            (Some(Kind::Bytes), FactValue::Text(text)) => {
                digest::decode_base64_standard(text).map(Self::Bytes)
            }
            (Some(Kind::Boolean) | None, FactValue::Bool(value)) => Some(Self::Boolean(*value)),
            (None, FactValue::Number(number)) => match number.as_i64() {
                Some(integer) => Some(Self::Integer(integer.into())),
                None => Decimal::of_number(*number).map(Self::Decimal),
            },
            _ => None,
        }
    }

    /// The value as the fact it is written as, or `None` where it has no spelling.
    pub fn to_fact(&self) -> Option<FactValue> {
        match self {
            Self::Integer(value) => {
                Some(FactValue::Number(Number::from(i64::try_from(*value).ok()?)))
            }
            Self::Decimal(value) => value.to_number().map(FactValue::Number),
            Self::Timestamp(instant) => Some(FactValue::Text(instant.to_rfc3339())),
            Self::Duration(seconds) => {
                (*seconds >= 0).then(|| FactValue::Text(format!("PT{seconds}S")))
            }
            Self::String(text) => Some(FactValue::Text(text.clone())),
            Self::Bytes(bytes) => Some(FactValue::Text(digest::base64_standard(bytes))),
            Self::Boolean(value) => Some(FactValue::Bool(*value)),
        }
    }

    fn as_decimal(&self) -> Option<Decimal> {
        match self {
            Self::Integer(value) => Some(Decimal::integer(*value)),
            Self::Decimal(value) => Some(*value),
            _ => None,
        }
    }
}

/// Reads ISO 8601 elapsed time, `PT[<n>H][<n>M][<n>S]` in whole units with at least one part, as
/// seconds; anything with days, months, years, weeks, a fraction or a sign is `None`.
pub fn parse_elapsed(text: &str) -> Option<i64> {
    let mut rest = text.strip_prefix("PT")?;
    if rest.is_empty() {
        return None;
    }
    let mut total: i64 = 0;
    let mut last_rank = 0u8;
    while !rest.is_empty() {
        let digits_end = rest
            .bytes()
            .position(|byte| !byte.is_ascii_digit())
            .unwrap_or(rest.len());
        if digits_end == 0 || digits_end == rest.len() {
            return None;
        }
        let amount: i64 = rest[..digits_end].parse().ok()?;
        let (rank, unit) = match rest.as_bytes()[digits_end] {
            b'H' => (1, 3_600),
            b'M' => (2, 60),
            b'S' => (3, 1),
            _ => return None,
        };
        if rank <= last_rank {
            return None;
        }
        last_rank = rank;
        total = total.checked_add(amount.checked_mul(unit)?)?;
        rest = &rest[digits_end + 1..];
    }
    Some(total)
}

/// An exact decimal, `units × 10^-scale`, normalised so one value has one spelling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Decimal {
    units: i128,
    scale: u32,
}

/// The largest scale a computed decimal keeps; past it the value is unknown.
const MAX_DECIMAL_SCALE: u32 = 38;

impl Decimal {
    /// `units × 10^-scale`, normalised; `None` past the largest scale kept.
    pub fn new(units: i128, scale: u32) -> Option<Self> {
        let (mut units, mut scale) = (units, scale);
        while scale > 0 && units % 10 == 0 {
            units /= 10;
            scale -= 1;
        }
        (scale <= MAX_DECIMAL_SCALE).then_some(Self { units, scale })
    }

    fn integer(units: i128) -> Self {
        Self { units, scale: 0 }
    }

    /// The exact decimal a number carries, where it carries one.
    pub fn of_number(number: Number) -> Option<Self> {
        let (units, scale) = number.exact_parts()?;
        Self::new(units, u32::from(scale))
    }

    /// The number this decimal is written as, where a written number keeps every digit.
    pub fn to_number(self) -> Option<Number> {
        if self.scale == 0 {
            return Some(Number::from(i64::try_from(self.units).ok()?));
        }
        Number::decimal_literal(&self.to_string())
    }

    fn negate(self) -> Option<Self> {
        Some(Self {
            units: self.units.checked_neg()?,
            scale: self.scale,
        })
    }

    /// Both at the larger scale.
    fn aligned(self, other: Self) -> Option<(i128, i128, u32)> {
        let scale = self.scale.max(other.scale);
        Some((
            widen(self.units, scale - self.scale)?,
            widen(other.units, scale - other.scale)?,
            scale,
        ))
    }

    fn add(self, other: Self) -> Option<Self> {
        let (a, b, scale) = self.aligned(other)?;
        Self::new(a.checked_add(b)?, scale)
    }

    fn subtract(self, other: Self) -> Option<Self> {
        let (a, b, scale) = self.aligned(other)?;
        Self::new(a.checked_sub(b)?, scale)
    }

    fn multiply(self, other: Self) -> Option<Self> {
        Self::new(
            self.units.checked_mul(other.units)?,
            self.scale.checked_add(other.scale)?,
        )
    }

    fn compare(self, other: Self) -> Option<std::cmp::Ordering> {
        let (a, b, _) = self.aligned(other)?;
        Some(a.cmp(&b))
    }

    /// This value at `digits` fractional digits, the dropped digits settled by `mode`.
    pub fn round(self, digits: u8, mode: Rounding) -> Option<Self> {
        let digits = u32::from(digits);
        if self.scale <= digits {
            return Some(self);
        }
        let divisor = widen(1, self.scale - digits)?;
        Self::new(round_quotient(self.units, divisor, mode)?, digits)
    }

    /// `self / other` at `digits` fractional digits, settled by `mode`, computed from the exact
    /// quotient; `None` where the divisor is zero or a bound is passed.
    fn divide(self, other: Self, digits: u8, mode: Rounding) -> Option<Self> {
        if other.units == 0 {
            return None;
        }
        // self / other = (a · 10^-sa) / (b · 10^-sb) = (a · 10^(sb + digits)) / (b · 10^sa) · 10^-digits
        let numerator = self
            .units
            .checked_mul(widen(1, other.scale.checked_add(u32::from(digits))?)?)?;
        let denominator = other.units.checked_mul(widen(1, self.scale)?)?;
        Self::new(
            round_quotient(numerator, denominator, mode)?,
            u32::from(digits),
        )
    }
}

impl fmt::Display for Decimal {
    /// Without an exponent and without trailing zeroes.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let digits = self.units.unsigned_abs().to_string();
        let sign = if self.units < 0 { "-" } else { "" };
        let scale = usize::try_from(self.scale).unwrap_or(usize::MAX);
        if scale == 0 {
            return write!(f, "{sign}{digits}");
        }
        let padded = if digits.len() <= scale {
            format!("{}{digits}", "0".repeat(scale - digits.len() + 1))
        } else {
            digits
        };
        let (integer, fraction) = padded.split_at(padded.len() - scale);
        write!(f, "{sign}{integer}.{fraction}")
    }
}

fn widen(units: i128, places: u32) -> Option<i128> {
    (0..places).try_fold(units, |units, _| units.checked_mul(10))
}

/// `numerator / denominator`, an integer, settled by `mode`.
fn round_quotient(numerator: i128, denominator: i128, mode: Rounding) -> Option<i128> {
    if denominator == 0 {
        return None;
    }
    let quotient = numerator.checked_div(denominator)?;
    let remainder = numerator.checked_rem(denominator)?;
    if remainder == 0 || mode == Rounding::Down {
        return Some(quotient);
    }
    let negative = (numerator < 0) != (denominator < 0);
    let away = if negative {
        quotient.checked_sub(1)?
    } else {
        quotient.checked_add(1)?
    };
    let twice = remainder.unsigned_abs().checked_mul(2)?;
    let whole = denominator.unsigned_abs();
    let rounds_away = match twice.cmp(&whole) {
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Greater => true,
        std::cmp::Ordering::Equal => match mode {
            Rounding::HalfUp => true,
            Rounding::HalfEven => quotient % 2 != 0,
            Rounding::Down => false,
        },
    };
    Some(if rounds_away { away } else { quotient })
}

/// The kind `op` gives over `left` and `right`, or `None` where it applies to neither.
pub fn arith_kind(op: ArithOp, left: Kind, right: Kind) -> Option<Kind> {
    use Kind::{Decimal as D, Duration as Dur, Integer as I, String as S, Timestamp as T};
    match (op, left, right) {
        (ArithOp::Divide, a, b) if a.numeric() && b.numeric() => Some(D),
        (_, I, I) => Some(I),
        (_, a, b) if a.numeric() && b.numeric() => Some(D),
        (ArithOp::Add | ArithOp::Subtract, T, Dur) | (ArithOp::Add, Dur, T) => Some(T),
        (ArithOp::Subtract, T, T)
        | (ArithOp::Add | ArithOp::Subtract, Dur, Dur)
        | (ArithOp::Multiply, Dur, I)
        | (ArithOp::Multiply, I, Dur) => Some(Dur),
        (ArithOp::Add, S, S) => Some(S),
        _ => None,
    }
}

fn arith_rule(op: ArithOp) -> &'static str {
    match op {
        ArithOp::Add => {
            "`+` adds two Integers or Decimals, moves a Timestamp by a Duration, adds two \
             Durations, or joins two Strings"
        }
        ArithOp::Subtract => {
            "`-` subtracts two Integers or Decimals, two Durations, a Duration from a Timestamp, \
             or a Timestamp from a Timestamp, which is a Duration"
        }
        ArithOp::Multiply => "`*` multiplies two Integers or Decimals, or a Duration by an Integer",
        ArithOp::Divide => "`/` divides two Integers or Decimals, and is an Optional<Decimal>",
    }
}

fn call_type(function: Function, types: &[ExprType]) -> Option<ExprType> {
    let any_optional = types.iter().any(|typed| typed.optional);
    let with = |kind| ExprType {
        kind,
        optional: any_optional,
    };
    match (function, types) {
        (Function::Abs, [value])
            if matches!(value.kind, Kind::Integer | Kind::Decimal | Kind::Duration) =>
        {
            Some(*value)
        }
        (Function::Min | Function::Max, [a, b]) if a.kind.ordered() && b.kind.ordered() => {
            if a.kind == b.kind {
                Some(with(a.kind))
            } else if a.kind.numeric() && b.kind.numeric() {
                Some(with(Kind::Decimal))
            } else {
                None
            }
        }
        (Function::Coalesce, [a, b]) => {
            let kind = if a.kind == b.kind {
                a.kind
            } else if a.kind.numeric() && b.kind.numeric() {
                Kind::Decimal
            } else {
                return None;
            };
            Some(ExprType {
                kind,
                optional: b.optional,
            })
        }
        (Function::Sha256, [value]) if matches!(value.kind, Kind::String | Kind::Bytes) => {
            Some(with(Kind::Bytes))
        }
        (Function::Base64Url, [value]) if value.kind == Kind::Bytes => Some(with(Kind::String)),
        (Function::Ascii, [value]) if value.kind == Kind::String => Some(with(Kind::Bytes)),
        _ => None,
    }
}

fn call_rule(function: Function) -> &'static str {
    match function {
        Function::Abs => "`abs` takes an Integer, a Decimal or a Duration",
        Function::Min | Function::Max => {
            "it takes two values of one ordered type (Integer, Decimal, Timestamp, Duration, \
             String), or an Integer and a Decimal"
        }
        Function::Coalesce => {
            "`coalesce` takes two values of one type, or an Integer and a Decimal"
        }
        Function::Sha256 => "`sha256` takes a String (its UTF-8 bytes) or Bytes",
        Function::Base64Url => "`base64url` takes Bytes, such as `sha256(…)`",
        Function::Ascii => "`ascii` takes a String",
    }
}

fn arith(op: ArithOp, left: Value, right: Value) -> Result<Option<Value>, Unknown> {
    use Value::{Duration as Dur, Integer as I, Timestamp as T};
    let bound = |value: Option<i128>| value.ok_or(Unknown);
    Ok(Some(match (op, left, right) {
        (ArithOp::Divide, left, right) => {
            return divide(&left, &right, DIVISION_DIGITS, Rounding::HalfEven);
        }
        (ArithOp::Add, I(a), I(b)) => I(bound(a.checked_add(b))?),
        (ArithOp::Subtract, I(a), I(b)) => I(bound(a.checked_sub(b))?),
        (ArithOp::Multiply, I(a), I(b)) => I(bound(a.checked_mul(b))?),
        (ArithOp::Add, T(at), Dur(seconds)) | (ArithOp::Add, Dur(seconds), T(at)) => {
            T(moved(at, seconds)?)
        }
        (ArithOp::Subtract, T(at), Dur(seconds)) => T(moved(at, bound(seconds.checked_neg())?)?),
        (ArithOp::Subtract, T(later), T(earlier)) => {
            Dur(later.whole_seconds_since(earlier).ok_or(Unknown)?.into())
        }
        (ArithOp::Add, Dur(a), Dur(b)) => Dur(bound(a.checked_add(b))?),
        (ArithOp::Subtract, Dur(a), Dur(b)) => Dur(bound(a.checked_sub(b))?),
        (ArithOp::Multiply, Dur(a), I(b)) | (ArithOp::Multiply, I(b), Dur(a)) => {
            Dur(bound(a.checked_mul(b))?)
        }
        (ArithOp::Add, Value::String(a), Value::String(b)) => Value::String(a + &b),
        (op, left, right) => {
            let (Some(a), Some(b)) = (left.as_decimal(), right.as_decimal()) else {
                return Err(Unknown);
            };
            Value::Decimal(
                match op {
                    ArithOp::Add => a.add(b),
                    ArithOp::Subtract => a.subtract(b),
                    ArithOp::Multiply => a.multiply(b),
                    ArithOp::Divide => unreachable!("handled above"),
                }
                .ok_or(Unknown)?,
            )
        }
    }))
}

fn moved(at: Rfc3339Instant, seconds: i128) -> Result<Rfc3339Instant, Unknown> {
    at.plus_elapsed(i64::try_from(seconds).map_err(|_| Unknown)?)
        .ok_or(Unknown)
}

fn divide(
    left: &Value,
    right: &Value,
    digits: u8,
    mode: Rounding,
) -> Result<Option<Value>, Unknown> {
    let (Some(a), Some(b)) = (left.as_decimal(), right.as_decimal()) else {
        return Err(Unknown);
    };
    if b.units == 0 {
        return Ok(None);
    }
    a.divide(b, digits, mode)
        .map(|quotient| Some(Value::Decimal(quotient)))
        .ok_or(Unknown)
}

fn call(
    function: Function,
    arguments: &[Expr],
    facts: &dyn Reads,
) -> Result<Option<Value>, Unknown> {
    if function == Function::Coalesce {
        let [first, second] = arguments else {
            return Err(Unknown);
        };
        return match first.eval(facts)? {
            Some(value) => Ok(Some(value)),
            None => second.eval(facts),
        };
    }
    let mut values = Vec::with_capacity(arguments.len());
    for argument in arguments {
        let Some(value) = argument.eval(facts)? else {
            return Ok(None);
        };
        values.push(value);
    }
    Ok(Some(match (function, values.as_slice()) {
        (Function::Abs, [Value::Integer(value)]) => {
            Value::Integer(value.checked_abs().ok_or(Unknown)?)
        }
        (Function::Abs, [Value::Decimal(value)]) => Value::Decimal(if value.units < 0 {
            value.negate().ok_or(Unknown)?
        } else {
            *value
        }),
        (Function::Abs, [Value::Duration(value)]) => {
            Value::Duration(value.checked_abs().ok_or(Unknown)?)
        }
        (Function::Min | Function::Max, [a, b]) => {
            let ordering = order(a, b).ok_or(Unknown)?;
            let first = (function == Function::Min) == (ordering != std::cmp::Ordering::Greater);
            let chosen = if first { a } else { b };
            match (a, b) {
                (Value::Integer(_), Value::Decimal(_)) | (Value::Decimal(_), Value::Integer(_)) => {
                    Value::Decimal(chosen.as_decimal().ok_or(Unknown)?)
                }
                _ => chosen.clone(),
            }
        }
        (Function::Sha256, [Value::String(text)]) => {
            Value::Bytes(digest::sha256(text.as_bytes()).to_vec())
        }
        (Function::Sha256, [Value::Bytes(bytes)]) => Value::Bytes(digest::sha256(bytes).to_vec()),
        (Function::Base64Url, [Value::Bytes(bytes)]) => Value::String(digest::base64url(bytes)),
        (Function::Ascii, [Value::String(text)]) if text.is_ascii() => {
            Value::Bytes(text.as_bytes().to_vec())
        }
        _ => return Err(Unknown),
    }))
}

fn order(a: &Value, b: &Value) -> Option<std::cmp::Ordering> {
    match (a, b) {
        (Value::Integer(a), Value::Integer(b)) | (Value::Duration(a), Value::Duration(b)) => {
            Some(a.cmp(b))
        }
        (Value::Timestamp(a), Value::Timestamp(b)) => Some(a.cmp(b)),
        (Value::String(a), Value::String(b)) => Some(a.as_bytes().cmp(b.as_bytes())),
        _ => a.as_decimal()?.compare(b.as_decimal()?),
    }
}

impl fmt::Display for Expr {
    /// The canonical text: what [`Expr::parse`] reads back to this tree, with no parenthesis the
    /// grammar does not need.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{value}"),
            Self::Decimal(value) => write!(f, "{value}"),
            Self::Duration { seconds, unit } => {
                write!(f, "{}{}", seconds / unit.seconds(), unit.letter())
            }
            Self::Text(text) => {
                f.write_str("\"")?;
                for character in text.chars() {
                    if matches!(character, '"' | '\\') {
                        f.write_str("\\")?;
                    }
                    write!(f, "{character}")?;
                }
                f.write_str("\"")
            }
            Self::Boolean(value) => write!(f, "{value}"),
            Self::Now => f.write_str("now"),
            Self::Read { path, .. } => write!(f, "{path}"),
            Self::Negate(inner) => {
                if inner.precedence() < 3 || matches!(**inner, Self::Negate(_)) {
                    write!(f, "-({inner})")
                } else {
                    write!(f, "-{inner}")
                }
            }
            Self::Arith { op, left, right } => {
                let mine = op.precedence();
                if left.precedence() < mine {
                    write!(f, "({left})")?;
                } else {
                    write!(f, "{left}")?;
                }
                write!(f, " {} ", op.symbol())?;
                if right.precedence() <= mine {
                    write!(f, "({right})")
                } else {
                    write!(f, "{right}")
                }
            }
            Self::Round {
                value,
                digits,
                mode,
            } => write!(f, "round({value}, {digits}, {})", mode.keyword()),
            Self::Call {
                function,
                arguments,
            } => {
                write!(f, "{}(", function.name())?;
                for (position, argument) in arguments.iter().enumerate() {
                    if position > 0 {
                        f.write_str(", ")?;
                    }
                    write!(f, "{argument}")?;
                }
                f.write_str(")")
            }
        }
    }
}

/// One token of the compact form.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Number(String),
    Duration(i64, ElapsedUnit),
    Text(String),
    Word(String),
    Open,
    Close,
    Comma,
    Op(char),
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Number(text) | Self::Word(text) => f.write_str(text),
            Self::Duration(seconds, unit) => {
                write!(f, "{}{}", seconds / unit.seconds(), unit.letter())
            }
            Self::Text(text) => write!(f, "\"{text}\""),
            Self::Open => f.write_str("("),
            Self::Close => f.write_str(")"),
            Self::Comma => f.write_str(","),
            Self::Op(op) => write!(f, "{op}"),
        }
    }
}

fn word_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

#[allow(
    clippy::too_many_lines,
    reason = "one arm per token the compact form spells, each reading its own bytes"
)]
fn lex(text: &str) -> Result<Vec<Token>, ExprError> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        match byte {
            b' ' | b'\t' | b'\n' | b'\r' => at += 1,
            b'(' => {
                tokens.push(Token::Open);
                at += 1;
            }
            b')' => {
                tokens.push(Token::Close);
                at += 1;
            }
            b',' => {
                tokens.push(Token::Comma);
                at += 1;
            }
            b'+' | b'-' | b'*' | b'/' => {
                tokens.push(Token::Op(char::from(byte)));
                at += 1;
            }
            b'"' => {
                let mut value = String::new();
                let mut chars = text[at + 1..].char_indices();
                let mut closed = None;
                while let Some((offset, character)) = chars.next() {
                    match character {
                        '"' => {
                            closed = Some(offset);
                            break;
                        }
                        '\\' => match chars.next() {
                            Some((_, escaped @ ('"' | '\\'))) => value.push(escaped),
                            _ => {
                                return Err(ExprError::new(format!(
                                    "`{text}`: inside a quoted text `\\` escapes only `\"` and `\\`"
                                )))
                            }
                        },
                        other => value.push(other),
                    }
                }
                let Some(offset) = closed else {
                    return Err(ExprError::new(format!(
                        "`{text}`: a quoted text is not closed"
                    )));
                };
                tokens.push(Token::Text(value));
                at += offset + 2;
            }
            b'0'..=b'9' => {
                let start = at;
                while at < bytes.len() && bytes[at].is_ascii_digit() {
                    at += 1;
                }
                if at + 1 < bytes.len() && bytes[at] == b'.' && bytes[at + 1].is_ascii_digit() {
                    at += 1;
                    while at < bytes.len() && bytes[at].is_ascii_digit() {
                        at += 1;
                    }
                    tokens.push(Token::Number(text[start..at].to_owned()));
                } else if at < bytes.len()
                    && matches!(bytes[at], b's' | b'm' | b'h')
                    && !bytes.get(at + 1).copied().is_some_and(word_byte)
                {
                    let written = &text[start..=at];
                    let (seconds, unit) =
                        ElapsedUnit::parse_magnitude(written).ok_or_else(|| {
                            ExprError::new(format!(
                            "`{text}`: `{written}` is no elapsed time; write a whole number of \
                             `s`, `m` or `h` without a leading zero, at most {} seconds",
                            crate::time::CurrentTime::MAX_OFFSET_SECONDS
                        ))
                        })?;
                    tokens.push(Token::Duration(seconds, unit));
                    at += 1;
                } else {
                    if bytes.get(at).copied().is_some_and(word_byte) {
                        return Err(ExprError::new(format!(
                            "`{text}`: `{}` is neither a number nor an elapsed time (`60s`, `5m`, \
                             `1h`)",
                            &text[start..=at]
                        )));
                    }
                    tokens.push(Token::Number(text[start..at].to_owned()));
                }
            }
            byte if word_byte(byte) => {
                let start = at;
                while at < bytes.len()
                    && (word_byte(bytes[at])
                        || (bytes[at] == b'.' && bytes.get(at + 1).copied().is_some_and(word_byte)))
                {
                    at += 1;
                }
                tokens.push(Token::Word(text[start..at].to_owned()));
            }
            _ => {
                let character = text[at..].chars().next().unwrap_or('?');
                return Err(ExprError::new(format!(
                    "`{text}`: `{character}` is no part of an expression; it is written with \
                     paths, literals, `+ - * /`, parentheses and function calls"
                )));
            }
        }
    }
    Ok(tokens)
}

struct Parser<'t> {
    tokens: &'t [Token],
    at: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.at)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.at).cloned();
        self.at += 1;
        token
    }

    fn expect(&mut self, wanted: &Token, context: &str) -> Result<(), ExprError> {
        match self.next() {
            Some(token) if &token == wanted => Ok(()),
            Some(token) => Err(ExprError::new(format!(
                "{context}: expected `{wanted}`, found `{token}`"
            ))),
            None => Err(ExprError::new(format!(
                "{context}: expected `{wanted}`, found the end"
            ))),
        }
    }

    fn expr(&mut self) -> Result<Expr, ExprError> {
        let mut left = self.term()?;
        while let Some(Token::Op(op @ ('+' | '-'))) = self.peek() {
            let op = if *op == '+' {
                ArithOp::Add
            } else {
                ArithOp::Subtract
            };
            self.at += 1;
            let right = self.term()?;
            left = Expr::Arith {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<Expr, ExprError> {
        let mut left = self.unary()?;
        while let Some(Token::Op(op @ ('*' | '/'))) = self.peek() {
            let op = if *op == '*' {
                ArithOp::Multiply
            } else {
                ArithOp::Divide
            };
            self.at += 1;
            let right = self.unary()?;
            left = Expr::Arith {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, ExprError> {
        if self.peek() == Some(&Token::Op('-')) {
            self.at += 1;
            return Ok(Expr::Negate(Box::new(self.unary()?)));
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ExprError> {
        match self.next() {
            Some(Token::Number(text)) => number(&text),
            Some(Token::Duration(seconds, unit)) => Ok(Expr::Duration { seconds, unit }),
            Some(Token::Text(text)) => Ok(Expr::Text(text)),
            Some(Token::Open) => {
                let inner = self.expr()?;
                self.expect(&Token::Close, "a parenthesis")?;
                Ok(inner)
            }
            Some(Token::Word(word)) => {
                if self.peek() == Some(&Token::Open) {
                    self.at += 1;
                    return self.call(&word);
                }
                match word.as_str() {
                    "true" => Ok(Expr::Boolean(true)),
                    "false" => Ok(Expr::Boolean(false)),
                    "now" => Ok(Expr::Now),
                    _ => FactPath::new(&word)
                        .map(|path| Expr::Read { path, kind: None })
                        .map_err(|error| ExprError::new(format!("`{word}`: {error}"))),
                }
            }
            Some(token) => Err(ExprError::new(format!(
                "`{token}` cannot start an operand; an operand is a path, a literal, `now`, a \
                 call or a parenthesised expression"
            ))),
            None => Err(ExprError::new(
                "the expression ends where an operand is expected",
            )),
        }
    }

    fn arguments(&mut self, name: &str) -> Result<Vec<Expr>, ExprError> {
        let mut arguments = Vec::new();
        if self.peek() == Some(&Token::Close) {
            self.at += 1;
            return Ok(arguments);
        }
        loop {
            arguments.push(self.expr()?);
            match self.next() {
                Some(Token::Comma) => {}
                Some(Token::Close) => return Ok(arguments),
                Some(token) => {
                    return Err(ExprError::new(format!(
                        "`{name}(…)`: expected `,` or `)`, found `{token}`"
                    )))
                }
                None => {
                    return Err(ExprError::new(format!("`{name}(…)` is not closed")));
                }
            }
        }
    }

    fn call(&mut self, name: &str) -> Result<Expr, ExprError> {
        if name == "round" {
            let value = self.expr()?;
            self.expect(&Token::Comma, "`round(value, digits, mode)`")?;
            let digits = match self.next() {
                Some(Token::Number(text)) => text
                    .parse::<u8>()
                    .ok()
                    .filter(|digits| *digits <= MAX_ROUND_DIGITS),
                _ => None,
            }
            .ok_or_else(|| {
                ExprError::new(format!(
                    "`round(value, digits, mode)` takes the digits kept as a whole number from 0 \
                     to {MAX_ROUND_DIGITS}"
                ))
            })?;
            self.expect(&Token::Comma, "`round(value, digits, mode)`")?;
            let mode = match self.next() {
                Some(Token::Word(word)) => Rounding::parse(&word),
                _ => None,
            }
            .ok_or_else(|| {
                ExprError::new(
                    "`round(value, digits, mode)` takes the mode `half_even`, `half_up` or `down`",
                )
            })?;
            self.expect(&Token::Close, "`round(value, digits, mode)`")?;
            return Ok(Expr::Round {
                value: Box::new(value),
                digits,
                mode,
            });
        }
        let Some(function) = Function::named(name) else {
            return Err(ExprError::new(format!(
                "`{name}` is no function of the expression language; the functions are `round`, {}",
                Function::ALL
                    .iter()
                    .map(|function| format!("`{}`", function.name()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )));
        };
        let arguments = self.arguments(name)?;
        if arguments.len() != function.arity() {
            return Err(ExprError::new(format!(
                "`{name}` takes {} argument{}, and {} are written",
                function.arity(),
                if function.arity() == 1 { "" } else { "s" },
                arguments.len()
            )));
        }
        Ok(Expr::Call {
            function,
            arguments,
        })
    }
}

fn number(text: &str) -> Result<Expr, ExprError> {
    if text.len() > 1 && text.starts_with('0') && !text.starts_with("0.") {
        return Err(ExprError::new(format!(
            "`{text}` has a leading zero, a second spelling of one number"
        )));
    }
    match text.split_once('.') {
        None => text
            .parse::<i128>()
            .map(Expr::Integer)
            .map_err(|_| ExprError::new(format!("`{text}` is past the 128-bit integer bound"))),
        Some((integer, fraction)) => {
            let units = format!("{integer}{fraction}")
                .parse::<i128>()
                .map_err(|_| ExprError::new(format!("`{text}` has more digits than are kept")))?;
            let scale = u32::try_from(fraction.len()).unwrap_or(u32::MAX);
            Decimal::new(units, scale)
                .map(Expr::Decimal)
                .ok_or_else(|| ExprError::new(format!("`{text}` has more places than are kept")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Evaluated, Expr, ExprType, FactPath, FactValue, Kind, Reading, Value};
    use crate::facts::Number;

    fn parse(text: &str) -> Expr {
        Expr::parse(text).unwrap_or_else(|error| panic!("{text}: {error}"))
    }

    fn with<'a>(facts: &'a [(&'a str, FactValue)]) -> impl Fn(&FactPath) -> Reading + 'a {
        move |path: &FactPath| {
            facts
                .iter()
                .find(|(at, _)| *at == path.to_string())
                .map_or(Reading::Absent, |(_, value)| Reading::Value(value.clone()))
        }
    }

    fn kinds(expr: &Expr, table: &[(&str, Kind)]) -> Expr {
        expr.with_kinds(&|path| {
            table
                .iter()
                .find(|(at, _)| *at == path.to_string())
                .map(|(_, kind)| *kind)
        })
    }

    fn text(value: &str) -> FactValue {
        FactValue::Text(value.to_owned())
    }

    fn decimal(value: &str) -> FactValue {
        FactValue::Number(Number::decimal_literal(value).expect("a decimal"))
    }

    #[test]
    fn the_canonical_text_reads_back_to_the_same_tree() {
        for written in [
            "input.ended_at - input.started_at",
            "a - (b - c)",
            "(a + b) * c",
            "a * b / c",
            "a / (b * c)",
            "-a + 1",
            "-(a + 1)",
            "round(input.total / input.count, 2, half_even)",
            "coalesce(a / b, 0)",
            "base64url(sha256(ascii(input.verifier)))",
            "now - 60s",
            "max(a, 1.5) + 5m",
            "\"a \\\"quoted\\\" text\" + b",
        ] {
            let tree = parse(written);
            assert_eq!(tree.to_string(), written, "{written}");
            assert_eq!(parse(&tree.to_string()), tree);
        }
        assert_eq!(parse("((a)) - ((b))").to_string(), "a - b");
    }

    #[test]
    fn malformed_text_is_refused_by_what_is_wrong() {
        for (written, says) in [
            ("a +", "ends where an operand"),
            ("a b", "follows a complete expression"),
            ("len(a)", "no function"),
            ("round(a, 2)", "`round(value, digits, mode)`: expected `,`"),
            ("round(a, 2, nearest)", "takes the mode"),
            ("abs(a, b)", "takes 1 argument"),
            ("5d", "neither a number"),
            ("007", "leading zero"),
            ("a == b", "no part of an expression"),
        ] {
            let error = Expr::parse(written).expect_err(written);
            assert!(error.message.contains(says), "{written}: {error}");
        }
    }

    #[test]
    fn timestamp_difference_is_a_duration_of_elapsed_seconds() {
        let expr = kinds(
            &parse("ended_at - started_at"),
            &[
                ("ended_at", Kind::Timestamp),
                ("started_at", Kind::Timestamp),
            ],
        );
        let facts = [
            ("started_at", text("2026-01-01T10:00:00Z")),
            ("ended_at", text("2026-01-01T12:30:05+01:00")),
        ];
        assert_eq!(
            expr.evaluate(&with(&facts)).to_fact(),
            Some(text("PT5405S"))
        );
        let reversed = kinds(
            &parse("started_at - ended_at"),
            &[
                ("ended_at", Kind::Timestamp),
                ("started_at", Kind::Timestamp),
            ],
        );
        assert_eq!(
            reversed.evaluate(&with(&facts)),
            Evaluated::Value(Value::Duration(-5405))
        );
        assert_eq!(reversed.evaluate(&with(&facts)).to_fact(), None);
        let typed = expr.infer(&|_| {
            Ok(ExprType {
                kind: Kind::Timestamp,
                optional: false,
            })
        });
        assert_eq!(
            typed,
            Ok(ExprType {
                kind: Kind::Duration,
                optional: false
            })
        );
    }

    #[test]
    fn a_quotient_rounds_half_even_to_six_places_or_as_declared() {
        let facts = [
            ("a", decimal("1")),
            ("b", decimal("8")),
            ("zero", decimal("0")),
        ];
        let value = |written: &str| parse(written).evaluate(&with(&facts)).to_fact();
        assert_eq!(value("a / b"), Some(decimal("0.125")));
        assert_eq!(value("round(a / b, 2, half_even)"), Some(decimal("0.12")));
        assert_eq!(value("round(a / b, 2, half_up)"), Some(decimal("0.13")));
        assert_eq!(value("round(a / b, 2, down)"), Some(decimal("0.12")));
        assert_eq!(value("round(-a / b, 2, half_up)"), Some(decimal("-0.13")));
        assert_eq!(value("2 / 3"), Some(decimal("0.666667")));
        assert_eq!(value("1 / 3"), Some(decimal("0.333333")));
        assert_eq!(value("0.0000005 / 1"), Some(decimal("0")));
        assert_eq!(value("0.0000015 / 1"), Some(decimal("0.000002")));
        assert_eq!(parse("a / zero").evaluate(&with(&facts)), Evaluated::Absent);
        assert_eq!(value("coalesce(a / zero, 0)"), Some(decimal("0")));
        assert_eq!(value("coalesce(a / b, 0)"), Some(decimal("0.125")));
    }

    #[test]
    fn integer_arithmetic_is_exact_and_unknown_past_its_bound() {
        let facts = [("big", FactValue::Number(Number::from(i64::MAX)))];
        let expr = kinds(&parse("big * big * big"), &[("big", Kind::Integer)]);
        assert_eq!(expr.evaluate(&with(&facts)), Evaluated::Unknown);
        let fits = kinds(&parse("big * big"), &[("big", Kind::Integer)]);
        assert!(matches!(
            fits.evaluate(&with(&facts)),
            Evaluated::Value(Value::Integer(_))
        ));
        assert_eq!(fits.evaluate(&with(&facts)).to_fact(), None);
        assert_eq!(
            parse("7 - 10").evaluate(&with(&[])).to_fact(),
            Some(FactValue::Number(Number::from(-3_i64)))
        );
    }

    #[test]
    fn time_moves_by_elapsed_seconds_only() {
        let facts = [("at", text("2026-01-01T00:00:00Z")), ("d", text("PT1H30M"))];
        let expr = kinds(
            &parse("at + d - 5m"),
            &[("at", Kind::Timestamp), ("d", Kind::Duration)],
        );
        assert_eq!(
            expr.evaluate(&with(&facts)).to_fact(),
            Some(text("2026-01-01T01:25:00Z"))
        );
        let days = [("at", text("2026-01-01T00:00:00Z")), ("d", text("P1D"))];
        assert_eq!(expr.evaluate(&with(&days)), Evaluated::Unknown);
        let doubled = kinds(&parse("d * 2 + 1h"), &[("d", Kind::Duration)]);
        assert_eq!(
            doubled.evaluate(&with(&facts)).to_fact(),
            Some(text("PT14400S"))
        );
    }

    #[test]
    fn an_absent_operand_is_absent_until_coalesced() {
        let expr = kinds(&parse("a + 1"), &[("a", Kind::Integer)]);
        assert_eq!(expr.evaluate(&with(&[])), Evaluated::Absent);
        let unknown = |_: &FactPath| Reading::Unknown;
        assert_eq!(expr.evaluate(&unknown), Evaluated::Unknown);
        assert_eq!(
            parse("coalesce(a, 4)").evaluate(&with(&[])).to_fact(),
            Some(FactValue::Number(Number::from(4_i64)))
        );
    }

    #[test]
    fn digests_compose_to_the_rfc_7636_s256_challenge() {
        let expr = kinds(
            &parse("base64url(sha256(ascii(verifier)))"),
            &[("verifier", Kind::String)],
        );
        let facts = [(
            "verifier",
            text("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        )];
        assert_eq!(
            expr.evaluate(&with(&facts)).to_fact(),
            Some(text("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"))
        );
        let wide = [("verifier", text("caf\u{e9}"))];
        assert_eq!(expr.evaluate(&with(&wide)), Evaluated::Unknown);
        let direct = kinds(
            &parse("base64url(sha256(verifier))"),
            &[("verifier", Kind::String)],
        );
        assert_eq!(
            direct.evaluate(&with(&facts)).to_fact(),
            Some(text("E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"))
        );
    }

    #[test]
    fn typing_refuses_by_the_form_and_the_types_it_met() {
        let read = |path: &FactPath| {
            Ok(ExprType {
                kind: match path.to_string().as_str() {
                    "t" => Kind::Timestamp,
                    "s" => Kind::String,
                    "flag" => Kind::Boolean,
                    _ => Kind::Integer,
                },
                optional: path.to_string() == "maybe",
            })
        };
        let error = |written: &str| parse(written).infer(&read).expect_err(written).message;
        assert!(error("t + t").contains("applies `+` to a Timestamp and a Timestamp"));
        assert!(error("base64url(s)").contains("`base64url` takes Bytes"));
        assert!(error("sha256(flag)").contains("`sha256` takes a String"));
        assert!(error("-s").contains("unary `-`"));
        assert_eq!(
            parse("n / maybe").infer(&read),
            Ok(ExprType {
                kind: Kind::Decimal,
                optional: true
            })
        );
        assert_eq!(
            parse("coalesce(maybe, 0)").infer(&read),
            Ok(ExprType {
                kind: Kind::Integer,
                optional: false
            })
        );
        assert_eq!(
            parse("round(n / maybe, 2, down)").infer(&read),
            Ok(ExprType {
                kind: Kind::Decimal,
                optional: true
            })
        );
    }
}
