//! Adversary pass 2 on beyond10x/ess#706 (a row predicate choosing the trigger branch).

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const BINDING: &str = "issue-on-eligibility-change";

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("r.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn refused_as(synthesis: &Synthesis, id: &str) -> Vec<String> {
    refusals(synthesis)
        .into_iter()
        .filter(|refusal| refusal.contains(&format!("`{BINDING}/binding/{id}`")))
        .collect()
}

fn binding_scenarios(synthesis: &Synthesis) -> Vec<String> {
    let document: serde_json::Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"]
        .as_object()
        .expect("scenarios")
        .keys()
        .filter(|id| id.starts_with(BINDING))
        .cloned()
        .collect()
}

/// The binding scenarios the honest interpreted model of the same specification fails, with the
/// failed checks.
fn failing_binding_scenarios(text: &str) -> (Vec<(String, Vec<String>)>, Synthesis) {
    let ir = model(text);
    let synthesis = synthesize(&ir);
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    let failing = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|scenario| {
            scenario.scenario.to_string().starts_with(BINDING) && scenario.status != Status::Passed
        })
        .map(|scenario| {
            let failed: Vec<_> = scenario
                .checks
                .iter()
                .filter(|check| format!("{check:?}").contains("Failed"))
                .map(|check| format!("{check:?}"))
                .collect();
            (scenario.scenario.to_string(), failed)
        })
        .collect();
    (failing, synthesis)
}

/// The trigger `Touch` copies the tier from its own account row (`{subject: tier_id}`) and chooses
/// its publishing branch by `when_related: {via: tier_id, predicate: eligible == true}`;
/// its default branch publishes `Skipped` only. `Open`, which creates the account naming the tier,
/// refuses a frozen tier only. A row arranged for `Issue/not-eligible` (an ineligible tier) sends `Touch`
/// down its default branch, so `Touch` never publishes for it: a written `refusal/not-eligible`
/// expecting `Touch/ok` fails the honest model.
const STORED_SUBJECT: &str = r"
format: ess/27
system: demo
version: v1
domain: demo.r
types:
  - {name: demo.r.TierId, kind: newtype, of: Uuid}
  - {name: demo.r.AccountId, kind: newtype, of: Uuid}
  - {name: demo.r.LeaseId, kind: newtype, of: Uuid}
