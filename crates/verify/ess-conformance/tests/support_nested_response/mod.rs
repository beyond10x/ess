//! Independent responses and events for nested relationship mutation controls.
pub mod admission;
pub mod foreign;
pub mod values;
use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::{target::*, AdmittedSuite};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::{cell::Cell, collections::BTreeMap};

pub const MODEL: &str = r"format: ess/14
system: demo
version: v1
domain: demo.api
types:
  - name: demo.api.Packet
    kind: struct
    fields:
      - {name: value, type: Integer}
events:
  - name: demo.api.Returned
    fields:
      - {name: packet, type: demo.api.Packet}
      - {name: receipt, type: String}
commands:
  - name: demo.api.Read
    response:
      - {name: value, type: Integer}
    outcomes:
      - name: returned
        emits: [demo.api.Returned]
        payload:
          demo.api.Returned:
            packet:
              value: {response: value}
            receipt: {generated: true}
";

pub fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("nested-response.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap();
    compile(&spec, &SourceMap::new()).unwrap()
}

pub fn suite(text: &str) -> AdmittedSuite {
    let synthesis = ess_conformance::synthesize::synthesize(&model(text));
    assert!(synthesis.refusals.is_empty(), "{:?}", synthesis.refusals);
    assert_eq!(synthesis.suite.scenarios.len(), 1);
    AdmittedSuite::from_suite(&synthesis.suite).unwrap()
}

#[derive(Clone, Copy, Debug)]
pub enum Fault {
    None,
    Event,
    Response,
    GeneratedSibling,
}

pub struct Backend {
    pub fault: Fault,
    pub calls: Cell<usize>,
    pub callbacks: Cell<usize>,
    pub values: Option<(BTreeMap<String, Node>, BTreeMap<String, Node>)>,
}

impl Backend {
    pub fn new(fault: Fault) -> Self {
        Self {
            fault,
            calls: Cell::new(0),
            callbacks: Cell::new(0),
            values: None,
        }
    }
}

impl ConformanceTarget for Backend {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.callbacks.set(self.callbacks.get() + 1);
        Ok(ImplementationIdentity::new("nested-response-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.callbacks.set(self.callbacks.get() + 1);
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.callbacks.set(self.callbacks.get() + 1);
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.calls.set(self.calls.get() + 1);
        self.callbacks.set(self.callbacks.get() + 1);
        assert_eq!(request.command.to_string(), "demo.api.Read");
        let outcome =
            ess_compiler::refs::OutcomeRef::new(request.command, "returned".parse().unwrap());
        let mut result = SemanticCommandResult::took(outcome);
        let response = if matches!(self.fault, Fault::Response) {
            38_i64
        } else {
            37
        };
        let emitted = if matches!(self.fault, Fault::Event) {
            38_i64
        } else {
            37
        };
        result.response = Some(BTreeMap::from([(
            "value".into(),
            Node::Number(response.into()),
        )]));
        let mut event = ObservedEvent::new("demo.api.Returned".parse().unwrap());
        event.payload = BTreeMap::from([
            (
                "packet".into(),
                Node::Map(BTreeMap::from([(
                    "value".into(),
                    Node::Number(emitted.into()),
                )])),
            ),
            (
                "receipt".into(),
                if matches!(self.fault, Fault::GeneratedSibling) {
                    Node::Null
                } else {
                    Node::Text("generated-receipt".into())
                },
            ),
        ]);
        if let Some((response, payload)) = &self.values {
            result.response = Some(response.clone());
            event.payload = payload.clone();
        }
        result.direct_events.push(event);
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", "unused"))
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported("external", "unused"))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported("redelivery", "unused"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Err(TargetError::unsupported("events", "unused"))
    }
    fn observe_invocations(
        &self,
        _: InvocationObservationRequest,
    ) -> Result<Vec<ObservedInvocation>, TargetError> {
        Err(TargetError::unsupported("invocations", "unused"))
    }
}
