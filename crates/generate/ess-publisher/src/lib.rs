//! A generated event publisher for one component, over the channels an `ess-transport/1` or `/2`
//! document binds (beyond10x/ess#395; `docs/design/event-publishers.md`).
//!
//! [`plan`] decides what is generated: one publish operation per event the component publishes and
//! the transport binds, refusing what this version cannot honour. [`rust`] and [`go`] render that
//! plan on top of a types library the caller already realized (`ess generate types`, event roots),
//! and return the files to write beside it. The core of each library depends on nothing the types
//! library does not; the NATS adapter is opt-in (Rust crate `nats/`, Go module `natsjs/`).
//!
//! Nothing here connects to a broker. Nothing generated creates, updates or deletes a stream.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::ir::{ResolvedBody, ResolvedEvent, ResolvedTypeRef};
use ess_compiler::EssIr;
use ess_domain::binding::Delivery;
use ess_transport::{Batch, Envelope, Owner, ParameterSource, TransportIr};

/// The report format written beside a generated publisher.
pub const CLIENT_REPORT_FORMAT: &str = "ess-client-report/1";

/// The report format used when at least one operation has a dynamic subject.
pub const PARAMETERIZED_CLIENT_REPORT_FORMAT: &str = "ess-client-report/2";

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParameterAccess {
    rust: String,
    go: String,
    go_required_structs: Vec<String>,
}

/// One publish operation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Operation {
    /// The event's qualified name.
    pub event: String,
    /// `snake_case` of the event's last name segment, the stem of every generated name.
    pub stem: String,
    /// The native type the types library declares for the event payload.
    pub payload_type: String,
    /// The subject one message is published to.
    pub subject: String,
    /// Authored address-expression sources, normalized by semantic event path.
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub parameters: BTreeMap<String, ParameterSource>,
    #[serde(skip)]
    parameter_access: BTreeMap<String, ParameterAccess>,
    /// How one message carries the payload.
    pub envelope: Envelope,
    /// How an `array` envelope fills.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch: Option<Batch>,
    /// The publisher's promise.
    pub delivery: Delivery,
    /// The stream capturing the subject, and who owns it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<String>,
    /// Who may create, update or delete that stream.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_owner: Option<Owner>,
    /// The broker's protocol.
    pub protocol: &'static str,
}

/// What one component's publisher holds.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PublisherPlan {
    /// The component.
    pub component: String,
    /// Its publish operations, by event name.
    pub operations: Vec<Operation>,
    /// What the generated library does not discharge.
    pub obligations: Vec<String>,
}

/// Why a publisher cannot be generated.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct Refusal {
    /// A stable rule name.
    pub rule: &'static str,
    /// What is wrong.
    pub detail: String,
}

impl PublisherPlan {
    /// The events the types library has to realize: every operation's payload.
    pub fn roots(ir: &EssIr, component: &str, transport: &TransportIr) -> Vec<String> {
        published(ir, component)
            .into_iter()
            .filter(|event| transport.channel(event).is_some())
            .collect()
    }
}

fn published(ir: &EssIr, component: &str) -> Vec<String> {
    ir.components()
        .values()
        .filter(|item| item.name.as_str() == component)
        .flat_map(|item| {
            item.publishes
                .iter()
                .map(|handle| handle.name().to_string())
        })
        .collect()
}

