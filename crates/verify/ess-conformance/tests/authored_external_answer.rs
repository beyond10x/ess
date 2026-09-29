//! An authored act that expects an `external:` branch (beyond10x/ess#243).
//!
//! No input decides an external branch (§12), so an act that expects one is satisfiable only if the
//! suite tells the target which answer to give for that call. Synthesis does so with
//! [`ScenarioStep::ConfigureExternalOutcome`]; an authored act states the same answer by naming the
//! branch under `outcome:`, and compiles to the same step before its `ExecuteCommand`. An act that
//! claims what only an external branch produces — its error, or an event nothing else publishes —
//! without naming the branch is refused (`ESS-AUTHOR-037`), naming every branch it could mean.

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_conformance::{ConformanceSuite, ScenarioStep, SuiteProvenance};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;
use std::path::{Path, PathBuf};

/// A command with one accepting branch, one input refusal and four external branches, two of which
/// report the same error, and one of which publishes an event no other branch publishes.
const COURIER: &str = r"
format: ess/16
system: courier
version: v1
domain: courier.mail
events:
  - name: courier.mail.MailSent
    fields:
      - {name: recipient, type: String}
  - name: courier.mail.MailQueued
    fields:
      - {name: recipient, type: String}
errors:
  - name: courier.mail.Undeliverable
    fields: []
  - name: courier.mail.Throttled
    fields: []
commands:
  - name: courier.mail.SendMail
    input:
      - {name: recipient, type: String}
      - {name: attempt, type: Integer}
    outcomes:
      - name: slowed
        when: attempt > 10
        error: courier.mail.Throttled
      - name: sent
        emits: [courier.mail.MailSent]
        payload:
          courier.mail.MailSent:
            recipient: input.recipient
      - name: queued
        external: the provider defers delivery
        emits: [courier.mail.MailQueued]
        payload:
          courier.mail.MailQueued:
            recipient: input.recipient
      - name: failed
        external: the provider rejects the address
        error: courier.mail.Undeliverable
      - name: bounced
        external: the provider accepts the mail and bounces it later
        error: courier.mail.Undeliverable
      - name: throttled
        external: the provider asks the sender to slow down
        error: courier.mail.Throttled
";

