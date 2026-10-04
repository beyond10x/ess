//! How the events of one exact ESS travel: broker, subject, stream and envelope.
//!
//! An event is a fact and a channel is one way to carry it, so the model holds no broker. A message
//! contract is mostly transport, though — publish to this subject, a stream captures it for this
//! long, one message carries an array, producers never create the stream — and that has to be
//! stated somewhere a projection and a generated client can read. [`TransportSpec`] is that place:
//! adopter-authored `ess-transport/1` or `/2`, pinned to one specification's source digest the way
//! `ess-realization/1` is, and compiled by [`compile`] against that specification's [`EssIr`] into a
//! deterministic [`TransportIr`]. The designs are `docs/design/event-transport-binding.md` and
//! `docs/design/parameterized-event-channel-addresses.md`.
//!
//! Nothing here connects to a broker or creates a stream. A stream with `owner: external` is a
//! promise a publisher keeps by not touching it.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ess_compiler::ir::{ResolvedBody, ResolvedEvent, ResolvedField, ResolvedTypeRef};
use ess_compiler::EssIr;
use ess_domain::{binding::Delivery, Primitive};

/// The adopter-authored transport format.
pub const TRANSPORT_FORMAT: &str = "ess-transport/1";

/// The authored format that admits payload-bound address expressions.
pub const PARAMETERIZED_TRANSPORT_FORMAT: &str = "ess-transport/2";

/// The compiled transport format.
pub const TRANSPORT_IR_FORMAT: &str = "ess-transport-ir/1";

/// The compiled format for an authored [`PARAMETERIZED_TRANSPORT_FORMAT`] document.
pub const PARAMETERIZED_TRANSPORT_IR_FORMAT: &str = "ess-transport-ir/2";

/// The exact ESS a transport document binds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpecificationIdentity {
    /// The system's qualified name.
    pub system: String,
    /// The specification version.
    pub version: String,
    /// `sha256:<hex>` of the specification's source.
    pub source_digest: String,
}

/// The wire protocol a broker speaks. A closed set: a second member is a format change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Protocol {
    /// NATS, optionally with `JetStream` persistence.
    Nats,
}

impl Protocol {
    /// The protocol as `AsyncAPI` names it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Nats => "nats",
        }
    }
}

/// How one message carries the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Envelope {
    /// One message is one event payload.
    Single,
    /// One message is a JSON array of event payloads, also when it holds one.
    Array,
}

/// Where a stream keeps what it captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Storage {
    /// On disk.
    File,
    /// In memory.
    Memory,
}

/// When a stream lets a message go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Retention {
    /// When a configured limit (age, count, size) is reached.
    Limits,
    /// When every interested consumer has acknowledged it.
    Interest,
    /// When one consumer has acknowledged it.
    WorkQueue,
}

/// Who may create, update or delete a stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Owner {
    /// An operator. A publisher never creates, updates or deletes it.
    External,
    /// A publisher may create it when it is missing.
    Publisher,
}

/// How a publisher fills an `array` envelope: a producer default, not a wire limit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Batch {
    /// Flush when this many payloads are buffered.
    pub max_items: u32,
    /// Flush when the oldest buffered payload has waited this long.
    pub max_delay_ms: u64,
}

/// One broker, before resolution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrokerSpec {
    /// A stable identifier, unique in the document.
    pub id: String,
    /// The protocol it speaks.
    pub protocol: Protocol,
    /// The broker persists subjects into streams.
    #[serde(default)]
    pub jetstream: bool,
    /// Where it listens, when the contract fixes that.
    #[serde(default)]
    pub host: Option<String>,
}

/// One event's channel, before resolution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChannelSpec {
    /// The event's qualified name.
    pub event: String,
    /// The broker it is published to.
    pub broker: String,
    /// The subject one message of it is published to.
    pub subject: String,
    /// Address-expression name to required event payload path.
    #[serde(default, deserialize_with = "channel_parameters")]
    pub parameters: Option<BTreeMap<String, String>>,
    /// How one message carries it.
    pub envelope: Envelope,
    /// What the publisher promises about each message.
    pub delivery: Delivery,
    /// How a publisher fills an `array` envelope.
    #[serde(default)]
    pub batch: Option<Batch>,
}

fn channel_parameters<'de, D>(
    deserializer: D,
) -> Result<Option<BTreeMap<String, String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    <BTreeMap<String, String> as serde::Deserialize>::deserialize(deserializer).map(Some)
}

