//! Generated publishers compile offline and behave as `docs/design/event-publishers.md` says,
//! against an in-memory transport (beyond10x/ess#395).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::Read as _;
use std::net::{SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::schema::ModelTypes;
use ess_publisher::{plan, PublisherPlan};
use ess_transport::{TransportIr, TransportSpec};
use sha2::{Digest as _, Sha256};

const MODEL: &str = "format: ess/20
system: metering
version: v1
domain: metering.items
types:
  - name: metering.items.Measurement
    kind: struct
    fields:
      - {name: amount, type: Integer}
      - {name: unit, type: String}
    invariants:
      - amount >= -2147483648
      - amount <= 2147483647
events:
  - name: metering.items.UsageRecorded
    naming: {wire: usage}
    fields:
      - {name: id, type: String}
      - {name: usage, type: 'List<metering.items.Measurement>'}
      - {name: created_at, type: Timestamp, wire: createdAt}
  - name: metering.items.Audited
    naming: {wire: audit}
    fields:
      - {name: id, type: String}
commands:
  - name: metering.items.RecordUsage
    input:
      - {name: id, type: String}
    outcomes:
      - name: recorded
        emits: [metering.items.UsageRecorded, metering.items.Audited]
        payload:
          metering.items.UsageRecorded: {id: input.id, usage: {generated: true}, created_at: {generated: true}}
          metering.items.Audited: {id: input.id}
components:
  - component: producer
    owns: {domains: [metering.items]}
    accepts: {commands: [metering.items.RecordUsage]}
    publishes: {events: [metering.items.UsageRecorded, metering.items.Audited]}
";

const TRANSPORT: &str = "brokers:
  - {id: events, protocol: nats, jetstream: true}
channels:
  - event: metering.items.UsageRecorded
    broker: events
    subject: usage
    envelope: array
    delivery: at_most_once
    batch: {max_items: 2, max_delay_ms: 60000}
  - event: metering.items.Audited
    broker: events
    subject: audit
    envelope: single
    delivery: at_most_once
streams:
  - {name: USAGE, broker: events, subjects: [usage, audit], storage: file, retention: limits, owner: external}
";

const PARAMETER_MODEL: &str = "format: ess/20
system: routing
version: v1
domain: routing.events
types:
  - name: routing.events.Source
    kind: struct
    fields:
      - {name: service, wire: serviceName, type: String}
      - {name: environment, type: String}
events:
  - name: routing.events.UsageRecorded
    fields:
      - {name: id, type: String}
      - {name: source, wire: origin, type: routing.events.Source}
  - name: routing.events.Audited
    fields:
      - {name: id, type: String}
  - name: routing.events.Routed
    fields:
      - {name: id, type: String}
      - {name: source, wire: origin, type: routing.events.Source}
components:
  - component: producer
    owns: {domains: [routing.events]}
    publishes: {events: [routing.events.UsageRecorded, routing.events.Audited, routing.events.Routed]}
";

const PARAMETER_TRANSPORT: &str = "brokers:
  - {id: events, protocol: nats, jetstream: true}
channels:
  - event: routing.events.UsageRecorded
    broker: events
    subject: 'usage.{service}.{again}.{environment}'
    parameters:
      service: event.source.service
      again: event.source.service
      environment: event.source.environment
    envelope: array
    delivery: at_most_once
    batch: {max_items: 2, max_delay_ms: 50}
streams:
  - {name: USAGE, broker: events, subjects: ['usage.>'], storage: file, retention: limits, owner: external}
";

fn model() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("metering.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("metering.yaml", MODEL);
    compile(&spec, &sources).expect("compiles")
}

fn transport(ir: &EssIr, body: &str) -> TransportIr {
    let text = format!(
        "type: ess-transport/1\nspecification:\n  system: metering\n  version: v1\n  source_digest: sha256:{}\n{body}",
        ir.source_digest()
    );
    ess_transport::compile(&TransportSpec::from_yaml(&text).expect("parses"), ir).expect("compiles")
}

struct Generated {
    plan: PublisherPlan,
    types: schema_contract::realize::Plan,
    transport: TransportIr,
}

fn generated(body: &str) -> Generated {
    let ir = model();
    let transport = transport(&ir, body);
    let roots: BTreeSet<String> = PublisherPlan::roots(&ir, "producer", &transport)
        .into_iter()
        .collect();
    let selection = ModelTypes::select(&ir, &roots).expect("selects");
    let types = schema_contract::realize::Plan::from_model(&selection).expect("plans");
    let report =
        serde_json::to_value(types.rust("metering-client").expect("rust").report).expect("report");
    let declarations: BTreeMap<String, String> =
        serde_json::from_value(report["declarations"].clone()).expect("declarations");
    let plan = plan(&ir, "producer", &transport, &declarations).expect("plans a publisher");
    Generated {
        plan,
        types,
        transport,
    }
}

fn parameterized_generated() -> Generated {
    let body = PARAMETER_TRANSPORT
        .replace(
            "streams:\n",
            "  - event: routing.events.Audited\n    broker: events\n    subject: audit\n    envelope: array\n    delivery: at_most_once\n    batch: {max_items: 2, max_delay_ms: 50}\n  - event: routing.events.Routed\n    broker: events\n    subject: 'single.{service}'\n    parameters: {service: event.source.service}\n    envelope: single\n    delivery: at_most_once\nstreams:\n",
        )
        .replace(
            "subjects: ['usage.>']",
            "subjects: ['usage.>', audit, 'single.>']",
        );
    parameterized_generated_with(&body)
}

fn parameterized_generated_with(body: &str) -> Generated {
    let raw = RawSpecFile::parse(PARAMETER_MODEL).expect("well formed");
    let spec = Specification::assemble([(Source::new("routing.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("routing.yaml", PARAMETER_MODEL);
    let ir = compile(&spec, &sources).expect("compiles");
    let text = format!(
        "type: ess-transport/2\nspecification:\n  system: routing\n  version: v1\n  source_digest: sha256:{}\n{body}",
        ir.source_digest()
    );
    let transport = ess_transport::compile(
        &TransportSpec::from_yaml(&text).expect("parses"),
        &ir,
    )
    .expect("compiles");
    let roots: BTreeSet<String> = PublisherPlan::roots(&ir, "producer", &transport)
        .into_iter()
        .collect();
    let selection = ModelTypes::select(&ir, &roots).expect("selects");
    let types = schema_contract::realize::Plan::from_model(&selection).expect("plans");
    let report =
        serde_json::to_value(types.rust("routing-client").expect("rust").report).expect("report");
    let declarations: BTreeMap<String, String> =
        serde_json::from_value(report["declarations"].clone()).expect("declarations");
    let plan = plan(&ir, "producer", &transport, &declarations).expect("plans a publisher");
    Generated {
        plan,
        types,
        transport,
    }
}

fn scratch(name: &str) -> PathBuf {
    let root =
        Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("scratch");
    root
}

fn write(root: &Path, files: &BTreeMap<String, String>) {
    for (path, contents) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().expect("a parent")).expect("directory");
        fs::write(path, contents).expect("write");
    }
}

fn released_sha256(text: &str) -> String {
    // A release bump deliberately changes generated banners; normalize only that independent
    // version fact before comparing the transport/1 generator bytes released by 0.52.0.
    let normalized = text.replace(env!("CARGO_PKG_VERSION"), "0.52.0");
    format!("{:x}", Sha256::digest(normalized.as_bytes()))
}

#[test]
fn transport_1_publisher_files_keep_the_released_0_52_0_bytes() {
    let generated = generated(TRANSPORT);
    let rust_types = generated.types.rust("metering-client").expect("rust");
    let rust = ess_publisher::rust(
        &generated.plan,
        &generated.transport,
        "metering-client",
        &rust_types.supporting["Cargo.toml"],
    )
    .expect("renders");
    let rust_expected = BTreeMap::from([
        ("Cargo.toml", "2325520266d68aaf5339e092b86afcb13c776f1f778669d5a34b3f05ee884a76"),
        ("client-report.json", "3574a3a961a5931f3e47652769a0fa22c5ad6558580f2be6d98781420b82146b"),
        ("lib.rs", "5eadda7a48de2e8d7379966fc0e176509a31d0d5f2096d4cf2aed002b03fb096"),
        ("nats/Cargo.toml", "4b476cba08b6e6b649480fca5e0670ca3523bf7e7d7a83407612ac20405d07f3"),
        ("nats/lib.rs", "55a802b681debfa59947ba750c08c54e979bb7cf89a6ea7ebdd6264e3360e6fa"),
    ]);
    assert_eq!(rust.keys().map(String::as_str).collect::<Vec<_>>(), rust_expected.keys().copied().collect::<Vec<_>>());
    for (path, expected) in rust_expected {
        assert_eq!(released_sha256(&rust[path]), expected, "released Rust bytes changed at {path}");
    }

    let module = "example.invalid/meteringclient";
    let go = ess_publisher::go(&generated.plan, &generated.transport, "meteringclient", module);
    let go_expected = BTreeMap::from([
        ("client-report.json", "949ac1792bfda5155bc0ccf719816eb2cd8d62165ad3e35226dd0eef7c19c70e"),
        ("natsjs/go.mod", "45766bb361d6be8847dc60b8f09ded6006172381f40ea580b3700882e8ab2059"),
        ("natsjs/natsjs.go", "65ff37bbd70686810aafa7229856bedadd38e9b35f3040f23a190d0ecef137be"),
        ("publisher.go", "05ee3855d25072526660e37caf2f89fa3c018e2c652f0275330def37b46ac4a9"),
    ]);
    assert_eq!(go.keys().map(String::as_str).collect::<Vec<_>>(), go_expected.keys().copied().collect::<Vec<_>>());
    for (path, expected) in go_expected {
        assert_eq!(released_sha256(&go[path]), expected, "released Go bytes changed at {path}");
    }
}

#[test]
fn the_plan_has_one_operation_per_bound_published_event() {
    let generated = generated(TRANSPORT);
    let stems: Vec<&str> = generated
        .plan
        .operations
        .iter()
        .map(|op| op.stem.as_str())
        .collect();
    assert_eq!(stems, ["audited", "usage_recorded"]);
}

const RUST_TEST: &str = r##"
use std::sync::{Arc, Mutex};
use metering_client::*;

#[derive(Clone, Default)]
struct Memory {
    sent: Arc<Mutex<Vec<(String, String)>>>,
    fail: bool,
}

impl Transport for Memory {
    type Error = String;
    fn publish(&self, subject: &str, payload: &[u8]) -> Result<(), String> {
        if self.fail {
            return Err("refused".to_owned());
        }
        self.sent.lock().unwrap().push((subject.to_owned(), String::from_utf8(payload.to_vec()).unwrap()));
        Ok(())
    }
}

fn item(id: &str) -> MeteringItemsUsageRecorded {
    serde_json::from_str(&format!(r#"{{"id":"{id}","usage":[{{"amount":1,"unit":"count"}}],"createdAt":"2026-10-03T00:00:00Z"}}"#)).unwrap()
}

fn audited(id: &str) -> MeteringItemsAudited {
    serde_json::from_str(&format!(r#"{{"id":"{id}"}}"#)).unwrap()
}

fn wait_for(memory: &Memory, count: usize) {
    for _ in 0..200 {
        if memory.sent.lock().unwrap().len() >= count {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("{count} message(s) never arrived: {:?}", memory.sent.lock().unwrap());
}

#[test]
fn an_array_channel_flushes_at_max_items_and_on_close() {
    assert_eq!(USAGE_RECORDED_SUBJECT, "usage");
    assert_eq!(USAGE_RECORDED_MAX_ITEMS, 2);
    assert_eq!(USAGE_RECORDED_STREAM, "USAGE");
    let memory = Memory::default();
    let publisher = ProducerPublisher::new(memory.clone(), PublisherOptions::default());
    publisher.publish_usage_recorded(&item("a")).unwrap();
    publisher.publish_usage_recorded(&item("b")).unwrap();
    wait_for(&memory, 1);
    publisher.publish_usage_recorded(&item("c")).unwrap();
    publisher.close().unwrap();
    let sent = memory.sent.lock().unwrap().clone();
    assert_eq!(sent.len(), 2, "{sent:?}");
    let first: serde_json::Value = serde_json::from_str(&sent[0].1).unwrap();
    assert_eq!(sent[0].0, "usage");
    assert_eq!(first.as_array().unwrap().len(), 2);
    assert_eq!(first[0]["createdAt"], "2026-10-03T00:00:00Z");
    let last: serde_json::Value = serde_json::from_str(&sent[1].1).unwrap();
    assert_eq!(last[0]["id"], "c");
}

#[test]
fn now_and_single_publish_directly() {
    let memory = Memory::default();
    let publisher = ProducerPublisher::new(memory.clone(), PublisherOptions::default());
    publisher.publish_usage_recorded_now(&[item("x")]).unwrap();
    publisher.publish_audited(&audited("y")).unwrap();
    let sent = memory.sent.lock().unwrap().clone();
    assert_eq!(sent[0], ("usage".to_owned(), sent[0].1.clone()));
    assert!(sent[0].1.starts_with('['), "{}", sent[0].1);
    assert_eq!(sent[1], ("audit".to_owned(), r#"{"id":"y"}"#.to_owned()));
    publisher.close().unwrap();
}

#[test]
fn a_failed_background_flush_reaches_the_callback_and_is_dropped() {
    let failures = Arc::new(Mutex::new(Vec::new()));
    let seen = failures.clone();
    let options = PublisherOptions {
        on_error: Some(Arc::new(move |error: PublishError| seen.lock().unwrap().push(error))),
    };
    let publisher = ProducerPublisher::new(Memory { fail: true, ..Memory::default() }, options);
    publisher.publish_usage_recorded(&item("a")).unwrap();
    publisher.publish_usage_recorded(&item("b")).unwrap();
    for _ in 0..200 {
        if !failures.lock().unwrap().is_empty() {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let failures = failures.lock().unwrap().clone();
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert_eq!(failures[0].items, 2);
    assert_eq!(failures[0].subject, "usage");
    assert!(publisher.publish_audited(&audited("z")).is_err());
    publisher.close().unwrap();
}

#[derive(Clone, Default)]
struct Slow {
    sent: Arc<Mutex<Vec<String>>>,
}

impl Transport for Slow {
    type Error = String;
    fn publish(&self, _subject: &str, payload: &[u8]) -> Result<(), String> {
        let first = self.sent.lock().unwrap().is_empty();
        if first {
            std::thread::sleep(std::time::Duration::from_millis(300));
        }
        self.sent.lock().unwrap().push(String::from_utf8(payload.to_vec()).unwrap());
        Ok(())
    }
}

#[test]
fn no_message_carries_more_than_max_items_when_the_broker_is_slow() {
    let slow = Slow::default();
    let publisher = ProducerPublisher::new(slow.clone(), PublisherOptions::default());
    for index in 0..7 {
        publisher.publish_usage_recorded(&item(&format!("i{index}"))).unwrap();
    }
    publisher.close().unwrap();
    let sizes: Vec<usize> = slow
        .sent
        .lock()
        .unwrap()
        .iter()
        .map(|message| serde_json::from_str::<Vec<serde_json::Value>>(message).unwrap().len())
        .collect();
    assert_eq!(sizes.iter().sum::<usize>(), 7, "{sizes:?}");
    assert!(sizes.iter().all(|size| *size <= USAGE_RECORDED_MAX_ITEMS), "max_items is {USAGE_RECORDED_MAX_ITEMS}, messages carried {sizes:?}");
}

"##;

#[test]
fn the_rust_publisher_compiles_offline_and_behaves() {
    let generated = generated(TRANSPORT);
    let types = generated.types.rust("metering-client").expect("rust");
    let mut files = ess_publisher::rust(
        &generated.plan,
        &generated.transport,
        "metering-client",
        &types.supporting["Cargo.toml"],
    )
    .expect("renders");
    files.insert("types.rs".to_owned(), types.declarations);
    files.insert("tests/publisher.rs".to_owned(), RUST_TEST.to_owned());
    assert!(files.contains_key("nats/lib.rs") && files.contains_key("nats/Cargo.toml"));
    assert!(
        !files["Cargo.toml"].contains("async-nats"),
        "the core needs no NATS dependency"
    );
    let root = scratch("generated-rust-publisher");
    write(&root, &files);
    let output = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("generated-publisher-target"),
        )
        .output()
        .expect("cargo runs");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const GO_TEST: &str = r#"package meteringclient

import (
	"context"
	"encoding/json"
	"errors"
	"sync"
	"testing"
	"time"
)

type memory struct {
	mu   sync.Mutex
	sent [][2]string
	fail bool
}

func (m *memory) Publish(_ context.Context, subject string, payload []byte) error {
	if m.fail {
		return errors.New("refused")
	}
	m.mu.Lock()
	defer m.mu.Unlock()
	m.sent = append(m.sent, [2]string{subject, string(payload)})
	return nil
}

func (m *memory) count() int {
	m.mu.Lock()
	defer m.mu.Unlock()
	return len(m.sent)
}

func item(t *testing.T, id string) MeteringItemsUsageRecorded {
	var v MeteringItemsUsageRecorded
	if err := json.Unmarshal([]byte(`{"id":"`+id+`","usage":[{"amount":1,"unit":"count"}],"createdAt":"2026-10-03T00:00:00Z"}`), &v); err != nil {
		t.Fatal(err)
	}
	return v
}

func TestArrayChannelFlushesAtMaxItemsAndOnClose(t *testing.T) {
	m := &memory{}
	p := NewProducerPublisher(m, PublisherOptions{})
	ctx := context.Background()
	for _, id := range []string{"a", "b", "c"} {
		if err := p.PublishUsageRecorded(ctx, item(t, id)); err != nil {
			t.Fatal(err)
		}
	}
	if err := p.Close(ctx); err != nil {
		t.Fatal(err)
	}
	if m.count() != 2 {
		t.Fatalf("sent %v", m.sent)
	}
	for _, sent := range m.sent {
		if sent[0] != UsageRecordedSubject {
			t.Fatalf("subject %q", sent[0])
		}
		var batch []map[string]any
		if err := json.Unmarshal([]byte(sent[1]), &batch); err != nil {
			t.Fatal(err)
		}
	}
	if err := p.PublishUsageRecorded(ctx, item(t, "d")); !errors.Is(err, ErrClosed) {
		t.Fatalf("after close: %v", err)
	}
}

func TestFailedFlushReachesOnError(t *testing.T) {
	var mu sync.Mutex
	var failures []error
	p := NewProducerPublisher(&memory{fail: true}, PublisherOptions{OnError: func(err error) {
		mu.Lock()
		defer mu.Unlock()
		failures = append(failures, err)
	}})
	ctx := context.Background()
	_ = p.PublishUsageRecorded(ctx, item(t, "a"))
	_ = p.PublishUsageRecorded(ctx, item(t, "b"))
	deadline := time.Now().Add(2 * time.Second)
	for time.Now().Before(deadline) {
		mu.Lock()
		n := len(failures)
		mu.Unlock()
		if n > 0 {
			break
		}
		time.Sleep(10 * time.Millisecond)
	}
	_ = p.Close(ctx)
	var publish *PublishError
	if len(failures) != 1 || !errors.As(failures[0], &publish) || publish.Items != 2 {
		t.Fatalf("failures %v", failures)
	}
	var audited MeteringItemsAudited
	_ = json.Unmarshal([]byte(`{"id":"z"}`), &audited)
	if err := p.PublishAudited(ctx, audited); err == nil {
		t.Fatal("a refused single publish returned no error")
	}
}

func TestPublishNowAfterCloseIsRefused(t *testing.T) {
	m := &memory{}
	p := NewProducerPublisher(m, PublisherOptions{})
	ctx := context.Background()
	if err := p.Close(ctx); err != nil {
		t.Fatal(err)
	}
	err := p.PublishUsageRecordedNow(ctx, []MeteringItemsUsageRecorded{item(t, "late")})
	if !errors.Is(err, ErrClosed) || m.count() != 0 {
		t.Fatalf("after close: err=%v, sent=%v", err, m.sent)
	}
}

"#;

#[test]
fn the_go_publisher_compiles_and_behaves() {
    if Command::new("go").arg("version").output().is_err() {
        println!("skipped: no `go` on PATH");
        return;
    }
    let generated = generated(TRANSPORT);
    let module = "example.invalid/meteringclient";
    let types = generated.types.go("meteringclient", module).expect("go");
    let mut files = ess_publisher::go(
        &generated.plan,
        &generated.transport,
        "meteringclient",
        module,
    );
    files.extend(types.supporting);
    files.insert("types.go".to_owned(), types.declarations);
    files.insert("publisher_test.go".to_owned(), GO_TEST.to_owned());
    let root = scratch("generated-go-publisher");
    write(&root, &files);
    let output = Command::new("go")
        .args(["test", "-race", "."])
        .current_dir(&root)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .expect("go runs");
    let text = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    if text.contains("-race requires cgo") {
        let output = Command::new("go")
            .args(["test", "."])
            .current_dir(&root)
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off")
            .output()
            .expect("go runs");
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    assert!(output.status.success(), "{text}");
}

#[test]
fn at_least_once_is_refused_by_name() {
    let ir = model();
    let transport = transport(
        &ir,
        &TRANSPORT.replace(
            "delivery: at_most_once\n    batch",
            "delivery: at_least_once\n    batch",
        ),
    );
    let declarations = BTreeMap::from([
        ("metering.items.UsageRecorded".to_owned(), "A".to_owned()),
        ("metering.items.Audited".to_owned(), "B".to_owned()),
    ]);
    let refusals = plan(&ir, "producer", &transport, &declarations).expect_err("refused");
    assert_eq!(refusals[0].rule, "unsupported_delivery");
}

const PARAMETER_RUST_TEST: &str = r##"
use std::sync::{Arc, Mutex};
use routing_client::*;

#[derive(Clone, Default)]
struct Memory { sent: Arc<Mutex<Vec<(String, String)>>> }
impl Transport for Memory {
    type Error = String;
    fn publish(&self, subject: &str, payload: &[u8]) -> Result<(), String> {
        self.sent.lock().unwrap().push((subject.to_owned(), String::from_utf8(payload.to_vec()).unwrap()));
        Ok(())
    }
}
fn item(id: &str, service: &str, environment: &str) -> RoutingEventsUsageRecorded {
    serde_json::from_str(&format!(r#"{{"id":"{id}","origin":{{"serviceName":"{service}","environment":"{environment}"}}}}"#)).unwrap()
}
fn routed(id: &str, service: &str, environment: &str) -> RoutingEventsRouted {
    serde_json::from_str(&format!(r#"{{"id":"{id}","origin":{{"serviceName":"{service}","environment":"{environment}"}}}}"#)).unwrap()
}
fn wait_for(memory: &Memory, count: usize) {
    for _ in 0..200 {
        if memory.sent.lock().unwrap().len() >= count { return; }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("{count} messages never arrived: {:?}", memory.sent.lock().unwrap());
}

#[test]
fn renders_aliases_and_never_mixes_subject_buckets() {
    assert_eq!(USAGE_RECORDED_SUBJECT_TEMPLATE, "usage.{service}.{again}.{environment}");
    let memory = Memory::default();
    let publisher = ProducerPublisher::new(memory.clone(), PublisherOptions::default());
    for payload in [item("a1", "service-a", "eu"), item("b1", "service-b", "us"), item("a2", "service-a", "eu"), item("b2", "service-b", "us")] {
        publisher.publish_usage_recorded(&payload).unwrap();
    }
    wait_for(&memory, 2);
    publisher.close().unwrap();
    let sent = memory.sent.lock().unwrap().clone();
    assert_eq!(sent.len(), 2, "{sent:?}");
    for (subject, body) in sent {
        let payloads: Vec<serde_json::Value> = serde_json::from_str(&body).unwrap();
        assert_eq!(payloads.len(), 2);
        for payload in payloads {
            let service = payload["origin"]["serviceName"].as_str().unwrap();
            let environment = payload["origin"]["environment"].as_str().unwrap();
            assert_eq!(subject, format!("usage.{service}.{service}.{environment}"));
        }
    }
}

#[test]
fn a_single_envelope_renders_and_publishes_one_payload() {
    let memory = Memory::default();
    let publisher = ProducerPublisher::new(memory.clone(), PublisherOptions::default());
    publisher.publish_routed(&routed("one", "service-a", "eu")).unwrap();
    publisher.close().unwrap();
    let sent = memory.sent.lock().unwrap();
    assert_eq!(sent.len(), 1, "{sent:?}");
    assert_eq!(sent[0].0, "single.service-a");
    assert!(!sent[0].1.starts_with('['), "{}", sent[0].1);
}

#[test]
fn refuses_invalid_and_mixed_subjects_before_io() {
    let memory = Memory::default();
    let publisher = ProducerPublisher::new(memory.clone(), PublisherOptions::default());
    publisher.publish_usage_recorded_now(&[]).unwrap();
    for invalid in ["", "bad token", "bad*token", "bad>token", "bad.token"] {
        let error = publisher.publish_usage_recorded(&item("bad", invalid, "eu")).unwrap_err();
        assert_eq!(error.subject, USAGE_RECORDED_SUBJECT_TEMPLATE);
        if !invalid.is_empty() {
            assert!(!error.message.contains(invalid), "{}", error.message);
        }
    }
    let error = publisher.publish_usage_recorded_now(&[item("a", "service-a", "eu"), item("b", "service-b", "us")]).unwrap_err();
    assert_eq!(error.subject, USAGE_RECORDED_SUBJECT_TEMPLATE);
    assert!(memory.sent.lock().unwrap().is_empty());
    publisher.close().unwrap();
}

#[test]
fn timer_flushes_and_explicit_flush_retires_unique_subjects() {
    let memory = Memory::default();
    let publisher = ProducerPublisher::new(memory.clone(), PublisherOptions::default());
    publisher.publish_usage_recorded(&item("timer", "timer", "eu")).unwrap();
    wait_for(&memory, 1);
    for index in 0..500 {
        publisher.publish_usage_recorded(&item(&format!("i{index}"), &format!("service-{index}"), "eu")).unwrap();
        publisher.flush().unwrap();
    }
    publisher.close().unwrap();
    assert_eq!(memory.sent.lock().unwrap().len(), 501);
}

#[derive(Clone, Default)]
struct Flaky {
    calls: Arc<::std::sync::atomic::AtomicUsize>,
    sent: Arc<Mutex<Vec<String>>>,
}
impl Transport for Flaky {
    type Error = String;
    fn publish(&self, _subject: &str, payload: &[u8]) -> Result<(), String> {
        if self.calls.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst) == 0 {
            return Err("first send refused".to_owned());
        }
        self.sent.lock().unwrap().push(String::from_utf8(payload.to_vec()).unwrap());
        Ok(())
    }
}

#[test]
fn a_failed_drain_retires_its_bucket_before_the_next_same_subject_batch() {
    let transport = Flaky::default();
    let failures = Arc::new(Mutex::new(Vec::new()));
    let seen = failures.clone();
    let publisher = ProducerPublisher::new(transport.clone(), PublisherOptions {
        on_error: Some(Arc::new(move |error| seen.lock().unwrap().push(error))),
    });
    publisher.publish_usage_recorded(&item("a", "service-a", "eu")).unwrap();
    publisher.publish_usage_recorded(&item("b", "service-a", "eu")).unwrap();
    for _ in 0..200 {
        if !failures.lock().unwrap().is_empty() { break; }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    publisher.publish_usage_recorded(&item("c", "service-a", "eu")).unwrap();
    publisher.publish_usage_recorded(&item("d", "service-a", "eu")).unwrap();
    publisher.close().unwrap();
    assert_eq!(failures.lock().unwrap().len(), 1);
    let sent = transport.sent.lock().unwrap();
    assert_eq!(sent.len(), 1, "{sent:?}");
    let payloads: Vec<serde_json::Value> = serde_json::from_str(&sent[0]).unwrap();
    assert_eq!(payloads[0]["id"], "c");
    assert_eq!(payloads[1]["id"], "d");
}

#[derive(Clone, Default)]
struct Slow {
    entered: Arc<::std::sync::atomic::AtomicBool>,
    release: Arc<::std::sync::atomic::AtomicBool>,
    calls: Arc<::std::sync::atomic::AtomicUsize>,
    sent: Arc<Mutex<Vec<String>>>,
}
impl Transport for Slow {
    type Error = String;
    fn publish(&self, _subject: &str, payload: &[u8]) -> Result<(), String> {
        if self.calls.fetch_add(1, ::std::sync::atomic::Ordering::SeqCst) == 0 {
            self.entered.store(true, ::std::sync::atomic::Ordering::SeqCst);
            while !self.release.load(::std::sync::atomic::Ordering::SeqCst) {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        self.sent.lock().unwrap().push(String::from_utf8(payload.to_vec()).unwrap());
        Ok(())
    }
}

#[test]
fn a_new_same_subject_bucket_waits_behind_its_in_flight_predecessor() {
    let transport = Slow::default();
    let publisher = Arc::new(ProducerPublisher::new(transport.clone(), PublisherOptions::default()));
    publisher.publish_usage_recorded(&item("a", "service-a", "eu")).unwrap();
    publisher.publish_usage_recorded(&item("b", "service-a", "eu")).unwrap();
    for _ in 0..200 {
        if transport.entered.load(::std::sync::atomic::Ordering::SeqCst) { break; }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(transport.entered.load(::std::sync::atomic::Ordering::SeqCst));
    let queued = publisher.clone();
    let finished = Arc::new(::std::sync::atomic::AtomicBool::new(false));
    let finished_by_worker = finished.clone();
    let thread = std::thread::spawn(move || {
        queued.publish_usage_recorded(&item("c", "service-a", "eu")).unwrap();
        queued.publish_usage_recorded(&item("d", "service-a", "eu")).unwrap();
        finished_by_worker.store(true, ::std::sync::atomic::Ordering::SeqCst);
    });
    std::thread::sleep(std::time::Duration::from_millis(20));
    assert!(!finished.load(::std::sync::atomic::Ordering::SeqCst));
    transport.release.store(true, ::std::sync::atomic::Ordering::SeqCst);
    thread.join().unwrap();
    Arc::try_unwrap(publisher).ok().expect("only publisher owner").close().unwrap();
    let sent = transport.sent.lock().unwrap();
    assert_eq!(sent.len(), 2, "{sent:?}");
    let first: Vec<serde_json::Value> = serde_json::from_str(&sent[0]).unwrap();
    let second: Vec<serde_json::Value> = serde_json::from_str(&sent[1]).unwrap();
    assert_eq!((first[0]["id"].as_str(), first[1]["id"].as_str()), (Some("a"), Some("b")));
    assert_eq!((second[0]["id"].as_str(), second[1]["id"].as_str()), (Some("c"), Some("d")));
}

#[test]
fn close_drains_subjects_in_lexical_order() {
    let memory = Memory::default();
    let publisher = ProducerPublisher::new(memory.clone(), PublisherOptions::default());
    publisher.publish_usage_recorded(&item("z", "z-service", "eu")).unwrap();
    publisher.publish_usage_recorded(&item("a", "a-service", "eu")).unwrap();
    publisher.close().unwrap();
    let subjects: Vec<String> = memory.sent.lock().unwrap().iter().map(|sent| sent.0.clone()).collect();
    assert_eq!(subjects, ["usage.a-service.a-service.eu", "usage.z-service.z-service.eu"]);
}
"##;

#[test]
fn the_parameterized_rust_publisher_compiles_offline_and_behaves() {
    let generated = parameterized_generated();
    let types = generated.types.rust("routing-client").expect("rust");
    let mut files = ess_publisher::rust(
        &generated.plan,
        &generated.transport,
        "routing-client",
        &types.supporting["Cargo.toml"],
    )
    .expect("renders");
    files.insert("types.rs".to_owned(), types.declarations);
    files.insert("tests/publisher.rs".to_owned(), PARAMETER_RUST_TEST.to_owned());
    let report: serde_json::Value = serde_json::from_str(&files["client-report.json"]).unwrap();
    assert_eq!(report["format"], "ess-client-report/2");
    let operations = report["operations"].as_array().expect("operations");
    let dynamic = operations
        .iter()
        .find(|operation| operation["event"] == "routing.events.UsageRecorded")
        .expect("dynamic operation");
    assert_eq!(
        dynamic["parameters"]["service"]["path"][1],
        "service"
    );
    let literal = operations
        .iter()
        .find(|operation| operation["event"] == "routing.events.Audited")
        .expect("literal operation");
    assert!(literal.get("parameters").is_none(), "{literal}");
    let root = scratch("generated-parameterized-rust-publisher");
    write(&root, &files);
    let output = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("generated-parameterized-publisher-target"),
        )
        .output()
        .expect("cargo runs");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const PARAMETER_GO_TEST: &str = r#"package routingclient

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"strings"
	"sync"
    "testing"
    "time"
)

type parameterMemory struct { mu sync.Mutex; sent [][2]string }
func (m *parameterMemory) Publish(_ context.Context, subject string, payload []byte) error {
    m.mu.Lock(); defer m.mu.Unlock()
    m.sent = append(m.sent, [2]string{subject, string(payload)})
    return nil
}
func (m *parameterMemory) count() int { m.mu.Lock(); defer m.mu.Unlock(); return len(m.sent) }
func parameterItem(t *testing.T, id, service, environment string) RoutingEventsUsageRecorded {
    var value RoutingEventsUsageRecorded
    if err := json.Unmarshal([]byte(`{"id":"`+id+`","origin":{"serviceName":"`+service+`","environment":"`+environment+`"}}`), &value); err != nil { t.Fatal(err) }
    return value
}
func routedItem(t *testing.T, id, service, environment string) RoutingEventsRouted {
    var value RoutingEventsRouted
    if err := json.Unmarshal([]byte(`{"id":"`+id+`","origin":{"serviceName":"`+service+`","environment":"`+environment+`"}}`), &value); err != nil { t.Fatal(err) }
    return value
}
func waitParameter(t *testing.T, memory *parameterMemory, count int) {
    deadline := time.Now().Add(2*time.Second)
    for time.Now().Before(deadline) { if memory.count() >= count { return }; time.Sleep(10*time.Millisecond) }
    t.Fatalf("wanted %d messages, found %v", count, memory.sent)
}
func TestParameterizedBucketsAndRefusals(t *testing.T) {
    memory := &parameterMemory{}
    publisher := NewProducerPublisher(memory, PublisherOptions{})
    ctx := context.Background()
    for _, value := range []RoutingEventsUsageRecorded{parameterItem(t,"a1","service-a","eu"), parameterItem(t,"b1","service-b","us"), parameterItem(t,"a2","service-a","eu"), parameterItem(t,"b2","service-b","us")} {
        if err := publisher.PublishUsageRecorded(ctx, value); err != nil { t.Fatal(err) }
    }
    waitParameter(t, memory, 2)
    if buckets := publisher.usageRecorded.bucketCount(); buckets != 0 { t.Fatalf("max-items retained %d buckets", buckets) }
    before := memory.count()
    if err := publisher.PublishUsageRecordedNow(ctx, nil); err != nil { t.Fatal(err) }
    for _, invalid := range []string{"", "bad token", "bad*token", "bad>token", "bad.token"} {
        if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"bad",invalid,"eu")); err == nil { t.Fatalf("invalid token %q accepted", invalid) } else if invalid != "" && strings.Contains(err.Error(), invalid) { t.Fatalf("error echoed invalid token: %v", err) }
    }
    if err := publisher.PublishUsageRecordedNow(ctx, []RoutingEventsUsageRecorded{parameterItem(t,"a","service-a","eu"), parameterItem(t,"b","service-b","us")}); err == nil { t.Fatal("mixed subjects accepted") }
    var missingRequiredStruct RoutingEventsUsageRecorded
    if err := publisher.PublishUsageRecorded(ctx, missingRequiredStruct); err == nil { t.Fatal("nil required address-source struct accepted") }
    if memory.count() != before { t.Fatalf("refusal performed I/O: %v", memory.sent) }
    for index := 0; index < 500; index++ {
        if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"unique",fmt.Sprintf("service-%d", index),"eu")); err != nil { t.Fatal(err) }
        if err := publisher.Flush(ctx); err != nil { t.Fatal(err) }
        if buckets := publisher.usageRecorded.bucketCount(); buckets != 0 { t.Fatalf("retained %d historical buckets", buckets) }
    }
    if err := publisher.Close(ctx); err != nil { t.Fatal(err) }
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"late","service-a","eu")); !errors.Is(err, ErrClosed) { t.Fatalf("after close: %v", err) }
    for _, sent := range memory.sent {
        var payloads []map[string]any
        if err := json.Unmarshal([]byte(sent[1]), &payloads); err != nil { t.Fatal(err) }
        for _, payload := range payloads {
            origin := payload["origin"].(map[string]any)
            service := origin["serviceName"].(string)
            environment := origin["environment"].(string)
            if sent[0] != "usage."+service+"."+service+"."+environment { t.Fatalf("subject %q payload %v", sent[0], payload) }
        }
    }
}

func TestDynamicSingleEnvelope(t *testing.T) {
    memory := &parameterMemory{}
    publisher := NewProducerPublisher(memory, PublisherOptions{})
    ctx := context.Background()
    if err := publisher.PublishRouted(ctx, routedItem(t,"one","service-a","eu")); err != nil { t.Fatal(err) }
    if err := publisher.Close(ctx); err != nil { t.Fatal(err) }
    if len(memory.sent) != 1 || memory.sent[0][0] != "single.service-a" || strings.HasPrefix(memory.sent[0][1], "[") { t.Fatalf("sent %v", memory.sent) }
}

func TestTimerDrainRetiresBucket(t *testing.T) {
    memory := &parameterMemory{}
    publisher := NewProducerPublisher(memory, PublisherOptions{})
    ctx := context.Background()
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"timer","timer","eu")); err != nil { t.Fatal(err) }
    waitParameter(t, memory, 1)
    if buckets := publisher.usageRecorded.bucketCount(); buckets != 0 { t.Fatalf("timer retained %d buckets", buckets) }
    if err := publisher.Close(ctx); err != nil { t.Fatal(err) }
}

type flakyParameterTransport struct { mu sync.Mutex; calls int; sent []string }
func (f *flakyParameterTransport) Publish(_ context.Context, _ string, payload []byte) error {
    f.mu.Lock(); defer f.mu.Unlock()
    f.calls++
    if f.calls == 1 { return errors.New("first send refused") }
    f.sent = append(f.sent, string(payload))
    return nil
}
func TestFailedDrainRetiresBucket(t *testing.T) {
    transport := &flakyParameterTransport{}
    failures := make(chan error, 1)
    publisher := NewProducerPublisher(transport, PublisherOptions{OnError: func(err error) { failures <- err }})
    ctx := context.Background()
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"a","service-a","eu")); err != nil { t.Fatal(err) }
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"b","service-a","eu")); err != nil { t.Fatal(err) }
    select { case <-failures: case <-time.After(2*time.Second): t.Fatal("background failure was not reported") }
    if buckets := publisher.usageRecorded.bucketCount(); buckets != 0 { t.Fatalf("failed send retained %d buckets", buckets) }
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"c","service-a","eu")); err != nil { t.Fatal(err) }
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"d","service-a","eu")); err != nil { t.Fatal(err) }
    if err := publisher.Close(ctx); err != nil { t.Fatal(err) }
    transport.mu.Lock(); defer transport.mu.Unlock()
    if len(transport.sent) != 1 { t.Fatalf("sent %v", transport.sent) }
    var payloads []map[string]any
    if err := json.Unmarshal([]byte(transport.sent[0]), &payloads); err != nil { t.Fatal(err) }
    if payloads[0]["id"] != "c" || payloads[1]["id"] != "d" { t.Fatalf("sent %v", payloads) }
}

type slowParameterTransport struct {
    once sync.Once
    entered chan struct{}
    release chan struct{}
    mu sync.Mutex
    sent []string
}
func (s *slowParameterTransport) Publish(_ context.Context, _ string, payload []byte) error {
    s.once.Do(func() { close(s.entered); <-s.release })
    s.mu.Lock(); defer s.mu.Unlock()
    s.sent = append(s.sent, string(payload))
    return nil
}
func TestSameSubjectFIFOAcrossInFlightDrain(t *testing.T) {
    transport := &slowParameterTransport{entered: make(chan struct{}), release: make(chan struct{})}
    publisher := NewProducerPublisher(transport, PublisherOptions{})
    ctx := context.Background()
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"a","service-a","eu")); err != nil { t.Fatal(err) }
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"b","service-a","eu")); err != nil { t.Fatal(err) }
    select { case <-transport.entered: case <-time.After(2*time.Second): t.Fatal("first batch did not enter transport") }
    done := make(chan error, 1)
    go func() {
        if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"c","service-a","eu")); err != nil { done <- err; return }
        done <- publisher.PublishUsageRecorded(ctx, parameterItem(t,"d","service-a","eu"))
    }()
    select { case err := <-done: t.Fatalf("new bucket passed in-flight predecessor: %v", err); case <-time.After(20*time.Millisecond): }
    close(transport.release)
    if err := <-done; err != nil { t.Fatal(err) }
    if err := publisher.Close(ctx); err != nil { t.Fatal(err) }
    transport.mu.Lock(); defer transport.mu.Unlock()
    if len(transport.sent) != 2 { t.Fatalf("sent %v", transport.sent) }
    var first, second []map[string]any
    if err := json.Unmarshal([]byte(transport.sent[0]), &first); err != nil { t.Fatal(err) }
    if err := json.Unmarshal([]byte(transport.sent[1]), &second); err != nil { t.Fatal(err) }
    if first[0]["id"] != "a" || first[1]["id"] != "b" || second[0]["id"] != "c" || second[1]["id"] != "d" { t.Fatalf("FIFO: %v then %v", first, second) }
}