fn fixture(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let specification = Specification::assemble([(SpecSource::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

/// An example directory, compiled from the files it lives in.
fn example(name: &str) -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples")
        .join(name)
        .canonicalize()
        .unwrap_or_else(|error| panic!("`{name}` exists: {error}"));
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("the example is readable") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|it| it == "yaml") {
                found.push(path);
            }
        }
    }
    found.sort();
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path
            .strip_prefix(&base)
            .expect("inside the example")
            .display()
            .to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((SpecSource::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("validates");
    compile(&specification, &sources).expect("resolves")
}

fn authoring(ir: &EssIr, text: &str) -> Authoring {
    compile_authored(ir, &[Source::new("scenario.yaml", text)])
}

/// A document in `domain` whose timeline is `acts`.
fn document(domain: &str, acts: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: {domain}\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\ntimeline:\n{acts}"
    )
}

/// One `SendMail` act at `at`, with `rest` appended under it (outcome, error, events).
fn mail(at: &str, recipient: &str, attempt: u32, rest: &str) -> String {
    format!(
        "  - at: 2026-01-05T09:00:{at}Z\n    command: courier.mail.SendMail\n    input:\n      \
         recipient: {recipient}\n      attempt: {attempt}\n{rest}"
    )
}

/// The steps, each as `tag` or `tag:<forced outcome>` for a configured answer.
fn shape(authoring: &Authoring) -> Vec<String> {
    assert!(authoring.is_complete(), "{:?}", authoring.refusals);
    let scenario = authoring.scenarios.values().next().expect("one scenario");
    scenario
        .steps
        .iter()
        .map(|step| match step {
            ScenarioStep::ConfigureExternalOutcome { force, times } => {
                assert_eq!(*times, None, "one direct call is forced once");
                format!("configure:{force}")
            }
            ScenarioStep::ExecuteCommand { .. } => "execute".to_owned(),
            ScenarioStep::ExpectOutcome { outcome } => format!("outcome:{outcome}"),
            ScenarioStep::ExpectError { error, .. } => format!("error:{error}"),
            ScenarioStep::ExpectEvent { event, .. } => format!("event:{event}"),
            other => panic!("unexpected step {other:?}"),
        })
        .collect()
}

/// The codes and texts a document is refused with.
fn refusals(ir: &EssIr, text: &str) -> Vec<(String, String)> {
    let authoring = authoring(ir, text);
    assert!(
        authoring.scenarios.is_empty(),
        "a refused document produced a scenario"
    );
    authoring
        .refusals
        .iter()
        .map(|refusal| (refusal.code().to_string(), refusal.to_string()))
        .collect()
}

const SEND_FAILED: &str = "  - at: 2026-01-05T09:00:00Z\n    command: billing.email.SendEmail\n    \
     input:\n      recipient: nobody@example.test\n      template: welcome\n    outcome: failed\n    \
     error:\n      name: billing.email.Undeliverable\n";

#[test]
fn an_act_expecting_an_external_branch_configures_that_answer_before_the_call() {
    let ir = example("billing");
    let compiled = authoring(&ir, &document("billing.email", SEND_FAILED));
    assert_eq!(
        shape(&compiled),
        vec![
            "configure:billing.email.SendEmail/failed",
            "execute",
            "outcome:billing.email.SendEmail/failed",
            "error:billing.email.Undeliverable",
        ]
    );
}

#[test]
fn a_target_following_the_scenario_answers_the_external_branch() {
    let ir = example("billing");
    let compiled = authoring(&ir, &document("billing.email", SEND_FAILED));
    assert!(compiled.is_complete(), "{:?}", compiled.refusals);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
    for (id, scenario) in compiled.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).expect("admitted");
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(&admitted, &ess_conformance::reference::Billing::new())
        .into_report();
    assert_eq!(
        report.status,
        ess_conformance::report::ConformanceStatus::Passed,
        "the reference target answers `sent` unless told otherwise: {report:?}"
    );
}

#[test]
fn each_external_act_forces_the_branch_it_names_and_no_other_act_is_forced() {
    // `failed` and `bounced` report the same error: the answer forced is the branch written, not the
    // first external branch that fits. Every external act gets its own control, and an accepting act
    // between them gets none — a control left armed would be spent on the wrong call.
    let ir = fixture(COURIER);
    let acts = [
        mail("00", "one@example.test", 1, "    outcome: bounced\n"),
        mail("01", "two@example.test", 2, "    outcome: sent\n"),
        mail("02", "three@example.test", 3, "    outcome: failed\n"),
        mail("03", "four@example.test", 4, "    outcome: queued\n"),
        mail("04", "five@example.test", 11, "    outcome: slowed\n"),
        mail("05", "six@example.test", 5, "    outcome: throttled\n"),
    ]
    .concat();
    let compiled = authoring(&ir, &document("courier.mail", &acts));
    assert_eq!(
        shape(&compiled),
        vec![
            "configure:courier.mail.SendMail/bounced",
            "execute",
            "outcome:courier.mail.SendMail/bounced",
            "execute",
            "outcome:courier.mail.SendMail/sent",
            "configure:courier.mail.SendMail/failed",
            "execute",
            "outcome:courier.mail.SendMail/failed",
            "configure:courier.mail.SendMail/queued",
            "execute",
            "outcome:courier.mail.SendMail/queued",
            "execute",
            "outcome:courier.mail.SendMail/slowed",
            "configure:courier.mail.SendMail/throttled",
            "execute",
            "outcome:courier.mail.SendMail/throttled",
        ]
    );
}

#[test]
fn an_act_naming_a_non_external_branch_compiles_as_before() {
    // No control for a branch the input decides; the committed billing suite's bytes are checked
    // by `the_committed_billing_suite_holds_the_authored_scenario_beside_the_generated_ones`.
    let ir = fixture(COURIER);
    let acts = mail(
        "00",
        "one@example.test",
        1,
        "    outcome: sent\n    events:\n      - event: courier.mail.MailSent\n        \
         payload: {recipient: one@example.test}\n",
    );
    let compiled = authoring(&ir, &document("courier.mail", &acts));
    assert_eq!(
        shape(&compiled),
        vec![
            "execute",
            "outcome:courier.mail.SendMail/sent",
            "event:courier.mail.MailSent",
        ]
    );
}

