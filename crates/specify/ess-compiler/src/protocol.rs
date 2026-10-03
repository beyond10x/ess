//! Admission of experimental protocol documents into immutable executable models.
//!
//! The first profile resolves local names by exhaustive admission rather than minting handles:
//! the immutable admitted document is the only executor input. Unsupported types and operations
//! are refused before execution. Ordinary ESS projections do not consume this sidecar.
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use ess_domain::protocol::{
    self, Duration, Effect, Field, Participant, Property, RawProtocol, Trigger, ValueSource,
};
use ess_domain::{Primitive, TypeRef};
use ess_primitives::node::Node;
use sha2::{Digest, Sha256};

/// One accumulated source refusal.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Diagnostic {
    /// Source position within the document.
    pub path: String,
    /// Stable refusal code.
    pub code: String,
    /// Human-readable explanation.
    pub message: String,
}

/// Immutable, fully admitted protocol. It cannot be deserialized around validation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompiledProtocol {
    model: RawProtocol,
    canonical: String,
    digest: String,
}

impl CompiledProtocol {
    /// The immutable declarations whose references admission resolved.
    pub fn model(&self) -> &RawProtocol {
        &self.model
    }
    /// SHA-256 of canonical semantic bytes, including format and execution bounds.
    pub fn digest(&self) -> &str {
        &self.digest
    }
    /// Canonical JSON with one trailing newline.
    pub fn to_canonical_json(&self) -> &str {
        &self.canonical
    }
}

/// Parse a strict experimental document and accumulate semantic refusals.
pub fn parse_and_compile(text: &str) -> Result<CompiledProtocol, Vec<Diagnostic>> {
    let raw = RawProtocol::parse(text).map_err(|error| {
        vec![Diagnostic {
            path: "$".into(),
            code: "MalformedProtocol".into(),
            message: error.to_string(),
        }]
    })?;
    compile(raw)
}

/// Admit every reference, field assignment, timer expression and finite bound.
pub fn compile(model: RawProtocol) -> Result<CompiledProtocol, Vec<Diagnostic>> {
    let mut validation = Validation::default();
    validation.document(&model);
    if !validation.errors.is_empty() {
        return Err(validation.errors);
    }
    let canonical = serde_json::to_string(&model).map_err(|error| {
        vec![Diagnostic {
            path: "$".into(),
            code: "CanonicalProtocol".into(),
            message: error.to_string(),
        }]
    })? + "\n";
    let digest = Sha256::digest(canonical.as_bytes()).iter().fold(
        String::with_capacity(64),
        |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        },
    );
    Ok(CompiledProtocol {
        model,
        canonical,
        digest,
    })
}

#[derive(Default)]
struct Validation {
    errors: Vec<Diagnostic>,
}