/// Decides the operations of `component`'s publisher.
///
/// `declarations` maps an event's qualified name to the native type the types library declares for
/// its payload (`types-report.json` `declarations`).
pub fn plan(
    ir: &EssIr,
    component: &str,
    transport: &TransportIr,
    declarations: &BTreeMap<String, String>,
) -> Result<PublisherPlan, Vec<Refusal>> {
    let mut refusals = Vec::new();
    if !ir
        .components()
        .values()
        .any(|item| item.name.as_str() == component)
    {
        refusals.push(Refusal {
            rule: "unknown_component",
            detail: format!("the specification declares no component `{component}`"),
        });
        return Err(refusals);
    }
    let mut operations = Vec::new();
    let mut obligations = Vec::new();
    for event in published(ir, component) {
        let Some(channel) = transport.channel(&event) else {
            obligations.push(format!(
                "`{event}` is published by `{component}` and bound by no transport channel; no publish operation is generated for it"
            ));
            continue;
        };
        if channel.delivery == Delivery::AtLeastOnce {
            refusals.push(Refusal {
                rule: "unsupported_delivery",
                detail: format!(
                    "`{event}` promises at_least_once delivery, which needs a retry policy (attempts, backoff, what happens at close) the transport document does not state"
                ),
            });
            continue;
        }
        let Some(payload_type) = declarations.get(&event) else {
            refusals.push(Refusal {
                rule: "missing_payload_type",
                detail: format!("the types library declares no type for `{event}`"),
            });
            continue;
        };
        let broker = &transport.brokers()[&channel.broker];
        let stream_owner = channel
            .stream
            .as_ref()
            .map(|name| transport.streams()[name].owner);
        if stream_owner == Some(Owner::Publisher) {
            obligations.push(format!(
                "stream `{}` has owner: publisher; creating it is the application's, the generated library never touches a stream",
                channel.stream.as_deref().unwrap_or_default()
            ));
        }
        operations.push(Operation {
            stem: snake(event.rsplit('.').next().unwrap_or(&event)),
            event: event.clone(),
            payload_type: payload_type.clone(),
            subject: channel.subject.clone(),
            parameters: channel.parameters.clone(),
            parameter_access: parameter_access(ir, &event, &channel.parameters),
            envelope: channel.envelope,
            batch: channel.batch,
            delivery: channel.delivery,
            stream: channel.stream.clone(),
            stream_owner,
            protocol: broker.protocol.as_str(),
        });
    }
    if operations.is_empty() && refusals.is_empty() {
        refusals.push(Refusal {
            rule: "nothing_to_publish",
            detail: format!("`{component}` publishes no event the transport binds"),
        });
    }
    let mut stems: Vec<&str> = operations.iter().map(|op| op.stem.as_str()).collect();
    stems.sort_unstable();
    if let Some(stem) = stems.windows(2).find_map(|pair| match pair {
        [first, second] if first == second => Some(*first),
        _ => None,
    }) {
        refusals.push(Refusal {
            rule: "operation_name_collision",
            detail: format!("two published events would both generate `publish_{stem}`"),
        });
    }
    if refusals.is_empty() {
        obligations.push(
            "a failed publish is reported to the error callback and dropped; nothing retries it (delivery: at_most_once)"
                .to_owned(),
        );
        Ok(PublisherPlan {
            component: component.to_owned(),
            operations,
            obligations,
        })
    } else {
        Err(refusals)
    }
}

/// `UsageRecorded` → `usage_recorded`.
fn snake(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (index, character) in name.chars().enumerate() {
        if character.is_ascii_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.push(character.to_ascii_lowercase());
        } else if character == '-' {
            out.push('_');
        } else {
            out.push(character);
        }
    }
    out
}

/// `usage_recorded` → `UsageRecorded`; `producer-service` → `ProducerService`.
fn camel(name: &str) -> String {
    name.split(['_', '-', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_ascii_uppercase().to_string() + characters.as_str()
            })
        })
        .collect()
}

fn upper(stem: &str) -> String {
    stem.to_ascii_uppercase()
}

fn report(plan: &PublisherPlan, transport: &TransportIr, target: &str) -> String {
    let format = if plan
        .operations
        .iter()
        .any(|operation| !operation.parameters.is_empty())
    {
        PARAMETERIZED_CLIENT_REPORT_FORMAT
    } else {
        CLIENT_REPORT_FORMAT
    };
    let document = serde_json::json!({
        "format": format,
        "target": target,
        "component": plan.component,
        "transport": transport.specification(),
        "operations": plan.operations,
        "obligations": plan.obligations,
        "types_report": "types-report.json",
    });
    let mut text = serde_json::to_string_pretty(&document).expect("the report serializes");
    text.push('\n');
    text
}

fn parameter_access(
    ir: &EssIr,
    event_name: &str,
    parameters: &BTreeMap<String, ParameterSource>,
) -> BTreeMap<String, ParameterAccess> {
    if parameters.is_empty() {
        return BTreeMap::new();
    }
    let event = ir
        .events()
        .values()
        .find(|event| event.name.to_string() == event_name)
        .unwrap_or_else(|| unreachable!("publisher operation names a resolved event"));
    parameters
        .iter()
        .map(|(name, source)| {
            let wire = parameter_wire_path(ir, event, source.event_path());
            let rust = wire
                .iter()
                .map(|field| rust_field_name(field))
                .collect::<Vec<_>>()
                .join(".");
            let go = wire
                .iter()
                .map(|field| go_field_name(field))
                .collect::<Vec<_>>()
                .join(".");
            let go_required_structs = wire[..wire.len() - 1]
                .iter()
                .scan(String::new(), |prefix, field| {
                    if !prefix.is_empty() {
                        prefix.push('.');
                    }
                    prefix.push_str(&go_field_name(field));
                    Some(prefix.clone())
                })
                .collect();
            (
                name.clone(),
                ParameterAccess {
                    rust,
                    go,
                    go_required_structs,
                },
            )
        })
        .collect()
}

fn parameter_wire_path(ir: &EssIr, event: &ResolvedEvent, path: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(path.len());
    let mut field = event
        .field(&path[0])
        .unwrap_or_else(|| unreachable!("transport compilation resolved the event path"));
    out.push(
        field
            .naming
            .wire
            .as_deref()
            .unwrap_or(&field.name)
            .to_owned(),
    );
    for member in &path[1..] {
        let ResolvedTypeRef::Declared { name } = &field.type_ref else {
            unreachable!("transport compilation admitted only required struct intermediates")
        };
        let ResolvedBody::Struct { fields, .. } = &ir.named_type(name).body else {
            unreachable!("transport compilation admitted only required struct intermediates")
        };
        field = fields
            .iter()
            .find(|field| field.name == *member)
            .unwrap_or_else(|| unreachable!("transport compilation resolved the event path"));
        out.push(
            field
                .naming
                .wire
                .as_deref()
                .unwrap_or(&field.name)
                .to_owned(),
        );
    }
    out
}

