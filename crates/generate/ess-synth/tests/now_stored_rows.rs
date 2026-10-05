//! Generated Rust and Go behaviour reading `now` over stored and related rows
//! (`docs/design/expression-family-source22.md`, "A3: current time over stored and related rows",
//! "One observed decision instant per command occurrence"; beyond10x/ess#244 part a, unit U5).
//!
//! A command whose guard orders an instant against the current time is generated from `ess/22`
//! with an explicit command-clock capability on its context: read once per decision, before any
//! guard, and every guard reads that one instant. With no clock the decision that needs one is
//! refused naming the command clock; every answer decided before such a guard stands.
//!
//! Each emitted tree is compiled and run against a scripted clock answering the setup instant
//! `T0` for the arranging commands and the decision instant `T1` for the command under test,
//! healthy and with each faulty seam patched in: the setup reading reused, the clock reread per
//! guard, the wrong related row, the row as the effect would leave it, and instants compared by
//! their bytes.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, CapabilityKind, SynthesisDisposition, Target};
use std::path::{Path, PathBuf};
use std::process::Command;

const LEASES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/now-stored-rows.yaml");

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("leases.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// Every artifact of `target`'s synthesis of `text`, written under a fresh directory.
fn emit(text: &str, target: Target, case: &str) -> PathBuf {
    let synthesis = synthesize_for(&model(text), target).unwrap();
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "now-stored-{}-{case}-{}",
        target.name(),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&directory);
    for (relative, artifact) in synthesis.artifacts {
        let path = directory.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, artifact.contents).unwrap();
    }
    eprintln!("emitted {}", directory.display());
    directory
}

#[test]
fn a3_commands_reading_now_over_rows_are_generated_with_a_command_clock() {
    let ir = model(LEASES);
    for target in [Target::Rust, Target::Go] {
        let synthesis = synthesize_for(&ir, target).unwrap();
        for command in ["demo.leases.RenewLease", "demo.leases.Join"] {
            assert_eq!(
                synthesis
                    .plan
                    .disposition_of(CapabilityKind::CommandBehavior, command),
                Some(&SynthesisDisposition::Generated),
                "{target:?}: {command}"
            );
        }
        // Several related rows stay owed, as they are without `now` (beyond10x/ess#283).
        assert!(matches!(
            synthesis
                .plan
                .disposition_of(CapabilityKind::CommandBehavior, "demo.leases.Lend"),
            Some(SynthesisDisposition::Obligation(_))
        ));
    }
    emit(LEASES, Target::Rust, "probe");
    emit(LEASES, Target::Go, "probe");
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
    (log, output.status.success())
}

/// Replaces every `from` with `to` in `path`, asserting `from` is there: one faulty seam.
fn patch(path: &Path, from: &str, to: &str) {
    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.contains(from), "{} carries `{from}`", path.display());
    std::fs::write(path, text.replace(from, to)).unwrap();
}

/// The scripted clock and the instants every control reads: the setup instant `T0`, the decision
/// instant `T1`, and the stored instants on each side of every boundary at `T1`.
const RUST_CONTROL: &str = r#"
use demo_types::behaviour::{BookStorage, Context, Generated, LeaseStorage, MemberStorage};
use demo_types::leases::{self, obligations::*};
use demo_types::obligation::UnmetObligation;
use demo_types::primitives::{Timestamp, Uuid};

const T0: &str = "2000-06-01T00:00:00Z";
const T1_SECONDS: i64 = 1_791_115_200;
const PAST: &str = "1990-01-01T00:00:00Z";
const FUTURE: &str = "2090-01-01T00:00:00Z";
const T1_LESS_1H: &str = "2026-10-04T11:00:00.123456789Z";
const T1_LESS_1H_ELSEWHERE: &str = "2026-10-04T12:00:00.123456789+01:00";
const T1_LESS_1H_LESS_1NS: &str = "2026-10-04T11:00:00.123456788Z";
const T1_LESS_1H_LESS_1NS_ELSEWHERE: &str = "2026-10-04T12:00:00.123456788+01:00";
const T1_PLUS_5M: &str = "2026-10-04T12:05:00.123456789Z";
const T1_PLUS_5M_PLUS_1NS: &str = "2026-10-04T12:05:00.12345679Z";
const T1_PLUS_30S: &str = "2026-10-04T12:00:30.123456789Z";
const T1_PLUS_30S_PLUS_1NS: &str = "2026-10-04T12:00:30.12345679Z";

