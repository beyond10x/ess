//! A field whose name begins with an underscore (beyond10x/ess#141), emitted for every target.
//!
//! The wire keeps `_url`. Rust keeps the identifier `_url`, which is a valid field name there. Go
//! cannot export a name that starts with an underscore, so `_url` pascal-cases to `Url` like every
//! other field — and beside a field called `url`, which also wants `Url`, one of the two is moved
//! by the repair every generated struct takes (`Url_`, in declaration order). What this file holds
//! is that every place a Go emitter spells a struct member uses the repaired spelling: a member
//! access spelled from the specification name alone reads the wrong field, or one that does not
//! exist.

use std::collections::BTreeMap;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

const ORDERS: &str = r"
format: ess/14
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.OrderView
    kind: struct
    fields:
      - {name: _url, type: Optional<String>}
      - {name: url, type: String}
      - {name: id, type: String}
events:
  - name: demo.orders.Linked
    fields:
      - {name: _url, type: String}
      - {name: url, type: String}
      - {name: view, type: demo.orders.OrderView}
commands:
  - name: demo.orders.Link
    input:
      - {name: _url, type: String}
      - {name: url, type: String}
      - {name: view, type: demo.orders.OrderView}
    outcomes:
      - name: linked
        emits: [demo.orders.Linked]
        payload:
          demo.orders.Linked:
            _url: {generated: true}
            url: {generated: true}
            view: {generated: true}
      - name: moved
        external: the link target answers with a redirect
        error: demo.orders.Moved
errors:
  - name: demo.orders.Moved
    fields:
      - {name: _url, type: String}
      - {name: url, type: String}
components:
  - component: orders-service
    owns: {domains: [demo.orders]}
    accepts: {commands: [demo.orders.Link]}
    publishes: {events: [demo.orders.Linked]}
    reached_by: network
";

/// A binding that copies, and converts, an event field whose Go spelling was repaired.
const RELAY: &str = r"
format: ess/14
system: relay
version: v1
domain: relay.core
types:
  - {name: relay.core.AccountId, kind: newtype, of: String}
  - {name: relay.core.AccountRef, kind: newtype, of: String}
conversions:
  - from: relay.core.AccountId
    to: relay.core.AccountRef
    because: an account may be referred to by its id rendered as text.
events:
  - name: relay.core.Fired
    fields:
      - {name: _url, type: String}
      - {name: url, type: String}
      - {name: _account, type: relay.core.AccountId}
      - {name: account, type: relay.core.AccountId}
  - name: relay.core.Handled
    fields:
      - {name: account, type: relay.core.AccountRef}
commands:
  - name: relay.core.Handle
    input:
      - {name: url, type: String}
      - {name: account, type: relay.core.AccountRef}
    outcomes:
      - name: done
        emits: [relay.core.Handled]
        payload:
          relay.core.Handled:
            account: input.account
components:
  - component: relay-service
    owns: {domains: [relay.core]}
    accepts: {commands: [relay.core.Handle]}
    publishes: {events: [relay.core.Handled]}
bindings:
  - id: relay-on-fired
    when: {event: relay.core.Fired}
    invoke: {command: relay.core.Handle}
    mapping:
      url: event.url
      account: event.account
    delivery: at_least_once
    on_failure: retry
";

fn emitted(source: &str, target: Target) -> BTreeMap<String, String> {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(source).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", source);
    let ir = compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    synthesize_for(&ir, target)
        .expect("a realizable target")
        .artifacts
        .into_iter()
        .map(|(path, artifact)| (path, artifact.contents))
        .collect()
}

fn artifact<'a>(artifacts: &'a BTreeMap<String, String>, path: &str) -> &'a str {
    artifacts.get(path).unwrap_or_else(|| {
        panic!(
            "no `{path}` among {:?}",
            artifacts.keys().collect::<Vec<_>>()
        )
    })
}

#[test]
fn rust_keeps_the_underscore_identifier_and_the_wire_keeps_the_name() {
    let rust = emitted(ORDERS, Target::Rust);
    let types = artifact(&rust, "crates/demo-types/src/orders.rs");
    assert!(types.contains("pub _url: Option<String>,"), "{types}");
    assert!(types.contains("pub url: String,"), "{types}");
    let wire = artifact(&rust, "crates/demo-server/src/wire.rs");
    assert!(wire.contains("json::member(out, \"_url\");"), "{wire}");
    assert!(wire.contains("value._url"), "{wire}");
}

#[test]
fn go_exports_the_underscore_field_and_repairs_the_collision_it_makes() {
    let go = emitted(ORDERS, Target::Go);
    let types = artifact(&go, "types/orders/orders.go");
    assert!(
        types.contains("\t// Url is `_url` — `Optional<String>`.\n\tUrl *string\n"),
        "{types}"
    );
    assert!(
        types.contains("\t// Url_ is `url` — `String`.\n\tUrl_ string\n"),
        "{types}"
    );
    assert!(
        !types.lines().any(|line| line.starts_with("\t_")),
        "no Go field is spelled with a leading underscore:\n{types}"
    );
}

#[test]
fn go_wire_code_reads_and_writes_the_repaired_member() {
    let go = emitted(ORDERS, Target::Go);
    let wire = artifact(&go, "server/wire.go");
    // The struct, the event, the error and the command input each hold `_url` as `Url` and `url`
    // as `Url_`. The event's encoder is the one a command's `published` list is written with.
    assert_eq!(
        wire.matches("out[\"url\"] = value.Url_\n").count(),
        3,
        "the struct, the event and the error encoders write `url` from `Url_`:\n{wire}"
    );
    assert!(
        wire.contains("func encodeEventDemoOrdersLinked("),
        "the published event has its encoder:\n{wire}"
    );
    assert!(
        !wire.contains("out[\"url\"] = value.Url\n"),
        "`url` is not read from `_url`'s field:\n{wire}"
    );
    assert_eq!(
        wire.matches("out.Url_ = ").count(),
        2,
        "the struct and command decoders fill `Url_` from `url`:\n{wire}"
    );
    assert!(wire.contains("\"_url\""), "the wire keeps `_url`:\n{wire}");
}

#[test]
fn go_generated_transformation_copies_and_converts_from_the_repaired_member() {
    let go = emitted(RELAY, Target::Go);
    let system = artifact(&go, "system/system.go");
    assert!(system.contains("Url: event.Url_,"), "{system}");
    assert!(
        system.contains("(event.Account_)"),
        "the conversion reads `account`, not `_account`:\n{system}"
    );
    assert!(!system.contains("Url: event.Url,"), "{system}");
}
