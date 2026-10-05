//! Adversary pass 1 for story:feature-request-391: generated publishers, driven from
//! `docs/design/event-publishers.md` (the released per-channel behaviour table) and
//! `docs/design/parameterized-event-channel-addresses.md`.
//!
//! Every behaviour is run twice: against a package generated from `ess-transport/1` (the released
//! control) and against one generated from `ess-transport/2` with a dynamic subject.

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

const STATIC_MODEL: &str = "format: ess/20
system: metering
version: v1
domain: metering.items
events:
  - name: metering.items.UsageRecorded
    naming: {wire: usage}
    fields:
      - {name: id, type: String}
components:
  - component: producer
    owns: {domains: [metering.items]}
    publishes: {events: [metering.items.UsageRecorded]}
";

const STATIC_TRANSPORT: &str = "brokers:
  - {id: events, protocol: nats, jetstream: true}
channels:
  - event: metering.items.UsageRecorded
    broker: events
    subject: usage
    envelope: array
    delivery: at_most_once
    batch: {max_items: 2, max_delay_ms: 60000}
streams:
  - {name: USAGE, broker: events, subjects: [usage], storage: file, retention: limits, owner: external}
";

const DYNAMIC_MODEL: &str = "format: ess/20
system: routing
version: v1
domain: routing.events
types:
  - name: routing.events.Source
    kind: struct
    fields:
      - {name: service, type: String}
events:
  - name: routing.events.UsageRecorded
    fields:
      - {name: id, type: String}
      - {name: source, type: routing.events.Source}
components:
  - component: producer
    owns: {domains: [routing.events]}
    publishes: {events: [routing.events.UsageRecorded]}
";

const DYNAMIC_TRANSPORT: &str = "brokers:
  - {id: events, protocol: nats, jetstream: true}
channels:
  - event: routing.events.UsageRecorded
    broker: events
    subject: 'usage.{service}'
    parameters: {service: event.source.service}
    envelope: array
    delivery: at_most_once
    batch: {max_items: 2, max_delay_ms: 60000}
streams:
  - {name: USAGE, broker: events, subjects: ['usage.>'], storage: file, retention: limits, owner: external}
";

struct Generated {
    plan: PublisherPlan,
    types: schema_contract::realize::Plan,
    transport: TransportIr,
}

fn generated(model: &str, system: &str, format: &str, body: &str, package: &str) -> Generated {
    let raw = RawSpecFile::parse(model).expect("well formed");
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)]).expect("validates");
    let mut sources = SourceMap::new();
    sources.insert("model.yaml", model);
    let ir: EssIr = compile(&spec, &sources).expect("compiles");
    let text = format!(
        "type: {format}\nspecification:\n  system: {system}\n  version: v1\n  source_digest: sha256:{}\n{body}",
        ir.source_digest()
    );
    let transport = ess_transport::compile(&TransportSpec::from_yaml(&text).expect("parses"), &ir)
        .expect("transport compiles");
    let roots: BTreeSet<String> = PublisherPlan::roots(&ir, "producer", &transport)
        .into_iter()
        .collect();
    let selection = ModelTypes::select(&ir, &roots).expect("selects");
    let types = schema_contract::realize::Plan::from_model(&selection).expect("plans");
    let report = serde_json::to_value(types.rust(package).expect("rust").report).expect("report");
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

/// The two behaviours, in Rust, over whatever `item(id, service)` builds.
const RUST_CASES: &str = r#"
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering::SeqCst};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

fn eventually(flag: &AtomicBool, limit: Duration) -> bool {
    let start = Instant::now();
    while start.elapsed() < limit {
        if flag.load(SeqCst) { return true; }
        std::thread::sleep(Duration::from_millis(5));
    }
    flag.load(SeqCst)
}

