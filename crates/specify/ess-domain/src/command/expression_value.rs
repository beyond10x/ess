//! An expression value, `{expr: "<expression>"}`, in `sets:`, an event's `payload:` and an error's
//! `payload:` (source format `ess/27`, `story:expression-arithmetic-and-time`, beyond10x/ess#560,
//! `docs/design/generic-language-features.md` § G1).
//!
//! A value position takes `{expr: …}` rather than bare text, because bare text there is already a
//! literal or an input path (open question 3). `expr` is no source keyword, so the mapping is
//! recognised by its exact shape, as `{payload: …}` is; below `ess/27` it is read back as the nested
//! mapping it always was ([`super::minted_value::read_below_ess_27`]), and a refusal of that reading
//! names `ess/27`.
//!
//! The expression reads `input.<path>` — any path a command input resolves — and, on an outcome
//! acting on an existing subject, `subject.<field>` as the row held it before the outcome; the
//! checker of [`crate::expression::value`] types it against the field it fills.

use ess_primitives::error::{ConstructRef, ValidationCode, ValidationError};
use ess_primitives::facts::FactPath;

use super::{CommandSpec, RawPayloadSource};
use crate::expression::value::{self, Checked, Refusal};
use crate::types::{Field, TypeRef, TypeRegistry};

/// The key an expression value is written under.
pub const EXPR_KEY: &str = "expr";

/// `{expr: "<expression>"}`, written as exactly this key over text.
#[derive(serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct RawExprSource {
    /// The expression, in the compact text form.
    expr: String,
}

impl RawExprSource {
    /// The mapping `{expr: <text>}`, and nothing else.
    pub(super) fn recognise(entries: &[(String, RawPayloadSource)]) -> Option<Self> {
        let [(key, RawPayloadSource::Text(text))] = entries else {
            return None;
        };
        (key == EXPR_KEY).then(|| Self { expr: text.clone() })
    }

    /// The written form of an expression value.
    pub(super) fn of(expr: &str) -> Self {
        Self {
            expr: expr.to_owned(),
        }
    }

    /// The source it is.
    pub(super) fn into_source(self) -> super::PayloadSource {
        super::PayloadSource::Expression { expr: self.expr }
    }
}

/// The roots a value of one outcome reads.
pub(super) struct Roots<'a> {
    /// The command whose input `input.` reads.
    pub command: &'a CommandSpec,
    /// The entity `subject.` reads, where the outcome acts on one that exists before it.
    pub subject: Option<&'a crate::entity::EntitySpec>,
    /// The declared types.
    pub types: &'a TypeRegistry,
}

impl Roots<'_> {
    /// The declared type of `path`, or why the outcome reads no such value.
    fn read(&self, path: &FactPath) -> Result<TypeRef, Refusal> {
        let written = path.to_string();
        if let Some(rest) = written.strip_prefix("input.") {
            return super::input_path::resolve(self.command, self.types, rest)
                .map(|resolved| resolved.type_ref())
                .map_err(|_| {
                    Refusal::new(
                        ValidationCode::UndeclaredReference,
                        format!(
                            "`{written}` reads `{rest}`, which `{}` does not declare as input",
                            self.command.name
                        ),
                    )
                    .with_hint(format!(
                        "inputs of `{}`: {}",
                        self.command.name,
                        self.command
                            .input
                            .iter()
                            .map(|field| field.name.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ))
                });
        }
        if let Some(rest) = written.strip_prefix("subject.") {
            let Some(entity) = self.subject else {
                return Err(Refusal::new(
                    ValidationCode::UndeclaredReference,
                    format!(
                        "`{written}` reads the entity an outcome acts on as it was before the \
                         outcome, and this outcome acts on no entity that exists before it"
                    ),
                )
                .with_hint("read the value from the input, `input.<field>`"));
            };
            let held = if entity.identity.name == rest {
                Some(&entity.identity)
            } else {
                entity.field(rest)
            };
            return held.map(|field| field.type_ref.clone()).ok_or_else(|| {
                Refusal::new(
                    ValidationCode::UndeclaredReference,
                    format!(
                        "`{written}` reads `{rest}`, which `{}` does not declare",
                        entity.name
                    ),
                )
                .with_hint(format!(
                    "fields of `{}`: {}",
                    entity.name,
                    std::iter::once(entity.identity.name.as_str())
                        .chain(entity.fields.iter().map(|field| field.name.as_str()))
                        .collect::<Vec<_>>()
                        .join(", ")
                ))
            });
        }
        Err(Refusal::new(
            ValidationCode::UndeclaredReference,
            format!(
                "`{written}` names no value an outcome reads; an expression in `sets:` or \
                 `payload:` reads `input.<field>` and, on an existing subject, `subject.<field>`"
            ),
        )
        .with_hint(format!("write `input.{written}` or `subject.{written}`")))
    }
}

/// Checks `{expr: written}` over `target` at `at`.
pub(super) fn check(
    roots: &Roots<'_>,
    at: &ConstructRef,
    target: &Field,
    written: &str,
) -> Result<Checked, ValidationError> {
    value::check(
        written,
        &|path| roots.read(path),
        roots.types,
        (&target.name, &target.type_ref),
    )
    .map_err(|refusal| refused(at, refusal))
}

/// `{expr: written}` over `target`, for an outcome of `command` whose subject before the outcome is
/// `subject`: the tree with every read's kind, and its type, as the compiler lowers it.
pub fn check_value(
    command: &CommandSpec,
    subject: Option<&crate::entity::EntitySpec>,
    types: &TypeRegistry,
    target: &Field,
    written: &str,
) -> Result<Checked, Refusal> {
    let roots = Roots {
        command,
        subject,
        types,
    };
    value::check(
        written,
        &|path| roots.read(path),
        types,
        (&target.name, &target.type_ref),
    )
}

/// A refusal as the diagnostic at `at`.
pub(crate) fn refused(at: &ConstructRef, refusal: Refusal) -> ValidationError {
    let error = ValidationError::at(at.clone(), refusal.code, refusal.message);
    match refusal.hint {
        Some(hint) => error.with_hint(hint),
        None => error,
    }
}

/// Whether `written` reads a path under `root.`; `false` where it does not parse.
pub(super) fn reads_root(written: &str, root: &str) -> bool {
    ess_primitives::expr::Expr::parse(written).is_ok_and(|expr| {
        expr.reads()
            .iter()
            .any(|path| path.segments().first().map(String::as_str) == Some(root))
    })
}

/// Where an expression value is refused: inside a nested mapping, whose leaves keep the sources
/// they had.
pub(super) fn nested_elsewhere(at: &ConstructRef) -> ValidationError {
    ValidationError::at(
        at.clone(),
        ValidationCode::UnsupportedConstruct,
        "`{expr: …}` fills a whole field; a field inside a nested mapping keeps the sources it had",
    )
    .with_hint("compute the struct's field where the struct is a field of its own")
}