/// The decision instant moved by `days`: `T1` is 2026-10-04T12:00:00.123456789Z.
fn decision(days: i64) -> String {
    let seconds = T1_SECONDS + days * 86_400;
    let (day, rest) = (seconds.div_euclid(86_400), seconds.rem_euclid(86_400));
    let z = day + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}.123456789Z",
        rest / 3600,
        rest / 60 % 60,
        rest % 60
    )
}

#[derive(Default)]
struct Ports {
    leases: Vec<leases::LeaseSnapshot>,
    members: Vec<leases::MemberSnapshot>,
    books: Vec<leases::BookSnapshot>,
    minted: u32,
    clocked: bool,
    testing: bool,
    in_decision: i64,
    reads: usize,
}

impl Ports {
    fn uuid(&mut self) -> Uuid {
        self.minted += 1;
        Uuid(format!("00000000-0000-4000-8000-{:012}", self.minted))
    }
}

impl Context for Ports {
    fn generate_demo_leases_book_id(&mut self) -> leases::BookId {
        leases::BookId(self.uuid())
    }
    fn generate_demo_leases_lease_id(&mut self) -> leases::LeaseId {
        leases::LeaseId(self.uuid())
    }
    fn generate_demo_leases_member_id(&mut self) -> leases::MemberId {
        leases::MemberId(self.uuid())
    }
    /// `T0` while arranging; `T1` for the decision under test, each further read two days later.
    fn command_clock(&mut self) -> Option<Timestamp> {
        if !self.clocked {
            return None;
        }
        self.reads += 1;
        if !self.testing {
            return Some(Timestamp(T0.to_owned()));
        }
        self.in_decision += 1;
        Some(Timestamp(decision(2 * (self.in_decision - 1))))
    }
}

impl LeaseStorage for Ports {
    fn get(&self, identity: &leases::LeaseId) -> Option<leases::LeaseSnapshot> {
        self.leases.iter().find(|row| &row.data.lease_id == identity).cloned()
    }
    fn put(&mut self, snapshot: leases::LeaseSnapshot) {
        LeaseStorage::delete(self, &snapshot.data.lease_id.clone());
        self.leases.push(snapshot);
    }
    fn delete(&mut self, identity: &leases::LeaseId) {
        self.leases.retain(|row| &row.data.lease_id != identity);
    }
    fn list(&self) -> Vec<leases::LeaseSnapshot> {
        self.leases.clone()
    }
}

impl MemberStorage for Ports {
    fn get(&self, identity: &leases::MemberId) -> Option<leases::MemberSnapshot> {
        self.members.iter().find(|row| &row.data.member_id == identity).cloned()
    }
    fn put(&mut self, snapshot: leases::MemberSnapshot) {
        MemberStorage::delete(self, &snapshot.data.member_id.clone());
        self.members.push(snapshot);
    }
    fn delete(&mut self, identity: &leases::MemberId) {
        self.members.retain(|row| &row.data.member_id != identity);
    }
    fn list(&self) -> Vec<leases::MemberSnapshot> {
        self.members.clone()
    }
}

impl BookStorage for Ports {
    fn get(&self, identity: &leases::BookId) -> Option<leases::BookSnapshot> {
        self.books.iter().find(|row| &row.data.book_id == identity).cloned()
    }
    fn put(&mut self, snapshot: leases::BookSnapshot) {
        BookStorage::delete(self, &snapshot.data.book_id.clone());
        self.books.push(snapshot);
    }
    fn delete(&mut self, identity: &leases::BookId) {
        self.books.retain(|row| &row.data.book_id != identity);
    }
    fn list(&self) -> Vec<leases::BookSnapshot> {
        self.books.clone()
    }
}

fn fresh(clocked: bool) -> Generated<Ports> {
    Generated::new(Ports { clocked, ..Ports::default() })
}

fn open(service: &mut Generated<Ports>, expires_at: &str, grace_until: &str) -> leases::LeaseId {
    match service
        .open_lease(leases::OpenLease {
            expires_at: Timestamp(expires_at.to_owned()),
            grace_until: Timestamp(grace_until.to_owned()),
        })
        .unwrap()
    {
        leases::OpenLeaseOutcome::Opened { lease_opened } => lease_opened.lease_id,
    }
}

fn register(service: &mut Generated<Ports>, banned_until: &str) -> leases::MemberId {
    match service
        .register_member(leases::RegisterMember {
            banned_until: Timestamp(banned_until.to_owned()),
        })
        .unwrap()
    {
        leases::RegisterMemberOutcome::Registered { member_registered } => {
            member_registered.member_id
        }
    }
}

