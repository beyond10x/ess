//! The generated Rust and Go behaviours decide a guard comparing a fact with one constant offset of
//! another (`docs/design/expression-family-source22.md`, A2): `upper >= lower + 6` with the exact sum,
//! never wrapped, clamped, rounded through binary64 or turned Unknown at the ends of `i64`, and
//! `expires_at >= issued_at + 1h` by the instants, moved by elapsed seconds. Each lane is compiled
//! and run healthy, then once per paired fault patched into the emitted seam, and each fault must
//! fail. A view filter's offset stays owed by name; an entity invariant's refuses the target by name.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use std::path::{Path, PathBuf};
use std::process::Command;

const MODEL: &str = "format: ess/22
system: pool
version: v1
domain: pool.lease
events:
  - {name: pool.lease.Wide, fields: []}
  - {name: pool.lease.Inverted, fields: []}
  - {name: pool.lease.Late, fields: []}
  - {name: pool.lease.Soon, fields: []}
  - {name: pool.lease.Accepted, fields: []}
  - {name: pool.lease.Matched, fields: []}
  - {name: pool.lease.Apart, fields: []}
  - {name: pool.lease.Neither, fields: []}
commands:
  - name: pool.lease.Above
    input:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
    outcomes:
      - name: equal
        when: upper == lower + 1
        emits: [pool.lease.Matched]
      - name: apart
        when: upper != lower + 1
        emits: [pool.lease.Apart]
      - name: neither
        emits: [pool.lease.Neither]
  - name: pool.lease.Below
    input:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
    outcomes:
      - name: equal
        when: upper == lower - 1
        emits: [pool.lease.Matched]
      - name: apart
        when: upper != lower - 1
        emits: [pool.lease.Apart]
      - name: neither
        emits: [pool.lease.Neither]
  - name: pool.lease.Open
    input:
      - {name: lower, type: Integer}
      - {name: upper, type: Integer}
      - {name: issued_at, type: Timestamp}
      - {name: expires_at, type: Timestamp}
    outcomes:
      - name: wide
        when: upper >= lower + 6
        emits: [pool.lease.Wide]
      - name: inverted
        when: upper < lower - 3
        emits: [pool.lease.Inverted]
      - name: late
        when: expires_at >= issued_at + 1h
        emits: [pool.lease.Late]
      - name: soon
        when: expires_at < issued_at - 5m
        emits: [pool.lease.Soon]
      - name: accepted
        emits: [pool.lease.Accepted]
components:
  - component: pool-service
    owns: {domains: [pool.lease]}
    accepts: {commands: [pool.lease.Open, pool.lease.Above, pool.lease.Below]}
    publishes: {events: [pool.lease.Wide, pool.lease.Inverted, pool.lease.Late, pool.lease.Soon, pool.lease.Accepted, pool.lease.Matched, pool.lease.Apart, pool.lease.Neither]}
";

