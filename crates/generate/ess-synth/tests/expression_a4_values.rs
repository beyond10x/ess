//! The generated Rust and Go behaviours copy values through input paths
//! (`docs/design/expression-family-source22.md`, A4): `input.opening.generation_id` into a stored
//! field, an event and an error; `input.previous.generation_id` through an absent `Optional`
//! parent as absence; `input.sealed.label` through a newtype over a struct; and
//! `{input: previous.label, else: input.settings.defaults.label}` falling back to the second path.
//!
//! Each target is compiled and run against the same assertions, then run again with the emitted
//! seam patched to read the same-named top-level input, the struct's first member, or nothing in
//! place of the fallback; each patched variant must fail those assertions, not its build.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};
use std::path::{Path, PathBuf};
use std::process::Command;

const MODEL: &str =
    include_str!("../../../specify/ess-compiler/tests/fixtures/input-value-paths.yaml");

const COMPONENTS: &str = "
components:
  - component: lease-service
    owns: {domains: [leases.pool]}
    accepts: {commands: [leases.pool.Open]}
    publishes: {events: [leases.pool.Opened]}
";

fn ir_of(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("leases.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn ir() -> EssIr {
    ir_of(&format!("{MODEL}{COMPONENTS}"))
}

fn emit(target: Target, label: &str) -> PathBuf {
    let synthesis = synthesize_for(&ir(), target).unwrap();
    assert!(
        synthesis.plan.is_generated(
            ess_synth::CapabilityKind::CommandBehavior,
            "leases.pool.Open"
        ),
        "{target:?}: the paths are generated, not owed: {:#?}",
        synthesis.plan.disposition_of(
            ess_synth::CapabilityKind::CommandBehavior,
            "leases.pool.Open"
        )
    );
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "expression-a4-values-{}-{label}-{}",
        target.name(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    directory
}

fn run(directory: &Path, program: &str, arguments: &[&str]) -> (bool, String) {
    let mut command = Command::new(program);
    command.args(arguments).current_dir(directory);
    if program == env!("CARGO") {
        command
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
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
    (output.status.success(), log)
}

/// Replaces every `from` in the emitted `file` with `to`; the seam must be there to patch.
fn patch(file: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(file).unwrap();
    assert!(
        text.contains(from),
        "the emitted seam `{from}` is in {}",
        file.display()
    );
    std::fs::write(file, text.replace(from, to)).unwrap();
}

const RUST_TESTS: &str = r#"
use leases_types::behaviour::{Generated, LeaseStorage};
use leases_types::pool::obligations::{LeaseDetailsQuery, OpenBehavior};
use leases_types::pool::*;
use leases_types::primitives::Uuid;

#[derive(Default)]
struct Store(Vec<LeaseSnapshot>);

impl LeaseStorage for Store {
    fn get(&self, identity: &LeaseId) -> Option<LeaseSnapshot> {
        self.0.iter().find(|held| &held.data.lease_id == identity).cloned()
    }
    fn put(&mut self, snapshot: LeaseSnapshot) {
        self.0.retain(|held| held.data.lease_id != snapshot.data.lease_id);
        self.0.push(snapshot);
    }
    fn delete(&mut self, identity: &LeaseId) {
        self.0.retain(|held| &held.data.lease_id != identity);
    }
    fn list(&self) -> Vec<LeaseSnapshot> {
        self.0.clone()
    }
}

fn opening(generation: &str, label: &str) -> Opening {
    Opening { generation_id: GenerationId(generation.to_owned()), label: label.to_owned() }
}

fn request(previous: bool, reject: bool) -> Open {
    Open {
        lease_id: LeaseId(Uuid("00000000-0000-4000-8000-000000000001".to_owned())),
        generation_id: GenerationId("decoy".to_owned()),
        opening: opening("gen-open", "open"),
        previous: previous.then(|| opening("gen-prev", "prev")),
        sealed: Sealed(opening("gen-sealed", "sealed")),
        settings: Settings { defaults: Defaults { label: "default".to_owned() } },
        reject,
    }
}

#[test]
fn present_paths_are_read_structurally() {
    let mut generated = Generated::new(Store::default());
    let OpenOutcome::Opened { opened } = generated.open(request(true, false)).unwrap() else {
        panic!("opened");
    };
    assert_eq!(opened.generation_id, GenerationId("gen-open".to_owned()));
    assert_eq!(opened.previous, Some(GenerationId("gen-prev".to_owned())));
    assert_eq!(opened.label, "prev");
    let row = generated.lease_details().unwrap().remove(0);
    assert_eq!(row.generation_id, GenerationId("gen-open".to_owned()));
    assert_eq!(row.previous_generation, Some(GenerationId("gen-prev".to_owned())));
    assert_eq!(row.label, "prev");
    assert_eq!(row.sealed_label, "sealed");
    assert_eq!(row.copied.label, "open");
}

#[test]
fn an_absent_parent_is_absence_and_selects_the_fallback() {
    let mut generated = Generated::new(Store::default());
    let OpenOutcome::Opened { opened } = generated.open(request(false, false)).unwrap() else {
        panic!("opened");
    };
    assert_eq!(opened.previous, None);
    assert_eq!(opened.label, "default");
    let row = generated.lease_details().unwrap().remove(0);
    assert_eq!(row.previous_generation, None);
    assert_eq!(row.label, "default");
}

#[test]
fn the_error_payload_reads_the_paths() {
    let mut generated = Generated::new(Store::default());
    let OpenOutcome::Rejected { error } = generated.open(request(false, true)).unwrap() else {
        panic!("rejected");
    };
    assert_eq!(error.generation_id, GenerationId("gen-open".to_owned()));
    assert_eq!(error.label, "default");
}
"#;

const GO_TESTS: &str = r#"
package behaviour

import (
	"testing"

	"example.invalid/leases/types/pool"
	"example.invalid/leases/types/primitives"
)

type store struct{ rows []pool.LeaseSnapshot }

func (s *store) Get(identity pool.LeaseId) (pool.LeaseSnapshot, bool) {
	for _, row := range s.rows {
		if row.Data.LeaseId == identity {
			return row, true
		}
	}
	return pool.LeaseSnapshot{}, false
}
func (s *store) Put(snapshot pool.LeaseSnapshot) { s.rows = append(s.rows, snapshot) }
func (s *store) Delete(identity pool.LeaseId)    {}
func (s *store) List() []pool.LeaseSnapshot      { return s.rows }

func opening(generation, label string) pool.Opening {
	return pool.Opening{GenerationId: pool.NewGenerationId(generation), Label: label}
}

func request(previous, reject bool) pool.Open {
	input := pool.Open{
		LeaseId:      pool.NewLeaseId(primitives.NewUuid("00000000-0000-4000-8000-000000000001")),
		GenerationId: pool.NewGenerationId("decoy"),
		Opening:      opening("gen-open", "open"),
		Sealed:       pool.NewSealed(opening("gen-sealed", "sealed")),
		Settings:     pool.Settings{Defaults: pool.Defaults{Label: "default"}},
		Reject:       reject,
	}
	if previous {
		held := opening("gen-prev", "prev")
		input.Previous = &held
	}
	return input
}

func TestPresentPaths(t *testing.T) {
	ports := &store{}
	outcome, err := New(Ports{LeaseStorage: ports}).Open(request(true, false))
	if err != nil {
		t.Fatal(err)
	}
	opened := outcome.(pool.OpenOutcomeOpened).Opened
	if opened.GenerationId.Value() != "gen-open" || opened.Previous == nil || opened.Previous.Value() != "gen-prev" || opened.Label != "prev" {
		t.Fatalf("event %+v", opened)
	}
	row := ports.rows[0].Data
	if row.GenerationId.Value() != "gen-open" || row.PreviousGeneration == nil || row.Label != "prev" || row.SealedLabel != "sealed" || row.Copied.Label != "open" {
		t.Fatalf("row %+v", row)
	}
}

func TestAbsentParent(t *testing.T) {
	ports := &store{}
	outcome, err := New(Ports{LeaseStorage: ports}).Open(request(false, false))
	if err != nil {
		t.Fatal(err)
	}
	opened := outcome.(pool.OpenOutcomeOpened).Opened
	if opened.Previous != nil || opened.Label != "default" {
		t.Fatalf("event %+v", opened)
	}
	row := ports.rows[0].Data
	if row.PreviousGeneration != nil || row.Label != "default" {
		t.Fatalf("row %+v", row)
	}
}

func TestErrorPayload(t *testing.T) {
	outcome, err := New(Ports{LeaseStorage: &store{}}).Open(request(false, true))
	if err != nil {
		t.Fatal(err)
	}
	refused := outcome.(pool.OpenOutcomeRejected).Error
	if refused.GenerationId.Value() != "gen-open" || refused.Label != "default" {
		t.Fatalf("error %+v", refused)
	}
}
"#;

/// The emitted Rust seams a faulty variant replaces, and what with.
const RUST_FAULTS: &[(&str, &str, &str)] = &[
    (
        "decoy",
        "generation_id: input.opening.generation_id.clone(),",
        "generation_id: input.generation_id.clone(),",
    ),
    (
        "first-child",
        "input.sealed.0.label.clone()",
        "input.sealed.0.generation_id.0.clone()",
    ),
    (
        "ignored-fallback",
        "None => input.settings.defaults.label.clone()",
        "None => String::new()",
    ),
    (
        "unwrapped-parent",
        "input.previous.as_ref().map(|value| &value.generation_id).cloned()",
        "Some(input.previous.as_ref().map(|value| value.generation_id.clone()).unwrap_or(GenerationIdDefault::default()))",
    ),
];

fn rust_variant(label: &str, fault: Option<(&str, &str)>) -> (bool, String) {
    let directory = emit(Target::Rust, label);
    let tests = directory.join("crates/leases-types/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("values.rs"), RUST_TESTS).unwrap();
    if let Some((from, to)) = fault {
        let behaviour = directory.join("crates/leases-types/src/behaviour.rs");
        patch(&behaviour, from, to);
        // A default for the unwrapped-parent variant: the empty generation an implementation that
        // unwraps an absent parent would write.
        let mut text = std::fs::read_to_string(&behaviour).unwrap();
        text.push_str(
            "\nstruct GenerationIdDefault;\nimpl GenerationIdDefault {\n    #[allow(dead_code)]\n    fn default() -> crate::pool::GenerationId {\n        crate::pool::GenerationId(String::new())\n    }\n}\n",
        );
        std::fs::write(&behaviour, text).unwrap();
    }
    let outcome = run(
        &directory,
        env!("CARGO"),
        &[
            "test",
            "--offline",
            "-p",
            "leases-types",
            "--test",
            "values",
        ],
    );
    let _ = std::fs::remove_dir_all(&directory);
    outcome
}

#[test]
fn a4_generated_rust_copies_through_paths() {
    let (passed, log) = rust_variant("healthy", None);
    eprintln!("{log}");
    assert!(passed && log.contains("3 passed; 0 failed"), "{log}");
}

#[test]
fn a4_generated_rust_faulty_seams_fail_their_assertions() {
    for (label, from, to) in RUST_FAULTS {
        let (passed, log) = rust_variant(label, Some((from, to)));
        assert!(!passed, "{label}: a faulty seam passes:\n{log}");
        assert!(
            log.contains("test result: FAILED"),
            "{label}: fails its assertions, not its build:\n{log}"
        );
    }
}

/// The emitted Go seams a faulty variant replaces, and what with.
const GO_FAULTS: &[(&str, &str, &str)] = &[
    (
        "decoy",
        "GenerationId: input.Opening.GenerationId, PreviousGeneration",
        "GenerationId: input.GenerationId, PreviousGeneration",
    ),
    (
        "first-child",
        "SealedLabel: input.Sealed.Value().Label",
        "SealedLabel: input.Sealed.Value().GenerationId.Value()",
    ),
    (
        "ignored-fallback",
        "= input.Settings.Defaults.Label",
        "= \"\"",
    ),
];

fn go_variant(label: &str, fault: Option<(&str, &str)>) -> (bool, String) {
    let directory = emit(Target::Go, label);
    std::fs::write(directory.join("types/behaviour/values_test.go"), GO_TESTS).unwrap();
    if let Some((from, to)) = fault {
        patch(&directory.join("types/behaviour/behaviour.go"), from, to);
    }
    let outcome = run(
        &directory,
        "go",
        &["test", "-count=1", "-v", "./types/behaviour"],
    );
    let _ = std::fs::remove_dir_all(&directory);
    outcome
}

#[test]
fn a4_generated_go_copies_through_paths() {
    let (passed, log) = go_variant("healthy", None);
    eprintln!("{log}");
    for test in ["TestPresentPaths", "TestAbsentParent", "TestErrorPayload"] {
        assert!(log.contains(&format!("--- PASS: {test}")), "{test}:\n{log}");
    }
    assert!(passed, "{log}");
}

#[test]
fn a4_generated_go_faulty_seams_fail_their_assertions() {
    for (label, from, to) in GO_FAULTS {
        let (passed, log) = go_variant(label, Some((from, to)));
        assert!(!passed, "{label}: a faulty seam passes:\n{log}");
        assert!(
            log.contains("--- FAIL: Test"),
            "{label}: fails its assertions, not its build:\n{log}"
        );
    }
}

/// Below `ess/22` nothing here exists: the same model is refused as it always was, so no plan or
/// generated output changes.
#[test]
fn a4_below_ess22_the_model_is_refused_before_synthesis() {
    let old = format!("{MODEL}{COMPONENTS}").replace("format: ess/22\n", "format: ess/21\n");
    let refused = RawSpecFile::parse(&old)
        .map_err(|error| error.to_string())
        .and_then(|raw| {
            Specification::assemble([(Source::new("leases.yaml"), raw)])
                .map(|_| ())
                .map_err(|errors| errors.to_string())
        });
    assert!(
        refused.is_err(),
        "an ess/21 source reading a path is refused"
    );
}