/// One stream, before resolution.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamSpec {
    /// The stream's name on the broker.
    pub name: String,
    /// The `JetStream` broker it lives on.
    pub broker: String,
    /// The subjects it captures; `*` and `>` are wildcards.
    pub subjects: Vec<String>,
    /// Where it keeps messages.
    pub storage: Storage,
    /// When it lets them go.
    pub retention: Retention,
    /// The oldest message it keeps, in seconds.
    #[serde(default)]
    pub max_age_seconds: Option<u64>,
    /// Who may create, update or delete it.
    pub owner: Owner,
}

/// An adopter-authored `ess-transport/1` or `/2` document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportSpec {
    format: String,
    specification: SpecificationIdentity,
    brokers: Vec<BrokerSpec>,
    channels: Vec<ChannelSpec>,
    #[serde(default)]
    streams: Vec<StreamSpec>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct AuthoredTransportSpec {
    #[serde(rename = "type")]
    format: String,
    specification: SpecificationRef,
    brokers: Vec<BrokerSpec>,
    channels: Vec<ChannelSpec>,
    streams: Vec<StreamSpec>,
}

impl<'de> serde::Deserialize<'de> for TransportSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let authored =
            <AuthoredTransportSpec as serde::Deserialize>::deserialize(deserializer)?;
        if authored.format == TRANSPORT_FORMAT
            && authored
                .channels
                .iter()
                .any(|channel| channel.parameters.is_some())
        {
            return Err(serde::de::Error::custom(
                "`parameters` is not a field of an ess-transport/1 channel",
            ));
        }
        Ok(Self {
            format: authored.format,
            specification: authored.specification,
            brokers: authored.brokers,
            channels: authored.channels,
            streams: authored.streams,
        })
    }
}

impl TransportSpec {
    /// Reads strict YAML. Semantic validation happens in [`compile`].
    pub fn from_yaml(text: &str) -> Result<Self, serde_yaml::Error> {
        serde_yaml::from_str(text)
    }

    /// Reads strict JSON. Semantic validation happens in [`compile`].
    pub fn from_json(text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(text)
    }
}

/// One normalized source for a channel address expression.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ParameterSource {
    /// A required payload path on this channel's event.
    EventPath {
        /// Semantic field names, excluding the leading authored `event` segment.
        path: Vec<String>,
    },
}

impl ParameterSource {
    /// The normalized semantic event path.
    pub fn event_path(&self) -> &[String] {
        match self {
            Self::EventPath { path } => path,
        }
    }
}

/// A resolved broker.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Broker {
    /// The protocol it speaks.
    pub protocol: Protocol,
    /// The broker persists subjects into streams.
    pub jetstream: bool,
    /// Where it listens, when the contract fixes that.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
}

/// A resolved channel: one event, its subject, and the stream that captures it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Channel {
    /// The broker it is published to.
    pub broker: String,
    /// The subject one message is published to.
    pub subject: String,
    /// Address expressions, ordered by their authored name. Absent from literal channel JSON.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, ParameterSource>,
    /// How one message carries the event.
    pub envelope: Envelope,
    /// What the publisher promises about each message.
    pub delivery: Delivery,
    /// How a publisher fills an `array` envelope.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch: Option<Batch>,
    /// The stream capturing the subject, on a `JetStream` broker.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
}

/// A resolved stream.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Stream {
    /// The `JetStream` broker it lives on.
    pub broker: String,
    /// The subjects it captures.
    pub subjects: Vec<String>,
    /// Where it keeps messages.
    pub storage: Storage,
    /// When it lets them go.
    pub retention: Retention,
    /// The oldest message it keeps, in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_age_seconds: Option<u64>,
    /// Who may create, update or delete it.
    pub owner: Owner,
}

/// A compiled transport IR: every collection ordered, every reference resolved.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TransportIr {
    format: &'static str,
    specification: SpecificationIdentity,
    brokers: BTreeMap<String, Broker>,
    channels: BTreeMap<String, Channel>,
    streams: BTreeMap<String, Stream>,
}

impl TransportIr {
    /// The specification this transport binds.
    pub fn specification(&self) -> &SpecificationIdentity {
        &self.specification
    }

    /// Every broker, by id.
    pub fn brokers(&self) -> &BTreeMap<String, Broker> {
        &self.brokers
    }

    /// Every channel, by the qualified name of its event.
    pub fn channels(&self) -> &BTreeMap<String, Channel> {
        &self.channels
    }