fn rust_field_name(wire: &str) -> String {
    let mut result = String::new();
    let mut lower = false;
    for character in wire.chars() {
        if !character.is_ascii_alphanumeric() && character != '_' {
            if !result.ends_with('_') {
                result.push('_');
            }
            lower = false;
        } else {
            if character.is_ascii_uppercase() && lower {
                result.push('_');
            }
            result.push(character.to_ascii_lowercase());
            lower = character.is_ascii_lowercase() || character.is_ascii_digit();
        }
    }
    let name = result.trim_matches('_');
    match name {
        "self" | "super" | "crate" => format!("{name}_value"),
        "as" | "async" | "await" | "break" | "const" | "continue" | "dyn" | "else"
        | "enum" | "extern" | "false" | "fn" | "for" | "if" | "impl" | "in" | "let"
        | "loop" | "match" | "mod" | "move" | "mut" | "pub" | "ref" | "return"
        | "static" | "struct" | "trait" | "true" | "type" | "unsafe" | "use" | "where"
        | "while" | "abstract" | "become" | "box" | "do" | "final" | "macro"
        | "override" | "priv" | "typeof" | "unsized" | "virtual" | "yield" | "try"
        | "gen" => format!("r#{name}"),
        _ => name.to_owned(),
    }
}

fn go_field_name(wire: &str) -> String {
    let mut result = String::new();
    for part in wire
        .split(|character: char| !character.is_ascii_alphanumeric())
        .filter(|part| !part.is_empty())
    {
        let mut characters = part.chars();
        result.push(
            characters
                .next()
                .expect("a nonempty field-name part")
                .to_ascii_uppercase(),
        );
        result.push_str(characters.as_str());
    }
    result
}

const RUST_SUPPORT: &str = include_str!("rust_support.rs.txt");
const RUST_DYNAMIC_SUPPORT: &str = include_str!("rust_dynamic_support.rs.txt");
const RUST_NATS: &str = include_str!("rust_nats.rs.txt");
const GO_SUPPORT: &str = include_str!("go_support.go.txt");
const GO_DYNAMIC_SUPPORT: &str = include_str!("go_dynamic_support.go.txt");
const GO_NATS: &str = include_str!("go_natsjs.go.txt");

