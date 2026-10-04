//! `ess-history/2`: an operation may carry the instant its command decision observed
//! (`decision_time`), and nothing else changes (beyond10x/ess#244 part a, unit U4;
//! `docs/design/expression-family-source22.md`, "Recorded history format 2").
//!
//! - The writer selects format 2 exactly when an operation records a time, and refuses format 1
//!   forced onto one that does.
//! - The retained format 1 reader refuses format 2 before it reads an operation; the current
//!   reader refuses a format 1 relabel carrying the field, an explicit `null`, a repeated or
//!   unknown field, a malformed instant and an unknown future major.
//! - Rust, Go and TypeScript write the same bytes for the same operations, in both formats
//!   (`tests/fixtures/history2/written*.json`), and agree on the one spelling of an instant
//!   (`tests/fixtures/history2/decision-time-vectors.json`). The TypeScript half is
//!   `src/ts/explore.test.ts`, run by `tests/typescript_runtime.rs`.

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::history::{self, HistoryFormat, HistoryRefusal};
use ess_conformance::occurrence_clock::DecisionInstant;
use ess_primitives::evidence::SpecDigest;
use serde_json::{json, Value};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/history2")
        .join(name)
}

fn bytes(name: &str) -> Vec<u8> {
    std::fs::read(fixture(name)).unwrap_or_else(|error| panic!("{name}: {error}"))
}

fn digest() -> SpecDigest {
    SpecDigest::new("ab".repeat(32)).unwrap()
}

fn written() -> Value {
    serde_json::from_slice(&bytes("written.json")).unwrap()
}

fn refusal(value: &Value) -> HistoryRefusal {
    history::read(&serde_json::to_vec(value).unwrap(), &digest())
        .expect_err("the document is refused")
}

fn vectors() -> (Vec<String>, Vec<String>) {
    let document: Value = serde_json::from_slice(&bytes("decision-time-vectors.json")).unwrap();
    let list = |key: &str| -> Vec<String> {
        document[key]
            .as_array()
            .unwrap()
            .iter()
            .map(|text| text.as_str().unwrap().to_owned())
            .collect()
    };
    (list("valid"), list("invalid"))
}

#[test]
fn format2_reads_and_writes_back_byte_for_byte() {
    let document = bytes("written.json");
    let read = history::read(&document, &digest()).expect("admitted");
    assert_eq!(read.format, HistoryFormat::EssHistory2);
    let times: Vec<Option<String>> = read
        .operations
        .iter()
        .map(|operation| operation.decision_time.map(DecisionInstant::to_rfc3339))
        .collect();
    assert_eq!(
        times,
        [
            Some("2000-06-01T00:00:00Z".to_owned()),
            Some("2026-10-04T12:00:00.123456789Z".to_owned()),
            None,
            None,
            Some("2000-07-01T00:00:00.5Z".to_owned()),
        ],
        "every reading kept at full precision, an Indeterminate one included"
    );
    assert_eq!(serde_json::to_vec(&read).unwrap(), document);
    assert_eq!(
        HistoryFormat::for_operations(&read.operations),
        HistoryFormat::EssHistory2
    );
}

#[test]
fn format1_without_times_keeps_its_bytes() {
    let document = bytes("written-format1.json");
    let read = history::read(&document, &digest()).expect("admitted");
    assert_eq!(read.format, HistoryFormat::EssHistory1);
    assert!(read
        .operations
        .iter()
        .all(|operation| operation.decision_time.is_none()));
    assert_eq!(serde_json::to_vec(&read).unwrap(), document);
    assert_eq!(
        HistoryFormat::for_operations(&read.operations),
        HistoryFormat::EssHistory1
    );
    // The retained reader admits it unchanged.
    assert_eq!(
        history::read_format1(&document, &digest()).expect("admitted"),
        read
    );

    // The committed format 1 histories read and write back the same document.
    for name in ["linearizable.json", "not-linearizable.json"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/register")
            .join(name);
        let text = std::fs::read(&path).unwrap();
        let value: Value = serde_json::from_slice(&text).unwrap();
        let committed = SpecDigest::new(value["spec_digest"].as_str().unwrap()).unwrap();
        let read = history::read(&text, &committed).expect("admitted");
        assert_eq!(read.format, HistoryFormat::EssHistory1, "{name}");
        assert_eq!(serde_json::to_value(&read).unwrap(), value, "{name}");
    }
}

