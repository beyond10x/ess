//! A `{generated: true}` payload field of type `Optional<T>` goes through a generator port that
//! answers an optional `T`, in the generated Rust and Go behaviours (beyond10x/ess#467).
//!
//! `RATES` emits `RateQuoted.rate: Optional<Decimal>` as `{generated: true}`: the context port gains
//! `generate_optional_decimal` (`GenerateOptionalDecimal` in Go), answering `Option<Decimal>`
//! (`*primitives.Decimal`), and the behaviour fills `rate` with that answer. The generated
//! demonstration context answers it absent, as the interpreter does, so the served entry starts.
//! `UNSET` is the same event in `ess/3`, where an outcome may leave a field undetermined: that field
//! stays absent and no port is asked. Each emitted tree is compiled with a control in which a present
//! and an absent port answer each reach the emitted event.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const RATES: &str = r"
format: ess/23
system: demo
version: v1
domain: demo.rates
events:
  - name: demo.rates.RateQuoted
    fields:
      - {name: amount, type: Decimal}
      - {name: rate, type: Optional<Decimal>}
commands:
  - name: demo.rates.Quote
    input:
      - {name: amount, type: Decimal}
    outcomes:
      - name: quoted
        emits: [demo.rates.RateQuoted]
        payload:
          demo.rates.RateQuoted:
            amount: input.amount
            rate: {generated: true}
components:
  - component: rates-service
    owns: {domains: [demo.rates]}
    accepts: {commands: [demo.rates.Quote]}
    publishes: {events: [demo.rates.RateQuoted]}
    reached_by: network
";

/// `RATES` before explicit payload ownership: nothing sets `rate`.
const UNSET: &str = r"
format: ess/3
system: demo
version: v1
domain: demo.rates
events:
  - name: demo.rates.RateQuoted
    fields:
      - {name: amount, type: Decimal}
      - {name: rate, type: Optional<Decimal>}
commands:
  - name: demo.rates.Quote
    input:
      - {name: amount, type: Decimal}
    outcomes:
      - name: quoted
        emits: [demo.rates.RateQuoted]
        payload:
          demo.rates.RateQuoted:
            amount: input.amount
components:
  - component: rates-service
    owns: {domains: [demo.rates]}
    accepts: {commands: [demo.rates.Quote]}
    publishes: {events: [demo.rates.RateQuoted]}
    reached_by: network
";

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("rates.yaml"),
        RawSpecFile::parse(text).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// Every artifact of `target`'s synthesis of `text`, by path, with the behaviour generated.
