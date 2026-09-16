//! Occurrence identity and complete observation intervals for live conformance.
//!
//! Targets report append-only facts on one scenario-local monotonic clock. They do not
//! report whether an assertion passed. A watermark is exclusive: every occurrence before
//! it has been delivered. Disconnects, overflow and decoding failures invalidate the
//! lifetime; reconnecting cannot repair its history. These primitives do not extend the
//! persisted suite vocabulary by themselves.

use crate::scenario::EventRef;
use ess_primitives::node::Node;
use std::collections::BTreeMap;

/// One observed occurrence. Equal event names and payloads still have distinct identities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    /// Contiguous ordinal across all sources in this observation lifetime, starting at one.
    pub sequence: u64,
    /// Milliseconds on the shared scenario-local monotonic clock.
    pub at_ms: u64,
    /// Declared event observed through its real source.
    pub event: EventRef,
    /// Actual payload, copied at observation time.
    pub payload: BTreeMap<String, Node>,
}

/// An adapter's complete extension of one observation lifetime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Batch {
    /// Exact lifetime returned when observation began, before stimulus.
    pub lifetime: String,
    /// Last ordinal delivered by the preceding batch, or zero initially.
    pub after: u64,
    /// Every occurrence strictly before this time is present in the ledger.
    pub complete_before_ms: u64,
    /// Newly observed facts in contiguous ordinal and monotonic time order.
    pub occurrences: Vec<Occurrence>,
}

/// A payload matcher compares only explicitly named fields, preserving absent versus null.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Matcher {
    /// Declared event name.
    pub event: EventRef,
    /// Required equal field values, resolved from literals or actual captured identities.
    pub payload: BTreeMap<String, Node>,
}

impl Matcher {
    /// Whether this occurrence has the required event and every required payload value.
    pub fn matches(&self, occurrence: &Occurrence) -> bool {
        self.event == occurrence.event
            && self
                .payload
                .iter()
                .all(|(field, value)| path_value(&occurrence.payload, field) == Some(value))
    }
}

/// Read a declared dotted field path from an actual observed payload.
pub fn path_value<'a>(payload: &'a BTreeMap<String, Node>, path: &str) -> Option<&'a Node> {
    let mut parts = path.split('.');
    let mut value = payload.get(parts.next()?)?;
    for part in parts {
        let Node::Map(fields) = value else {
            return None;
        };
        value = fields.get(part)?;
    }
    Some(value)
}

fn overlapping_paths(left: &str, right: &str) -> bool {
    left == right
        || left
            .strip_prefix(right)
            .is_some_and(|suffix| suffix.starts_with('.'))
        || right
            .strip_prefix(left)
            .is_some_and(|suffix| suffix.starts_with('.'))
}

/// A refused observation or a refuted temporal claim. Neither is a passing interval.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The lifetime is not complete and cannot acquire a receipt from later good batches.
    Incomplete(String),
    /// The requested anchor does not belong to this ledger.
    UnknownAnchor(u64),
    /// A bounded assertion must claim a positive representable interval.
    InvalidWindow,
    /// A stability claim must constrain at least one payload field.
    EmptyClaim,
    /// A field being checked cannot also filter which snapshots are inspected.
    FilteredClaim,
    /// The target has not yet completed the requested window.
    Unfinished {
        /// Required exclusive watermark.
        required_ms: u64,
        /// Actual exclusive watermark.
        observed_ms: u64,
    },
    /// The real occurrence that contradicted the assertion.
    Counterexample(Occurrence),
    /// The named occurrences are not in the requested strict order.
    Order {
        /// Required earlier occurrence.
        before: u64,
        /// Required later occurrence.
        after: u64,
    },
}

/// A scenario's immutable occurrence history after native admission of each batch.
#[derive(Debug)]
pub struct Ledger {
    lifetime: String,
    occurrences: Vec<Occurrence>,
    complete_before_ms: u64,
    broken: Option<String>,
}

impl Ledger {
    /// Begin one lifetime. Reusing a ledger across reconnects or scenarios is forbidden.
    pub fn new(lifetime: String) -> Result<Self, Error> {
        if lifetime.trim().is_empty() {
            return Err(Error::Incomplete("empty observation lifetime".into()));
        }
        Ok(Self {
            lifetime,
            occurrences: Vec::new(),
            complete_before_ms: 0,
            broken: None,
        })
    }

    /// Record a source failure permanently, including loss not visible in an ordinal gap.
    pub fn invalidate(&mut self, reason: impl Into<String>) -> Error {
        let reason = self.broken.get_or_insert_with(|| reason.into());
        Error::Incomplete(reason.clone())
    }

    fn healthy(&self) -> Result<(), Error> {
        match &self.broken {
            Some(reason) => Err(Error::Incomplete(reason.clone())),
            None => Ok(()),
        }
    }

    /// Admit a complete append. A malformed batch contributes no partial facts and poisons
    /// the lifetime, so a later buffered good frame cannot conceal the gap.
    pub fn accept(&mut self, batch: Batch) -> Result<(), Error> {
        self.healthy()?;
        if let Err(reason) = self.validate(&batch) {
            return Err(self.invalidate(reason));
        }
        self.complete_before_ms = batch.complete_before_ms;
        self.occurrences.extend(batch.occurrences);
        Ok(())
    }

