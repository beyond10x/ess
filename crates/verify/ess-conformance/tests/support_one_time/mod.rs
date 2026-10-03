//! Stateful one-time disclosure controls, shared before foreign-runtime ports consume them.
use ess_conformance::{report::Status, scenario::OutcomeRef, target::*, AdmittedSuite};
use ess_primitives::node::Node;
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
};

pub const FIRST: &str = "private-first-token";

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Mode {
    Healthy,
    Retry,
    DirectEvent,
    MissingOrigin,
    Empty,
    Error,
    Unsupported,
    View,
    DelayedEvent,
    ConstrainedHealthy,
    ConstrainedInvalid,
    IdentityError,
    KeyLeak,
    DeclaredError,
    LaterReuse,
    CaptureBound,
    WindowUnsupported,
    WindowShort,
    WindowHealthy,
    IdentitySuccess,
}

pub struct Service {
    mode: Mode,
    pub calls: Cell<usize>,
    observations: Cell<usize>,
    trace: RefCell<Vec<&'static str>>,
    returned_plaintexts: RefCell<Vec<String>>,
}

impl Service {
    pub fn new(mode: Mode) -> Self {
        Self {
            mode,
            calls: Cell::new(0),
            observations: Cell::new(0),
            trace: RefCell::new(Vec::new()),
            returned_plaintexts: RefCell::new(Vec::new()),
        }
    }
    pub fn trace(&self) -> Vec<&'static str> {
        self.trace.borrow().clone()
    }
    /// Test-only observation inventory, never serialized into fixture manifests or reports.
    pub fn returned_plaintexts(&self) -> Vec<String> {
        self.returned_plaintexts.borrow().clone()
    }
    fn observed(&self, method: &'static str) {
        self.trace.borrow_mut().push(method);
    }
}

impl Mode {
    pub const ALL: [Self; 20] = [
        Self::Healthy,
        Self::Retry,
        Self::DirectEvent,
        Self::MissingOrigin,
        Self::Empty,
        Self::Error,
        Self::Unsupported,
        Self::View,
        Self::DelayedEvent,
        Self::ConstrainedHealthy,
        Self::ConstrainedInvalid,
        Self::IdentityError,
        Self::KeyLeak,
        Self::DeclaredError,
        Self::LaterReuse,
        Self::CaptureBound,
        Self::WindowUnsupported,
        Self::WindowShort,
        Self::WindowHealthy,
        Self::IdentitySuccess,
    ];
    pub fn expected(self) -> (Status, &'static str) {
        match self {
            Self::Healthy
            | Self::ConstrainedHealthy
            | Self::IdentityError
            | Self::IdentitySuccess
            | Self::WindowHealthy => (Status::Passed, "ESS-CF-DISCLOSURE"),
            Self::Error => (Status::Error, "ESS-CF-TARGET"),
            Self::Unsupported
            | Self::CaptureBound
            | Self::WindowUnsupported
            | Self::WindowShort => (Status::Unsupported, "ESS-CF-TARGET"),
            Self::Empty | Self::ConstrainedInvalid => (Status::Failed, "ESS-CF-PAYLOAD"),
            _ => (Status::Failed, "ESS-CF-DISCLOSURE"),
        }
    }
}