/// Holds the first message until released, as a broker does while it has not acknowledged.
#[derive(Clone, Default)]
struct Unacknowledged { calls: Arc<AtomicUsize>, release: Arc<AtomicBool>, sent: Arc<Mutex<Vec<String>>> }
impl Transport for Unacknowledged {
    type Error = String;
    fn publish(&self, _subject: &str, payload: &[u8]) -> Result<(), String> {
        if self.calls.fetch_add(1, SeqCst) == 0 {
            while !self.release.load(SeqCst) { std::thread::sleep(Duration::from_millis(1)); }
        }
        self.sent.lock().unwrap().push(String::from_utf8(payload.to_vec()).unwrap());
        Ok(())
    }
}

/// event-publishers.md, `envelope: array` row: `publish_<event>` "encodes, buffers; ... what
/// arrives while a message waits for its acknowledgement goes in the next one".
#[test]
fn adversary_a_publish_is_buffered_while_an_earlier_message_awaits_its_acknowledgement() {
    let transport = Unacknowledged::default();
    let publisher = Arc::new(ProducerPublisher::new(transport.clone(), PublisherOptions::default()));
    publisher.publish_usage_recorded(&item("a1", "service-a")).unwrap();
    publisher.publish_usage_recorded(&item("a2", "service-a")).unwrap();
    let start = Instant::now();
    while transport.calls.load(SeqCst) == 0 && start.elapsed() < Duration::from_secs(2) {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(transport.calls.load(SeqCst), 1, "the max-items message never reached the transport");
    let returned = Arc::new(AtomicBool::new(false));
    let (offering, done) = (publisher.clone(), returned.clone());
    let thread = std::thread::spawn(move || {
        offering.publish_usage_recorded(&item("b1", "service-b")).unwrap();
        done.store(true, SeqCst);
    });
    let buffered_while_unacknowledged = eventually(&returned, Duration::from_secs(1));
    transport.release.store(true, SeqCst);
    thread.join().unwrap();
    publisher.flush().unwrap();
    assert!(
        buffered_while_unacknowledged,
        "publish_usage_recorded blocked for 1s behind another message's unacknowledged publish; the released array contract buffers it"
    );
}

/// Fails its first message only.
#[derive(Clone, Default)]
struct RefusesFirst { calls: Arc<AtomicUsize>, sent: Arc<Mutex<Vec<String>>> }
impl Transport for RefusesFirst {
    type Error = String;
    fn publish(&self, _subject: &str, payload: &[u8]) -> Result<(), String> {
        if self.calls.fetch_add(1, SeqCst) == 0 { return Err("refused".to_owned()); }
        self.sent.lock().unwrap().push(String::from_utf8(payload.to_vec()).unwrap());
        Ok(())
    }
}

/// event-publishers.md: "a failed publish is reported to the error callback ... and the batch is
/// dropped; nothing retries it". The callback is the application's only hook to retry, and the
/// released publisher lets it offer the payloads again.
#[test]
fn adversary_the_error_callback_may_offer_a_payload_to_the_same_operation() {
    let transport = RefusesFirst::default();
    let cell: Arc<OnceLock<Arc<ProducerPublisher>>> = Arc::new(OnceLock::new());
    let offered = Arc::new(AtomicBool::new(false));
    let (from_callback, flag) = (cell.clone(), offered.clone());
    let publisher = Arc::new(ProducerPublisher::new(transport.clone(), PublisherOptions {
        on_error: Some(Arc::new(move |_error: PublishError| {
            if let Some(publisher) = from_callback.get() {
                if publisher.publish_usage_recorded(&item("retry", "service-a")).is_ok() {
                    flag.store(true, SeqCst);
                }
            }
        })),
    }));
    assert!(cell.set(publisher.clone()).is_ok());
    publisher.publish_usage_recorded(&item("a1", "service-a")).unwrap();
    publisher.publish_usage_recorded(&item("a2", "service-a")).unwrap();
    // On failure the operation's worker is stuck inside the callback; nothing below touches the
    // publisher again, and the reference cycle keeps it from being dropped.
    assert!(
        eventually(&offered, Duration::from_secs(2)),
        "the error callback's publish never returned: the operation deadlocked on its own worker"
    );
    publisher.flush().unwrap();
    assert!(transport.sent.lock().unwrap().iter().any(|body| body.contains("retry")));
}
"#;

const RUST_STATIC_ITEM: &str = r##"
use metering_client::*;
fn item(id: &str, _service: &str) -> MeteringItemsUsageRecorded {
    serde_json::from_str(&format!(r#"{{"id":"{id}"}}"#)).unwrap()
}
"##;

const RUST_DYNAMIC_ITEM: &str = r##"
use routing_client::*;
fn item(id: &str, service: &str) -> RoutingEventsUsageRecorded {
    serde_json::from_str(&format!(r#"{{"id":"{id}","source":{{"service":"{service}"}}}}"#)).unwrap()
}
"##;

fn run_rust(generated: &Generated, package: &str, item: &str, name: &str) {
    let types = generated.types.rust(package).expect("rust");
    let mut files = ess_publisher::rust(
        &generated.plan,
        &generated.transport,
        package,
        &types.supporting["Cargo.toml"],
    )
    .expect("renders");
    files.insert("types.rs".to_owned(), types.declarations);
    files.insert(
        "tests/adversary.rs".to_owned(),
        format!("{item}\n{RUST_CASES}"),
    );
    let root = scratch(name);
    write(&root, &files);
    let output = Command::new(env!("CARGO"))
        .args(["test", "--offline", "--manifest-path"])
        .arg(root.join("Cargo.toml"))
        .args(["--test", "adversary"])
        .env(
            "CARGO_TARGET_DIR",
            Path::new(env!("CARGO_TARGET_TMPDIR")).join("adversary-391-publisher-target"),
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

#[test]
fn released_rust_publisher_buffers_during_an_ack_and_allows_callback_publishes() {
    let generated = generated(
        STATIC_MODEL,
        "metering",
        "ess-transport/1",
        STATIC_TRANSPORT,
        "metering-client",
    );
    run_rust(
        &generated,
        "metering-client",
        RUST_STATIC_ITEM,
        "adversary-391-rust-static",
    );
}

#[test]
fn parameterized_rust_publisher_buffers_during_an_ack_and_allows_callback_publishes() {
    let generated = generated(
        DYNAMIC_MODEL,
        "routing",
        "ess-transport/2",
        DYNAMIC_TRANSPORT,
        "routing-client",
    );
    run_rust(
        &generated,
        "routing-client",
        RUST_DYNAMIC_ITEM,
        "adversary-391-rust-dynamic",
    );
}

const GO_CASES: &str = r#"
type unacknowledged struct {
	mu      sync.Mutex
	calls   int
	entered chan struct{}
	release chan struct{}
	sent    []string
}

func (u *unacknowledged) Publish(_ context.Context, _ string, payload []byte) error {
	u.mu.Lock()
	u.calls++
	first := u.calls == 1
	u.mu.Unlock()
	if first {
		close(u.entered)
		<-u.release
	}
	u.mu.Lock()
	defer u.mu.Unlock()
	u.sent = append(u.sent, string(payload))
	return nil
}

// event-publishers.md, envelope: array: Publish<Event> encodes and buffers; what arrives while a
// message waits for its acknowledgement goes in the next one.
func TestAdversaryPublishIsBufferedWhileAnEarlierMessageAwaitsItsAck(t *testing.T) {
	transport := &unacknowledged{entered: make(chan struct{}), release: make(chan struct{})}
	publisher := NewProducerPublisher(transport, PublisherOptions{})
	ctx := context.Background()
	if err := publisher.PublishUsageRecorded(ctx, item(t, "a1", "service-a")); err != nil { t.Fatal(err) }
	if err := publisher.PublishUsageRecorded(ctx, item(t, "a2", "service-a")); err != nil { t.Fatal(err) }
	select {
	case <-transport.entered:
	case <-time.After(2 * time.Second):
		t.Fatal("the max-items message never reached the transport")
	}
	returned := make(chan error, 1)
	go func() { returned <- publisher.PublishUsageRecorded(ctx, item(t, "b1", "service-b")) }()
	buffered := false
	select {
	case err := <-returned:
		if err != nil { t.Fatal(err) }
		buffered = true
	case <-time.After(time.Second):
	}
	close(transport.release)
	if !buffered {
		if err := <-returned; err != nil { t.Fatal(err) }
	}
	if err := publisher.Close(ctx); err != nil { t.Fatal(err) }
	if !buffered {
		t.Fatal("PublishUsageRecorded blocked for 1s behind another message's unacknowledged publish; the released array contract buffers it")
	}
}

type refusesFirst struct {
	mu    sync.Mutex
	calls int
	sent  []string
}

func (r *refusesFirst) Publish(_ context.Context, _ string, payload []byte) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.calls++
	if r.calls == 1 { return errors.New("refused") }
	r.sent = append(r.sent, string(payload))
	return nil
}

// event-publishers.md: a failed publish is reported to the error callback and dropped; nothing
// retries it. The callback is the application's only hook to offer the payloads again.
func TestAdversaryTheErrorCallbackMayOfferAPayloadToTheSameOperation(t *testing.T) {
	transport := &refusesFirst{}
	ctx := context.Background()
	offered := make(chan error, 1)
	var publisher *ProducerPublisher
	publisher = NewProducerPublisher(transport, PublisherOptions{OnError: func(error) {
		offered <- publisher.PublishUsageRecorded(ctx, item(t, "retry", "service-a"))
	}})
	if err := publisher.PublishUsageRecorded(ctx, item(t, "a1", "service-a")); err != nil { t.Fatal(err) }
	if err := publisher.PublishUsageRecorded(ctx, item(t, "a2", "service-a")); err != nil { t.Fatal(err) }
	select {
	case err := <-offered:
		if err != nil { t.Fatal(err) }
	case <-time.After(2 * time.Second):
		// The operation's worker is stuck inside the callback; the publisher is abandoned.
		t.Fatal("the error callback's publish never returned: the operation deadlocked on its own worker")
	}
	if err := publisher.Close(ctx); err != nil { t.Fatal(err) }
	transport.mu.Lock()
	defer transport.mu.Unlock()
	if len(transport.sent) != 1 || !strings.Contains(transport.sent[0], "retry") { t.Fatalf("sent %v", transport.sent) }
}
"#;

const GO_DYNAMIC_STRESS: &str = r#"
type recording struct {
	mu   sync.Mutex
	sent [][2]string
}

func (r *recording) Publish(_ context.Context, subject string, payload []byte) error {
	r.mu.Lock()
	defer r.mu.Unlock()
	r.sent = append(r.sent, [2]string{subject, string(payload)})
	return nil
}

// Many producers on overlapping subjects, concurrent flushes, then close: every accepted item is
// delivered once, on its own subject, at most max-items per message, FIFO per producer and subject.
func TestAdversaryConcurrentPublishFlushAndCloseLoseNothing(t *testing.T) {
	transport := &recording{}
	publisher := NewProducerPublisher(transport, PublisherOptions{})
	ctx := context.Background()
	var producers sync.WaitGroup
	accepted := make(chan string, 8*200)
	for producer := 0; producer < 8; producer++ {
		producers.Add(1)
		go func(producer int) {
			defer producers.Done()
			for index := 0; index < 200; index++ {
				service := fmt.Sprintf("s%d", (producer+index)%5)
				id := fmt.Sprintf("p%d-%04d", producer, index)
				if err := publisher.PublishUsageRecorded(ctx, item(t, id, service)); err != nil { t.Error(err); return }
				accepted <- id
				if index%17 == 0 { if err := publisher.Flush(ctx); err != nil { t.Error(err) } }
			}
		}(producer)
	}
	producers.Wait()
	close(accepted)
	if err := publisher.Close(ctx); err != nil { t.Fatal(err) }
	if buckets := publisher.usageRecorded.bucketCount(); buckets != -1 { t.Fatalf("closed batcher reports %d", buckets) }
	want := map[string]bool{}
	for id := range accepted { want[id] = true }
	seen := map[string]bool{}
	last := map[string]string{}
	transport.mu.Lock()
	defer transport.mu.Unlock()
	for _, sent := range transport.sent {
		var payloads []map[string]any
		if err := json.Unmarshal([]byte(sent[1]), &payloads); err != nil { t.Fatal(err) }
		if len(payloads) == 0 || len(payloads) > UsageRecordedMaxItems { t.Fatalf("message of %d items", len(payloads)) }
		for _, payload := range payloads {
			id := payload["id"].(string)
			service := payload["source"].(map[string]any)["service"].(string)
			if sent[0] != "usage."+service { t.Fatalf("item %s on %q", id, sent[0]) }
			if seen[id] { t.Fatalf("item %s delivered twice", id) }
			seen[id] = true
			key := strings.SplitN(id, "-", 2)[0] + "/" + service
			if previous, ok := last[key]; ok && previous >= id { t.Fatalf("FIFO: %s after %s on %s", id, previous, service) }
			last[key] = id
		}
	}
	if len(seen) != len(want) { t.Fatalf("delivered %d of %d accepted items", len(seen), len(want)) }
}
"#;

const GO_STATIC_HEADER: &str = r#"package meteringclient

import (
	"context"
	"encoding/json"
	"errors"
	"strings"
	"sync"
	"testing"
	"time"
)

var (
	_ = json.Marshal
	_ = errors.New
	_ = strings.Contains
	_ = sync.NewCond
	_ = time.Second
)

func item(t *testing.T, id, _ string) MeteringItemsUsageRecorded {
	var value MeteringItemsUsageRecorded
	if err := json.Unmarshal([]byte(`{"id":"`+id+`"}`), &value); err != nil { t.Fatal(err) }
	return value
}
"#;

const GO_DYNAMIC_HEADER: &str = r#"package routingclient

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

var (
	_ = errors.New
	_ = fmt.Sprintf
	_ = strings.Contains
	_ = time.Second
)

func item(t *testing.T, id, service string) RoutingEventsUsageRecorded {
	var value RoutingEventsUsageRecorded
	if err := json.Unmarshal([]byte(`{"id":"`+id+`","source":{"service":"`+service+`"}}`), &value); err != nil { t.Fatal(err) }
	return value
}
"#;

fn run_go(generated: &Generated, package: &str, test: String, name: &str) {
    let module = format!("example.invalid/{package}");
    let types = generated.types.go(package, &module).expect("go");
    let mut files = ess_publisher::go(&generated.plan, &generated.transport, package, &module);
    files.extend(types.supporting);
    files.insert("types.go".to_owned(), types.declarations);
    files.insert("adversary_test.go".to_owned(), test);
    let root = scratch(name);
    write(&root, &files);
    let output = Command::new("go")
        .args([
            "test",
            "-race",
            "-count=1",
            "-v",
            "-run",
            "TestAdversary",
            ".",
        ])
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

#[test]
fn released_go_publisher_buffers_during_an_ack_and_allows_callback_publishes() {
    let generated = generated(
        STATIC_MODEL,
        "metering",
        "ess-transport/1",
        STATIC_TRANSPORT,
        "metering-client",
    );
    run_go(
        &generated,
        "meteringclient",
        format!("{GO_STATIC_HEADER}{GO_CASES}"),
        "adversary-391-go-static",
    );
}

#[test]
fn parameterized_go_publisher_buffers_during_an_ack_and_allows_callback_publishes() {
    let generated = generated(
        DYNAMIC_MODEL,
        "routing",
        "ess-transport/2",
        DYNAMIC_TRANSPORT,
        "routing-client",
    );
    run_go(
        &generated,
        "routingclient",
        format!("{GO_DYNAMIC_HEADER}{GO_CASES}"),
        "adversary-391-go-dynamic",
    );
}

#[test]
fn parameterized_go_publisher_survives_concurrent_publish_flush_and_close() {
    let generated = generated(
        DYNAMIC_MODEL,
        "routing",
        "ess-transport/2",
        DYNAMIC_TRANSPORT,
        "routing-client",
    );
    run_go(
        &generated,
        "routingclient",
        format!("{GO_DYNAMIC_HEADER}{GO_DYNAMIC_STRESS}"),
        "adversary-391-go-stress",
    );
}