    fn validate(&self, batch: &Batch) -> Result<(), String> {
        if batch.lifetime != self.lifetime {
            return Err("observation lifetime changed".into());
        }
        if batch.after != self.cursor() {
            return Err("observation cursor is not the preceding receipt".into());
        }
        if batch.complete_before_ms < self.complete_before_ms {
            return Err("observation watermark regressed".into());
        }
        let mut previous = self.cursor();
        let mut at = self.complete_before_ms;
        for occurrence in &batch.occurrences {
            if previous.checked_add(1) != Some(occurrence.sequence) {
                return Err("missing or duplicate occurrence ordinal".into());
            }
            if occurrence.at_ms < at || occurrence.at_ms >= batch.complete_before_ms {
                return Err("occurrence falls outside the complete monotonic interval".into());
            }
            at = occurrence.at_ms;
            previous = occurrence.sequence;
        }
        Ok(())
    }

    /// Last accepted occurrence ordinal, for the next observation request.
    pub fn cursor(&self) -> u64 {
        self.occurrences.last().map_or(0, |item| item.sequence)
    }

    /// Current exclusive watermark, used to fence a newly activated quiet interval.
    pub fn complete_before_ms(&self) -> u64 {
        self.complete_before_ms
    }

    /// Latest scoped snapshot, after a full quiet interval beginning no earlier than
    /// the fresh activation fence. A newer scoped snapshot restarts the interval.
    pub fn quiet(
        &self,
        matcher: &Matcher,
        floor_ms: u64,
        duration_ms: u64,
    ) -> Result<&Occurrence, Error> {
        self.healthy()?;
        if duration_ms == 0 {
            return Err(Error::InvalidWindow);
        }
        let item = self
            .occurrences
            .iter()
            .rev()
            .find(|item| matcher.matches(item))
            .ok_or(Error::Unfinished {
                required_ms: 0,
                observed_ms: self.complete_before_ms,
            })?;
        let end = item
            .at_ms
            .max(floor_ms)
            .checked_add(duration_ms)
            .ok_or(Error::InvalidWindow)?;
        self.complete(end)?;
        Ok(item)
    }

    /// The first matching occurrence strictly after an earlier occurrence, or after
    /// subscription when `after` is zero. Payload matching never borrows another call's event.
    pub fn find(&self, matcher: &Matcher, after: u64) -> Result<Option<&Occurrence>, Error> {
        self.healthy()?;
        if after != 0 {
            self.anchor(after)?;
        }
        Ok(self
            .occurrences
            .iter()
            .find(|item| item.sequence > after && matcher.matches(item)))
    }

    /// Resolve an actual occurrence; an alias or command completion is not an anchor.
    pub fn anchor(&self, sequence: u64) -> Result<&Occurrence, Error> {
        self.healthy()?;
        self.occurrences
            .iter()
            .find(|item| item.sequence == sequence)
            .ok_or(Error::UnknownAnchor(sequence))
    }

    /// Assert strict observation order, including distinct occurrences in one clock tick.
    pub fn ordered(&self, before: u64, after: u64) -> Result<(), Error> {
        self.anchor(before)?;
        self.anchor(after)?;
        if before >= after {
            return Err(Error::Order { before, after });
        }
        Ok(())
    }

    fn end(&self, anchor: u64, duration_ms: u64) -> Result<u64, Error> {
        let start = self.anchor(anchor)?;
        if duration_ms == 0 {
            return Err(Error::InvalidWindow);
        }
        start
            .at_ms
            .checked_add(duration_ms)
            .ok_or(Error::InvalidWindow)
    }

    fn complete(&self, required_ms: u64) -> Result<(), Error> {
        self.healthy()?;
        if self.complete_before_ms < required_ms {
            return Err(Error::Unfinished {
                required_ms,
                observed_ms: self.complete_before_ms,
            });
        }
        Ok(())
    }

    /// Require no matching occurrence after the anchor and before its exclusive end.
    /// An offer exactly at wrap-up expiry is allowed. An early counterexample refutes
    /// immediately; success still requires the whole interval to be observed.
    pub fn absent(&self, matcher: &Matcher, anchor: u64, duration_ms: u64) -> Result<(), Error> {
        let end = self.end(anchor, duration_ms)?;
        if let Some(item) = self
            .occurrences
            .iter()
            .find(|item| item.sequence > anchor && item.at_ms < end && matcher.matches(item))
        {
            return Err(Error::Counterexample(item.clone()));
        }
        self.complete(end)
    }

    /// Require the baseline and every scoped occurrence before the exclusive end to
    /// satisfy the required payload. `scope` selects the account/queue stream; `required`
    /// adds the claimed metric values. A transient zero cannot hide between endpoints.
    pub fn stable(
        &self,
        scope: &Matcher,
        required: &BTreeMap<String, Node>,
        anchor: u64,
        duration_ms: u64,
    ) -> Result<(), Error> {
        let baseline = self.anchor(anchor)?;
        let end = self.end(anchor, duration_ms)?;
        if required.is_empty() {
            return Err(Error::EmptyClaim);
        }
        if required.keys().any(|field| {
            scope
                .payload
                .keys()
                .any(|scoped| overlapping_paths(field, scoped))
        }) {
            return Err(Error::FilteredClaim);
        }
        if !scope.matches(baseline) {
            return Err(Error::Counterexample(baseline.clone()));
        }
        for item in self
            .occurrences
            .iter()
            .filter(|item| item.sequence >= anchor && item.at_ms < end && scope.matches(item))
        {
            if required
                .iter()
                .any(|(field, value)| path_value(&item.payload, field) != Some(value))
            {
                return Err(Error::Counterexample(item.clone()));
            }
        }
        self.complete(end)
    }
}