impl ConformanceTarget for Service {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.observed("identity");
        if matches!(self.mode, Mode::IdentityError) {
            return Err(TargetError::unavailable(FIRST, FIRST));
        }
        if matches!(self.mode, Mode::IdentitySuccess) {
            return Ok(ImplementationIdentity::new(FIRST, FIRST));
        }
        Ok(ImplementationIdentity::new("one-time-controls", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.observed("begin_scenario");
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.observed("end_scenario");
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.observed("execute_command");
        let call = self.calls.get();
        self.calls.set(call + 1);
        if matches!(self.mode, Mode::Error) {
            return Err(TargetError::unavailable(FIRST, FIRST));
        }
        if matches!(self.mode, Mode::Unsupported) {
            return Err(TargetError::unsupported(FIRST, FIRST));
        }
        let outcome = if matches!(self.mode, Mode::MissingOrigin)
            || (call > 0 && matches!(self.mode, Mode::DeclaredError))
        {
            "other"
        } else {
            "issued"
        };
        let mut result =
            SemanticCommandResult::took(OutcomeRef::new(request.command, outcome.parse().unwrap()));
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("write-1").unwrap());
        let value = if matches!(self.mode, Mode::ConstrainedHealthy) {
            if call == 0 {
                "abca".into()
            } else {
                "abcb".into()
            }
        } else if matches!(self.mode, Mode::Empty) {
            String::new()
        } else if call == 0
            || matches!(self.mode, Mode::Retry)
            || (call == 2 && matches!(self.mode, Mode::LaterReuse))
        {
            FIRST.into()
        } else {
            format!("fresh-token-{call:08x}")
        };
        result.response = Some(BTreeMap::from([("secret".into(), Node::Text(value))]));
        if call > 0 && matches!(self.mode, Mode::DeclaredError) {
            result.response = None;
            result.error = Some(
                DeclaredErrorValue::new("credentials.api.Refused".parse().unwrap())
                    .with("reason", Node::Text(FIRST.into())),
            );
        }
        if matches!(self.mode, Mode::DirectEvent) {
            result.direct_events.push(
                ObservedEvent::new("credentials.api.Issued".parse().unwrap())
                    .with("leak", Node::Text(FIRST.into())),
            );
        }
        if let Some(response) = &result.response {
            for value in response.values() {
                if let Node::Text(text) = value {
                    if !text.is_empty() {
                        self.returned_plaintexts.borrow_mut().push(text.clone());
                    }
                }
            }
        }
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.observed("query_view");
        if matches!(self.mode, Mode::KeyLeak) {
            return Ok(SemanticViewResult::of([BTreeMap::from([(
                format!("prefix-{FIRST}"),
                Node::Null,
            )])]));
        }
        Ok(SemanticViewResult::of([BTreeMap::from([(
            "audit".into(),
            Node::Text(format!("wrapped-{FIRST}-suffix")),
        )])]))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.observed("observe_events");
        let count = self.observations.get() + 1;
        self.observations.set(count);
        Ok(vec![ObservedEvent::new(
            "credentials.api.Issued".parse().unwrap(),
        )
        .with(
            "audit",
            Node::Text(if matches!(self.mode, Mode::DelayedEvent) && count >= 3 {
                FIRST.into()
            } else {
                "harmless".into()
            }),
        )])
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        unreachable!()
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        unreachable!()
    }
    fn mark_instant(&self, _: InstantMark) -> Result<(), TargetError> {
        self.observed("mark_instant");
        if matches!(self.mode, Mode::WindowUnsupported) {
            Err(TargetError::unsupported(FIRST, FIRST))
        } else {
            Ok(())
        }
    }
    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        self.observed("observe_elapsed");
        Ok(ElapsedObservation {
            elapsed_ms: if matches!(self.mode, Mode::WindowShort) {
                0
            } else {
                request.hold.millis()
            },
            published: 0,
        })
    }
}

pub fn admitted(mode: Mode) -> AdmittedSuite {
    let fixture = if matches!(mode, Mode::ConstrainedHealthy | Mode::ConstrainedInvalid) {
        include_str!("../fixtures/one-time-response/valid-constrained-string.json")
    } else if matches!(
        mode,
        Mode::DelayedEvent | Mode::WindowUnsupported | Mode::WindowShort | Mode::WindowHealthy
    ) {
        include_str!("../fixtures/one-time-response/valid-event-window.json")
    } else {
        include_str!("../fixtures/one-time-response/valid-string.json")
    };
    let mut value: serde_json::Value = serde_json::from_str(fixture).unwrap();
    for scenario in value["scenarios"].as_object_mut().unwrap().values_mut() {
        if matches!(mode, Mode::View | Mode::KeyLeak) {
            scenario["steps"]
                .as_array_mut()
                .unwrap()
                .push(serde_json::json!({"step":"query_view","view":"credentials.api.Audit"}));
        } else if !matches!(
            mode,
            Mode::DelayedEvent | Mode::WindowUnsupported | Mode::WindowShort | Mode::WindowHealthy
        ) {
            let repeated = scenario["steps"].as_array().unwrap().clone();
            let repetitions = if matches!(mode, Mode::CaptureBound) {
                256
            } else if matches!(mode, Mode::LaterReuse) {
                2
            } else {
                1
            };
            for _ in 0..repetitions {
                scenario["steps"]
                    .as_array_mut()
                    .unwrap()
                    .extend(repeated.clone());
            }
        }
    }
    AdmittedSuite::from_json(&serde_json::to_string(&value).unwrap()).unwrap()
}
