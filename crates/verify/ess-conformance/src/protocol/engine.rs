use super::{
    Action, Configuration, ExecutionError, Observation, Peer, Step, Timer, Trace, TraceOrigin,
    TraceStep, Transmission, TRACE_FORMAT,
};
use ess_compiler::protocol::CompiledProtocol;
use ess_domain::protocol::{Duration, Effect, Ordering, Property, Trigger, ValueSource};
use ess_primitives::Node;
use std::collections::{BTreeMap, BTreeSet};

/// Construct the protocol's sole initial configuration.
pub fn initial(model: &CompiledProtocol) -> Configuration {
    let mut configuration = Configuration {
        now_ms: 0,
        steps: 0,
        next_transmission: 1,
        queue: vec![],
        violations: BTreeSet::new(),
        peers: model
            .model()
            .participants
            .iter()
            .map(|p| {
                (
                    p.name.clone(),
                    Peer {
                        state: p.initial.clone(),
                        fields: p
                            .fields
                            .iter()
                            .map(|f| (f.name.clone(), f.initial.clone()))
                            .collect(),
                        timers: BTreeMap::new(),
                        generations: BTreeMap::new(),
                        intervals: BTreeMap::new(),
                        events: BTreeMap::new(),
                        unflushed: BTreeSet::new(),
                        flushed: BTreeSet::new(),
                        closed: false,
                    },
                )
            })
            .collect(),
    };
    check_properties(model, &mut configuration);
    configuration
}

fn value(
    source: &ValueSource,
    peer: &Peer,
    input: &BTreeMap<String, Node>,
) -> Result<Node, ExecutionError> {
    match source {
        ValueSource::Literal { value } => Ok(value.clone()),
        ValueSource::Local { field } => peer
            .fields
            .get(field)
            .cloned()
            .ok_or_else(|| ExecutionError::invalid(format!("missing local field {field}"))),
        ValueSource::Input { field } => input
            .get(field)
            .cloned()
            .ok_or_else(|| ExecutionError::invalid(format!("missing input field {field}"))),
    }
}
fn text(
    source: &ValueSource,
    peer: &Peer,
    input: &BTreeMap<String, Node>,
) -> Result<String, ExecutionError> {
    value(source, peer, input)?
        .as_text()
        .map(str::to_owned)
        .ok_or_else(|| ExecutionError::invalid("correlation identity is not a String"))
}
fn next_id(c: &mut Configuration) -> Result<u64, ExecutionError> {
    let id = c.next_transmission;
    c.next_transmission = id
        .checked_add(1)
        .ok_or_else(|| ExecutionError::bound("transmission identity overflow"))?;
    Ok(id)
}
fn capacity(
    model: &CompiledProtocol,
    c: &Configuration,
    channel: &str,
) -> Result<(), ExecutionError> {
    let declared = model
        .model()
        .channels
        .iter()
        .find(|ch| ch.name == channel)
        .ok_or_else(|| ExecutionError::invalid("unknown channel"))?;
    if c.queue.iter().filter(|m| m.channel == channel).count() >= declared.capacity {
        return Err(ExecutionError::bound(format!(
            "channel {channel} capacity exhausted"
        )));
    }
    Ok(())
}
fn queue_index(c: &Configuration, id: u64) -> Result<usize, ExecutionError> {
    c.queue
        .iter()
        .position(|m| m.id == id)
        .ok_or_else(|| ExecutionError::invalid(format!("unknown transmission {id}")))
}

