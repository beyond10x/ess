use super::*;
use ess_compiler::protocol::CompiledProtocol;

#[test]
fn empty_trace_is_not_conformance_evidence() {
    let model = fixture();
    let trace = Trace {
        format: TRACE_FORMAT.into(),
        model_digest: model.digest().into(),
        origin: TraceOrigin::Model,
        steps: vec![],
        complete: true,
    };
    assert_eq!(check_trace(&model, &trace).verdict, Verdict::Inconclusive);
}

fn fixture() -> CompiledProtocol {
    ess_compiler::protocol::parse_and_compile(r"
format: ess-protospec/1
name: terminal
participants:
  - name: server
    initial: open
    states: [open, closed]
    inputs: [{name: finish, fields: []}]
    transitions:
      - name: finish
        from: open
        to: closed
        trigger: {kind: input, name: finish}
        effects:
          - {kind: send, channel: wire, message: response, exchange: {kind: literal, value: x}, logical: {kind: literal, value: final}, payload: {}}
          - {kind: flush, channel: wire}
          - {kind: close}
  - name: client
    initial: waiting
    states: [waiting, done]
    transitions:
      - name: receive
        from: waiting
        to: done
        trigger: {kind: receive, channel: wire, message: response}
messages: [{name: response, fields: []}]
channels: [{name: wire, from: server, to: client, ordering: fifo, capacity: 2, loss: false, duplication: false}]
properties: [{kind: flush_before_close, name: flush, participant: server, channel: wire}]
bounds: {max_steps: 10, max_states: 30, max_time_ms: 100}
").unwrap()
}

fn input(participant: &str, name: &str) -> Action {
    Action::Input {
        participant: participant.into(),
        name: name.into(),
        payload: BTreeMap::new(),
    }
}
fn document(name: &str) -> CompiledProtocol {
    let text = std::fs::read_to_string(format!(
        "{}/../../../examples/protocols/{name}.yaml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap();
    ess_compiler::protocol::parse_and_compile(&text).unwrap()
}
fn actions(name: &str) -> Vec<Action> {
    serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/../../../examples/protocols/{name}.actions.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap()
}
#[test]
fn terminal_and_lost_ack_examples_replay_and_mutants_fail() {
    for name in ["terminal-response", "lost-ack"] {
        let model = document(name);
        let mut trace = simulate(&model, &actions(name)).unwrap();
        assert_eq!(
            check_trace(&model, &trace).verdict,
            Verdict::Passed,
            "{name}"
        );
        let step = trace
            .steps
            .iter_mut()
            .find(|s| {
                s.observations
                    .iter()
                    .any(|o| matches!(o, Observation::Emitted { .. }))
            })
            .unwrap();
        step.observations.push(Observation::Emitted {
            participant: "client".into(),
            name: "final-response".into(),
        });
        assert_eq!(check_trace(&model, &trace).verdict, Verdict::Failed);
    }
}
#[test]
fn initial_property_violation_is_not_hidden_by_first_transition() {
    let mut raw = fixture().model().clone();
    raw.properties
        .push(ess_domain::protocol::Property::NeverState {
            name: "bad-initial".into(),
            participant: "server".into(),
            state: "open".into(),
        });
    let model = ess_compiler::protocol::compile(raw).unwrap();
    let report = explore(&model, &[]);
    assert_eq!(report.verdict, Verdict::Failed);
    assert_eq!(report.explored_states, 1);
}
#[test]
fn deadline_cannot_be_crossed_and_stale_timer_cannot_fire() {
    let model = document("lost-ack");
    let mut state = initial(&model);
    for action in actions("lost-ack").iter().take(5) {
        state = successors(&model, &state, action)
            .unwrap()
            .remove(0)
            .configuration;
    }
    assert!(successors(&model, &state, &Action::AdvanceTo { millis: 501 }).is_err());
    assert!(successors(
        &model,
        &state,
        &Action::Fire {
            participant: "server".into(),
            timer: "G".into(),
            generation: 0
        }
    )
    .is_err());
    let step = successors(
        &model,
        &state,
        &Action::Fire {
            participant: "server".into(),
            timer: "G".into(),
            generation: 1,
        },
    )
    .unwrap()
    .remove(0);
    assert_eq!(
        step.configuration.peers["server"].timers["G"].deadline_ms,
        1500
    );
    assert_eq!(step.configuration.peers["server"].timers["G"].generation, 2);
}
#[test]
fn trace_digest_completeness_unknown_fields_and_duplicates_are_checked() {
    let model = fixture();
    let mut trace = simulate(
        &model,
        &[
            input("server", "finish"),
            Action::Deliver { transmission: 1 },
        ],
    )
    .unwrap();
    assert_eq!(check_trace(&model, &trace).verdict, Verdict::Passed);
    trace.complete = false;
    assert_eq!(check_trace(&model, &trace).verdict, Verdict::Inconclusive);
    trace.model_digest = "wrong".into();
    assert_eq!(check_trace(&model, &trace).verdict, Verdict::Failed);
    assert!(serde_json::from_str::<Action>(
        r#"{"kind":"input","participant":"x","name":"y","payload":{"x":1,"x":2}}"#
    )
    .is_err());
    assert!(
        serde_json::from_str::<Action>(r#"{"kind":"advance_to","millis":1,"unknown":true}"#)
            .is_err()
    );
}

// Independent adapter fixture. It implements a small transport/application boundary without
// invoking the model executor or reading model effects to manufacture its observations.
struct TerminalTarget {
    early_close: bool,
    state: u8,
    closed: bool,
}
impl ProtocolTarget for TerminalTarget {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            implementation: "handwritten-terminal-fixture/1".into(),
            all_actions: true,
            complete_observations: true,
        }
    }
    fn open(&mut self, _: &str) -> Result<(), String> {
        self.state = 0;
        Ok(())
    }
    fn drive(&mut self, action: &Action) -> Result<Vec<Observation>, String> {
        match (self.state, action) {
            (
                0,
                Action::Input {
                    participant, name, ..
                },
            ) if participant == "server" && name == "finish" => {
                self.state = 1;
                let sent = Observation::Sent {
                    message: Transmission {
                        id: 1,
                        original: 1,
                        channel: "wire".into(),
                        message: "response".into(),
                        exchange: "x".into(),
                        logical: "final".into(),
                        payload: BTreeMap::new(),
                    },
                };
                let flush = Observation::Flushed {
                    participant: "server".into(),
                    channel: "wire".into(),
                };
                let close = Observation::Closed {
                    participant: "server".into(),
                };
                Ok(if self.early_close {
                    vec![sent, close, flush]
                } else {
                    vec![sent, flush, close]
                })
            }
            (1, Action::Deliver { transmission: 1 }) => {
                self.state = 2;
                Ok(vec![Observation::Delivered { transmission: 1 }])
            }
            _ => Err("invalid fixture action".into()),
        }
    }
    fn close(&mut self) -> Result<bool, String> {
        self.closed = true;
        Ok(self.state == 2)
    }
}
#[test]
fn independent_target_checks_actual_flush_order_and_always_cleans_up() {
    let model = fixture();
    let actions = [
        input("server", "finish"),
        Action::Deliver { transmission: 1 },
    ];
    for (early_close, verdict) in [(false, Verdict::Passed), (true, Verdict::Failed)] {
        let mut target = TerminalTarget {
            early_close,
            state: 0,
            closed: false,
        };
        assert_eq!(run_target(&model, &actions, &mut target).verdict, verdict);
        assert!(target.closed);
    }
    let mut target = TerminalTarget {
        early_close: false,
        state: 0,
        closed: false,
    };
    assert_eq!(
        run_target(&model, &[input("server", "wrong")], &mut target).verdict,
        Verdict::Inconclusive
    );
    assert!(target.closed);
}

#[test]
fn target_admission_errors_are_inconclusive_and_keep_origin() {
    struct UnavailableTarget(Capabilities);
    impl ProtocolTarget for UnavailableTarget {
        fn capabilities(&self) -> Capabilities {
            self.0.clone()
        }
        fn open(&mut self, _: &str) -> Result<(), String> {
            panic!("unadmitted target must not open")
        }
        fn drive(&mut self, _: &Action) -> Result<Vec<Observation>, String> {
            panic!("unadmitted target must not drive")
        }
        fn close(&mut self) -> Result<bool, String> {
            panic!("unopened target must not close")
        }
    }
    for (implementation, all_actions, complete_observations) in [
        ("", true, true),
        ("fixture", false, true),
        ("fixture", true, false),
    ] {
        let mut target = UnavailableTarget(Capabilities {
            implementation: implementation.into(),
            all_actions,
            complete_observations,
        });
        let report = run_target(&fixture(), &[input("server", "finish")], &mut target);
        assert_eq!(report.verdict, Verdict::Inconclusive);
        assert_eq!(
            report.origin,
            Some(TraceOrigin::Target {
                implementation: implementation.into(),
            })
        );
    }
}

#[test]
fn replay_keeps_both_nondeterministic_states_until_observation_disambiguates() {
    let mut raw = fixture().model().clone();
    let server = &mut raw.participants[0];
    server.states.push("alternate".into());
    let mut branch = server.transitions[0].clone();
    branch.name = "alternate".into();
    branch.to = "alternate".into();
    // Same observations, distinct local states. Neither branch can be thrown away.
    server.transitions.push(branch);
    let model = ess_compiler::protocol::compile(raw).unwrap();
    let mut state = initial(&model);
    let action = input("server", "finish");
    let next = successors(&model, &state, &action).unwrap();
    assert_eq!(next.len(), 2);
    let mut trace = Trace {
        format: TRACE_FORMAT.into(),
        model_digest: model.digest().into(),
        origin: TraceOrigin::Model,
        complete: true,
        steps: vec![TraceStep {
            action,
            observations: next[0].observations.clone(),
        }],
    };
    state = next[0].configuration.clone();
    let action = Action::Deliver { transmission: 1 };
    trace.steps.push(TraceStep {
        observations: successors(&model, &state, &action).unwrap()[0]
            .observations
            .clone(),
        action,
    });
    let report = check_trace(&model, &trace);
    assert_eq!(report.verdict, Verdict::Passed);
    assert_eq!(report.surviving_states, 2);
    assert!(simulate(&model, &[input("server", "finish")]).is_err());
}

#[test]
fn exploration_reports_bound_exhaustion_and_replayable_counterexample() {
    let mut raw = fixture().model().clone();
    raw.bounds.max_states = 1;
    let model = ess_compiler::protocol::compile(raw).unwrap();
    assert_eq!(explore(&model, &[]).verdict, Verdict::Inconclusive);
    let mut raw = fixture().model().clone();
    raw.participants[0].transitions[0].effects.remove(1); // Missing transport flush.
    let model = ess_compiler::protocol::compile(raw).unwrap();
    let result = explore(&model, &[]);
    assert_eq!(result.verdict, Verdict::Failed);
    let trace = result.counterexample.unwrap();
    assert_eq!(trace.steps.len(), 1);
    assert_eq!(check_trace(&model, &trace).verdict, Verdict::Failed);
}

#[test]
fn fifo_loss_duplication_and_payload_admission_are_enforced() {
    let model = fixture();
    let state = successors(&model, &initial(&model), &input("server", "finish"))
        .unwrap()
        .remove(0)
        .configuration;
    for action in [
        Action::Drop { transmission: 1 },
        Action::Duplicate { transmission: 1 },
        Action::Deliver { transmission: 99 },
    ] {
        assert!(successors(&model, &state, &action).is_err());
    }
    let mut raw = model.model().clone();
    raw.channels[0].duplication = true;
    raw.channels[0].loss = true;
    let model = ess_compiler::protocol::compile(raw).unwrap();
    let state = successors(&model, &state, &Action::Duplicate { transmission: 1 })
        .unwrap()
        .remove(0)
        .configuration;
    assert_eq!(state.queue[1].original, 1);
    assert_eq!(state.queue[1].id, 2);
    assert!(successors(&model, &state, &Action::Deliver { transmission: 2 }).is_err());
    assert!(
        successors(&model, &state, &Action::Duplicate { transmission: 1 })
            .unwrap_err()
            .bound
    );
    let state = successors(&model, &state, &Action::Drop { transmission: 1 })
        .unwrap()
        .remove(0)
        .configuration;
    assert!(successors(&model, &state, &Action::Deliver { transmission: 2 }).is_ok());
}

struct AckTarget {
    phase: usize,
    duplicate_notification: bool,
    closed: bool,
}
fn sent(id: u64, channel: &str, message: &str, logical: &str) -> Observation {
    Observation::Sent {
        message: Transmission {
            id,
            original: id,
            channel: channel.into(),
            message: message.into(),
            exchange: "invite-1".into(),
            logical: logical.into(),
            payload: if message == "rejected" {
                BTreeMap::from([("status".into(), serde_json::from_str("486").unwrap())])
            } else {
                BTreeMap::new()
            },
        },
    }
}
fn armed(participant: &str, timer: &str, generation: u64, deadline_ms: u64) -> Observation {
    Observation::TimerArmed {
        participant: participant.into(),
        timer: timer.into(),
        generation,
        deadline_ms,
    }
}
fn fired(participant: &str, timer: &str) -> Observation {
    Observation::TimerFired {
        participant: participant.into(),
        timer: timer.into(),
        generation: 1,
    }
}
impl ProtocolTarget for AckTarget {
    fn capabilities(&self) -> Capabilities {
        Capabilities {
            implementation: "handwritten-ack-fixture/1".into(),
            all_actions: true,
            complete_observations: true,
        }
    }
    fn open(&mut self, _: &str) -> Result<(), String> {
        self.phase = 0;
        Ok(())
    }
    fn drive(&mut self, action: &Action) -> Result<Vec<Observation>, String> {
        let observations = match (self.phase, action) {
            (
                0,
                Action::Input {
                    participant, name, ..
                },
            ) if participant == "client" && name == "invite" => {
                vec![sent(1, "requests", "invite", "request-1")]
            }
            (1, Action::Deliver { transmission: 1 }) => vec![
                Observation::Delivered { transmission: 1 },
                sent(2, "responses", "rejected", "final-1"),
                armed("server", "G", 1, 500),
                armed("server", "H", 1, 32000),
            ],
            (2, Action::Deliver { transmission: 2 }) => vec![
                Observation::Delivered { transmission: 2 },
                Observation::Emitted {
                    participant: "client".into(),
                    name: "final-response".into(),
                },
                sent(3, "requests", "ack", "ack-1"),
                armed("client", "D", 1, 32000),
            ],
            (3, Action::Drop { transmission: 3 }) => vec![Observation::Dropped { transmission: 3 }],
            (4, Action::AdvanceTo { millis: 500 }) => {
                vec![Observation::TimeAdvanced { millis: 500 }]
            }
            (
                5,
                Action::Fire {
                    participant,
                    timer,
                    generation: 1,
                },
            ) if participant == "server" && timer == "G" => vec![
                fired("server", "G"),
                sent(4, "responses", "rejected", "final-1"),
                armed("server", "G", 2, 1500),
            ],
            (6, Action::Deliver { transmission: 4 }) => {
                let mut observations = vec![Observation::Delivered { transmission: 4 }];
                if self.duplicate_notification {
                    observations.push(Observation::Emitted {
                        participant: "client".into(),
                        name: "final-response".into(),
                    });
                }
                observations.push(sent(5, "requests", "ack", "ack-1"));
                observations
            }
            (7, Action::Deliver { transmission: 5 }) => vec![
                Observation::Delivered { transmission: 5 },
                Observation::TimerCanceled {
                    participant: "server".into(),
                    timer: "G".into(),
                },
                Observation::TimerCanceled {
                    participant: "server".into(),
                    timer: "H".into(),
                },
                armed("server", "I", 1, 5500),
            ],
            (8, Action::AdvanceTo { millis: 5500 }) => {
                vec![Observation::TimeAdvanced { millis: 5500 }]
            }
            (
                9,
                Action::Fire {
                    participant,
                    timer,
                    generation: 1,
                },
            ) if participant == "server" && timer == "I" => vec![fired("server", "I")],
            (10, Action::AdvanceTo { millis: 32000 }) => {
                vec![Observation::TimeAdvanced { millis: 32000 }]
            }
            (
                11,
                Action::Fire {
                    participant,
                    timer,
                    generation: 1,
                },
            ) if participant == "client" && timer == "D" => vec![fired("client", "D")],
            _ => return Err("unexpected action in bounded ACK fixture".into()),
        };
        self.phase += 1;
        Ok(observations)
    }
    fn close(&mut self) -> Result<bool, String> {
        self.closed = true;
        Ok(self.phase == 12)
    }
}
#[test]
fn independent_ack_target_detects_duplicate_application_notification() {
    for (duplicate_notification, verdict) in [(false, Verdict::Passed), (true, Verdict::Failed)] {
        let mut target = AckTarget {
            phase: 0,
            duplicate_notification,
            closed: false,
        };
        assert_eq!(
            run_target(&document("lost-ack"), &actions("lost-ack"), &mut target).verdict,
            verdict
        );
        assert!(target.closed);
    }
}

#[test]
fn enabling_backoff_without_prior_interval_cannot_pass_exploration() {
    let mut raw = fixture().model().clone();
    let server = &mut raw.participants[0];
    server.timers.push("retry".into());
    server.transitions[0].effects.insert(
        0,
        ess_domain::protocol::Effect::Arm {
            timer: "retry".into(),
            after: ess_domain::protocol::Duration::Backoff {
                timer: "retry".into(),
                factor: 2,
                ceiling_ms: 100,
            },
        },
    );
    let model = ess_compiler::protocol::compile(raw).unwrap();
    assert_eq!(explore(&model, &[]).verdict, Verdict::Inconclusive);
}

#[test]
fn retransmission_backoff_caps_and_ack_cancels_both_server_timers() {
    let model = document("lost-ack");
    let mut state = initial(&model);
    for action in actions("lost-ack").iter().take(2) {
        state = successors(&model, &state, action)
            .unwrap()
            .remove(0)
            .configuration;
    }
    state = successors(&model, &state, &Action::Drop { transmission: 2 })
        .unwrap()
        .remove(0)
        .configuration;
    for (index, (now, deadline)) in [
        (500, 1500),
        (1500, 3500),
        (3500, 7500),
        (7500, 11500),
        (11500, 15500),
    ]
    .into_iter()
    .enumerate()
    {
        state = successors(&model, &state, &Action::AdvanceTo { millis: now })
            .unwrap()
            .remove(0)
            .configuration;
        state = successors(
            &model,
            &state,
            &Action::Fire {
                participant: "server".into(),
                timer: "G".into(),
                generation: u64::try_from(index + 1).unwrap(),
            },
        )
        .unwrap()
        .remove(0)
        .configuration;
        assert_eq!(state.peers["server"].timers["G"].deadline_ms, deadline);
        let transmission = state.queue[0].id;
        state = successors(&model, &state, &Action::Drop { transmission })
            .unwrap()
            .remove(0)
            .configuration;
    }
    let mut state = initial(&model);
    for action in actions("lost-ack").iter().take(8) {
        state = successors(&model, &state, action)
            .unwrap()
            .remove(0)
            .configuration;
    }
    assert_eq!(state.peers["server"].state, "confirmed");
    assert!(!state.peers["server"].timers.contains_key("G"));
    assert!(!state.peers["server"].timers.contains_key("H"));
    assert_eq!(state.peers["server"].timers["I"].deadline_ms, 5500);
    assert_eq!(state.peers["client"].timers["D"].deadline_ms, 32000);
    assert_eq!(state.peers["client"].events["final-response"], 1);
}

#[test]
fn exploration_refuses_invalid_input_witness_even_when_other_inputs_run() {
    assert_eq!(
        explore(&fixture(), &[input("server", "typo")]).verdict,
        Verdict::Inconclusive
    );
}

#[test]
fn future_timer_deadline_can_be_canceled_inside_exploration_horizon() {
    use ess_domain::protocol::{Duration, Effect, Message, Transition, Trigger};
    let mut raw = fixture().model().clone();
    let server = &mut raw.participants[0];
    server.states.push("waiting".into());
    server.timers.push("retry".into());
    server.transitions[0].to = "waiting".into();
    server.transitions[0].effects = vec![Effect::Arm {
        timer: "retry".into(),
        after: Duration::Fixed { millis: 100 },
    }];
    for name in ["rearm", "cancel"] {
        server.inputs.push(Message {
            name: name.into(),
            fields: vec![],
        });
    }
    server.transitions.push(Transition {
        name: "rearm".into(),
        from: "waiting".into(),
        to: "waiting".into(),
        trigger: Trigger::Input {
            name: "rearm".into(),
        },
        guards: vec![],
        effects: server.transitions[0].effects.clone(),
    });
    server.transitions.push(Transition {
        name: "cancel".into(),
        from: "waiting".into(),
        to: "closed".into(),
        trigger: Trigger::Input {
            name: "cancel".into(),
        },
        guards: vec![],
        effects: vec![
            Effect::Cancel {
                timer: "retry".into(),
            },
            Effect::Flush {
                channel: "wire".into(),
            },
            Effect::Close,
        ],
    });
    let model = ess_compiler::protocol::compile(raw).unwrap();
    let trace = simulate(
        &model,
        &[
            input("server", "finish"),
            Action::AdvanceTo { millis: 99 },
            input("server", "rearm"),
            input("server", "cancel"),
        ],
    )
    .unwrap();
    assert_eq!(check_trace(&model, &trace).verdict, Verdict::Passed);
    assert!(trace.steps[2]
        .observations
        .contains(&armed("server", "retry", 2, 199)));
}

#[test]
fn missing_ack_timeout_notifies_application_and_cancels_future_retry() {
    let model = document("lost-ack");
    let mut state = initial(&model);
    for action in actions("lost-ack").iter().take(2) {
        state = successors(&model, &state, action)
            .unwrap()
            .remove(0)
            .configuration;
    }
    state = successors(&model, &state, &Action::Drop { transmission: 2 })
        .unwrap()
        .remove(0)
        .configuration;
    for generation in 1..=10 {
        let millis = state.peers["server"].timers["G"].deadline_ms;
        state = successors(&model, &state, &Action::AdvanceTo { millis })
            .unwrap()
            .remove(0)
            .configuration;
        state = successors(
            &model,
            &state,
            &Action::Fire {
                participant: "server".into(),
                timer: "G".into(),
                generation,
            },
        )
        .unwrap()
        .remove(0)
        .configuration;
        let transmission = state.queue[0].id;
        state = successors(&model, &state, &Action::Drop { transmission })
            .unwrap()
            .remove(0)
            .configuration;
    }
    assert_eq!(state.peers["server"].timers["G"].deadline_ms, 35500);
    state = successors(&model, &state, &Action::AdvanceTo { millis: 32000 })
        .unwrap()
        .remove(0)
        .configuration;
    let step = successors(
        &model,
        &state,
        &Action::Fire {
            participant: "server".into(),
            timer: "H".into(),
            generation: 1,
        },
    )
    .unwrap()
    .remove(0);
    assert_eq!(step.configuration.peers["server"].state, "terminated");
    assert!(step.configuration.peers["server"].timers.is_empty());
    assert_eq!(
        step.observations,
        vec![
            fired("server", "H"),
            Observation::TimerCanceled {
                participant: "server".into(),
                timer: "G".into()
            },
            Observation::Emitted {
                participant: "server".into(),
                name: "transaction-failure".into()
            }
        ]
    );
}

#[test]
fn typed_input_guards_and_local_assignments_preserve_exchange_values_through_delivery() {
    use ess_domain::protocol::{Effect, Equality, Field, LocalField, ValueSource};
    let mut raw = fixture().model().clone();
    let field = Field {
        name: "id".into(),
        type_ref: ess_domain::TypeRef::parse("String").unwrap(),
    };
    raw.messages[0].fields.push(field.clone());
    raw.participants[0].inputs[0].fields.push(field.clone());
    for peer in &mut raw.participants {
        peer.fields.push(LocalField {
            name: field.name.clone(),
            type_ref: field.type_ref.clone(),
            initial: Node::Text("unset".into()),
        });
    }
    let server = &mut raw.participants[0].transitions[0];
    server.guards.push(Equality {
        left: ValueSource::Input { field: "id".into() },
        right: ValueSource::Literal {
            value: Node::Text("accepted".into()),
        },
    });
    if let Effect::Send {
        exchange, payload, ..
    } = &mut server.effects[0]
    {
        *exchange = ValueSource::Local { field: "id".into() };
        payload.insert("id".into(), ValueSource::Local { field: "id".into() });
    } else {
        panic!("fixture must send first");
    }
    let assign = Effect::Set {
        field: "id".into(),
        value: ValueSource::Input { field: "id".into() },
    };
    server.effects.insert(0, assign.clone());
    raw.participants[1].transitions[0].effects.push(assign);
    let model = ess_compiler::protocol::compile(raw).unwrap();
    let action = |value| Action::Input {
        participant: "server".into(),
        name: "finish".into(),
        payload: BTreeMap::from([("id".into(), value)]),
    };
    let state = initial(&model);
    assert!(successors(&model, &state, &action(Node::Bool(true))).is_err());
    assert!(successors(&model, &state, &action(Node::Text("wrong".into()))).is_err());
    let sent = successors(&model, &state, &action(Node::Text("accepted".into())))
        .unwrap()
        .remove(0);
    assert_eq!(sent.configuration.queue[0].exchange, "accepted");
    assert_eq!(
        sent.configuration.queue[0].payload["id"],
        Node::Text("accepted".into())
    );
    assert_eq!(
        sent.configuration.peers["server"].fields["id"],
        Node::Text("accepted".into())
    );
    let delivered = successors(
        &model,
        &sent.configuration,
        &Action::Deliver { transmission: 1 },
    )
    .unwrap()
    .remove(0);
    assert_eq!(
        delivered.configuration.peers["client"].fields["id"],
        Node::Text("accepted".into())
    );
    assert_eq!(delivered.configuration.peers["client"].state, "done");
}
