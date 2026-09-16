//! Admission for actual response captures and payload-path event matchers.
//! New semantics require suite 10 (ordinary) or 11 (coverage); legacy meanings stay fixed.
use crate::admission::AdmissionError;
use crate::scenario::{ConformanceSuite, ScenarioStep, ScenarioValue};
use std::collections::BTreeSet;

/// Whether fresh compilation needs the capture/matcher vocabulary.
pub fn used_by(suite: &ConformanceSuite) -> bool {
    suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .any(|step| {
            matches!(
                step,
                ScenarioStep::CaptureResponse { .. } | ScenarioStep::EventuallyMatchingEvent { .. }
            )
        })
}

#[allow(
    clippy::too_many_lines,
    reason = "Keep all ordered binding checks in one exhaustive step dispatch."
)]
pub(crate) fn admit(suite: &ConformanceSuite) -> Result<(), AdmissionError> {
    for (id, scenario) in &suite.scenarios {
        crate::live_trace::admit(
            &scenario.steps,
            suite.provenance.suite_version.major().into(),
        )
        .map_err(|reason| {
            AdmissionError::new("InvalidLiveTrace", format!("/scenarios/{id}"), reason)
        })?;
    }
    if suite.provenance.live_inputs_digest.is_some() && suite.provenance.suite_version.major() < 10
    {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "/provenance/live_inputs_digest",
            "live input manifests require suite/10 or /11",
        ));
    }
    if !used_by(suite) {
        return Ok(());
    }
    if suite.provenance.suite_version.major() < 10 {
        return Err(AdmissionError::new(
            "UnsupportedVocabulary",
            "/provenance/suite_version",
            "response captures and payload-path matchers require suite/10 or /11",
        ));
    }
    for (id, scenario) in &suite.scenarios {
        let mut bound = BTreeSet::new();
        let mut last_command = None;
        for (index, step) in scenario.steps.iter().enumerate() {
            let path = format!("/scenarios/{id}/steps/{index}");
            let error = |detail: &str| AdmissionError::new("InvalidLiveBinding", &path, detail);
            match step {
                ScenarioStep::ExecuteCommand { command, input, .. } => {
                    for value in input.values() {
                        if let ScenarioValue::Instance { instance } = value {
                            if !bound.contains(instance) {
                                return Err(error("input capture is not bound by an earlier step"));
                            }
                        }
                    }
                    last_command = Some(command);
                }
                ScenarioStep::CaptureInstance { instance, .. }
                | ScenarioStep::EstablishEntity { instance, .. }
                | ScenarioStep::CheckLive {
                    trace:
                        crate::live_trace::Check::Capture { instance, .. }
                        | crate::live_trace::Check::Offset { instance, .. },
                } => {
                    if !bound.insert(instance) {
                        return Err(error("duplicate capture"));
                    }
                }
                ScenarioStep::CaptureResponse {
                    command,
                    instance,
                    field,
                    shape,
                } => {
                    if last_command != Some(command) {
                        return Err(error("capture does not name the preceding command"));
                    }
                    if !bound.insert(instance) {
                        return Err(error("duplicate capture"));
                    }
                    if field.is_empty() || field.contains('.') {
                        return Err(error("response capture requires one declared field"));
                    }
                    if shape.leaves().get(field).is_none_or(|leaf| leaf.optional) {
                        return Err(error(
                            "captured response field must have a required declared leaf shape",
                        ));
                    }
                }
                ScenarioStep::EventuallyMatchingEvent { matches, shape, .. } => {
                    if matches.is_empty() {
                        return Err(error("event matcher must constrain a payload path"));
                    }
                    for (field, value) in matches {
                        let leaf = shape
                            .leaves()
                            .get(field)
                            .ok_or_else(|| error("match path has no declared shape"))?;
                        match value {
                            ScenarioValue::Literal { value } if leaf.admits(Some(value)) => {}
                            ScenarioValue::Instance { instance } if bound.contains(instance) => {}
                            _ => {
                                return Err(error(
                                    "matcher requires an admitted literal or a preceding actual capture",
                                ));
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    Ok(())
}
