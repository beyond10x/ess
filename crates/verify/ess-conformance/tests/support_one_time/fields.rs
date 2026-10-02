//! Multiple-field live controls; exact-position exemptions never exempt another field.
use super::{Mode, Service, FIRST};
use ess_conformance::{report::Status, target::*, AdmittedSuite};
use ess_primitives::node::Node;
use std::cell::RefCell;

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldMode {
    Healthy,
    SameMarked,
    SameUnmarked,
    OldOther,
}
impl FieldMode {
    pub const ALL: [Self; 4] = [
        Self::Healthy,
        Self::SameMarked,
        Self::SameUnmarked,
        Self::OldOther,
    ];
    pub fn expected(self) -> Status {
        if matches!(self, Self::Healthy) {
            Status::Passed
        } else {
            Status::Failed
        }
    }
}
pub struct FieldService {
    pub inner: Service,
    pub mode: FieldMode,
    returned: RefCell<Vec<String>>,
}
impl FieldService {
    pub fn new(mode: FieldMode) -> Self {
        Self {
            inner: Service::new(Mode::Healthy),
            mode,
            returned: RefCell::new(Vec::new()),
        }
    }
    pub fn returned_plaintexts(&self) -> Vec<String> {
        self.returned.borrow().clone()
    }
}
impl ConformanceTarget for FieldService {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.inner.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let call = self.inner.calls.get();
        let mut result = self.inner.execute_command(request)?;
        let response = result.response.as_mut().unwrap();
        let Node::Text(secret) = response["secret"].clone() else {
            unreachable!()
        };
        let recovery = match self.mode {
            FieldMode::SameMarked => secret.clone(),
            FieldMode::OldOther if call > 0 => FIRST.to_owned(),
            _ => format!("private-recovery-{call:08x}"),
        };
        response.insert("recovery".into(), Node::Text(recovery.clone()));
        response.insert(
            "audit".into(),
            Node::Text(if matches!(self.mode, FieldMode::SameUnmarked) {
                secret.clone()
            } else {
                "public".into()
            }),
        );
        self.returned.borrow_mut().extend([secret, recovery]);
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}
pub fn admitted() -> AdmittedSuite {
    let mut suite = super::support_one_time::admitted(Mode::Healthy)
        .suite()
        .clone();
    for scenario in suite.scenarios.values_mut() {
        let policy = scenario.one_time_response.as_mut().unwrap();
        let origin = &mut policy.origins[0];
        let mut recovery = origin.response.fields[0].clone();
        recovery.name = "recovery".into();
        let mut audit = origin.response.fields[0].clone();
        audit.name = "audit".into();
        origin.response.fields.extend([recovery, audit]);
        origin.fields.push("recovery".into());
        for step in &mut scenario.steps {
            if let ess_conformance::ScenarioStep::ExpectDirectResponse { response } = step {
                response.fields.clone_from(&origin.response.fields);
            }
        }
    }
    AdmittedSuite::from_suite(&suite).unwrap()
}