impl Validation {
    fn error(&mut self, path: &str, code: &str, message: impl Into<String>) {
        self.errors.push(Diagnostic {
            path: path.into(),
            code: code.into(),
            message: message.into(),
        });
    }
    fn check(&mut self, condition: bool, path: &str, code: &str, message: impl Into<String>) {
        if !condition {
            self.error(path, code, message);
        }
    }
    fn names<'a>(&mut self, names: impl IntoIterator<Item = &'a str>, path: &str) {
        let mut found = BTreeSet::new();
        for name in names {
            self.check(
                !name.is_empty()
                    && name
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')),
                path,
                "InvalidIdentity",
                format!("invalid identity `{name}`"),
            );
            self.check(
                found.insert(name),
                path,
                "DuplicateIdentity",
                format!("duplicate identity `{name}`"),
            );
        }
    }
    fn supported_type(&mut self, type_ref: &TypeRef, path: &str) {
        self.check(
            matches!(
                type_ref,
                TypeRef::Primitive(Primitive::String | Primitive::Boolean | Primitive::Integer)
            ),
            path,
            "UnsupportedProtocolType",
            format!("{type_ref}: ess-protospec/1 supports String, Boolean and Integer only"),
        );
    }
    fn fields(&mut self, fields: &[Field], path: &str) {
        self.names(fields.iter().map(|field| field.name.as_str()), path);
        for field in fields {
            self.supported_type(&field.type_ref, &format!("{path}.{}", field.name));
        }
    }
    fn document(&mut self, model: &RawProtocol) {
        self.check(
            model.format == protocol::FORMAT,
            "format",
            "UnsupportedProtocolFormat",
            format!("expected {}, found {}", protocol::FORMAT, model.format),
        );
        self.names([model.name.as_str()], "name");
        self.check(
            !model.participants.is_empty(),
            "participants",
            "EmptyProtocol",
            "at least one participant is required",
        );
        self.check(
            model.bounds.max_steps > 0 && model.bounds.max_steps <= 100_000,
            "bounds.max_steps",
            "InvalidBound",
            "max_steps must be in 1..=100000",
        );
        self.check(
            model.bounds.max_states > 0 && model.bounds.max_states <= 1_000_000,
            "bounds.max_states",
            "InvalidBound",
            "max_states must be in 1..=1000000",
        );
        self.check(
            model.bounds.max_time_ms > 0 && model.bounds.max_time_ms <= 9_007_199_254_740_991,
            "bounds.max_time_ms",
            "InvalidBound",
            "max_time_ms must be positive and exactly representable as an integer across targets",
        );
        self.names(
            model.participants.iter().map(|p| p.name.as_str()),
            "participants",
        );
        self.names(model.messages.iter().map(|m| m.name.as_str()), "messages");
        self.names(model.channels.iter().map(|c| c.name.as_str()), "channels");
        for message in &model.messages {
            self.fields(&message.fields, &format!("messages.{}", message.name));
        }
        for channel in &model.channels {
            let at = format!("channels.{}", channel.name);
            self.check(
                model.participants.iter().any(|p| p.name == channel.from),
                &at,
                "UnknownParticipant",
                &channel.from,
            );
            self.check(
                model.participants.iter().any(|p| p.name == channel.to),
                &at,
                "UnknownParticipant",
                &channel.to,
            );
            self.check(
                channel.capacity > 0 && channel.capacity <= 100_000,
                &at,
                "InvalidBound",
                "channel capacity must be in 1..=100000",
            );
        }
        for participant in &model.participants {
            self.participant(model, participant);
        }
        self.properties(model);
    }
    fn participant(&mut self, model: &RawProtocol, participant: &Participant) {
        let at = format!("participants.{}", participant.name);
        self.names(
            participant.states.iter().map(String::as_str),
            &format!("{at}.states"),
        );
        self.names(
            participant.timers.iter().map(String::as_str),
            &format!("{at}.timers"),
        );
        self.names(
            participant.inputs.iter().map(|input| input.name.as_str()),
            &format!("{at}.inputs"),
        );
        self.names(
            participant.fields.iter().map(|field| field.name.as_str()),
            &format!("{at}.fields"),
        );
        self.names(
            participant
                .transitions
                .iter()
                .map(|transition| transition.name.as_str()),
            &format!("{at}.transitions"),
        );
        self.check(
            participant.states.contains(&participant.initial),
            &at,
            "UnknownState",
            &participant.initial,
        );
        for field in &participant.fields {
            self.supported_type(&field.type_ref, &at);
            self.check(
                protocol::admits(&field.type_ref, &field.initial),
                &at,
                "InvalidInitialValue",
                format!("{} must hold {}", field.name, field.type_ref),
            );
        }
        for input in &participant.inputs {
            self.fields(&input.fields, &at);
        }
        for transition in &participant.transitions {
            let at = format!("{at}.transitions.{}", transition.name);
            for state in [&transition.from, &transition.to] {
                self.check(
                    participant.states.contains(state),
                    &at,
                    "UnknownState",
                    state,
                );
            }
            let input = self.trigger(model, participant, &transition.trigger, &at);
            for guard in &transition.guards {
                let left = self.source(participant, input, &guard.left, &at);
                let right = self.source(participant, input, &guard.right, &at);
                if let (Some(left), Some(right)) = (left, right) {
                    self.check(
                        left == right,
                        &at,
                        "GuardTypeMismatch",
                        format!("cannot compare {left} with {right}"),
                    );
                }
            }
            for effect in &transition.effects {
                self.effect(model, participant, input, effect, &at);
            }
        }
    }
    fn trigger<'a>(
        &mut self,
        model: &'a RawProtocol,
        participant: &'a Participant,
        trigger: &Trigger,
        at: &str,
    ) -> &'a [Field] {
        match trigger {
            Trigger::Input { name } => {
                let input = participant.inputs.iter().find(|input| input.name == *name);
                self.check(input.is_some(), at, "UnknownInput", name);
                input.map_or(&[], |input| input.fields.as_slice())
            }
            Trigger::Receive { channel, message } => {
                let declared = model
                    .channels
                    .iter()
                    .find(|declared| declared.name == *channel);
                self.check(
                    declared.is_some_and(|c| c.to == participant.name),
                    at,
                    "InvalidReceiveChannel",
                    format!("{channel} does not deliver to {}", participant.name),
                );
                let message = model
                    .messages
                    .iter()
                    .find(|declared| declared.name == *message);
                self.check(
                    message.is_some(),
                    at,
                    "UnknownMessage",
                    "receive message is undeclared",
                );
                message.map_or(&[], |message| message.fields.as_slice())
            }
            Trigger::Timer { name } => {
                self.check(participant.timers.contains(name), at, "UnknownTimer", name);
                &[]
            }
        }
    }
    fn source(
        &mut self,
        participant: &Participant,
        input: &[Field],
        source: &ValueSource,
        at: &str,
    ) -> Option<TypeRef> {
        let ty = match source {
            ValueSource::Literal {
                value: Node::Text(_),
            } => Some(TypeRef::Primitive(Primitive::String)),
            ValueSource::Literal {
                value: Node::Bool(_),
            } => Some(TypeRef::Primitive(Primitive::Boolean)),
            ValueSource::Literal {
                value: Node::Number(number),
            } if number.as_i64().is_some() => Some(TypeRef::Primitive(Primitive::Integer)),
            ValueSource::Literal { .. } => None,
            ValueSource::Local { field } => participant
                .fields
                .iter()
                .find(|f| f.name == *field)
                .map(|f| f.type_ref.clone()),
            ValueSource::Input { field } => input
                .iter()
                .find(|f| f.name == *field)
                .map(|f| f.type_ref.clone()),
        };
        self.check(
            ty.is_some(),
            at,
            "InvalidValueSource",
            format!("undeclared field or unsupported scalar: {source:?}"),
        );
        ty
    }
    fn expected(
        &mut self,
        participant: &Participant,
        input: &[Field],
        source: &ValueSource,
        expected: &TypeRef,
        at: &str,
    ) {
        if let Some(actual) = self.source(participant, input, source, at) {
            self.check(
                actual == *expected,
                at,
                "ValueTypeMismatch",
                format!("expected {expected}, found {actual}"),
            );
        }
    }
    fn outgoing(
        &mut self,
        model: &RawProtocol,
        participant: &Participant,
        channel: &str,
        at: &str,
    ) {
        self.check(
            model
                .channels
                .iter()
                .any(|c| c.name == channel && c.from == participant.name),
            at,
            "InvalidSendChannel",
            format!("{channel} is not outgoing from {}", participant.name),
        );
    }
    // One exhaustive arm per effect keeps syntax admission auditable against the public enum.
    #[allow(clippy::too_many_lines)]
    fn effect(
        &mut self,
        model: &RawProtocol,
        participant: &Participant,
        input: &[Field],
        effect: &Effect,
        at: &str,
    ) {
        match effect {
            Effect::Set { field, value } => {
                if let Some(field) = participant
                    .fields
                    .iter()
                    .find(|declared| declared.name == *field)
                {
                    self.expected(participant, input, value, &field.type_ref, at);
                } else {
                    self.error(at, "UnknownLocalField", field);
                }
            }
            Effect::Send {
                channel,
                message,
                exchange,
                logical,
                payload,
            } => {
                self.outgoing(model, participant, channel, at);
                for identity in [exchange, logical] {
                    self.expected(
                        participant,
                        input,
                        identity,
                        &TypeRef::Primitive(Primitive::String),
                        at,
                    );
                }
                if let Some(message) = model
                    .messages
                    .iter()
                    .find(|declared| declared.name == *message)
                {
                    let fields: BTreeMap<_, _> = message
                        .fields
                        .iter()
                        .map(|field| (&field.name, &field.type_ref))
                        .collect();
                    self.check(
                        fields.len() == payload.len()
                            && payload.keys().all(|key| fields.contains_key(key)),
                        at,
                        "PayloadFieldsMismatch",
                        "send payload must name exactly the message's declared fields",
                    );
                    for (name, source) in payload {
                        if let Some(ty) = fields.get(name) {
                            self.expected(participant, input, source, ty, at);
                        }
                    }
                } else {
                    self.error(at, "UnknownMessage", message);
                }
            }
            Effect::Arm { timer, after } => {
                self.check(
                    participant.timers.contains(timer),
                    at,
                    "UnknownTimer",
                    timer,
                );
                match after {
                    Duration::Fixed { millis } => self.check(
                        *millis > 0 && *millis <= model.bounds.max_time_ms,
                        at,
                        "InvalidTimerDuration",
                        "fixed interval must be positive and within max_time_ms",
                    ),
                    Duration::Backoff {
                        timer,
                        factor,
                        ceiling_ms,
                    } => {
                        self.check(
                            participant.timers.contains(timer),
                            at,
                            "UnknownTimer",
                            timer,
                        );
                        self.check(
                            *factor > 0
                                && *ceiling_ms > 0
                                && *ceiling_ms <= model.bounds.max_time_ms,
                            at,
                            "InvalidTimerDuration",
                            "backoff needs a positive multiplier and ceiling within max_time_ms",
                        );
                    }
                }
            }
            Effect::Cancel { timer } => self.check(
                participant.timers.contains(timer),
                at,
                "UnknownTimer",
                timer,
            ),
            Effect::Emit { name } => self.names([name.as_str()], at),
            Effect::Flush { channel } => self.outgoing(model, participant, channel, at),
            Effect::Close => {}
        }
    }
    fn state(&mut self, model: &RawProtocol, participant: &str, state: &str, at: &str) {
        self.check(
            model
                .participants
                .iter()
                .any(|p| p.name == participant && p.states.iter().any(|s| s == state)),
            at,
            "UnknownParticipantState",
            format!("{participant}.{state}"),
        );
    }
    fn properties(&mut self, model: &RawProtocol) {
        let mut names = Vec::new();
        for property in &model.properties {
            match property {
                Property::NeverState {
                    name,
                    participant,
                    state,
                } => {
                    names.push(name.as_str());
                    self.state(model, participant, state, name);
                }
                Property::StateImplication {
                    name,
                    participant,
                    state,
                    other,
                    other_state,
                } => {
                    names.push(name.as_str());
                    self.state(model, participant, state, name);
                    self.state(model, other, other_state, name);
                }
                Property::EventCount {
                    name,
                    participant,
                    event,
                    ..
                } => {
                    names.push(name.as_str());
                    let owner = model.participants.iter().find(|p| p.name == *participant);
                    self.check(owner.is_some(), name, "UnknownParticipant", participant);
                    self.check(
                        owner.is_some_and(|p| {
                            p.transitions.iter().any(|t| {
                                t.effects
                                    .iter()
                                    .any(|e| matches!(e, Effect::Emit { name } if name == event))
                            })
                        }),
                        name,
                        "UnknownEvent",
                        event,
                    );
                }
                Property::FlushBeforeClose {
                    name,
                    participant,
                    channel,
                } => {
                    names.push(name.as_str());
                    self.check(
                        model
                            .channels
                            .iter()
                            .any(|c| c.name == *channel && c.from == *participant),
                        name,
                        "InvalidFlushProperty",
                        "flush property must name an outgoing channel of its participant",
                    );
                }
            }
        }
        self.names(names, "properties");
    }
}