/// Renders the Rust publisher beside a realized Rust types library.
///
/// `manifest` is the types library's `Cargo.toml` for `package`; the returned files replace it
/// and add `lib.rs`, the `nats/` adapter crate and `client-report.json`. The types declarations
/// stay in `types.rs`.
pub fn rust(
    plan: &PublisherPlan,
    transport: &TransportIr,
    package: &str,
    manifest: &str,
) -> Result<BTreeMap<String, String>, Refusal> {
    let manifest = rust_manifest(manifest)?;
    let publisher = format!("{}Publisher", camel(&plan.component));
    let dynamic = plan
        .operations
        .iter()
        .any(|operation| !operation.parameters.is_empty());
    let mut lib = String::new();
    let _ = writeln!(
        lib,
        "// @generated by ESS {}; do not edit.\n//! The `{}` publisher: one operation per event its transport binds.\n\nmod types;\npub use types::*;\n",
        env!("CARGO_PKG_VERSION"),
        plan.component
    );
    lib.push_str(if dynamic {
        RUST_DYNAMIC_SUPPORT
    } else {
        RUST_SUPPORT
    });
    for op in &plan.operations {
        let name = upper(&op.stem);
        if op.parameters.is_empty() {
            let _ = writeln!(
                lib,
                "\n/// The subject `{}` is published to.\npub const {name}_SUBJECT: &str = {:?};",
                op.event, op.subject
            );
        } else {
            let _ = writeln!(
                lib,
                "\n/// The subject template `{}` is rendered from each payload.\npub const {name}_SUBJECT_TEMPLATE: &str = {:?};",
                op.event, op.subject
            );
        }
        if let Some(batch) = op.batch {
            let _ = writeln!(
                lib,
                "/// Flush `{}` when this many payloads are buffered.\npub const {name}_MAX_ITEMS: usize = {};\n/// Flush `{}` when the oldest buffered payload has waited this long.\npub const {name}_MAX_DELAY: ::std::time::Duration = ::std::time::Duration::from_millis({});",
                op.event, batch.max_items, op.event, batch.max_delay_ms
            );
        }
        if let Some(stream) = &op.stream {
            if op.parameters.is_empty() {
                let _ = writeln!(
                    lib,
                    "/// The stream capturing `{name}_SUBJECT`. The publisher never creates, updates or deletes it.\npub const {name}_STREAM: &str = {stream:?};"
                );
            } else {
                let _ = writeln!(
                    lib,
                    "/// The stream capturing every expansion of `{name}_SUBJECT_TEMPLATE`. The publisher never creates, updates or deletes it.\npub const {name}_STREAM: &str = {stream:?};"
                );
            }
        }
        if !op.parameters.is_empty() {
            rust_subject_renderer(&mut lib, op);
        }
    }
    let _ = writeln!(
        lib,
        "\n/// Publishes what `{}` publishes.\npub struct {publisher} {{\n    #[allow(dead_code)]\n    sink: Sink,",
        plan.component
    );
    for op in plan
        .operations
        .iter()
        .filter(|op| op.envelope == Envelope::Array)
    {
        let _ = writeln!(lib, "    {}: Batcher,", op.stem);
    }
    let _ = writeln!(
        lib,
        "    #[allow(dead_code)]\n    on_error: Option<ErrorCallback>,\n}}\n\nimpl {publisher} {{\n    /// A publisher over `transport`. Every `array` channel starts its flusher thread here.\n    pub fn new<T: Transport>(transport: T, options: PublisherOptions) -> Self {{\n        let sink = sink(transport);\n        Self {{"
    );
    for op in plan
        .operations
        .iter()
        .filter(|op| op.envelope == Envelope::Array)
    {
        let name = upper(&op.stem);
        let (items, delay) = match op.batch {
            Some(_) => (format!("{name}_MAX_ITEMS"), format!("{name}_MAX_DELAY")),
            None => (
                "1".to_owned(),
                "::std::time::Duration::from_millis(0)".to_owned(),
            ),
        };
        let subject = if op.parameters.is_empty() {
            format!("{name}_SUBJECT")
        } else {
            format!("{name}_SUBJECT_TEMPLATE")
        };
        let _ = writeln!(
            lib,
            "            {}: Batcher::start({subject}, {items}, {delay}, sink.clone(), options.on_error.clone()),",
            op.stem,
        );
    }
    lib.push_str("            sink,\n            on_error: options.on_error,\n        }\n    }\n");
    for op in &plan.operations {
        rust_operation(&mut lib, op, dynamic);
    }
    let arrays: Vec<&Operation> = plan
        .operations
        .iter()
        .filter(|op| op.envelope == Envelope::Array)
        .collect();
    lib.push_str("\n    /// Publishes everything buffered now, returning the first failure.\n    pub fn flush(&self) -> ::std::result::Result<(), PublishError> {\n        let mut first = Ok(());\n");
    for op in &arrays {
        let _ = writeln!(
            lib,
            "        if let Err(error) = self.{}.flush() {{\n            if first.is_ok() {{\n                first = Err(error);\n            }}\n        }}",
            op.stem
        );
    }
    if dynamic {
        lib.push_str("        first\n    }\n\n    /// Publishes everything buffered, stops every flusher thread and waits for it.\n    pub fn close(self) -> ::std::result::Result<(), PublishError> {\n        let mut result = self.flush();\n");
    } else {
        lib.push_str("        first\n    }\n\n    /// Publishes everything buffered, stops every flusher thread and waits for it.\n    pub fn close(self) -> ::std::result::Result<(), PublishError> {\n        let result = self.flush();\n");
    }
    for op in &arrays {
        if dynamic {
            let _ = writeln!(lib, "        if let Err(error) = self.{}.stop() {{\n            if result.is_ok() {{\n                result = Err(error);\n            }}\n        }}", op.stem);
        } else {
            let _ = writeln!(lib, "        self.{}.stop();", op.stem);
        }
    }
    lib.push_str("        result\n    }\n}\n");

    let mut files = BTreeMap::new();
    let (nats_manifest, nats_source) = rust_nats(package);
    files.insert("Cargo.toml".to_owned(), manifest);
    files.insert("nats/Cargo.toml".to_owned(), nats_manifest);
    files.insert("nats/lib.rs".to_owned(), nats_source);
    files.insert("lib.rs".to_owned(), lib);
    files.insert(
        "client-report.json".to_owned(),
        report(plan, transport, "rust"),
    );
    Ok(files)
}

fn rust_subject_renderer(lib: &mut String, op: &Operation) {
    let stem = &op.stem;
    let ty = &op.payload_type;
    let name = upper(stem);
    let _ = writeln!(
        lib,
        "\nfn {stem}_subject(payload: &{ty}) -> ::std::result::Result<String, PublishError> {{"
    );
    for (parameter, access) in &op.parameter_access {
        let _ = writeln!(
            lib,
            "    parameter_token({name}_SUBJECT_TEMPLATE, {parameter:?}, payload.{}.as_str())?;",
            access.rust
        );
    }
    lib.push_str("    let mut subject = String::new();\n");
    for (index, token) in op.subject.split('.').enumerate() {
        if index > 0 {
            lib.push_str("    subject.push('.');\n");
        }
        if let Some(parameter) = token
            .strip_prefix('{')
            .and_then(|token| token.strip_suffix('}'))
        {
            let _ = writeln!(
                lib,
                "    subject.push_str(payload.{}.as_str());",
                op.parameter_access[parameter].rust
            );
        } else {
            let _ = writeln!(lib, "    subject.push_str({token:?});");
        }
    }
    lib.push_str("    Ok(subject)\n}\n");
}

