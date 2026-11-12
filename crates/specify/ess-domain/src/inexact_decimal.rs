//! An authored `Decimal` that binary64 cannot hold exactly, refused by name at the literal.
//!
//! `amount: 0.1000000000000000000001` used to validate and compile to `0.1`. Two readers lose the
//! digits before any check could see them:
//!
//! * A **guard** is read through `Number::parse_decimal`, which keeps the authored digits only
//!   where they are the canonical decimal of their binary64. [`ess_primitives::facts`] notes each
//!   literal that collapses while a guard is read ([`collecting_inexact_decimals`]), and
//!   [`crate::expression::lexical::Written`] keeps them with the site that wrote them.
//! * A **`sets:` or `payload:` literal** is a YAML number, which reaches the domain as an `f64`.
//!   [`authored`] reads the spelling of each such number back out of the text, beside the typed
//!   reading, and keeps the ones the `f64` does not hold.
//!
//! Both end in [`refusals`], one [`InexactDecimalLiteral`](ValidationCode::InexactDecimalLiteral)
//! per literal, at the place it was written. Nothing here changes what is written: the exact
//! decimal writer is a separate, coordinated format change.
//!
//! [`collecting_inexact_decimals`]: ess_primitives::facts::collecting_inexact_decimals

use ess_primitives::error::{
    ConstructKind, ConstructRef, ValidationCode, ValidationError, ValidationErrors,
};
use ess_primitives::facts::InexactDecimal;
use ess_yaml::{Step, Value};

/// Whether `text` holds a run of digits (one `.` allowed inside it) long enough that a number
/// might not be its own binary64: a decimal of at most 15 significant digits always is.
fn may_be_inexact(text: &str) -> bool {
    let mut digits = 0_usize;
    for character in text.chars() {
        match character {
            '0'..='9' => {
                digits += 1;
                if digits >= 16 {
                    return true;
                }
            }
            '.' => {}
            _ => digits = 0,
        }
    }
    false
}

/// Every number a `sets:` or `payload:` of an outcome of `document` writes whose digits its
/// `f64` does not hold, with the location it is refused at.
///
/// `text` is the document as written, which is the only place the digits survive.
pub(crate) fn authored(document: &Value, text: &str) -> Vec<(String, InexactDecimal)> {
    if !may_be_inexact(text) {
        return Vec::new();
    }
    let mut found = Vec::new();
    for (path, spelling) in ess_yaml::float_spellings(text, document) {
        let [Step::Key(commands), Step::Index(command), Step::Key(outcomes), Step::Index(outcome), Step::Key(place), rest @ ..] =
            path.as_slice()
        else {
            continue;
        };
        if commands != "commands"
            || outcomes != "outcomes"
            || !matches!(place.as_str(), "sets" | "payload")
        {
            continue;
        }
        let Some(number) = InexactDecimal::of_text(&spelling) else {
            continue;
        };
        let name = |value: Option<&Value>| value?.get("name")?.as_str().map(str::to_owned);
        let command = document
            .get("commands")
            .and_then(|commands| commands.get(*command));
        let (Some(command_name), Some(outcome_name)) = (
            name(command),
            name(command.and_then(|command| command.get("outcomes")?.get(*outcome))),
        ) else {
            continue;
        };
        let site = rest.iter().fold(
            ConstructRef::new(ConstructKind::Command, command_name)
                .key("outcomes")
                .named(outcome_name)
                .key(place.as_str()),
            |site, step| match step {
                Step::Key(key) => site.named(key.as_str()),
                Step::Index(index) => site.index(*index),
            },
        );
        found.push((site.render(), number));
    }
    found
}

/// One refusal per literal in `found`, each at the location it was written.
pub(crate) fn refusals(found: &[(String, InexactDecimal)]) -> ValidationErrors {
    let mut errors = ValidationErrors::new();
    for (location, number) in found {
        errors.push(
            ValidationError::new(
                ValidationCode::InexactDecimalLiteral,
                location.clone(),
                format!(
                    "`{}` is a decimal binary64 cannot hold exactly; every reader would take it \
                     as its nearest binary64 value `{}`",
                    number.spelled, number.nearest
                ),
            )
            .with_hint(format!(
                "write the value you mean with the digits binary64 holds, such as `{}`",
                number.nearest
            )),
        );
    }
    errors
}
