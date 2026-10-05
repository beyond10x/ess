//! Independent response service, linked unchanged by native and emitted-module browser tests.
//! It never receives the suite, expected outputs or compiler model.
use ess_compiler::refs::OutcomeRef;
use ess_conformance::{
    target::*,
    web_execution::{Installation, Installed, RunContext},
    Clock, RunnerConfig,
};
use ess_primitives::{facts::Number, node::Node, time::Timestamp};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
    sync::atomic::{AtomicU32, Ordering},
};

static FACTORIES: AtomicU32 = AtomicU32::new(0);
/// Independent fixture instrumentation, never a product admission bypass.
pub fn factory_calls() -> u32 {
    FACTORIES.load(Ordering::Relaxed)
}

/// Fixed implementation faults, chosen by the consumer's concrete installation type.
/// 0 healthy; 1 wrong response; 2 wrong event; 3 absent response; 4 wrong type;
/// 5 extra field; 6 wrong outcome.
pub struct BrowserInstallation<const MODE: u8>;
/// Execution-budget and implementation wall clock over one shared authority.
pub struct SharedClock(Rc<Cell<u64>>);
impl Clock for SharedClock {
    fn now(&mut self) -> Timestamp {
        let now = self.0.get();
        self.0.set(now + 100);
        Timestamp::from_epoch_millis(now)
    }
    fn wall(&mut self) -> Timestamp {
        Timestamp::from_epoch_millis(self.0.get())
    }
}
/// Independent response/event implementation, with isolated per-run storage.
pub struct Backend<const MODE: u8> {
    namespace: String,
    clock: Rc<Cell<u64>>,
    events: RefCell<Vec<ObservedEvent>>,
    active: RefCell<Option<String>>,
    invocations: Cell<u64>,
}

/// Independently implemented optional/list API. Faults affect one command each:
/// 1 omit a required-null field; 2 send null for required omission; 3 reorder a list;
/// 4 remove a duplicate; 5 replace an exact integer by its adjacent large integer;
/// 6 omit an actually present optional response while its event retains the value.
pub struct ValueFormsInstallation<const MODE: u8>;
pub struct ValueFormsTarget<const MODE: u8> {
    inner: Backend<0>,
}
impl<const MODE: u8> Installation for ValueFormsInstallation<MODE> {
    type Target = ValueFormsTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        context: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        let installed = BrowserInstallation::<0>::create(context)?;
        Ok(Installed {
            target: ValueFormsTarget {
                inner: installed.target,
            },
            clock: installed.clock,
            config: installed.config,
        })
    }
}
fn ordered_value_items(mode: u8) -> Node {
    const A: i64 = 9_007_199_254_740_993;
    const B: i64 = 9_007_199_254_740_994;
    let first = match mode {
        3 => vec![B, B, A],
        4 => vec![A, B],
        5 => vec![B, B, B],
        _ => vec![A, B, B],
    };
    let numbers = |values: Vec<i64>| {
        Node::Seq(
            values
                .into_iter()
                .map(|n| Node::Number(Number::from(n)))
                .collect(),
        )
    };
    Node::Seq(vec![
        Node::Map(BTreeMap::from([
            ("numbers".into(), numbers(first)),
            ("note".into(), Node::Null),
        ])),
        Node::Map(BTreeMap::from([
            ("numbers".into(), numbers(vec![B, A])),
            ("note".into(), Node::Text("second".into())),
        ])),
    ])
}
impl<const MODE: u8> ConformanceTarget for ValueFormsTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "independent-browser-value-forms",
            "1",
        ))
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
        if self.inner.active.borrow().as_deref() != Some(request.correlation.to_string().as_str()) {
            return Err(TargetError::unsupported(
                "context",
                "inactive isolated scenario",
            ));
        }
        let command = request.command.to_string();
        let (response, observed, event_name) = if command == "values.api.Lists" {
            (
                BTreeMap::from([("value".into(), ordered_value_items(0))]),
                BTreeMap::from([("value".into(), ordered_value_items(MODE))]),
                "values.api.ListReturned",
            )
        } else {
            let (mut returned, observed) = match command.as_str() {
                "values.api.Absent" | "values.api.OmitPolicy" => (None, None),
                "values.api.Null" | "values.api.NullPolicy" => (Some(Node::Null), Some(Node::Null)),
                "values.api.Equivalent" => (None, Some(Node::Null)),
                "values.api.Present" => {
                    let value = Node::Number(Number::from(9_007_199_254_740_993_i64));
                    (Some(value.clone()), Some(value))
                }
                _ => {
                    return Err(TargetError::unsupported(
                        "command",
                        "not this service's API",
                    ));
                }
            };
            if (MODE == 1 && command == "values.api.NullPolicy")
                || (MODE == 6 && command == "values.api.Present")
            {
                returned = None;
            }
            if MODE == 2 && command == "values.api.OmitPolicy" {
                returned = Some(Node::Null);
            }
            let field = |value: Option<Node>| {
                value
                    .into_iter()
                    .map(|value| ("value".into(), value))
                    .collect()
            };
            (
                field(returned),
                field(observed),
                "values.api.OptionalReturned",
            )
        };
        self.inner.invocations.set(self.inner.invocations.get() + 1);
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            "returned".parse().unwrap(),
        ));
        result.response = Some(response);
        let event = ObservedEvent::new(event_name.parse().unwrap())
            .with("packet", Node::Map(observed))
            .with(
                "receipt",
                Node::Text(format!(
                    "{}-{}",
                    self.inner.namespace,
                    self.inner.invocations.get()
                )),
            )
            .in_activity(request.correlation)
            .at(self.inner.invocations.get());
        self.inner.events.borrow_mut().push(event.clone());
        result.direct_events.push(event);
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
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}
impl<const MODE: u8> Installation for BrowserInstallation<MODE> {
    type Target = Backend<MODE>;
    type Clock = SharedClock;
    fn create(
        context: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        let clock = Rc::new(Cell::new(1_767_603_600_000));
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        Ok(Installed {
            target: Backend {
                namespace: context.namespace.clone(),
                clock: Rc::clone(&clock),
                events: RefCell::new(Vec::new()),
                active: RefCell::new(None),
                invocations: Cell::new(0),
            },
            clock: SharedClock(clock),
            config: RunnerConfig::new(500),
        })
    }
}
fn item(remaining: i64) -> Node {
    Node::Map(BTreeMap::from([
        ("remaining".into(), Node::Number(Number::from(remaining))),
        ("created".into(), Node::Text("2026-01-05T08:00:00Z".into())),
        ("ended".into(), Node::Text("2026-01-05T09:00:00Z".into())),
        ("state".into(), Node::Null),
        ("call_type".into(), Node::Text("incoming".into())),
        (
            "features".into(),
            Node::Map(BTreeMap::from([("recording".into(), Node::Bool(true))])),
        ),
    ]))
}