/// Return every permitted result of one action, retaining nondeterministic branches.
#[allow(clippy::too_many_lines)]
pub fn successors(
    model: &CompiledProtocol,
    state: &Configuration,
    action: &Action,
) -> Result<Vec<Step>, ExecutionError> {
    if state.steps >= model.model().bounds.max_steps {
        return Err(ExecutionError::bound("step bound exhausted"));
    }
    let mut base = state.clone();
    base.steps += 1;
    let mut observations = vec![];
    let empty = BTreeMap::new();
    let (participant, trigger, input) = match action {
        Action::Input {
            participant,
            name,
            payload,
        } => {
            let p = model
                .model()
                .participants
                .iter()
                .find(|p| p.name == *participant)
                .ok_or_else(|| ExecutionError::invalid("unknown participant"))?;
            let declaration = p
                .inputs
                .iter()
                .find(|i| i.name == *name)
                .ok_or_else(|| ExecutionError::invalid("unknown input"))?;
            if declaration.fields.len() != payload.len()
                || declaration.fields.iter().any(|f| {
                    payload
                        .get(&f.name)
                        .is_none_or(|v| !ess_domain::protocol::admits(&f.type_ref, v))
                })
            {
                return Err(ExecutionError::invalid(
                    "input payload does not match declared fields",
                ));
            }
            (
                participant.clone(),
                Trigger::Input { name: name.clone() },
                payload.clone(),
            )
        }
        Action::Deliver { transmission } => {
            let index = queue_index(&base, *transmission)?;
            let m = base
                .queue
                .get(index)
                .ok_or_else(|| ExecutionError::invalid("missing queue entry"))?
                .clone();
            let ch = model
                .model()
                .channels
                .iter()
                .find(|ch| ch.name == m.channel)
                .ok_or_else(|| ExecutionError::invalid("unknown channel"))?;
            if ch.ordering == Ordering::Fifo
                && base
                    .queue
                    .iter()
                    .take(index)
                    .any(|earlier| earlier.channel == m.channel)
            {
                return Err(ExecutionError::invalid(
                    "FIFO delivery cannot overtake an earlier occurrence",
                ));
            }
            base.queue.remove(index);
            observations.push(Observation::Delivered {
                transmission: *transmission,
            });
            (
                ch.to.clone(),
                Trigger::Receive {
                    channel: m.channel,
                    message: m.message,
                },
                m.payload,
            )
        }
        Action::Fire {
            participant,
            timer,
            generation,
        } => {
            let p = base
                .peers
                .get_mut(participant)
                .ok_or_else(|| ExecutionError::invalid("unknown timer owner"))?;
            let armed = p
                .timers
                .get(timer)
                .ok_or_else(|| ExecutionError::invalid("timer is not armed"))?;
            if armed.generation != *generation || armed.deadline_ms != base.now_ms {
                return Err(ExecutionError::invalid(
                    "stale timer generation or timer not due",
                ));
            }
            p.timers.remove(timer);
            observations.push(Observation::TimerFired {
                participant: participant.clone(),
                timer: timer.clone(),
                generation: *generation,
            });
            (
                participant.clone(),
                Trigger::Timer {
                    name: timer.clone(),
                },
                empty,
            )
        }
        Action::AdvanceTo { millis } => {
            if *millis <= state.now_ms {
                return Err(ExecutionError::invalid("time must advance strictly"));
            }
            if *millis > model.model().bounds.max_time_ms {
                return Err(ExecutionError::bound("logical time bound exhausted"));
            }
            if state
                .peers
                .values()
                .flat_map(|p| p.timers.values())
                .any(|t| t.deadline_ms < *millis)
            {
                return Err(ExecutionError::invalid(
                    "time advancement would cross an unfired deadline",
                ));
            }
            base.now_ms = *millis;
            observations.push(Observation::TimeAdvanced { millis: *millis });
            return Ok(vec![Step {
                configuration: base,
                observations,
            }]);
        }
        Action::Drop { transmission } | Action::Duplicate { transmission } => {
            let index = queue_index(&base, *transmission)?;
            let mut m = base
                .queue
                .get(index)
                .ok_or_else(|| ExecutionError::invalid("missing queue entry"))?
                .clone();
            let ch = model
                .model()
                .channels
                .iter()
                .find(|ch| ch.name == m.channel)
                .ok_or_else(|| ExecutionError::invalid("unknown channel"))?;
            if matches!(action, Action::Drop { .. }) {
                if !ch.loss {
                    return Err(ExecutionError::invalid("channel does not permit loss"));
                }
                base.queue.remove(index);
                observations.push(Observation::Dropped {
                    transmission: *transmission,
                });
            } else {
                if !ch.duplication {
                    return Err(ExecutionError::invalid(
                        "channel does not permit duplication",
                    ));
                }
                capacity(model, &base, &m.channel)?;
                m.id = next_id(&mut base)?;
                observations.push(Observation::Duplicated {
                    original: *transmission,
                    transmission: m.id,
                });
                base.queue.push(m);
            }
            return Ok(vec![Step {
                configuration: base,
                observations,
            }]);
        }
    };
    let peer = base
        .peers
        .get(&participant)
        .ok_or_else(|| ExecutionError::invalid("unknown participant"))?;
    if peer.closed {
        return Err(ExecutionError::invalid("participant is closed"));
    }
    let declaration = model
        .model()
        .participants
        .iter()
        .find(|p| p.name == participant)
        .ok_or_else(|| ExecutionError::invalid("unknown participant"))?;
    let mut results = vec![];
    for transition in &declaration.transitions {
        if transition.from != peer.state || transition.trigger != trigger {
            continue;
        }
        let mut guarded = true;
        for guard in &transition.guards {
            if value(&guard.left, peer, &input)? != value(&guard.right, peer, &input)? {
                guarded = false;
                break;
            }
        }
        if !guarded {
            continue;
        }
        let mut c = base.clone();
        let mut o = observations.clone();
        c.peers
            .get_mut(&participant)
            .ok_or_else(|| ExecutionError::invalid("unknown participant"))?
            .state
            .clone_from(&transition.to);
        for effect in &transition.effects {
            apply(model, &mut c, &participant, &input, effect, &mut o).map_err(|error| {
                ExecutionError::bound(format!("model transition could not execute: {error}"))
            })?;
            check_properties(model, &mut c);
        }
        check_properties(model, &mut c);
        results.push(Step {
            configuration: c,
            observations: o,
        });
    }
    if results.is_empty() {
        return Err(ExecutionError::invalid(
            "no enabled transition for this action",
        ));
    }
    Ok(results)
}