    /// Every stream, by name.
    pub fn streams(&self) -> &BTreeMap<String, Stream> {
        &self.streams
    }

    /// The channel of one event, if the transport binds it.
    pub fn channel(&self, event: &str) -> Option<&Channel> {
        self.channels.get(event)
    }

    /// The canonical JSON bytes, with a trailing newline.
    pub fn to_canonical_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(self)
            .unwrap_or_else(|error| panic!("transport IR serializes: {error}"));
        json.push('\n');
        json
    }
}

/// One refusal, located at the key that caused it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, serde::Serialize)]
pub struct TransportDiagnostic {
    /// A stable code, `ESS-TRANSPORT-NNN`.
    pub code: &'static str,
    /// The path of the offending key, such as `channels[0].subject`.
    pub path: String,
    /// What is wrong, in a sentence.
    pub message: String,
}

impl fmt::Display for TransportDiagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "error[{}]: {}: {}",
            self.code, self.path, self.message
        )
    }
}

/// Every refusal a document earned, in document order.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TransportDiagnostics(pub Vec<TransportDiagnostic>);

impl fmt::Display for TransportDiagnostics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, diagnostic) in self.0.iter().enumerate() {
            if index > 0 {
                writeln!(formatter)?;
            }
            write!(formatter, "{diagnostic}")?;
        }
        Ok(())
    }
}

impl std::error::Error for TransportDiagnostics {}

struct Refusals(Vec<TransportDiagnostic>);

impl Refusals {
    fn refuse(&mut self, code: &'static str, path: impl Into<String>, message: impl Into<String>) {
        self.0.push(TransportDiagnostic {
            code,
            path: path.into(),
            message: message.into(),
        });
    }
}

/// Resolves a transport document against the specification it names.
///
/// Every check in `docs/design/event-transport-binding.md` runs, and every refusal is returned at
/// once rather than the first.
pub fn compile(spec: &TransportSpec, ir: &EssIr) -> Result<TransportIr, TransportDiagnostics> {
    let mut refusals = Refusals(Vec::new());
    let parameterized = match spec.format.as_str() {
        TRANSPORT_FORMAT => false,
        PARAMETERIZED_TRANSPORT_FORMAT => true,
        _ => {
            refusals.refuse(
                "ESS-TRANSPORT-001",
                "type",
                format!(
                    "expected `{TRANSPORT_FORMAT}` or `{PARAMETERIZED_TRANSPORT_FORMAT}`, found `{}`",
                    spec.format
                ),
            );
            false
        }
    };
    specification(&spec.specification, ir, &mut refusals);
    let brokers = brokers(&spec.brokers, &mut refusals);
    let streams = streams(&spec.streams, &brokers, &mut refusals);
    let channels = channels(
        &spec.channels,
        ir,
        &brokers,
        &streams,
        parameterized,
        &mut refusals,
    );
    if refusals.0.is_empty() {
        Ok(TransportIr {
            format: if parameterized {
                PARAMETERIZED_TRANSPORT_IR_FORMAT
            } else {
                TRANSPORT_IR_FORMAT
            },
            specification: spec.specification.clone(),
            brokers,
            channels,
            streams,
        })
    } else {
        Err(TransportDiagnostics(refusals.0))
    }
}

fn specification(identity: &SpecificationIdentity, ir: &EssIr, refusals: &mut Refusals) {
    let digest = format!("sha256:{}", ir.source_digest());
    for (key, found, expected) in [
        ("system", identity.system.as_str(), ir.system().to_string()),
        (
            "version",
            identity.version.as_str(),
            ir.version().to_string(),
        ),
        ("source_digest", identity.source_digest.as_str(), digest),
    ] {
        if found != expected {
            refusals.refuse(
                "ESS-TRANSPORT-002",
                format!("specification.{key}"),
                format!("names `{found}`, and the specification is `{expected}`"),
            );
        }
    }
}

