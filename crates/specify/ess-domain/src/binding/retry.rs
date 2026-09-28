//! A bounded `retry`: how many attempts a binding makes, and which refusals end it at once
//! (ess/16, beyond10x/ess#165).
//!
//! `docs/design/binding-delivery-guarantees.md`, "A bound that is the component's behaviour", is
//! the binding design.
//!
//! ```yaml
//! on_failure:
//!   retry: {attempts: 3, final: [demo.ledger.Unknown]}
//! ```
//!
//! `on_failure: retry` written bare keeps its meaning — try again, on whatever schedule the
//! transport provides, with no count — and its bytes. The block is for the case where the count is
//! a constant in the sender's code rather than a deployment decision, which a receiver can observe:
//! it sees exactly `attempts` invocations when every one fails, and exactly one when the failure is
//! one of the `final` refusals. After the last attempt the event's effect is lost, as under `drop`.
//!
//! The model has outcomes, not transport status codes, so `final` names **refusals of the invoked
//! command**: an outcome that carries an `error:`, by its name, or the error itself, which stands
//! for every outcome of the command that reports it. Any other failure is retried up to the bound.
//!
//! | rule | code |
//! |---|---|
//! | the bound is written under a format older than `ess/16` | [`UnsupportedFormatVersion`](ValidationCode::UnsupportedFormatVersion) |
//! | fewer than two attempts | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration): one attempt is `drop` |
//! | a `final` name that is no refusal of the invoked command | [`UndeclaredReference`](ValidationCode::UndeclaredReference) |
//! | one `final` name written twice | [`DuplicateDeclaration`](ValidationCode::DuplicateDeclaration) |
//! | a bound beside a policy that is not `retry` | [`ConflictingDeclaration`](ValidationCode::ConflictingDeclaration): only reachable in code |

use std::collections::BTreeSet;

use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

use crate::binding::{BindingSpec, Failure};
use crate::command::CommandSpec;
use crate::system::FormatVersion;

/// How many attempts a `retry` makes, and which refusals end it at once.
///
/// Read from a document as the block under `retry:`, and serialized the same way, so a binding
/// that states no bound serializes exactly as it did before the block existed. The reader is
/// written by hand so that every refusal names `retry:` — `retry: {emits: …}` is the escalation
/// block under the wrong word, and a message naming only `emits` would not say so.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetryBound {
    /// Invocations in all, the first included. At least two: one attempt is `drop`.
    // `min` is [`Self::MIN_ATTEMPTS`]; the attribute takes a literal only.
    #[schemars(range(min = 2))]
    pub attempts: u32,
    /// Refusals of the invoked command that end the retry at once: an outcome name, or an error
    /// the command reports. Each name at most once.
    #[serde(default, rename = "final", skip_serializing_if = "Vec::is_empty")]
    #[schemars(schema_with = "unique_names")]
    pub finals: Vec<String>,
}

/// `final`'s schema: the list of strings a `Vec<String>` projects, with the uniqueness the reader
/// refuses a repeated name for (`DuplicateDeclaration`).
fn unique_names(generator: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    let mut schema = generator.subschema_for::<Vec<String>>().into_object();
    schema.array().unique_items = Some(true);
    schema.into()
}

impl RetryBound {
    /// The fewest attempts a bound may state.
    pub const MIN_ATTEMPTS: u32 = 2;

    /// Whether `name` — an outcome name or an error — names one of `command`'s refusals.
    pub fn names_refusal(command: &CommandSpec, name: &str) -> bool {
        command.outcomes.iter().any(|outcome| {
            outcome
                .error
                .as_ref()
                .is_some_and(|error| outcome.name.to_string() == name || error.to_string() == name)
        })
    }

    /// The outcomes of `command` this bound makes final, in the command's declaration order.
    pub fn final_outcomes<'a>(
        &self,
        command: &'a CommandSpec,
    ) -> Vec<&'a crate::command::OutcomeName> {
        command
            .outcomes
            .iter()
            .filter(|outcome| {
                outcome.error.as_ref().is_some_and(|error| {
                    self.finals
                        .iter()
                        .any(|name| outcome.name.to_string() == *name || error.to_string() == *name)
                })
            })
            .map(|outcome| &outcome.name)
            .collect()
    }
}

impl<'de> serde::Deserialize<'de> for RetryBound {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Block;

        impl<'de> serde::de::Visitor<'de> for Block {
            type Value = RetryBound;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a `retry:` block with `attempts:` and, optionally, `final:`")
            }

