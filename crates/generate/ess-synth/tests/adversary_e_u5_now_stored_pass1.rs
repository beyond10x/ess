//! Adversary, unit E-U5 (`now` over stored and related rows), pass 1.
//!
//! `docs/design/expression-family-source22.md`, "One observed decision instant per command
//! occurrence": "Missing capability yields a named unsupported result when the decision needs a
//! clock-dependent leaf, before any effect; earlier definitive refusals remain executable." and "The
//! network entry supplies the host UTC clock" (unit claim).
//!
//! - `fallible_clock_*`: a host context that answers the command clock as a named unavailable
//!   answer (the `TryContext::try_command_clock` / `FallibleContext.TryCommandClock` error the
//!   generated trait documents) must still let an input refusal, an unknown identity and a missing
//!   related row answer, exactly as the `None` / `false` clock does.
//! - `served_*`: the generated network entry of a model whose `when_subject:` orders a stored
//!   instant against `now` builds, and decides by the host's clock at the moment of the request.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_synth::{synthesize_for, Target};
use std::io::{BufRead as _, Read as _, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

const LEASES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/now-stored-rows.yaml");
const NOTES: &str = include_str!("fixtures/served-notes/system.yaml");

fn model(text: &str) -> EssIr {
    let spec = Specification::assemble([(
        Source::new("adversary.yaml"),
        RawSpecFile::parse(text).unwrap(),
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn emit(text: &str, target: Target, case: &str) -> PathBuf {
    let synthesis = synthesize_for(&model(text), target).unwrap();
    let directory = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-e-u5-{}-{case}-{}",
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

const RUST_FALLIBLE: &str = r#"
use demo_types::behaviour::{BookStorage, Generated, LeaseStorage, MemberStorage, TryContext};
use demo_types::leases::{self, obligations::*};
use demo_types::obligation::UnmetObligation;
use demo_types::primitives::{Timestamp, Uuid};

const FUTURE: &str = "2090-01-01T00:00:00Z";
const PAST: &str = "1990-01-01T00:00:00Z";

#[derive(Default)]
struct Ports {
    leases: Vec<leases::LeaseSnapshot>,
    members: Vec<leases::MemberSnapshot>,
    books: Vec<leases::BookSnapshot>,
    minted: u32,
}

impl Ports {
    fn uuid(&mut self) -> Uuid {
        self.minted += 1;
        Uuid(format!("00000000-0000-4000-8000-{:012}", self.minted))
    }
}

/// A host whose command clock is unavailable, named as the generated trait documents.
impl TryContext for Ports {
    fn try_generate_demo_leases_book_id(&mut self) -> Result<leases::BookId, UnmetObligation> {
        Ok(leases::BookId(self.uuid()))
    }
    fn try_generate_demo_leases_lease_id(&mut self) -> Result<leases::LeaseId, UnmetObligation> {
        Ok(leases::LeaseId(self.uuid()))
    }
    fn try_generate_demo_leases_member_id(&mut self) -> Result<leases::MemberId, UnmetObligation> {
        Ok(leases::MemberId(self.uuid()))
    }
    fn try_command_clock(&mut self) -> Result<Option<Timestamp>, UnmetObligation> {
        Err(UnmetObligation { capability: "command clock", source: "host" })
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

#[test]
fn an_unavailable_clock_leaves_earlier_refusals_answered() {
    let mut service = Generated::new(Ports::default());
    let lease = match service
        .open_lease(leases::OpenLease {
            expires_at: Timestamp(FUTURE.to_owned()),
            grace_until: Timestamp(FUTURE.to_owned()),
        })
        .unwrap()
    {
        leases::OpenLeaseOutcome::Opened { lease_opened } => lease_opened.lease_id,
    };
    let _ = service
        .register_member(leases::RegisterMember { banned_until: Timestamp(PAST.to_owned()) })
        .unwrap();
    // The input refusal is decided before any row is read and before any `now` leaf.
    let blank = service.renew_lease(leases::RenewLease {
        lease_id: lease.clone(),
        new_expires_at: Timestamp(FUTURE.to_owned()),
        note: String::new(),
    });
    assert!(
        matches!(blank, Ok(leases::RenewLeaseOutcome::BlankNote { .. })),
        "blank-note: {blank:?}"
    );
    // An unknown identity answers before the stored row's `now` leaf.
    let nobody = leases::LeaseId(Uuid("00000000-0000-4000-8000-00000000dead".to_owned()));
    let unknown = service.renew_lease(leases::RenewLease {
        lease_id: nobody,
        new_expires_at: Timestamp(FUTURE.to_owned()),
        note: "renewal".to_owned(),
    });
    assert!(
        matches!(unknown, Ok(leases::RenewLeaseOutcome::UnknownLease { .. })),
        "unknown-lease: {unknown:?}"
    );
    // A missing related row answers before the related row's `now` leaf.
    let stranger = leases::MemberId(Uuid("00000000-0000-4000-8000-00000000dead".to_owned()));
    let joined = service.join(leases::Join { member_id: stranger });
    assert!(
        matches!(joined, Ok(leases::JoinOutcome::NoMemberToJoin { .. })),
        "no-member-to-join: {joined:?}"
    );
}
"#;

#[test]
fn fallible_clock_rust_unavailable_answer_keeps_earlier_refusals() {
    let directory = emit(LEASES, Target::Rust, "fallible");
    let tests = directory.join("crates/demo-types/tests");
    std::fs::create_dir_all(&tests).unwrap();
    std::fs::write(tests.join("fallible.rs"), RUST_FALLIBLE).unwrap();
    let (log, passed) = run(
        &directory,
        env!("CARGO"),
        &[
            "test",
            "--offline",
            "-p",
            "demo-types",
            "--test",
            "fallible",
        ],
    );
    let _ = std::fs::remove_dir_all(&directory);
    assert!(
        log.contains("test an_unavailable_clock_leaves_earlier_refusals_answered"),
        "the control compiled and ran:\n{log}"
    );
    assert!(passed, "{log}");
}

const GO_FALLIBLE: &str = r#"
package behaviour

import (
	"fmt"
	"testing"

	"example.invalid/demo/types/leases"
	"example.invalid/demo/types/obligation"
	"example.invalid/demo/types/primitives"
)

type adversaryRows struct {
	leases  []leases.LeaseSnapshot
	members []leases.MemberSnapshot
	books   []leases.BookSnapshot
	minted  int
}

// adversaryHost answers every assigned identity and names its command clock unavailable.
type adversaryHost struct{ c *adversaryRows }

func (h adversaryHost) uuid() primitives.Uuid {
	h.c.minted++
	return primitives.NewUuid(fmt.Sprintf("00000000-0000-4000-8000-%012d", h.c.minted))
}
func (h adversaryHost) TryGenerateDemoLeasesBookId() (leases.BookId, *obligation.UnmetObligation) {
	return leases.NewBookId(h.uuid()), nil
}
func (h adversaryHost) TryGenerateDemoLeasesLeaseId() (leases.LeaseId, *obligation.UnmetObligation) {
	return leases.NewLeaseId(h.uuid()), nil
}
func (h adversaryHost) TryGenerateDemoLeasesMemberId() (leases.MemberId, *obligation.UnmetObligation) {
	return leases.NewMemberId(h.uuid()), nil
}
func (h adversaryHost) TryCommandClock() (primitives.Timestamp, bool, *obligation.UnmetObligation) {
	return primitives.Timestamp{}, false, &obligation.UnmetObligation{Capability: "command clock", Source: "host"}
}

type adversaryLeases struct{ c *adversaryRows }

func (s adversaryLeases) Get(identity leases.LeaseId) (leases.LeaseSnapshot, bool) {
	for _, row := range s.c.leases {
		if row.Data.LeaseId == identity {
			return row, true
		}
	}
	return leases.LeaseSnapshot{}, false
}
func (s adversaryLeases) Put(snapshot leases.LeaseSnapshot) {
	s.Delete(snapshot.Data.LeaseId)
	s.c.leases = append(s.c.leases, snapshot)
}
func (s adversaryLeases) Delete(identity leases.LeaseId) {
	kept := s.c.leases[:0]
	for _, row := range s.c.leases {
		if row.Data.LeaseId != identity {
			kept = append(kept, row)
		}
	}
	s.c.leases = kept
}
func (s adversaryLeases) List() []leases.LeaseSnapshot {
	return append([]leases.LeaseSnapshot{}, s.c.leases...)
}

type adversaryMembers struct{ c *adversaryRows }

func (s adversaryMembers) Get(identity leases.MemberId) (leases.MemberSnapshot, bool) {
	for _, row := range s.c.members {
		if row.Data.MemberId == identity {
			return row, true
		}
	}
	return leases.MemberSnapshot{}, false
}
func (s adversaryMembers) Put(snapshot leases.MemberSnapshot) {
	s.Delete(snapshot.Data.MemberId)
	s.c.members = append(s.c.members, snapshot)
}
func (s adversaryMembers) Delete(identity leases.MemberId) {
	kept := s.c.members[:0]
	for _, row := range s.c.members {
		if row.Data.MemberId != identity {
			kept = append(kept, row)
		}
	}
	s.c.members = kept
}
func (s adversaryMembers) List() []leases.MemberSnapshot {
	return append([]leases.MemberSnapshot{}, s.c.members...)
}

type adversaryBooks struct{ c *adversaryRows }

func (s adversaryBooks) Get(identity leases.BookId) (leases.BookSnapshot, bool) {
	for _, row := range s.c.books {
		if row.Data.BookId == identity {
			return row, true
		}
	}
	return leases.BookSnapshot{}, false
}
func (s adversaryBooks) Put(snapshot leases.BookSnapshot) {
	s.Delete(snapshot.Data.BookId)
	s.c.books = append(s.c.books, snapshot)
}
func (s adversaryBooks) Delete(identity leases.BookId) {
	kept := s.c.books[:0]
	for _, row := range s.c.books {
		if row.Data.BookId != identity {
			kept = append(kept, row)
		}
	}
	s.c.books = kept
}
func (s adversaryBooks) List() []leases.BookSnapshot {
	return append([]leases.BookSnapshot{}, s.c.books...)
}

func TestAdversaryUnavailableClockKeepsEarlierRefusals(t *testing.T) {
	c := &adversaryRows{}
	g := NewWithContext(Ports{LeaseStorage: adversaryLeases{c}, MemberStorage: adversaryMembers{c}, BookStorage: adversaryBooks{c}}, adversaryHost{c})
	opened, err := g.OpenLease(leases.OpenLease{ExpiresAt: primitives.NewTimestamp("2090-01-01T00:00:00Z"), GraceUntil: primitives.NewTimestamp("2090-01-01T00:00:00Z")})
	if err != nil {
		t.Fatal(err)
	}
	lease := opened.(leases.OpenLeaseOutcomeOpened).LeaseOpened.LeaseId
	blank, err := g.RenewLease(leases.RenewLease{LeaseId: lease, NewExpiresAt: primitives.NewTimestamp("2090-01-01T00:00:00Z"), Note: ""})
	if _, ok := blank.(leases.RenewLeaseOutcomeBlankNote); err != nil || !ok {
		t.Errorf("blank-note: %v %v", blank, err)
	}
	nobody := leases.NewLeaseId(primitives.NewUuid("00000000-0000-4000-8000-00000000dead"))
	unknown, err := g.RenewLease(leases.RenewLease{LeaseId: nobody, NewExpiresAt: primitives.NewTimestamp("2090-01-01T00:00:00Z"), Note: "renewal"})
	if _, ok := unknown.(leases.RenewLeaseOutcomeUnknownLease); err != nil || !ok {
		t.Errorf("unknown-lease: %v %v", unknown, err)
	}
	stranger := leases.NewMemberId(primitives.NewUuid("00000000-0000-4000-8000-00000000dead"))
	joined, err := g.Join(leases.Join{MemberId: stranger})
	if _, ok := joined.(leases.JoinOutcomeNoMemberToJoin); err != nil || !ok {
		t.Errorf("no-member-to-join: %v %v", joined, err)
	}
}
"#;

#[test]
fn fallible_clock_go_unavailable_answer_keeps_earlier_refusals() {
    let directory = emit(LEASES, Target::Go, "fallible");
    std::fs::write(
        directory.join("types/behaviour/adversary_fallible_test.go"),
        GO_FALLIBLE,
    )
    .unwrap();
    let (log, passed) = run(
        &directory,
        "go",
        &[
            "test",
            "-count=1",
            "-v",
            "-run",
            "TestAdversary",
            "./types/behaviour",
        ],
    );
    let _ = std::fs::remove_dir_all(&directory);
    assert!(
        log.contains("TestAdversaryUnavailableClockKeepsEarlierRefusals"),
        "the control compiled and ran:\n{log}"
    );
    assert!(passed, "{log}");
}

// ---- the network entry's host clock -----------------------------------------------------------

/// The served notes fixture at `ess/22`, each note carrying a stored `due_at`, archived only once
/// the decision's instant has reached it.
fn due_notes() -> String {
    [
        ("format: ess/18", "format: ess/22"),
        (
            "    fields:\n      - {name: text, type: String}\n    lifecycle:",
            "    fields:\n      - {name: text, type: String}\n      - {name: due_at, type: Timestamp}\n    lifecycle:",
        ),
        (
            "  - {name: notebook.notes.AlreadyArchived, summary: The note is already archived.}",
            "  - {name: notebook.notes.AlreadyArchived, summary: The note is already archived.}\n  - {name: notebook.notes.NotDue, summary: The note is not due yet.}",
        ),
        (
            "      - {name: text, type: String}\n    outcomes:",
            "      - {name: text, type: String}\n      - {name: due_at, type: Timestamp}\n    outcomes:",
        ),
        (
            "sets: {text: input.text}",
            "sets: {text: input.text, due_at: input.due_at}",
        ),
        (
            "      - name: archived\n        moves: notebook.notes.Note.archive",
            "      - name: archived\n        when_subject: {predicate: due_at <= now}\n        moves: notebook.notes.Note.archive",
        ),
        (
            "      - {name: unknown, unknown_instance: true, error: notebook.notes.NoSuchNote}",
            "      - {name: not-due, error: notebook.notes.NotDue}\n      - {name: unknown, unknown_instance: true, error: notebook.notes.NoSuchNote}",
        ),
    ]
    .iter()
    .fold(NOTES.to_owned(), |text, (before, after)| {
        assert!(text.contains(before), "{before}");
        text.replacen(before, after, 1)
    })
}

/// Builds the generated `notes-server` for `target`, or the build log.
fn served(target: Target) -> Result<PathBuf, String> {
    let root = emit(&due_notes(), target, "served");
    let name = "notes-server";
    match target {
        Target::Go => {
            let (log, ok) = run(
                &root,
                "go",
                &["build", "-o", name, &format!("./cmd/{name}")],
            );
            ok.then(|| root.join(name)).ok_or(log)
        }
        Target::Rust => {
            let target_dir = root.join("adversary-target");
            let target_arg = target_dir.display().to_string();
            let (log, ok) = run(
                &root,
                env!("CARGO"),
                &[
                    "build",
                    "--offline",
                    "--bin",
                    name,
                    "--target-dir",
                    &target_arg,
                ],
            );
            ok.then(|| target_dir.join(format!("debug/{name}")))
                .ok_or(log)
        }
        _ => unreachable!(),
    }
}

struct Server {
    child: std::process::Child,
    address: String,
}

impl Server {
    fn start(binary: &Path) -> Self {
        let mut child = Command::new(binary)
            .args(["--listen", "127.0.0.1:0", "--callers", "actor-header"])
            .stdout(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let stdout = child.stdout.take().unwrap();
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                if let Ok(value) = serde_json::from_str::<serde_json::Value>(&line) {
                    if value["event"] == "system.ready" {
                        let _ =
                            sender.send(value["runtime"]["address"].as_str().unwrap().to_owned());
                    }
                }
            }
        });
        let Ok(address) = receiver.recv_timeout(Duration::from_secs(10)) else {
            let _ = child.kill();
            let output = child.wait_with_output().unwrap();
            panic!(
                "server did not become ready: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        };
        Self { child, address }
    }

    fn json(&self, method: &str, path: &str, body: &serde_json::Value) -> (u16, serde_json::Value) {
        let body = body.to_string();
        let mut stream = std::net::TcpStream::connect(&self.address).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(5)))
            .unwrap();
        write!(stream, "{method} {path} HTTP/1.1\r\nHost: localhost\r\nContent-Type: application/json\r\nContent-Length: {}\r\nAuthorization: Actor Writer\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        let mut bytes = Vec::new();
        stream.read_to_end(&mut bytes).unwrap();
        let boundary = bytes
            .windows(4)
            .position(|bytes| bytes == b"\r\n\r\n")
            .unwrap();
        let head = std::str::from_utf8(&bytes[..boundary]).unwrap();
        let status = head.split_whitespace().nth(1).unwrap().parse().unwrap();
        let answer = serde_json::from_slice(&bytes[boundary + 4..]).unwrap_or(
            serde_json::Value::String(String::from_utf8_lossy(&bytes[boundary + 4..]).into_owned()),
        );
        (status, answer)
    }

    fn state(&self, note: u64) -> String {
        let (_, rows) = self.json("GET", "/notes/views/notes", &serde_json::Value::Null);
        rows["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["note_id"] == note)
            .map(|row| row["state"].as_str().unwrap().to_owned())
            .unwrap_or_default()
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn rfc3339(at: time::OffsetDateTime) -> String {
    at.format(&time::format_description::well_known::Rfc3339)
        .unwrap()
}

fn exercise_served(target: Target) {
    let root = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-e-u5-{}-served-{}",
        target.name(),
        std::process::id()
    ));
    let binary = served(target).unwrap_or_else(|log| panic!("{target:?} build: {log}"));
    let server = Server::start(&binary);
    let soon = time::OffsetDateTime::now_utc() + time::Duration::seconds(3);
    for (note, due) in [
        (1, "2000-01-01T00:00:00Z".to_owned()),
        (2, "2090-01-01T00:00:00Z".to_owned()),
        (3, rfc3339(soon)),
    ] {
        let (status, answer) = server.json(
            "POST",
            "/notes/commands/add-note",
            &serde_json::json!({"note_id": note, "text": "due", "due_at": due}),
        );
        assert_eq!(status, 202, "{target:?} add {note}: {answer}");
    }
    let archive = |note: u64| {
        server.json(
            "POST",
            "/notes/commands/archive-note",
            &serde_json::json!({"note_id": note}),
        )
    };
    let (status, answer) = archive(1);
    assert_eq!(
        (status, answer["outcome"].as_str()),
        (202, Some("archived")),
        "{target:?}: a note due in 2000 is archived by the host clock: {answer}"
    );
    assert_eq!(server.state(1), "Archived", "{target:?}");
    let (status, answer) = archive(2);
    assert_ne!(answer["outcome"].as_str(), Some("archived"), "{target:?}");
    assert!(
        status < 500,
        "{target:?}: a decided refusal, not an unmet clock: {status} {answer}"
    );
    assert_eq!(server.state(2), "Active", "{target:?}");
    // Read at each decision: not due now, due three seconds later.
    let (status, answer) = archive(3);
    assert!(status < 500, "{target:?}: {status} {answer}");
    assert_eq!(server.state(3), "Active", "{target:?}: {answer}");
    std::thread::sleep(Duration::from_secs(4));
    let (status, answer) = archive(3);
    assert_eq!(
        (status, answer["outcome"].as_str()),
        (202, Some("archived")),
        "{target:?}: the host clock is read at the later decision: {answer}"
    );
    drop(server);
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn served_go_entry_decides_stored_rows_by_the_host_clock() {
    exercise_served(Target::Go);
}

#[test]
fn served_rust_entry_decides_stored_rows_by_the_host_clock() {
    exercise_served(Target::Rust);
}