/// Retention belongs to the service: records are keyed by real input and trusted caller,
/// and hold the actual first response. No expected result or suite enters this target.
pub struct RetainedInstallation<const MODE: u8>;
pub struct RetainedTarget<const MODE: u8> {
    inner: Backend<0>,
    records: RefCell<Vec<RetainedRecord>>,
}
struct RetainedRecord {
    input: BTreeMap<String, Node>,
    actor: Option<String>,
    caller: Option<BTreeMap<String, Node>>,
    response: BTreeMap<String, Node>,
    row: ViewRow,
}
impl<const MODE: u8> Installation for RetainedInstallation<MODE> {
    type Target = RetainedTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        context: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        let installed = BrowserInstallation::<0>::create(context)?;
        Ok(Installed {
            target: RetainedTarget {
                inner: installed.target,
                records: RefCell::new(Vec::new()),
            },
            clock: installed.clock,
            config: installed.config,
        })
    }
}
impl<const MODE: u8> ConformanceTarget for RetainedTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "independent-browser-retained",
            "1",
        ))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.records.borrow_mut().clear();
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() != "retained.core.Seed"
            || self.inner.active.borrow().as_deref()
                != Some(request.correlation.to_string().as_str())
        {
            return Err(TargetError::unsupported(
                "command",
                "inactive context or unknown API",
            ));
        }
        let actor = request.actor.as_ref().map(ToString::to_string);
        let mut records = self.records.borrow_mut();
        let retained = records.iter_mut().find(|entry| {
            entry.input == request.input && entry.actor == actor && entry.caller == request.caller
        });
        if let Some(entry) = retained {
            let mut result = SemanticCommandResult::took(OutcomeRef::new(
                request.command,
                "replayed".parse().unwrap(),
            ));
            let mut response = entry.response.clone();
            // 1 wrong result; 2 stale timestamp; 3 illicit mutation of the retained subject.
            if MODE == 1 {
                response.insert(
                    "number".into(),
                    Node::Number(Number::from(9_007_199_254_740_994_i64)),
                );
            }
            if MODE == 2 {
                response.insert("stamp".into(), Node::Text("2026-01-05T08:59:59Z".into()));
            }
            if MODE == 3 {
                entry
                    .row
                    .insert("stamp".into(), Node::Text("2026-01-05T09:00:01Z".into()));
            }
            result.response = Some(response);
            result.consistency =
                Some(ess_primitives::consistency::ConsistencyToken::new("retained-write").unwrap());
            return Ok(result);
        }
        let serial = records.len() + 1;
        let id = Node::Text(format!("00000000-0000-4000-8000-{serial:012}"));
        let stamp = Node::Text("2026-01-05T09:00:00Z".into());
        let response = BTreeMap::from([
            ("revision_id".into(), id.clone()),
            ("stamp".into(), stamp.clone()),
            (
                "number".into(),
                Node::Number(Number::from(9_007_199_254_740_993_i64)),
            ),
            (
                "values".into(),
                Node::Seq(vec![
                    Node::Number(Number::from(i64::MIN)),
                    Node::Number(Number::from(i64::MAX)),
                ]),
            ),
        ]);
        let row = BTreeMap::from([
            ("record_id".into(), id.clone()),
            (
                "value".into(),
                request.input.get("document").cloned().unwrap(),
            ),
            ("stamp".into(), stamp),
            ("state".into(), Node::Text("Committed".into())),
        ]);
        records.push(RetainedRecord {
            input: request.input,
            actor,
            caller: request.caller,
            response: response.clone(),
            row,
        });
        let event = ObservedEvent::new("retained.core.Seeded".parse().unwrap())
            .with("record_id", id)
            .in_activity(request.correlation)
            .at(serial as u64);
        self.inner.events.borrow_mut().push(event.clone());
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            "seeded".parse().unwrap(),
        ));
        result.response = Some(response);
        result.direct_events.push(event);
        result.consistency =
            Some(ess_primitives::consistency::ConsistencyToken::new("retained-write").unwrap());
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        if request.view.to_string() != "retained.core.Records" {
            return Err(TargetError::unsupported("view", "unknown API"));
        }
        Ok(SemanticViewResult {
            rows: self
                .records
                .borrow()
                .iter()
                .map(|entry| entry.row.clone())
                .collect(),
            total: None,
        })
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("retained replay must arise from an actual duplicate invocation")
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}

/// Fresh identities are allocated by the independent service, then used for the row,
/// response and event. Fault 1 reuses the first identity; fault 2 misassociates the event.
pub struct CreationInstallation<const MODE: u8>;
pub struct CreationTarget<const MODE: u8> {
    inner: Backend<0>,
    rows: RefCell<BTreeMap<String, ViewRow>>,
}
impl<const MODE: u8> Installation for CreationInstallation<MODE> {
    type Target = CreationTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        context: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        let installed = BrowserInstallation::<0>::create(context)?;
        Ok(Installed {
            target: CreationTarget {
                inner: installed.target,
                rows: RefCell::new(BTreeMap::new()),
            },
            clock: installed.clock,
            config: installed.config,
        })
    }
}
impl<const MODE: u8> ConformanceTarget for CreationTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "independent-browser-creation",
            "1",
        ))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        self.inner.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.inner.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() != "demo.response.Create"
            || self.inner.active.borrow().as_deref()
                != Some(request.correlation.to_string().as_str())
        {
            return Err(TargetError::unsupported(
                "command",
                "inactive context or unknown API",
            ));
        }
        let serial = self.inner.invocations.get() + 1;
        self.inner.invocations.set(serial);
        let allocated = if MODE == 1 { 1 } else { serial };
        let id = format!("00000000-0000-4000-8000-{allocated:012}");
        self.rows.borrow_mut().insert(
            id.clone(),
            BTreeMap::from([("id".into(), Node::Text(id.clone()))]),
        );
        let event_id = if MODE == 2 {
            format!("00000000-0000-4000-8000-{:012}", serial + 100)
        } else {
            id.clone()
        };
        let event = ObservedEvent::new("demo.response.Created".parse().unwrap())
            .with("id", Node::Text(event_id))
            .in_activity(request.correlation)
            .at(serial);
        self.inner.events.borrow_mut().push(event.clone());
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            "created".parse().unwrap(),
        ));
        result.response = Some(BTreeMap::from([("id".into(), Node::Text(id))]));
        result.direct_events.push(event);
        result.consistency = Some(
            ess_primitives::consistency::ConsistencyToken::new(format!("creation-{serial}"))
                .unwrap(),
        );
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        if request.view.to_string() != "demo.response.CreatedRows" {
            return Err(TargetError::unsupported("view", "unknown API"));
        }
        Ok(SemanticViewResult {
            rows: self.rows.borrow().values().cloned().collect(),
            total: None,
        })
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.inner.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}
impl<const MODE: u8> ConformanceTarget for Backend<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "browser-independent-response-service",
            "1",
        ))
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.events.borrow_mut().clear();
        self.invocations.set(0);
        *self.active.borrow_mut() = Some(scenario.correlation.to_string());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.active.borrow_mut() = None;
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let nested = request.command.to_string() == "demo.api.Read";
        if !nested && request.command.to_string() != "demo.api.Cancel" {
            return Err(TargetError::unsupported(
                "command",
                "not this service's API",
            ));
        }
        if self.active.borrow().as_deref() != Some(request.correlation.to_string().as_str()) {
            return Err(TargetError::unsupported(
                "context",
                "inactive isolated scenario",
            ));
        }
        self.invocations.set(self.invocations.get() + 1);
        if nested {
            let mut result = SemanticCommandResult::took(OutcomeRef::new(
                request.command,
                "returned".parse().unwrap(),
            ));
            let response = if MODE == 4 {
                Node::Text("wrong type".into())
            } else {
                Node::Number(Number::from(if MODE == 1 { 38_i64 } else { 37_i64 }))
            };
            let mut fields = BTreeMap::from([("value".into(), response)]);
            if MODE == 5 {
                fields.insert("undeclared".into(), Node::Null);
            }
            if MODE != 3 {
                result.response = Some(fields);
            }
            let event = ObservedEvent::new("demo.api.Returned".parse().unwrap())
                .with(
                    "packet",
                    Node::Map(BTreeMap::from([(
                        "value".into(),
                        Node::Number(Number::from(if MODE == 2 { 38_i64 } else { 37_i64 })),
                    )])),
                )
                .with(
                    "receipt",
                    Node::Text(format!("{}-{}", self.namespace, self.invocations.get())),
                )
                .in_activity(request.correlation)
                .at(self.invocations.get());
            self.events.borrow_mut().push(event.clone());
            result.direct_events.push(event);
            return Ok(result);
        }
        let response_item = if MODE == 4 {
            Node::Text("wrong type".into())
        } else {
            item(if MODE == 1 {
                9_007_199_254_740_994
            } else {
                9_007_199_254_740_993
            })
        };
        let event_item = item(if MODE == 2 {
            9_007_199_254_740_994
        } else {
            9_007_199_254_740_993
        });
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            if MODE == 6 { "wrong" } else { "cancelled" }
                .parse()
                .unwrap(),
        ));
        let mut response = BTreeMap::from([("item".into(), response_item)]);
        if MODE == 5 {
            response.insert("undeclared".into(), Node::Null);
        }
        if MODE != 3 {
            result.response = Some(response);
        }
        let event = ObservedEvent::new("demo.api.Returned".parse().unwrap())
            .with("item", event_item)
            .with(
                "receipt",
                Node::Text(format!(
                    "{}-{}-{}",
                    self.namespace,
                    self.clock.get(),
                    self.invocations.get()
                )),
            )
            .in_activity(request.correlation)
            .at(self.invocations.get());
        self.events.borrow_mut().push(event.clone());
        result.direct_events.push(event);
        Ok(result)
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", "not this service's API"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|event| {
                event.event == request.event
                    && event.correlation.as_ref() == Some(&request.correlation)
            })
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "not this service's API",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivery",
            "not this service's API",
        ))
    }
}