#[allow(clippy::too_many_lines)]
fn apply(
    model: &CompiledProtocol,
    c: &mut Configuration,
    participant: &str,
    input: &BTreeMap<String, Node>,
    effect: &Effect,
    observations: &mut Vec<Observation>,
) -> Result<(), ExecutionError> {
    let peer = c
        .peers
        .get(participant)
        .ok_or_else(|| ExecutionError::invalid("unknown participant"))?;
    match effect {
        Effect::Set {
            field,
            value: source,
        } => {
            let v = value(source, peer, input)?;
            c.peers
                .get_mut(participant)
                .ok_or_else(|| ExecutionError::invalid("unknown participant"))?
                .fields
                .insert(field.clone(), v);
        }
        Effect::Send {
            channel,
            message,
            exchange,
            logical,
            payload,
        } => {
            if peer.closed {
                return Err(ExecutionError::invalid("send after close"));
            }
            capacity(model, c, channel)?;
            let exchange = text(exchange, peer, input)?;
            let logical = text(logical, peer, input)?;
            let payload = payload
                .iter()
                .map(|(k, v)| Ok((k.clone(), value(v, peer, input)?)))
                .collect::<Result<BTreeMap<_, _>, ExecutionError>>()?;
            let id = next_id(c)?;
            let message = Transmission {
                id,
                original: id,
                channel: channel.clone(),
                message: message.clone(),
                exchange,
                logical,
                payload,
            };
            c.queue.push(message.clone());
            if model.model().properties.iter().any(|property| matches!(property, Property::FlushBeforeClose { participant: owner, channel: declared, .. } if owner == participant && declared == channel)) {
                c.peers.get_mut(participant).ok_or_else(|| ExecutionError::invalid("unknown participant"))?.unflushed.insert(channel.clone());
            }
            observations.push(Observation::Sent { message });
        }
        Effect::Arm { timer, after } => {
            let millis = match after {
                Duration::Fixed { millis } => *millis,
                Duration::Backoff {
                    timer,
                    factor,
                    ceiling_ms,
                } => peer
                    .intervals
                    .get(timer)
                    .ok_or_else(|| ExecutionError::invalid("backoff has no prior interval"))?
                    .checked_mul(u64::from(*factor))
                    .ok_or_else(|| ExecutionError::bound("backoff arithmetic overflow"))?
                    .min(*ceiling_ms),
            };
            let deadline_ms = c
                .now_ms
                .checked_add(millis)
                .ok_or_else(|| ExecutionError::bound("timer deadline overflow"))?;
            let generation = peer
                .generations
                .get(timer)
                .copied()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or_else(|| ExecutionError::bound("timer generation overflow"))?;
            let peer = c
                .peers
                .get_mut(participant)
                .ok_or_else(|| ExecutionError::invalid("unknown participant"))?;
            peer.generations.insert(timer.clone(), generation);
            peer.intervals.insert(timer.clone(), millis);
            peer.timers.insert(
                timer.clone(),
                Timer {
                    generation,
                    deadline_ms,
                },
            );
            observations.push(Observation::TimerArmed {
                participant: participant.into(),
                timer: timer.clone(),
                generation,
                deadline_ms,
            });
        }
        Effect::Cancel { timer } => {
            c.peers
                .get_mut(participant)
                .ok_or_else(|| ExecutionError::invalid("unknown participant"))?
                .timers
                .remove(timer);
            observations.push(Observation::TimerCanceled {
                participant: participant.into(),
                timer: timer.clone(),
            });
        }
        Effect::Emit { name } => {
            let count = c
                .peers
                .get_mut(participant)
                .ok_or_else(|| ExecutionError::invalid("unknown participant"))?
                .events
                .entry(name.clone())
                .or_default();
            *count = count
                .checked_add(1)
                .ok_or_else(|| ExecutionError::bound("event count overflow"))?;
            observations.push(Observation::Emitted {
                participant: participant.into(),
                name: name.clone(),
            });
        }
        Effect::Flush { channel } => {
            if peer.closed {
                return Err(ExecutionError::invalid("flush after close"));
            }
            let peer = c
                .peers
                .get_mut(participant)
                .ok_or_else(|| ExecutionError::invalid("unknown participant"))?;
            peer.unflushed.remove(channel);
            peer.flushed.insert(channel.clone());
            observations.push(Observation::Flushed {
                participant: participant.into(),
                channel: channel.clone(),
            });
        }
        Effect::Close => {
            c.peers
                .get_mut(participant)
                .ok_or_else(|| ExecutionError::invalid("unknown participant"))?
                .closed = true;
            observations.push(Observation::Closed {
                participant: participant.into(),
            });
        }
    }
    Ok(())
}

