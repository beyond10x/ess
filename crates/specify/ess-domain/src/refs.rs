//! Where a construct came from: the records outside this model that explain it.
//!
//! A specification says what the system does and gives an author nowhere to say *why this exists*.
//! `Conversion.because` is the only prose field in the whole model, and every other construct is
//! `deny_unknown_fields`, so the answer went into YAML comments — which no projection reads, no
//! validation checks, and no generated page carries.
//!
//! # A reference, not a paragraph
//!
//! This is deliberately not a `because:` on everything. A sentence explaining a binding is a
//! sentence that goes stale beside a ticket that does not, and the thing a reader actually wants is
//! the conversation: the ticket, the incident, the decision record. So a construct names the record
//! and the record keeps the prose.
//!
//! The spelling is `provider:key` — `jira:DEV-630`, `zendesk:204519` — which is AEP's spelling for
//! the same idea on a planning artifact, resolved through the same `providers:` map in
//! `.engineering/project.yaml`. One vocabulary across both tools, so a repository that records a
//! ticket on a story and on the binding that story produced writes it the same way twice.
//!
//! # The URL is not here
//!
//! A reference is `jira:DEV-630` and never `https://…/DEV-630`. A model that carried the host would
//! have to be rewritten when a tracker moves, in every construct that named one, and the copies are
//! exactly what is wrong after a migration. A projection that wants a link builds one from the
//! project's map; a provider with no entry renders as text, because a link that opens the wrong
//! page cannot be told from a right one by looking at it.

use std::fmt;
use std::str::FromStr;

use ess_primitives::error::ParseError;

/// One record outside this model, named as `provider:key`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, schemars::JsonSchema)]
#[schemars(with = "String")]
pub struct ExternalRef {
    /// The system holding the record, such as `jira`.
    pub provider: String,
    /// Its key in that system, such as `DEV-630`.
    pub reference: String,
}

impl ExternalRef {
    /// Parses `provider:key`.
    pub fn parse(value: &str) -> Result<Self, ParseError> {
        let reject =
            |reason: String| Err(ParseError::identifier("external reference", value, reason));

        let Some((provider, reference)) = value.split_once(':') else {
            return reject("must be written `provider:key`, such as `jira:DEV-630`".to_owned());
        };
        if provider.is_empty() || reference.is_empty() {
            return reject("has an empty provider or key".to_owned());
        }
        // The provider is a key into a map somebody else writes, so it is held to the shape of a
        // name rather than left free: `Jira ` and `jira` looking up differently is a class of bug
        // that only shows as a missing link.
        for character in provider.chars() {
            if !(character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-') {
                return reject(format!(
                    "has a provider containing {character:?}; a provider is lower-case letters, \
                     digits and hyphens"
                ));
            }
        }
        if reference.trim() != reference {
            return reject("has a key with leading or trailing whitespace".to_owned());
        }
        // The one mistake worth naming. `https://tracker.example/DEV-630` splits into a provider
        // `https` — lower-case letters, so otherwise legal — and a key that is the rest of a URL,
        // which would become a provider nobody declared and a link nobody can build. It is also
        // exactly what somebody reaches for first.
        if reference.starts_with("//") {
            return reject(
                "looks like a URL. A reference is `provider:key`, and the address is built from \
                 the project's `providers:` map — a model that carried the host would have to be \
                 rewritten in every construct when the tracker moves"
                    .to_owned(),
            );
        }
        Ok(Self {
            provider: provider.to_owned(),
            reference: reference.to_owned(),
        })
    }
}

impl fmt::Display for ExternalRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.provider, self.reference)
    }
}

impl FromStr for ExternalRef {
    type Err = ParseError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl serde::Serialize for ExternalRef {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for ExternalRef {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let written = String::deserialize(deserializer)?;
        Self::parse(&written).map_err(serde::de::Error::custom)
    }
}

/// The records a construct names, in the order they were written.
///
/// Order is the author's, not sorted: the first reference is usually the one that caused the thing
/// to exist and the rest are what happened to it afterwards, which is a reading order rather than
/// an alphabet.
pub type Refs = Vec<ExternalRef>;

/// Whether a construct names nothing, for output suppression.
pub fn is_empty(refs: &Refs) -> bool {
    refs.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reference_is_a_provider_and_a_key_and_round_trips() {
        let parsed = ExternalRef::parse("jira:DEV-630").expect("valid");
        assert_eq!(parsed.provider, "jira");
        assert_eq!(parsed.reference, "DEV-630");
        assert_eq!(parsed.to_string(), "jira:DEV-630");
    }

    #[test]
    fn a_key_may_hold_a_colon_because_only_the_first_one_separates() {
        let parsed = ExternalRef::parse("wiki:space:Runbooks/ACD").expect("valid");
        assert_eq!(parsed.provider, "wiki");
        assert_eq!(
            parsed.reference, "space:Runbooks/ACD",
            "a key is opaque and belongs to the provider, not to this parser"
        );
    }

    #[test]
    fn a_url_is_refused_where_a_reference_belongs() {
        // The one mistake worth naming: a URL parses as `https` plus a key, which would silently
        // become a provider nobody declared and a link nobody can build.
        let refusal = ExternalRef::parse("https://tracker.example/DEV-630")
            .expect_err("refused")
            .to_string();
        assert!(
            refusal.contains("looks like a URL"),
            "`https` is lower-case letters and would otherwise pass as a provider: {refusal}"
        );
    }

    #[test]
    fn a_provider_that_is_not_a_key_into_a_map_is_refused() {
        for bad in [
            "Jira:DEV-630",
            "jira :DEV-630",
            ":DEV-630",
            "jira:",
            "DEV-630",
        ] {
            assert!(ExternalRef::parse(bad).is_err(), "`{bad}` should not parse");
        }
    }
}
