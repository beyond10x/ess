//! A binding whose trigger and invoked command are both decided by a predicate over one related
//! row (beyond10x/ess#706).
//!
//! `Refresh` publishes `EligibilityChanged` on one of two branches chosen by `eligible` of the
//! `Tier` its input names, and the binding invokes `Issue`, which refuses `not-eligible` on the
//! same row. One row decides both commands, so a scenario arranges it for the invoked command's
//! branch, evaluates the trigger's guards on it, and keeps the arrangement where the branch the
//! trigger takes is the one that publishes.

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use serde_json::Value;
use sha2::Digest;

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

const BINDING: &str = "issue-on-eligibility-change";

fn model(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let spec = Specification::assemble([(Source::new("r.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

fn rewrite(from: &str, to: &str) -> String {
    assert_eq!(FIXTURE.matches(from).count(), 1, "`{from}` occurs once");
    FIXTURE.replacen(from, to, 1)
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

fn steps_of(synthesis: &Synthesis, id: &str) -> Vec<Value> {
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    document["scenarios"][format!("{BINDING}/binding/{id}")]["steps"]
        .as_array()
        .unwrap_or_else(|| panic!("`{id}` is synthesized: {:#?}", refusals(synthesis)))
        .clone()
}

/// The `eligible` the tier the trigger is sent naming was created with.
fn tier_eligible(steps: &[Value]) -> Value {
    let sent = steps
        .iter()
        .find(|step| step["command"] == "demo.r.Refresh")
        .expect("the trigger");
    let named = sent["input"]["tier_id"]["instance"]
        .as_str()
        .unwrap_or_else(|| panic!("the trigger names an arranged tier: {sent}"));
    let captured = steps
        .iter()
        .position(|step| step["step"] == "capture_instance" && step["instance"] == named)
        .expect("the tier is captured");
    let created = &steps[captured - 2];
    assert_eq!(created["command"], "demo.r.DefineTier", "{steps:#?}");
    created["input"]["eligible"]["value"].clone()
}

fn statuses(ir: EssIr, synthesis: &Synthesis) -> Vec<(String, Status)> {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

fn status_of(statuses: &[(String, Status)], id: &str) -> Status {
    statuses
        .iter()
        .find(|(scenario, _)| scenario == &format!("{BINDING}/binding/{id}"))
        .unwrap_or_else(|| panic!("`{id}` ran: {statuses:#?}"))
        .1
}

#[test]
fn one_row_deciding_trigger_and_invoked_command_gives_flow_delivery_and_the_refusal() {
    let synthesis = synthesize(&model(FIXTURE));
    for id in ["flow", "delivery", "refusal/not-eligible"] {
        assert_eq!(refused_as(&synthesis, id), Vec::<String>::new(), "{id}");
        steps_of(&synthesis, id);
    }
    let document: Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    assert_eq!(
        (
            document["scenarios"].as_object().expect("scenarios").len(),
            synthesis.refusals.len()
        ),
        (12, 1),
        "{:#?}",
        refusals(&synthesis)
    );
    let refused = refused_as(&synthesis, "refusal/tier-missing");
    assert_eq!(refused.len(), 1, "{:#?}", refusals(&synthesis));
    assert!(
        refusals(&synthesis).iter().all(|refusal| refusal
            .contains(&format!("`{BINDING}/binding/refusal/tier-missing`"))
            || !refusal.contains(BINDING)),
        "{:#?}",
        refusals(&synthesis)
    );
}

#[test]
fn the_tier_is_created_for_the_branch_the_scenario_claims() {
    let synthesis = synthesize(&model(FIXTURE));
    for id in ["flow", "delivery"] {
        let steps = steps_of(&synthesis, id);
        assert_eq!(tier_eligible(&steps), Value::Bool(true), "{id}");
        assert!(
            steps.iter().any(|step| step["event"] == "demo.r.Issued"),
            "{id} expects the lease issued: {steps:#?}"
        );
    }
    let refusal = steps_of(&synthesis, "refusal/not-eligible");
    assert_eq!(tier_eligible(&refusal), Value::Bool(false));
    assert!(
        !refusal.iter().any(|step| step["event"] == "demo.r.Issued"),
        "{refusal:#?}"
    );
}

#[test]
fn the_interpreted_target_passes_every_scenario() {
    let ir = model(FIXTURE);
    let synthesis = synthesize(&ir);
    let statuses = statuses(ir, &synthesis);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| *status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "{failing:#?}");
    for id in ["flow", "delivery", "refusal/not-eligible"] {
        status_of(&statuses, id);
    }
}

#[test]
fn a_target_that_never_invokes_issue_fails_the_flow() {
    let synthesis = synthesize(&model(FIXTURE));
    let without = rewrite(
        "  - id: issue-on-eligibility-change
    when: {event: demo.r.EligibilityChanged}
    invoke: {command: demo.r.Issue}
    mapping: {member_id: event.member_id, tier_id: event.tier_id}
    delivery: at_least_once
    on_failure:
      drop: [tier-missing, not-eligible]
      retry: {except: [tier-missing, not-eligible]}
",
        "",
    );
    let statuses = statuses(model(&without), &synthesis);
    assert_ne!(status_of(&statuses, "flow"), Status::Passed);
}

#[test]
fn a_target_that_retries_not_eligible_fails_the_refusal() {
    let synthesis = synthesize(&model(FIXTURE));
    let retrying = rewrite(
        "      drop: [tier-missing, not-eligible]
      retry: {except: [tier-missing, not-eligible]}
",
        "      drop: [tier-missing]
      retry: {except: [tier-missing]}
",
    );
    let statuses = statuses(model(&retrying), &synthesis);
    assert_ne!(status_of(&statuses, "refusal/not-eligible"), Status::Passed);
}

fn kept(text: &str) -> Synthesis {
    let synthesis = synthesize(&model(text));
    for id in ["flow", "delivery", "refusal/not-eligible"] {
        assert!(
            refused_as(&synthesis, id)
                .iter()
                .any(|refusal| refusal.contains("ESS-SYNTH-010")),
            "{id} keeps ESS-SYNTH-010: {:#?}",
            refusals(&synthesis)
        );
    }
    synthesis
}

#[test]
fn a_trigger_predicate_also_reading_an_input_keeps_its_refusals() {
    let text = rewrite(
        "predicate: eligible == true}",
        "predicate: eligible == input.flag}",
    )
    .replacen(
        "  - name: demo.r.Refresh
    input:
",
        "  - name: demo.r.Refresh
    input:
      - {name: flag, type: Boolean}
",
        1,
    );
    kept(&text);
}

/// The model where the trigger reads a badge row through its own `badge_id`, which the event
/// carries as the `tier_id` the binding copies, across a declared conversion that hands the value
/// on as it is: the invoked command reads a tier row through it.
fn badge_model() -> EssIr {
    let refresh_from = FIXTURE.find("  - name: demo.r.Refresh").expect("Refresh");
    let issue_from = FIXTURE.find("  - name: demo.r.Issue\n").expect("Issue");
    let bindings_from = FIXTURE.find("bindings:\n").expect("bindings");
    let views_from = FIXTURE.find("views:\n").expect("views");
    let domain = format!(
        "{}{}{}{}",
        &FIXTURE[..refresh_from],
        REFRESH_BY_BADGE,
        &FIXTURE[issue_from..bindings_from],
        &FIXTURE[views_from..],
    )
    .replacen("format: ess/24\nsystem: demo\nversion: v1\n", "", 1)
    .replacen(
        "  - {name: demo.r.TierId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.r.TierId, kind: newtype, of: Uuid}\n  - {name: demo.r.BadgeId, kind: newtype, of: Uuid}\n",
        1,
    )
    .replacen(
        "  - name: demo.r.Membership\n    identity",
        BADGE_ENTITY,
        1,
    )
    .replacen(
        "  - name: demo.r.Joined\n    fields",
        "  - name: demo.r.BadgeDefined\n    fields: [{name: badge_id, type: demo.r.BadgeId}]\n  - name: demo.r.Joined\n    fields",
        1,
    )
    .replacen("  - name: demo.r.Join\n    input:", DEFINE_BADGE, 1);
    let bindings = &FIXTURE[bindings_from..views_from];
    let components = format!("{COMPONENTS}{bindings}");
    let parsed = [
        ("system.yaml", SYSTEM),
        ("components.yaml", components.as_str()),
        ("r.yaml", domain.as_str()),
    ]
    .map(|(label, text)| {
        let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{label}: {error}"));
        (Source::new(label), raw)
    });
    let spec = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|d| panic!("the fixture resolves:\n{d}"))
}

#[test]
fn a_trigger_reading_a_row_of_another_entity_keeps_its_refusals() {
    let synthesis = synthesize(&badge_model());
    for id in ["flow", "delivery", "refusal/not-eligible"] {
        assert!(
            refused_as(&synthesis, id)
                .iter()
                .any(|refusal| refusal.contains("ESS-SYNTH-010")),
            "{id} keeps ESS-SYNTH-010: {:#?}",
            refusals(&synthesis)
        );
    }
}

const SYSTEM: &str = "format: ess/25
system: demo
version: v1
domains:
  - demo.r
";

const COMPONENTS: &str = "conversions:
  - from: demo.r.BadgeId
    to: demo.r.TierId
    because: A badge names the tier it grants.
components:
  - component: svc
    owns:
      domains: [demo.r]
    accepts:
      commands:
        [demo.r.DefineTier, demo.r.DefineBadge, demo.r.Join, demo.r.Refresh, demo.r.Issue]
";

const BADGE_ENTITY: &str = "  - name: demo.r.Badge
    identity: {name: badge_id, type: demo.r.BadgeId}
    fields:
      - {name: eligible, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.r.Membership
    identity";

const DEFINE_BADGE: &str = "  - name: demo.r.DefineBadge
    input: [{name: eligible, type: Boolean}]
    outcomes:
      - name: defined
        creates: demo.r.Badge
        instance: badge_id
        sets: {eligible: input.eligible}
        emits: [demo.r.BadgeDefined]
        payload: {demo.r.BadgeDefined: {badge_id: {generated: true}}}
  - name: demo.r.Join
    input:";

const REFRESH_BY_BADGE: &str = "  - name: demo.r.Refresh
    input:
      - {name: member_id, type: demo.r.MemberRef}
      - {name: badge_id, type: demo.r.BadgeId}
    outcomes:
      - name: badge-missing
        when_related: {via: input.badge_id, exists: false}
        error: demo.r.NoTier
      - name: eligible
        when_related: {via: input.badge_id, predicate: eligible == true}
        updates: demo.r.Membership
        instance: member_id
        emits: [demo.r.EligibilityChanged]
        payload:
          demo.r.EligibilityChanged:
            {member_id: {subject: member_id}, tier_id: input.badge_id, eligible: true}
        sets: {eligible: true}
      - name: not-eligible
        updates: demo.r.Membership
        instance: member_id
        emits: [demo.r.EligibilityChanged]
        payload:
          demo.r.EligibilityChanged:
            {member_id: {subject: member_id}, tier_id: input.badge_id, eligible: false}
        sets: {eligible: false}
";

#[test]
fn a_trigger_publishing_only_for_rows_the_invoked_refusal_does_not_refuse_names_the_predicate() {
    // The default branch publishes nothing: the trigger publishes only for an eligible tier, so
    // no row the invoked command refuses reaches the binding.
    let text = rewrite(
        "        emits: [demo.r.EligibilityChanged]
        payload:
          demo.r.EligibilityChanged:
            {member_id: {subject: member_id}, tier_id: input.tier_id, eligible: false}
",
        "        emits: [demo.r.Unchanged]
        payload: {demo.r.Unchanged: {member_id: {subject: member_id}}}
",
    );
    let synthesis = synthesize(&model(&text));
    for id in ["flow", "delivery"] {
        assert_eq!(refused_as(&synthesis, id), Vec::<String>::new(), "{id}");
    }
    let refused = refused_as(&synthesis, "refusal/not-eligible");
    assert_eq!(refused.len(), 1, "{:#?}", refusals(&synthesis));
    assert!(
        refused[0].contains("chooses its branch by `when_related:` predicates over that row"),
        "{}",
        refused[0]
    );
    assert!(
        !refused[0].contains("synthesis arranges one itself"),
        "the help no longer states conditions that hold: {}",
        refused[0]
    );
}

const REFRESH_COPYING: &str = "  - name: demo.r.Refresh
    input:
      - {name: member_id, type: demo.r.MemberRef}
      - {name: tier_id, type: demo.r.TierId}
    outcomes:
      - name: tier-missing
        when_related: {via: input.tier_id, exists: false}
        error: demo.r.NoTier
      - name: refreshed
        updates: demo.r.Membership
        instance: member_id
        emits: [demo.r.EligibilityChanged]
        payload:
          demo.r.EligibilityChanged:
            member_id: {subject: member_id}
            tier_id: input.tier_id
            eligible: {related: {via: input.tier_id, field: eligible}}
        sets:
          tier_id: input.tier_id
          eligible: {related: {via: input.tier_id, field: eligible}}
";

#[test]
fn the_control_trigger_copying_the_flag_keeps_its_suite() {
    let refresh_from = FIXTURE.find("  - name: demo.r.Refresh").expect("Refresh");
    let issue_from = FIXTURE.find("  - name: demo.r.Issue\n").expect("Issue");
    let text = format!(
        "{}{}{}",
        &FIXTURE[..refresh_from],
        REFRESH_COPYING,
        &FIXTURE[issue_from..],
    );
    let synthesis = synthesize(&model(&text));
    let canonical = synthesis.suite.to_canonical_json().expect("serialises");
    let document: Value = serde_json::from_str(&canonical).expect("JSON");
    let scenarios = document["scenarios"].as_object().expect("scenarios").len();
    assert_eq!(
        (scenarios, synthesis.refusals.len()),
        (11, 1),
        "{:#?}",
        refusals(&synthesis)
    );
    // The suite bytes as they were before the trigger's row predicate was arranged.
    let printed =
        sha2::Sha256::digest(canonical.as_bytes())
            .iter()
            .fold(String::new(), |mut hex, byte| {
                use std::fmt::Write;
                write!(hex, "{byte:02x}").expect("writes to a string");
                hex
            });
    assert_eq!(
        (printed.as_str(), canonical.len()),
        (CONTROL_DIGEST, CONTROL_BYTES)
    );
}

const CONTROL_DIGEST: &str = "bdcc1e48a2845cc6da4064b4f08e19af1b0516a5006d905b22fcbcc52dc84f6b";
const CONTROL_BYTES: usize = 70422;

// Adversary cases (turn 1).

/// The outcome the scenario expects of the trigger `demo.r.Refresh`.
fn adversary_trigger_expected(steps: &[Value]) -> Value {
    let sent = steps
        .iter()
        .position(|step| step["command"] == "demo.r.Refresh")
        .expect("the trigger is sent");
    let expected = &steps[sent + 1];
    assert_eq!(expected["step"], "expect_outcome", "{steps:#?}");
    expected["outcome"]["outcome"].clone()
}

/// Acceptance: "In `binding/refusal/not-eligible` ... the trigger is expected on its `not-eligible`
/// branch"; flow and delivery expect it on `eligible`.
#[test]
fn adversary_the_trigger_is_expected_on_the_branch_the_row_sends_it_down() {
    let synthesis = synthesize(&model(FIXTURE));
    for id in ["flow", "delivery"] {
        assert_eq!(
            adversary_trigger_expected(&steps_of(&synthesis, id)),
            Value::from("eligible"),
            "{id}"
        );
    }
    assert_eq!(
        adversary_trigger_expected(&steps_of(&synthesis, "refusal/not-eligible")),
        Value::from("not-eligible")
    );
}

/// The trigger refuses an ineligible tier on an earlier sibling and publishes on its default branch:
/// it publishes only for rows the invoked refusal does not refuse, as in the story's third kept case,
/// with the guard on the refusing sibling instead of the publishing branch. The refusal must name
/// the trigger's predicate and must not repeat the help whose two conditions hold here.
#[test]
fn adversary_a_refusing_sibling_guard_names_the_trigger_predicate() {
    let refresh_from = FIXTURE.find("      - name: eligible\n").expect("eligible");
    let issue_from = FIXTURE.find("  - name: demo.r.Issue\n").expect("Issue");
    let text = format!(
        "{}{}{}",
        &FIXTURE[..refresh_from],
        "      - name: blocked
        when_related: {via: input.tier_id, predicate: eligible == false}
        error: demo.r.NoTier
      - name: refreshed
        updates: demo.r.Membership
        instance: member_id
        emits: [demo.r.EligibilityChanged]
        payload:
          demo.r.EligibilityChanged:
            {member_id: {subject: member_id}, tier_id: input.tier_id, eligible: true}
        sets: {tier_id: input.tier_id, eligible: true}
",
        &FIXTURE[issue_from..],
    );
    let synthesis = synthesize(&model(&text));
    for id in ["flow", "delivery"] {
        assert_eq!(refused_as(&synthesis, id), Vec::<String>::new(), "{id}");
    }
    let refused = refused_as(&synthesis, "refusal/not-eligible");
    assert_eq!(refused.len(), 1, "{:#?}", refusals(&synthesis));
    assert!(
        !refused[0].contains("synthesis arranges one itself"),
        "the help states two conditions that hold: {}",
        refused[0]
    );
    assert!(
        refused[0].contains("demo.r.Refresh"),
        "the refusal names the trigger's predicate: {}",
        refused[0]
    );
}

/// The trigger's predicate reads an input (story kept case 1). `Refresh` publishes the event on its
/// unguarded `not-eligible` branch too, so a refusal saying it publishes only where its predicate
/// holds states something false.
#[test]
fn adversary_an_input_reading_predicate_is_not_called_the_only_publisher() {
    let text = rewrite(
        "predicate: eligible == true}",
        "predicate: eligible == input.flag}",
    )
    .replacen(
        "  - name: demo.r.Refresh
    input:
",
        "  - name: demo.r.Refresh
    input:
      - {name: flag, type: Boolean}
",
        1,
    );
    let synthesis = kept(&text);
    let refused = refused_as(&synthesis, "refusal/not-eligible");
    assert_eq!(refused.len(), 1, "{:#?}", refusals(&synthesis));
    assert!(
        !refused[0].contains("publishes the event only where"),
        "Refresh/not-eligible publishes the event too: {}",
        refused[0]
    );
}

/// The trigger `Touch` copies the tier from its own row (`{subject: tier_id}`), which `Open` created
/// naming it; `Open` refuses an ineligible tier on a branch that happens to share its name `ok` with
/// `Touch`'s publishing branch. A scenario arranging an ineligible tier for `Open` makes `Open`
/// refuse and `Touch` never run, so a written `refusal/not-eligible` must still pass against the
/// interpreted model of the same specification.
const STORED: &str = r"
format: ess/24
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
  - name: demo.r.Open
    input: [{name: tier_id, type: demo.r.TierId}]
    outcomes:
      - name: tier-missing
        when_related: {via: input.tier_id, exists: false}
        error: demo.r.NoTier
      - name: ok
        when_related: {via: input.tier_id, predicate: eligible == false}
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
      - name: ok
        updates: demo.r.Account
        instance: account_id
        sets: {touched: true}
        emits: [demo.r.Touched]
        payload: {demo.r.Touched: {account_id: input.account_id, tier_id: {subject: tier_id}}}
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
  - name: demo.r.Accounts
    source: demo.r.Account
    consistency: read_your_writes
    fields:
      - {name: account_id, type: demo.r.AccountId}
      - {name: tier_id, type: demo.r.TierId}
      - {name: touched, type: Boolean}
";

#[test]
fn adversary_a_creator_branch_sharing_the_publisher_name_does_not_pass_for_the_trigger() {
    let ir = model(STORED);
    let synthesis = synthesize(&ir);
    let admitted = AdmittedSuite::from_suite(&synthesis.suite).expect("the suite is admitted");
    let failing: Vec<_> = Runner::for_suite(admitted.suite())
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
    assert!(
        failing.is_empty(),
        "binding scenarios failing against the honest model: {failing:#?}\nrefusals: {:#?}",
        refusals(&synthesis)
    );
}
