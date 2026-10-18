//! A present mint, `{generated: true, present: true}`, into an `Optional` field calls the required
//! generator in the generated Rust and Go behaviours and wraps its answer present (`ess/27`,
//! beyond10x/ess#545, `docs/design/minted-value-sources.md`).
//!
//! `RATES` emits `RateQuoted.rate: Optional<Decimal>` as a present mint: the context port is
//! `generate_decimal` (`GenerateDecimal` in Go), answering a `Decimal` rather than an optional one,
//! and the behaviour fills `rate` with that answer, present. An outcome whose `sets:` reads its own
//! payload (`{payload: …}`) stays an obligation in both targets, named by the plan.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

const RATES: &str = r"
format: ess/27
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
            rate: {generated: true, present: true}
components:
  - component: rates-service
    owns: {domains: [demo.rates]}
    accepts: {commands: [demo.rates.Quote]}
    publishes: {events: [demo.rates.RateQuoted]}
    reached_by: network
";

/// A quote stored on a row, its `sets:` reading the rate the payload minted.
const STORED: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.rates
types:
  - {name: demo.rates.QuoteId, kind: newtype, of: Uuid}
entities:
  - name: demo.rates.Quote
    identity: {name: quote_id, type: demo.rates.QuoteId}
    fields:
      - {name: rate, type: Optional<Decimal>}
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
      transitions: []
events:
  - name: demo.rates.RateQuoted
    fields:
      - {name: quote_id, type: demo.rates.QuoteId}
      - {name: rate, type: Decimal}
commands:
  - name: demo.rates.OpenQuote
    input: []
    outcomes:
      - name: opened
        creates: demo.rates.Quote
        instance: quote_id
        sets:
          rate: {payload: demo.rates.RateQuoted.rate}
        emits: [demo.rates.RateQuoted]
        payload:
          demo.rates.RateQuoted:
            quote_id: {generated: true}
            rate: {generated: true}
components:
  - component: rates-service
    owns: {domains: [demo.rates]}
    accepts: {commands: [demo.rates.OpenQuote]}
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

/// Every artifact of `target`'s synthesis of `RATES`, by path, with the behaviour generated.
fn artifacts(target: Target) -> BTreeMap<String, String> {
    let synthesis = synthesize_for(&model(RATES), target).expect("a realizable target");
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

/// Every artifact of `target`'s synthesis of `RATES`, written under a fresh directory.
fn emit(target: Target) -> PathBuf {
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "minted-present-545-{}-{}",
        target.name(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, contents) in artifacts(target) {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, contents).unwrap();
    }
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

#[test]
fn rust_present_mint_calls_the_required_generator() {
    let rust = artifacts(Target::Rust);
    let behaviour = artifact(&rust, "crates/demo-types/src/behaviour.rs");
    assert!(
        behaviour.contains("    fn generate_decimal(&mut self) -> crate::primitives::Decimal;"),
        "the context port answers a required decimal:\n{behaviour}"
    );
    assert!(
        behaviour.contains("rate: Some(self.ports.try_generate_decimal()?)"),
        "the event's `rate` is the required port's answer, present:\n{behaviour}"
    );
    assert!(
        !behaviour.contains("generate_optional_decimal"),
        "no optional generator is asked:\n{behaviour}"
    );
    let directory = emit(Target::Rust);
    let (log, passed) = run(
        &directory,
        env!("CARGO"),
        &["check", "--offline", "-p", "demo-types"],
    );
    assert!(passed, "{log}");
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn go_present_mint_calls_the_required_generator() {
    let go = artifacts(Target::Go);
    let behaviour = artifact(&go, "types/behaviour/behaviour.go");
    assert!(
        behaviour.contains("\tGenerateDecimal() primitives.Decimal\n"),
        "the context port answers a required decimal:\n{behaviour}"
    );
    assert!(
        behaviour.contains("m0, contextErr1 := b.readGenerateDecimal()"),
        "the behaviour reads the required port:\n{behaviour}"
    );
    assert!(
        behaviour.contains("Rate: some(m0)"),
        "the event's `Rate` is the port's answer, present:\n{behaviour}"
    );
    assert!(
        !behaviour.contains("GenerateOptionalDecimal"),
        "no optional generator is asked:\n{behaviour}"
    );
    let directory = emit(Target::Go);
    let (log, passed) = run(&directory, "go", &["build", "-buildvcs=false", "./..."]);
    assert!(passed, "{log}");
    let _ = std::fs::remove_dir_all(&directory);
}

#[test]
fn a_payload_read_in_sets_stays_an_obligation_in_both_targets() {
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&model(STORED), target).expect("a realizable target");
        let disposition = synthesis
            .plan
            .disposition_of(CapabilityKind::CommandBehavior, "demo.rates.OpenQuote");
        assert!(
            matches!(disposition, Some(SynthesisDisposition::Obligation(owed))
                if owed.reason.describes().contains("payload")),
            "{target:?}: a `sets:` reading the payload is left to the implementor: {disposition:?}"
        );
    }
}
