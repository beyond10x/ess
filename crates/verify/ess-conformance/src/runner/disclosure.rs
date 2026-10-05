//! Private observations: never serialized, rendered, or used to construct diagnostics.
use crate::{
    one_time_response::{Trace, MAX_CAPTURES, MAX_CAPTURE_BYTES},
    scenario::{CommandRef, OutcomeRef},
    target::SemanticCommandResult,
};
use ess_primitives::node::Node;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy)]
pub(super) enum Violation {
    Disclosure,
    Payload,
    Resource,
}

pub(super) struct Captures {
    policy: Trace,
    values: Vec<String>,
    bytes: usize,
    observed: BTreeSet<OutcomeRef>,
}

impl Captures {
    pub(super) fn new(policy: Trace) -> Self {
        Self {
            policy,
            values: Vec::new(),
            bytes: 0,
            observed: BTreeSet::new(),
        }
    }

    pub(super) fn command(
        &mut self,
        command: &CommandRef,
        result: &SemanticCommandResult,
    ) -> Result<(), Violation> {
        // Check every response against previous values before authorizing new positions.
        if let Some(response) = &result.response {
            self.map(response)?;
        }
        let origin = self.policy.origins.iter().find(|origin| {
            &origin.command == command && Some(&origin.outcome) == result.outcome.as_ref()
        });
        if let Some(origin) = origin {
            if result.error.is_some() {
                return Err(Violation::Payload);
            }
            let shape = crate::direct_response::Observation {
                command: origin.command.clone(),
                outcome: Some(origin.outcome.clone()),
                fields: origin.response.fields.clone(),
                declarations: origin.response.declarations.clone(),
                expected: BTreeMap::new(),
            };
            shape
                .compare(result.response.as_ref())
                .map_err(|_| Violation::Payload)?;
            let response = result.response.as_ref().ok_or(Violation::Payload)?;
            for field in &origin.response.fields {
                if let Some(value) = response.get(&field.name) {
                    constraints(&origin.response, &field.type_ref, value)?;
                }
            }
            let mut added = Vec::new();
            for field in &origin.fields {
                let Some(Node::Text(value)) = response.get(field) else {
                    return Err(Violation::Payload);
                };
                if value.is_empty() {
                    return Err(Violation::Payload);
                }
                if added
                    .iter()
                    .any(|other: &String| other.contains(value) || value.contains(other))
                {
                    return Err(Violation::Disclosure);
                }
                added.push(value.clone());
            }
            let added_bytes = added.iter().map(String::len).sum::<usize>();
            if self.values.len().saturating_add(added.len()) > MAX_CAPTURES
                || self.bytes.saturating_add(added_bytes) > MAX_CAPTURE_BYTES
            {
                return Err(Violation::Resource);
            }
            // Same-invocation copies outside the exact field are already disclosures.
            for (name, value) in response {
                scan_text(name, &added)?;
                if !origin.fields.contains(name) {
                    scan(value, &added)?;
                }
            }
            self.observed.insert(origin.outcome.clone());
            self.bytes += added_bytes;
            self.values.extend(added);
        }
        self.maps(result.direct_events.iter().map(|event| &event.payload))?;
        if let Some(error) = &result.error {
            self.map(&error.fields)?;
        }
        Ok(())
    }

    pub(super) fn map(&self, value: &BTreeMap<String, Node>) -> Result<(), Violation> {
        Budget::default().map(value, 0)?;
        encoded(value)?;
        scan_map(value, &self.values)
    }

    pub(super) fn maps<'a>(
        &self,
        values: impl IntoIterator<Item = &'a BTreeMap<String, Node>>,
    ) -> Result<(), Violation> {
        let values: Vec<_> = values.into_iter().take(65_537).collect();
        let mut budget = Budget::default();
        budget.add(values.len())?;
        for value in &values {
            budget.map(value, 1)?;
        }
        encoded(&values)?;
        for value in values {
            scan_map(value, &self.values)?;
        }
        Ok(())
    }

    pub(super) fn complete(&self) -> bool {
        self.policy
            .required_origins
            .iter()
            .all(|origin| self.observed.contains(origin))
    }
}

