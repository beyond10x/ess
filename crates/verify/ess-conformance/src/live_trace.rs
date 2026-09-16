//! Closed occurrence-based live assertions. Targets provide complete facts, never verdicts.
use crate::scenario::{EventRef, InstanceName, PayloadShape, ScenarioStep, ScenarioValue};
use crate::temporal::{self, Batch, Ledger, Matcher};
use ess_primitives::node::Node;
use std::collections::{BTreeMap, BTreeSet};

/// A typed payload-path claim resolved against an original event declaration.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    /// Original qualified event.
    pub event: EventRef,
    /// Literals and earlier actual captures at declared paths.
    pub matches: BTreeMap<String, ScenarioValue>,
    /// Original leaf types for those paths.
    pub shape: PayloadShape,
}

/// Suite/10 and /11 live checks; these versions are the unreleased live vocabulary.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Check {
    /// Subscribe before any setup or stimulus.
    Open {
        /// Fixed complete set of source events for this scenario.
        events: BTreeSet<EventRef>,
    },
    /// Bind the first real matching occurrence after a preceding occurrence, if named.
    Await {
        /// Unique occurrence name, separate from response variables.
        anchor: InstanceName,
        /// Earlier occurrence, or subscription start.
        after: Option<InstanceName>,
        /// Payload assertion.
        claim: Claim,
    },
    /// No matching occurrence in the entire anchored interval.
    Absent {
        /// Earlier observed occurrence.
        anchor: InstanceName,
        /// Positive window length in milliseconds, exclusive at its end.
        duration_ms: u64,
        /// Forbidden event payload.
        claim: Claim,
    },
    /// Baseline and every scoped snapshot must agree throughout an interval.
    Stable {
        /// Earlier observed baseline.
        anchor: InstanceName,
        /// Positive window length in milliseconds.
        duration_ms: u64,
        /// Which account or resource snapshots belong to the claim.
        claim: Claim,
        /// Values checked on every scoped snapshot; cannot overlap the scope filter.
        required: BTreeMap<String, ScenarioValue>,
        /// Original types of asserted fields.
        shape: PayloadShape,
    },
    /// Require two distinct actual occurrences in strict order.
    Ordered {
        /// Earlier occurrence.
        before: InstanceName,
        /// Later occurrence.
        after: InstanceName,
    },
}

impl Check {
    /// Events required from the target.
    pub fn events(&self) -> BTreeSet<EventRef> {
        match self {
            Self::Open { events } => events.clone(),
            Self::Await { claim, .. } | Self::Absent { claim, .. } | Self::Stable { claim, .. } => {
                [claim.event.clone()].into()
            }
            Self::Ordered { .. } => BTreeSet::new(),
        }
    }
}

/// Request for an append-only complete extension. Zero `through_ms` asks for new facts.
#[derive(Debug, Clone)]
pub struct Request {
    /// Exact subscribed lifetime.
    pub lifetime: String,
    /// Last accepted ordinal.
    pub after: u64,
    /// Minimum exclusive watermark required by a bounded assertion.
    pub through_ms: u64,
}

/// Native execution state, never accepted from the target or host report.
#[derive(Debug)]
pub struct State {
    lifetime: String,
    ledger: Ledger,
    anchors: BTreeMap<InstanceName, u64>,
    events: BTreeSet<EventRef>,
}
impl State {
    /// Begin a subscribed lifetime.
    pub fn new(lifetime: String, events: BTreeSet<EventRef>) -> Result<Self, temporal::Error> {
        Ok(Self {
            ledger: Ledger::new(lifetime.clone())?,
            lifetime,
            anchors: BTreeMap::new(),
            events,
        })
    }
    /// Native cursor, not an adapter-selected starting point.
    pub fn request(&self, through_ms: u64) -> Request {
        Request {
            lifetime: self.lifetime.clone(),
            after: self.ledger.cursor(),
            through_ms,
        }
    }
    /// Admit every fact in one batch; unexpected events are not silently dropped.
    pub fn accept(&mut self, batch: Batch) -> Result<(), temporal::Error> {
        if batch
            .occurrences
            .iter()
            .any(|item| !self.events.contains(&item.event))
        {
            return Err(self
                .ledger
                .invalidate("unsubscribed event in observation batch"));
        }
        self.ledger.accept(batch)
    }
    fn anchor(&self, name: &InstanceName) -> Result<u64, temporal::Error> {
        self.anchors
            .get(name)
            .copied()
            .ok_or(temporal::Error::UnknownAnchor(0))
    }
    /// Evaluate a check from retained facts; Unfinished asks the evaluator for another batch.
    pub fn evaluate(
        &mut self,
        check: &Check,
        values: &BTreeMap<String, Node>,
        required: &BTreeMap<String, Node>,
    ) -> Result<(), temporal::Error> {
        match check {
            Check::Open { .. } => Err(temporal::Error::Incomplete("lifetime already open".into())),
            Check::Await {
                anchor,
                after,
                claim,
            } => {
                let after = after
                    .as_ref()
                    .map(|name| self.anchor(name))
                    .transpose()?
                    .unwrap_or(0);
                let matcher = Matcher {
                    event: claim.event.clone(),
                    payload: values.clone(),
                };
                let item =
                    self.ledger
                        .find(&matcher, after)?
                        .ok_or(temporal::Error::Unfinished {
                            required_ms: 0,
                            observed_ms: 0,
                        })?;
                if claim
                    .shape
                    .leaves()
                    .iter()
                    .any(|(path, leaf)| !leaf.admits(temporal::path_value(&item.payload, path)))
                {
                    return Err(temporal::Error::Counterexample(item.clone()));
                }
                self.anchors.insert(anchor.clone(), item.sequence);
                Ok(())
            }
            Check::Absent {
                anchor,
                duration_ms,
                claim,
            } => self.ledger.absent(
                &Matcher {
                    event: claim.event.clone(),
                    payload: values.clone(),
                },
                self.anchor(anchor)?,
                *duration_ms,
            ),
            Check::Stable {
                anchor,
                duration_ms,
                claim,
                ..
            } => self.ledger.stable(
                &Matcher {
                    event: claim.event.clone(),
                    payload: values.clone(),
                },
                required,
                self.anchor(anchor)?,
                *duration_ms,
            ),
            Check::Ordered { before, after } => self
                .ledger
                .ordered(self.anchor(before)?, self.anchor(after)?),
        }
    }
}

