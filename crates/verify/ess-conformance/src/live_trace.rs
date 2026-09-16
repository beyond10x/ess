//! Closed occurrence-based live assertions. Targets provide complete facts, never verdicts.
use crate::scenario::{EventRef, InstanceName, PayloadShape, ScenarioStep, ScenarioValue};
use crate::temporal::{self, Batch, Ledger, Matcher};
use ess_primitives::facts::Number;
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
    /// Capture one declared field from the named actual occurrence (suite/12+).
    Capture {
        /// Previously observed occurrence.
        anchor: InstanceName,
        /// Unique value binding.
        instance: InstanceName,
        /// Original declared event.
        event: EventRef,
        /// Declared dotted field path.
        path: String,
        /// Original field shape, including optionality.
        shape: PayloadShape,
        /// Explicit presence requirement when the original field is optional.
        require_present: bool,
    },
    /// Derive one exact signed integer from a preceding typed capture (suite/12+).
    Offset {
        /// Unique derived value binding.
        instance: InstanceName,
        /// Preceding integer binding.
        source: InstanceName,
        /// Checked signed addition.
        plus: i64,
    },
    /// Assert bounds on a previously selected occurrence, never as a filter.
    IntegerBounds {
        /// Previously observed occurrence.
        anchor: InstanceName,
        /// Original declared event.
        event: EventRef,
        /// Declared integer payload path.
        path: String,
        /// Original integer field shape.
        shape: PayloadShape,
        /// Inclusive lower bound.
        min: i64,
        /// Inclusive upper bound.
        max: i64,
    },
    /// Name the latest snapshot after a quiet interval fenced at activation.
    Quiet {
        /// Unique name for the selected actual occurrence.
        anchor: InstanceName,
        /// Full quiet interval after activation and the latest scoped receipt.
        duration_ms: u64,
        /// Account or resource scope, independent of the later captured count.
        claim: Claim,
    },
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
    /// Whether this check needs the captured-observation vocabulary.
    pub(crate) fn requires_v12(&self) -> bool {
        matches!(
            self,
            Self::Capture { .. }
                | Self::Offset { .. }
                | Self::IntegerBounds { .. }
                | Self::Quiet { .. }
        )
    }

    /// Events required from the target.
    pub fn events(&self) -> BTreeSet<EventRef> {
        match self {
            Self::Open { events } => events.clone(),
            Self::Await { claim, .. }
            | Self::Absent { claim, .. }
            | Self::Stable { claim, .. }
            | Self::Quiet { claim, .. } => [claim.event.clone()].into(),
            Self::Capture { event, .. } | Self::IntegerBounds { event, .. } => {
                [event.clone()].into()
            }
            Self::Ordered { .. } | Self::Offset { .. } => BTreeSet::new(),
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
    bindings: BTreeMap<InstanceName, Node>,
    quiet: Option<QuietStart>,
}

#[derive(Debug)]
struct QuietStart {
    anchor: InstanceName,
    floor_ms: Option<u64>,
}
impl State {
    /// Begin a subscribed lifetime.
    pub fn new(lifetime: String, events: BTreeSet<EventRef>) -> Result<Self, temporal::Error> {
        Ok(Self {
            ledger: Ledger::new(lifetime.clone())?,
            lifetime,
            anchors: BTreeMap::new(),
            events,
            bindings: BTreeMap::new(),
            quiet: None,
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
        self.ledger.accept(batch)?;
        if let Some(quiet) = self.quiet.as_mut() {
            if quiet.floor_ms.is_none() {
                quiet.floor_ms = Some(self.ledger.complete_before_ms());
            }
        }
        Ok(())
    }
    /// Publish the actual or derived value only after its native check succeeded.
    pub(crate) fn binding(&self, check: &Check) -> Option<(InstanceName, Node)> {
        match check {
            Check::Capture { instance, .. } | Check::Offset { instance, .. } => self
                .bindings
                .get(instance)
                .map(|value| (instance.clone(), value.clone())),
            _ => None,
        }
    }
    fn bind(&mut self, instance: &InstanceName, value: Node) -> Result<(), temporal::Error> {
        if self.bindings.insert(instance.clone(), value).is_some() {
            return Err(temporal::Error::Incomplete("duplicate live capture".into()));
        }
        Ok(())
    }
    fn anchor(&self, name: &InstanceName) -> Result<u64, temporal::Error> {
        self.anchors
            .get(name)
            .copied()
            .ok_or(temporal::Error::UnknownAnchor(0))
    }
    /// Evaluate a check from retained facts; Unfinished asks the evaluator for another batch.
    #[allow(
        clippy::too_many_lines,
        reason = "One exhaustive dispatch keeps all native receipt checks and their failures visible together."
    )]
    pub fn evaluate(
        &mut self,
        check: &Check,
        values: &BTreeMap<String, Node>,
        required: &BTreeMap<String, Node>,
    ) -> Result<(), temporal::Error> {
        match check {
            Check::Capture {
                anchor,
                instance,
                event,
                path,
                shape,
                ..
            } => {
                let item = self.ledger.anchor(self.anchor(anchor)?)?;
                let value = temporal::path_value(&item.payload, path);
                if &item.event != event
                    || value.is_none()
                    || value == Some(&Node::Null)
                    || shape
                        .leaves()
                        .get(path)
                        .is_none_or(|leaf| !leaf.admits(value))
                {
                    return Err(temporal::Error::Counterexample(item.clone()));
                }
                self.bind(instance, value.expect("checked actual field").clone())
            }
            Check::Offset { instance, plus, .. } => {
                let value = values
                    .get("base")
                    .and_then(|value| match value {
                        Node::Number(value) => value.as_i64(),
                        _ => None,
                    })
                    .ok_or_else(|| {
                        temporal::Error::Incomplete(
                            "offset requires a present signed integer".into(),
                        )
                    })?;
                let result = value.checked_add(*plus).ok_or_else(|| {
                    temporal::Error::Incomplete("signed integer offset overflow".into())
                })?;
                self.bind(instance, Node::Number(Number::from(result)))
            }
            Check::IntegerBounds {
                anchor,
                event,
                path,
                min,
                max,
                ..
            } => {
                let item = self.ledger.anchor(self.anchor(anchor)?)?;
                let value =
                    temporal::path_value(&item.payload, path).and_then(|value| match value {
                        Node::Number(value) => value.as_i64(),
                        _ => None,
                    });
                if &item.event != event
                    || !value.is_some_and(|value| *min <= value && value <= *max)
                {
                    return Err(temporal::Error::Counterexample(item.clone()));
                }
                Ok(())
            }
            Check::Quiet {
                anchor,
                duration_ms,
                claim,
            } => {
                if self.quiet.as_ref().map(|quiet| &quiet.anchor) != Some(anchor) {
                    self.quiet = Some(QuietStart {
                        anchor: anchor.clone(),
                        floor_ms: None,
                    });
                }
                let floor = self.quiet.as_ref().and_then(|quiet| quiet.floor_ms).ok_or(
                    temporal::Error::Unfinished {
                        required_ms: 0,
                        observed_ms: 0,
                    },
                )?;
                let item = self.ledger.quiet(
                    &Matcher {
                        event: claim.event.clone(),
                        payload: values.clone(),
                    },
                    floor,
                    *duration_ms,
                )?;
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
#[allow(
    clippy::too_many_lines,
    reason = "Validate the ordered trace vocabulary in one pass over anchors and typed bindings."
)]
pub(crate) fn admit(steps: &[ScenarioStep], major: u64) -> Result<(), String> {
    let mut opened = None;
    let mut anchors = BTreeMap::new();
    let mut captures = BTreeSet::new();
    let mut integer_captures = BTreeSet::new();
    for (index, step) in steps.iter().enumerate() {
        match step {
            ScenarioStep::CaptureResponse {
                instance,
                field,
                shape,
                ..
            } => {
                captures.insert(instance.clone());
                if shape.leaves().get(field).is_some_and(integer_leaf) {
                    integer_captures.insert(instance.clone());
                }
            }
            ScenarioStep::CaptureInstance { instance, .. }
            | ScenarioStep::EstablishEntity { instance, .. } => {
                captures.insert(instance.clone());
            }
            ScenarioStep::CheckLive { trace } => {
                if trace.requires_v12() && major < 12 {
                    return Err(
                        "captured observations and quiet baselines require suite/12 or /13".into(),
                    );
                }
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
                    if anchors.contains_key(name) {
                        Ok(())
                    } else {
                        Err("unknown or forward occurrence anchor".to_owned())
                    }
                };
                match trace {
                    Check::Capture {
                        anchor,
                        instance,
                        event,
                        path,
                        shape,
                        require_present,
                    } => {
                        if anchors.get(anchor) != Some(event) {
                            return Err(
                                "capture event differs from its actual occurrence anchor".into()
                            );
                        }
                        let leaf = single_leaf(shape, path)?;
                        if leaf.optional && !require_present {
                            return Err("optional event capture requires explicit presence".into());
                        }
                        if !captures.insert(instance.clone()) {
                            return Err("duplicate live value capture".into());
                        }
                        if integer_leaf(leaf) {
                            integer_captures.insert(instance.clone());
                        }
                    }
                    Check::Offset {
                        instance, source, ..
                    } => {
                        if !integer_captures.contains(source) {
                            return Err("offset requires a preceding typed integer capture".into());
                        }
                        if !captures.insert(instance.clone()) {
                            return Err("duplicate live value capture".into());
                        }
                        integer_captures.insert(instance.clone());
                    }
                    Check::IntegerBounds {
                        anchor,
                        event,
                        path,
                        shape,
                        min,
                        max,
                    } => {
                        if anchors.get(anchor) != Some(event)
                            || min > max
                            || !integer_leaf(single_leaf(shape, path)?)
                        {
                            return Err("integer bounds require their actual event, integer path and ordered range".into());
                        }
                    }
                    Check::Quiet {
                        anchor,
                        duration_ms,
                        claim,
                    } => {
                        if *duration_ms == 0 {
                            return Err("live quiet window must be positive".into());
                        }
                        admit_values(&claim.matches, &claim.shape, &captures)?;
                        if anchors
                            .insert(anchor.clone(), claim.event.clone())
                            .is_some()
                        {
                            return Err("duplicate occurrence anchor".into());
                        }
                    }
                    Check::Await {
                        anchor,
                        after,
                        claim,
                    } => {
                        if let Some(after) = after {
                            known(after)?;
                        }
                        admit_values(&claim.matches, &claim.shape, &captures)?;
                        if anchors
                            .insert(anchor.clone(), claim.event.clone())
                            .is_some()
                        {
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
pub(crate) fn integer_leaf(leaf: &crate::scenario::LeafShape) -> bool {
    matches!(
        leaf.holds,
        crate::scenario::Holds::Primitive {
            kind: ess_domain::types::Primitive::Integer
        }
    )
}
fn single_leaf<'a>(
    shape: &'a PayloadShape,
    path: &str,
) -> Result<&'a crate::scenario::LeafShape, String> {
    if shape.leaves().len() != 1 || path.is_empty() || path.split('.').any(str::is_empty) {
        return Err("event capture needs exactly one declared payload path".into());
    }
    shape
        .leaves()
        .get(path)
        .ok_or_else(|| "event capture has no declared field shape".into())
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
                );
            }
        }
    }
    Ok(())
}
