//! A generated event publisher for one component, over the channels an `ess-transport/1` document
//! binds (beyond10x/ess#395; `docs/design/event-publishers.md`).
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

use ess_compiler::EssIr;
use ess_domain::binding::Delivery;
use ess_transport::{Batch, Envelope, Owner, TransportIr};

/// The report format written beside a generated publisher.
pub const CLIENT_REPORT_FORMAT: &str = "ess-client-report/1";

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
    let document = serde_json::json!({
        "format": CLIENT_REPORT_FORMAT,
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

const RUST_SUPPORT: &str = include_str!("rust_support.rs.txt");
const RUST_NATS: &str = include_str!("rust_nats.rs.txt");
const GO_SUPPORT: &str = include_str!("go_support.go.txt");
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
    let mut lib = String::new();
    let _ = writeln!(
        lib,
        "// @generated by ESS {}; do not edit.\n//! The `{}` publisher: one operation per event its transport binds.\n\nmod types;\npub use types::*;\n",
        env!("CARGO_PKG_VERSION"),
        plan.component
    );
    lib.push_str(RUST_SUPPORT);
    for op in &plan.operations {
        let name = upper(&op.stem);
        let _ = writeln!(
            lib,
            "\n/// The subject `{}` is published to.\npub const {name}_SUBJECT: &str = {:?};",
            op.event, op.subject
        );
        if let Some(batch) = op.batch {
            let _ = writeln!(
                lib,
                "/// Flush `{}` when this many payloads are buffered.\npub const {name}_MAX_ITEMS: usize = {};\n/// Flush `{}` when the oldest buffered payload has waited this long.\npub const {name}_MAX_DELAY: ::std::time::Duration = ::std::time::Duration::from_millis({});",
                op.event, batch.max_items, op.event, batch.max_delay_ms
            );
        }
        if let Some(stream) = &op.stream {
            let _ = writeln!(
                lib,
                "/// The stream capturing `{name}_SUBJECT`. The publisher never creates, updates or deletes it.\npub const {name}_STREAM: &str = {stream:?};"
            );
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
        let _ = writeln!(
            lib,
            "            {}: Batcher::start({name}_SUBJECT, {items}, {delay}, sink.clone(), options.on_error.clone()),",
            op.stem
        );
    }
    lib.push_str("            sink,\n            on_error: options.on_error,\n        }\n    }\n");
    for op in &plan.operations {
        rust_operation(&mut lib, op);
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
    lib.push_str("        first\n    }\n\n    /// Publishes everything buffered, stops every flusher thread and waits for it.\n    pub fn close(self) -> ::std::result::Result<(), PublishError> {\n        let result = self.flush();\n");
    for op in &arrays {
        let _ = writeln!(lib, "        self.{}.stop();", op.stem);
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

fn rust_operation(lib: &mut String, op: &Operation) {
    let name = upper(&op.stem);
    let ty = &op.payload_type;
    let stem = &op.stem;
    match op.envelope {
        Envelope::Single => {
            let _ = writeln!(
                lib,
                "\n    /// Publishes one `{}` as one message on `{name}_SUBJECT`.\n    pub fn publish_{stem}(&self, payload: &{ty}) -> ::std::result::Result<(), PublishError> {{\n        let encoded = encode({name}_SUBJECT, payload)?;\n        (self.sink)({name}_SUBJECT, &encoded).map_err(|message| PublishError::new({name}_SUBJECT, 1, message))\n    }}",
                op.event
            );
        }
        Envelope::Array => {
            let _ = writeln!(
                lib,
                "\n    /// Buffers one `{}`; the flusher publishes the buffer as one JSON array on `{name}_SUBJECT`.\n    pub fn publish_{stem}(&self, payload: &{ty}) -> ::std::result::Result<(), PublishError> {{\n        let encoded = encode({name}_SUBJECT, payload)?;\n        self.{stem}.enqueue(encoded)\n    }}\n\n    /// Publishes `payloads` as one JSON array on `{name}_SUBJECT` now, bypassing the buffer.\n    pub fn publish_{stem}_now(&self, payloads: &[{ty}]) -> ::std::result::Result<(), PublishError> {{\n        let encoded = payloads\n            .iter()\n            .map(|payload| encode({name}_SUBJECT, payload))\n            .collect::<::std::result::Result<Vec<_>, _>>()?;\n        self.{stem}.send(encoded)\n    }}",
                op.event
            );
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
    let mut src = String::new();
    let _ = writeln!(
        src,
        "// Code generated by ESS {}; DO NOT EDIT.\n\n// The `{}` publisher: one operation per event its transport binds.\npackage {package}\n",
        env!("CARGO_PKG_VERSION"),
        plan.component
    );
    src.push_str(GO_SUPPORT);
    for op in &plan.operations {
        let name = camel(&op.stem);
        let _ = writeln!(
            src,
            "\n// {name}Subject is the subject `{}` is published to.\nconst {name}Subject = {:?}",
            op.event, op.subject
        );
        if let Some(batch) = op.batch {
            let _ = writeln!(
                src,
                "\n// {name}MaxItems and {name}MaxDelay are when `{}` flushes.\nconst (\n\t{name}MaxItems = {}\n\t{name}MaxDelay = {} * time.Millisecond\n)",
                op.event, batch.max_items, batch.max_delay_ms
            );
        }
        if let Some(stream) = &op.stream {
            let _ = writeln!(
                src,
                "\n// {name}Stream captures {name}Subject. The publisher never creates, updates or deletes it.\nconst {name}Stream = {stream:?}"
            );
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
        let _ = writeln!(
            src,
            "\t\t{}: newBatcher({name}Subject, {items}, {delay}, transport, options.OnError),",
            lower_camel(&op.stem)
        );
    }
    src.push_str("\t}\n}\n");
    for op in &plan.operations {
        go_operation(&mut src, &publisher, op);
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

fn go_operation(src: &mut String, publisher: &str, op: &Operation) {
    let name = camel(&op.stem);
    let field = lower_camel(&op.stem);
    let ty = &op.payload_type;
    match op.envelope {
        Envelope::Single => {
            let _ = writeln!(
                src,
                "\n// Publish{name} publishes one `{}` as one message on {name}Subject.\nfunc (p *{publisher}) Publish{name}(ctx context.Context, payload {ty}) error {{\n\tencoded, err := json.Marshal(payload)\n\tif err != nil {{\n\t\treturn &PublishError{{Subject: {name}Subject, Items: 1, Err: err}}\n\t}}\n\tif err := p.transport.Publish(ctx, {name}Subject, encoded); err != nil {{\n\t\treturn &PublishError{{Subject: {name}Subject, Items: 1, Err: err}}\n\t}}\n\treturn nil\n}}",
                op.event
            );
        }
        Envelope::Array => {
            let _ = writeln!(
                src,
                "\n// Publish{name} buffers one `{}`; a flush publishes the buffer as one JSON array on {name}Subject.\nfunc (p *{publisher}) Publish{name}(ctx context.Context, payload {ty}) error {{\n\tencoded, err := json.Marshal(payload)\n\tif err != nil {{\n\t\treturn &PublishError{{Subject: {name}Subject, Items: 1, Err: err}}\n\t}}\n\treturn p.{field}.enqueue(encoded)\n}}\n\n// Publish{name}Now publishes payloads as one JSON array on {name}Subject now, bypassing the buffer.\nfunc (p *{publisher}) Publish{name}Now(ctx context.Context, payloads []{ty}) error {{\n\tencoded := make([][]byte, 0, len(payloads))\n\tfor _, payload := range payloads {{\n\t\tdata, err := json.Marshal(payload)\n\t\tif err != nil {{\n\t\t\treturn &PublishError{{Subject: {name}Subject, Items: len(payloads), Err: err}}\n\t\t}}\n\t\tencoded = append(encoded, data)\n\t}}\n\treturn p.{field}.sendNow(ctx, encoded)\n}}",
                op.event
            );
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