            fn visit_map<A: serde::de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> Result<Self::Value, A::Error> {
                let mut attempts = None;
                let mut finals = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "attempts" if attempts.is_none() => attempts = Some(map.next_value()?),
                        "final" if finals.is_none() => finals = Some(map.next_value()?),
                        "attempts" | "final" => {
                            return Err(serde::de::Error::custom(format!(
                                "`retry:` says `{key}` twice"
                            )))
                        }
                        other => {
                            return Err(serde::de::Error::custom(format!(
                                "`retry:` takes `attempts:` and `final:`, and `{other}` is \
                                 neither; only `escalate` publishes anything, and no timing is \
                                 a claim here"
                            )))
                        }
                    }
                }
                let Some(attempts) = attempts else {
                    return Err(serde::de::Error::custom(
                        "`retry:` with a block states `attempts:`, the invocations in all; write \
                         `on_failure: retry` for a retry with no bound",
                    ));
                };
                Ok(RetryBound {
                    attempts,
                    finals: finals.unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_map(Block)
    }
}

/// Everything a bound can be wrong about on its own binding: its attempts, a repeated `final`
/// name, and a bound beside another policy.
pub(crate) fn check_shape(binding: &BindingSpec) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    let Some(bound) = &binding.retry else {
        return errors;
    };
    let at = format!("binding.{}.on_failure", binding.name);
    if binding.failure != Failure::Retry {
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                at.clone(),
                format!(
                    "binding `{}` fails with `{}` and also states a retry bound; only `retry` \
                     makes attempts",
                    binding.name, binding.failure
                ),
            )
            .with_hint(
                "a document cannot write this — the bound is the block under `retry:` — so this \
                 binding was assembled in code; drop the bound, or make the policy `retry`",
            ),
        );
    }
    if bound.attempts < RetryBound::MIN_ATTEMPTS {
        errors.push(
            ValidationError::new(
                ValidationCode::ConflictingDeclaration,
                format!("{at}.retry.attempts"),
                format!(
                    "binding `{}` retries with `attempts: {}`, which makes no second attempt",
                    binding.name, bound.attempts
                ),
            )
            .with_hint(
                "`attempts` counts invocations including the first; one attempt whose failure is \
                 lost is `on_failure: drop`, so write that, or state at least 2",
            ),
        );
    }
    let mut seen = BTreeSet::new();
    for name in &bound.finals {
        if !seen.insert(name.as_str()) {
            errors.push(
                ValidationError::new(
                    ValidationCode::DuplicateDeclaration,
                    format!("{at}.retry.final"),
                    format!("`{name}` is written twice under `final:`"),
                )
                .with_hint("name each refusal that ends the retry once"),
            );
        }
    }
    errors
}

/// The format gate and each `final` name against the invoked command, for every binding of the
/// specification, including one assembled in code.
pub fn validate_specification(spec: &crate::Specification) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for binding in spec.bindings().values() {
        let Some(bound) = &binding.retry else {
            continue;
        };
        let at = format!("binding.{}.on_failure.retry", binding.name);
        if spec.system().format.major() < FormatVersion::V16.major() {
            errors.push(
                ValidationError::new(
                    ValidationCode::UnsupportedFormatVersion,
                    at.clone(),
                    "a bounded retry requires specification format ess/16",
                )
                .with_hint(
                    "write `format: ess/16` on the source that declares the system, or write \
                     `on_failure: retry` with no block",
                ),
            );
        }
        // A command nobody declares is reported once, by the binding's own validation.
        let Some(command) = spec.commands().get(&binding.command) else {
            continue;
        };
        for name in &bound.finals {
            if RetryBound::names_refusal(command, name) {
                continue;
            }
            let refusals: Vec<String> = command
                .outcomes
                .iter()
                .filter_map(|outcome| {
                    outcome
                        .error
                        .as_ref()
                        .map(|error| format!("`{}` ({error})", outcome.name))
                })
                .collect();
            let hint = if refusals.is_empty() {
                format!(
                    "`{}` declares no outcome with `error:`, so nothing it answers can end the \
                     retry; drop `final:`",
                    command.name
                )
            } else {
                format!(
                    "name an outcome of `{}` that carries `error:`, or its error: {}",
                    command.name,
                    refusals.join(", ")
                )
            };
            errors.push(
                ValidationError::new(
                    ValidationCode::UndeclaredReference,
                    format!("{at}.final"),
                    format!(
                        "`{name}` under `final:` is no refusal of `{}`, the command `{}` invokes",
                        command.name, binding.name
                    ),
                )
                .with_hint(hint),
            );
        }
    }
    errors
}
