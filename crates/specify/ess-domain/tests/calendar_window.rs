//! Calendar-window guards at source (beyond10x/ess#244 part b, `docs/design/calendar-window-guards.md`).
//!
//! From `ess/22` a command guard may hold an instant to a weekly window at a fixed UTC offset:
//! `window: {at: now | <Timestamp fact>, days, from, to, offset}`. It is admitted where the
//! current time is — a command outcome's `when:`, its `when_subject:` predicate and an
//! identity-addressed `when_related:` predicate — and refused everywhere else, below `ess/22`, over
//! a value that is no `Timestamp`, and with a named time zone.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const RELEASES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/calendar-windows.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("releases.yaml"), raw)])
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
        .collect::<Vec<_>>()
        .join("\n")
}

fn validates(text: &str) {
    if let Err(errors) = assemble(text) {
        panic!("{}\n---\n{text}", listed(&errors));
    }
}

/// Every refusal of `text` with `code` whose message holds `needle`; at least one.
fn refused(text: &str, code: ValidationCode, needle: &str) -> Vec<String> {
    let errors = assemble(text).expect_err("refused");
    let found: Vec<String> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == code && error.message.contains(needle))
        .map(|error| format!("{} {}", error.location, error.message))
        .collect();
    assert_ne!(
        found.len(),
        0,
        "no {code:?} holding {needle:?}:\n{}",
        listed(&errors)
    );
    found
}

/// The fixture with `from` replaced by `to`, once.
fn edited(from: &str, to: &str) -> String {
    assert!(RELEASES.contains(from), "the fixture writes {from:?}");
    RELEASES.replacen(from, to, 1)
}

const SCHEDULE_WINDOW: &str = r#"window: {at: starts_at, days: [mon, tue, wed, thu], from: "08:00", to: "16:00", offset: "+01:00"}"#;

#[test]
fn window_the_fixture_validates_at_every_admitted_site() {
    validates(RELEASES);
    // The current time and a stored instant in an identity-addressed `when_related:` predicate.
    validates(DESKS);
}

/// A member's desk booked only on weekdays from 09:00 to 17:00 UTC, and only for a member who
/// joined on a Monday at +02:00: two windows over a related row.
const DESKS: &str = r#"format: ess/22
system: demo
version: v1
domain: demo.desks
types:
  - {name: demo.desks.MemberId, kind: newtype, of: Uuid}
entities:
  - name: demo.desks.Member
    identity: {name: member_id, type: demo.desks.MemberId}
    fields:
      - {name: joined_at, type: Timestamp}
    lifecycle: {initial: Registered, states: [Registered], terminal: [Registered]}
errors:
  - {name: demo.desks.NoMember, summary: No member., fields: []}
  - {name: demo.desks.Closed, summary: Closed., fields: []}
events:
  - name: demo.desks.Registered
    fields:
      - {name: member_id, type: demo.desks.MemberId}
  - name: demo.desks.Booked
    fields:
      - {name: member_id, type: demo.desks.MemberId}
commands:
  - name: demo.desks.Register
    input:
      - {name: joined_at, type: Timestamp}
    outcomes:
      - name: registered
        creates: demo.desks.Member
        instance: member_id
        sets: {joined_at: input.joined_at}
        emits: [demo.desks.Registered]
        payload:
          demo.desks.Registered: {member_id: {generated: true}}
  - name: demo.desks.Book
    input:
      - {name: member_id, type: demo.desks.MemberId}
    outcomes:
      - name: no-member
        when_related: {via: input.member_id, exists: false}
        error: demo.desks.NoMember
      - name: closed
        when_related:
          via: input.member_id
          predicate:
            any:
              - not: {window: {at: now, days: [mon, tue, wed, thu, fri], from: "09:00", to: "17:00", offset: Z}}
              - not: {window: {at: joined_at, days: [mon], from: "00:00", to: "24:00", offset: "+02:00"}}
        error: demo.desks.Closed
      - name: booked
        emits: [demo.desks.Booked]
        payload:
          demo.desks.Booked: {member_id: input.member_id}