/// Reject malformed sequences before any target operation.
pub(crate) fn admit(steps: &[ScenarioStep], major: u64) -> Result<(), String> {
    let mut opened = None;
    let mut anchors = BTreeSet::new();
    let mut captures = BTreeSet::new();
    for (index, step) in steps.iter().enumerate() {
        match step {
            ScenarioStep::CaptureResponse { instance, .. }
            | ScenarioStep::CaptureInstance { instance, .. }
            | ScenarioStep::EstablishEntity { instance, .. } => {
                captures.insert(instance.clone());
            }
            ScenarioStep::CheckLive { trace } => {
                if major < 10 {
                    return Err("live occurrence checks require suite/10 or /11".into());
                }
                if let Check::Open { events } = trace {
                    if index != 0 || events.is_empty() {
                        return Err(
                            "one nonempty observation subscription must be the first step".into(),
                        );
                    }
                    opened = Some(events);
                    continue;
                }
                let events = opened.ok_or("live assertion before subscription")?;
                if !trace.events().is_subset(events) {
                    return Err("assertion event was not subscribed".into());
                }
                let known = |name: &InstanceName| {
                    if anchors.contains(name) {
                        Ok(())
                    } else {
                        Err("unknown or forward occurrence anchor".to_owned())
                    }
                };
                match trace {
                    Check::Await {
                        anchor,
                        after,
                        claim,
                    } => {
                        if let Some(after) = after {
                            known(after)?;
                        }
                        admit_values(&claim.matches, &claim.shape, &captures)?;
                        if !anchors.insert(anchor.clone()) {
                            return Err("duplicate occurrence anchor".into());
                        }
                    }
                    Check::Absent {
                        anchor,
                        duration_ms,
                        claim,
                    }
                    | Check::Stable {
                        anchor,
                        duration_ms,
                        claim,
                        ..
                    } => {
                        known(anchor)?;
                        if *duration_ms == 0 {
                            return Err("live window must be positive".into());
                        }
                        admit_values(&claim.matches, &claim.shape, &captures)?;
                        if let Check::Stable {
                            required, shape, ..
                        } = trace
                        {
                            admit_values(required, shape, &captures)?;
                            for a in required.keys() {
                                for b in claim.matches.keys() {
                                    if a == b
                                        || a.starts_with(&format!("{b}."))
                                        || b.starts_with(&format!("{a}."))
                                    {
                                        return Err(
                                            "stability claim overlaps its scope filter".into()
                                        );
                                    }
                                }
                            }
                        }
                    }
                    Check::Ordered { before, after } => {
                        known(before)?;
                        known(after)?;
                    }
                    Check::Open { .. } => unreachable!(),
                }
            }
            _ => {}
        }
    }
    Ok(())
}
fn admit_values(
    values: &BTreeMap<String, ScenarioValue>,
    shape: &PayloadShape,
    captures: &BTreeSet<InstanceName>,
) -> Result<(), String> {
    if values.is_empty() {
        return Err("live claim must constrain a declared payload path".into());
    }
    for (path, value) in values {
        let leaf = shape
            .leaves()
            .get(path)
            .ok_or("live claim path has no declared type")?;
        match value {
            ScenarioValue::Literal { value } if leaf.admits(Some(value)) => {}
            ScenarioValue::Instance { instance } if captures.contains(instance) => {}
            _ => {
                return Err(
                    "live claim requires a typed literal or an earlier response capture".into(),
                )
            }
        }
    }
    Ok(())
}