fn brokers(specs: &[BrokerSpec], refusals: &mut Refusals) -> BTreeMap<String, Broker> {
    let mut brokers = BTreeMap::new();
    for (index, broker) in specs.iter().enumerate() {
        let path = format!("brokers[{index}]");
        if !is_identifier(&broker.id) {
            refusals.refuse(
                "ESS-TRANSPORT-003",
                format!("{path}.id"),
                format!(
                    "`{}` is not an identifier: lowercase ASCII letters, digits and single `.`, `_` or `-` separators, starting with a letter",
                    broker.id
                ),
            );
        }
        let resolved = Broker {
            protocol: broker.protocol,
            jetstream: broker.jetstream,
            host: broker.host.clone(),
        };
        if brokers.insert(broker.id.clone(), resolved).is_some() {
            refusals.refuse(
                "ESS-TRANSPORT-004",
                format!("{path}.id"),
                format!("broker `{}` is declared twice", broker.id),
            );
        }
    }
    brokers
}

fn streams(
    specs: &[StreamSpec],
    brokers: &BTreeMap<String, Broker>,
    refusals: &mut Refusals,
) -> BTreeMap<String, Stream> {
    let mut streams = BTreeMap::new();
    for (index, stream) in specs.iter().enumerate() {
        let path = format!("streams[{index}]");
        if !is_stream_name(&stream.name) {
            refusals.refuse(
                "ESS-TRANSPORT-005",
                format!("{path}.name"),
                format!(
                    "`{}` is not a stream name: non-empty, with no whitespace, `.`, `*`, `>`, `/` or `\\`",
                    stream.name
                ),
            );
        }
        match brokers.get(&stream.broker) {
            None => refusals.refuse(
                "ESS-TRANSPORT-006",
                format!("{path}.broker"),
                format!("no broker `{}` is declared", stream.broker),
            ),
            Some(broker) if !broker.jetstream => refusals.refuse(
                "ESS-TRANSPORT-007",
                format!("{path}.broker"),
                format!(
                    "broker `{}` does not declare `jetstream: true`, so it keeps no streams",
                    stream.broker
                ),
            ),
            Some(_) => {}
        }
        if stream.subjects.is_empty() {
            refusals.refuse(
                "ESS-TRANSPORT-008",
                format!("{path}.subjects"),
                "a stream captures at least one subject",
            );
        }
        for (position, subject) in stream.subjects.iter().enumerate() {
            if let Err(reason) = subject_pattern(subject) {
                refusals.refuse(
                    "ESS-TRANSPORT-008",
                    format!("{path}.subjects[{position}]"),
                    format!("`{subject}` is not a subject pattern: {reason}"),
                );
            }
        }
        let resolved = Stream {
            broker: stream.broker.clone(),
            subjects: stream.subjects.clone(),
            storage: stream.storage,
            retention: stream.retention,
            max_age_seconds: stream.max_age_seconds,
            owner: stream.owner,
        };
        if streams.insert(stream.name.clone(), resolved).is_some() {
            refusals.refuse(
                "ESS-TRANSPORT-009",
                format!("{path}.name"),
                format!("stream `{}` is declared twice", stream.name),
            );
        }
    }
    streams
}

