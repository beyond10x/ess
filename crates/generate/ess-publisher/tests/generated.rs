//! Generated publishers compile offline and behave as `docs/design/event-publishers.md` says,
//! against an in-memory transport (beyond10x/ess#395).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::schema::ModelTypes;
use ess_publisher::{plan, PublisherPlan};
use ess_transport::{TransportIr, TransportSpec};

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