#[test]
fn an_error_only_an_external_branch_reports_is_refused_without_the_branch_named() {
    let ir = example("billing");
    let text = document(
        "billing.email",
        &SEND_FAILED.replace("    outcome: failed\n", ""),
    );
    let refused = refusals(&ir, &text);
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert_eq!(refused[0].0, "ESS-AUTHOR-037");
    assert!(
        refused[0].1.contains("`billing.email.SendEmail/failed`"),
        "the branch is named: {}",
        refused[0].1
    );
}

#[test]
fn every_external_branch_that_could_answer_is_named() {
    // `failed` and `bounced` both report `Undeliverable`; naming only the first would send the
    // author to one of two answers the claim could mean.
    let ir = fixture(COURIER);
    let acts = mail(
        "00",
        "one@example.test",
        1,
        "    error:\n      name: courier.mail.Undeliverable\n",
    );
    let refused = refusals(&ir, &document("courier.mail", &acts));
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert_eq!(refused[0].0, "ESS-AUTHOR-037");
    assert!(
        refused[0].1.contains("`courier.mail.SendMail/failed`"),
        "{}",
        refused[0].1
    );
    assert!(
        refused[0].1.contains("`courier.mail.SendMail/bounced`"),
        "{}",
        refused[0].1
    );
    assert!(
        !refused[0].1.contains("SendMail/throttled"),
        "{}",
        refused[0].1
    );
}

#[test]
fn an_event_only_an_external_branch_publishes_is_refused_without_the_branch_named() {
    let ir = fixture(COURIER);
    let acts = mail(
        "00",
        "one@example.test",
        1,
        "    events:\n      - event: courier.mail.MailQueued\n",
    );
    let refused = refusals(&ir, &document("courier.mail", &acts));
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert_eq!(refused[0].0, "ESS-AUTHOR-037");
    assert!(
        refused[0].1.contains("`courier.mail.SendMail/queued`"),
        "{}",
        refused[0].1
    );
    assert!(refused[0].1.contains("MailQueued"), "{}", refused[0].1);
}

#[test]
fn every_unanswerable_claim_of_one_act_is_refused() {
    // An error and an event, both external-only, on one act: two refusals, not the first alone.
    let ir = fixture(COURIER);
    let acts = mail(
        "00",
        "one@example.test",
        1,
        "    error:\n      name: courier.mail.Undeliverable\n    events:\n      \
         - event: courier.mail.MailQueued\n",
    );
    let refused = refusals(&ir, &document("courier.mail", &acts));
    let codes: Vec<&str> = refused.iter().map(|(code, _)| code.as_str()).collect();
    assert_eq!(
        codes,
        vec!["ESS-AUTHOR-037", "ESS-AUTHOR-037"],
        "{refused:?}"
    );
}

#[test]
fn a_claim_a_decided_branch_can_also_satisfy_is_not_refused() {
    // `Throttled` is reported by the input-decided `slowed` as well as by the external
    // `throttled`, and `MailSent` by the accepting `sent`: the input decides these, so there is
    // nothing to configure and nothing to refuse.
    let ir = fixture(COURIER);
    let acts = [
        mail(
            "00",
            "one@example.test",
            11,
            "    error:\n      name: courier.mail.Throttled\n",
        ),
        mail(
            "01",
            "two@example.test",
            1,
            "    events:\n      - event: courier.mail.MailSent\n",
        ),
    ]
    .concat();
    let compiled = authoring(&ir, &document("courier.mail", &acts));
    assert_eq!(
        shape(&compiled),
        vec![
            "execute",
            "error:courier.mail.Throttled",
            "execute",
            "event:courier.mail.MailSent",
        ]
    );
}

#[test]
fn a_named_external_branch_with_its_claims_is_not_refused() {
    let ir = fixture(COURIER);
    let acts = mail(
        "00",
        "one@example.test",
        1,
        "    outcome: queued\n    events:\n      - event: courier.mail.MailQueued\n        \
         payload: {recipient: one@example.test}\n",
    );
    let compiled = authoring(&ir, &document("courier.mail", &acts));
    assert_eq!(
        shape(&compiled),
        vec![
            "configure:courier.mail.SendMail/queued",
            "execute",
            "outcome:courier.mail.SendMail/queued",
            "event:courier.mail.MailQueued",
        ]
    );
}