fn channels(
    specs: &[ChannelSpec],
    ir: &EssIr,
    brokers: &BTreeMap<String, Broker>,
    streams: &BTreeMap<String, Stream>,
    parameterized_format: bool,
    refusals: &mut Refusals,
) -> BTreeMap<String, Channel> {
    let mut channels = BTreeMap::new();
    for (index, channel) in specs.iter().enumerate() {
        let path = format!("channels[{index}]");
        let event = ir
            .events()
            .values()
            .find(|event| event.name.to_string() == channel.event);
        if event.is_none() {
            refusals.refuse(
                "ESS-TRANSPORT-010",
                format!("{path}.event"),
                format!("the specification declares no event `{}`", channel.event),
            );
        }

        let address = if parameterized_format {
            parameterized_address(channel, event, ir, &path, refusals)
        } else {
            if let Err(reason) = publish_subject(&channel.subject) {
                refusals.refuse(
                    "ESS-TRANSPORT-012",
                    format!("{path}.subject"),
                    format!(
                        "`{}` is not a publishable subject: {reason}",
                        channel.subject
                    ),
                );
            }
            Some((BTreeMap::new(), SubjectLanguage::Concrete))
        };

        if let Some(event) = event {
            match (&address, event.naming.wire.as_deref()) {
                (Some((_, SubjectLanguage::Template(_))), Some(wire)) => refusals.refuse(
                    "ESS-TRANSPORT-011",
                    format!("{path}.subject"),
                    format!(
                        "the event's wire name is `{wire}`; a parameterized subject requires it absent"
                    ),
                ),
                (_, Some(wire)) if wire != channel.subject => refusals.refuse(
                    "ESS-TRANSPORT-011",
                    format!("{path}.subject"),
                    format!(
                        "the event's wire name is `{wire}`; the subject `{}` must equal it",
                        channel.subject
                    ),
                ),
                _ => {}
            }
        }

        match (channel.envelope, channel.batch) {
            (Envelope::Single, Some(_)) => refusals.refuse(
                "ESS-TRANSPORT-013",
                format!("{path}.batch"),
                "`batch` fills an `array` envelope; this channel's envelope is `single`",
            ),
            (Envelope::Array, Some(batch)) if batch.max_items == 0 || batch.max_delay_ms == 0 => {
                refusals.refuse(
                    "ESS-TRANSPORT-013",
                    format!("{path}.batch"),
                    "`max_items` and `max_delay_ms` are each at least 1",
                );
            }
            _ => {}
        }
        let stream = match brokers.get(&channel.broker) {
            None => {
                refusals.refuse(
                    "ESS-TRANSPORT-006",
                    format!("{path}.broker"),
                    format!("no broker `{}` is declared", channel.broker),
                );
                None
            }
            Some(broker) if broker.jetstream => {
                if let Some((_, language)) = address.as_ref() {
                    capturing_stream(channel, streams, language, &path, refusals)
                } else {
                    None
                }
            }
            Some(_) => None,
        };
        let parameters = address
            .map(|(parameters, _)| parameters)
            .unwrap_or_default();
        let resolved = Channel {
            broker: channel.broker.clone(),
            subject: channel.subject.clone(),
            parameters,
            envelope: channel.envelope,
            delivery: channel.delivery,
            batch: channel.batch,
            stream,
        };
        if channels.insert(channel.event.clone(), resolved).is_some() {
            refusals.refuse(
                "ESS-TRANSPORT-014",
                format!("{path}.event"),
                format!("event `{}` has a second channel", channel.event),
            );
        }
    }
    channels
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SubjectToken {
    Static(String),
    Variable(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum SubjectLanguage {
    Concrete,
    Template(Vec<SubjectToken>),
}

fn parameterized_address(
    channel: &ChannelSpec,
    event: Option<&ResolvedEvent>,
    ir: &EssIr,
    path: &str,
    refusals: &mut Refusals,
) -> Option<(BTreeMap<String, ParameterSource>, SubjectLanguage)> {
    let start = refusals.0.len();
    let authored_tokens = match tokens(&channel.subject) {
        Ok(tokens) => tokens,
        Err(reason) => {
            refusals.refuse(
                "ESS-TRANSPORT-012",
                format!("{path}.subject"),
                format!(
                    "`{}` is not a publishable subject: {reason}",
                    channel.subject
                ),
            );
            return None;
        }
    };

    let mut parsed = Vec::with_capacity(authored_tokens.len());
    let mut expressions = BTreeSet::new();
    for token in authored_tokens {
        if token.contains(['*', '>']) {
            refusals.refuse(
                "ESS-TRANSPORT-012",
                format!("{path}.subject"),
                format!(
                    "`{}` is not a publishable subject: a publish subject has no wildcard",
                    channel.subject
                ),
            );
            continue;
        }
        if token.contains(['{', '}']) {
            let name = token
                .strip_prefix('{')
                .and_then(|rest| rest.strip_suffix('}'));
            match name {
                Some(name)
                    if !name.is_empty()
                        && !name.contains(['{', '}'])
                        && name
                            .chars()
                            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-')) =>
                {
                    expressions.insert(name.to_owned());
                    parsed.push((None, Some(name.to_owned())));
                }
                _ => refusals.refuse(
                    "ESS-TRANSPORT-017",
                    format!("{path}.subject"),
                    format!(
                        "`{token}` is not one whole address expression `{{name}}` with an ASCII letter, digit, `_` or `-` name"
                    ),
                ),
            }
        } else {
            parsed.push((Some(token.to_owned()), None));
        }
    }

    if refusals.0.len() != start {
        return None;
    }

    if expressions.is_empty() {
        if let Some(parameters) = channel.parameters.as_ref() {
            if parameters.is_empty() {
                refusals.refuse(
                    "ESS-TRANSPORT-018",
                    format!("{path}.parameters"),
                    "a literal subject has no parameter mappings",
                );
            } else {
                for name in parameters.keys() {
                    refusals.refuse(
                        "ESS-TRANSPORT-018",
                        format!("{path}.parameters.{name}"),
                        format!("parameter `{name}` is not used by the subject"),
                    );
                }
            }
        }
        return (refusals.0.len() == start)
            .then_some((BTreeMap::new(), SubjectLanguage::Concrete));
    }

    let Some(authored_parameters) = channel.parameters.as_ref() else {
        for name in &expressions {
            refusals.refuse(
                "ESS-TRANSPORT-018",
                format!("{path}.parameters.{name}"),
                format!("address expression `{{{name}}}` has no parameter mapping"),
            );
        }
        return None;
    };
    if authored_parameters.is_empty() {
        refusals.refuse(
            "ESS-TRANSPORT-018",
            format!("{path}.parameters"),
            "a parameterized subject has a nonempty parameter mapping",
        );
        return None;
    }
    for name in &expressions {
        if !authored_parameters.contains_key(name) {
            refusals.refuse(
                "ESS-TRANSPORT-018",
                format!("{path}.parameters.{name}"),
                format!("address expression `{{{name}}}` has no parameter mapping"),
            );
        }
    }
    for name in authored_parameters.keys() {
        if !expressions.contains(name) {
            refusals.refuse(
                "ESS-TRANSPORT-018",
                format!("{path}.parameters.{name}"),
                format!("parameter `{name}` is not used by the subject"),
            );
        }
    }

    let Some(event) = event else {
        return None;
    };
    let mut parameters = BTreeMap::new();
    for name in &expressions {
        let Some(source) = authored_parameters.get(name) else {
            continue;
        };
        match resolve_event_path(event, ir, source) {
            Ok(resolved) => {
                parameters.insert(name.clone(), ParameterSource::EventPath { path: resolved });
            }
            Err((code, message)) => refusals.refuse(
                code,
                format!("{path}.parameters.{name}"),
                message,
            ),
        }
    }

    if refusals.0.len() != start {
        return None;
    }
    let language = parsed
        .into_iter()
        .map(|(fixed, expression)| match (fixed, expression) {
            (Some(fixed), None) => SubjectToken::Static(fixed),
            (None, Some(expression)) => SubjectToken::Variable(
                parameters[&expression].event_path().to_vec(),
            ),
            _ => unreachable!("one parsed subject token kind"),
        })
        .collect();
    Some((parameters, SubjectLanguage::Template(language)))
}

fn resolve_event_path(
    event: &ResolvedEvent,
    ir: &EssIr,
    source: &str,
) -> Result<Vec<String>, (&'static str, String)> {
    let parts: Vec<&str> = source.split('.').collect();
    if parts.first() != Some(&"event")
        || !(2..=4).contains(&parts.len())
        || parts.iter().any(|part| part.is_empty())
    {
        return Err((
            "ESS-TRANSPORT-019",
            format!(
                "`{source}` is not `event.<field>` with one to three payload field members"
            ),
        ));
    }
    let semantic: Vec<String> = parts[1..].iter().map(|part| (*part).to_owned()).collect();
    let mut field = event.field(&semantic[0]).ok_or_else(|| {
        (
            "ESS-TRANSPORT-019",
            format!(
                "event `{}` has no payload field `{}`",
                event.name, semantic[0]
            ),
        )
    })?;
    for member in &semantic[1..] {
        field = required_struct_field(field, ir, member)?;
    }
    if !matches!(
        &field.type_ref,
        ResolvedTypeRef::Primitive {
            name: Primitive::String
        }
    ) {
        return Err((
            "ESS-TRANSPORT-020",
            format!(
                "`{source}` ends in `{}`; an address parameter ends in required primitive `String`",
                field.type_ref
            ),
        ));
    }
    Ok(semantic)
}

fn required_struct_field<'a>(
    field: &'a ResolvedField,
    ir: &'a EssIr,
    member: &str,
) -> Result<&'a ResolvedField, (&'static str, String)> {
    let ResolvedTypeRef::Declared { name } = &field.type_ref else {
        return Err((
            "ESS-TRANSPORT-020",
            format!(
                "payload member `{}` has type `{}`; an intermediate address path member is a required struct",
                field.name, field.type_ref
            ),
        ));
    };
    let declared = ir.named_type(name);
    let ResolvedBody::Struct { fields, .. } = &declared.body else {
        return Err((
            "ESS-TRANSPORT-020",
            format!(
                "payload member `{}` has type `{}`; an intermediate address path member is a required struct",
                field.name, field.type_ref
            ),
        ));
    };
    fields.iter().find(|field| field.name == member).ok_or_else(|| {
        (
            "ESS-TRANSPORT-019",
            format!("struct `{}` has no field `{member}`", declared.name),
        )
    })
}

fn covers(pattern: &str, language: &[SubjectToken]) -> bool {
    let pattern: Vec<&str> = pattern.split('.').collect();
    let has_tail = pattern.last() == Some(&">");
    let prefix = if has_tail {
        &pattern[..pattern.len() - 1]
    } else {
        pattern.as_slice()
    };
    if (has_tail && language.len() <= prefix.len())
        || (!has_tail && language.len() != prefix.len())
    {
        return false;
    }
    prefix
        .iter()
        .zip(language)
        .all(|(pattern, token)| match (*pattern, token) {
            ("*", _) => true,
            (exact, SubjectToken::Static(fixed)) => exact == fixed,
            (_, SubjectToken::Variable(_)) => false,
        })
}

fn intersects(pattern: &str, language: &[SubjectToken]) -> bool {
    let pattern: Vec<&str> = pattern.split('.').collect();
    let has_tail = pattern.last() == Some(&">");
    let prefix = if has_tail {
        &pattern[..pattern.len() - 1]
    } else {
        pattern.as_slice()
    };
    if (has_tail && language.len() <= prefix.len())
        || (!has_tail && language.len() != prefix.len())
    {
        return false;
    }
    let mut bindings: BTreeMap<Vec<String>, &str> = BTreeMap::new();
    for (pattern, token) in prefix.iter().zip(language) {
        if *pattern == "*" {
            continue;
        }
        match token {
            SubjectToken::Static(fixed) if *pattern != fixed => return false,
            SubjectToken::Static(_) => {}
            SubjectToken::Variable(source) => match bindings.get(source) {
                Some(bound) if bound != pattern => return false,
                Some(_) => {}
                None => {
                    bindings.insert(source.clone(), pattern);
                }
            },
        }
    }
    true
}

/// The one stream on the channel's broker that captures its subject.
fn capturing_stream(
    channel: &ChannelSpec,
    streams: &BTreeMap<String, Stream>,
    language: &SubjectLanguage,
    path: &str,
    refusals: &mut Refusals,
) -> Option<String> {
    let on_broker: Vec<(&String, &Stream)> = streams
        .iter()
        .filter(|(_, stream)| stream.broker == channel.broker)
        .collect();
    let capturing: Vec<&String> = on_broker
        .iter()
        .filter(|(_, stream)| {
            stream.subjects.iter().any(|pattern| match language {
                SubjectLanguage::Concrete => matches(pattern, &channel.subject),
                SubjectLanguage::Template(tokens) => covers(pattern, tokens),
            })
        })
        .map(|(name, _)| *name)
        .collect();
    match capturing.as_slice() {
        [one] => {
            if matches!(language, SubjectLanguage::Template(_)) {
                let overlaps: Vec<&str> = on_broker
                    .iter()
                    .filter(|(name, _)| *name != *one)
                    .filter(|(_, stream)| {
                        stream.subjects.iter().any(|pattern| match language {
                            SubjectLanguage::Template(tokens) => intersects(pattern, tokens),
                            SubjectLanguage::Concrete => false,
                        })
                    })
                    .map(|(name, _)| name.as_str())
                    .collect();
                if !overlaps.is_empty() {
                    refusals.refuse(
                        "ESS-TRANSPORT-016",
                        format!("{path}.subject"),
                        format!(
                            "stream `{}` covers every expansion of `{}` but streams {overlaps:?} also intersect it; NATS refuses overlapping streams",
                            one, channel.subject
                        ),
                    );
                    return None;
                }
            }
            Some((*one).clone())
        }
        [] => {
            refusals.refuse(
                "ESS-TRANSPORT-015",
                format!("{path}.subject"),
                format!(
                    "no stream on JetStream broker `{}` captures `{}`; a publish to it would fail with \"no stream found for given subject\"",
                    channel.broker, channel.subject
                ),
            );
            None
        }
        many => {
            let names: Vec<&str> = many.iter().map(|name| name.as_str()).collect();
            refusals.refuse(
                "ESS-TRANSPORT-016",
                format!("{path}.subject"),
                format!(
                    "streams {names:?} all capture `{}`; NATS refuses overlapping streams",
                    channel.subject
                ),
            );
            None
        }
    }
}

fn is_identifier(value: &str) -> bool {
    let mut previous_separator = false;
    value.chars().next().is_some_and(|c| c.is_ascii_lowercase())
        && value.chars().all(|c| {
            let separator = matches!(c, '.' | '_' | '-');
            let accepted = (c.is_ascii_lowercase() || c.is_ascii_digit() || separator)
                && !(separator && previous_separator);
            previous_separator = separator;
            accepted
        })
        && !previous_separator
}

fn is_stream_name(value: &str) -> bool {
    !value.is_empty()
        && !value
            .chars()
            .any(|c| c.is_whitespace() || matches!(c, '.' | '*' | '>' | '/' | '\\'))
}

fn tokens(subject: &str) -> Result<Vec<&str>, &'static str> {
    if subject.is_empty() {
        return Err("it is empty");
    }
    if subject.chars().any(char::is_whitespace) {
        return Err("it contains whitespace");
    }
    let tokens: Vec<&str> = subject.split('.').collect();
    if tokens.iter().any(|token| token.is_empty()) {
        return Err("it has an empty token");
    }
    Ok(tokens)
}