#[test]
fn the_writer_refuses_format1_forced_onto_a_recorded_time() {
    let mut read = history::read(&bytes("written.json"), &digest()).unwrap();
    read.format = HistoryFormat::EssHistory1;
    let refused = serde_json::to_vec(&read).expect_err("format 1 cannot carry a decision time");
    assert!(refused.to_string().contains("decision_time"), "{refused}");
}

#[test]
fn the_format1_reader_refuses_format2_before_reading_an_operation() {
    let mut value = written();
    // Operations no reader admits: a refusal naming them would mean they were read.
    value["operations"] = json!(7);
    let refused = history::read_format1(&serde_json::to_vec(&value).unwrap(), &digest())
        .expect_err("refused");
    assert_eq!(
        refused,
        HistoryRefusal::UnsupportedFormat {
            found: Some("ess-history/2".to_owned())
        }
    );
}

#[test]
fn a_format1_relabel_carrying_a_time_is_refused() {
    let mut value = written();
    value["format"] = json!("ess-history/1");
    let refused = refusal(&value);
    assert_eq!(refused.code(), "history.malformed");
    assert_eq!(
        refused.to_string(),
        "history.malformed: operation 00000000-0000-4000-8000-000000000001 records a \
         `decision_time`, which `ess-history/1` does not carry; write `ess-history/2`"
    );
}

#[test]
fn an_explicit_null_time_is_refused() {
    let mut value = written();
    value["operations"][2]["decision_time"] = Value::Null;
    let refused = refusal(&value);
    assert_eq!(refused.code(), "history.malformed");
    assert_eq!(
        refused.to_string(),
        "history.malformed: `operations[2].decision_time` is `null`; leave it out where no \
         decision was observed"
    );
}

#[test]
fn a_repeated_or_unknown_field_is_refused() {
    let text = String::from_utf8(bytes("written.json")).unwrap();
    let repeated = text.replacen(
        r#""decision_time":"2000-06-01T00:00:00Z""#,
        r#""decision_time":"2000-06-01T00:00:00Z","decision_time":"2000-06-01T00:00:00Z""#,
        1,
    );
    assert_ne!(repeated, text);
    let refused = history::read(repeated.as_bytes(), &digest()).expect_err("refused");
    assert_eq!(refused.code(), "history.malformed", "{refused}");

    let mut value = written();
    value["operations"][0]["decision_at"] = json!("2000-06-01T00:00:00Z");
    assert_eq!(refusal(&value).code(), "history.malformed");
}

#[test]
fn a_malformed_instant_is_refused() {
    let (valid, invalid) = vectors();
    for text in &valid {
        let mut value = written();
        value["operations"][0]["decision_time"] = json!(text);
        let read = history::read(&serde_json::to_vec(&value).unwrap(), &digest())
            .unwrap_or_else(|refusal| panic!("`{text}` is a decision instant: {refusal}"));
        assert_eq!(
            read.operations[0]
                .decision_time
                .map(DecisionInstant::to_rfc3339),
            Some(text.clone())
        );
    }
    for text in &invalid {
        let mut value = written();
        value["operations"][0]["decision_time"] = json!(text);
        assert_eq!(refusal(&value).code(), "history.malformed", "`{text}`");
    }
    let mut value = written();
    value["operations"][0]["decision_time"] = json!(1_790_510_400_000_u64);
    assert_eq!(
        refusal(&value).code(),
        "history.malformed",
        "a monotonic coordinate"
    );
}

#[test]
fn an_unknown_future_major_is_refused() {
    for format in ["ess-history/3", "ess-history/20", "ess-history/2.1"] {
        let mut value = written();
        value["format"] = json!(format);
        assert_eq!(
            refusal(&value),
            HistoryRefusal::UnsupportedFormat {
                found: Some(format.to_owned())
            }
        );
    }
}

#[test]
fn a_decision_instant_has_one_spelling() {
    let (valid, invalid) = vectors();
    assert!(
        valid.len() >= 8 && invalid.len() >= 25,
        "the vectors were read"
    );
    for text in &valid {
        let instant =
            DecisionInstant::parse(text).unwrap_or_else(|error| panic!("`{text}`: {error}"));
        assert_eq!(instant.to_rfc3339(), *text);
        assert_eq!(instant.to_string(), *text);
        assert_eq!(serde_json::to_value(instant).unwrap(), json!(text));
    }
    for text in &invalid {
        assert!(
            DecisionInstant::parse(text).is_err(),
            "`{text}` was admitted"
        );
        assert!(
            serde_json::from_value::<DecisionInstant>(json!(text)).is_err(),
            "`{text}` was deserialized"
        );
    }
}

// ---- the Go writer -------------------------------------------------------------------------------