// Reuse the existing independently stateful target, not its expected-status/suite helpers.
// This source stays in the real fixture crate; no fixture implementation is copied into a host.
#[path = "../../../../../../verify/ess-conformance/tests/support_one_time/mod.rs"]
#[allow(dead_code)]
// The shared fixture's native inventory helpers are not linked host entrypoints.
mod protected_service;

/// A distinct installation for the existing stateful protected-value service.
pub struct ProtectedInstallation<const MODE: u8>;
/// Existing protected-value service with namespace-specific actual values.
pub struct ProtectedTarget {
    inner: protected_service::Service,
    namespace: String,
}
impl<const MODE: u8> Installation for ProtectedInstallation<MODE> {
    type Target = ProtectedTarget;
    type Clock = SharedClock;
    fn create(
        context: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        use protected_service::Mode;
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        let mode = match MODE {
            0 => Mode::Healthy,
            1 => Mode::Retry,
            2 => Mode::DirectEvent,
            3 => Mode::IdentitySuccess,
            4 => Mode::View,
            _ => Mode::Error,
        };
        Ok(Installed {
            target: ProtectedTarget {
                inner: protected_service::Service::new(mode),
                namespace: context.namespace.clone(),
            },
            clock: SharedClock(Rc::new(Cell::new(1_767_603_600_000))),
            config: RunnerConfig::new(500),
        })
    }
}
impl ProtectedTarget {
    fn protect(&self, value: Node) -> Node {
        match value {
            Node::Text(text) if text.starts_with("private-") => {
                Node::Text(format!("{}-{text}", self.namespace))
            }
            Node::Map(fields) => Node::Map(
                fields
                    .into_iter()
                    .map(|(key, value)| (key, self.protect(value)))
                    .collect(),
            ),
            Node::Seq(values) => Node::Seq(
                values
                    .into_iter()
                    .map(|value| self.protect(value))
                    .collect(),
            ),
            other => other,
        }
    }
    fn fields(&self, fields: BTreeMap<String, Node>) -> BTreeMap<String, Node> {
        fields
            .into_iter()
            .map(|(key, value)| (key, self.protect(value)))
            .collect()
    }
    fn events(&self, events: Vec<ObservedEvent>) -> Vec<ObservedEvent> {
        events
            .into_iter()
            .map(|mut event| {
                event.payload = self.fields(event.payload);
                event
            })
            .collect()
    }
}
impl ConformanceTarget for ProtectedTarget {
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
        let mut result = self.inner.execute_command(request)?;
        result.response = result.response.map(|fields| self.fields(fields));
        result.direct_events = self.events(result.direct_events);
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.inner.query_view(request)?;
        result.rows = result
            .rows
            .into_iter()
            .map(|fields| self.fields(fields))
            .collect();
        Ok(result)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self.events(self.inner.observe_events(request)?))
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        self.inner.mark_instant(request)
    }
    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        self.inner.observe_elapsed(request)
    }
}

/// The shared one-time observer manifest's stateful service, unchanged. `MODE` indexes the
/// shared `Mode::ALL` order of `one-time-execution/manifest.json`; the clock and configuration
/// equal `Runner::for_suite`, which produced that manifest's statuses, counts and traces.
pub struct OneTimeInstallation<const MODE: u8>;
/// The shared service; its value-free callback trace is kept when the product drops it.
pub struct OneTimeTarget(protected_service::Service);
static ONE_TIME_TRACE: std::sync::Mutex<Vec<&'static str>> = std::sync::Mutex::new(Vec::new());
/// The callback trace of the most recently dropped one-time service (native inspection only).
pub fn last_one_time_trace() -> Vec<&'static str> {
    ONE_TIME_TRACE.lock().unwrap().clone()
}
/// The shared manifest's case name for an installation mode.
pub fn one_time_case(mode: u8) -> Option<String> {
    let mode = protected_service::Mode::ALL.get(usize::from(mode))?;
    serde_json::to_value(mode)
        .ok()?
        .as_str()
        .map(ToOwned::to_owned)
}
impl Drop for OneTimeTarget {
    fn drop(&mut self) {
        *ONE_TIME_TRACE.lock().unwrap() = self.0.trace();
    }
}
impl<const MODE: u8> Installation for OneTimeInstallation<MODE> {
    type Target = OneTimeTarget;
    type Clock = ess_conformance::AdvancingClock;
    fn create(
        _: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        let mode = *protected_service::Mode::ALL
            .get(usize::from(MODE))
            .ok_or(ess_conformance::web_execution::Error::InstallationRequired)?;
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        Ok(Installed {
            target: OneTimeTarget(protected_service::Service::new(mode)),
            clock: ess_conformance::AdvancingClock::default(),
            config: RunnerConfig::default(),
        })
    }
}
impl ConformanceTarget for OneTimeTarget {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(context)
    }
    fn end_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(context)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        self.0.query_view(request)
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(request)
    }
    fn configure_external_outcome(
        &self,
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.0.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(request)
    }
    fn mark_instant(&self, request: InstantMark) -> Result<(), TargetError> {
        self.0.mark_instant(request)
    }
    fn observe_elapsed(
        &self,
        request: ElapsedObservationRequest,
    ) -> Result<ElapsedObservation, TargetError> {
        self.0.observe_elapsed(request)
    }
}