entities:
  - name: demo.r.Tier
    identity: {name: tier_id, type: demo.r.TierId}
    fields:
      - {name: eligible, type: Boolean}
      - {name: frozen, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.r.Account
    identity: {name: account_id, type: demo.r.AccountId}
    fields:
      - {name: tier_id, type: demo.r.TierId}
      - {name: touched, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.r.Lease
    identity: {name: lease_id, type: demo.r.LeaseId}
    fields:
      - {name: tier_id, type: demo.r.TierId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: demo.r.NoTier, fields: []}
  - {name: demo.r.NoAccount, fields: []}
events:
  - name: demo.r.TierDefined
    fields: [{name: tier_id, type: demo.r.TierId}]
  - name: demo.r.Opened
    fields: [{name: account_id, type: demo.r.AccountId}]
  - name: demo.r.Touched
    fields:
      - {name: account_id, type: demo.r.AccountId}
      - {name: tier_id, type: demo.r.TierId}
  - name: demo.r.Skipped
    fields: [{name: account_id, type: demo.r.AccountId}]
  - name: demo.r.Issued
    fields: [{name: lease_id, type: demo.r.LeaseId}]
commands:
  - name: demo.r.DefineTier
    input: [{name: eligible, type: Boolean}, {name: frozen, type: Boolean}]
    outcomes:
      - name: defined
        creates: demo.r.Tier
        instance: tier_id
        sets: {eligible: input.eligible, frozen: input.frozen}
        emits: [demo.r.TierDefined]
        payload: {demo.r.TierDefined: {tier_id: {generated: true}}}
  - name: demo.r.Open
    input: [{name: tier_id, type: demo.r.TierId}]
    outcomes:
      - name: tier-missing
        when_related: {via: input.tier_id, exists: false}
        error: demo.r.NoTier
      - name: frozen
        when_related: {via: input.tier_id, predicate: frozen == true}
        error: demo.r.NoTier
      - name: opened
        creates: demo.r.Account
        instance: account_id
        sets: {tier_id: input.tier_id, touched: false}
        emits: [demo.r.Opened]
        payload: {demo.r.Opened: {account_id: {generated: true}}}
  - name: demo.r.Touch
    input: [{name: account_id, type: demo.r.AccountId}]
    outcomes:
      - name: tier-gone
        when_related: {via: tier_id, exists: false}
        error: demo.r.NoTier
      - name: ok
        when_related: {via: tier_id, predicate: eligible == true}
        updates: demo.r.Account
        instance: account_id
        sets: {touched: true}
        emits: [demo.r.Touched]
        payload: {demo.r.Touched: {account_id: input.account_id, tier_id: {subject: tier_id}}}
      - name: skipped
        updates: demo.r.Account
        instance: account_id
        sets: {touched: false}
        emits: [demo.r.Skipped]
        payload: {demo.r.Skipped: {account_id: input.account_id}}
      - {name: no-account, unknown_instance: true, error: demo.r.NoAccount}
  - name: demo.r.Issue
    input: [{name: tier_id, type: demo.r.TierId}]
    outcomes:
      - name: tier-missing
        when_related: {via: input.tier_id, exists: false}
        error: demo.r.NoTier
      - name: not-eligible
        when_related: {via: input.tier_id, predicate: eligible == false}
        error: demo.r.NoTier
      - name: issued
        creates: demo.r.Lease
        instance: lease_id
        sets: {tier_id: input.tier_id}
        emits: [demo.r.Issued]
        payload: {demo.r.Issued: {lease_id: {generated: true}}}
bindings:
  - id: issue-on-eligibility-change
    when: {event: demo.r.Touched}
    invoke: {command: demo.r.Issue}
    mapping: {tier_id: event.tier_id}
    delivery: at_least_once
    on_failure:
      drop: [tier-missing, not-eligible]
      retry: {except: [tier-missing, not-eligible]}
views:
  - name: demo.r.Tiers
    source: demo.r.Tier
    consistency: read_your_writes
    fields:
      - {name: tier_id, type: demo.r.TierId}
      - {name: eligible, type: Boolean}
      - {name: frozen, type: Boolean}
  - name: demo.r.Accounts
    source: demo.r.Account
    consistency: read_your_writes
    fields:
      - {name: account_id, type: demo.r.AccountId}
      - {name: tier_id, type: demo.r.TierId}
      - {name: touched, type: Boolean}
";

const FROZEN_BRANCH: &str = "      - name: frozen
        when_related: {via: input.tier_id, predicate: frozen == true}
        error: demo.r.NoTier
";

/// The creator `Open` carries a row predicate (`frozen == true`), so the stored path reaches
/// `trigger_publishes` (new in this story); `Touch`'s own predicate over the same row is not read.
#[test]
fn adv2_a_stored_trigger_guarded_by_its_own_row_is_not_sent_for_a_refused_row() {
    assert_eq!(STORED_SUBJECT.matches(FROZEN_BRANCH).count(), 1);
    let (failing, synthesis) = failing_binding_scenarios(STORED_SUBJECT);
    assert!(
        failing.is_empty(),
        "binding scenarios failing against the honest model: {failing:#?}\nwritten: {:#?}\nrefusals: {:#?}",
        binding_scenarios(&synthesis),
        refusals(&synthesis)
    );
}

/// The same with no predicate on the creator: the stored path takes `trigger_publishes`'s early
/// `true`, as the base did.
#[test]
fn adv2_a_stored_trigger_with_an_unguarded_creator_is_not_sent_for_a_refused_row() {
    let text = STORED_SUBJECT.replacen(FROZEN_BRANCH, "", 1);
    let (failing, synthesis) = failing_binding_scenarios(&text);
    assert!(
        failing.is_empty(),
        "binding scenarios failing against the honest model: {failing:#?}\nwritten: {:#?}\nrefusals: {:#?}",
        binding_scenarios(&synthesis),
        refusals(&synthesis)
    );
}

/// No row the refusal refuses can be created at all: `DefineTier` sets `eligible: true` whatever it
/// is sent. `Refresh` publishes the event on its default branch for any row, so its predicate is not
/// what keeps the row from being arranged; the refusal must still say what is (the creator does
/// not take the field the predicate reads as input), not blame the trigger's predicate.
#[test]
fn adv2_a_creator_that_cannot_make_the_refused_row_is_not_blamed_on_the_trigger() {
    let from = "    input: [{name: eligible, type: Boolean}]
    outcomes:
      - name: defined
        creates: demo.r.Tier
        instance: tier_id
        sets: {eligible: input.eligible}
";
    assert_eq!(FIXTURE.matches(from).count(), 1);
    let text = FIXTURE.replacen(
        from,
        "    input: []
    outcomes:
      - name: defined
        creates: demo.r.Tier
        instance: tier_id
        sets: {eligible: true}
",
        1,
    );
    let synthesis = synthesize(&model(&text));
    let refused = refused_as(&synthesis, "refusal/not-eligible");
    assert_eq!(refused.len(), 1, "{:#?}", refusals(&synthesis));
    assert!(
        !refused[0].contains("chooses its branch by `when_related:` predicates"),
        "the trigger's predicate is named as the cause, though its default branch publishes for \
         any row: {}",
        refused[0]
    );
    assert!(
        refused[0].contains("takes each field the predicate reads as input"),
        "the help no longer names the condition that fails: {}",
        refused[0]
    );
}

/// The unit's fixture, copied from `binding_trigger_row_predicate.rs`.
const FIXTURE: &str = r"
format: ess/24
system: demo
version: v1
domain: demo.r
types:
  - {name: demo.r.TierId, kind: newtype, of: Uuid}
  - {name: demo.r.MemberRef, kind: newtype, of: Uuid}
  - {name: demo.r.LeaseId, kind: newtype, of: Uuid}
entities:
  - name: demo.r.Tier
    identity: {name: tier_id, type: demo.r.TierId}
    fields:
      - {name: eligible, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.r.Membership
    identity: {name: member_id, type: demo.r.MemberRef}
    fields:
      - {name: tier_id, type: demo.r.TierId}
      - {name: eligible, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.r.Lease
    identity: {name: lease_id, type: demo.r.LeaseId}
    fields:
      - {name: member_id, type: demo.r.MemberRef}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
errors:
  - {name: demo.r.NoTier, fields: []}
events:
  - name: demo.r.TierDefined
    fields: [{name: tier_id, type: demo.r.TierId}]
  - name: demo.r.Joined
    fields: [{name: member_id, type: demo.r.MemberRef}]
  - name: demo.r.EligibilityChanged
    fields:
      - {name: member_id, type: demo.r.MemberRef}
      - {name: tier_id, type: demo.r.TierId}
      - {name: eligible, type: Boolean}
  - name: demo.r.Unchanged
    fields: [{name: member_id, type: demo.r.MemberRef}]
  - name: demo.r.Issued
    fields: [{name: lease_id, type: demo.r.LeaseId}]
commands:
  - name: demo.r.DefineTier
    input: [{name: eligible, type: Boolean}]
    outcomes:
      - name: defined
        creates: demo.r.Tier
        instance: tier_id
        sets: {eligible: input.eligible}
        emits: [demo.r.TierDefined]
        payload: {demo.r.TierDefined: {tier_id: {generated: true}}}
  - name: demo.r.Join
    input:
      - {name: member_id, type: demo.r.MemberRef}
      - {name: tier_id, type: demo.r.TierId}
    outcomes:
      - name: joined
        creates: demo.r.Membership
        instance: member_id
        sets: {tier_id: input.tier_id, eligible: false}
        emits: [demo.r.Joined]
        payload: {demo.r.Joined: {member_id: input.member_id}}
  - name: demo.r.Refresh
    input:
      - {name: member_id, type: demo.r.MemberRef}
      - {name: tier_id, type: demo.r.TierId}
    outcomes:
      - name: tier-missing
        when_related: {via: input.tier_id, exists: false}
        error: demo.r.NoTier
      - name: eligible
        when_related: {via: input.tier_id, predicate: eligible == true}
        updates: demo.r.Membership
        instance: member_id
        emits: [demo.r.EligibilityChanged]
        payload:
          demo.r.EligibilityChanged:
            {member_id: {subject: member_id}, tier_id: input.tier_id, eligible: true}
        sets: {tier_id: input.tier_id, eligible: true}
      - name: not-eligible
        updates: demo.r.Membership
        instance: member_id
        emits: [demo.r.EligibilityChanged]
        payload:
          demo.r.EligibilityChanged:
            {member_id: {subject: member_id}, tier_id: input.tier_id, eligible: false}
        sets: {tier_id: input.tier_id, eligible: false}
  - name: demo.r.Issue
    input:
      - {name: member_id, type: demo.r.MemberRef}
      - {name: tier_id, type: demo.r.TierId}
    outcomes:
      - name: tier-missing
        when_related: {via: input.tier_id, exists: false}
        error: demo.r.NoTier
      - name: not-eligible
        when_related: {via: input.tier_id, predicate: eligible == false}
        error: demo.r.NoTier
      - name: issued
        creates: demo.r.Lease
        instance: lease_id
        sets: {member_id: input.member_id}
        emits: [demo.r.Issued]
        payload: {demo.r.Issued: {lease_id: {generated: true}}}
bindings:
  - id: issue-on-eligibility-change
    when: {event: demo.r.EligibilityChanged}
    invoke: {command: demo.r.Issue}
    mapping: {member_id: event.member_id, tier_id: event.tier_id}
    delivery: at_least_once
    on_failure:
      drop: [tier-missing, not-eligible]
      retry: {except: [tier-missing, not-eligible]}
views:
  - name: demo.r.Tiers
    source: demo.r.Tier
    consistency: read_your_writes
    fields:
      - {name: tier_id, type: demo.r.TierId}
      - {name: eligible, type: Boolean}
  - name: demo.r.Memberships
    source: demo.r.Membership
    consistency: read_your_writes
    fields:
      - {name: member_id, type: demo.r.MemberRef}
      - {name: tier_id, type: demo.r.TierId}
      - {name: eligible, type: Boolean}
";