fn constraints(
    authority: &crate::one_time_response::Response,
    ty: &ess_domain::TypeRef,
    value: &Node,
) -> Result<(), Violation> {
    use crate::selection::Declaration;
    use ess_domain::TypeRef;
    match (ty, value) {
        (TypeRef::Named(name), _) => {
            if let Some(rules) = authority.constraints.get(name) {
                let Node::Text(text) = value else {
                    return Err(Violation::Payload);
                };
                if rules.alphabet.as_ref().is_some_and(|alphabet| {
                    text.chars().any(|character| !alphabet.contains(character))
                }) || rules
                    .prefix
                    .as_ref()
                    .is_some_and(|prefix| !text.starts_with(prefix))
                {
                    return Err(Violation::Payload);
                }
                let mut facts = ess_primitives::facts::FactStore::new();
                facts.set(
                    ess_primitives::facts::FactPath::new("value")
                        .map_err(|_| Violation::Payload)?,
                    ess_primitives::facts::FactValue::Text(text.clone()),
                );
                for predicate in &rules.invariants {
                    match predicate.evaluate(&facts) {
                        ess_primitives::predicate::Truth::True => {}
                        ess_primitives::predicate::Truth::False => return Err(Violation::Payload),
                        ess_primitives::predicate::Truth::Unknown => {
                            return Err(Violation::Resource)
                        }
                    }
                }
            }
            match authority.declarations.get(name).ok_or(Violation::Payload)? {
                Declaration::Newtype { of } => constraints(authority, of, value)?,
                Declaration::Struct { fields } => {
                    let Node::Map(values) = value else {
                        return Err(Violation::Payload);
                    };
                    for field in fields {
                        if let Some(value) = values.get(&field.name) {
                            constraints(authority, &field.type_ref, value)?;
                        }
                    }
                }
                Declaration::Union { tag, variants } => {
                    let Node::Map(values) = value else {
                        return Err(Violation::Payload);
                    };
                    let Some(Node::Text(label)) = values.get(tag) else {
                        return Err(Violation::Payload);
                    };
                    let ty = variants.get(label).ok_or(Violation::Payload)?;
                    match (ty, values.get(ess_gen::schema::union_content_key(tag))) {
                        (Some(ty), Some(value)) => constraints(authority, ty, value)?,
                        // A unit variant (ess/22) is the tag alone.
                        (None, Some(_)) => return Err(Violation::Payload),
                        (_, None) => {}
                    }
                }
                Declaration::Enum { .. } => {}
            }
        }
        (TypeRef::Optional(_), Node::Null) => {}
        (TypeRef::Optional(of), _) => constraints(authority, of, value)?,
        (TypeRef::List(of), Node::Seq(values)) => {
            for value in values {
                constraints(authority, of, value)?;
            }
        }
        (TypeRef::Map(_, of), Node::Map(values)) => {
            for value in values.values() {
                constraints(authority, of, value)?;
            }
        }
        _ => {}
    }
    Ok(())
}

impl<C: super::Clock> super::Runner<C> {
    pub(super) fn disclosure_final<T: crate::target::ConformanceTarget>(
        &mut self,
        scenario: &crate::scenario::ConformanceScenario,
        run: &mut super::Run,
        target: &T,
    ) {
        let Some(policy) = &scenario.one_time_response else {
            return;
        };
        if run.disclosure_stopped {
            return;
        }
        for event in &policy.events {
            let result = target.observe_events(crate::target::EventObservationRequest {
                event: event.event.clone(),
                correlation: run.context.correlation.clone(),
                deadline: crate::target::Deadline::at(self.clock.now()),
            });
            match result {
                Ok(observed) => run.remember(&observed),
                Err(error) => {
                    run.record(super::target_failure(
                        &run.id,
                        "final disclosure observation",
                        &error,
                    ));
                    return;
                }
            }
            if run.disclosure_stopped {
                return;
            }
        }
    }

    pub(super) fn disclosure_windows<T: crate::target::ConformanceTarget>(
        &mut self,
        scenario: &crate::scenario::ConformanceScenario,
        index: usize,
        run: &mut super::Run,
        target: &T,
    ) -> super::Flow {
        let Some(policy) = &scenario.one_time_response else {
            return super::Flow::Continue;
        };
        let start = self.clock.now();
        for window in policy
            .event_windows
            .iter()
            .filter(|window| window.after_step == index)
        {
            let instant = match mark_window(scenario, window, run, target) {
                Ok(instant) => instant,
                Err(error) => {
                    run.record(super::target_failure(
                        &run.id,
                        "opening disclosure window",
                        &error,
                    ));
                    return super::Flow::Stop;
                }
            };
            let deadline =
                crate::target::Deadline::at(ess_primitives::time::Timestamp::from_epoch_millis(
                    start.epoch_millis().saturating_add(window.within_ms),
                ));
            let mut final_scan = window.within_ms == 0;
            for count in 0..65_536 {
                if final_scan {
                    if let Some(instant) = &instant {
                        if let Err(error) = close_window(instant, window, run, target) {
                            run.record(super::target_failure(
                                &run.id,
                                "closing disclosure window",
                                &error,
                            ));
                            return super::Flow::Stop;
                        }
                    }
                }
                let observed = match target.observe_events(crate::target::EventObservationRequest {
                    event: window.event.clone(),
                    correlation: run.context.correlation.clone(),
                    deadline,
                }) {
                    Ok(observed) => observed,
                    Err(error) => {
                        run.record(super::target_failure(
                            &run.id,
                            "independent disclosure event observation",
                            &error,
                        ));
                        return super::Flow::Stop;
                    }
                };
                run.remember(&observed);
                if run.disclosure_stopped {
                    return super::Flow::Stop;
                }
                if final_scan {
                    break;
                }
                if count == 65_535 {
                    run.disclosure_violation(Violation::Resource);
                    return super::Flow::Stop;
                }
                // An early harmless answer is not a completed observation window.
                final_scan = deadline.has_passed(self.clock.now());
            }
        }
        super::Flow::Continue
    }
}