"#;

#[test]
fn window_below_ess22_is_refused_naming_ess22() {
    let older = RELEASES.replacen("format: ess/22", "format: ess/21", 1);
    let error = RawSpecFile::parse(&older).expect_err("an ess/21 source has no window");
    let message = error.to_string();
    assert!(message.contains("ess/22"), "{message}");
    assert!(message.contains("window"), "{message}");

    // A window assembled under an older header without the reader's help is refused at assembly.
    let mut raw = RawSpecFile::parse(RELEASES).expect("parses");
    raw.format = Some("ess/21".parse().expect("a format"));
    let errors =
        Specification::assemble([(Source::new("releases.yaml"), raw)]).expect_err("refused");
    let found: Vec<_> = errors
        .as_slice()
        .iter()
        .filter(|error| {
            error.code == ValidationCode::UnsupportedFormatVersion
                && error.message.contains("calendar window")
                && error.message.contains("ess/22")
        })
        .collect();
    assert_ne!(found.len(), 0, "{}", listed(&errors));
}

#[test]
fn window_outside_a_command_guard_is_refused_naming_where_it_is_admitted() {
    // An entity invariant.
    let invariant = edited(
        "      - {name: ready_at, type: Timestamp}\n    lifecycle:",
        "      - {name: ready_at, type: Timestamp}\n    invariants:\n      - window: {at: ready_at, days: [mon], from: \"08:00\", to: \"16:00\", offset: Z}\n    lifecycle:",
    );
    refused(
        &invariant,
        ValidationCode::TypeMismatch,
        "command outcome's guard",
    );
    // A view filter.
    let filter = edited(
        "    consistency: read_your_writes\n    fields:",
        "    consistency: read_your_writes\n    filter:\n      window: {at: ready_at, days: [mon], from: \"08:00\", to: \"16:00\", offset: Z}\n    fields:",
    );
    refused(
        &filter,
        ValidationCode::TypeMismatch,
        "command outcome's guard",
    );
}

#[test]
fn window_at_must_read_a_timestamp() {
    let integer = edited(
        "  - name: demo.releases.Schedule\n    input:\n      - {name: starts_at, type: Timestamp}",
        "  - name: demo.releases.Schedule\n    input:\n      - {name: starts_at, type: Integer}",
    );
    refused(
        &integer,
        ValidationCode::TypeMismatch,
        "places `starts_at` in a calendar window, and `starts_at` is `Integer`, not a Timestamp",
    );
    // A window reads its fact like any guard does: one the command does not declare is refused as
    // unobservable at the guard.
    let missing = edited(
        SCHEDULE_WINDOW,
        &SCHEDULE_WINDOW.replace("starts_at", "begins_at"),
    );
    refused(
        &missing,
        ValidationCode::UnobservableFact,
        "reads `begins_at`, which `demo.releases.Schedule` does not declare as input",
    );
}

#[test]
fn window_at_now_beside_a_field_named_now_is_refused() {
    let shadowed = edited(
        "  - name: demo.releases.Deploy\n    input:\n      - {name: release_id, type: demo.releases.ReleaseId}",
        "  - name: demo.releases.Deploy\n    input:\n      - {name: release_id, type: demo.releases.ReleaseId}\n      - {name: now, type: Timestamp}",
    );
    refused(&shadowed, ValidationCode::TypeMismatch, "`now`");
}

#[test]
fn window_a_named_zone_is_refused_at_source() {
    let zoned = edited(
        SCHEDULE_WINDOW,
        &SCHEDULE_WINDOW.replace("\"+01:00\"", "Europe/Berlin"),
    );
    let error = RawSpecFile::parse(&zoned).expect_err("a zone name is refused");
    let message = error.to_string();
    assert!(
        message.contains("`Europe/Berlin` names a time zone"),
        "{message}"
    );
    assert!(message.contains("fixed offset"), "{message}");
}
