//! An expression value checked against the declarations (G1,
//! `docs/design/generic-language-features.md` § G1).
//!
//! The syntax tree, its typing rules and its evaluation are [`ess_primitives::expr`]'s; this is
//! the half that needs the declarations: which declared type each path an expression reads has,
//! what that type is to the expression (a [`Kind`], possibly absent), and whether the expression's
//! type fills the field it is written into. A position supplies the paths it admits as a resolver,
//! so a command outcome reads `input.` and `subject.`, and a view field its source row, through one
//! checker.

use ess_primitives::error::ValidationCode;
use ess_primitives::expr::{Expr, ExprError, ExprType, Kind};
use ess_primitives::facts::FactPath;

use crate::types::{Primitive, TypeRef, TypeRegistry};

/// Why an expression value is refused: the code, the sentence, and the repair where there is one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// The diagnostic's code.
    pub code: ValidationCode,
    /// What is wrong.
    pub message: String,
    /// What to write instead, where something can be.
    pub hint: Option<String>,
}

impl Refusal {
    /// A refusal with no repair.
    pub fn new(code: ValidationCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            hint: None,
        }
    }

    /// The same refusal with `hint` as its repair.
    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// An expression value that passed every check: its tree with each read's [`Kind`] filled, and its
/// type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Checked {
    /// The tree, every read carrying the kind its declaration gives it.
    pub expr: Expr,
    /// What it computes.
    pub typed: ExprType,
}

/// What `reference` is to an expression: its [`Kind`] through every newtype, and whether an
/// `Optional` on the way may leave it absent; `None` for a type no expression computes over.
pub fn kind_of(types: &TypeRegistry, reference: &TypeRef) -> Option<ExprType> {
    let layers = types.newtype_layers(reference);
    let TypeRef::Primitive(primitive) = layers.terminal else {
        return None;
    };
    let kind = match primitive {
        Primitive::Integer => Kind::Integer,
        Primitive::Decimal => Kind::Decimal,
        Primitive::Timestamp => Kind::Timestamp,
        Primitive::Duration => Kind::Duration,
        Primitive::String => Kind::String,
        Primitive::Bytes => Kind::Bytes,
        Primitive::Boolean => Kind::Boolean,
        Primitive::Binary64 | Primitive::Uuid | Primitive::Json => return None,
    };
    Some(ExprType {
        kind,
        optional: layers.optional,
    })
}

/// Why a path's declared type is no operand.
fn no_operand(path: &FactPath, declared: &TypeRef) -> Refusal {
    Refusal::new(
        ValidationCode::TypeMismatch,
        format!(
            "`{path}` is `{declared}`, and an expression computes over Integer, Decimal, \
             Timestamp, Duration, String, Bytes and Boolean values; a Binary64 takes part in \
             comparisons only, and an enum, a Uuid, a struct or a collection in none"
        ),
    )
    .with_hint(format!(
        "copy the value with its own source instead, such as `{path}` written without `{{expr: …}}`"
    ))
}

/// Parses `written`, types every path it reads by `read`, and holds its type to `target`, the
/// declared type of the field it fills, named `target_name`.
///
/// `read` answers the declared type of a path the position admits, or the refusal of one it does
/// not. `now` is refused: a value read at the current time has no clock a suite fixes.
pub fn check(
    written: &str,
    read: &dyn Fn(&FactPath) -> Result<TypeRef, Refusal>,
    types: &TypeRegistry,
    (target_name, target): (&str, &TypeRef),
) -> Result<Checked, Refusal> {
    let parsed = Expr::parse(written).map_err(|error| {
        Refusal::new(ValidationCode::UnparsablePredicate, error.message).with_hint(
            "an expression is written with paths, literals (`1`, `2.5`, `\"text\"`, `60s`), `+ - \
             * /`, parentheses and the functions `round`, `abs`, `min`, `max`, `coalesce`, \
             `sha256`, `base64url` and `ascii`",
        )
    })?;
    if parsed.reads_now() {
        return Err(Refusal::new(
            ValidationCode::UnsupportedConstruct,
            format!(
                "`{written}` reads `now` as a value; a value read at the current time has no clock \
                 a suite can fix yet, so no scenario could check it"
            ),
        )
        .with_hint(
            "send the instant as an input and compute from it, such as `input.at - \
             input.started_at`; `now` orders a Timestamp in a command's guards",
        ));
    }
    let mut refused: Option<Refusal> = None;
    let mut kinds: Vec<(FactPath, Kind)> = Vec::new();
    for path in parsed.reads() {
        match read(path) {
            Ok(declared) => match kind_of(types, &declared) {
                Some(operand) => kinds.push((path.clone(), operand.kind)),
                None => {
                    refused.get_or_insert_with(|| no_operand(path, &declared));
                }
            },
            Err(refusal) => {
                refused.get_or_insert(refusal);
            }
        }
    }
    if let Some(refusal) = refused {
        return Err(refusal);
    }
    let computed = parsed
        .infer(&|path| {
            read(path)
                .ok()
                .and_then(|declared| kind_of(types, &declared))
                .ok_or_else(|| ExprError {
                    message: format!("`{path}` has no type"),
                })
        })
        .map_err(|error| Refusal::new(ValidationCode::TypeMismatch, error.message))?;
    let Some(wanted) = kind_of(types, target) else {
        return Err(Refusal::new(
            ValidationCode::TypeMismatch,
            format!(
                "`{target_name}` is `{target}`, and an expression computes an Integer, a Decimal, a \
                 Timestamp, a Duration, a String, Bytes or a Boolean"
            ),
        ));
    };
    if computed.kind != wanted.kind {
        return Err(Refusal::new(
            ValidationCode::TypeMismatch,
            format!(
                "`{written}` computes {} `{}`, and `{target_name}` is `{target}`",
                article(computed.kind),
                computed.kind
            ),
        )
        .with_hint(match (computed.kind, wanted.kind) {
            (Kind::Duration, Kind::Integer | Kind::Decimal) => {
                "a difference of two instants is a Duration; declare the field `Duration`"
                    .to_owned()
            }
            _ => format!("declare `{target_name}` as {}", computed.kind),
        }));
    }
    if computed.optional && !wanted.optional {
        return Err(Refusal::new(
            ValidationCode::TypeMismatch,
            format!(
                "`{written}` is `{computed}`: a quotient is absent where its divisor is zero, and an \
                 Optional read where it is absent; `{target_name}` is `{target}`, which is always \
                 present"
            ),
        )
        .with_hint(format!(
            "say what stands in for no value, such as `coalesce({written}, 0)`, or declare \
             `{target_name}` Optional"
        )));
    }
    let expr = parsed.with_kinds(&|path| {
        kinds
            .iter()
            .find(|(read, _)| read == path)
            .map(|(_, kind)| *kind)
    });
    Ok(Checked {
        expr,
        typed: computed,
    })
}

fn article(kind: Kind) -> &'static str {
    match kind {
        Kind::Integer => "an",
        _ => "a",
    }
}
