//! An authored `Decimal` that binary64 cannot hold exactly, refused by name at the literal.
//!
//! `amount: 0.1000000000000000000001` used to validate and compile to `0.1`. Two readers lose the
//! digits before any check could see them:
//!
//! * A **compact guard** (`amount < 0.1000000000000000000001`) is read through
//!   `Number::parse_decimal`, which keeps the authored digits only where they are the canonical
//!   decimal of their binary64. [`ess_primitives::facts`] notes each literal that collapses while a
//!   guard is read ([`collecting_inexact_decimals`]), and [`crate::expression::lexical::Written`]
//!   keeps them with the site that wrote them.
//! * A **YAML number** (a `sets:` or `payload:` literal, and the number of a guard written as a
//!   mapping: `{amount: {lt: 0.1000000000000000000001}}`, `{amount: 0.1…}`, an `in` or `any_of`
//!   list) reaches the domain as an `f64`. [`floats`] reads the spelling of each such number back
//!   out of the text, beside the typed reading, and keeps the ones the `f64` does not hold, by
//!   their path in the document. A predicate site claims the ones under it
//!   ([`crate::expression::lexical::Written::from_document`]); [`authored`] takes the rest that sit
//!   in a `sets:` or `payload:`.
//!
//! Both end in [`refusals`], one [`InexactDecimalLiteral`](ValidationCode::InexactDecimalLiteral)
//! per literal, at the place it was written. Nothing here changes what is written: the exact
//! decimal writer is a separate, coordinated format change.
//!
//! [`collecting_inexact_decimals`]: ess_primitives::facts::collecting_inexact_decimals

use std::collections::{BTreeMap, BTreeSet};

use ess_primitives::error::{
    ConstructKind, ConstructRef, ValidationCode, ValidationError, ValidationErrors,
};
use ess_primitives::facts::InexactDecimal;
use ess_yaml::{Step, Value};

/// The numbers of one document that binary64 does not hold, by their path in it.
pub(crate) type Floats = BTreeMap<Vec<Step>, InexactDecimal>;

/// Whether `text` holds a number that might not be its own binary64.
///
/// Two spellings can be: a run of 16 or more digits (one `.` allowed inside it), which a binary64
/// does not carry, and a digit run with an exponent, which can leave the binary64 range and read
/// as `0` (`1e-400`) whatever its digits. A decimal of at most 15 digits and no exponent is always
/// its own binary64, so a document with neither is skipped without walking it again. A word that
/// only looks like an exponent costs the walk, never a refusal.
fn may_be_inexact(text: &str) -> bool {
    let bytes = text.as_bytes();
    let mut digits = 0_usize;
    for (at, byte) in bytes.iter().enumerate() {
        match byte {
            b'0'..=b'9' => {
                digits += 1;
                if digits >= 16 {
                    return true;
                }
            }
            b'.' => {}
            b'e' | b'E' => {
                let after = match bytes.get(at + 1) {
                    Some(b'+' | b'-') => bytes.get(at + 2),
                    next => next,
                };
                if digits > 0 && after.is_some_and(u8::is_ascii_digit) {
                    return true;
                }
                digits = 0;
            }
            _ => digits = 0,
        }
    }
    false
}

/// Every number of `document` whose digits its `f64` does not hold, by path.
///
/// `text` is the document as written, which is the only place the digits survive.
pub(crate) fn floats(document: &Value, text: &str) -> Floats {
    if !may_be_inexact(text) {
        return Floats::new();
    }
    ess_yaml::float_spellings(text, document)
        .into_iter()
        .filter_map(|(path, spelling)| Some((path, InexactDecimal::of_text(&spelling)?)))
        .collect()
}

/// The numbers of `floats` that sit at or under `path`, with where each is.
pub(crate) fn under<'a>(
    floats: &'a Floats,
    path: &'a [Step],
) -> impl Iterator<Item = (&'a Vec<Step>, &'a InexactDecimal)> {
    floats
        .range(path.to_vec()..)
        .take_while(move |(at, _)| at.starts_with(path))
}

/// Every number of `floats` that a `sets:` or `payload:` of an outcome writes and no predicate site
/// has claimed, with the location it is refused at.
pub(crate) fn authored(
    document: &Value,
    floats: &Floats,
    claimed: &BTreeSet<Vec<Step>>,
) -> Vec<(String, InexactDecimal)> {
    let mut found = Vec::new();
    for (path, number) in floats {
        let [Step::Key(commands), Step::Index(command), Step::Key(outcomes), Step::Index(outcome), Step::Key(place), rest @ ..] =
            path.as_slice()
        else {
            continue;
        };
        if commands != "commands"
            || outcomes != "outcomes"
            || !matches!(place.as_str(), "sets" | "payload")
            || claimed.contains(path)
        {
            continue;
        }
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
        found.push((site.render(), number.clone()));
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

#[cfg(test)]
mod tests {
    use super::may_be_inexact;

    #[test]
    fn a_document_that_cannot_hold_an_inexact_number_is_skipped() {
        for text in [
            "format: ess/35\nsets: {amount: 0.5}\n",
            "a: 123456789012345\nb: 0.12345678901234\n",
            "name: demo.pay.Opened\nsee: e\n",
        ] {
            assert!(!may_be_inexact(text), "{text}");
        }
    }

    #[test]
    fn a_document_that_might_hold_one_is_walked() {
        for text in [
            "a: 0.1000000000000000000001\n",
            "a: 1234567890123456\n",
            "a: 1e-400\n",
            "a: 2.5E+330\n",
            "a: 0.5e-1\n",
            "a: .5e3\n",
        ] {
            assert!(may_be_inexact(text), "{text}");
        }
    }
}