/// The independent order service exercises real lifecycle, projections, grants and bindings.
/// This is the hand-written fixed service, never the model interpreter.
pub struct OrderInstallation<const MODE: u8>;
impl<const MODE: u8> Installation for OrderInstallation<MODE> {
    type Target = ess_conformance::reference::Oracle;
    type Clock = SharedClock;
    fn create(
        _: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        use ess_conformance::reference::Oracle;
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        let target = match MODE {
            1 => Oracle::new().with_binding_dropped(Oracle::HANDOFF_ON_PLACED),
            2 => Oracle::new().with_mapping_swapped(Oracle::HANDOFF_ON_PLACED),
            _ => Oracle::new(),
        };
        Ok(Installed {
            target,
            clock: SharedClock(Rc::new(Cell::new(1_767_603_600_000))),
            config: RunnerConfig::default(),
        })
    }
}

/// Independent controlled timer installation. Fault 1 corrupts a fresh read mapping; 2 fires early.
pub struct PeriodicInstallation<const MODE: u8>;
/// Fixed-contract timer with independent occurrence and controlled-time state.
pub struct TimerTarget<const MODE: u8> {
    namespace: String,
    clock: Rc<Cell<u64>>,
    timer: RefCell<Timer>,
    events: RefCell<Vec<ObservedEvent>>,
}
#[derive(Default)]
struct Timer {
    scope: Option<ess_conformance::periodic::Scope>,
    fixture: Option<ess_conformance::periodic::Fixture>,
    anchor: u64,
    now: u64,
    next: u64,
    active: bool,
    busy: Option<(u64, u64)>,
    pending: Option<u64>,
    records: Vec<ess_conformance::periodic::Record>,
}
impl Timer {
    fn record(&mut self, fact: ess_conformance::periodic::Fact) {
        self.records.push(ess_conformance::periodic::Record {
            scope: self.scope.clone().expect("active timer has a bound scope"),
            fact,
        });
    }
    fn receive<const MODE: u8>(&mut self, ordinal: u64) {
        use ess_conformance::periodic::{Fact, Fixture};
        let eligible = !(self.fixture == Some(Fixture::InitiallyInactive) && ordinal == 1);
        let due = ordinal * 2000;
        self.record(Fact::Received {
            ordinal,
            due_ms: due,
            at_ms: if MODE == 2 { due - 1 } else { self.now },
            eligible,
        });
        if eligible {
            self.busy = Some((
                ordinal,
                if self.fixture == Some(Fixture::SlowFirstRead) && ordinal == 1 {
                    self.now + 5000
                } else {
                    self.now
                },
            ));
        } else {
            self.record(Fact::Completed {
                ordinal,
                at_ms: self.now,
            });
        }
    }
    fn complete<const MODE: u8>(&mut self, ordinal: u64, namespace: &str) {
        use ess_conformance::periodic::{Fact, Fixture};
        if self.fixture == Some(Fixture::FirstReadFails) && ordinal == 1 {
            self.record(Fact::ReadFailed {
                ordinal,
                at_ms: self.now,
            });
        } else {
            let read = BTreeMap::from([("status".into(), Node::Text(format!("fresh-{ordinal}")))]);
            let mut input = read.clone();
            input.insert("agent_id".into(), Node::Text(namespace.into()));
            if MODE == 1 {
                input.insert("status".into(), Node::Text("stale".into()));
            }
            self.record(Fact::Invoked {
                ordinal,
                at_ms: self.now,
                invocation: format!("{namespace}-poll-{ordinal}"),
                command: "example.poll.Refresh".parse().unwrap(),
                read,
                input,
            });
        }
        self.record(Fact::Completed {
            ordinal,
            at_ms: self.now,
        });
        self.busy = None;
        if let Some(pending) = self.pending.take() {
            self.receive::<MODE>(pending);
        }
    }
    fn advance<const MODE: u8>(&mut self, through: u64, namespace: &str) {
        loop {
            let complete = self.busy.map_or(u64::MAX, |(_, at)| at);
            let tick = if self.active { self.next } else { u64::MAX };
            let next = complete.min(tick);
            if next > through {
                break;
            }
            self.now = next;
            if complete <= tick {
                self.complete::<MODE>(self.busy.unwrap().0, namespace);
            } else {
                let ordinal = tick / 2000;
                self.next += 2000;
                if self.busy.is_none() {
                    self.receive::<MODE>(ordinal);
                } else if self.pending.is_none() {
                    self.pending = Some(ordinal);
                } else {
                    self.record(ess_conformance::periodic::Fact::Dropped {
                        first: ordinal,
                        last: ordinal,
                        at_ms: self.now,
                    });
                }
            }
        }
        self.now = through;
    }
}
impl<const MODE: u8> Installation for PeriodicInstallation<MODE> {
    type Target = TimerTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        context: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        let clock = Rc::new(Cell::new(1_767_603_600_000));
        Ok(Installed {
            target: TimerTarget {
                namespace: context.namespace.clone(),
                clock: Rc::clone(&clock),
                timer: RefCell::new(Timer::default()),
                events: RefCell::new(Vec::new()),
            },
            clock: SharedClock(clock),
            config: RunnerConfig::default(),
        })
    }
}
impl<const MODE: u8> ConformanceTarget for TimerTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "independent-browser-timer",
            "1",
        ))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.timer.borrow_mut() = Timer::default();
        self.events.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.timer.borrow_mut().active = false;
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() != "example.poll.Refresh" {
            return Err(TargetError::unsupported("command", "not the timer service"));
        }
        let status = request.input.get("status").cloned().unwrap_or(Node::Null);
        let event = ObservedEvent::new("example.poll.Updated".parse().unwrap())
            .with("status", status)
            .in_activity(request.correlation);
        self.events.borrow_mut().push(event.clone());
        Ok(SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            "refreshed".parse().unwrap(),
        ))
        .emitting(event))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", "not the timer service"))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|event| {
                event.event == request.event
                    && event.correlation.as_ref() == Some(&request.correlation)
            })
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "not the timer service",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivery",
            "not the timer service",
        ))
    }
    fn mark_instant(&self, _: InstantMark) -> Result<(), TargetError> {
        self.timer.borrow_mut().anchor = self.clock.get();
        Ok(())
    }
    fn open_periodic(
        &self,
        request: ess_conformance::periodic::Open,
    ) -> Result<ess_conformance::periodic::Opened, TargetError> {
        use ess_conformance::periodic::{Opened, Scope};
        if request.check.periodic.host.authority.as_str() != "authenticated-session-status"
            || request.check.periodic.every.milliseconds() != 2000
            || request.check.command.to_string() != "example.poll.Refresh"
        {
            return Err(TargetError::unsupported(
                "periodic host authority",
                "not the installed timer contract",
            ));
        }
        let scope = Scope {
            correlation: request.mark.correlation,
            binding: request.check.binding,
            lifetime: self.namespace.clone(),
        };
        let mut timer = self.timer.borrow_mut();
        timer.scope = Some(scope.clone());
        timer.fixture = Some(request.check.fixture);
        timer.next = 2000;
        timer.active = true;
        Ok(Opened {
            scope,
            anchor_ms: 0,
            context: BTreeMap::from([("agent_id".into(), Node::Text(self.namespace.clone()))]),
        })
    }
    fn observe_periodic(
        &self,
        request: ess_conformance::periodic::Observe,
    ) -> Result<ess_conformance::periodic::Observation, TargetError> {
        let mut timer = self.timer.borrow_mut();
        if timer.scope.as_ref() != Some(&request.scope) {
            return Err(TargetError::unavailable("periodic capture", "stale scope"));
        }
        let through = u64::from(request.elapsed.hold.get()) * 1000;
        timer.advance::<MODE>(through, &self.namespace);
        self.clock.set(self.clock.get().max(timer.anchor + through));
        let cursor = usize::try_from(request.after)
            .map_err(|_| TargetError::unavailable("periodic capture", "cursor overflow"))?;
        let records = timer
            .records
            .get(cursor..)
            .ok_or_else(|| TargetError::unavailable("periodic capture", "unknown cursor"))?
            .to_vec();
        Ok(ess_conformance::periodic::Observation {
            elapsed_ms: through,
            complete_through_ms: through,
            cursor: timer.records.len() as u64,
            records,
        })
    }
    fn close_periodic(
        &self,
        scope: ess_conformance::periodic::Scope,
    ) -> Result<ess_conformance::periodic::Closed, TargetError> {
        let mut timer = self.timer.borrow_mut();
        timer.active = false;
        while let Some((_, at)) = timer.busy {
            timer.advance::<MODE>(at, &self.namespace);
        }
        self.clock
            .set(self.clock.get().max(timer.anchor + timer.now));
        Ok(ess_conformance::periodic::Closed {
            scope,
            at_ms: timer.now,
        })
    }
}