fn rust_operation(lib: &mut String, op: &Operation, dynamic_package: bool) {
    let name = upper(&op.stem);
    let ty = &op.payload_type;
    let stem = &op.stem;
    let dynamic = !op.parameters.is_empty();
    match op.envelope {
        Envelope::Single => {
            if dynamic {
                let _ = writeln!(
                    lib,
                    "\n    /// Publishes one `{}` on the subject rendered from its payload.\n    pub fn publish_{stem}(&self, payload: &{ty}) -> ::std::result::Result<(), PublishError> {{\n        let subject = {stem}_subject(payload)?;\n        let encoded = encode(&subject, payload)?;\n        (self.sink)(&subject, &encoded).map_err(|message| PublishError::new(subject, 1, message))\n    }}",
                    op.event
                );
            } else {
                let _ = writeln!(
                    lib,
                    "\n    /// Publishes one `{}` as one message on `{name}_SUBJECT`.\n    pub fn publish_{stem}(&self, payload: &{ty}) -> ::std::result::Result<(), PublishError> {{\n        let encoded = encode({name}_SUBJECT, payload)?;\n        (self.sink)({name}_SUBJECT, &encoded).map_err(|message| PublishError::new({name}_SUBJECT, 1, message))\n    }}",
                    op.event
                );
            }
        }
        Envelope::Array => {
            if dynamic {
                let _ = writeln!(
                    lib,
                    "\n    /// Buffers one `{}` in the bucket selected by its rendered subject.\n    pub fn publish_{stem}(&self, payload: &{ty}) -> ::std::result::Result<(), PublishError> {{\n        let subject = {stem}_subject(payload)?;\n        let encoded = encode(&subject, payload)?;\n        self.{stem}.enqueue(subject, encoded)\n    }}\n\n    /// Publishes `payloads` as one JSON array now when all select the same subject.\n    pub fn publish_{stem}_now(&self, payloads: &[{ty}]) -> ::std::result::Result<(), PublishError> {{\n        let Some(first) = payloads.first() else {{ return Ok(()); }};\n        let subject = {stem}_subject(first)?;\n        for payload in &payloads[1..] {{\n            if {stem}_subject(payload)? != subject {{\n                return Err(PublishError::new({name}_SUBJECT_TEMPLATE, payloads.len(), \"every payload in one array must render the same subject\"));\n            }}\n        }}\n        let encoded = payloads.iter().map(|payload| encode(&subject, payload)).collect::<::std::result::Result<Vec<_>, _>>()?;\n        self.{stem}.send(subject, encoded)\n    }}",
                    op.event
                );
            } else if dynamic_package {
                let _ = writeln!(
                    lib,
                    "\n    /// Buffers one `{}`; the flusher publishes the buffer as one JSON array on `{name}_SUBJECT`.\n    pub fn publish_{stem}(&self, payload: &{ty}) -> ::std::result::Result<(), PublishError> {{\n        let encoded = encode({name}_SUBJECT, payload)?;\n        self.{stem}.enqueue({name}_SUBJECT.to_owned(), encoded)\n    }}\n\n    /// Publishes `payloads` as one JSON array on `{name}_SUBJECT` now, bypassing the buffer.\n    pub fn publish_{stem}_now(&self, payloads: &[{ty}]) -> ::std::result::Result<(), PublishError> {{\n        let encoded = payloads.iter().map(|payload| encode({name}_SUBJECT, payload)).collect::<::std::result::Result<Vec<_>, _>>()?;\n        self.{stem}.send({name}_SUBJECT.to_owned(), encoded)\n    }}",
                    op.event
                );
            } else {
                let _ = writeln!(
                    lib,
                    "\n    /// Buffers one `{}`; the flusher publishes the buffer as one JSON array on `{name}_SUBJECT`.\n    pub fn publish_{stem}(&self, payload: &{ty}) -> ::std::result::Result<(), PublishError> {{\n        let encoded = encode({name}_SUBJECT, payload)?;\n        self.{stem}.enqueue(encoded)\n    }}\n\n    /// Publishes `payloads` as one JSON array on `{name}_SUBJECT` now, bypassing the buffer.\n    pub fn publish_{stem}_now(&self, payloads: &[{ty}]) -> ::std::result::Result<(), PublishError> {{\n        let encoded = payloads\n            .iter()\n            .map(|payload| encode({name}_SUBJECT, payload))\n            .collect::<::std::result::Result<Vec<_>, _>>()?;\n        self.{stem}.send(encoded)\n    }}",
                    op.event
                );
            }
        }
    }
}

