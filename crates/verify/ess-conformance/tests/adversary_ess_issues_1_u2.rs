//! Adversary pass on the presence partition (`defined(x)` / `not defined(x)` proved exhaustive
//! without a default): every shape validation now admits must be witnessed branch by branch by the
//! synthesized suite, and every scenario must pass on the interpreted target.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::Synthesis, AdmittedSuite, Runner,
};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

fn synthesis(text: &str) -> (EssIr, Synthesis) {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap();
    let result = ess_conformance::synthesize::synthesize(&ir);
    (ir, result)
}

/// Every branch in `branches` has a scenario, synthesis refuses nothing, and every scenario
/// passes on the interpreted target.
fn witnessed(text: &str, branches: &[&str]) {
    let (ir, result) = synthesis(text);
    let refusals: Vec<String> = result
        .refusals
        .iter()
        .map(|refusal| format!("{} {}", refusal.cause.code(), refusal.cause))
        .collect();
    assert!(refusals.is_empty(), "synthesis refused: {refusals:#?}");
    let ids: Vec<String> = result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect();
    for branch in branches {
        assert!(
            ids.iter().any(|id| id == branch),
            "no scenario {branch} in {ids:#?}"
        );
    }
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = Runner::for_suite(&result.suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let statuses: Vec<(String, Status)> = report
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect();
    assert_eq!(statuses.len(), ids.len(), "{statuses:#?}");
    assert!(
        statuses.iter().all(|(_, status)| *status == Status::Passed),
        "{statuses:#?}"
    );
}

/// `demo.orders.Checkout` with `input` and `outcomes`, each branch emitting its own event.
fn checkout(input: &str, outcomes: &[(&str, &str)]) -> String {
    use std::fmt::Write as _;
    let mut events = String::new();
    let mut branches = String::new();
    for (name, guard) in outcomes {
        let emitted = event(name);
        writeln!(events, "  - {{name: demo.orders.{emitted}, fields: []}}").unwrap();
        writeln!(
            branches,
            "      - name: {name}\n        when: {guard}\n        emits: [demo.orders.{emitted}]"
        )
        .unwrap();
    }
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
      - {{name: note, type: Optional<String>}}
events:
{events}actors:
  - name: demo.orders.Shopper
    may: [demo.orders.Checkout]
commands:
  - name: demo.orders.Checkout
    input:
{input}    outcomes:
{branches}"
    )
}

/// `full-price` as `FullPrice`.
fn event(name: &str) -> String {
    name.split('-')
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map_or(String::new(), |first| {
                first.to_uppercase().chain(chars).collect()
            })
        })
        .collect()
}

#[test]
fn a_presence_by_value_grid_is_witnessed_and_passes() {
    witnessed(
        &checkout(
            "      - {name: coupon, type: Optional<String>}\n      - {name: kind, type: demo.orders.Kind}\n",
            &[
                ("gift", "{all: [defined(coupon), kind == Gift]}"),
                ("promo", "{all: [defined(coupon), kind == Promo]}"),
                ("full-price", "not defined(coupon)"),
            ],
        ),
        &[
            "demo.orders.Checkout/outcome/gift",
            "demo.orders.Checkout/outcome/promo",
            "demo.orders.Checkout/outcome/full-price",
        ],
    );
}

#[test]
fn a_pair_over_an_optional_list_is_witnessed_and_passes() {
    witnessed(
        &checkout(
            "      - {name: tags, type: Optional<List<String>>}\n",
            &[
                ("tagged", "defined(tags)"),
                ("untagged", "not defined(tags)"),
            ],
        ),
        &[
            "demo.orders.Checkout/outcome/tagged",
            "demo.orders.Checkout/outcome/untagged",
        ],
    );
}

#[test]
fn a_pair_over_an_optional_struct_is_witnessed_and_passes() {
    witnessed(
        &checkout(
            "      - {name: addr, type: Optional<demo.orders.Address>}\n",
            &[
                ("shipped", "defined(addr)"),
                ("collected", "not defined(addr)"),
            ],
        ),
        &[
            "demo.orders.Checkout/outcome/shipped",
            "demo.orders.Checkout/outcome/collected",
        ],
    );
}

#[test]
fn a_pair_over_an_optional_member_of_a_required_struct_is_witnessed_and_passes() {
    witnessed(
        &checkout(
            "      - {name: addr, type: demo.orders.Address}\n",
            &[
                ("noted", "defined(addr.note)"),
                ("unnoted", "not defined(addr.note)"),
            ],
        ),
        &[
            "demo.orders.Checkout/outcome/noted",
            "demo.orders.Checkout/outcome/unnoted",
        ],
    );
}

#[test]
fn a_pair_under_any_is_witnessed_and_passes() {
    witnessed(
        &checkout(
            "      - {name: coupon, type: Optional<String>}\n      - {name: voucher, type: Optional<String>}\n",
            &[
                ("discounted", "{any: [defined(coupon), defined(voucher)]}"),
                (
                    "full-price",
                    "{all: [not defined(coupon), not defined(voucher)]}",
                ),
            ],
        ),
        &[
            "demo.orders.Checkout/outcome/discounted",
            "demo.orders.Checkout/outcome/full-price",
        ],
    );
}

