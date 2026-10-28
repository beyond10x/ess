//! `input.<field>` in a `when_subject` predicate (beyond10x/ess#157, E6) and `equals_ignore_case` /
//! `in_ignore_case` (beyond10x/ess#140, E7) at validation: what they admit under `ess/15`, the refusal
//! under every earlier header, the typing each is held to, and the finite proof declining both.
//! `docs/design/value-expressions.md` §§ E6, E7 is the binding design.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn admits(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(Source::new("guards.yaml"), raw)])
}

fn refusal(text: &str) -> ValidationErrors {
    match admits(text) {
        Ok(_) => panic!("must refuse:\n{text}"),
        Err(errors) => errors,
    }
}

fn codes(errors: &ValidationErrors) -> Vec<ValidationCode> {
    errors.as_slice().iter().map(|error| error.code).collect()
}

/// The #157 repro: a stop is ignored when it names a recording other than the call's current one.
///
/// `{guard}` is the `when_subject` predicate, `{default}` the guard of the sibling (empty for a
/// genuine default), and the entity and the input carry one field of every kind the typing tells
/// apart.
fn recording(format: &str, guard: &str, default: &str) -> String {
    format!(
        r#"format: {format}
system: calls
version: v1
domain: calls.rec
types:
  - name: calls.rec.RecordingId
    kind: newtype
    of: String
  - name: calls.rec.Mode
    kind: enum
    variants: [Audio, Video]
entities:
  - name: calls.rec.Call
    identity: {{name: call_id, type: Uuid}}
    fields:
      - {{name: recording_id, type: calls.rec.RecordingId}}
      - {{name: label, type: String}}
      - {{name: attempts, type: Integer}}
      - {{name: mode, type: calls.rec.Mode}}
    lifecycle:
      initial: Recording
      states: [Recording, Stopped]
      terminal: [Stopped]
      transitions:
        - {{name: stop, from: [Recording], to: Stopped}}
events:
  - name: calls.rec.Started
    fields: [{{name: call_id, type: Uuid}}]
  - name: calls.rec.Stopped
    fields: []
commands:
  - name: calls.rec.Start
    input:
      - {{name: recording_id, type: calls.rec.RecordingId}}
    outcomes:
      - name: started
        creates: calls.rec.Call
        instance: call_id
        sets: {{recording_id: input.recording_id, label: "x", attempts: 0, mode: Audio}}
        emits: [calls.rec.Started]
        payload:
          calls.rec.Started:
            call_id: {{generated: true}}
  - name: calls.rec.Stop
    input:
      - {{name: call_id, type: Uuid}}
      - {{name: recording_id, type: calls.rec.RecordingId}}
      - {{name: label, type: String}}
      - {{name: attempts, type: Integer}}
      - {{name: mode, type: calls.rec.Mode}}
      - {{name: maybe, type: Optional<String>}}
    outcomes:
      - name: other-recording
        when_subject:
          predicate: {guard}
        preserves: calls.rec.Call
        instance: call_id
      - name: stopped
        moves: calls.rec.Call.stop
        instance: call_id
        emits: [calls.rec.Stopped]{default}
"#
    )
}

const GUARD: &str = "recording_id != input.recording_id";