fn mark_window<T: crate::target::ConformanceTarget>(
    scenario: &crate::scenario::ConformanceScenario,
    window: &crate::one_time_response::EventWindow,
    run: &super::Run,
    target: &T,
) -> Result<Option<crate::scenario::InstantName>, crate::target::TargetError> {
    if window.within_ms == 0 {
        return Ok(None);
    }
    for serial in 0..65_536 {
        let name: crate::scenario::InstantName =
            format!("one-time-window-{}-{serial}", window.after_step)
                .parse()
                .map_err(|_| {
                    crate::target::TargetError::unsupported(
                        "disclosure window",
                        "instant cannot be represented",
                    )
                })?;
        if scenario.steps.iter().any(
            |step| matches!(step, crate::ScenarioStep::MarkInstant { instant } if instant == &name),
        ) {
            continue;
        }
        target.mark_instant(crate::target::InstantMark {
            instant: name.clone(),
            correlation: run.context.correlation.clone(),
        })?;
        return Ok(Some(name));
    }
    Err(crate::target::TargetError::unsupported(
        "disclosure window",
        "no disjoint instant name within resource bound",
    ))
}

fn close_window<T: crate::target::ConformanceTarget>(
    instant: &crate::scenario::InstantName,
    window: &crate::one_time_response::EventWindow,
    run: &super::Run,
    target: &T,
) -> Result<(), crate::target::TargetError> {
    // The existing hold API has whole-second resolution. Rounding upward observes at
    // least the declared finite window; it never silently shortens it.
    let seconds = window.within_ms.div_ceil(1000).try_into().map_err(|_| {
        crate::target::TargetError::unsupported(
            "disclosure window",
            "duration exceeds elapsed observation authority",
        )
    })?;
    let observed = target.observe_elapsed(crate::target::ElapsedObservationRequest {
        instant: instant.clone(),
        hold: crate::scenario::Elapsed::seconds(seconds),
        watching: Some(window.event.clone()),
        correlation: run.context.correlation.clone(),
    })?;
    if observed.elapsed_ms < window.within_ms {
        return Err(crate::target::TargetError::unsupported(
            "disclosure window",
            "target did not establish complete observation window",
        ));
    }
    Ok(())
}

#[derive(Default)]
struct Budget {
    members: usize,
}
impl Budget {
    fn add(&mut self, members: usize) -> Result<(), Violation> {
        self.members = self.members.saturating_add(members);
        if self.members > 65_536 {
            Err(Violation::Resource)
        } else {
            Ok(())
        }
    }
    fn map(&mut self, values: &BTreeMap<String, Node>, depth: usize) -> Result<(), Violation> {
        if depth > 128 {
            return Err(Violation::Resource);
        }
        self.add(values.len())?;
        for value in values.values() {
            self.value(value, depth + 1)?;
        }
        Ok(())
    }
    fn value(&mut self, value: &Node, depth: usize) -> Result<(), Violation> {
        if depth > 128 {
            return Err(Violation::Resource);
        }
        match value {
            Node::Map(values) => self.map(values, depth)?,
            Node::Seq(values) => {
                self.add(values.len())?;
                for value in values {
                    self.value(value, depth + 1)?;
                }
            }
            _ => {}
        }
        Ok(())
    }
}

// Traversal bounds are established before serialization; the writer never retains payload bytes.
fn encoded(value: &impl serde::Serialize) -> Result<(), Violation> {
    struct Counter(usize);
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0 = self.0.saturating_add(bytes.len());
            if self.0 > 1_048_576 {
                return Err(std::io::Error::other("payload resource bound"));
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    serde_json::to_writer(Counter(0), value).map_err(|_| Violation::Resource)
}

fn scan_text(text: &str, captured: &[String]) -> Result<(), Violation> {
    if captured.iter().any(|value| text.contains(value)) {
        Err(Violation::Disclosure)
    } else {
        Ok(())
    }
}

fn scan_map(values: &BTreeMap<String, Node>, captured: &[String]) -> Result<(), Violation> {
    for (key, value) in values {
        scan_text(key, captured)?;
        scan(value, captured)?;
    }
    Ok(())
}

fn scan(value: &Node, captured: &[String]) -> Result<(), Violation> {
    match value {
        Node::Text(text) => {
            scan_text(text, captured)?;
        }
        Node::Seq(values) => {
            for value in values {
                scan(value, captured)?;
            }
        }
        Node::Map(values) => {
            scan_map(values, captured)?;
        }
        Node::Null | Node::Bool(_) | Node::Number(_) => {}
    }
    Ok(())
}