fn rust_manifest(manifest: &str) -> Result<String, Refusal> {
    let refuse = |what: &str| Refusal {
        rule: "unexpected_types_manifest",
        detail: format!("the types library manifest has no {what}"),
    };
    if !manifest.contains("path = \"types.rs\"") {
        return Err(refuse("`path = \"types.rs\"`"));
    }
    Ok(manifest.replacen("path = \"types.rs\"", "path = \"lib.rs\"", 1))
}

/// The `JetStream` adapter crate, `nats/`: its own package so the core resolves offline.
fn rust_nats(package: &str) -> (String, String) {
    let manifest = format!(
        "[package]\nname = \"{package}-nats\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[lib]\npath = \"lib.rs\"\n\n[dependencies]\n{package} = {{ path = \"..\" }}\nasync-nats = \"0.38\"\ntokio = {{ version = \"1\", features = [\"rt\", \"rt-multi-thread\"] }}\n\n[workspace]\n"
    );
    let source = format!(
        "// @generated by ESS {}; do not edit.\n{}",
        env!("CARGO_PKG_VERSION"),
        RUST_NATS.replace("CORE", &package.replace('-', "_"))
    );
    (manifest, source)
}

/// Renders the Go publisher beside a realized Go types library in package `package` of `module`.
///
/// Returns `publisher.go`, the `natsjs/` adapter module and `client-report.json`; the types
/// library's `go.mod` and `types.go` stay as they are.
pub fn go(
    plan: &PublisherPlan,
    transport: &TransportIr,
    package: &str,
    module: &str,
) -> BTreeMap<String, String> {
    let publisher = format!("{}Publisher", camel(&plan.component));
    let dynamic = plan
        .operations
        .iter()
        .any(|operation| !operation.parameters.is_empty());
    let mut src = String::new();
    let _ = writeln!(
        src,
        "// Code generated by ESS {}; DO NOT EDIT.\n\n// The `{}` publisher: one operation per event its transport binds.\npackage {package}\n",
        env!("CARGO_PKG_VERSION"),
        plan.component
    );
    src.push_str(if dynamic { GO_DYNAMIC_SUPPORT } else { GO_SUPPORT });
    for op in &plan.operations {
        let name = camel(&op.stem);
        if op.parameters.is_empty() {
            let _ = writeln!(
                src,
                "\n// {name}Subject is the subject `{}` is published to.\nconst {name}Subject = {:?}",
                op.event, op.subject
            );
        } else {
            let _ = writeln!(
                src,
                "\n// {name}SubjectTemplate is rendered from each `{}` payload.\nconst {name}SubjectTemplate = {:?}",
                op.event, op.subject
            );
        }
        if let Some(batch) = op.batch {
            let _ = writeln!(
                src,
                "\n// {name}MaxItems and {name}MaxDelay are when `{}` flushes.\nconst (\n\t{name}MaxItems = {}\n\t{name}MaxDelay = {} * time.Millisecond\n)",
                op.event, batch.max_items, batch.max_delay_ms
            );
        }
        if let Some(stream) = &op.stream {
            if op.parameters.is_empty() {
                let _ = writeln!(
                    src,
                    "\n// {name}Stream captures {name}Subject. The publisher never creates, updates or deletes it.\nconst {name}Stream = {stream:?}"
                );
            } else {
                let _ = writeln!(
                    src,
                    "\n// {name}Stream captures every expansion of {name}SubjectTemplate. The publisher never creates, updates or deletes it.\nconst {name}Stream = {stream:?}"
                );
            }
        }
        if !op.parameters.is_empty() {
            go_subject_renderer(&mut src, op);
        }
    }
    let _ = writeln!(
        src,
        "\n// {publisher} publishes what `{}` publishes.\ntype {publisher} struct {{\n\ttransport Transport",
        plan.component
    );
    for op in plan
        .operations
        .iter()
        .filter(|op| op.envelope == Envelope::Array)
    {
        let _ = writeln!(src, "\t{} *batcher", lower_camel(&op.stem));
    }
    let _ = writeln!(
        src,
        "}}\n\n// New{publisher} returns a publisher over transport.\nfunc New{publisher}(transport Transport, options PublisherOptions) *{publisher} {{\n\treturn &{publisher}{{\n\t\ttransport: transport,"
    );
    for op in plan
        .operations
        .iter()
        .filter(|op| op.envelope == Envelope::Array)
    {
        let name = camel(&op.stem);
        let (items, delay) = if op.batch.is_some() {
            (format!("{name}MaxItems"), format!("{name}MaxDelay"))
        } else {
            ("1".to_owned(), "0".to_owned())
        };
        let subject = if op.parameters.is_empty() {
            format!("{name}Subject")
        } else {
            format!("{name}SubjectTemplate")
        };
        let _ = writeln!(
            src,
            "\t\t{}: newBatcher({subject}, {items}, {delay}, transport, options.OnError),",
            lower_camel(&op.stem),
        );
    }
    src.push_str("\t}\n}\n");
    for op in &plan.operations {
        go_operation(&mut src, &publisher, op, dynamic);
    }
    go_lifecycle(&mut src, &publisher, plan);

    let mut files = BTreeMap::new();
    files.insert("publisher.go".to_owned(), src);
    files.insert(
        "natsjs/go.mod".to_owned(),
        format!(
            "module {module}/natsjs\n\ngo 1.23.0\n\nrequire github.com/nats-io/nats.go v1.48.0\n"
        ),
    );
    files.insert(
        "natsjs/natsjs.go".to_owned(),
        format!(
            "// Code generated by ESS {}; DO NOT EDIT.\n\n{GO_NATS}",
            env!("CARGO_PKG_VERSION")
        ),
    );
    files.insert(
        "client-report.json".to_owned(),
        report(plan, transport, "go"),
    );
    files
}