fn artifacts(text: &str, target: Target) -> BTreeMap<String, String> {
    let synthesis = synthesize_for(&model(text), target).expect("a realizable target");
    assert_eq!(
        synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.rates.Quote"),
        Some(&SynthesisDisposition::Generated),
        "{target:?}: the behaviour is generated"
    );
    synthesis
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

/// Every artifact of `target`'s synthesis of `text`, written under a fresh directory.
fn emit(text: &str, target: Target, case: &str) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "optional-generated-467-{}-{case}-{}",
        target.name(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, contents) in artifacts(text, target) {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
    eprintln!("emitted {}", directory.display());
    directory
}

/// Runs `program` in `directory`; the log, and whether it succeeded.
fn run(directory: &Path, program: &str, arguments: &[&str]) -> (String, bool) {
    let mut command = Command::new(program);
    command.args(arguments).current_dir(directory);
    if program == env!("CARGO") {
        command
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env_remove("RUSTC_WRAPPER")
            .env("RUSTFLAGS", "-D warnings");
    } else {
        command
            .env("GOWORK", "off")
            .env("GOFLAGS", "-mod=mod")
            .env("GOPROXY", "off");
    }
    let output = command.output().unwrap();
    let log = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    eprintln!("{program} {arguments:?} in {}\n{log}", directory.display());
    (log, output.status.success())
}

// ---- Rust ----------------------------------------------------------------------------------------

#[test]
fn rust_optional_generated_field_reads_an_optional_generator_port() {
    let rust = artifacts(RATES, Target::Rust);
    let behaviour = artifact(&rust, "crates/demo-types/src/behaviour.rs");
    assert!(
        behaviour.contains(
            "    fn generate_optional_decimal(&mut self) -> Option<crate::primitives::Decimal>;"
        ),
        "the context port answers an optional decimal:\n{behaviour}"
    );
    assert!(
        behaviour.contains(
            "fn try_generate_optional_decimal(&mut self) -> Result<Option<crate::primitives::Decimal>, UnmetObligation>;"
        ),
        "the fallible companion answers an optional decimal:\n{behaviour}"
    );
    assert!(
        behaviour.contains("rate: self.ports.try_generate_optional_decimal()?"),
        "the event's `rate` is the port's answer:\n{behaviour}"
    );
    assert!(
        !behaviour.contains("rate: None"),
        "the event's `rate` is not a constant absent value:\n{behaviour}"
    );
    // The demonstration context answers the optional value absent, and the entry starts.
    let memory = artifact(&rust, "crates/demo-server/src/memory.rs");
    assert!(
        memory.contains(
            "fn try_generate_optional_decimal(&mut self) -> Result<Option<demo_types::primitives::Decimal>, demo_types::obligation::UnmetObligation> { Ok(None) }"
        ),
        "{memory}"
    );
    let entry = artifact(&rust, "crates/demo-server/src/bin/rates-service-server.rs");
    assert!(
        !entry.contains("unmet startup obligations"),
        "an optional generated value refuses no startup:\n{entry}"
    );
}

#[test]
fn rust_optional_field_nothing_sets_stays_absent_with_no_port() {
    let rust = artifacts(UNSET, Target::Rust);
    let behaviour = artifact(&rust, "crates/demo-types/src/behaviour.rs");
    assert!(
        behaviour.contains("rate: None"),
        "the undetermined optional field is absent:\n{behaviour}"
    );
    for (path, contents) in &rust {
        assert!(
            !contents.contains("generate_optional"),
            "no generator port for a field nothing sets, in `{path}`:\n{contents}"
        );
    }
    assert!(!behaviour.contains("pub trait Context"), "{behaviour}");
}

/// The control the emitted Rust workspace runs: a present and an absent answer each reach the
/// event, and an unavailable answer is the typed refusal, never an absent value.
const RUST_CONTROL: &str = r#"
use std::cell::Cell;
use std::rc::Rc;

use demo_types::behaviour::{unmet_context, Context, Generated, TryContext};
use demo_types::obligation::UnmetObligation;
use demo_types::primitives::Decimal;
use demo_types::rates::{self, obligations::QuoteBehavior};

/// An implementor's context: the rate it measured, or none, counting each question.
struct Measured {
    rate: Option<Decimal>,
    asked: Rc<Cell<usize>>,
}

impl Context for Measured {
    fn generate_optional_decimal(&mut self) -> Option<Decimal> {
        self.asked.set(self.asked.get() + 1);
        self.rate.clone()
    }
}

/// A context that cannot answer.
struct Unavailable;

impl TryContext for Unavailable {
    fn try_generate_optional_decimal(&mut self) -> Result<Option<Decimal>, UnmetObligation> {
        Err(unmet_context("assigned value: Optional<Decimal>"))
    }
}

fn quote<P: TryContext>(ports: P) -> Result<rates::RateQuoted, UnmetObligation> {
    let outcome = Generated::new(ports).quote(rates::Quote {
        amount: Decimal("12.50".to_owned()),
    })?;
    match outcome {
        rates::QuoteOutcome::Quoted { rate_quoted } => Ok(rate_quoted),
    }
}

#[test]
fn a_present_answer_reaches_the_event() {
    let asked = Rc::new(Cell::new(0));
    let event = quote(Measured {
        rate: Some(Decimal("0.25".to_owned())),
        asked: asked.clone(),
    })
    .unwrap();
    assert_eq!(event.rate, Some(Decimal("0.25".to_owned())));
    assert_eq!(event.amount, Decimal("12.50".to_owned()));
    assert_eq!(asked.get(), 1);
}

#[test]
fn an_absent_answer_reaches_the_event() {
    let asked = Rc::new(Cell::new(0));
    let event = quote(Measured {
        rate: None,
        asked: asked.clone(),
    })
    .unwrap();
    assert_eq!(event.rate, None);
    assert_eq!(asked.get(), 1);
}

#[test]
fn an_unavailable_answer_is_refused_by_name() {
    let refused = quote(Unavailable).unwrap_err();
    assert_eq!(refused.capability, "context answer");
    assert_eq!(refused.source, "assigned value: Optional<Decimal>");
}
"#;

#[test]
fn rust_present_and_absent_port_answers_reach_the_emitted_event() {
    let directory = emit(RATES, Target::Rust, "control");
    let tests = directory.join("crates/demo-types/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("control.rs"), RUST_CONTROL).unwrap();
    let (log, passed) = run(
        &directory,
        env!("CARGO"),
        &["test", "--offline", "-p", "demo-types", "--test", "control"],
    );
    assert!(passed && log.contains("3 passed; 0 failed"), "{log}");
    // The demonstration context and the entry over it compile too.
    let (log, passed) = run(
        &directory,
        env!("CARGO"),
        &["check", "--offline", "-p", "demo-server", "--bins"],
    );
    assert!(passed, "{log}");
    let _ = std::fs::remove_dir_all(&directory);
}

// ---- Go ------------------------------------------------------------------------------------------

#[test]
fn go_optional_generated_field_reads_an_optional_generator_port() {
    let go = artifacts(RATES, Target::Go);
    let behaviour = artifact(&go, "types/behaviour/behaviour.go");
    assert!(
        behaviour.contains("\tGenerateOptionalDecimal() *primitives.Decimal\n"),
        "the context port answers an optional decimal:\n{behaviour}"
    );
    assert!(
        behaviour.contains(
            "\tTryGenerateOptionalDecimal() (*primitives.Decimal, *obligation.UnmetObligation)\n"
        ),
        "the fallible companion answers an optional decimal:\n{behaviour}"
    );
    assert!(
        behaviour.contains("m0, contextErr1 := b.readGenerateOptionalDecimal()"),
        "the behaviour reads the port:\n{behaviour}"
    );
    assert!(
        behaviour.contains("Rate: m0"),
        "the event's `Rate` is the port's answer:\n{behaviour}"
    );
    assert!(
        !behaviour.contains("Rate: nil"),
        "the event's `Rate` is not a constant absent value:\n{behaviour}"
    );
    // The demonstration context answers the optional value absent, and the entry starts with it.
    let memory = artifact(&go, "server/ess_memory.go");
    assert!(
        memory.contains(
            "func (*MemoryContext) TryGenerateOptionalDecimal() (*primitives.Decimal, *obligation.UnmetObligation) {\n\treturn nil, nil\n}"
        ),
        "{memory}"
    );
    let entry = artifact(&go, "cmd/rates-service-server/main.go");
    assert!(
        !entry.contains("unmet startup obligations"),
        "an optional generated value refuses no startup:\n{entry}"
    );
    assert!(entry.contains("NewWithContext("), "{entry}");
}

#[test]
fn go_optional_field_nothing_sets_stays_absent_with_no_port() {
    let go = artifacts(UNSET, Target::Go);
    let behaviour = artifact(&go, "types/behaviour/behaviour.go");
    assert!(
        behaviour.contains("Rate: nil"),
        "the undetermined optional field is absent:\n{behaviour}"
    );
    for (path, contents) in &go {
        assert!(
            !contents.contains("GenerateOptional"),
            "no generator port for a field nothing sets, in `{path}`:\n{contents}"
        );
    }
    assert!(!behaviour.contains("type Context interface"), "{behaviour}");
}

/// The control the emitted Go module runs, as `RUST_CONTROL`.
const GO_CONTROL: &str = r#"package behaviour

import (
	"testing"

	"example.invalid/demo/types/obligation"
	"example.invalid/demo/types/primitives"
	"example.invalid/demo/types/rates"
)

// measured is an implementor's context: the rate it measured, or none, counting each question.
type measured struct {
	rate  *primitives.Decimal
	asked int
}

func (m *measured) GenerateOptionalDecimal() *primitives.Decimal {
	m.asked++
	return m.rate
}

// unavailable is a context that cannot answer.
type unavailable struct{}

func (unavailable) TryGenerateOptionalDecimal() (*primitives.Decimal, *obligation.UnmetObligation) {
	return nil, UnmetContext("assigned value: Optional<Decimal>")
}

func quote(g *Generated) (rates.RateQuoted, *obligation.UnmetObligation) {
	outcome, err := g.Quote(rates.Quote{Amount: primitives.NewDecimal("12.50")})
	if err != nil {
		return rates.RateQuoted{}, err
	}
	return outcome.(rates.QuoteOutcomeQuoted).RateQuoted, nil
}

func TestPresentAnswer(t *testing.T) {
	rate := primitives.NewDecimal("0.25")
	c := &measured{rate: &rate}
	event, err := quote(New(Ports{Context: c}))
	if err != nil || event.Rate == nil || event.Rate.Value() != "0.25" || event.Amount.Value() != "12.50" || c.asked != 1 {
		t.Fatalf("present: %+v %v asked %d", event, err, c.asked)
	}
}

func TestAbsentAnswer(t *testing.T) {
	c := &measured{}
	event, err := quote(New(Ports{Context: c}))
	if err != nil || event.Rate != nil || c.asked != 1 {
		t.Fatalf("absent: %+v %v asked %d", event, err, c.asked)
	}
}

func TestUnavailableAnswer(t *testing.T) {
	_, err := quote(NewWithContext(Ports{}, unavailable{}))
	if err == nil || err.Capability != "context answer" || err.Source != "assigned value: Optional<Decimal>" {
		t.Fatalf("unavailable: %v", err)
	}
}
"#;

#[test]
fn go_present_and_absent_port_answers_reach_the_emitted_event() {
    let directory = emit(RATES, Target::Go, "control");
    std::fs::write(
        directory.join("types/behaviour/control_test.go"),
        GO_CONTROL,
    )
    .unwrap();
    let (log, passed) = run(
        &directory,
        "go",
        &["test", "-count=1", "-v", "./types/behaviour"],
    );
    assert!(passed, "{log}");
    for control in [
        "TestPresentAnswer",
        "TestAbsentAnswer",
        "TestUnavailableAnswer",
    ] {
        assert!(log.contains(&format!("--- PASS: {control}")), "{log}");
    }
    // The demonstration context and the entry over it compile too.
    let (log, passed) = run(&directory, "go", &["build", "-buildvcs=false", "./..."]);
    assert!(passed, "{log}");
    let _ = std::fs::remove_dir_all(&directory);
}