/// A decision made while arranging, which reads the clock at `T0`: a renewal of a lease nobody holds.
fn arranging_decision(service: &mut Generated<Ports>) {
    let nobody = leases::LeaseId(Uuid("00000000-0000-4000-8000-00000000beef".to_owned()));
    let answered = service.renew_lease(leases::RenewLease {
        lease_id: nobody,
        new_expires_at: Timestamp(FUTURE.to_owned()),
        note: "arranging".to_owned(),
    });
    assert!(matches!(answered, Ok(leases::RenewLeaseOutcome::UnknownLease { .. })));
}

/// The decision under test begins: its reads answer `T1`.
fn testing(service: &mut Generated<Ports>) {
    service.ports.testing = true;
    service.ports.in_decision = 0;
}

fn renew(
    service: &mut Generated<Ports>,
    lease: leases::LeaseId,
    note: &str,
) -> Result<leases::RenewLeaseOutcome, UnmetObligation> {
    testing(service);
    service.renew_lease(leases::RenewLease {
        lease_id: lease,
        new_expires_at: Timestamp(FUTURE.to_owned()),
        note: note.to_owned(),
    })
}

fn renewal(outcome: &leases::RenewLeaseOutcome) -> &'static str {
    match outcome {
        leases::RenewLeaseOutcome::BlankNote { .. } => "blank-note",
        leases::RenewLeaseOutcome::Renewed { .. } => "renewed",
        leases::RenewLeaseOutcome::Graced { .. } => "graced",
        leases::RenewLeaseOutcome::Lapsed { .. } => "lapsed",
        leases::RenewLeaseOutcome::NotActive { .. } => "not-active",
        leases::RenewLeaseOutcome::UnknownLease { .. } => "unknown-lease",
    }
}

fn joining(outcome: &leases::JoinOutcome) -> &'static str {
    match outcome {
        leases::JoinOutcome::NoMemberToJoin { .. } => "no-member-to-join",
        leases::JoinOutcome::BannedFromJoining { .. } => "banned-from-joining",
        leases::JoinOutcome::Joined { .. } => "joined",
    }
}

#[test]
fn subject_rows_are_decided_at_the_decision_instant() {
    for (expires_at, grace_until, expected) in [
        (T1_LESS_1H, PAST, "renewed"),
        (T1_LESS_1H_ELSEWHERE, PAST, "renewed"),
        (T1_LESS_1H_LESS_1NS, PAST, "lapsed"),
        (T1_LESS_1H_LESS_1NS_ELSEWHERE, PAST, "lapsed"),
        (PAST, T1_PLUS_5M_PLUS_1NS, "graced"),
        (PAST, T1_PLUS_5M, "lapsed"),
    ] {
        let mut service = fresh(true);
        let lease = open(&mut service, expires_at, grace_until);
        arranging_decision(&mut service);
        let answered = renew(&mut service, lease, "renewal").unwrap();
        assert_eq!(renewal(&answered), expected, "{expires_at} / {grace_until}");
        assert_eq!(service.ports.reads, 2, "one read per decision");
    }
}

#[test]
fn related_rows_are_decided_at_the_decision_instant_and_by_the_row_named() {
    for (banned_until, expected) in [
        (T1_PLUS_30S_PLUS_1NS, "banned-from-joining"),
        (T1_PLUS_30S, "joined"),
    ] {
        let mut service = fresh(true);
        // A decoy on the other side of the bound, stored first.
        let decoy = if expected == "joined" { FUTURE } else { PAST };
        let _ = register(&mut service, decoy);
        let member = register(&mut service, banned_until);
        arranging_decision(&mut service);
        testing(&mut service);
        let answered = service.join(leases::Join { member_id: member }).unwrap();
        assert_eq!(joining(&answered), expected, "{banned_until}");
        assert_eq!(service.ports.reads, 2, "one read per decision");
    }
}