/// `Flush` and `Close`, over every `array` channel.
fn go_lifecycle(src: &mut String, publisher: &str, plan: &PublisherPlan) {
    let arrays: Vec<&Operation> = plan
        .operations
        .iter()
        .filter(|op| op.envelope == Envelope::Array)
        .collect();
    for (method, doc, call) in [
        (
            "Flush",
            "Flush publishes everything buffered now and returns the first failure.",
            "flush",
        ),
        (
            "Close",
            "Close publishes everything buffered, stops accepting payloads and waits for in-flight flushes.",
            "close",
        ),
    ] {
        let _ = writeln!(
            src,
            "\n// {doc}\nfunc (p *{publisher}) {method}(ctx context.Context) error {{\n\tvar first error"
        );
        for op in &arrays {
            let _ = writeln!(
                src,
                "\tif err := p.{}.{call}(ctx); err != nil && first == nil {{\n\t\tfirst = err\n\t}}",
                lower_camel(&op.stem)
            );
        }
        src.push_str("\treturn first\n}\n");
    }
}

fn lower_camel(stem: &str) -> String {
    let camel = camel(stem);
    let mut characters = camel.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_ascii_lowercase().to_string() + characters.as_str()
    })
}

fn go_subject_renderer(src: &mut String, op: &Operation) {
    let name = camel(&op.stem);
    let function = lower_camel(&format!("{}_subject", op.stem));
    let ty = &op.payload_type;
    let _ = writeln!(src, "\nfunc {function}(payload {ty}) (string, error) {{");
    for (parameter, access) in &op.parameter_access {
        for required in &access.go_required_structs {
            let _ = writeln!(
                src,
                "\tif payload.{required} == nil {{ return \"\", &PublishError{{Subject: {name}SubjectTemplate, Items: 1, Err: fmt.Errorf(\"address parameter %q crosses a nil required struct\", {parameter:?})}} }}"
            );
        }
        let _ = writeln!(
            src,
            "\tif err := parameterToken({name}SubjectTemplate, {parameter:?}, payload.{}); err != nil {{ return \"\", err }}",
            access.go
        );
    }
    src.push_str("\tvar subject strings.Builder\n");
    for (index, token) in op.subject.split('.').enumerate() {
        if index > 0 {
            src.push_str("\tsubject.WriteByte('.')\n");
        }
        if let Some(parameter) = token
            .strip_prefix('{')
            .and_then(|token| token.strip_suffix('}'))
        {
            let _ = writeln!(
                src,
                "\tsubject.WriteString(payload.{})",
                op.parameter_access[parameter].go
            );
        } else {
            let _ = writeln!(src, "\tsubject.WriteString({token:?})");
        }
    }
    src.push_str("\treturn subject.String(), nil\n}\n");
}