#[test]
fn a_coverage_inventory_carries_the_refusal_and_is_admitted() {
    use ess_conformance::{
        coverage::{Origins, Scope},
        coverage_build::{build, CoverageSource},
    };
    let ir = example("billing");
    let refused = document(
        "billing.email",
        &SEND_FAILED.replace("    outcome: failed\n", ""),
    );
    let accepted = document("billing.email", SEND_FAILED).replace("a-scenario", "b-scenario");
    let input = build(
        &ir,
        &[
            CoverageSource::new("a.yaml", refused).unwrap(),
            CoverageSource::new("b.yaml", accepted).unwrap(),
        ],
        Scope::System,
        Origins::Authored,
    )
    .expect("an inventory with an ESS-AUTHOR-037 refusal is admitted");
    let inventory = input.selected().coverage().unwrap();
    let codes: Vec<&str> = inventory.refused.iter().map(|r| r.code.as_str()).collect();
    assert_eq!(codes, vec!["ESS-AUTHOR-037"]);
    ess_conformance::AdmittedSuite::from_json(input.selected().original_json())
        .expect("the written inventory reads back");
}

/// A chain of two escalating bindings, `Place` → `Notify` → `Forward`, each invoked command with
/// two external branches that report one error, and an own external branch on `Place` whose event
/// sets off a third binding. Every value a reader could confuse differs: no two branches, errors or
/// events share a name across commands.
const RELAY: &str = r"
format: ess/16
system: relay
version: v1
domain: relay.flow
events:
  - name: relay.flow.Placed
    fields:
      - {name: ref, type: String}
  - name: relay.flow.Rerouted
    fields:
      - {name: ref, type: String}
  - name: relay.flow.Notified
    fields:
      - {name: ref, type: String}
  - name: relay.flow.Forwarded
    fields:
      - {name: ref, type: String}
  - name: relay.flow.Archived
    fields:
      - {name: ref, type: String}
  - name: relay.flow.NotifyEscalated
    fields: []
  - name: relay.flow.ForwardEscalated
    fields: []
errors:
  - name: relay.flow.Lost
    fields: []
  - name: relay.flow.Dropped
    fields: []
commands:
  - name: relay.flow.Place
    input:
      - {name: ref, type: String}
    outcomes:
      - name: placed
        emits: [relay.flow.Placed]
        payload:
          relay.flow.Placed:
            ref: input.ref
      - name: rerouted
        external: the carrier reroutes the parcel
        emits: [relay.flow.Rerouted]
        payload:
          relay.flow.Rerouted:
            ref: input.ref
  - name: relay.flow.Notify
    input:
      - {name: note, type: String}
    outcomes:
      - name: notified
        emits: [relay.flow.Notified]
        payload:
          relay.flow.Notified:
            ref: input.note
      - name: lost
        external: the notifier loses the note
        error: relay.flow.Lost
      - name: deferred
        external: the notifier defers the note past its deadline
        error: relay.flow.Lost
  - name: relay.flow.Forward
    input:
      - {name: parcel, type: String}
    outcomes:
      - name: forwarded
        emits: [relay.flow.Forwarded]
        payload:
          relay.flow.Forwarded:
            ref: input.parcel
      - name: dropped
        external: the next hop drops the parcel
        error: relay.flow.Dropped
      - name: stalled
        external: the next hop stalls
        error: relay.flow.Dropped
  - name: relay.flow.Archive
    input:
      - {name: label, type: String}
    outcomes:
      - name: archived
        emits: [relay.flow.Archived]
        payload:
          relay.flow.Archived:
            ref: input.label