/// Builds the same operations in Go, writes them with the explorer's own writer, and compares the
/// bytes with the committed documents the Rust reader reads and writes back; checks the shared
/// instant vectors; and checks the recorder keeps a recorded completion's instant.
const GO_TEST: &str = r#"package essconform

import (
	"encoding/json"
	"errors"
	"os"
	"testing"
	"time"
)

func history2Operations(times bool) []exploreOperation {
	at := func(text string) *DecisionInstant {
		if !times {
			return nil
		}
		instant, err := ParseDecisionInstant(text)
		if err != nil {
			panic(err)
		}
		return &instant
	}
	subject := "00000000-0000-4000-8000-00000000a001"
	return []exploreOperation{
		{client: 0, command: "demo.offers.OpenOffer", subjectKey: subject, invokedAt: 1, returnedAt: 2, returned: true, outcome: "opened", creates: "demo.offers.Offer", decisionTime: at("2000-06-01T00:00:00Z")},
		{client: 0, command: "demo.offers.AcceptOffer", subjectKey: subject, invokedAt: 3, decisionTime: at("2026-10-04T12:00:00.123456789Z")},
		{client: 1, command: "demo.offers.AcceptOffer", subjectKey: subject, invokedAt: 4, returnedAt: 5, returned: true, outcome: "lapsed"},
		{client: 1, command: "demo.offers.Offers", invokedAt: 6, returnedAt: 7, returned: true, outcome: "read", source: "demo.offers.Offer", rows: []string{subject}},
		{client: 0, command: "demo.offers.AcceptOffer", subjectKey: subject, invokedAt: 8, returnedAt: 9, returned: true, outcome: "accepted", retryOf: 2, decisionTime: at("2000-07-01T00:00:00.5Z")},
	}
}

func TestHistory2Written(t *testing.T) {
	digest := "abababababababababababababababababababababababababababababababab"
	for _, c := range []struct {
		times bool
		file  string
	}{{true, "written.json"}, {false, "written-format1.json"}} {
		want, err := os.ReadFile(os.Getenv("ESS_HISTORY2_FIXTURES") + "/" + c.file)
		if err != nil {
			t.Fatal(err)
		}
		got := exploreHistoryBytes(digest, 7, 2, history2Operations(c.times))
		if string(got) != string(want) {
			t.Fatalf("%s:\n got %s\nwant %s", c.file, got, want)
		}
	}
}

func TestHistory2Vectors(t *testing.T) {
	text, err := os.ReadFile(os.Getenv("ESS_HISTORY2_FIXTURES") + "/decision-time-vectors.json")
	if err != nil {
		t.Fatal(err)
	}
	var vectors struct {
		Valid   []string `json:"valid"`
		Invalid []string `json:"invalid"`
	}
	if err := json.Unmarshal(text, &vectors); err != nil {
		t.Fatal(err)
	}
	if len(vectors.Valid) < 8 || len(vectors.Invalid) < 25 {
		t.Fatal("the vectors were not read")
	}
	for _, valid := range vectors.Valid {
		instant, err := ParseDecisionInstant(valid)
		if err != nil {
			t.Errorf("%q: %v", valid, err)
		} else if instant.String() != valid {
			t.Errorf("%q written back as %q", valid, instant.String())
		}
	}
	for _, invalid := range vectors.Invalid {
		if _, err := ParseDecisionInstant(invalid); err == nil {
			t.Errorf("%q was admitted", invalid)
		}
	}
}

type history2Plain struct{ err error }

func (p history2Plain) Complete() (CommandResult, error) {
	return CommandResult{Outcome: "lapsed"}, p.err
}

type history2Recorded struct {
	history2Plain
	at    *DecisionInstant
	calls *int
}

func (p history2Recorded) CompleteRecorded() RecordedCompletion {
	*p.calls++
	result, err := p.Complete()
	return RecordedCompletion{Result: result, Err: err, DecisionTime: p.at}
}