#[test]
fn without_a_clock_a_needed_instant_is_refused_by_name_and_earlier_answers_stand() {
    let clock = UnmetObligation { capability: "command clock", source: "demo.leases.RenewLease" };
    let mut service = fresh(false);
    let lease = open(&mut service, FUTURE, FUTURE);
    assert_eq!(renew(&mut service, lease.clone(), "renewal"), Err(clock));
    assert_eq!(renewal(&renew(&mut service, lease.clone(), "").unwrap()), "blank-note");
    let nobody = leases::LeaseId(Uuid("00000000-0000-4000-8000-00000000dead".to_owned()));
    assert_eq!(renewal(&renew(&mut service, nobody, "renewal").unwrap()), "unknown-lease");
    // No effect was licensed by the refused decision: the lease renews once a clock is read.
    service.ports.clocked = true;
    assert_eq!(renewal(&renew(&mut service, lease, "renewal").unwrap()), "renewed");
    let mut service = fresh(false);
    let member = register(&mut service, PAST);
    assert_eq!(
        service.join(leases::Join { member_id: member }),
        Err(UnmetObligation { capability: "command clock", source: "demo.leases.Join" })
    );
    let nobody = leases::MemberId(Uuid("00000000-0000-4000-8000-00000000dead".to_owned()));
    assert_eq!(
        joining(&service.join(leases::Join { member_id: nobody }).unwrap()),
        "no-member-to-join"
    );
}
"#;

