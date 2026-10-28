//! Adversary pass on the presence partition (`defined(x)` / `not defined(x)` proved exhaustive
//! without a default) and on the identity hint of a `when_subject` predicate.
//!
//! Each case states what a sound proof answers; a red one is a finding.

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn admits(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(Source::new("adversary.yaml"), raw)])
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

// ---- Input partitions -----------------------------------------------------------------------

/// `demo.orders.Checkout` with `input` (list items, indented six spaces) and `outcomes` (each an
/// emitting branch, indented six spaces).
fn checkout(input: &str, outcomes: &str) -> String {
    format!(
        "format: ess/32
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.Kind
    kind: enum
    variants: [Gift, Promo]
  - name: demo.orders.Address
    kind: struct
    fields:
      - {{name: line, type: String}}
      - {{name: urgent, type: Boolean}}
      - {{name: note, type: Optional<String>}}
events:
  - name: demo.orders.CheckedOut
    fields: []
commands:
  - name: demo.orders.Checkout
    input:
{input}    outcomes:
{outcomes}"
    )
}

/// One emitting branch named `name`, selected by the input guard `guard`.
fn branch(name: &str, guard: &str) -> String {
    format!(
        "      - name: {name}\n        when: {guard}\n        emits: [demo.orders.CheckedOut]\n"
    )
}

fn branches(list: &[(&str, &str)]) -> String {
    list.iter()
        .map(|(name, guard)| branch(name, guard))
        .collect()
}

/// The story's rule: a path whose resolution is Optional and that every guard reads only through
/// `defined(…)` has the domain {absent, present}. `ship.note` is such a path: `ship` is a required
/// struct input, `note` its Optional member. The Boolean control over `ship.urgent` shows the
/// struct member read itself is admitted.
#[test]
fn a_pair_over_an_optional_member_of_a_required_struct_input_is_exhaustive() {
    let input = "      - {name: ship, type: demo.orders.Address}\n";
    admits(&checkout(
        input,
        &branches(&[
            ("rush", "ship.urgent == true"),
            ("plain", "ship.urgent == false"),
        ]),
    ))
    .unwrap_or_else(|errors| panic!("control, a Boolean member: {errors}"));
    admits(&checkout(
        input,
        &branches(&[
            ("noted", "defined(ship.note)"),
            ("unnoted", "not defined(ship.note)"),
        ]),
    ))
    .unwrap_or_else(|errors| panic!("an Optional member read only through defined: {errors}"));
}

#[test]
fn two_optional_inputs_one_per_branch_leave_a_case_unanswered() {
    let errors = refusal(&checkout(
        "      - {name: coupon, type: Optional<String>}\n      - {name: voucher, type: Optional<String>}\n",
        &branches(&[
            ("discounted", "defined(coupon)"),
            ("full-price", "not defined(voucher)"),
        ]),
    ));
    assert!(
        codes(&errors).contains(&ValidationCode::NonExhaustiveBranches)
            && errors
                .to_string()
                .contains("coupon = absent, voucher = present"),
        "{errors}"
    );
}

const COUPON_AND_KIND: &str = "      - {name: coupon, type: Optional<String>}\n      - {name: kind, type: demo.orders.Kind}\n";

#[test]
fn presence_beside_a_sibling_value_names_the_uncovered_present_case() {
    let errors = refusal(&checkout(
        COUPON_AND_KIND,
        &branches(&[
            ("gift", "{all: [defined(coupon), kind == Gift]}"),
            ("full-price", "not defined(coupon)"),
        ]),
    ));
    assert!(
        codes(&errors) == [ValidationCode::NonExhaustiveBranches]
            && errors
                .to_string()
                .contains("coupon = present, kind = Promo"),
        "{errors}"
    );
}