#[test]
fn the_157_repro_validates_under_ess_15() {
    admits(&recording("ess/15", GUARD, "")).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn every_comparison_operator_and_connective_reads_the_input() {
    for guard in [
        "recording_id == input.recording_id",
        "label == input.label",
        "attempts < input.attempts",
        "mode == input.mode",
        "{all: [label == x, recording_id != input.recording_id]}",
        "{not: {recording_id: {eq: input.recording_id}}}",
        "label == input.maybe",
    ] {
        admits(&recording("ess/15", guard, ""))
            .unwrap_or_else(|errors| panic!("{guard}: {errors}"));
    }
}

#[test]
fn every_earlier_header_refuses_an_input_operand_with_the_format_it_needs() {
    for format in ["ess/9", "ess/13", "ess/14"] {
        let errors = refusal(&recording(format, GUARD, ""));
        let text = errors.to_string();
        assert_eq!(
            codes(&errors),
            [ValidationCode::UnsupportedFormatVersion],
            "{format}: {text}"
        );
        assert!(text.contains("ess/15"), "{format}: {text}");
        assert!(text.contains("when_subject"), "{format}: {text}");
    }
}

#[test]
fn operands_of_types_that_do_not_compare_are_refused() {
    for guard in [
        "attempts == input.label",
        "label == input.attempts",
        "recording_id == input.attempts",
    ] {
        let errors = refusal(&recording("ess/15", guard, ""));
        assert!(
            codes(&errors).contains(&ValidationCode::TypeMismatch),
            "{guard}: {errors}"
        );
    }
}

#[test]
fn an_input_field_the_command_does_not_declare_is_refused_by_name() {
    let errors = refusal(&recording("ess/15", "recording_id != input.recording", ""));
    let text = errors.to_string();
    assert!(
        codes(&errors).contains(&ValidationCode::UnobservableFact)
            || codes(&errors).contains(&ValidationCode::UndeclaredReference),
        "{text}"
    );
    assert!(text.contains("input.recording"), "{text}");
}

#[test]
fn a_selector_past_an_input_scalar_is_refused() {
    let errors = refusal(&recording(
        "ess/15",
        "recording_id != input.recording_id.x",
        "",
    ));
    assert!(
        codes(&errors).contains(&ValidationCode::UnobservableFact),
        "{errors}"
    );
}

/// A bare word on the right is a literal, as everywhere: `input` alone is the text "input", and an
/// input on the left compares with the text a bare stored-field name spells (#74), not the field.
#[test]
fn a_bare_word_on_the_right_stays_a_literal() {
    admits(&recording("ess/15", "recording_id != input", ""))
        .unwrap_or_else(|errors| panic!("{errors}"));
    let errors = refusal(&recording(
        "ess/15",
        "input.recording_id != recording_id",
        "",
    ));
    assert!(
        codes(&errors).contains(&ValidationCode::TypeMismatch),
        "{errors}"
    );
}

/// The finite prover cannot enumerate an input's value against a stored one, so the guard is open
/// and the command needs a genuine default, as for any open guard.
#[test]
fn the_guard_is_open_so_a_guarded_sibling_needs_a_default() {
    let guarded_sibling = recording(
        "ess/15",
        GUARD,
        "\n        when_subject:\n          predicate: recording_id == input.recording_id",
    );
    let errors = refusal(&guarded_sibling);
    assert!(
        codes(&errors).contains(&ValidationCode::NonExhaustiveBranches),
        "{errors}"
    );
    // Even over an enum, which alone the prover would close.
    let enum_guard = recording(
        "ess/15",
        "mode == input.mode",
        "\n        when_subject:\n          predicate: mode != input.mode",
    );
    assert!(
        codes(&refusal(&enum_guard)).contains(&ValidationCode::NonExhaustiveBranches),
        "an enum comparison with the input is still open"
    );
}

/// A stored field named `input` keeps being read as that field: no document that validated before
/// changes meaning under the new header.
#[test]
fn a_stored_field_named_input_is_still_read_as_the_stored_field() {
    let text = recording("ess/15", "input.name == x", "")
        .replace(
            "      - {name: label, type: String}\n",
            "      - {name: label, type: String}\n      - {name: input, type: calls.rec.Party}\n",
        )
        .replace(
            "types:\n",
            "types:\n  - name: calls.rec.Party\n    kind: struct\n    fields:\n      - {name: name, type: String}\n",
        )
        .replace(
            "sets: {recording_id: input.recording_id, label: \"x\", attempts: 0, mode: Audio}",
            "sets: {recording_id: input.recording_id, label: \"x\", attempts: 0, mode: Audio, input: {name: n}}",
        );
    // `input.name` is the stored struct's member; the command declares no input `name`.
    admits(&text).unwrap_or_else(|errors| panic!("{errors}"));
}

// ---- E7 ------------------------------------------------------------------------------------------

/// The #140 repro, with `lower(source) == "web"` written as the operator it asked for.
fn lookup(format: &str, guard: &str) -> String {
    format!(
        r"format: {format}
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Source
    kind: newtype
    of: String
  - name: demo.orders.Channel
    kind: enum
    variants: [Web, Phone]
errors:
  - name: demo.orders.UnknownSource
    summary: The source is not a known source.
    fields: []
events:
  - name: demo.orders.Looked
    fields:
      - {{name: source, type: Optional<String>}}
commands:
  - name: demo.orders.Lookup
    input:
      - {{name: source, type: Optional<String>}}
      - {{name: typed, type: demo.orders.Source}}
      - {{name: channel, type: demo.orders.Channel}}
      - {{name: count, type: Integer}}
      - {{name: at, type: Timestamp}}
      - {{name: tags, type: List<String>}}
    outcomes:
      - name: unknown-source
        when: {guard}
        error: demo.orders.UnknownSource
      - name: found
        emits: [demo.orders.Looked]
        payload:
          demo.orders.Looked:
            source: input.source
"
    )
}

#[test]
fn the_140_repro_validates_under_ess_15() {
    for guard in [
        "{source: {equals_ignore_case: web}}",
        "{not: {source: {in_ignore_case: [web, phone]}}}",
        "{typed: {equals_ignore_case: WEB}}",
        "{typed: {in_ignore_case: [a]}}",
        "{exists: {in: tags, as: t, that: {t: {equals_ignore_case: vip}}}}",
        r#"{source: {equals_ignore_case: ""}}"#,
    ] {
        admits(&lookup("ess/15", guard)).unwrap_or_else(|errors| panic!("{guard}: {errors}"));
    }
}

#[test]
fn every_earlier_header_refuses_the_operators_with_the_format_they_need() {
    for format in ["ess/8", "ess/14"] {
        for guard in [
            "{source: {equals_ignore_case: web}}",
            "{source: {in_ignore_case: [web]}}",
        ] {
            let errors = refusal(&lookup(format, guard));
            let text = errors.to_string();
            assert_eq!(
                codes(&errors),
                [ValidationCode::UnsupportedFormatVersion],
                "{format} {guard}: {text}"
            );
            assert!(text.contains("ess/15"), "{text}");
        }
    }
}

#[test]
fn only_string_and_its_newtypes_are_folded() {
    for guard in [
        "{channel: {equals_ignore_case: web}}",
        "{count: {equals_ignore_case: web}}",
        "{at: {in_ignore_case: [web]}}",
        "{tags: {equals_ignore_case: web}}",
    ] {
        let errors = refusal(&lookup("ess/15", guard));
        assert!(
            codes(&errors).contains(&ValidationCode::TypeMismatch),
            "{guard}: {errors}"
        );
    }
}

#[test]
fn a_literal_that_is_not_text_is_refused() {
    for guard in [
        "{source: {equals_ignore_case: 44}}",
        "{source: {in_ignore_case: [web, true]}}",
    ] {
        let errors = refusal(&lookup("ess/15", guard));
        assert!(
            codes(&errors).contains(&ValidationCode::TypeMismatch),
            "{guard}: {errors}"
        );
    }
}

#[test]
fn an_empty_list_is_refused_because_it_matches_nothing() {
    let errors = refusal(&lookup("ess/15", "{source: {in_ignore_case: []}}"));
    assert!(
        codes(&errors).contains(&ValidationCode::EmptyDeclaration),
        "{errors}"
    );
}

#[test]
fn the_operators_are_gated_under_when_subject_too() {
    let text = recording("ess/14", "{label: {equals_ignore_case: x}}", "");
    let errors = refusal(&text);
    assert!(
        codes(&errors).contains(&ValidationCode::UnsupportedFormatVersion),
        "{errors}"
    );
    admits(&recording("ess/15", "{label: {equals_ignore_case: x}}", ""))
        .unwrap_or_else(|errors| panic!("{errors}"));
}

/// A fold is open to the finite prover, so it needs a default beside it like any open guard.
#[test]
fn a_fold_guard_is_open_to_the_finite_proof() {
    let text = lookup("ess/15", "{typed: {equals_ignore_case: web}}").replace(
        "      - name: found\n",
        "      - name: found\n        when: {not: {typed: {equals_ignore_case: web}}}\n",
    );
    let errors = refusal(&text);
    assert!(
        codes(&errors).contains(&ValidationCode::NonExhaustiveBranches),
        "{errors}"
    );
}

// ---- The identity in `when_subject` (declined, with the existence-step idiom) -------------------

/// A grant per parent, minted create-or-update: `first` creates the row the existence step found
/// absent, `already` updates the stored one. `{guard}` is the `when_subject` predicate written on
/// `already`; with `default`, an unguarded `kept` update answers every other stored row.
fn mint(guard: &str, default: bool) -> String {
    let kept = if default {
        "      - name: kept\n        updates: demo.grants.Grant\n        instance: parent_id\n        \
         emits: [demo.grants.Minted]\n        payload: {demo.grants.Minted: {parent_id: input.parent_id}}\n        \
         sets: {enabled: false}\n"
    } else {
        ""
    };
    format!(
        r"format: ess/32
system: demo
version: v1
domain: demo.grants
types:
  - {{name: demo.grants.ParentId, kind: newtype, of: Uuid}}
entities:
  - name: demo.grants.Grant
    identity: {{name: parent_id, type: demo.grants.ParentId}}
    fields:
      - {{name: enabled, type: Boolean}}
    lifecycle: {{initial: Live, states: [Live], terminal: [Live]}}
events:
  - name: demo.grants.Minted
    fields: [{{name: parent_id, type: demo.grants.ParentId}}]
commands:
  - name: demo.grants.Mint
    input: [{{name: parent_id, type: demo.grants.ParentId}}]
    outcomes:
      - name: first
        unknown_instance: true
        creates: demo.grants.Grant
        instance: parent_id
        emits: [demo.grants.Minted]
        payload: {{demo.grants.Minted: {{parent_id: input.parent_id}}}}
        sets: {{enabled: true}}
      - name: already
        when_subject:
          predicate: {guard}
        updates: demo.grants.Grant
        instance: parent_id
        emits: [demo.grants.Minted]
        payload: {{demo.grants.Minted: {{parent_id: input.parent_id}}}}
        sets: {{enabled: true}}
{kept}"
    )
}

/// The two spellings of "the addressed row is stored" the identity read stands for.
const IDENTITY_READS: [&str; 2] = ["parent_id == input.parent_id", "defined(parent_id)"];

/// What the hint names in place of the stored fields: the existence step.
const IDENTITY_HINT: &str = "`parent_id` is the identity that addresses this row, so on every row \
                             `when_subject:` reads it equals `input.parent_id` and is defined; \
                             answer an identity no row carries with an `unknown_instance: true` \
                             branch and drop `when_subject:` from the update";

#[test]
fn an_identity_read_in_when_subject_names_the_existence_step() {
    for guard in IDENTITY_READS {
        for default in [false, true] {
            let errors = refusal(&mint(guard, default));
            assert_eq!(
                codes(&errors),
                [ValidationCode::UnobservableFact],
                "{guard} (default: {default}): {errors}"
            );
            let refused = &errors.as_slice()[0];
            assert_eq!(
                refused.hint.as_deref(),
                Some(IDENTITY_HINT),
                "{guard} (default: {default}): {errors}"
            );
            assert!(
                refused
                    .message
                    .contains("`when_subject` reads the stored fields of `demo.grants.Grant`"),
                "the message is unchanged: {errors}"
            );
        }
    }
}

#[test]
fn a_refused_when_subject_predicate_draws_no_coverage_refusal() {
    for guard in IDENTITY_READS {
        let errors = refusal(&mint(guard, false));
        assert!(
            !codes(&errors).contains(&ValidationCode::NonExhaustiveBranches),
            "{guard}: the refused predicate is the one cause: {errors}"
        );
    }
}

#[test]
fn a_non_identity_unknown_root_keeps_the_stored_fields_hint() {
    let errors = refusal(&mint("missing == true", true));
    assert_eq!(
        codes(&errors),
        [ValidationCode::UnobservableFact],
        "{errors}"
    );
    let hint = errors.as_slice()[0].hint.as_deref().unwrap_or_default();
    assert!(hint.starts_with("stored fields: `enabled`;"), "{errors}");
    assert!(!hint.contains("unknown_instance"), "{errors}");
}

/// The idiom the hint names validates: the update unguarded beside the `unknown_instance:` creator.
#[test]
fn the_existence_step_idiom_validates() {
    let idiom =
        mint("true", false).replace("        when_subject:\n          predicate: true\n", "");
    admits(&idiom).unwrap_or_else(|errors| panic!("{errors}"));
}

// ---- `defined(x)` / `not defined(x)` partitions over one Optional path --------------------------

/// An account with an optional reset token, opened without one. `ResendReset` refuses an account no
/// row carries, then answers by `{branches}`, each a `when_subject:` update over the stored token.
fn account(branches: &str) -> String {
    format!(
        r"format: ess/32
system: demo
version: v1
domain: demo.accounts
types:
  - {{name: demo.accounts.AccountId, kind: newtype, of: Uuid}}
entities:
  - name: demo.accounts.Account
    identity: {{name: account_id, type: demo.accounts.AccountId}}
    fields:
      - {{name: reset_token, type: Optional<String>}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - {{name: demo.accounts.UnknownAccount, summary: No account carries the identity., fields: []}}
events:
  - name: demo.accounts.Opened
    fields: [{{name: account_id, type: demo.accounts.AccountId}}]
  - name: demo.accounts.ResetSent
    fields: [{{name: account_id, type: demo.accounts.AccountId}}]
commands:
  - name: demo.accounts.Open
    input: []
    outcomes:
      - name: opened
        creates: demo.accounts.Account
        instance: account_id
        emits: [demo.accounts.Opened]
        payload: {{demo.accounts.Opened: {{account_id: {{generated: true}}}}}}
  - name: demo.accounts.ResendReset
    input: [{{name: account_id, type: demo.accounts.AccountId}}]
    outcomes:
      - name: unknown
        unknown_instance: true
        error: demo.accounts.UnknownAccount
{branches}"
    )
}

/// One `when_subject:` update of the account named `name`, selected by `predicate`.
fn resend(name: &str, predicate: &str) -> String {
    format!(
        "      - name: {name}\n        when_subject: {{predicate: {predicate}}}\n        \
         updates: demo.accounts.Account\n        instance: account_id\n        \
         sets: {{reset_token: t}}\n        emits: [demo.accounts.ResetSent]\n        \
         payload: {{demo.accounts.ResetSent: {{account_id: input.account_id}}}}\n"
    )
}

/// The stored pair: `resent` where a token is stored, `issued` where none is.
fn stored_pair() -> String {
    account(&format!(
        "{}{}",
        resend("resent", "defined(reset_token)"),
        resend("issued", "not defined(reset_token)")
    ))
}

/// The input pair: a checkout answered by whether a coupon is sent.
const CHECKOUT: &str = "  - name: demo.orders.Checkout
    input: [{name: coupon, type: Optional<String>}]
    outcomes:
      - name: discounted
        when: defined(coupon)
        emits: [demo.orders.CheckedOut]
      - name: full-price
        when: not defined(coupon)
        emits: [demo.orders.CheckedOut]
";

fn checkout() -> String {
    format!(
        "format: ess/32
system: demo
version: v1
domain: demo.orders
events:
  - name: demo.orders.CheckedOut
    fields: []
commands:
{CHECKOUT}"
    )
}

#[test]
fn defined_and_not_defined_over_one_stored_optional_field_are_exhaustive() {
    admits(&stored_pair()).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn a_defined_pair_over_an_optional_input_is_exhaustive() {
    // At shape time, with no type registry: the coverage is deferred to the typed pass.
    // The list item, as a mapping of its own.
    let command = CHECKOUT
        .lines()
        .map(|line| {
            line.strip_prefix("  - ")
                .or_else(|| line.strip_prefix("    "))
                .unwrap_or(line)
        })
        .collect::<Vec<_>>()
        .join("\n");
    let raw: ess_domain::command::RawCommandSpec =
        ess_yaml::from_str(&command).unwrap_or_else(|error| panic!("{error}\n{command}"));
    ess_domain::command::CommandSpec::try_from(raw)
        .unwrap_or_else(|errors| panic!("shape time: {errors}"));
    // And typed, where the proof runs.
    admits(&checkout()).unwrap_or_else(|errors| panic!("typed: {errors}"));
}

/// A tier with an optional note, read through the seat's reference to it.
fn related(noted: &str, unnoted: &str) -> String {
    format!(
        r"format: ess/32
system: demo
version: v1
domain: demo.seats
types:
  - {{name: demo.seats.SeatId, kind: newtype, of: Uuid}}
  - {{name: demo.seats.TierId, kind: newtype, of: Uuid}}
entities:
  - name: demo.seats.Tier
    identity: {{name: tier_id, type: demo.seats.TierId}}
    fields:
      - {{name: note, type: Optional<String>}}
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
  - name: demo.seats.Seat
    identity: {{name: seat_id, type: demo.seats.SeatId}}
    fields:
      - {{name: tier_id, type: demo.seats.TierId}}
    relations:
      - {{name: tier, kind: references, target: demo.seats.Tier, cardinality: one, via: tier_id}}
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
errors:
  - {{name: demo.seats.UnknownSeat, summary: No seat carries the identity., fields: []}}
  - {{name: demo.seats.UnknownTier, summary: No tier carries the identity., fields: []}}
events:
  - name: demo.seats.TierDefined
    fields: [{{name: tier_id, type: demo.seats.TierId}}]
  - name: demo.seats.Issued
    fields: [{{name: seat_id, type: demo.seats.SeatId}}]
  - name: demo.seats.TierChanged
    fields: [{{name: seat_id, type: demo.seats.SeatId}}]
commands:
  - name: demo.seats.DefineTier
    input: [{{name: note, type: Optional<String>}}]
    outcomes:
      - name: defined
        creates: demo.seats.Tier
        instance: tier_id
        sets: {{note: input.note}}
        emits: [demo.seats.TierDefined]
        payload: {{demo.seats.TierDefined: {{tier_id: {{generated: true}}}}}}
  - name: demo.seats.Issue
    input: [{{name: tier_id, type: demo.seats.TierId}}]
    outcomes:
      - name: issued
        creates: demo.seats.Seat
        instance: seat_id
        sets: {{tier_id: input.tier_id}}
        emits: [demo.seats.Issued]
        payload: {{demo.seats.Issued: {{seat_id: {{generated: true}}}}}}
  - name: demo.seats.ChangeTier
    input:
      - {{name: seat_id, type: demo.seats.SeatId}}
      - {{name: tier_id, type: demo.seats.TierId}}
    outcomes:
      - name: unknown-seat
        unknown_instance: true
        error: demo.seats.UnknownSeat
      - name: unknown-tier
        when_related: {{via: input.tier_id, exists: false}}
        error: demo.seats.UnknownTier
      - name: noted
        when_related: {{via: input.tier_id, predicate: {noted}}}
        updates: demo.seats.Seat
        instance: seat_id
        sets: {{tier_id: input.tier_id}}
        emits: [demo.seats.TierChanged]
        payload: {{demo.seats.TierChanged: {{seat_id: input.seat_id}}}}
      - name: unnoted
        when_related: {{via: input.tier_id, predicate: {unnoted}}}
        updates: demo.seats.Seat
        instance: seat_id
        sets: {{tier_id: input.tier_id}}
        emits: [demo.seats.TierChanged]
        payload: {{demo.seats.TierChanged: {{seat_id: input.seat_id}}}}
"
    )
}

#[test]
fn a_defined_pair_over_a_related_optional_field_is_exhaustive() {
    admits(&related("defined(note)", "not defined(note)"))
        .unwrap_or_else(|errors| panic!("{errors}"));
}

/// The text the prover's Unsupported decline renders.
const UNSUPPORTED: &str =
    "a guard reads an optional or collection path, or a construct other than equality and membership";

#[test]
fn defined_beside_a_value_comparison_still_declines() {
    let errors = refusal(&account(&format!(
        "{}{}",
        resend("resent", "defined(reset_token)"),
        resend("issued", "reset_token == a")
    )));
    assert_eq!(
        codes(&errors),
        [ValidationCode::NonExhaustiveBranches],
        "{errors}"
    );
    assert!(errors.to_string().contains(UNSUPPORTED), "{errors}");
}

#[test]
fn defined_alone_without_a_default_names_the_absent_case() {
    let errors = refusal(&account(&resend("resent", "defined(reset_token)")));
    assert_eq!(
        codes(&errors),
        [ValidationCode::NonExhaustiveBranches],
        "{errors}"
    );
    let text = errors.to_string();
    assert!(
        text.contains("stored [reset_token = absent]") && text.contains("select 0 branches"),
        "{text}"
    );
}

/// Below the one precedence rule (`ess/27`), where two selected branches conflict rather than the
/// first declared answering.
#[test]
fn an_overlapping_defined_pair_names_the_present_case() {
    let errors = refusal(
        &account(&format!(
            "{}{}",
            resend("resent", "defined(reset_token)"),
            resend(
                "again",
                "{any: [defined(reset_token), not defined(reset_token)]}"
            )
        ))
        .replace("format: ess/32\n", "format: ess/26\n"),
    );
    let text = errors.to_string();
    assert!(
        codes(&errors).contains(&ValidationCode::ConflictingDeclaration)
            && text.contains("stored [reset_token = present]")
            && text.contains("select 2 branches"),
        "{text}"
    );
}

#[test]
fn a_default_bearing_command_keeps_its_enum_only_proof() {
    let text = account(&format!(
        "{}{}      - name: kept\n        updates: demo.accounts.Account\n        \
         instance: account_id\n        emits: [demo.accounts.ResetSent]\n        \
         payload: {{demo.accounts.ResetSent: {{account_id: input.account_id}}}}\n",
        resend("resent", "defined(reset_token)"),
        resend("again", "defined(reset_token)")
    ));
    admits(&text).unwrap_or_else(|errors| panic!("{errors}"));
}

/// A ticket closed from `Open`, refused from `Closed`, and refused first when a note is sent.
const HELD: &str = r"format: ess/32
system: demo
version: v1
domain: demo.tickets
types:
  - {name: demo.tickets.TicketId, kind: newtype, of: Uuid}
entities:
  - name: demo.tickets.Ticket
    identity: {name: ticket_id, type: demo.tickets.TicketId}
    fields: []
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - {name: demo.tickets.NoteRefused, summary: A note is not accepted on close., fields: []}
  - {name: demo.tickets.AlreadyClosed, summary: The ticket is closed., fields: []}
events:
  - name: demo.tickets.Opened
    fields: [{name: ticket_id, type: demo.tickets.TicketId}]
  - name: demo.tickets.Closed
    fields: [{name: ticket_id, type: demo.tickets.TicketId}]
commands:
  - name: demo.tickets.OpenTicket
    input: []
    outcomes:
      - name: opened
        creates: demo.tickets.Ticket
        instance: ticket_id
        emits: [demo.tickets.Opened]
        payload: {demo.tickets.Opened: {ticket_id: {generated: true}}}
  - name: demo.tickets.Close
    input:
      - {name: ticket_id, type: demo.tickets.TicketId}
      - {name: note, type: Optional<String>}
    outcomes:
      - name: noted
        when: defined(note)
        error: demo.tickets.NoteRefused
      - name: closed
        when_subject_state: Open
        moves: demo.tickets.Ticket.close
        instance: ticket_id
        emits: [demo.tickets.Closed]
        payload: {demo.tickets.Closed: {ticket_id: input.ticket_id}}
      - name: already-closed
        when_subject_state: Closed
        error: demo.tickets.AlreadyClosed
";

/// What 0.74.0 answers for [`HELD`]: the held-state proof drops the `defined` refusal it cannot
/// decide and proves the rest, so the command validates.
#[test]
fn a_held_state_partition_keeps_its_decline() {
    admits(HELD).unwrap_or_else(|errors| panic!("{errors}"));
}