fn go_operation(src: &mut String, publisher: &str, op: &Operation, dynamic_package: bool) {
    let name = camel(&op.stem);
    let field = lower_camel(&op.stem);
    let ty = &op.payload_type;
    let function = lower_camel(&format!("{}_subject", op.stem));
    let dynamic = !op.parameters.is_empty();
    match op.envelope {
        Envelope::Single => {
            if dynamic {
                let _ = writeln!(
                    src,
                    "\n// Publish{name} publishes one `{}` on the subject rendered from its payload.\nfunc (p *{publisher}) Publish{name}(ctx context.Context, payload {ty}) error {{\n\tsubject, err := {function}(payload)\n\tif err != nil {{ return err }}\n\tencoded, err := json.Marshal(payload)\n\tif err != nil {{ return &PublishError{{Subject: subject, Items: 1, Err: err}} }}\n\tif err := p.transport.Publish(ctx, subject, encoded); err != nil {{ return &PublishError{{Subject: subject, Items: 1, Err: err}} }}\n\treturn nil\n}}",
                    op.event
                );
            } else {
                let _ = writeln!(
                    src,
                    "\n// Publish{name} publishes one `{}` as one message on {name}Subject.\nfunc (p *{publisher}) Publish{name}(ctx context.Context, payload {ty}) error {{\n\tencoded, err := json.Marshal(payload)\n\tif err != nil {{\n\t\treturn &PublishError{{Subject: {name}Subject, Items: 1, Err: err}}\n\t}}\n\tif err := p.transport.Publish(ctx, {name}Subject, encoded); err != nil {{\n\t\treturn &PublishError{{Subject: {name}Subject, Items: 1, Err: err}}\n\t}}\n\treturn nil\n}}",
                    op.event
                );
            }
        }
        Envelope::Array => {
            if dynamic {
                let _ = writeln!(
                    src,
                    "\n// Publish{name} buffers one `{}` in the bucket selected by its rendered subject.\nfunc (p *{publisher}) Publish{name}(ctx context.Context, payload {ty}) error {{\n\tsubject, err := {function}(payload)\n\tif err != nil {{ return err }}\n\tencoded, err := json.Marshal(payload)\n\tif err != nil {{ return &PublishError{{Subject: subject, Items: 1, Err: err}} }}\n\treturn p.{field}.enqueue(subject, encoded)\n}}\n\n// Publish{name}Now publishes one array when every payload selects the same subject.\nfunc (p *{publisher}) Publish{name}Now(ctx context.Context, payloads []{ty}) error {{\n\tif len(payloads) == 0 {{ return nil }}\n\tsubject, err := {function}(payloads[0])\n\tif err != nil {{ return err }}\n\tfor _, payload := range payloads[1:] {{\n\t\tfound, err := {function}(payload)\n\t\tif err != nil {{ return err }}\n\t\tif found != subject {{ return &PublishError{{Subject: {name}SubjectTemplate, Items: len(payloads), Err: errors.New(\"every payload in one array must render the same subject\")}} }}\n\t}}\n\tencoded := make([][]byte, 0, len(payloads))\n\tfor _, payload := range payloads {{ data, err := json.Marshal(payload); if err != nil {{ return &PublishError{{Subject: subject, Items: len(payloads), Err: err}} }}; encoded = append(encoded, data) }}\n\treturn p.{field}.sendNow(ctx, subject, encoded)\n}}",
                    op.event
                );
            } else if dynamic_package {
                let _ = writeln!(
                    src,
                    "\n// Publish{name} buffers one `{}`; a flush publishes the buffer as one JSON array on {name}Subject.\nfunc (p *{publisher}) Publish{name}(ctx context.Context, payload {ty}) error {{\n\tencoded, err := json.Marshal(payload)\n\tif err != nil {{ return &PublishError{{Subject: {name}Subject, Items: 1, Err: err}} }}\n\treturn p.{field}.enqueue({name}Subject, encoded)\n}}\n\n// Publish{name}Now publishes payloads as one JSON array on {name}Subject now, bypassing the buffer.\nfunc (p *{publisher}) Publish{name}Now(ctx context.Context, payloads []{ty}) error {{\n\tencoded := make([][]byte, 0, len(payloads))\n\tfor _, payload := range payloads {{ data, err := json.Marshal(payload); if err != nil {{ return &PublishError{{Subject: {name}Subject, Items: len(payloads), Err: err}} }}; encoded = append(encoded, data) }}\n\treturn p.{field}.sendNow(ctx, {name}Subject, encoded)\n}}",
                    op.event
                );
            } else {
                let _ = writeln!(
                    src,
                    "\n// Publish{name} buffers one `{}`; a flush publishes the buffer as one JSON array on {name}Subject.\nfunc (p *{publisher}) Publish{name}(ctx context.Context, payload {ty}) error {{\n\tencoded, err := json.Marshal(payload)\n\tif err != nil {{\n\t\treturn &PublishError{{Subject: {name}Subject, Items: 1, Err: err}}\n\t}}\n\treturn p.{field}.enqueue(encoded)\n}}\n\n// Publish{name}Now publishes payloads as one JSON array on {name}Subject now, bypassing the buffer.\nfunc (p *{publisher}) Publish{name}Now(ctx context.Context, payloads []{ty}) error {{\n\tencoded := make([][]byte, 0, len(payloads))\n\tfor _, payload := range payloads {{\n\t\tdata, err := json.Marshal(payload)\n\t\tif err != nil {{\n\t\t\treturn &PublishError{{Subject: {name}Subject, Items: len(payloads), Err: err}}\n\t\t}}\n\t\tencoded = append(encoded, data)\n\t}}\n\treturn p.{field}.sendNow(ctx, encoded)\n}}",
                    op.event
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{camel, lower_camel, snake};

    #[test]
    fn names_derive_from_the_event_and_component() {
        assert_eq!(snake("UsageRecorded"), "usage_recorded");
        assert_eq!(camel("usage_recorded"), "UsageRecorded");
        assert_eq!(camel("producer-service"), "ProducerService");
        assert_eq!(lower_camel("usage_recorded"), "usageRecorded");
    }
}