#[test]
fn the_complete_presence_by_value_grid_validates() {
    admits(&checkout(
        COUPON_AND_KIND,
        &branches(&[
            ("gift", "{all: [defined(coupon), kind == Gift]}"),
            ("promo", "{all: [defined(coupon), kind == Promo]}"),
            ("full-price", "not defined(coupon)"),
        ]),
    ))
    .unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn a_third_branch_shadowed_by_the_pair_is_an_overlap() {
    let errors = refusal(&checkout(
        COUPON_AND_KIND,
        &branches(&[
            ("discounted", "defined(coupon)"),
            ("full-price", "not defined(coupon)"),
            ("gift", "{all: [defined(coupon), kind == Gift]}"),
        ]),
    ));
    assert!(
        codes(&errors).contains(&ValidationCode::ConflictingDeclaration)
            && errors.to_string().contains("coupon = present, kind = Gift"),
        "{errors}"
    );
}

/// `defined(addr.note)` true implies `defined(addr)`; the request with an address and no note
/// selects neither branch.
#[test]
fn defined_child_against_not_defined_parent_leaves_the_present_parent_unanswered() {
    let errors = refusal(&checkout(
        "      - {name: addr, type: Optional<demo.orders.Address>}\n",
        &branches(&[
            ("noted", "defined(addr.note)"),
            ("no-address", "not defined(addr)"),
        ]),
    ));
    assert!(
        codes(&errors).contains(&ValidationCode::NonExhaustiveBranches),
        "{errors}"
    );
}

#[test]
fn a_pair_under_any_all_and_not_is_exhaustive() {
    let input = "      - {name: coupon, type: Optional<String>}\n      - {name: voucher, type: Optional<String>}\n";
    admits(&checkout(
        input,
        &branches(&[
            ("discounted", "{any: [defined(coupon), defined(voucher)]}"),
            (
                "full-price",
                "{not: {any: [defined(coupon), defined(voucher)]}}",
            ),
        ]),
    ))
    .unwrap_or_else(|errors| panic!("{errors}"));
    let errors = refusal(&checkout(
        input,
        &branches(&[
            ("discounted", "{all: [defined(coupon), defined(voucher)]}"),
            (
                "full-price",
                "{all: [not defined(coupon), not defined(voucher)]}",
            ),
        ]),
    ));
    assert!(
        codes(&errors).contains(&ValidationCode::NonExhaustiveBranches),
        "one of two present selects nothing: {errors}"
    );
}

// ---- Stored fields --------------------------------------------------------------------------

/// An account with two optional tokens; `ResendReset` takes an optional coupon and answers by
/// `branches`, each a `when_subject:` update with an optional `when:`.
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
      - {{name: other_token, type: Optional<String>}}
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
    input:
      - {{name: account_id, type: demo.accounts.AccountId}}
      - {{name: coupon, type: Optional<String>}}
    outcomes:
      - name: unknown
        unknown_instance: true
        error: demo.accounts.UnknownAccount
{branches}"
    )
}

fn resend(name: &str, stored: &str, input: Option<&str>) -> String {
    let when = input.map_or(String::new(), |guard| format!("        when: {guard}\n"));
    format!(
        "      - name: {name}\n        when_subject: {{predicate: {stored}}}\n{when}        \
         updates: demo.accounts.Account\n        instance: account_id\n        \
         sets: {{reset_token: t}}\n        emits: [demo.accounts.ResetSent]\n        \
         payload: {{demo.accounts.ResetSent: {{account_id: input.account_id}}}}\n"
    )
}

#[test]
fn two_stored_optional_fields_one_per_branch_leave_a_row_unanswered() {
    let errors = refusal(&account(&format!(
        "{}{}",
        resend("resent", "defined(reset_token)", None),
        resend("issued", "not defined(other_token)", None)
    )));
    assert!(
        codes(&errors).contains(&ValidationCode::NonExhaustiveBranches)
            && errors
                .to_string()
                .contains("other_token = present, reset_token = absent"),
        "{errors}"
    );
}