fn publish_subject(subject: &str) -> Result<(), &'static str> {
    if tokens(subject)?
        .iter()
        .any(|token| token.contains(['*', '>']))
    {
        return Err("a publish subject has no wildcard");
    }
    Ok(())
}

fn subject_pattern(subject: &str) -> Result<(), &'static str> {
    let tokens = tokens(subject)?;
    let last = tokens.len() - 1;
    for (index, token) in tokens.iter().enumerate() {
        if token.contains(['*', '>']) && *token != "*" && *token != ">" {
            return Err("a wildcard is a whole token");
        }
        if *token == ">" && index != last {
            return Err("`>` is only the last token");
        }
    }
    Ok(())
}

/// Whether a NATS subject pattern captures a concrete subject.
pub fn matches(pattern: &str, subject: &str) -> bool {
    let mut pattern = pattern.split('.');
    let mut subject = subject.split('.');
    loop {
        match (pattern.next(), subject.next()) {
            (Some(">"), Some(_)) | (None, None) => return true,
            (Some("*"), Some(_)) => {}
            (Some(expected), Some(found)) if expected == found => {}
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{covers, intersects, matches, publish_subject, subject_pattern, SubjectToken};

    #[test]
    fn nats_wildcards_match_whole_tokens() {
        assert!(matches("usage", "usage"));
        assert!(matches("usage.>", "usage.ivr.eu"));
        assert!(!matches("usage.>", "usage"));
        assert!(matches("usage.*.eu", "usage.ivr.eu"));
        assert!(!matches("usage.*", "usage.ivr.eu"));
        assert!(!matches("usage", "usage.ivr"));
        assert!(matches(">", "anything.at.all"));
    }

    #[test]
    fn subjects_are_checked_for_their_position() {
        assert!(publish_subject("usage.ivr").is_ok());
        assert!(publish_subject("usage.*").is_err());
        assert!(publish_subject("usage..ivr").is_err());
        assert!(publish_subject("usage ivr").is_err());
        assert!(subject_pattern("usage.>").is_ok());
        assert!(subject_pattern("usage.>.ivr").is_err());
        assert!(subject_pattern("usage.a*").is_err());
    }

    #[test]
    fn stream_language_predicates_preserve_variable_correlations() {
        let same = vec![
            SubjectToken::Static("usage".to_owned()),
            SubjectToken::Variable(vec!["source".to_owned(), "service".to_owned()]),
            SubjectToken::Variable(vec!["source".to_owned(), "service".to_owned()]),
        ];
        assert!(covers("usage.*.*", &same));
        assert!(covers("usage.>", &same));
        assert!(!covers("usage.a.a", &same));
        assert!(!covers("usage.*", &same));
        assert!(!covers("usage.*.*.*", &same));
        assert!(intersects("usage.a.a", &same));
        assert!(intersects("usage.*.a", &same));
        assert!(!intersects("usage.a.b", &same));

        let independent = vec![
            SubjectToken::Static("usage".to_owned()),
            SubjectToken::Variable(vec!["source".to_owned(), "service".to_owned()]),
            SubjectToken::Variable(vec!["source".to_owned(), "environment".to_owned()]),
        ];
        assert!(intersects("usage.a.b", &independent));
        assert!(!intersects("other.>", &independent));
        assert!(!intersects("usage.*", &independent));
        assert!(!intersects("usage.*.*.*", &independent));
    }
}