/// The service's own grant table: which actor it admits to which command. Never read from a model.
const NOTE_GRANTS: &[(&str, &[&str])] = &[(
    "demo.notes.AccountUser",
    &["demo.notes.CreateNote", "demo.notes.EditNote"],
)];

/// An independent note service. It authenticates every command as the caller it is sent as,
/// keeps its own rows, decides an edit from the stored agent and refuses actors its own grant
/// table does not admit. Faults: 1 records the first caller's account for every caller; 2 never
/// refuses an edit; 3 lets only the first caller it ever served edit; 4 cannot authenticate as a
/// caller; 5 runs a command for an actor it does not grant.
pub struct NotesInstallation<const MODE: u8>;
/// Caller-scoped note rows of one installed run.
pub struct NotesTarget<const MODE: u8> {
    active: RefCell<Option<String>>,
    notes: RefCell<BTreeMap<String, ViewRow>>,
    events: RefCell<Vec<ObservedEvent>>,
    minted: Cell<u64>,
    first: RefCell<Option<BTreeMap<String, Node>>>,
}
impl<const MODE: u8> Installation for NotesInstallation<MODE> {
    type Target = NotesTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        _: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        Ok(Installed {
            target: NotesTarget {
                active: RefCell::new(None),
                notes: RefCell::new(BTreeMap::new()),
                events: RefCell::new(Vec::new()),
                minted: Cell::new(0),
                first: RefCell::new(None),
            },
            clock: SharedClock(Rc::new(Cell::new(1_767_603_600_000))),
            config: RunnerConfig::new(500),
        })
    }
}
impl<const MODE: u8> NotesTarget<MODE> {
    fn granted(actor: Option<&str>, command: &str) -> bool {
        NOTE_GRANTS
            .iter()
            .any(|(name, commands)| Some(*name) == actor && commands.contains(&command))
    }
}
impl<const MODE: u8> ConformanceTarget for NotesTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "independent-browser-notes",
            "1",
        ))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.notes.borrow_mut().clear();
        self.events.borrow_mut().clear();
        *self.active.borrow_mut() = Some(context.correlation.to_string());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.active.borrow_mut() = None;
        Ok(())
    }
    #[allow(clippy::too_many_lines)]
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if self.active.borrow().as_deref() != Some(request.correlation.to_string().as_str()) {
            return Err(TargetError::unavailable(
                "context",
                "inactive isolated scenario",
            ));
        }
        let command = request.command.to_string();
        let actor = request.actor.as_ref().map(ToString::to_string);
        if MODE != 5 && !Self::granted(actor.as_deref(), &command) {
            return Err(TargetError::not_granted(actor));
        }
        if MODE == 4 {
            return Err(TargetError::unsupported(
                "sending a command as a caller",
                "this installation holds one credential",
            ));
        }
        let caller = match (request.caller.clone(), MODE) {
            (Some(caller), _) => caller,
            (None, 5) => BTreeMap::new(),
            (None, _) => {
                return Err(TargetError::unavailable(
                    "authenticating",
                    "this service authenticates every command as a caller",
                ))
            }
        };
        let first = self
            .first
            .borrow_mut()
            .get_or_insert_with(|| caller.clone())
            .clone();
        let serial = self.minted.get() + 1;
        self.minted.set(serial);
        let token =
            ess_primitives::consistency::ConsistencyToken::new(format!("notes-{serial}")).unwrap();
        let attribute = |caller: &BTreeMap<String, Node>, name: &str| {
            caller.get(name).cloned().unwrap_or(Node::Null)
        };
        let text = request.input.get("text").cloned().unwrap_or(Node::Null);
        let mut notes = self.notes.borrow_mut();
        let result = match command.as_str() {
            "demo.notes.CreateNote" => {
                let id = format!("00000000-0000-4000-8000-{serial:012}");
                let account = attribute(if MODE == 1 { &first } else { &caller }, "account_id");
                notes.insert(
                    id.clone(),
                    BTreeMap::from([
                        ("note_id".into(), Node::Text(id.clone())),
                        ("account_id".into(), account.clone()),
                        ("agent_id".into(), attribute(&caller, "agent_id")),
                        ("text".into(), text.clone()),
                        ("state".into(), Node::Text("Open".into())),
                    ]),
                );
                let event = ObservedEvent::new("demo.notes.NoteCreated".parse().unwrap())
                    .with("note_id", Node::Text(id))
                    .with("account_id", account)
                    .with("text", text)
                    .in_activity(request.correlation)
                    .at(serial);
                self.events.borrow_mut().push(event.clone());
                SemanticCommandResult::took(OutcomeRef::new(
                    request.command,
                    "created".parse().unwrap(),
                ))
                .emitting(event)
            }
            "demo.notes.EditNote" => {
                let Some(Node::Text(id)) = request.input.get("note_id").cloned() else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let Some(row) = notes.get_mut(&id) else {
                    return Ok(SemanticCommandResult::undeclared().with_consistency(token));
                };
                let refused = match MODE {
                    2 => false,
                    3 => caller != first,
                    _ => row.get("agent_id") != caller.get("agent_id"),
                };
                if refused {
                    SemanticCommandResult::took(OutcomeRef::new(
                        request.command,
                        "forbidden".parse().unwrap(),
                    ))
                    .with_error(DeclaredErrorValue::new(
                        "demo.notes.NotYourNote".parse().unwrap(),
                    ))
                } else {
                    row.insert("text".into(), text.clone());
                    let event = ObservedEvent::new("demo.notes.NoteEdited".parse().unwrap())
                        .with("note_id", Node::Text(id))
                        .with("text", text)
                        .in_activity(request.correlation)
                        .at(serial);
                    self.events.borrow_mut().push(event.clone());
                    SemanticCommandResult::took(OutcomeRef::new(
                        request.command,
                        "edited".parse().unwrap(),
                    ))
                    .emitting(event)
                }
            }
            _ => {
                return Err(TargetError::unsupported(
                    "command",
                    "not this service's API",
                ))
            }
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        if request.view.to_string() != "demo.notes.NoteDetails" {
            return Err(TargetError::unsupported("view", "not this service's API"));
        }
        Ok(SemanticViewResult::of(
            self.notes.borrow().values().cloned(),
        ))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|event| {
                event.event == request.event
                    && event.correlation.as_ref() == Some(&request.correlation)
            })
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "not this service's API",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivery",
            "not this service's API",
        ))
    }
}

