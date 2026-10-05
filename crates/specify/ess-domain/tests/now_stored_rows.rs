//! The current-time operand in a predicate over a stored row (`docs/design/expression-family-source22.md`,
//! "A3: current time over stored and related rows"; beyond10x/ess#244 part a, unit U5).
//!
//! From `ess/22` a command outcome's `when_subject:` predicate and the predicate of an
//! identity-addressed `when_related:` may order a stored `Timestamp` against `now`, moved by a whole
//! number of seconds, minutes or hours: one decision reads its input guard and every row it reads
//! with the one instant it observed. Below `ess/22` a well-formed ordering there is refused naming
//! `ess/22`; every other use of the word keeps the answer it had. An invariant, a view filter and a
//! set-effect filter still refuse the operand, and an equality with it is refused at every site.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const LEASES: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/now-stored-rows.yaml");

/// One lease, renewed while its stored expiry is not more than `{guard}` past, and one member,
/// joined while the guard over the stored ban does not hold: a `when_subject:` and an
/// identity-addressed `when_related:` under `{format}`.
fn model(format: &str, subject: &str, related: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.leases
types:
  - {{name: demo.leases.LeaseId, kind: newtype, of: Uuid}}
  - {{name: demo.leases.MemberId, kind: newtype, of: Uuid}}
entities:
  - name: demo.leases.Member
    identity: {{name: member_id, type: demo.leases.MemberId}}
    fields:
      - {{name: banned_until, type: Timestamp}}
    lifecycle: {{initial: Registered, states: [Registered], terminal: [Registered]}}
  - name: demo.leases.Lease
    identity: {{name: lease_id, type: demo.leases.LeaseId}}
    fields:
      - {{name: expires_at, type: Timestamp}}
    lifecycle:
      initial: Active
      states: [Active, Renewed]
      terminal: [Renewed]
      transitions:
        - {{name: renew, from: [Active], to: Renewed}}
errors:
  - {{name: demo.leases.Lapsed, summary: Lapsed., fields: []}}
  - {{name: demo.leases.NotActive, summary: Not active., fields: []}}
  - {{name: demo.leases.UnknownLease, summary: Unknown., fields: []}}
  - {{name: demo.leases.NoMember, summary: No member., fields: []}}
  - {{name: demo.leases.Banned, summary: Banned., fields: []}}
events:
  - name: demo.leases.LeaseRenewed
    fields:
      - {{name: lease_id, type: demo.leases.LeaseId}}
  - name: demo.leases.Joined
    fields:
      - {{name: member_id, type: demo.leases.MemberId}}
commands:
  - name: demo.leases.RenewLease
    input:
      - {{name: lease_id, type: demo.leases.LeaseId}}
    outcomes:
      - name: renewed
        when_subject: {{predicate: {subject}}}
        moves: demo.leases.Lease.renew
        instance: lease_id
        emits: [demo.leases.LeaseRenewed]
        payload:
          demo.leases.LeaseRenewed: {{lease_id: input.lease_id}}
      - {{name: lapsed, error: demo.leases.Lapsed}}
      - {{name: not-active, wrong_state: true, error: demo.leases.NotActive}}
      - {{name: unknown-lease, unknown_instance: true, error: demo.leases.UnknownLease}}
  - name: demo.leases.Join
    input:
      - {{name: member_id, type: demo.leases.MemberId}}
    outcomes:
      - name: no-member
        when_related: {{via: input.member_id, exists: false}}
        error: demo.leases.NoMember
      - name: banned
        when_related: {{via: input.member_id, predicate: {related}}}
        error: demo.leases.Banned
      - name: joined
        emits: [demo.leases.Joined]
        payload:
          demo.leases.Joined: {{member_id: input.member_id}}
"
    )
}

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("leases.yaml"), raw)])
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

/// Every refusal of `text` with `code` whose message holds `needle`, as `location message` lines.
fn refusals(text: &str, code: ValidationCode, needle: &str) -> Vec<String> {
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
        "expected {code:?} containing {needle:?}:\n{}",
        listed(&errors)
    );
    found
}

