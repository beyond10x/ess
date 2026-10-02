//! Shared live payload-budget controls. Payload recipes avoid megabyte fixture files.
use super::{Mode, Service};
use ess_conformance::{report::Status, target::*};
use ess_primitives::node::Node;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceMode {
    BytesExact,
    BytesOver,
    EscapesExact,
    EscapesOver,
    NumbersOver,
    MembersExact,
    MembersOver,
    DepthExact,
    DepthOver,
    IntegralExact,
    IntegralOver,
    FractionalExact,
    FractionalOver,
}
impl ResourceMode {
    pub const ALL: [Self; 13] = [
        Self::BytesExact,
        Self::BytesOver,
        Self::EscapesExact,
        Self::EscapesOver,
        Self::NumbersOver,
        Self::MembersExact,
        Self::MembersOver,
        Self::DepthExact,
        Self::DepthOver,
        Self::IntegralExact,
        Self::IntegralOver,
        Self::FractionalExact,
        Self::FractionalOver,
    ];
    pub fn expected(self) -> Status {
        match self {
            Self::BytesExact
            | Self::EscapesExact
            | Self::MembersExact
            | Self::DepthExact
            | Self::IntegralExact
            | Self::FractionalExact => Status::Passed,
            _ => Status::Unsupported,
        }
    }
    pub fn rows(self) -> Vec<ViewRow> {
        let overhead = 13; // [{"data":""}]
        let value = match self {
            Self::IntegralExact
            | Self::IntegralOver
            | Self::FractionalExact
            | Self::FractionalOver => {
                let fractional = matches!(self, Self::FractionalExact | Self::FractionalOver);
                let number = if fractional { 0.125 } else { 1.0 };
                let mut row = BTreeMap::from([
                    (
                        "number".into(),
                        Node::Number(ess_primitives::facts::Number::new(number).unwrap()),
                    ),
                    ("padding".into(), Node::Text(String::new())),
                ]);
                let framing = serde_json::to_vec(&[&row]).unwrap().len();
                let extra = usize::from(matches!(self, Self::IntegralOver | Self::FractionalOver));
                row.insert(
                    "padding".into(),
                    Node::Text("x".repeat(1_048_576 - framing + extra)),
                );
                return vec![row];
            }
            Self::BytesExact | Self::BytesOver => Node::Text(
                "x".repeat(1_048_576 - overhead + usize::from(matches!(self, Self::BytesOver))),
            ),
            Self::EscapesExact | Self::EscapesOver => {
                let available = 1_048_576 - overhead;
                Node::Text(format!(
                    "{}{}",
                    "\0".repeat(available / 6),
                    "x".repeat(available % 6 + usize::from(matches!(self, Self::EscapesOver)))
                ))
            }
            Self::NumbersOver => Node::Seq(vec![
                Node::Number(1_234_567_890_123_456_789_i64.into());
                65_534
            ]),
            Self::MembersExact | Self::MembersOver => {
                return vec![
                    BTreeMap::new();
                    65_536 + usize::from(matches!(self, Self::MembersOver))
                ]
            }
            Self::DepthExact | Self::DepthOver => {
                let mut value = Node::Null;
                for _ in 0..(126 + usize::from(matches!(self, Self::DepthOver))) {
                    value = Node::Seq(vec![value]);
                }
                value
            }
        };
        vec![BTreeMap::from([("data".into(), value)])]
    }
}

pub struct ResourceService {
    pub inner: Service,
    pub mode: ResourceMode,
}
impl ResourceService {
    pub fn new(mode: ResourceMode) -> Self {
        Self {
            inner: Service::new(Mode::Healthy),
            mode,
        }
    }
}
impl ConformanceTarget for ResourceService {
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
        self.inner.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.inner.query_view(request)?;
        Ok(SemanticViewResult::of(self.mode.rows()))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn configure_external_outcome(
        &self,
        control: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(control)
    }
}