func TestCloseDrainsSubjectsLexically(t *testing.T) {
    memory := &parameterMemory{}
    publisher := NewProducerPublisher(memory, PublisherOptions{})
    ctx := context.Background()
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"z","z-service","eu")); err != nil { t.Fatal(err) }
    if err := publisher.PublishUsageRecorded(ctx, parameterItem(t,"a","a-service","eu")); err != nil { t.Fatal(err) }
    if err := publisher.Close(ctx); err != nil { t.Fatal(err) }
    if len(memory.sent) != 2 || memory.sent[0][0] != "usage.a-service.a-service.eu" || memory.sent[1][0] != "usage.z-service.z-service.eu" { t.Fatalf("subjects %v", memory.sent) }
}
"#;

#[test]
fn the_parameterized_go_publisher_compiles_and_behaves() {
    let generated = parameterized_generated();
    let module = "example.invalid/routingclient";
    let types = generated.types.go("routingclient", module).expect("go");
    let mut files = ess_publisher::go(&generated.plan, &generated.transport, "routingclient", module);
    files.extend(types.supporting);
    files.insert("types.go".to_owned(), types.declarations);
    files.insert("publisher_test.go".to_owned(), PARAMETER_GO_TEST.to_owned());
    let root = scratch("generated-parameterized-go-publisher");
    write(&root, &files);
    let output = Command::new("go")
        .args(["test", "-race", "."])
        .current_dir(&root)
        .env("GOFLAGS", "-mod=mod")
        .env("GOPROXY", "off")
        .output()
        .expect("go runs");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

const NATS_RUST_MANIFEST: &str = r#"[package]
name = "parameterized-nats-acceptance"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
routing-client = { path = ".." }
routing-client-nats = { path = "../nats" }
async-nats = "=0.38.0"
tokio = { version = "=1.53.2", features = ["macros", "rt-multi-thread", "time"] }
futures-util = "=0.3.34"
serde_json = "=1.0.151"

[workspace]
"#;

const NATS_RUST_MAIN: &str = r##"use std::time::Duration;

use futures_util::StreamExt as _;
use routing_client::*;

fn item(id: &str, service: &str, environment: &str) -> RoutingEventsUsageRecorded {
    serde_json::from_str(&format!(r#"{{"id":"{id}","origin":{{"serviceName":"{service}","environment":"{environment}"}}}}"#)).unwrap()
}

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let url = std::env::var("ESS_TEST_NATS_URL").expect("ESS_TEST_NATS_URL");
    let client = async_nats::connect(url).await.unwrap();
    let jetstream = async_nats::jetstream::new(client.clone());
    let _ = jetstream.delete_stream("USAGE").await;
    jetstream.create_stream(async_nats::jetstream::stream::Config {
        name: "USAGE".to_owned(),
        subjects: vec!["usage.>".to_owned()],
        ..Default::default()
    }).await.unwrap();
    let mut stream = jetstream.get_stream("USAGE").await.unwrap();
    let before = stream.info().await.unwrap().config.subjects.clone();
    let mut subscription = client.subscribe("usage.>").await.unwrap();
    client.flush().await.unwrap();

    let transport = routing_client_nats::JetStreamTransport::new(jetstream.clone(), tokio::runtime::Handle::current());
    let publisher = ProducerPublisher::new(transport, PublisherOptions::default());
    for index in 0..150 {
        publisher.publish_usage_recorded(&item(&format!("a{index}"), "service-a", "eu")).unwrap();
        publisher.publish_usage_recorded(&item(&format!("b{index}"), "service-b", "us")).unwrap();
    }
    assert!(publisher.publish_usage_recorded(&item("bad", "bad.token", "eu")).is_err());
    assert!(publisher.publish_usage_recorded_now(&[item("mixed-a", "service-a", "eu"), item("mixed-b", "service-b", "us")]).is_err());
    publisher.close().unwrap();

    let mut messages = Vec::new();
    for _ in 0..4 {
        messages.push(tokio::time::timeout(Duration::from_secs(5), subscription.next()).await.unwrap().unwrap());
    }
    assert!(tokio::time::timeout(Duration::from_millis(250), subscription.next()).await.is_err());
    let mut items = 0;
    for message in messages {
        let payloads: Vec<serde_json::Value> = serde_json::from_slice(&message.payload).unwrap();
        assert!(payloads.len() <= 100);
        items += payloads.len();
        for payload in payloads {
            let service = payload["origin"]["serviceName"].as_str().unwrap();
            let environment = payload["origin"]["environment"].as_str().unwrap();
            assert_eq!(message.subject.as_str(), format!("usage.{service}.{service}.{environment}"));
        }
    }
    assert_eq!(items, 300);
    let after = stream.info().await.unwrap().config.subjects.clone();
    assert_eq!(before, after);
    println!("rust adapter: 300 items, 4 subject-separated messages, invalid and mixed inputs published nothing");
}
"##;

const NATS_GO_MANIFEST: &str = r#"module example.invalid/parameterized-acceptance

go 1.23.0

require (
    example.invalid/routingclient v0.0.0
    example.invalid/routingclient/natsjs v0.0.0
    github.com/nats-io/nats.go v1.48.0
)

replace example.invalid/routingclient => ../go
replace example.invalid/routingclient/natsjs => ../go/natsjs
"#;

const NATS_GO_MAIN: &str = r#"package main

import (
    "context"
    "encoding/json"
    "fmt"
    "os"
    "time"

    routing "example.invalid/routingclient"
    "example.invalid/routingclient/natsjs"
    "github.com/nats-io/nats.go"
    "github.com/nats-io/nats.go/jetstream"
)

func item(id, service, environment string) routing.RoutingEventsUsageRecorded {
    var value routing.RoutingEventsUsageRecorded
    raw := []byte(`{"id":"` + id + `","origin":{"serviceName":"` + service + `","environment":"` + environment + `"}}`)
    if err := json.Unmarshal(raw, &value); err != nil { panic(err) }
    return value
}

func main() {
    url := os.Getenv("ESS_TEST_NATS_URL")
    if url == "" { panic("ESS_TEST_NATS_URL") }
    connection, err := nats.Connect(url)
    if err != nil { panic(err) }
    defer connection.Close()
    js, err := jetstream.New(connection)
    if err != nil { panic(err) }
    ctx := context.Background()
    stream, err := js.Stream(ctx, "USAGE")
    if err != nil { panic(err) }
    before, err := stream.Info(ctx)
    if err != nil { panic(err) }
    subscription, err := connection.SubscribeSync("usage.>")
    if err != nil { panic(err) }
    if err := connection.Flush(); err != nil { panic(err) }

    publisher := routing.NewProducerPublisher(natsjs.New(js), routing.PublisherOptions{})
    for index := 0; index < 150; index++ {
        if err := publisher.PublishUsageRecorded(ctx, item(fmt.Sprintf("a%d", index), "service-a", "eu")); err != nil { panic(err) }
        if err := publisher.PublishUsageRecorded(ctx, item(fmt.Sprintf("b%d", index), "service-b", "us")); err != nil { panic(err) }
    }
    if err := publisher.PublishUsageRecorded(ctx, item("bad", "bad.token", "eu")); err == nil { panic("invalid token accepted") }
    if err := publisher.PublishUsageRecordedNow(ctx, []routing.RoutingEventsUsageRecorded{item("mixed-a", "service-a", "eu"), item("mixed-b", "service-b", "us")}); err == nil { panic("mixed subjects accepted") }
    if err := publisher.Close(ctx); err != nil { panic(err) }

    items := 0
    for index := 0; index < 4; index++ {
        message, err := subscription.NextMsg(5 * time.Second)
        if err != nil { panic(err) }
        var payloads []map[string]any
        if err := json.Unmarshal(message.Data, &payloads); err != nil { panic(err) }
        if len(payloads) > 100 { panic("batch exceeded 100") }
        items += len(payloads)
        for _, payload := range payloads {
            origin := payload["origin"].(map[string]any)
            service := origin["serviceName"].(string)
            environment := origin["environment"].(string)
            expected := "usage." + service + "." + service + "." + environment
            if message.Subject != expected { panic(fmt.Sprintf("subject %q, want %q", message.Subject, expected)) }
        }
    }
    if _, err := subscription.NextMsg(250 * time.Millisecond); err == nil { panic("invalid or mixed input published a message") }
    after, err := stream.Info(ctx)
    if err != nil { panic(err) }
    if fmt.Sprint(before.Config.Subjects) != fmt.Sprint(after.Config.Subjects) { panic("generated adapter changed stream subjects") }
    if items != 300 { panic(fmt.Sprintf("received %d items", items)) }
    fmt.Println("go adapter: 300 items, 4 subject-separated messages, invalid and mixed inputs published nothing")
}
"#;

const PINNED_NATS_IMAGE_DIGEST: &str =
    "eda962d67930eda338222072d9a9f3818855d922ad224c399b0b01d251e9b91b";

struct OwnedNatsContainer {
    id: Option<String>,
}

impl OwnedNatsContainer {
    fn remove(&mut self) -> Result<String, String> {
        let Some(id) = self.id.clone() else {
            return Ok("already removed".to_owned());
        };
        let removed = Command::new("docker")
            .args(["rm", "--force", &id])
            .output()
            .map_err(|error| format!("docker rm --force {id}: {error}"))?;
        if !removed.status.success() {
            return Err(format!(
                "docker rm --force {id}\n{}\n{}",
                String::from_utf8_lossy(&removed.stdout),
                String::from_utf8_lossy(&removed.stderr)
            ));
        }
        let id_filter = format!("id={id}");
        let inspection = Command::new("docker")
            .args([
                "container",
                "ls",
                "--all",
                "--quiet",
                "--no-trunc",
                "--filter",
                &id_filter,
            ])
            .output()
            .map_err(|error| format!("docker container lookup for {id}: {error}"))?;
        if !inspection.status.success() {
            return Err(format!(
                "docker container lookup for {id}\n{}\n{}",
                String::from_utf8_lossy(&inspection.stdout),
                String::from_utf8_lossy(&inspection.stderr)
            ));
        }
        let remaining = String::from_utf8(inspection.stdout)
            .map_err(|error| format!("docker container lookup for {id} returned non-UTF-8: {error}"))?;
        if remaining.lines().any(|found| found == id) {
            return Err(format!("owned container {id} still exists after removal"));
        }
        self.id = None;
        Ok(format!("removed owned NATS container {id}"))
    }

    fn stop(mut self) -> String {
        self.remove().unwrap_or_else(|error| panic!("{error}"))
    }
}

impl Drop for OwnedNatsContainer {
    fn drop(&mut self) {
        if let Err(error) = self.remove() {
            eprintln!("failed to clean owned NATS container: {error}");
        }
    }
}

fn actual_nats() -> (String, Option<OwnedNatsContainer>) {
    if let Ok(url) = std::env::var("ESS_TEST_NATS_URL") {
        assert!(!url.trim().is_empty(), "ESS_TEST_NATS_URL is empty");
        return (url, None);
    }

    let image = std::env::var("ESS_TEST_NATS_IMAGE")
        .expect("set ESS_TEST_NATS_URL or the pinned ESS_TEST_NATS_IMAGE");
    let (name, digest) = image
        .rsplit_once("@sha256:")
        .expect("ESS_TEST_NATS_IMAGE must be a digest-pinned image reference");
    assert!(
        name == "nats" || name.ends_with("/nats"),
        "ESS_TEST_NATS_IMAGE must name the official NATS image"
    );
    assert_eq!(
        digest, PINNED_NATS_IMAGE_DIGEST,
        "ESS_TEST_NATS_IMAGE is not the reviewed NATS 2.15.0 linux/amd64 digest"
    );
    let started = Command::new("docker")
        .args([
            "run",
            "--detach",
            "--rm",
            "--publish",
            "127.0.0.1::4222",
            &image,
            "--jetstream",
        ])
        .output()
        .expect("Docker is required when ESS_TEST_NATS_URL is absent");
    assert!(
        started.status.success(),
        "docker run\n{}\n{}",
        String::from_utf8_lossy(&started.stdout),
        String::from_utf8_lossy(&started.stderr)
    );
    let id = String::from_utf8(started.stdout)
        .expect("Docker returns a UTF-8 container id")
        .trim()
        .to_owned();
    assert!(!id.is_empty(), "Docker returned no container id");
    let container = OwnedNatsContainer { id: Some(id) };
    let id = container.id.as_deref().expect("owned container id");
    let published = Command::new("docker")
        .args(["port", id, "4222/tcp"])
        .output()
        .expect("docker port runs");
    assert!(
        published.status.success(),
        "docker port {id} 4222/tcp\n{}\n{}",
        String::from_utf8_lossy(&published.stdout),
        String::from_utf8_lossy(&published.stderr)
    );
    let ports = String::from_utf8(published.stdout).expect("Docker port output is UTF-8");
    let line = ports
        .lines()
        .find(|line| line.starts_with("127.0.0.1:"))
        .unwrap_or_else(|| panic!("Docker did not bind NATS to loopback: {ports}"));
    let port: u16 = line
        .rsplit_once(':')
        .and_then(|(_, port)| port.parse().ok())
        .unwrap_or_else(|| panic!("Docker returned an invalid NATS port: {line}"));
    let address = SocketAddr::from(([127, 0, 0, 1], port));
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        let last = match TcpStream::connect_timeout(&address, Duration::from_millis(250)) {
            Ok(mut stream) => {
                stream
                    .set_read_timeout(Some(Duration::from_millis(250)))
                    .expect("sets NATS readiness timeout");
                let mut info = [0_u8; 512];
                match stream.read(&mut info) {
                    Ok(count) if info[..count].starts_with(b"INFO ") => break,
                    Ok(count) => format!("read {count} non-INFO bytes"),
                    Err(error) => error.to_string(),
                }
            }
            Err(error) => error.to_string(),
        };
        assert!(
            Instant::now() < deadline,
            "owned NATS container {id} did not become ready at {address}: {last}"
        );
        std::thread::sleep(Duration::from_millis(50));
    }
    (format!("nats://{address}"), Some(container))
}

#[test]
#[ignore = "requires ESS_TEST_NATS_URL or a pinned ESS_TEST_NATS_IMAGE; CI runs this test explicitly"]
fn both_generated_adapters_publish_parameterized_batches_to_actual_nats() {
    assert_eq!(
        std::env::var("ESS_TRANSPORT_PREFETCH_DEPS").as_deref(),
        Ok("1"),
        "set ESS_TRANSPORT_PREFETCH_DEPS=1 to opt into network dependency preparation for the generated harnesses"
    );
    let body = PARAMETER_TRANSPORT
        .replace("max_items: 2", "max_items: 100")
        .replace("max_delay_ms: 50", "max_delay_ms: 60000");
    let generated = parameterized_generated_with(&body);
    let root = scratch("generated-parameterized-nats");

    let rust_types = generated.types.rust("routing-client").expect("rust");
    let mut rust_files = ess_publisher::rust(&generated.plan, &generated.transport, "routing-client", &rust_types.supporting["Cargo.toml"]).expect("renders");
    rust_files.insert("types.rs".to_owned(), rust_types.declarations);
    write(&root.join("rust"), &rust_files);
    fs::create_dir_all(root.join("rust/acceptance/src")).unwrap();
    fs::write(root.join("rust/acceptance/Cargo.toml"), NATS_RUST_MANIFEST).unwrap();
    fs::write(root.join("rust/acceptance/src/main.rs"), NATS_RUST_MAIN).unwrap();

    for (subcommand, arguments) in [
        ("generate-lockfile", Vec::<&str>::new()),
        ("fetch", vec!["--locked"]),
    ] {
        let prepared = Command::new(env!("CARGO"))
            .arg(subcommand)
            .args(arguments)
            .arg("--manifest-path")
            .arg(root.join("rust/acceptance/Cargo.toml"))
            .env("CARGO_NET_OFFLINE", "false")
            .output()
            .unwrap_or_else(|error| panic!("cargo {subcommand} runs: {error}"));
        assert!(
            prepared.status.success(),
            "cargo {subcommand}\n{}\n{}",
            String::from_utf8_lossy(&prepared.stdout),
            String::from_utf8_lossy(&prepared.stderr)
        );
    }
    assert!(root.join("rust/acceptance/Cargo.lock").is_file());

    let module = "example.invalid/routingclient";
    let go_types = generated.types.go("routingclient", module).expect("go");
    let mut go_files = ess_publisher::go(&generated.plan, &generated.transport, "routingclient", module);
    go_files.extend(go_types.supporting);
    go_files.insert("types.go".to_owned(), go_types.declarations);
    write(&root.join("go"), &go_files);
    fs::create_dir_all(root.join("go-acceptance")).unwrap();
    fs::write(root.join("go-acceptance/go.mod"), NATS_GO_MANIFEST).unwrap();
    fs::write(root.join("go-acceptance/main.go"), NATS_GO_MAIN).unwrap();
    let prepared = Command::new("go")
        .args(["mod", "download", "all"])
        .current_dir(root.join("go-acceptance"))
        .output()
        .expect("go dependency preparation runs");
    assert!(
        prepared.status.success(),
        "go mod download all\n{}\n{}",
        String::from_utf8_lossy(&prepared.stdout),
        String::from_utf8_lossy(&prepared.stderr)
    );
    assert!(root.join("go-acceptance/go.sum").is_file());

    let (nats_url, owned_container) = actual_nats();
    let rust = Command::new(env!("CARGO"))
        .args(["run", "--locked", "--offline", "--quiet", "--manifest-path"])
        .arg(root.join("rust/acceptance/Cargo.toml"))
        .env("CARGO_TARGET_DIR", root.join("rust-target"))
        .env("CARGO_NET_OFFLINE", "true")
        .env("ESS_TEST_NATS_URL", &nats_url)
        .output()
        .expect("Rust NATS harness runs");

    let go = Command::new("go")
        .args(["run", "."])
        .current_dir(root.join("go-acceptance"))
        .env("ESS_TEST_NATS_URL", &nats_url)
        .env("GOPROXY", "off")
        .env("GOFLAGS", "-mod=readonly")
        .output()
        .expect("Go NATS harness runs");
    if let Some(container) = owned_container {
        println!("{}", container.stop());
    }
    assert!(rust.status.success(), "{}\n{}", String::from_utf8_lossy(&rust.stdout), String::from_utf8_lossy(&rust.stderr));
    println!("{}", String::from_utf8_lossy(&rust.stdout));
    assert!(go.status.success(), "{}\n{}", String::from_utf8_lossy(&go.stdout), String::from_utf8_lossy(&go.stderr));
    println!("{}", String::from_utf8_lossy(&go.stdout));
}