/// The record service's own declared contract: entity, identity kind, required fields.
#[derive(Clone, Copy, PartialEq, Eq)]
enum IdentityKind {
    Text,
    Integer,
    Uuid,
}
const RECORD_ENTITIES: &[(&str, &str, IdentityKind)] = &[
    (
        "records.ids.TextRecord",
        "records.ids.TextRecords",
        IdentityKind::Text,
    ),
    (
        "records.ids.CountRecord",
        "records.ids.CountRecords",
        IdentityKind::Integer,
    ),
    (
        "records.ids.UuidRecord",
        "records.ids.UuidRecords",
        IdentityKind::Uuid,
    ),
];
/// Independently established upstream rows whose identities spell alike across entities.
/// It validates each setup against its own contract and stores rows per entity. Faults:
/// 1 acknowledges a setup without storing it; 2 stores a different amount; 3 keeps an Integer
/// identity as its text; 4 keys every entity's rows in one store by identity spelling.
pub struct SetupInstallation<const MODE: u8>;
/// Rows per entity, or (fault 4) one store keyed by identity spelling.
pub struct SetupTarget<const MODE: u8> {
    rows: RefCell<BTreeMap<(String, String), ViewRow>>,
    active: RefCell<Option<String>>,
}
impl<const MODE: u8> Installation for SetupInstallation<MODE> {
    type Target = SetupTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        _: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        Ok(Installed {
            target: SetupTarget {
                rows: RefCell::new(BTreeMap::new()),
                active: RefCell::new(None),
            },
            clock: SharedClock(Rc::new(Cell::new(1_767_603_600_000))),
            config: RunnerConfig::new(500),
        })
    }
}
fn spelled(identity: &Node) -> String {
    match identity {
        Node::Text(text) => text.clone(),
        Node::Number(number) => number.to_string(),
        other => format!("{other:?}"),
    }
}
impl<const MODE: u8> ConformanceTarget for SetupTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "independent-browser-records",
            "1",
        ))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.borrow_mut().clear();
        *self.active.borrow_mut() = Some(context.correlation.to_string());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.active.borrow_mut() = None;
        Ok(())
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        if self.active.borrow().as_deref() != Some(request.correlation.to_string().as_str()) {
            return Err(TargetError::unavailable(
                "entity setup",
                "inactive isolated scenario",
            ));
        }
        let entity = request.entity.to_string();
        let Some((_, _, kind)) = RECORD_ENTITIES.iter().find(|(name, ..)| *name == entity) else {
            return Err(TargetError::unsupported(
                "entity setup",
                "not this service's entity",
            ));
        };
        let valid_identity = match (kind, &request.identity) {
            (IdentityKind::Integer, Node::Number(_)) | (IdentityKind::Text, Node::Text(_)) => true,
            (IdentityKind::Uuid, Node::Text(text)) => {
                text.len() == 36
                    && text.char_indices().all(|(at, c)| {
                        if matches!(at, 8 | 13 | 18 | 23) {
                            c == '-'
                        } else {
                            c.is_ascii_hexdigit() && !c.is_ascii_uppercase()
                        }
                    })
            }
            _ => false,
        };
        let amount_valid = matches!(request.fields.get("amount"), Some(Node::Number(n)) if !n.to_string().starts_with('-'));
        let known = request
            .fields
            .keys()
            .all(|field| matches!(field.as_str(), "amount" | "note"));
        if !valid_identity || !amount_valid || !known || request.state.to_string() != "Kept" {
            return Err(TargetError::unavailable(
                "entity setup",
                "the setup violates this service's record contract",
            ));
        }
        if MODE == 1 {
            return Ok(());
        }
        let mut row = request.fields.clone();
        let identity = if MODE == 3 && *kind == IdentityKind::Integer {
            Node::Text(spelled(&request.identity))
        } else {
            request.identity.clone()
        };
        if MODE == 2 {
            if let Some(Node::Number(amount)) = row.get("amount").cloned() {
                let changed: i64 = amount.to_string().parse::<i64>().unwrap() + 1;
                row.insert("amount".into(), Node::Number(Number::from(changed)));
            }
        }
        row.insert("record_id".into(), identity);
        row.insert("entity".into(), Node::Text(entity.clone()));
        let key = if MODE == 4 {
            (String::new(), spelled(&request.identity))
        } else {
            (entity, spelled(&request.identity))
        };
        self.rows.borrow_mut().insert(key, row);
        Ok(())
    }
    fn execute_command(
        &self,
        _: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        Err(TargetError::unsupported(
            "command",
            "this service accepts no commands",
        ))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let view = request.view.to_string();
        let Some((entity, ..)) = RECORD_ENTITIES.iter().find(|(_, name, _)| *name == view) else {
            return Err(TargetError::unsupported("view", "not this service's API"));
        };
        Ok(SemanticViewResult::of(
            self.rows
                .borrow()
                .values()
                .filter(|row| row.get("entity") == Some(&Node::Text((*entity).into())))
                .map(|row| {
                    row.iter()
                        .filter(|(field, _)| matches!(field.as_str(), "record_id" | "amount"))
                        .map(|(field, value)| (field.clone(), value.clone()))
                        .collect()
                }),
        ))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "not this service's API",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivery",
            "not this service's API",
        ))
    }
}

/// The run namespace as `Ids::seeded` spells it: ASCII alphanumerics and single hyphens.
fn correlation_prefix(namespace: &str) -> String {
    let mut prefix = String::new();
    for character in namespace.chars() {
        if character.is_ascii_alphanumeric() {
            prefix.push(character);
        } else if !prefix.ends_with('-') {
            prefix.push('-');
        }
    }
    format!("{}-", prefix.trim_matches('-'))
}