/// The emitted Rust workspace, with `faults` patched into its behaviour, running the control: the
/// cargo log, and whether every control passed.
fn rust_control(case: &str, faults: &[(&str, &str)]) -> (String, bool) {
    let directory = emit(LEASES, Target::Rust, case);
    let behaviour = directory.join("crates/demo-types/src/behaviour.rs");
    for (from, to) in faults {
        patch(&behaviour, from, to);
    }
    let tests = directory.join("crates/demo-types/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("control.rs"), RUST_CONTROL).unwrap();
    let ran = run(
        &directory,
        env!("CARGO"),
        &["test", "--offline", "-p", "demo-types", "--test", "control"],
    );
    let _ = std::fs::remove_dir_all(&directory);
    ran
}

/// A fault the control catches: the workspace still builds, and the named control fails.
fn assert_caught(case: &str, (log, passed): (String, bool), control: &str) {
    assert!(
        !passed,
        "{case}: the faulty behaviour passed every control:\n{log}"
    );
    assert!(
        log.contains(&format!("test {control} ... FAILED")),
        "{case}: `{control}` fails, not the build:\n{log}"
    );
}

/// The decision's one instant, as the healthy behaviour reads it.
const READ_ONCE: &str = "let now = self.ports.try_command_clock();";

#[test]
fn a3_generated_rust_decides_every_row_with_the_one_instant() {
    let (log, passed) = rust_control("healthy", &[]);
    assert!(passed && log.contains("3 passed; 0 failed"), "{log}");
}

#[test]
fn a3_generated_rust_faulty_seams_fail_the_control() {
    // The first reading the process made, reused for every decision: the setup instant.
    assert_caught(
        "setup-reused",
        rust_control(
            "setup-reused",
            &[(
                READ_ONCE,
                "let now = { static FIRST: std::sync::Mutex<Option<Result<Option<crate::primitives::Timestamp>, UnmetObligation>>> = std::sync::Mutex::new(None); FIRST.lock().unwrap().get_or_insert(self.ports.try_command_clock()).clone() };",
            )],
        ),
        "subject_rows_are_decided_at_the_decision_instant",
    );
    // The clock read again for every guard.
    assert_caught(
        "reread",
        rust_control(
            "reread",
            &[(
                ".0.clone()), &now, ",
                ".0.clone()), &self.ports.try_command_clock(), ",
            )],
        ),
        "subject_rows_are_decided_at_the_decision_instant",
    );
    // The first stored member read, not the one the input names.
    assert_caught(
        "wrong-row",
        rust_control(
            "wrong-row",
            &[(
                "let related = reference.and_then(|identity| MemberStorage::get(&self.ports, identity));",
                "let related = reference.and_then(|_| MemberStorage::list(&self.ports).into_iter().next());",
            )],
        ),
        "related_rows_are_decided_at_the_decision_instant_and_by_the_row_named",
    );
    // The expiry the renewal would write, read instead of the one stored before it.
    assert_caught(
        "post-effect",
        rust_control(
            "post-effect",
            &[(
                "compare_with_now(Some(&held.data.expires_at)",
                "compare_with_now(Some(&input.new_expires_at)",
            )],
        ),
        "subject_rows_are_decided_at_the_decision_instant",
    );
    // Instants ordered by their spellings.
    assert_caught(
        "bytes",
        rust_control(
            "bytes",
            &[(
                "    let (at, nanos) = instant_of(&value?)?;\n    let (decided, decided_nanos) = instant_of(&now.as_ref().ok()?.as_ref()?.0)?;\n    Some(accepts((at, nanos).cmp(&(decided.checked_add(seconds)?, decided_nanos))))",
                "    let (decided, decided_nanos) = instant_of(&now.as_ref().ok()?.as_ref()?.0)?;\n    Some(accepts(value?.as_bytes().cmp(spell(decided.checked_add(seconds)?, decided_nanos).as_bytes())))\n}\n\nfn spell(seconds: i64, nanos: u32) -> String {\n    let (day, rest) = (seconds.div_euclid(86_400), seconds.rem_euclid(86_400));\n    let z = day + 719_468;\n    let era = z.div_euclid(146_097);\n    let doe = z - era * 146_097;\n    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;\n    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);\n    let mp = (5 * doy + 2) / 153;\n    let d = doy - (153 * mp + 2) / 5 + 1;\n    let m = if mp < 10 { mp + 3 } else { mp - 9 };\n    let y = yoe + era * 400 + i64::from(m <= 2);\n    let fraction = format!(\".{nanos:09}\");\n    format!(\"{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}{}Z\", rest / 3600, rest / 60 % 60, rest % 60, fraction.trim_end_matches('0').trim_end_matches('.'))",
            )],
        ),
        "subject_rows_are_decided_at_the_decision_instant",
    );
}

/// The Go control: the same instants, scripted clock and cases as [`RUST_CONTROL`].
const GO_CONTROL: &str = r#"
package behaviour

import (
	"fmt"
	"testing"
	"time"

	"example.invalid/demo/types/leases"
	"example.invalid/demo/types/obligation"
	"example.invalid/demo/types/primitives"
)

const (
	t0                    = "2000-06-01T00:00:00Z"
	t1                    = "2026-10-04T12:00:00.123456789Z"
	past                  = "1990-01-01T00:00:00Z"
	future                = "2090-01-01T00:00:00Z"
	t1Less1h              = "2026-10-04T11:00:00.123456789Z"
	t1Less1hElsewhere     = "2026-10-04T12:00:00.123456789+01:00"
	t1Less1hLess1ns       = "2026-10-04T11:00:00.123456788Z"
	t1Less1hLess1nsElse   = "2026-10-04T12:00:00.123456788+01:00"
	t1Plus5m              = "2026-10-04T12:05:00.123456789Z"
	t1Plus5mPlus1ns       = "2026-10-04T12:05:00.12345679Z"
	t1Plus30s             = "2026-10-04T12:00:30.123456789Z"
	t1Plus30sPlus1ns      = "2026-10-04T12:00:30.12345679Z"
)

type control struct {
	leases     []leases.LeaseSnapshot
	members    []leases.MemberSnapshot
	books      []leases.BookSnapshot
	minted     int
	clocked    bool
	testing    bool
	inDecision int
	reads      int
}

func (c *control) uuid() primitives.Uuid {
	c.minted++
	return primitives.NewUuid(fmt.Sprintf("00000000-0000-4000-8000-%012d", c.minted))
}

func (c *control) GenerateDemoLeasesBookId() leases.BookId     { return leases.NewBookId(c.uuid()) }
func (c *control) GenerateDemoLeasesLeaseId() leases.LeaseId   { return leases.NewLeaseId(c.uuid()) }
func (c *control) GenerateDemoLeasesMemberId() leases.MemberId { return leases.NewMemberId(c.uuid()) }

// CommandClock answers t0 while arranging; t1 for the decision under test, each further read two
// days later.
func (c *control) CommandClock() (primitives.Timestamp, bool) {
	if !c.clocked {
		return primitives.Timestamp{}, false
	}
	c.reads++
	if !c.testing {
		return primitives.NewTimestamp(t0), true
	}
	at, _ := time.Parse(time.RFC3339Nano, t1)
	at = at.Add(time.Duration(c.inDecision) * 48 * time.Hour)
	c.inDecision++
	return primitives.NewTimestamp(at.UTC().Format(time.RFC3339Nano)), true
}

type leaseRows struct{ c *control }

func (s leaseRows) Get(identity leases.LeaseId) (leases.LeaseSnapshot, bool) {
	for _, row := range s.c.leases {
		if row.Data.LeaseId == identity {
			return row, true
		}
	}
	return leases.LeaseSnapshot{}, false
}
func (s leaseRows) Put(snapshot leases.LeaseSnapshot) {
	s.Delete(snapshot.Data.LeaseId)
	s.c.leases = append(s.c.leases, snapshot)
}
func (s leaseRows) Delete(identity leases.LeaseId) {
	kept := s.c.leases[:0]
	for _, row := range s.c.leases {
		if row.Data.LeaseId != identity {
			kept = append(kept, row)
		}
	}
	s.c.leases = kept
}
func (s leaseRows) List() []leases.LeaseSnapshot { return append([]leases.LeaseSnapshot{}, s.c.leases...) }

type memberRows struct{ c *control }

func (s memberRows) Get(identity leases.MemberId) (leases.MemberSnapshot, bool) {
	for _, row := range s.c.members {
		if row.Data.MemberId == identity {
			return row, true
		}
	}
	return leases.MemberSnapshot{}, false
}
func (s memberRows) Put(snapshot leases.MemberSnapshot) {
	s.Delete(snapshot.Data.MemberId)
	s.c.members = append(s.c.members, snapshot)
}
func (s memberRows) Delete(identity leases.MemberId) {
	kept := s.c.members[:0]
	for _, row := range s.c.members {
		if row.Data.MemberId != identity {
			kept = append(kept, row)
		}
	}
	s.c.members = kept
}
func (s memberRows) List() []leases.MemberSnapshot {
	return append([]leases.MemberSnapshot{}, s.c.members...)
}

type bookRows struct{ c *control }

func (s bookRows) Get(identity leases.BookId) (leases.BookSnapshot, bool) {
	for _, row := range s.c.books {
		if row.Data.BookId == identity {
			return row, true
		}
	}
	return leases.BookSnapshot{}, false
}
func (s bookRows) Put(snapshot leases.BookSnapshot) {
	s.Delete(snapshot.Data.BookId)
	s.c.books = append(s.c.books, snapshot)
}
func (s bookRows) Delete(identity leases.BookId) {
	kept := s.c.books[:0]
	for _, row := range s.c.books {
		if row.Data.BookId != identity {
			kept = append(kept, row)
		}
	}
	s.c.books = kept
}
func (s bookRows) List() []leases.BookSnapshot { return append([]leases.BookSnapshot{}, s.c.books...) }

func fresh(clocked bool) (*Generated, *control) {
	c := &control{clocked: clocked}
	return New(Ports{LeaseStorage: leaseRows{c}, MemberStorage: memberRows{c}, BookStorage: bookRows{c}, Context: c}), c
}

func open(t *testing.T, g *Generated, expiresAt, graceUntil string) leases.LeaseId {
	outcome, err := g.OpenLease(leases.OpenLease{ExpiresAt: primitives.NewTimestamp(expiresAt), GraceUntil: primitives.NewTimestamp(graceUntil)})
	if err != nil {
		t.Fatal(err)
	}
	return outcome.(leases.OpenLeaseOutcomeOpened).LeaseOpened.LeaseId
}

func register(t *testing.T, g *Generated, bannedUntil string) leases.MemberId {
	outcome, err := g.RegisterMember(leases.RegisterMember{BannedUntil: primitives.NewTimestamp(bannedUntil)})
	if err != nil {
		t.Fatal(err)
	}
	return outcome.(leases.RegisterMemberOutcomeRegistered).MemberRegistered.MemberId
}

func renew(g *Generated, c *control, lease leases.LeaseId, note string) (leases.RenewLeaseOutcome, *obligation.UnmetObligation) {
	c.testing, c.inDecision = true, 0
	return g.RenewLease(leases.RenewLease{LeaseId: lease, NewExpiresAt: primitives.NewTimestamp(future), Note: note})
}

// arrangingDecision is a decision made while arranging, which reads the clock at t0.
func arrangingDecision(t *testing.T, g *Generated) {
	nobody := leases.NewLeaseId(primitives.NewUuid("00000000-0000-4000-8000-00000000beef"))
	outcome, err := g.RenewLease(leases.RenewLease{LeaseId: nobody, NewExpiresAt: primitives.NewTimestamp(future), Note: "arranging"})
	if _, ok := outcome.(leases.RenewLeaseOutcomeUnknownLease); err != nil || !ok {
		t.Fatalf("arranging: %v %v", outcome, err)
	}
}

func renewal(outcome leases.RenewLeaseOutcome) string {
	switch outcome.(type) {
	case leases.RenewLeaseOutcomeBlankNote:
		return "blank-note"
	case leases.RenewLeaseOutcomeRenewed:
		return "renewed"
	case leases.RenewLeaseOutcomeGraced:
		return "graced"
	case leases.RenewLeaseOutcomeLapsed:
		return "lapsed"
	case leases.RenewLeaseOutcomeNotActive:
		return "not-active"
	case leases.RenewLeaseOutcomeUnknownLease:
		return "unknown-lease"
	}
	return "none"
}

func joining(outcome leases.JoinOutcome) string {
	switch outcome.(type) {
	case leases.JoinOutcomeNoMemberToJoin:
		return "no-member-to-join"
	case leases.JoinOutcomeBannedFromJoining:
		return "banned-from-joining"
	case leases.JoinOutcomeJoined:
		return "joined"
	}
	return "none"
}

func TestSubjectRows(t *testing.T) {
	for _, row := range [][3]string{
		{t1Less1h, past, "renewed"},
		{t1Less1hElsewhere, past, "renewed"},
		{t1Less1hLess1ns, past, "lapsed"},
		{t1Less1hLess1nsElse, past, "lapsed"},
		{past, t1Plus5mPlus1ns, "graced"},
		{past, t1Plus5m, "lapsed"},
	} {
		g, c := fresh(true)
		lease := open(t, g, row[0], row[1])
		arrangingDecision(t, g)
		outcome, err := renew(g, c, lease, "renewal")
		if err != nil || renewal(outcome) != row[2] {
			t.Fatalf("%s / %s: %v %v, want %s", row[0], row[1], renewal(outcome), err, row[2])
		}
		if c.reads != 2 {
			t.Fatalf("%d reads, want one per decision", c.reads)
		}
	}
}

func TestRelatedRows(t *testing.T) {
	for _, row := range [][2]string{
		{t1Plus30sPlus1ns, "banned-from-joining"},
		{t1Plus30s, "joined"},
	} {
		g, c := fresh(true)
		decoy := past
		if row[1] == "joined" {
			decoy = future
		}
		register(t, g, decoy)
		member := register(t, g, row[0])
		arrangingDecision(t, g)
		c.testing, c.inDecision = true, 0
		outcome, err := g.Join(leases.Join{MemberId: member})
		if err != nil || joining(outcome) != row[1] {
			t.Fatalf("%s: %v %v, want %s", row[0], joining(outcome), err, row[1])
		}
		if c.reads != 2 {
			t.Fatalf("%d reads, want one per decision", c.reads)
		}
	}
}

func TestNoClock(t *testing.T) {
	g, c := fresh(false)
	lease := open(t, g, future, future)
	if _, err := renew(g, c, lease, "renewal"); err == nil || err.Capability != "command clock" || err.Source != "demo.leases.RenewLease" {
		t.Fatalf("the missing clock is named: %v", err)
	}
	if outcome, err := renew(g, c, lease, ""); err != nil || renewal(outcome) != "blank-note" {
		t.Fatalf("blank-note: %v %v", outcome, err)
	}
	nobody := leases.NewLeaseId(primitives.NewUuid("00000000-0000-4000-8000-00000000dead"))
	if outcome, err := renew(g, c, nobody, "renewal"); err != nil || renewal(outcome) != "unknown-lease" {
		t.Fatalf("unknown-lease: %v %v", outcome, err)
	}
	c.clocked = true
	if outcome, err := renew(g, c, lease, "renewal"); err != nil || renewal(outcome) != "renewed" {
		t.Fatalf("no effect was licensed: %v %v", outcome, err)
	}
	g, _ = fresh(false)
	member := register(t, g, past)
	if _, err := g.Join(leases.Join{MemberId: member}); err == nil || err.Capability != "command clock" {
		t.Fatalf("join: %v", err)
	}
	stranger := leases.NewMemberId(primitives.NewUuid("00000000-0000-4000-8000-00000000dead"))
	if outcome, err := g.Join(leases.Join{MemberId: stranger}); err != nil || joining(outcome) != "no-member-to-join" {
		t.Fatalf("no-member-to-join: %v %v", outcome, err)
	}
}
"#;

/// The emitted Go module, with `faults` patched into its behaviour, running the control: the
/// `go test` log, and whether every control passed.
fn go_control(case: &str, faults: &[(&str, &str)]) -> (String, bool) {
    let directory = emit(LEASES, Target::Go, case);
    let behaviour = directory.join("types/behaviour/behaviour.go");
    for (from, to) in faults {
        patch(&behaviour, from, to);
    }
    std::fs::write(
        directory.join("types/behaviour/control_test.go"),
        GO_CONTROL,
    )
    .unwrap();
    let ran = run(
        &directory,
        "go",
        &["test", "-count=1", "-v", "./types/behaviour"],
    );
    let _ = std::fs::remove_dir_all(&directory);
    ran
}

fn assert_go_caught(case: &str, (log, passed): (String, bool), control: &str) {
    assert!(
        !passed,
        "{case}: the faulty behaviour passed every control:\n{log}"
    );
    assert!(
        log.contains(&format!("--- FAIL: {control}")),
        "{case}: `{control}` fails, not the build:\n{log}"
    );
}

#[test]
fn a3_generated_go_decides_every_row_with_the_one_instant() {
    let (log, passed) = go_control("healthy", &[]);
    assert!(passed, "{log}");
    for control in ["TestSubjectRows", "TestRelatedRows", "TestNoClock"] {
        assert!(log.contains(&format!("--- PASS: {control}")), "{log}");
    }
}

#[test]
fn a3_generated_go_faulty_seams_fail_the_control() {
    let first = "\nvar firstReading *primitives.Timestamp\n\nfunc first(b *Generated) (primitives.Timestamp, bool, *obligation.UnmetObligation) {\n\tnow, clocked, err := b.readCommandClock()\n\tif firstReading == nil {\n\t\tfirstReading = &now\n\t}\n\treturn *firstReading, clocked, err\n}\n";
    assert_go_caught(
        "setup-reused",
        go_control_with(
            "setup-reused",
            &[(
                "now, clocked, clockUnavailable := b.readCommandClock()",
                "now, clocked, clockUnavailable := first(b)",
            )],
            first,
        ),
        "TestSubjectRows",
    );
    let reread = "\nfunc reread(b *Generated) string {\n\tnow, _, _ := b.readCommandClock()\n\treturn now.Value()\n}\n";
    assert_go_caught(
        "reread",
        go_control_with(
            "reread",
            &[("now.Value(), clocked,", "reread(b), clocked,")],
            reread,
        ),
        "TestSubjectRows",
    );
    assert_go_caught(
        "wrong-row",
        go_control(
            "wrong-row",
            &[(
                "row, found := b.ports.MemberStorage.Get(*reference)",
                "rows := b.ports.MemberStorage.List()\n\t\trow, found := leases.MemberSnapshot{}, len(rows) > 0\n\t\tif found {\n\t\t\trow = rows[0]\n\t\t}",
            )],
        ),
        "TestRelatedRows",
    );
    assert_go_caught(
        "post-effect",
        go_control(
            "post-effect",
            &[(
                "compareWithNow(some(held.Data.ExpiresAt.Value())",
                "compareWithNow(some(input.NewExpiresAt.Value())",
            )],
        ),
        "TestSubjectRows",
    );
    assert_go_caught(
        "bytes",
        go_control(
            "bytes",
            &[
                ("\t\"strings\"\n", "\t\"strings\"\n\t\"time\"\n"),
                (
                    "\tvalueSeconds, valueNanos, valueOk := instant(*value)\n\tnowSeconds, nowNanos, nowOk := instant(now)\n\tif !valueOk || !nowOk {\n\t\treturn unknown\n\t}\n\tnowSeconds += seconds\n\torder := 0\n\tswitch {\n\tcase valueSeconds < nowSeconds, valueSeconds == nowSeconds && valueNanos < nowNanos:\n\t\torder = -1\n\tcase valueSeconds > nowSeconds, valueSeconds == nowSeconds && valueNanos > nowNanos:\n\t\torder = 1\n\t}\n\treturn known(accepts(order))",
                    "\tparsed, err := time.Parse(time.RFC3339Nano, now)\n\tif err != nil {\n\t\treturn unknown\n\t}\n\treturn known(accepts(strings.Compare(*value, parsed.Add(time.Duration(seconds)*time.Second).UTC().Format(time.RFC3339Nano))))",
                ),
            ],
        ),
        "TestSubjectRows",
    );
}

/// [`go_control`], with `helper` placed in the behaviour beside the patched seams.
fn go_control_with(case: &str, faults: &[(&str, &str)], helper: &str) -> (String, bool) {
    let marker = "// clockUnmet is the typed refusal";
    let placed = format!("{helper}\n{marker}");
    let mut faults: Vec<(&str, &str)> = faults.to_vec();
    faults.push((marker, &placed));
    go_control(case, &faults)
}