func TestHistory2RecorderKeepsTheReceipt(t *testing.T) {
	at, _ := ParseDecisionInstant("2026-10-04T12:00:00.123456789Z")
	calls := 0
	cases := []struct {
		pending  PendingCommand
		returned bool
		want     *DecisionInstant
	}{
		{history2Recorded{history2Plain{}, &at, &calls}, true, &at},
		{history2Recorded{history2Plain{ErrIndeterminate}, &at, &calls}, false, &at},
		{history2Recorded{history2Plain{ErrIndeterminate}, nil, &calls}, false, nil},
		{history2Plain{}, true, nil},
	}
	for index, c := range cases {
		h := &exploreRecording{operations: []exploreOperation{{command: "demo.offers.AcceptOffer", invokedAt: 1}}, tokens: make([]string, 1), injected: newConcurrentInjected()}
		flight := &exploreFlight{pending: c.pending, command: &exploreCommand{name: "demo.offers.AcceptOffer", node: map[string]any{}}, index: 0}
		if _, _, err := h.complete(flight); err != nil {
			t.Fatal(err)
		}
		operation := h.operations[0]
		if operation.returned != c.returned {
			t.Errorf("case %d: returned %v", index, operation.returned)
		}
		if (operation.decisionTime == nil) != (c.want == nil) || (c.want != nil && *operation.decisionTime != *c.want) {
			t.Errorf("case %d: recorded %v, want %v", index, operation.decisionTime, c.want)
		}
	}
	if calls != 3 {
		t.Errorf("CompleteRecorded ran %d times for three recorded calls", calls)
	}
	receipt := exploreCompleteRecorded(history2Plain{errors.New("lost")})
	if receipt.DecisionTime != nil || receipt.Err == nil {
		t.Errorf("the default receipt is the answer once, with no time: %+v", receipt)
	}
}

func TestHistory2RecorderRefusesAnUnwritableInstant(t *testing.T) {
	calls := 0
	zero := DecisionInstant{}
	h := &exploreRecording{operations: []exploreOperation{{command: "demo.offers.AcceptOffer", invokedAt: 1}}, tokens: make([]string, 1), injected: newConcurrentInjected()}
	flight := &exploreFlight{pending: history2Recorded{history2Plain{}, &zero, &calls}, command: &exploreCommand{name: "demo.offers.AcceptOffer", node: map[string]any{}}, index: 0}
	if _, _, err := h.complete(flight); err == nil {
		t.Error("a zero DecisionInstant was accepted")
	}
	if h.operations[0].decisionTime != nil {
		t.Errorf("a zero DecisionInstant was recorded: %v", h.operations[0].decisionTime)
	}
	for _, year := range []int{-1, 10000} {
		if instant, err := DecisionInstantOf(time.Date(year, 1, 1, 0, 0, 0, 0, time.UTC)); err == nil {
			t.Errorf("year %d was admitted as %q", year, instant.String())
		}
	}
	for _, c := range []struct {
		at   time.Time
		want string
	}{
		{time.Date(0, 1, 1, 0, 0, 0, 0, time.UTC), "0000-01-01T00:00:00Z"},
		{time.Date(9999, 12, 31, 23, 59, 59, 999999999, time.UTC), "9999-12-31T23:59:59.999999999Z"},
		{time.Date(2026, 10, 4, 13, 0, 0, 120000000, time.FixedZone("x", 3600)), "2026-10-04T12:00:00.12Z"},
	} {
		instant, err := DecisionInstantOf(c.at)
		if err != nil || instant.String() != c.want {
			t.Errorf("%v: %q, %v; want %q", c.at, instant.String(), err, c.want)
		}
	}
}
"#;

/// A fresh module holding the emitted package with its explorer, under the build's temporary
/// directory.
fn go_package() -> PathBuf {
    let ir = support_occurrence_clock::model(support_occurrence_clock::OFFERS);
    let suite = ess_conformance::synthesize::synthesize(&ir).suite;
    let directory = std::env::temp_dir().join(format!("ess-history2-go-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&directory);
    std::fs::create_dir_all(directory.join("essconform")).unwrap();
    for artifact in ess_conformance::go::emit_with_model(&suite, &ir).expect("emitted") {
        let path = directory.join(artifact.path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/history2\n\ngo 1.21\n",
    )
    .unwrap();
    std::fs::write(directory.join("essconform/history2_test.go"), GO_TEST).unwrap();
    directory
}

mod support_occurrence_clock;

#[test]
fn the_go_writer_writes_the_same_bytes() {
    let directory = go_package();
    let output = Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "^TestHistory2",
            "-count=1",
            "-v",
        ])
        .env("GOWORK", "off")
        .env("GOMAXPROCS", "2")
        .env("ESS_HISTORY2_FIXTURES", fixture(""))
        .current_dir(&directory)
        .output()
        .expect("the Go toolchain is required for this lane");
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{log}");
    for test in [
        "TestHistory2Written",
        "TestHistory2Vectors",
        "TestHistory2RecorderKeepsTheReceipt",
        "TestHistory2RecorderRefusesAnUnwritableInstant",
    ] {
        assert!(
            log.contains(&format!("--- PASS: {test}")),
            "{test} did not run:\n{log}"
        );
    }
    let _ = std::fs::remove_dir_all(directory);
}