/// An independent job scheduler deciding `starts_at < now - 60s` from its own clock. Healthy, its
/// clock is the one the runner reads its wall time from. Faults: 1 decides against a historical
/// epoch of its own (2020-01-01); 2 runs one hour ahead of the runner's wall clock.
pub struct JobsInstallation<const MODE: u8>;
/// Job rows and the shared logical clock of one installed run.
pub struct JobsTarget<const MODE: u8> {
    clock: Rc<Cell<u64>>,
    active: RefCell<Option<String>>,
    jobs: RefCell<Vec<ViewRow>>,
    events: RefCell<Vec<ObservedEvent>>,
    minted: Cell<u64>,
}
impl<const MODE: u8> Installation for JobsInstallation<MODE> {
    type Target = JobsTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        _: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        let clock = Rc::new(Cell::new(1_767_603_600_000));
        Ok(Installed {
            target: JobsTarget {
                clock: Rc::clone(&clock),
                active: RefCell::new(None),
                jobs: RefCell::new(Vec::new()),
                events: RefCell::new(Vec::new()),
                minted: Cell::new(0),
            },
            clock: SharedClock(clock),
            config: RunnerConfig::new(500),
        })
    }
}
impl<const MODE: u8> ConformanceTarget for JobsTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("independent-browser-jobs", "1"))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        self.jobs.borrow_mut().clear();
        self.events.borrow_mut().clear();
        *self.active.borrow_mut() = Some(context.correlation.to_string());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.active.borrow_mut() = None;
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        use ess_primitives::time::Rfc3339Instant;
        if self.active.borrow().as_deref() != Some(request.correlation.to_string().as_str()) {
            return Err(TargetError::unavailable(
                "context",
                "inactive isolated scenario",
            ));
        }
        if request.command.to_string() != "demo.jobs.ScheduleJob" {
            return Err(TargetError::unsupported(
                "command",
                "not this service's API",
            ));
        }
        let actor = request.actor.as_ref().map(ToString::to_string);
        if actor.as_deref() != Some("demo.jobs.Clerk") {
            return Err(TargetError::not_granted(actor));
        }
        let Some(Node::Text(sent)) = request.input.get("starts_at") else {
            return Err(TargetError::unavailable(
                "ScheduleJob",
                "starts_at is not a timestamp",
            ));
        };
        let starts = Rfc3339Instant::parse_rfc3339(sent)
            .ok_or_else(|| TargetError::unavailable("ScheduleJob", "starts_at is not RFC 3339"))?;
        let now_ms = match MODE {
            1 => 1_577_836_800_000,
            2 => self.clock.get() + 3_600_000,
            _ => self.clock.get(),
        };
        let now = Rfc3339Instant::from_epoch_millis(i64::try_from(now_ms).unwrap()).unwrap();
        let serial = self.minted.get() + 1;
        self.minted.set(serial);
        let token =
            ess_primitives::consistency::ConsistencyToken::new(format!("jobs-{serial}")).unwrap();
        if starts < now.plus_seconds(-60).unwrap() {
            return Ok(SemanticCommandResult::took(OutcomeRef::new(
                request.command,
                "start-in-past".parse().unwrap(),
            ))
            .with_error(DeclaredErrorValue::new(
                "demo.jobs.StartInPast".parse().unwrap(),
            ))
            .with_consistency(token));
        }
        let id = Node::Text(format!("00000000-0000-4000-8000-{serial:012}"));
        self.jobs.borrow_mut().push(BTreeMap::from([
            ("job_id".into(), id.clone()),
            ("starts_at".into(), Node::Text(sent.clone())),
        ]));
        let event = ObservedEvent::new("demo.jobs.JobScheduled".parse().unwrap())
            .with("job_id", id)
            .with("starts_at", Node::Text(sent.clone()))
            .in_activity(request.correlation)
            .at(serial);
        self.events.borrow_mut().push(event.clone());
        Ok(SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            "scheduled".parse().unwrap(),
        ))
        .emitting(event)
        .with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        if request.view.to_string() != "demo.jobs.JobDetails" {
            return Err(TargetError::unsupported("view", "not this service's API"));
        }
        Ok(SemanticViewResult::of(self.jobs.borrow().iter().cloned()))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|event| {
                event.event == request.event
                    && event.correlation.as_ref() == Some(&request.correlation)
            })
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "not this service's API",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivery",
            "not this service's API",
        ))
    }
}

std::thread_local! {
    /// A store that outlives each installed run, as a deployed backend does. Healthy rows are
    /// keyed by the runner's namespaced scenario correlation; fault 1 keys them by scenario id.
    static DURABLE_ROWS: RefCell<BTreeMap<String, BTreeMap<String, ViewRow>>> =
        const { RefCell::new(BTreeMap::new()) };
}
/// A creation service over durable storage that isolates by the run namespace. Fault 1 isolates
/// by scenario name instead, so a repeated run in the same worker reads its earlier rows.
pub struct IsolatedStoreInstallation<const MODE: u8>;
/// The run namespace and the active durable partition.
pub struct IsolatedStoreTarget<const MODE: u8> {
    prefix: String,
    partition: RefCell<Option<(String, String)>>,
    events: RefCell<Vec<ObservedEvent>>,
}
impl<const MODE: u8> Installation for IsolatedStoreInstallation<MODE> {
    type Target = IsolatedStoreTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        context: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        Ok(Installed {
            target: IsolatedStoreTarget {
                prefix: correlation_prefix(&context.namespace),
                partition: RefCell::new(None),
                events: RefCell::new(Vec::new()),
            },
            clock: SharedClock(Rc::new(Cell::new(1_767_603_600_000))),
            config: RunnerConfig::new(500),
        })
    }
}
impl<const MODE: u8> ConformanceTarget for IsolatedStoreTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "independent-browser-durable-store",
            "1",
        ))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        let correlation = context.correlation.to_string();
        // The runner's ids and this installation's namespace must be the same namespace.
        if !correlation.starts_with(&self.prefix) {
            return Err(TargetError::unavailable(
                "isolation",
                "scenario correlation outside the installed run namespace",
            ));
        }
        let partition = if MODE == 1 {
            context.scenario.to_string()
        } else {
            correlation.clone()
        };
        *self.partition.borrow_mut() = Some((partition, correlation));
        self.events.borrow_mut().clear();
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.partition.borrow_mut() = None;
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let Some((partition, correlation)) = self.partition.borrow().clone() else {
            return Err(TargetError::unavailable("context", "no isolated scenario"));
        };
        if request.command.to_string() != "demo.response.Create"
            || correlation != request.correlation.to_string()
        {
            return Err(TargetError::unsupported(
                "command",
                "inactive context or unknown API",
            ));
        }
        let serial = DURABLE_ROWS.with(|rows| {
            let mut rows = rows.borrow_mut();
            let rows = rows.entry(partition).or_default();
            let serial = rows.len() + 1;
            let id = format!("00000000-0000-4000-8000-{serial:012}");
            rows.insert(id.clone(), BTreeMap::from([("id".into(), Node::Text(id))]));
            serial as u64
        });
        let id = format!("00000000-0000-4000-8000-{serial:012}");
        let event = ObservedEvent::new("demo.response.Created".parse().unwrap())
            .with("id", Node::Text(id.clone()))
            .in_activity(request.correlation)
            .at(serial);
        self.events.borrow_mut().push(event.clone());
        let mut result = SemanticCommandResult::took(OutcomeRef::new(
            request.command,
            "created".parse().unwrap(),
        ));
        result.response = Some(BTreeMap::from([("id".into(), Node::Text(id))]));
        result.direct_events.push(event);
        result.consistency = Some(
            ess_primitives::consistency::ConsistencyToken::new(format!("durable-{serial}"))
                .unwrap(),
        );
        Ok(result)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        if request.view.to_string() != "demo.response.CreatedRows" {
            return Err(TargetError::unsupported("view", "unknown API"));
        }
        let Some((partition, _)) = self.partition.borrow().clone() else {
            return Err(TargetError::unavailable("view", "no isolated scenario"));
        };
        Ok(SemanticViewResult::of(DURABLE_ROWS.with(|rows| {
            rows.borrow()
                .get(&partition)
                .map(|rows| rows.values().cloned().collect::<Vec<_>>())
                .unwrap_or_default()
        })))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|event| {
                event.event == request.event
                    && event.correlation.as_ref() == Some(&request.correlation)
            })
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "not this service's API",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivery",
            "not this service's API",
        ))
    }
}

