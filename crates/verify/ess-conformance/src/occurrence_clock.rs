//! One observed decision instant per command occurrence (beyond10x/ess#244 part a;
//! `docs/design/expression-family-source22.md`, "One observed decision instant per command
//! occurrence").
//!
//! A command edge freezes one UTC instant immediately before it decides the outcome against the
//! pre-outcome snapshot, and every predicate of that decision reads that one value. The instant
//! belongs to the invocation: not to the scenario, the arrangement, a row or a retry origin. A
//! retained answer delivered again makes no decision and carries no instant.
//!
//! [`DecisionInstant`] is that value. It wraps the validated RFC 3339 instant a `Timestamp` names,
//! at full precision, and has one spelling: UTC with `Z`, and a fraction of one to nine digits only
//! when it is not zero, with no trailing zero. Nothing constructs one from arbitrary text or from a
//! monotonic counter without that validation. `tests/fixtures/history2/decision-time-vectors.json`
//! is the spelling, shared with the Go and TypeScript writers.
//!
//! [`CommandClock`] is the caller-owned provider a native target reads at its decision edge,
//! exactly once per executed decision. This crate implements none: the default target holds no
//! provider and reads no host clock.

use std::fmt;

use ess_primitives::time::Rfc3339Instant;
use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize, Serializer};

/// The instant one command decision observed, at full precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecisionInstant(Rfc3339Instant);

/// Why text is not a [`DecisionInstant`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecisionInstantRefusal {
    /// The text refused.
    pub text: String,
}

impl fmt::Display for DecisionInstantRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "`{}` is not a decision instant: an RFC 3339 instant in UTC with `Z`, and a fraction \
             only when it is not zero, with no trailing zero",
            self.text
        )
    }
}

impl std::error::Error for DecisionInstantRefusal {}

impl DecisionInstant {
    /// Admits exactly the one spelling of an instant.
    ///
    /// # Errors
    ///
    /// [`DecisionInstantRefusal`] for text that is not an RFC 3339 instant, or names one in another
    /// spelling — an offset other than `Z`, a lower-case separator, a trailing zero in the fraction.
    pub fn parse(text: &str) -> Result<Self, DecisionInstantRefusal> {
        Rfc3339Instant::parse_rfc3339(text)
            .filter(|instant| instant.to_rfc3339() == text)
            .map(Self)
            .ok_or_else(|| DecisionInstantRefusal {
                text: text.to_owned(),
            })
    }

    /// The instant a clock read, already validated as an instant.
    ///
    /// Any instant is held, so a receipt keeps exactly what its decision read. One outside the years
    /// an RFC 3339 `date-time` spells (0000 through 9999) has no spelling: [`Self::is_spelled`] is
    /// `false`, and writing it is refused ([`Serialize`]), so no recorder writes an instant its
    /// reader refuses. [`Self::checked`] refuses it at construction.
    #[must_use]
    pub fn from_instant(instant: Rfc3339Instant) -> Self {
        Self(instant)
    }

    /// The instant a clock read, or `None` outside the years an RFC 3339 `date-time` spells.
    #[must_use]
    pub fn checked(instant: Rfc3339Instant) -> Option<Self> {
        Some(Self(instant)).filter(|decided| decided.is_spelled())
    }

    /// Whether the instant has its one spelling: it lies in the years 0000 through 9999, so its
    /// spelling reads back as the same instant.
    #[must_use]
    pub fn is_spelled(self) -> bool {
        Rfc3339Instant::parse_rfc3339(&self.0.to_rfc3339()) == Some(self.0)
    }

    /// The instant, as the predicate evaluator reads `now`.
    #[must_use]
    pub fn instant(self) -> Rfc3339Instant {
        self.0
    }

    /// The one spelling.
    #[must_use]
    pub fn to_rfc3339(self) -> String {
        self.0.to_rfc3339()
    }
}

impl fmt::Display for DecisionInstant {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.to_rfc3339())
    }
}

impl Serialize for DecisionInstant {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if !self.is_spelled() {
            return Err(serde::ser::Error::custom(format!(
                "the decision instant `{self}` lies outside the years 0000 through 9999 an RFC 3339 \
                 instant spells, and is not written"
            )));
        }
        serializer.serialize_str(&self.to_rfc3339())
    }
}

impl<'de> Deserialize<'de> for DecisionInstant {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text).map_err(de::Error::custom)
    }
}

/// A caller-owned command clock: read once at a command's decision edge.
///
/// `None` is a clock with no reading to give, which leaves every decision that needs one Unknown;
/// it is never replaced by another clock.
pub trait CommandClock {
    /// The instant of the decision being made now.
    fn read(&self) -> Option<DecisionInstant>;
}

impl<T: CommandClock + ?Sized> CommandClock for std::rc::Rc<T> {
    fn read(&self) -> Option<DecisionInstant> {
        (**self).read()
    }
}

impl<T: CommandClock + ?Sized> CommandClock for std::sync::Arc<T> {
    fn read(&self) -> Option<DecisionInstant> {
        (**self).read()
    }
}

impl<T: CommandClock + ?Sized> CommandClock for Box<T> {
    fn read(&self) -> Option<DecisionInstant> {
        (**self).read()
    }
}