#[test]
fn a3_now_orders_stored_rows_from_ess_22() {
    validates(LEASES);
    for (subject, related) in [
        ("expires_at >= now", "banned_until > now"),
        ("expires_at < now - 90s", "banned_until <= now + 15m"),
        (
            "{expires_at: {gt: now - 2h}}",
            "{banned_until: {ge: now - 1h}}",
        ),
        (
            "{all: [expires_at > now - 1h, expires_at < now + 24h]}",
            "{not: {banned_until: {lt: now}}}",
        ),
    ] {
        validates(&model("ess/22", subject, related));
    }
}

#[test]
fn a3_below_ess_22_a_stored_ordering_against_now_names_ess_22() {
    for format in ["ess/20", "ess/21"] {
        let text = model(format, "expires_at >= now - 1h", "banned_until > now + 30s");
        let found = refusals(&text, ValidationCode::UnsupportedFormatVersion, "ess/22");
        assert!(
            found.iter().any(|line| line.contains("RenewLease")
                && line.contains("expires_at")
                && line.contains("when_subject")),
            "{format}: the subject predicate names ess/22: {found:#?}"
        );
        assert!(
            found.iter().any(|line| line.contains("Join")
                && line.contains("banned_until")
                && line.contains("when_related")),
            "{format}: the related predicate names ess/22: {found:#?}"
        );
    }
    // A fixed instant at the same sites is unchanged below ess/22.
    validates(&model(
        "ess/21",
        "expires_at >= '2020-01-01T00:00:00Z'",
        "banned_until > '2020-01-01T00:00:00Z'",
    ));
}

#[test]
fn a3_below_ess_22_an_equality_with_now_keeps_its_refusal() {
    // An equality with the word is refused as it was before ess/22: the operand is not admitted
    // there at all, so the refusal names where it is.
    let text = model("ess/21", "expires_at == now", "banned_until > now + 30s");
    refusals(&text, ValidationCode::TypeMismatch, "`when:`");
}

#[test]
fn a3_now_is_ordered_against_never_equated_over_a_stored_row() {
    for (subject, related) in [
        ("expires_at == now", "banned_until > now"),
        ("expires_at >= now", "banned_until != now - 5m"),
    ] {
        let found = refusals(
            &model("ess/22", subject, related),
            ValidationCode::TypeMismatch,
            "never equated",
        );
        assert_eq!(found.len(), 1, "{found:#?}");
    }
}

#[test]
fn a3_a_malformed_offset_over_a_stored_row_is_refused_naming_the_spellings() {
    for (subject, related) in [
        ("expires_at >= now - 1d", "banned_until > now"),
        ("expires_at >= now", "banned_until > now - 060s"),
    ] {
        refusals(
            &model("ess/22", subject, related),
            ValidationCode::TypeMismatch,
            "`24h`",
        );
    }
}

#[test]
fn a3_now_stays_refused_outside_a_command_decision() {
    // An invariant is checked at rest, when no request is being handled.
    let invariant = LEASES.replacen(
        "      - {name: grace_until, type: Timestamp}\n",
        "      - {name: grace_until, type: Timestamp}\n    invariants: [grace_until > now]\n",
        1,
    );
    assert_ne!(invariant, LEASES);
    refusals(
        &invariant,
        ValidationCode::TypeMismatch,
        "set-effect filter",
    );
    // A view filter is read by a query.
    let filter = LEASES.replacen(
        "    source: demo.leases.Lease\n    consistency: read_your_writes\n",
        "    source: demo.leases.Lease\n    consistency: read_your_writes\n    filter: expires_at > now\n",
        1,
    );
    assert_ne!(filter, LEASES);
    refusals(&filter, ValidationCode::TypeMismatch, "`when_subject:`");
}

#[test]
fn a3_the_word_now_naming_a_stored_root_stays_a_fact() {
    // A field named `now` is read as itself from ess/22, at a stored site as at an input guard.
    let text = model("ess/22", "expires_at >= now", "banned_until > now").replacen(
        "      - {name: expires_at, type: Timestamp}\n",
        "      - {name: expires_at, type: Timestamp}\n      - {name: now, type: Timestamp}\n",
        1,
    );
    let specification = assemble(&text).unwrap_or_else(|errors| panic!("{}", listed(&errors)));
    let command = specification
        .commands()
        .get(&"demo.leases.RenewLease".parse().unwrap())
        .expect("declared");
    let guard = format!("{:?}", command.outcomes[0].condition);
    assert!(
        guard.contains("SubjectPredicate") && !guard.contains("\"now\")"),
        "the bare word reads the stored root `now`, not the current time: {guard}"
    );
}
