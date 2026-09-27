//! Adversary pass 1 against `story:field-names-underscore-and-newtype-map-keys`.
//!
//! Acceptance for beyond10x/ess#141 is that the repro validates *and generated code compiles for
//! it*. The unit's own synthesis cases only search the emitted text. These compile it: the Go
//! emission with `go vet`, the Rust emission with `cargo check`, and each with the colliding field
//! declared both before and after `_url`, with different types, so a member read from the wrong
//! field is a type error instead of a silent wrong value.

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};

/// `url` before `_url`, typed differently, in every declaration kind that holds fields.
const ORDERS_URL_FIRST: &str = r"
format: ess/14
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.OrderView
    kind: struct
    fields:
      - {name: url, type: Integer}
      - {name: _url, type: Optional<String>}
      - {name: id, type: String}
events:
  - name: demo.orders.Linked
    fields:
      - {name: url, type: Integer}
      - {name: _url, type: String}
      - {name: view, type: demo.orders.OrderView}
commands:
  - name: demo.orders.Link
    input:
      - {name: url, type: Integer}
      - {name: _url, type: String}
      - {name: view, type: demo.orders.OrderView}
    outcomes:
      - name: linked
        emits: [demo.orders.Linked]
        payload:
          demo.orders.Linked:
            url: {generated: true}
            _url: {generated: true}
            view: {generated: true}
      - name: moved
        external: the link target answers with a redirect
        error: demo.orders.Moved
errors:
  - name: demo.orders.Moved
    fields:
      - {name: url, type: Integer}
      - {name: _url, type: String}
components:
  - component: orders-service
    owns: {domains: [demo.orders]}
    accepts: {commands: [demo.orders.Link]}
    publishes: {events: [demo.orders.Linked]}
    reached_by: network
";

/// The same with `_url` first: the order the unit's own fixture uses.
fn orders_underscore_first() -> String {
    ORDERS_URL_FIRST
        .replace(
            "      - {name: url, type: Integer}\n      - {name: _url, type: Optional<String>}\n",
            "      - {name: _url, type: Optional<String>}\n      - {name: url, type: Integer}\n",
        )
        .replace(
            "      - {name: url, type: Integer}\n      - {name: _url, type: String}\n",
            "      - {name: _url, type: String}\n      - {name: url, type: Integer}\n",
        )
}

/// A binding copying `_url` and converting `account`, each beside a differently typed twin.
const RELAY_URL_FIRST: &str = r"
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
      - {name: url, type: Integer}
      - {name: _url, type: String}
      - {name: account, type: relay.core.AccountId}
      - {name: _account, type: Integer}
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
      url: event._url
      account: event.account
    delivery: at_least_once
    on_failure: retry
";

/// #141's repro, verbatim, in a system.
const ISSUE_141: &str = r"
format: ess/14
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.OrderView
    kind: struct
    fields:
      - {name: _url, type: Optional<String>}
      - {name: id, type: String}
";

fn emit(source: &str, target: Target, directory: &Path) {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(source).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", source);
    let ir = compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    let synthesis = synthesize_for(&ir, target).expect("a realizable target");
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().unwrap()).unwrap();
        std::fs::write(&destination, &artifact.contents).unwrap();
    }
}

fn scratch(label: &str) -> PathBuf {
    Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("adversary-names-pass1-{label}"))
}

fn go_modules(directory: &Path, found: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            go_modules(&path, found);
        } else if path.file_name().is_some_and(|name| name == "go.mod") {
            found.push(directory.to_path_buf());
        }
    }
}

fn go_vet(source: &str, label: &str) {
    let directory = scratch(label);
    emit(source, Target::Go, &directory);
    let mut modules = Vec::new();
    go_modules(&directory, &mut modules);
    assert!(!modules.is_empty(), "the Go emission carries a module");
    for module in modules {
        let output = Command::new("go")
            .args(["vet", "./..."])
            .current_dir(&module)
            .env(
                "GOCACHE",
                Path::new(env!("CARGO_TARGET_TMPDIR")).join("gocache"),
            )
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off")
            .env("GOTOOLCHAIN", "local")
            .output()
            .expect("the Go toolchain runs");
        assert!(
            output.status.success(),
            "`go vet` refuses the emission in {}:\n{}{}",
            module.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn go_compiles_with_url_declared_before_underscore_url() {
    go_vet(ORDERS_URL_FIRST, "go-url-first");
}

#[test]
fn go_compiles_with_underscore_url_declared_first() {
    go_vet(&orders_underscore_first(), "go-underscore-first");
}

#[test]
fn go_compiles_a_binding_reading_underscore_fields_beside_their_twins() {
    go_vet(RELAY_URL_FIRST, "go-relay");
}

#[test]
fn go_compiles_the_issue_141_repro() {
    go_vet(ISSUE_141, "go-141");
}

/// An entity identified by `_id` beside a differently typed `id`, and a view projecting both.
const DOCS: &str = r"
format: ess/14
system: demo
version: v1
domain: demo.docs
entities:
  - name: demo.docs.Doc
    identity: {name: _id, type: String}
    fields:
      - {name: id, type: Integer}
      - {name: _rev, type: Integer}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
views:
  - name: demo.docs.Docs
    source: demo.docs.Doc
    consistency: eventual
    fields:
      - {name: id, type: Integer}
      - {name: _id, type: String}
components:
  - component: docs-service
    owns: {domains: [demo.docs]}
";

#[test]
fn go_compiles_an_entity_identified_by_underscore_id() {
    go_vet(DOCS, "go-docs");
}

#[test]
fn rust_compiles_the_issue_141_repro_and_the_colliding_twins() {
    for (label, source) in [
        ("rust-141", ISSUE_141.to_owned()),
        ("rust-url-first", ORDERS_URL_FIRST.to_owned()),
        ("rust-docs", DOCS.to_owned()),
    ] {
        let directory = scratch(label);
        emit(&source, Target::Rust, &directory);
        let cargo = std::env::var_os("CARGO").expect("Cargo supplies its executable");
        let lock = Command::new(&cargo)
            .args([
                "generate-lockfile",
                "--offline",
                "--manifest-path",
                "Cargo.toml",
            ])
            .current_dir(&directory)
            .env_remove("CARGO_TARGET_DIR")
            .output()
            .unwrap();
        assert!(
            lock.status.success(),
            "{}",
            String::from_utf8_lossy(&lock.stderr)
        );
        let check = Command::new(&cargo)
            .args([
                "check",
                "--locked",
                "--offline",
                "--workspace",
                "--all-targets",
                "--manifest-path",
                "Cargo.toml",
                "--target-dir",
            ])
            .arg(scratch("rust-target"))
            .current_dir(&directory)
            .env_remove("CARGO_TARGET_DIR")
            .output()
            .unwrap();
        assert!(
            check.status.success(),
            "{label}: the generated Rust workspace does not compile:\n{}",
            String::from_utf8_lossy(&check.stderr)
        );
    }
}