/// The stored pair crossed with an input pair: `resent` where a token is stored, otherwise by
/// whether a coupon is sent.
const STORED_BY_INPUT: &str = r"format: ess/32
system: demo
version: v1
domain: demo.accounts
types:
  - {name: demo.accounts.AccountId, kind: newtype, of: Uuid}
entities:
  - name: demo.accounts.Account
    identity: {name: account_id, type: demo.accounts.AccountId}
    fields:
      - {name: reset_token, type: Optional<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.accounts.Opened
    fields: [{name: account_id, type: demo.accounts.AccountId}]
  - name: demo.accounts.ResetSent
    fields: [{name: account_id, type: demo.accounts.AccountId}]
  - name: demo.accounts.ResetIssued
    fields: [{name: account_id, type: demo.accounts.AccountId}]
  - name: demo.accounts.ResetDiscounted
    fields: [{name: account_id, type: demo.accounts.AccountId}]
actors:
  - name: demo.accounts.User
    may: [demo.accounts.Open, demo.accounts.ResendReset]
commands:
  - name: demo.accounts.Open
    input: []
    outcomes:
      - name: opened
        creates: demo.accounts.Account
        instance: account_id
        emits: [demo.accounts.Opened]
        payload: {demo.accounts.Opened: {account_id: {generated: true}}}
  - name: demo.accounts.ResendReset
    input:
      - {name: account_id, type: demo.accounts.AccountId}
      - {name: coupon, type: Optional<String>}
    outcomes:
      - name: resent
        when_subject: {predicate: defined(reset_token)}
        updates: demo.accounts.Account
        instance: account_id
        sets: {reset_token: t}
        emits: [demo.accounts.ResetSent]
        payload: {demo.accounts.ResetSent: {account_id: input.account_id}}
      - name: discounted
        when_subject: {predicate: not defined(reset_token)}
        when: defined(coupon)
        updates: demo.accounts.Account
        instance: account_id
        sets: {reset_token: t}
        emits: [demo.accounts.ResetDiscounted]
        payload: {demo.accounts.ResetDiscounted: {account_id: input.account_id}}
      - name: issued
        when_subject: {predicate: not defined(reset_token)}
        when: not defined(coupon)
        updates: demo.accounts.Account
        instance: account_id
        sets: {reset_token: t}
        emits: [demo.accounts.ResetIssued]
        payload: {demo.accounts.ResetIssued: {account_id: input.account_id}}
views:
  - name: demo.accounts.Accounts
    source: demo.accounts.Account
    consistency: read_your_writes
    fields:
      - {name: account_id, type: demo.accounts.AccountId}
      - {name: reset_token, type: Optional<String>}
";

#[test]
fn a_stored_pair_crossed_with_an_input_pair_is_witnessed_and_passes() {
    witnessed(
        STORED_BY_INPUT,
        &[
            "demo.accounts.ResendReset/outcome/resent",
            "demo.accounts.ResendReset/outcome/discounted",
            "demo.accounts.ResendReset/outcome/issued",
        ],
    );
}

/// A seat moved to a tier with an optional note, answered by whether the tier carries one.
const RELATED_PAIR: &str = r"format: ess/32
system: demo
version: v1
domain: demo.seats
types:
  - {name: demo.seats.SeatId, kind: newtype, of: Uuid}
  - {name: demo.seats.TierId, kind: newtype, of: Uuid}
entities:
  - name: demo.seats.Tier
    identity: {name: tier_id, type: demo.seats.TierId}
    fields:
      - {name: note, type: Optional<String>}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.seats.Seat
    identity: {name: seat_id, type: demo.seats.SeatId}
    fields:
      - {name: tier_id, type: demo.seats.TierId}
    relations:
      - {name: tier, kind: references, target: demo.seats.Tier, cardinality: one, via: tier_id}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
errors:
  - {name: demo.seats.UnknownSeat, summary: No seat carries the identity., fields: []}
  - {name: demo.seats.UnknownTier, summary: No tier carries the identity., fields: []}
events:
  - name: demo.seats.TierDefined
    fields: [{name: tier_id, type: demo.seats.TierId}]
  - name: demo.seats.Issued
    fields: [{name: seat_id, type: demo.seats.SeatId}]
  - name: demo.seats.Noted
    fields: [{name: seat_id, type: demo.seats.SeatId}]
  - name: demo.seats.Unnoted
    fields: [{name: seat_id, type: demo.seats.SeatId}]
actors:
  - name: demo.seats.Clerk
    may: [demo.seats.DefineTier, demo.seats.Issue, demo.seats.ChangeTier]
commands:
  - name: demo.seats.DefineTier
    input: [{name: note, type: Optional<String>}]
    outcomes:
      - name: defined
        creates: demo.seats.Tier
        instance: tier_id
        sets: {note: input.note}
        emits: [demo.seats.TierDefined]
        payload: {demo.seats.TierDefined: {tier_id: {generated: true}}}
  - name: demo.seats.Issue
    input: [{name: tier_id, type: demo.seats.TierId}]
    outcomes:
      - name: issued
        creates: demo.seats.Seat
        instance: seat_id
        sets: {tier_id: input.tier_id}
        emits: [demo.seats.Issued]
        payload: {demo.seats.Issued: {seat_id: {generated: true}}}
  - name: demo.seats.ChangeTier
    input:
      - {name: seat_id, type: demo.seats.SeatId}
      - {name: tier_id, type: demo.seats.TierId}
    outcomes:
      - name: unknown-seat
        unknown_instance: true
        error: demo.seats.UnknownSeat
      - name: unknown-tier
        when_related: {via: input.tier_id, exists: false}
        error: demo.seats.UnknownTier
      - name: noted
        when_related: {via: input.tier_id, predicate: defined(note)}
        updates: demo.seats.Seat
        instance: seat_id
        sets: {tier_id: input.tier_id}
        emits: [demo.seats.Noted]
        payload: {demo.seats.Noted: {seat_id: input.seat_id}}
      - name: unnoted
        when_related: {via: input.tier_id, predicate: not defined(note)}
        updates: demo.seats.Seat
        instance: seat_id
        sets: {tier_id: input.tier_id}
        emits: [demo.seats.Unnoted]
        payload: {demo.seats.Unnoted: {seat_id: input.seat_id}}
views:
  - name: demo.seats.Seats
    source: demo.seats.Seat
    consistency: read_your_writes
    fields:
      - {name: seat_id, type: demo.seats.SeatId}
      - {name: tier_id, type: demo.seats.TierId}
";

#[test]
fn a_related_pair_is_witnessed_and_passes() {
    witnessed(
        RELATED_PAIR,
        &[
            "demo.seats.ChangeTier/outcome/noted",
            "demo.seats.ChangeTier/outcome/unnoted",
        ],
    );
}

/// An account with an optional token, a flag and a plan; `Resend` answers by `first`, then
/// `special`, then `last`, each a `when_subject:` update that sets `sets`.
fn plans(first: &str, special: &str, last: &str, sets: &str) -> String {
    let branch = |name: &str, guard: &str| {
        format!(
            "      - name: {name}\n        when_subject: {{predicate: {guard}}}\n        \
             updates: demo.accounts.Account\n        instance: account_id\n        \
             sets: {{{sets}}}\n        emits: [demo.accounts.Sent]\n        \
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
actors:
  - name: demo.accounts.User
    may: [demo.accounts.Open, demo.accounts.Resend]
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
{}{}{}views:
  - name: demo.accounts.Accounts
    source: demo.accounts.Account
    consistency: read_your_writes
    fields:
      - {{name: account_id, type: demo.accounts.AccountId}}
      - {{name: reset_token, type: Optional<String>}}
      - {{name: flag, type: Boolean}}
      - {{name: plan, type: demo.accounts.Plan}}
",
        branch("first", first),
        branch("special", special),
        branch("last", last)
    )
}

/// What synthesis and the interpreted target answer for each `Resend` branch: its scenario's
/// status, or `None` where it has none; and the refusal codes.
fn answered(text: &str) -> (Vec<(String, Option<Status>)>, Vec<String>) {
    let (ir, result) = synthesis(text);
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap();
    let report = Runner::for_suite(&result.suite)
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    let branches = ["first", "special", "last"]
        .iter()
        .map(|branch| {
            let id = format!("demo.accounts.Resend/outcome/{branch}");
            let status = report
                .scenarios
                .iter()
                .find(|scenario| scenario.scenario.to_string() == id)
                .map(|scenario| scenario.status);
            ((*branch).to_owned(), status)
        })
        .collect();
    let refusals = result
        .refusals
        .iter()
        .map(|refusal| refusal.cause.code().to_string())
        .collect();
    (branches, refusals)
}

/// A third branch the pair shadows under the one rule is synthesized and run as its Boolean twin is.
#[test]
fn a_shadowed_third_branch_is_answered_as_its_boolean_twin() {
    let boolean = answered(&plans(
        "flag == true",
        "{all: [flag == true, plan == Pro]}",
        "flag == false",
        "flag: true",
    ));
    let presence = answered(&plans(
        "defined(reset_token)",
        "{all: [defined(reset_token), plan == Pro]}",
        "not defined(reset_token)",
        "reset_token: t",
    ));
    assert_eq!(presence, boolean);
    for branch in ["first", "last"] {
        assert!(
            presence
                .0
                .iter()
                .any(|(name, status)| name == branch && *status == Some(Status::Passed)),
            "{branch} is witnessed and passes: {presence:?}"
        );
    }
}