#[test]
fn a_stored_pair_crossed_with_an_input_pair_is_exhaustive_and_names_its_gap() {
    let full = account(&format!(
        "{}{}{}",
        resend("resent", "defined(reset_token)", None),
        resend(
            "issued-coupon",
            "not defined(reset_token)",
            Some("defined(coupon)")
        ),
        resend(
            "issued",
            "not defined(reset_token)",
            Some("not defined(coupon)")
        )
    ));
    admits(&full).unwrap_or_else(|errors| panic!("{errors}"));
    let errors = refusal(&account(&format!(
        "{}{}",
        resend("resent", "defined(reset_token)", None),
        resend(
            "issued-coupon",
            "not defined(reset_token)",
            Some("defined(coupon)")
        )
    )));
    assert!(
        errors.to_string().contains(
            "stored [reset_token = absent] and input [coupon = absent] select 0 branches"
        ),
        "{errors}"
    );
}

/// The hint names the input field that addresses the row. Here `instance: target` addresses it,
/// so `input.account_id` names no input at all.
#[test]
fn the_identity_hint_names_the_input_that_addresses_the_row() {
    let text = r"format: ess/32
system: demo
version: v1
domain: demo.accounts
types:
  - {name: demo.accounts.AccountId, kind: newtype, of: Uuid}
entities:
  - name: demo.accounts.Account
    identity: {name: account_id, type: demo.accounts.AccountId}
    fields:
      - {name: enabled, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.accounts.Touched
    fields: [{name: target, type: demo.accounts.AccountId}]
commands:
  - name: demo.accounts.Touch
    input: [{name: target, type: demo.accounts.AccountId}]
    outcomes:
      - name: first
        unknown_instance: true
        creates: demo.accounts.Account
        instance: target
        emits: [demo.accounts.Touched]
        payload: {demo.accounts.Touched: {target: input.target}}
        sets: {enabled: true}
      - name: again
        when_subject: {predicate: defined(account_id)}
        updates: demo.accounts.Account
        instance: target
        emits: [demo.accounts.Touched]
        payload: {demo.accounts.Touched: {target: input.target}}
        sets: {enabled: true}
";
    let errors = refusal(text);
    let identity: Vec<_> = errors
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::UnobservableFact)
        .collect();
    assert_eq!(identity.len(), 1, "{errors}");
    let hint = identity[0].hint.as_deref().unwrap_or_default();
    assert!(
        hint.contains("`input.target`") && !hint.contains("`input.account_id`"),
        "the hint names an input the command does not declare: {hint}"
    );
}

// ---- Related rows ---------------------------------------------------------------------------

/// A seat moved to a tier with an optional note; `via_type` is the type of the input naming the
/// tier, `first` branches of `ChangeTier` declared first and `extra` ones declared last.
fn seats(via_type: &str, missing: bool, first: &str, extra: &str) -> String {
    let missing = if missing {
        "      - name: unknown-tier\n        when_related: {via: input.tier_id, exists: false}\n        error: demo.seats.UnknownTier\n"
    } else {
        ""
    };
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
      - {{name: tier_id, type: Optional<demo.seats.TierId>}}
    relations:
      - {{name: tier, kind: references, target: demo.seats.Tier, cardinality: one, via: tier_id}}
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
errors:
  - {{name: demo.seats.UnknownSeat, summary: No seat carries the identity., fields: []}}
  - {{name: demo.seats.UnknownTier, summary: No tier carries the identity., fields: []}}
  - {{name: demo.seats.NoTier, summary: No tier is named., fields: []}}
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
    input: [{{name: tier_id, type: Optional<demo.seats.TierId>}}]
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
      - {{name: tier_id, type: {via_type}}}
    outcomes:
{first}      - name: unknown-seat
        unknown_instance: true
        error: demo.seats.UnknownSeat
{missing}      - name: noted
        when_related: {{via: input.tier_id, predicate: defined(note)}}
        updates: demo.seats.Seat
        instance: seat_id
        sets: {{tier_id: input.tier_id}}
        emits: [demo.seats.TierChanged]
        payload: {{demo.seats.TierChanged: {{seat_id: input.seat_id}}}}
      - name: unnoted
        when_related: {{via: input.tier_id, predicate: not defined(note)}}
        updates: demo.seats.Seat
        instance: seat_id
        sets: {{tier_id: input.tier_id}}
        emits: [demo.seats.TierChanged]
        payload: {{demo.seats.TierChanged: {{seat_id: input.seat_id}}}}
{extra}"
    )
}