bindings:
  - id: notify-on-placed
    summary: Tell someone a parcel was placed.
    when:
      event: relay.flow.Placed
    invoke:
      command: relay.flow.Notify
    mapping:
      note: event.ref
    delivery: at_least_once
    on_failure:
      escalate:
        emits: relay.flow.NotifyEscalated
  - id: forward-on-notified
    summary: Forward a parcel once someone was told.
    when:
      event: relay.flow.Notified
    invoke:
      command: relay.flow.Forward
    mapping:
      parcel: event.ref
    delivery: at_least_once
    on_failure:
      escalate:
        emits: relay.flow.ForwardEscalated
  - id: archive-on-rerouted
    summary: Archive a rerouted parcel.
    when:
      event: relay.flow.Rerouted
    invoke:
      command: relay.flow.Archive
    mapping:
      label: event.ref
    delivery: at_least_once
    on_failure: retry
";

/// One `Place` act, with `rest` appended under it.
fn place(rest: &str) -> String {
    format!(
        "  - at: 2026-01-05T09:00:00Z\n    command: relay.flow.Place\n    input:\n      \
         ref: parcel-7\n{rest}"
    )
}

/// The 037 refusals of `text`, each as its text; any other refusal fails the test.
fn external_refusals(ir: &EssIr, text: &str) -> Vec<String> {
    let refused = refusals(ir, text);
    assert!(
        refused.iter().all(|(code, _)| code == "ESS-AUTHOR-037"),
        "only ESS-AUTHOR-037: {refused:?}"
    );
    refused.into_iter().map(|(_, text)| text).collect()
}

fn names_exactly(text: &str, named: &[&str], unnamed: &[&str]) {
    for branch in named {
        assert!(
            text.contains(&format!("`{branch}`")),
            "names `{branch}`: {text}"
        );
    }
    for branch in unnamed {
        assert!(!text.contains(branch), "does not name `{branch}`: {text}");
    }
}

#[test]
fn an_escalation_two_bindings_downstream_is_refused_naming_every_branch_that_fails() {
    // `ForwardEscalated` is published only when `Forward`, invoked by a binding that `Notify`'s
    // accepting branch sets off, which is itself invoked by a binding on `Placed`, answers one of
    // its external branches. `outcome: placed` is written, so the check runs anyway.
    let ir = fixture(RELAY);
    let refused = external_refusals(
        &ir,
        &document(
            "relay.flow",
            &place(
                "    outcome: placed\n    events:\n      - event: relay.flow.ForwardEscalated\n",
            ),
        ),
    );
    assert_eq!(refused.len(), 1, "{refused:?}");
    names_exactly(
        &refused[0],
        &["relay.flow.Forward/dropped", "relay.flow.Forward/stalled"],
        &["Notify/", "Place/"],
    );
}

#[test]
fn every_downstream_escalation_claim_is_refused_on_its_own() {
    let ir = fixture(RELAY);
    let refused = external_refusals(
        &ir,
        &document(
            "relay.flow",
            &place(
                "    events:\n      - event: relay.flow.NotifyEscalated\n      \
                 - event: relay.flow.ForwardEscalated\n",
            ),
        ),
    );
    assert_eq!(refused.len(), 2, "{refused:?}");
    names_exactly(
        &refused[0],
        &["relay.flow.Notify/lost", "relay.flow.Notify/deferred"],
        &["Forward/"],
    );
    names_exactly(
        &refused[1],
        &["relay.flow.Forward/dropped", "relay.flow.Forward/stalled"],
        &["Notify/"],
    );
}

#[test]
fn a_no_events_claim_only_a_downstream_external_answer_satisfies_is_refused() {
    // `Forwarded` follows `placed` through two bindings whose invoked commands' decided branches
    // both carry it on; it stays unpublished only if `Notify` or `Forward` answers externally, or
    // `Place` answers its own external `rerouted` in place of the written `placed`.
    let ir = fixture(RELAY);
    let refused = external_refusals(
        &ir,
        &document(
            "relay.flow",
            &place("    outcome: placed\n    no_events:\n      - relay.flow.Forwarded\n"),
        ),
    );
    assert_eq!(refused.len(), 1, "{refused:?}");
    names_exactly(
        &refused[0],
        &[
            "relay.flow.Place/rerouted",
            "relay.flow.Notify/lost",
            "relay.flow.Notify/deferred",
            "relay.flow.Forward/dropped",
            "relay.flow.Forward/stalled",
        ],
        &["Place/placed", "Notify/notified", "Forward/forwarded"],
    );
}