/// An independent session service with a fixture provider of its own. The provider resolves
/// values for the scenario the runner names, in the installed namespace, before the scenario
/// opens; the service accepts only principals of its own namespace. Fault 1 provisions under a
/// shared namespace instead of the run's.
pub struct FixtureNamespaceInstallation<const MODE: u8>;
/// The run namespace and the scenario the provider resolved values for.
pub struct FixtureNamespaceTarget<const MODE: u8> {
    prefix: String,
    provisioned: RefCell<Option<String>>,
    active: RefCell<Option<String>>,
}
impl<const MODE: u8> FixtureNamespaceTarget<MODE> {
    fn email(namespace: &str) -> String {
        format!(
            "principal@{}.fixtures.test",
            namespace.trim_end_matches('-')
        )
    }
}
impl<const MODE: u8> Installation for FixtureNamespaceInstallation<MODE> {
    type Target = FixtureNamespaceTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        context: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        FACTORIES.fetch_add(1, Ordering::Relaxed);
        Ok(Installed {
            target: FixtureNamespaceTarget {
                prefix: correlation_prefix(&context.namespace),
                provisioned: RefCell::new(None),
                active: RefCell::new(None),
            },
            clock: SharedClock(Rc::new(Cell::new(1_767_603_600_000))),
            config: RunnerConfig::new(500),
        })
    }
}
impl<const MODE: u8> ConformanceTarget for FixtureNamespaceTarget<MODE> {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new(
            "independent-browser-fixtures",
            "1",
        ))
    }
    fn fixture_values(
        &self,
        scenario: &ScenarioContext,
        _: &ess_conformance::fixtures::Contract,
    ) -> Result<BTreeMap<String, Node>, TargetError> {
        let correlation = scenario.correlation.to_string();
        if !correlation.starts_with(&self.prefix) {
            return Err(TargetError::unavailable(
                "fixture values",
                "scenario correlation outside the installed run namespace",
            ));
        }
        if self.active.borrow().is_some() {
            return Err(TargetError::unavailable(
                "fixture values",
                "provisioning after the scenario opened",
            ));
        }
        *self.provisioned.borrow_mut() = Some(correlation);
        let namespace = if MODE == 1 {
            "shared-"
        } else {
            self.prefix.as_str()
        };
        Ok(BTreeMap::from([
            (
                "current-principal-id".into(),
                Node::Text("d248c830-37a4-466b-8319-af71667f1364".into()),
            ),
            (
                "current-principal-email".into(),
                Node::Text(Self::email(namespace)),
            ),
        ]))
    }
    fn begin_scenario(&self, context: &ScenarioContext) -> Result<(), TargetError> {
        let correlation = context.correlation.to_string();
        if self.provisioned.borrow().as_deref() != Some(correlation.as_str()) {
            return Err(TargetError::unavailable(
                "isolation",
                "fixtures were not provisioned for this scenario before it opened",
            ));
        }
        *self.active.borrow_mut() = Some(correlation);
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        *self.active.borrow_mut() = None;
        *self.provisioned.borrow_mut() = None;
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        if request.command.to_string() != "fixturetest.session.Open"
            || self.active.borrow().as_deref() != Some(request.correlation.to_string().as_str())
        {
            return Err(TargetError::unsupported(
                "command",
                "inactive context or unknown API",
            ));
        }
        // The service opens a principal's session only inside that principal's namespace; any
        // other principal gets an anonymous session.
        let own = Self::email(&self.prefix);
        let email = match request.input.get("email") {
            Some(Node::Text(email)) if *email == own => email.clone(),
            _ => "anonymous@fixtures.test".into(),
        };
        let event = ObservedEvent::new("fixturetest.session.Opened".parse().unwrap())
            .with(
                "principal_id",
                request
                    .input
                    .get("principal_id")
                    .cloned()
                    .unwrap_or(Node::Null),
            )
            .with("email", Node::Text(email))
            .with(
                "region",
                request.input.get("region").cloned().unwrap_or(Node::Null),
            )
            .in_activity(request.correlation);
        Ok(
            SemanticCommandResult::took(OutcomeRef::new(
                request.command,
                "opened".parse().unwrap(),
            ))
            .emitting(event),
        )
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Err(TargetError::unsupported("view", "not this service's API"))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "external",
            "not this service's API",
        ))
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        Err(TargetError::unsupported(
            "redelivery",
            "not this service's API",
        ))
    }
}

/// The healthy response service, holding the worker for a while inside every command so a test
/// can navigate, select or abort while the synchronous Runner is busy.
pub struct SlowInstallation<const MODE: u8>;
/// The healthy response service plus a bounded busy section per command.
pub struct SlowTarget<const MODE: u8> {
    inner: Backend<0>,
}
/// Busy iterations per command: long enough for a page to act while the worker is busy.
pub const SLOW_ITERATIONS: u64 = 3_000_000_000;
impl<const MODE: u8> Installation for SlowInstallation<MODE> {
    type Target = SlowTarget<MODE>;
    type Clock = SharedClock;
    fn create(
        context: &RunContext,
    ) -> ess_conformance::web_execution::Result<Installed<Self::Target, Self::Clock>> {
        let installed = BrowserInstallation::<0>::create(context)?;
        Ok(Installed {
            target: SlowTarget {
                inner: installed.target,
            },
            clock: installed.clock,
            config: installed.config,
        })
    }
}
impl<const MODE: u8> ConformanceTarget for SlowTarget<MODE> {
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
        let mut total = 0_u64;
        for step in 0..SLOW_ITERATIONS {
            total = total.wrapping_add(std::hint::black_box(step));
        }
        std::hint::black_box(total);
        self.inner.execute_command(request)
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
        request: ExternalOutcomeControl,
    ) -> Result<(), TargetError> {
        self.inner.configure_external_outcome(request)
    }
    fn redeliver_event(&self, request: RedeliveryRequest) -> Result<(), TargetError> {
        self.inner.redeliver_event(request)
    }
}