fn ir_of(text: &str) -> EssIr {
    let spec =
        Specification::assemble([(Source::new("pool.yaml"), RawSpecFile::parse(text).unwrap())])
            .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn emit(target: Target) -> PathBuf {
    let synthesis = synthesize_for(&ir_of(MODEL), target).unwrap();
    for command in ["pool.lease.Open", "pool.lease.Above", "pool.lease.Below"] {
        assert!(
            synthesis
                .plan
                .is_generated(CapabilityKind::CommandBehavior, command),
            "{target:?}: the guard of {command} is generated, not owed"
        );
    }
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "expression-a2-guards-{}-{}",
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

/// Runs `program` in `directory`; its output and whether it succeeded.
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

/// Replaces `before` with `after` in the one emitted file that holds it.
fn patch(directory: &Path, file: &str, before: &str, after: &str) {
    let path = directory.join(file);
    let text = std::fs::read_to_string(&path).unwrap();
    assert!(text.contains(before), "{file} holds `{before}`");
    std::fs::write(&path, text.replace(before, after)).unwrap();
}

const RUST_GUARDS: &str = r#"
use pool_types::{behaviour::Generated, lease::{Above, AboveOutcome, Below, BelowOutcome, Open, OpenOutcome, obligations::{AboveBehavior, BelowBehavior, OpenBehavior}}, primitives::Timestamp};
const T0: &str = "2020-01-01T00:00:00Z";
fn open(lower: i64, upper: i64, expires: &str) -> OpenOutcome {
    Generated::new(())
        .open(Open {
            lower,
            upper,
            issued_at: Timestamp(T0.to_owned()),
            expires_at: Timestamp(expires.to_owned()),
        })
        .unwrap()
}
fn wide(o: OpenOutcome) -> bool { matches!(o, OpenOutcome::Wide { .. }) }
fn inverted(o: OpenOutcome) -> bool { matches!(o, OpenOutcome::Inverted { .. }) }
fn late(o: OpenOutcome) -> bool { matches!(o, OpenOutcome::Late { .. }) }
fn soon(o: OpenOutcome) -> bool { matches!(o, OpenOutcome::Soon { .. }) }
fn accepted(o: OpenOutcome) -> bool { matches!(o, OpenOutcome::Accepted { .. }) }
#[test]
fn boundaries() {
    assert!(accepted(open(2, 7, T0)), "the boundary itself is not wide");
    assert!(wide(open(2, 8, T0)), "one above the boundary is wide");
    assert!(accepted(open(2, -1, T0)), "the subtraction's boundary is not inverted");
    assert!(inverted(open(2, -2, T0)), "one below it, below zero, is inverted");
    assert!(accepted(open(0, 0, "2020-01-01T00:59:59Z")), "a second before the hour");
    assert!(late(open(0, 0, "2020-01-01T01:00:00Z")), "the hour itself");
    assert!(late(open(0, 0, "2020-01-01T02:00:00+01:00")), "the hour, spelled in another offset");
    assert!(accepted(open(0, 0, "2019-12-31T23:55:00Z")), "five minutes before");
    assert!(soon(open(0, 0, "2019-12-31T23:54:59Z")), "a second more");
}
#[test]
fn extremes() {
    assert!(accepted(open(i64::MAX - 2, i64::MAX, T0)), "MAX is not at least MAX - 2 + 6");
    assert!(accepted(open(i64::MIN + 1, i64::MIN, T0)), "MIN is not below MIN + 1 - 3");
    assert!(wide(open(-1, i64::MAX, T0)), "MAX is at least 5");
    assert!(wide(open(i64::MIN, i64::MAX, T0)), "the opposite end against the largest offset");
    assert!(accepted(open(9_007_199_254_740_993, 9_007_199_254_740_998, T0)), "past binary64");
}
#[test]
fn equalities() {
    let above = |lower, upper| Generated::new(()).above(Above { lower, upper }).unwrap();
    let below = |lower, upper| Generated::new(()).below(Below { lower, upper }).unwrap();
    assert!(
        matches!(above(i64::MAX, i64::MAX), AboveOutcome::Apart { .. }),
        "MAX == MAX + 1 is false and MAX != MAX + 1 is true"
    );
    assert!(matches!(above(i64::MAX - 1, i64::MAX), AboveOutcome::Equal { .. }), "MAX == MAX - 1 + 1");
    assert!(
        matches!(below(i64::MIN, i64::MIN), BelowOutcome::Apart { .. }),
        "MIN == MIN - 1 is false and MIN != MIN - 1 is true"
    );
    assert!(matches!(below(i64::MIN + 1, i64::MIN), BelowOutcome::Equal { .. }), "MIN == MIN + 1 - 1");
}
"#;

#[test]
fn a2_offset_guards_in_the_generated_rust_behaviour_and_every_fault_fails() {
    let directory = emit(Target::Rust);
    let tests = directory.join("crates/pool-types/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("guards.rs"), RUST_GUARDS).unwrap();
    let cargo = env!("CARGO");
    let arguments = ["test", "--offline", "-p", "pool-types", "--test", "guards"];
    let (healthy, log) = run(&directory, cargo, &arguments);
    assert!(healthy && log.contains("3 passed; 0 failed"), "{log}");
    let behaviour = "crates/pool-types/src/behaviour.rs";
    let exact = "Some(accepts(left.cmp(&(base + offset))))";
    let faults = [
        (
            "wrap",
            exact,
            "Some(accepts((left as i64).cmp(&(base as i64).wrapping_add(offset as i64))))",
        ),
        (
            "clamp",
            exact,
            "Some(accepts((left as i64).cmp(&(base as i64).saturating_add(offset as i64))))",
        ),
        (
            "binary64",
            exact,
            "(left as f64).partial_cmp(&(base as f64 + offset as f64)).map(accepts)",
        ),
        (
            "unknown on overflow",
            exact,
            "Some(accepts((left as i64).cmp(&(base as i64).checked_add(offset as i64)?)))",
        ),
        ("ignored sign", "-3_i128", "3_i128"),
        ("ignored unit", "3600_i64", "1_i64"),
        (
            "ignored base",
            exact,
            "{ let _ = base; Some(accepts(left.cmp(&offset))) }",
        ),
    ];
    for (fault, before, after) in faults {
        patch(&directory, behaviour, before, after);
        let (passed, log) = run(&directory, cargo, &arguments);
        assert!(
            !passed && log.contains("FAILED"),
            "{fault}: the faulty guard passed\n{log}"
        );
        patch(&directory, behaviour, after, before);
    }
    let _ = std::fs::remove_dir_all(&directory);
}

const GO_GUARDS: &str = r#"
package behaviour

import (
	"math"
	"testing"

	"example.invalid/pool/types/lease"
	"example.invalid/pool/types/primitives"
)

const t0 = "2020-01-01T00:00:00Z"

func open(t *testing.T, lower, upper int64, expires string) lease.OpenOutcome {
	outcome, err := New(Ports{}).Open(lease.Open{
		Lower:     lower,
		Upper:     upper,
		IssuedAt:  primitives.NewTimestamp(t0),
		ExpiresAt: primitives.NewTimestamp(expires),
	})
	if err != nil {
		t.Fatal(err)
	}
	return outcome
}

func TestBoundaries(t *testing.T) {
	cases := []struct {
		name          string
		lower, upper  int64
		expires, want string
	}{
		{"the boundary itself is not wide", 2, 7, t0, "accepted"},
		{"one above the boundary is wide", 2, 8, t0, "wide"},
		{"the subtraction's boundary is not inverted", 2, -1, t0, "accepted"},
		{"one below it, below zero, is inverted", 2, -2, t0, "inverted"},
		{"a second before the hour", 0, 0, "2020-01-01T00:59:59Z", "accepted"},
		{"the hour itself", 0, 0, "2020-01-01T01:00:00Z", "late"},
		{"the hour, spelled in another offset", 0, 0, "2020-01-01T02:00:00+01:00", "late"},
		{"five minutes before", 0, 0, "2019-12-31T23:55:00Z", "accepted"},
		{"a second more", 0, 0, "2019-12-31T23:54:59Z", "soon"},
		{"MAX is not at least MAX - 2 + 6", math.MaxInt64 - 2, math.MaxInt64, t0, "accepted"},
		{"MIN is not below MIN + 1 - 3", math.MinInt64 + 1, math.MinInt64, t0, "accepted"},
		{"MAX is at least 5", -1, math.MaxInt64, t0, "wide"},
		{"the opposite end against the largest offset", math.MinInt64, math.MaxInt64, t0, "wide"},
		{"past binary64", 9007199254740993, 9007199254740998, t0, "accepted"},
	}
	for _, c := range cases {
		got := "accepted"
		switch open(t, c.lower, c.upper, c.expires).(type) {
		case lease.OpenOutcomeWide:
			got = "wide"
		case lease.OpenOutcomeInverted:
			got = "inverted"
		case lease.OpenOutcomeLate:
			got = "late"
		case lease.OpenOutcomeSoon:
			got = "soon"
		}
		if got != c.want {
			t.Errorf("%s: got %s, want %s", c.name, got, c.want)
		}
	}
}

func TestEqualities(t *testing.T) {
	behaviour := New(Ports{})
	above := func(lower, upper int64) lease.AboveOutcome {
		outcome, err := behaviour.Above(lease.Above{Lower: lower, Upper: upper})
		if err != nil {
			t.Fatal(err)
		}
		return outcome
	}
	below := func(lower, upper int64) lease.BelowOutcome {
		outcome, err := behaviour.Below(lease.Below{Lower: lower, Upper: upper})
		if err != nil {
			t.Fatal(err)
		}
		return outcome
	}
	if _, ok := above(math.MaxInt64, math.MaxInt64).(lease.AboveOutcomeApart); !ok {
		t.Error("MAX == MAX + 1 is false and MAX != MAX + 1 is true")
	}
	if _, ok := above(math.MaxInt64-1, math.MaxInt64).(lease.AboveOutcomeEqual); !ok {
		t.Error("MAX == MAX - 1 + 1")
	}
	if _, ok := below(math.MinInt64, math.MinInt64).(lease.BelowOutcomeApart); !ok {
		t.Error("MIN == MIN - 1 is false and MIN != MIN - 1 is true")
	}
	if _, ok := below(math.MinInt64+1, math.MinInt64).(lease.BelowOutcomeEqual); !ok {
		t.Error("MIN == MIN + 1 - 1")
	}
}
"#;

#[test]
fn a2_offset_guards_in_the_generated_go_behaviour_and_every_fault_fails() {
    let directory = emit(Target::Go);
    std::fs::write(directory.join("types/behaviour/guards_test.go"), GO_GUARDS).unwrap();
    let arguments = [
        "test",
        "-count=1",
        "-run",
        "TestBoundaries|TestEqualities",
        "./types/behaviour",
    ];
    let (healthy, log) = run(&directory, "go", &arguments);
    assert!(healthy && log.contains("ok"), "{log}");
    let behaviour = "types/behaviour/behaviour.go";
    let exact = "leftValue.Cmp(baseValue.Add(baseValue, delta))";
    let faults = [
        (
            "wrap",
            exact,
            "big.NewInt(leftValue.Int64()).Cmp(big.NewInt(baseValue.Int64() + delta.Int64()))",
        ),
        (
            "clamp",
            exact,
            "leftValue.Cmp(clampOffset(baseValue.Add(baseValue, delta)))",
        ),
        (
            "binary64",
            exact,
            "new(big.Float).SetInt(leftValue).Cmp(new(big.Float).SetFloat64(float64(baseValue.Int64()) + float64(delta.Int64())))",
        ),
        (
            "unknown on overflow",
            "return known(accepts(leftValue.Cmp(baseValue.Add(baseValue, delta))))",
            "if sum := new(big.Int).Add(baseValue, delta); !sum.IsInt64() {\n\t\treturn unknown\n\t}\n\treturn known(accepts(leftValue.Cmp(baseValue.Add(baseValue, delta))))",
        ),
        ("ignored sign", "\"-3\"", "\"3\""),
        ("ignored unit", "3600, isGe", "1, isGe"),
        ("ignored base", exact, "leftValue.Cmp(delta)"),
    ];
    let clamp = "\nfunc clampOffset(value *big.Int) *big.Int {\n\tif value.Cmp(big.NewInt(math.MaxInt64)) > 0 {\n\t\treturn big.NewInt(math.MaxInt64)\n\t}\n\tif value.Cmp(big.NewInt(math.MinInt64)) < 0 {\n\t\treturn big.NewInt(math.MinInt64)\n\t}\n\treturn value\n}\n";
    for (fault, before, after) in faults {
        patch(&directory, behaviour, before, after);
        if fault == "clamp" {
            let path = directory.join(behaviour);
            let mut text = std::fs::read_to_string(&path).unwrap();
            text.push_str(clamp);
            text = text.replacen("import (", "import (\n\t\"math\"", 1);
            std::fs::write(&path, text).unwrap();
        }
        let (passed, log) = run(&directory, "go", &arguments);
        assert!(
            !passed && log.contains("FAIL"),
            "{fault}: the faulty guard passed\n{log}"
        );
        if fault == "clamp" {
            let path = directory.join(behaviour);
            let text = std::fs::read_to_string(&path).unwrap();
            let text = text
                .replace(clamp, "")
                .replacen("import (\n\t\"math\"", "import (", 1);
            std::fs::write(&path, text).unwrap();
        }
        patch(&directory, behaviour, after, before);
    }
    let _ = std::fs::remove_dir_all(&directory);
}

fn disposition(text: &str, kind: CapabilityKind, source: &str) -> String {
    let synthesis =
        synthesize_for(&ir_of(text), Target::Rust).unwrap_or_else(|_| panic!("synthesizes"));
    match synthesis.plan.disposition_of(kind, source) {
        Some(SynthesisDisposition::Obligation(obligation)) => {
            serde_json::to_string(&obligation.reason).unwrap()
        }
        other => format!("{other:?}"),
    }
}

#[test]
fn a2_an_offset_in_a_view_filter_stays_owed_by_name() {
    let text = MODEL.replace(
        "events:\n",
        "entities:\n  - name: pool.lease.Lease\n    identity: {name: lease_id, type: Uuid}\n    fields:\n      - {name: lower, type: Integer}\n      - {name: upper, type: Integer}\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\nviews:\n  - name: pool.lease.WideRows\n    source: pool.lease.Lease\n    consistency: read_your_writes\n    filter: upper > lower + 5\n    fields:\n      - {name: lease_id, type: Uuid}\nevents:\n",
    );
    let reason = disposition(&text, CapabilityKind::ViewQuery, "pool.lease.WideRows");
    assert!(
        reason.contains("an offset the generated view query does not compare"),
        "{reason}"
    );
}

#[test]
fn a2_an_offset_invariant_refuses_the_generated_target_by_name() {
    let text = MODEL.replace(
        "events:\n",
        "entities:\n  - name: pool.lease.Lease\n    identity: {name: lease_id, type: Uuid}\n    fields:\n      - {name: lower, type: Integer}\n      - {name: upper, type: Integer}\n    invariants:\n      - upper <= lower + 5\n    lifecycle: {initial: Open, states: [Open], terminal: [Open]}\nevents:\n  - {name: pool.lease.Opened, fields: [{name: lease_id, type: Uuid}]}\n",
    )
    .replace(
        "      - name: accepted\n        emits: [pool.lease.Accepted]\n",
        "      - name: accepted\n        creates: pool.lease.Lease\n        instance: lease_id\n        sets: {lower: input.lower, upper: input.upper}\n        emits: [pool.lease.Accepted, pool.lease.Opened]\n        payload: {pool.lease.Opened: {lease_id: {generated: true}}}\n",
    )
    .replace("pool.lease.Accepted]}", "pool.lease.Accepted, pool.lease.Opened]}");
    for target in [Target::Rust, Target::Go] {
        let refused = synthesize_for(&ir_of(&text), target)
            .err()
            .unwrap_or_else(|| panic!("{target:?}: an offset invariant is refused"));
        let rendered = format!("{refused:?}");
        assert!(
            rendered.contains("moves a fact by a constant"),
            "{target:?}: {rendered}"
        );
    }
}