fn check_properties(model: &CompiledProtocol, c: &mut Configuration) {
    for property in &model.model().properties {
        let (name, violated) = match property {
            Property::NeverState {
                name,
                participant,
                state,
            } => (
                name,
                c.peers.get(participant).is_some_and(|p| p.state == *state),
            ),
            Property::EventCount {
                name,
                participant,
                event,
                max,
            } => (
                name,
                c.peers
                    .get(participant)
                    .is_some_and(|p| p.events.get(event).copied().unwrap_or(0) > *max),
            ),
            Property::FlushBeforeClose {
                name,
                participant,
                channel,
            } => (
                name,
                c.peers.get(participant).is_some_and(|p| {
                    p.closed && (!p.flushed.contains(channel) || p.unflushed.contains(channel))
                }),
            ),
            Property::StateImplication {
                name,
                participant,
                state,
                other,
                other_state,
            } => (
                name,
                c.peers.get(participant).is_some_and(|p| p.state == *state)
                    && c.peers.get(other).is_none_or(|p| p.state != *other_state),
            ),
        };
        if violated {
            c.violations.insert(name.clone());
        }
    }
}

/// Enumerate scheduling actions and supplied finite input witnesses in stable order.
pub fn enabled_actions(
    model: &CompiledProtocol,
    state: &Configuration,
    inputs: &[Action],
) -> Vec<Action> {
    let mut actions: BTreeSet<Action> = inputs
        .iter()
        .filter(|a| matches!(a, Action::Input { .. }))
        .cloned()
        .collect();
    for p in &model.model().participants {
        for input in &p.inputs {
            if input.fields.is_empty() {
                actions.insert(Action::Input {
                    participant: p.name.clone(),
                    name: input.name.clone(),
                    payload: BTreeMap::new(),
                });
            }
        }
    }
    for m in &state.queue {
        actions.insert(Action::Deliver { transmission: m.id });
        actions.insert(Action::Drop { transmission: m.id });
        actions.insert(Action::Duplicate { transmission: m.id });
    }
    let mut next = None;
    for (participant, peer) in &state.peers {
        for (timer, t) in &peer.timers {
            if t.deadline_ms == state.now_ms {
                actions.insert(Action::Fire {
                    participant: participant.clone(),
                    timer: timer.clone(),
                    generation: t.generation,
                });
            } else if t.deadline_ms > state.now_ms {
                next = Some(next.map_or(t.deadline_ms, |n: u64| n.min(t.deadline_ms)));
            }
        }
    }
    if let Some(millis) = next {
        actions.insert(Action::AdvanceTo { millis });
    }
    actions
        .into_iter()
        .filter(|a| successors(model, state, a).is_ok_or_bound())
        .collect()
}
trait Enabled {
    fn is_ok_or_bound(&self) -> bool;
}
impl Enabled for Result<Vec<Step>, ExecutionError> {
    fn is_ok_or_bound(&self) -> bool {
        self.as_ref().map_or_else(|e| e.bound, |_| true)
    }
}

/// Simulate an unambiguous action sequence. This produces model evidence, not target evidence.
pub fn simulate(model: &CompiledProtocol, actions: &[Action]) -> Result<Trace, ExecutionError> {
    let mut configuration = initial(model);
    let mut steps = vec![];
    for action in actions {
        let mut next = successors(model, &configuration, action)?;
        if next.len() != 1 {
            return Err(ExecutionError::invalid(
                "simulation requires an unambiguous transition; use exploration",
            ));
        }
        let step = next.remove(0);
        configuration = step.configuration;
        steps.push(TraceStep {
            action: action.clone(),
            observations: step.observations,
        });
    }
    Ok(Trace {
        format: TRACE_FORMAT.into(),
        model_digest: model.digest().into(),
        origin: TraceOrigin::Model,
        complete: configuration.settled(),
        steps,
    })
}