/// No `exists: false` branch: a tier id no row carries selects neither `when_related` branch.
#[test]
fn a_related_pair_without_a_missing_row_branch_is_refused() {
    let errors = refusal(&seats("demo.seats.TierId", false, "", ""));
    assert!(
        codes(&errors).contains(&ValidationCode::NonExhaustiveBranches),
        "{errors}"
    );
}

/// An Optional reference: `no-tier` answers the request that names no tier, the pair answers a
/// named, stored tier, `unknown-tier` one no row carries. Every request has one answer.
#[test]
fn an_absent_optional_reference_answered_by_not_defined_validates() {
    let no_tier = "      - name: no-tier\n        when: not defined(tier_id)\n        error: demo.seats.NoTier\n";
    admits(&seats("Optional<demo.seats.TierId>", true, no_tier, ""))
        .unwrap_or_else(|errors| panic!("{errors}"));
}

// ---- A third stored branch the pair shadows ------------------------------------------------------

/// An account with an optional token and a required plan; `Resend` answers by `first`, then
/// `special`, then `last`, each a `when_subject:` update. Under the one precedence rule the first
/// declared of two selected branches answers.
fn plans(first: &str, special: &str, last: &str) -> String {
    let branch = |name: &str, guard: &str| {
        format!(
            "      - name: {name}\n        when_subject: {{predicate: {guard}}}\n        \
             updates: demo.accounts.Account\n        instance: account_id\n        \
             sets: {{reset_token: t}}\n        emits: [demo.accounts.Sent]\n        \
             payload: {{demo.accounts.Sent: {{account_id: input.account_id}}}}\n"
        )
    };
    format!(
        r"format: ess/32
system: demo
version: v1
domain: demo.accounts
types:
  - {{name: demo.accounts.AccountId, kind: newtype, of: Uuid}}
  - {{name: demo.accounts.Plan, kind: enum, variants: [Basic, Pro]}}
entities:
  - name: demo.accounts.Account
    identity: {{name: account_id, type: demo.accounts.AccountId}}
    fields:
      - {{name: reset_token, type: Optional<String>}}
      - {{name: flag, type: Boolean}}
      - {{name: plan, type: demo.accounts.Plan}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: demo.accounts.Opened
    fields: [{{name: account_id, type: demo.accounts.AccountId}}]
  - name: demo.accounts.Sent
    fields: [{{name: account_id, type: demo.accounts.AccountId}}]
commands:
  - name: demo.accounts.Open
    input: [{{name: plan, type: demo.accounts.Plan}}]
    outcomes:
      - name: opened
        creates: demo.accounts.Account
        instance: account_id
        sets: {{plan: input.plan, flag: false}}
        emits: [demo.accounts.Opened]
        payload: {{demo.accounts.Opened: {{account_id: {{generated: true}}}}}}
  - name: demo.accounts.Resend
    input: [{{name: account_id, type: demo.accounts.AccountId}}]
    outcomes:
{}{}{}",
        branch("first", first),
        branch("special", special),
        branch("last", last)
    )
}

/// `special` is selected only where `first` is too, and `first` is declared above it: no stored
/// row is ever answered by `special`. The Boolean twin is the control: whatever it answers, the
/// presence partition must answer the same. (Both validate under the one rule; that is not this
/// change's.)
#[test]
fn a_stored_branch_the_pair_shadows_answers_as_its_boolean_twin() {
    let boolean = admits(&plans(
        "flag == true",
        "{all: [flag == true, plan == Pro]}",
        "flag == false",
    ))
    .err()
    .map(|errors| codes(&errors));
    let presence = admits(&plans(
        "defined(reset_token)",
        "{all: [defined(reset_token), plan == Pro]}",
        "not defined(reset_token)",
    ))
    .err()
    .map(|errors| codes(&errors));
    assert_eq!(presence, boolean);
}

// ---- The coverage refusal skipped beside an identity read ---------------------------------------

/// The skipped partition is said to repeat the identity refusal's cause. Here a second branch
/// reads an open stored field, so the command needs a default whatever the identity read becomes;
/// that refusal has its own cause and must still be reported.
#[test]
fn an_open_guard_beside_an_identity_read_keeps_its_coverage_refusal() {
    let text = r"format: ess/32
system: demo
version: v1
domain: demo.grants
types:
  - {name: demo.grants.ParentId, kind: newtype, of: Uuid}
entities:
  - name: demo.grants.Grant
    identity: {name: parent_id, type: demo.grants.ParentId}
    fields:
      - {name: uses, type: Integer}
    lifecycle: {initial: Live, states: [Live], terminal: [Live]}
events:
  - name: demo.grants.Minted
    fields: [{name: parent_id, type: demo.grants.ParentId}]
commands:
  - name: demo.grants.Mint
    input: [{name: parent_id, type: demo.grants.ParentId}]
    outcomes:
      - name: first
        unknown_instance: true
        creates: demo.grants.Grant
        instance: parent_id
        emits: [demo.grants.Minted]
        payload: {demo.grants.Minted: {parent_id: input.parent_id}}
        sets: {uses: 0}
      - name: already
        when_subject: {predicate: defined(parent_id)}
        updates: demo.grants.Grant
        instance: parent_id
        emits: [demo.grants.Minted]
        payload: {demo.grants.Minted: {parent_id: input.parent_id}}
        sets: {uses: 1}
      - name: busy
        when_subject: {predicate: uses > 3}
        updates: demo.grants.Grant
        instance: parent_id
        emits: [demo.grants.Minted]
        payload: {demo.grants.Minted: {parent_id: input.parent_id}}
        sets: {uses: 1}
";
    // The control: without the identity read the open guard draws its own coverage refusal.
    let alone = refusal(&text.replace(
        "        when_subject: {predicate: defined(parent_id)}\n",
        "        when_subject: {predicate: uses == 0}\n",
    ));
    assert!(
        alone.as_slice().iter().any(|error| {
            error.code == ValidationCode::NonExhaustiveBranches && error.message.contains("uses")
        }),
        "control: {alone}"
    );
    let errors = refusal(text);
    assert!(
        codes(&errors).contains(&ValidationCode::UnobservableFact),
        "{errors}"
    );
    assert!(
        errors.as_slice().iter().any(|error| {
            error.code == ValidationCode::NonExhaustiveBranches && error.message.contains("uses")
        }),
        "the open guard over `uses` needs a default on its own: {errors}"
    );
}

// ---- The default-bearing decision, where it is observable ---------------------------------------

/// With a default the stored partition keeps its enum-only proof. At `ess/32` the one rule answers
/// any overlap, so admitting presence there would change nothing visible; below `ess/27` it would
/// refuse the present case as two selected branches. This is where the decision can fail.
#[test]
fn a_default_bearing_pair_below_the_one_rule_keeps_its_enum_only_proof() {
    let kept = "      - name: kept\n        updates: demo.accounts.Account\n        \
                instance: account_id\n        emits: [demo.accounts.ResetSent]\n        \
                payload: {demo.accounts.ResetSent: {account_id: input.account_id}}\n";
    let text = account(&format!(
        "{}{}{kept}",
        resend("resent", "defined(reset_token)", None),
        resend("again", "defined(reset_token)", None)
    ))
    .replace("format: ess/32\n", "format: ess/26\n");
    admits(&text).unwrap_or_else(|errors| panic!("{errors}"));
}
