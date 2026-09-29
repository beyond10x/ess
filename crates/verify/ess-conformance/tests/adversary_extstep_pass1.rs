//! Adversary pass 1 on `story:authored-external-steps-state-their-answer` (beyond10x/ess#243).
//!
//! Acceptance: "a step expecting an external branch without a stated answer is refused by `ess
//! author` with the branch named". Each case below writes an act whose claim only an `external:`
//! branch can satisfy, and requires one of the two outcomes the story allows: the document is
//! refused with `ESS-AUTHOR-037`, or the compiled scenario is one the reference target — which
//! answers every external branch with its accepting sibling unless told otherwise — passes.
#![allow(clippy::too_many_lines)]

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_conformance::{ConformanceSuite, ScenarioStep, SuiteProvenance};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source as SpecSource;
use std::path::{Path, PathBuf};

fn fixture(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture is well formed");
    let specification = Specification::assemble([(SpecSource::new("fixture.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

fn billing() -> EssIr {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../examples/billing")
        .canonicalize()
        .expect("billing exists");
    let mut found: Vec<PathBuf> = Vec::new();
    let mut pending = vec![base.clone()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("readable") {
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
        let label = path.strip_prefix(&base).unwrap().display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text).expect("well formed");
        sources.insert(label.clone(), text);
        parsed.push((SpecSource::new(label), raw));
    }
    let specification = Specification::assemble(parsed).expect("validates");
    compile(&specification, &sources).expect("resolves")
}

fn document(domain: &str, acts: &str) -> String {
    format!(
        "type: ess-scenario/1\ndomain: {domain}\nscenario: a-scenario\n\
         summary: What this scenario proves, in one line.\ntimeline:\n{acts}"
    )
}

fn codes(authoring: &Authoring) -> Vec<String> {
    authoring
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal}", refusal.code()))
        .collect()
}

fn configures(authoring: &Authoring) -> Vec<String> {
    authoring
        .scenarios
        .values()
        .flat_map(|scenario| scenario.steps.iter())
        .filter_map(|step| match step {
            ScenarioStep::ConfigureExternalOutcome { force, .. } => Some(force.to_string()),
            _ => None,
        })
        .collect()
}

/// Either refused with `ESS-AUTHOR-037`, or a scenario the billing reference target passes.
fn refused_or_satisfiable_on_billing(ir: &EssIr, text: &str) {
    let authoring = compile_authored(ir, &[Source::new("scenario.yaml", text)]);
    let refused = codes(&authoring);
    if refused.iter().any(|it| it.starts_with("ESS-AUTHOR-037")) {
        return;
    }
    assert!(
        authoring.is_complete(),
        "refused for another reason: {refused:?}"
    );
    let configured = configures(&authoring);
    let mut suite = ConformanceSuite::new(SuiteProvenance::of(ir));
    for (id, scenario) in authoring.scenarios {
        suite.insert(id, scenario).expect("one id");
    }
    let admitted = ess_conformance::AdmittedSuite::from_suite(&suite).expect("admitted");
    let report = ess_conformance::Runner::for_suite(&suite)
        .run_admitted(&admitted, &ess_conformance::reference::Billing::new())
        .into_report();
    assert_eq!(
        report.status,
        ess_conformance::report::ConformanceStatus::Passed,
        "compiled without ESS-AUTHOR-037, with external answers configured {configured:?}, and the \
         reference target cannot satisfy it: {report:?}"
    );
}

/// `DeliveryEscalated` is published only when the binding's `SendEmail` fails, and `SendEmail`
/// fails only on its `external:` branch `failed`. The act invokes `CreateInvoice`, so its
/// `outcome:` cannot name `SendEmail/failed`; the claim expects an external answer nothing states.
/// `unstated_external` counts a binding's escalation as "decided elsewhere" and so never refuses it.
#[test]
fn an_escalation_only_a_downstream_external_failure_publishes_is_refused_or_armed() {
    let ir = billing();
    let text = document(
        "billing.invoice",
        "  - at: 2026-01-05T09:00:00Z\n    command: billing.invoice.CreateInvoice\n    \
         actor: billing.invoice.Customer\n    input:\n      \
         account_id: 00000000-0000-4000-8000-000000000001\n      \
         customer_email: buyer@example.test\n      amount: {amount: 10, currency: EUR}\n    \
         outcome: accepted\n    events:\n      - event: billing.email.DeliveryEscalated\n",
    );
    refused_or_satisfiable_on_billing(&ir, &text);
}

/// The same claim with no `outcome:` written, so `unstated_external` does run on it.
#[test]
fn an_escalation_claim_on_an_act_with_no_outcome_is_refused_or_armed() {
    let ir = billing();
    let text = document(
        "billing.invoice",
        "  - at: 2026-01-05T09:00:00Z\n    command: billing.invoice.CreateInvoice\n    \
         actor: billing.invoice.Customer\n    input:\n      \
         account_id: 00000000-0000-4000-8000-000000000001\n      \
         customer_email: buyer@example.test\n      amount: {amount: 10, currency: EUR}\n    \
         events:\n      - event: billing.email.DeliveryEscalated\n",
    );
    refused_or_satisfiable_on_billing(&ir, &text);
}

/// `SendEmail`'s only branch the input decides, `sent`, publishes `EmailSent`. An act naming no
/// branch that claims `EmailSent` is *not* published expects `failed` — an external answer — and
/// states none. `unstated_external` reads `error:` and `events:`, never `no_events:`.
#[test]
fn a_no_events_claim_only_an_external_branch_satisfies_is_refused_or_armed() {
    let ir = billing();
    let text = document(
        "billing.email",
        "  - at: 2026-01-05T09:00:00Z\n    command: billing.email.SendEmail\n    \
         input:\n      recipient: nobody@example.test\n      template: welcome\n    \
         no_events:\n      - billing.email.EmailSent\n",
    );
    refused_or_satisfiable_on_billing(&ir, &text);
}

/// `MailQueued` is published by `SendMail` only on its external `queued`; another command,
/// `QueueMail`, publishes it on a decided branch, but the act never invokes `QueueMail` and no
/// binding connects the two. The event check asks whether *any* command's decided branch emits
/// the event, not whether anything this act sets off does, so the claim compiles unarmed.
#[test]
fn an_event_decided_only_by_a_command_the_act_never_reaches_is_still_refused() {
    let ir = fixture(
        r"
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
commands:
  - name: courier.mail.SendMail
    input:
      - {name: recipient, type: String}
    outcomes:
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
  - name: courier.mail.QueueMail
    input:
      - {name: address, type: String}
    outcomes:
      - name: queued
        emits: [courier.mail.MailQueued]
        payload:
          courier.mail.MailQueued:
            recipient: input.address
",
    );
    let text = document(
        "courier.mail",
        "  - at: 2026-01-05T09:00:00Z\n    command: courier.mail.SendMail\n    input:\n      \
         recipient: one@example.test\n    events:\n      - event: courier.mail.MailQueued\n",
    );
    let authoring = compile_authored(&ir, &[Source::new("scenario.yaml", &text)]);
    let refused = codes(&authoring);
    let configured = configures(&authoring);
    assert!(
        refused.iter().any(|it| it.starts_with("ESS-AUTHOR-037"))
            || configured == vec!["courier.mail.SendMail/queued".to_owned()],
        "an act on SendMail claiming MailQueued, which only SendMail/queued (external) can \
         publish for it, compiled with refusals {refused:?} and configured {configured:?}"
    );
}