#[test]
fn a_no_events_claim_on_the_acts_own_command_names_its_external_branches() {
    // No outcome: `placed` is the one branch the input decides, and it publishes `Placed`.
    let ir = fixture(RELAY);
    let refused = external_refusals(
        &ir,
        &document(
            "relay.flow",
            &place("    no_events:\n      - relay.flow.Placed\n"),
        ),
    );
    assert_eq!(refused.len(), 1, "{refused:?}");
    names_exactly(
        &refused[0],
        &["relay.flow.Place/rerouted"],
        &["Notify/", "Forward/"],
    );
}

#[test]
fn an_event_a_binding_publishes_after_an_own_external_branch_is_refused_unless_armed() {
    let ir = fixture(RELAY);
    let refused = external_refusals(
        &ir,
        &document(
            "relay.flow",
            &place("    events:\n      - event: relay.flow.Archived\n"),
        ),
    );
    assert_eq!(refused.len(), 1, "{refused:?}");
    names_exactly(
        &refused[0],
        &["relay.flow.Place/rerouted"],
        &["Notify/", "Forward/"],
    );

    let armed = authoring(
        &ir,
        &document(
            "relay.flow",
            &place("    outcome: rerouted\n    events:\n      - event: relay.flow.Archived\n"),
        ),
    );
    assert!(armed.is_complete(), "{:?}", armed.refusals);
}

#[test]
fn claims_decided_downstream_are_not_refused() {
    let ir = fixture(RELAY);
    for rest in [
        "    outcome: placed\n    events:\n      - event: relay.flow.Forwarded\n",
        "    events:\n      - event: relay.flow.Notified\n",
        "    outcome: placed\n    no_events:\n      - relay.flow.Archived\n",
    ] {
        let compiled = authoring(&ir, &document("relay.flow", &place(rest)));
        assert!(compiled.is_complete(), "{rest}: {:?}", compiled.refusals);
    }
}

#[test]
fn a_decided_outcome_does_not_exempt_an_external_only_error_or_event() {
    let ir = fixture(COURIER);
    let acts = mail(
        "00",
        "one@example.test",
        1,
        "    outcome: sent\n    error:\n      name: courier.mail.Undeliverable\n    events:\n      \
         - event: courier.mail.MailQueued\n",
    );
    let refused = external_refusals(&ir, &document("courier.mail", &acts));
    assert_eq!(refused.len(), 2, "{refused:?}");
    names_exactly(
        &refused[0],
        &[
            "courier.mail.SendMail/failed",
            "courier.mail.SendMail/bounced",
        ],
        &["SendMail/queued", "SendMail/throttled"],
    );
    names_exactly(
        &refused[1],
        &["courier.mail.SendMail/queued"],
        &["SendMail/failed", "SendMail/bounced"],
    );
}

/// A command whose one input-decided branch returns nothing, and whose external branch returns.
const LOOKUP: &str = r"format: ess/17
system: lookup
version: v1
domain: lookup.api
events:
  - name: lookup.api.Deferred
    fields:
      - {name: key, type: String}
commands:
  - name: lookup.api.Fetch
    input:
      - {name: key, type: String}
    response:
      - {name: value, type: String}
    outcomes:
      - name: deferred
        emits: [lookup.api.Deferred]
        payload:
          lookup.api.Deferred:
            key: input.key
      - name: fetched
        external: the upstream answers in time
        returns: true
";

#[test]
fn a_response_claim_only_an_external_branch_returns_is_refused() {
    let ir = fixture(LOOKUP);
    let text = "type: ess-scenario/4\ndomain: lookup.api\nscenario: a-scenario\n\
                summary: What this scenario proves, in one line.\ntimeline:\n  \
                - at: 2026-01-05T09:00:00Z\n    command: lookup.api.Fetch\n    input:\n      \
                key: k-1\n    response: {}\n";
    let refused = external_refusals(&ir, text);
    assert_eq!(refused.len(), 1, "{refused:?}");
    names_exactly(
        &refused[0],
        &["lookup.api.Fetch/fetched"],
        &["Fetch/deferred"],
    );
}
