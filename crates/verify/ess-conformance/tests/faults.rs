//! Does the suite catch anything?
//!
//! `tests/execution.rs` shows that 29 generated scenarios pass against an implementation written by
//! hand from the same specification. That is the *satisfiability* claim, and it is the weaker half:
//! a suite that asserted nothing would pass exactly as green.
//!
//! This is the other half — design §25 and §26. Every test below runs a **deliberately wrong**
//! implementation and asks which named scenario noticed. Two properties make the matrix worth
//! anything, and both are asserted mechanically rather than read off a table:
//!
//! 1. each fault fails **the one scenario that exists to catch it** — not "the run went red", which
//!    a single panic would also achieve;
//! 2. a fault **does not simply break everything**, held to a per-fault allowance that has to be
//!    changed with a reason rather than relaxed.
//!
//! The matrix prints itself:
//!
//! ```console
//! cargo test -p ess-conformance --test faults -- --nocapture a_faults_blast_radius_is_accounted_for
//! ```
//!
//! # The rows worth reading are the ones nothing catches, and every one recorded has been closed
//!
//! A fault in [`Fault::ALL`] marked [`Caught::Nothing`] is not a hole in this file; it is the
//! finding, and [`a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped`] asserts that a
//! **correct** run still comes back for it — so the day the gap closes, the row fails here and has
//! to be rewritten rather than forgotten. That is what happened to all three that were recorded:
//! `extra-event` and `drop-consistency-token` were closed by teaching synthesis to ask for more,
//! and `wrong-event-payload` — the one that separated what a specification *declares* from what it
//! merely *names* — needed the model itself to change. It publishes `InvoicePaid` with an amount
//! nobody submitted; every field the event declares is present and well-typed, and until an
//! outcome could say `amount: input.amount` there was no check to make that was not a guess. The
//! wave 6.5 `payload:` construct is that declaration, and the row moved. `partial-event-payload`
//! is the same event with a declared field missing, caught all along, because the type was always
//! declared. `crates/verify/ess-conformance/src/faulty.rs` argues both.
//!
//! `wrong-refusal-error` is the row that arrived the other way round. It was uncatchable and is not
//! recorded as such, because the repair landed in the same change: a command can now declare what it
//! answers when its subject is in a state its moves do not start from, so an implementation that
//! refuses with the wrong error is a defect a named scenario reports rather than a difference no
//! assertion could see.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Caught, Fault, Injection, System};
use ess_conformance::history::{Completion, Verdict};
use ess_conformance::linearize;
use ess_conformance::record::{self, Atomic, Call, Subject};
use ess_conformance::reference::{Billing, Oracle, Retained, Untraced};
use ess_conformance::report::{CheckCode, ConformanceReport, ConformanceStatus, Status};
use ess_conformance::runner::Runner;
use ess_conformance::scenario::ConformanceSuite;
use ess_conformance::sessions::{self, Act, FaultInjection};
use ess_conformance::synthesize::synthesize;
use ess_conformance::target::{
    ConformanceTarget, Deadline, ImplementationIdentity, InvocationObservationRequest,
};
use ess_conformance::AdmittedSuite;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_primitives::time::Timestamp;

// ---- the specifications under test ---------------------------------------------------------------

/// An example directory, compiled from the files it lives in rather than from a copy inlined here.
fn example(name: &str) -> EssIr {
    model_at(&format!("examples/{name}"))
}

/// A specification directory, by its path from the workspace root, compiled from its files.
fn model_at(path: &str) -> EssIr {
    model_rewritten(path, ToOwned::to_owned)
}

/// [`model_at`], with each file's text passed through `rewrite` before it is parsed.
fn model_rewritten(path: &str, rewrite: impl Fn(&str) -> String) -> EssIr {
    let name = path;
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(path)
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
        let text = rewrite(&std::fs::read_to_string(&path).expect("readable"));
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        sources.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    let specification = Specification::assemble(parsed)
        .unwrap_or_else(|errors| panic!("`{name}` validates:\n{errors}"));
    compile(&specification, &sources)
        .unwrap_or_else(|diagnostics| panic!("`{name}` resolves:\n{diagnostics}"))
}

/// The suite one of the two systems obliges.
fn suite(system: System) -> ConformanceSuite {
    synthesize(&model_at(system.path())).suite
}

/// The report an admitted run of `system`'s current suite against `target` produces.
fn run<T: ConformanceTarget>(system: System, target: &T) -> ConformanceReport {
    let suite = suite(system);
    let admitted = AdmittedSuite::from_suite(&suite).expect("the synthesized suite is admitted");
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
}

/// The report a run against the implementation carrying `fault` produces.
///
/// Which of the two references is wrapped comes from the fault itself, so a row cannot be run
/// against a specification that does not declare what it breaks.
fn injected(fault: Fault) -> ConformanceReport {
    match fault.system() {
        System::Billing => run(fault.system(), &faulty::billing(fault)),
        System::Oracle => run(fault.system(), &faulty::oracle(fault)),
        System::Retry => run(fault.system(), &faulty::retry(fault)),
    }
}

/// Every scenario that did not pass, with the status it came to.
fn not_passed(report: &ConformanceReport) -> Vec<(String, Status)> {
    report
        .failures()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// The status of one named scenario, or `None` when the suite holds no such scenario.
fn status_of(report: &ConformanceReport, scenario: &str) -> Option<Status> {
    report
        .scenarios
        .iter()
        .find(|result| result.scenario.to_string() == scenario)
        .map(|result| result.status)
}

// ---- the control group -----------------------------------------------------------------------

#[test]
fn each_specification_is_passed_in_full_by_the_implementation_written_from_it() {
    // Without this, every row below could be read as "the target is broken in some way", and the
    // matrix would be measuring the reference rather than the suite. Both references, because the
    // oracle fixture is where §26's second claim is made and a fixture nothing passes proves less
    // than nothing.
    for (system, scenarios) in [
        (System::Billing, 32),
        (System::Oracle, 34),
        (System::Retry, RETRY_SCENARIOS),
    ] {
        let report = match system {
            System::Billing => run(system, &Billing::new()),
            System::Oracle => run(system, &Oracle::new()),
            System::Retry => run(system, &Retained::new()),
        };
        assert_eq!(
            report.scenarios.len(),
            scenarios,
            "`{}` obliges a different number of scenarios than the matrix was measured against",
            system.directory()
        );
        assert_eq!(
            report.status,
            ConformanceStatus::Passed,
            "the reference is written from this specification, so a scenario it fails is a defect \
             in one of the two:\n{report}"
        );
    }
}

// ---- property one: each fault fails the scenario that exists to catch it ------------------------

#[test]
fn each_fault_fails_the_scenario_that_exists_to_catch_it() {
    // §25's important invariant, and the reason it is stated in the negative there: "a generic panic
    // that causes the entire suite to fail proves nothing". So this asserts a *named* scenario, and
    // asserts `failed` specifically — `unsupported` would mean nobody found out, which is the
    // degradation `a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations` below
    // shows is real.
    let mut missed: Vec<String> = Vec::new();

    for fault in Fault::ALL {
        let Caught::By(scenario) = fault.caught() else {
            continue;
        };
        let report = injected(*fault);
        match status_of(&report, scenario) {
            None => missed.push(format!(
                "{fault:?} names `{scenario}`, which `{}` does not oblige at all",
                fault.system().directory()
            )),
            Some(Status::Failed) => {}
            Some(status) => missed.push(format!(
                "{fault:?} ({}) left `{scenario}` at `{status}`; scenarios that did not pass: {:?}",
                fault.describe(),
                not_passed(&report)
            )),
        }
    }

    assert!(
        missed.is_empty(),
        "a scenario that does not catch its own fault is not protecting the property it names:\n  \
         {}",
        missed.join("\n  ")
    );
}

#[test]
fn the_diagnostic_of_a_caught_fault_names_the_defect_rather_than_reporting_that_something_broke() {
    // §29: repair feedback, not a red light. The fault is the one whose defect is furthest from the
    // assertion that catches it — a binding reading the wrong field of an event — so the diagnostic
    // has the most work to do.
    let report = injected(Fault::WrongMapping);
    let result = report
        .scenarios
        .iter()
        .find(|result| result.scenario.to_string() == "handoff-on-placed/binding/mapping")
        .expect("the mapping scenario ran");

    let diagnostic = result
        .diagnostics()
        .find(|diagnostic| diagnostic.code == CheckCode::Invocation)
        .expect("the invocation check produced a diagnostic")
        .to_string();
    for required in [
        "ESS-CF-INVOCATION",
        "handoff-on-placed",
        "oracle.dispatch.Handoff",
        "expected:",
        r#"oracle.dispatch.Handoff.recipient = "contact""#,
        r#"recipient = "alternate_contact""#,
    ] {
        assert!(
            diagnostic.contains(required),
            "a wrong mapping is only repairable if the report names the value that was passed and \
             the one that was declared; {required:?} is missing from:\n{diagnostic}"
        );
    }
}

// ---- property two: a fault does not simply break everything -------------------------------------

#[test]
fn a_fault_does_not_simply_break_everything() {
    // If one fault failed every scenario, the scenario boundaries would carry no information at all:
    // a failure would tell an implementer to look everywhere.
    for fault in Fault::ALL {
        let report = injected(*fault);
        let total = report.scenarios.len();
        let broken = not_passed(&report).len();
        assert!(
            broken < total,
            "{fault:?} broke all {total} scenarios, so the report says nothing about where to look"
        );
    }
}

#[test]
fn a_faults_blast_radius_is_accounted_for() {
    // `aep_conformance::tests::faults`'s table, with ESS scenarios where AEP suite names are. Each
    // allowance is a *claim about why* the radius is what it is, so exceeding one means either a
    // scenario is over-reaching or the allowance needs updating with a reason. The default is one:
    // a new fault that touches anything beyond its own scenario has to say so here.
    //
    //   WrongEvent            24  `InvoiceCreated` carries the identity 17 further scenarios are
    //                             arranged from — including the two §20 value-object checks, whose
    //                             rows are put in place by `CreateInvoice` — so renaming it stops
    //                             them being set up at all; see the test below, which pins that
    //                             they come back as `error` rather than as 24 separate verdicts
    //                             about the implementation.
    //   DropBinding            4  the four aspects of the binding that stopped running: its flow,
    //                             its mapping, its delivery and its failure policy. The other two
    //                             bindings' seven scenarios stay green, which is the whole reason
    //                             `examples/oracle-fixture/` exists.
    //   StaleReadYourWrites    9  every scenario whose read-your-writes assertion is *positive* —
    //                             including the §20 value check at `OutstandingInvoices.total`,
    //                             whose first stale read answers an empty view and "every row, and
    //                             at least one" demands a row. A view answering from before the
    //                             write legitimately `Excludes` a row, and used to leave five
    //                             scenarios green on that reading alone: the same stale answer also
    //                             holds fewer rows than the scenario put there, and a count is a
    //                             claim `Excludes` cannot make. That is the floor doing its job, not
    //                             a scenario over-reaching — `OutstandingInvoices` ranks its rows,
    //                             so every scenario that reads it now says how many it arranged.
    //   AllowIllegalTransition 3  billing declares two states `cancel` must not run from, `Paid`
    //                             and `Cancelled`, so one missing guard is two refusals — and the
    //                             third is the invoice no record carries (beyond10x/ess#113): the
    //                             fault answers `cancelled` for it too, which is a move of nothing.
    //   IgnoreExternalOutcome  2  the forced failure is what makes the escalation reachable, so the
    //                             binding's failure policy has nothing to observe either.
    //   DropConsistencyToken   9  every scenario that reads a `read_your_writes` view — the ninth
    //                             is the §20 value check at `OutstandingInvoices.total`. The token
    //                             is missing from *every* command result, so §14's demand can be
    //                             made nowhere; the row designates one of the nine because a matrix
    //                             row names a scenario, not because the other eight are collateral.
    //   ExtraEvent             5  the five scenarios whose asserted command is `CancelInvoice`: its
    //                             own branch, the `cancel` move, the two states that move may not
    //                             run from, and the invoice no record carries — where the stray
    //                             `InvoicePaid` is caught too, because a command nothing honoured
    //                             may publish nothing declared.
    //   PartialEventPayload    2  `PayInvoice/settled` is asserted twice, once as §10's branch and
    //                             once as §19's move, and an event missing a declared field fails
    //                             both.
    //   WrongEventPayload      2  the same two scenarios, for the value where the other row is the
    //                             field: `payload: amount: input.amount` is declared on `settled`,
    //                             so both scenarios that run the branch hold the amount to what was
    //                             submitted. The row designates the transition scenario because the
    //                             outcome scenario already names `PartialEventPayload`, and one
    //                             scenario names one fault.
    //   NegativeProjectedTotal 4  two readings of the same projected value, times two scenarios
    //                             each. The entity's own invariant reads the same words off the
    //                             same view — `Invoice` declares `total.amount >= 0` too — and
    //                             `CreateInvoice.accepted` now says `sets: total: input.amount`,
    //                             so every scenario that runs `issue` also holds the row to the
    //                             amount that was submitted. Both readings fire on the branch
    //                             scenario and on the transition scenario. The checks at
    //                             `InvoiceById.total` stay green, which is the point of keying the
    //                             family by position.
    //   AcceptInvalidAmount    2  the injection rewrites every submitted amount to `1`, not only a
    //                             refused one. `cancel` runs from `Draft` and `Issued`, and the
    //                             `Issued` source is arranged on a second invoice (ess#111) whose
    //                             witness amount is `2` — so the rewritten row no longer holds what
    //                             was submitted, and the transition scenario says so. The row
    //                             designates the refusal, which is the branch the fault names.
    //   WrongRefusalError      4  `issue` runs from `Draft` alone, so `IssueInvoice` answers its
    //                             `wrong_state:` branch in the other three declared states and for
    //                             an invoice no record carries, and one wrong error name is wrong in
    //                             all four. Narrower is not available: the injection sees a command
    //                             and a result, not which invoice.
    //   LostUpdate             0  a race: every scenario runs one call at a time, so nothing happens
    //                             between the read and the write and every answer is the
    //                             reference's. The row is caught by a recorded concurrent history,
    //                             and the test after the next section says so.
    //   StaleReadUnderReadYourWrites
    //                          0  one client's reads always demand that client's newest write,
    //                             which is also the newest write anybody made, and that read is the
    //                             one that refreshes the copy. Only a second client writing between
    //                             the first client's write and its read leaves the copy behind.
    //   DoubleApplyOnRedelivery
    //                          0  the suite delivers `InvoiceCreated` a second time once, right
    //                             after the creation, and asserts only that the mail still goes
    //                             out; the second invoice it leaves is read by no scenario. Only a
    //                             history recorded with the declared faults injected reads
    //                             `InvoiceById` after a second delivery.
    //   RetryCreatesSecondEntity
    //                          0  the suite retries `Seed` only after the original answered, and
    //                             a request answered before is replayed. Only a retry sent while
    //                             its original is in flight finds nothing retained.
    let allowance: &[(Fault, usize)] = &[
        (Fault::WrongEvent, 24),
        (Fault::DropConsistencyToken, 9),
        (Fault::DropBinding, 4),
        (Fault::ExtraEvent, 5),
        (Fault::StaleReadYourWrites, 9),
        (Fault::WrongRefusalError, 4),
        (Fault::AcceptInvalidAmount, 2),
        (Fault::AllowIllegalTransition, 3),
        (Fault::IgnoreExternalOutcome, 2),
        (Fault::PartialEventPayload, 2),
        (Fault::WrongEventPayload, 2),
        (Fault::NegativeProjectedTotal, 4),
        (Fault::LostUpdate, 0),
        (Fault::StaleReadUnderReadYourWrites, 0),
        (Fault::DoubleApplyOnRedelivery, 0),
        (Fault::RetryCreatesSecondEntity, 0),
    ];

    for fault in Fault::ALL {
        let report = injected(*fault);
        let broken = not_passed(&report);
        let allowed = allowance
            .iter()
            .find(|(known, _)| known == fault)
            .map_or(1, |(_, allowed)| *allowed);
        // Printed so that `cargo test -p ess-conformance --test faults -- --nocapture
        // a_faults_blast_radius_is_accounted_for` reads as the matrix itself rather than as eleven
        // green ticks. §26 asks for a table; a table nobody can look at is a table on trust.
        println!(
            "{:<24} {:<14} {:<2} caught by {:<62} {} of {} still pass",
            fault.written(),
            fault.system().directory(),
            broken.len(),
            match fault.caught() {
                Caught::By(scenario) => scenario,
                Caught::ByHistory(_) => "— a concurrent history only",
                Caught::ByInjection(_) => "— a history with declared faults injected only",
                Caught::Nothing(_) => "— nothing",
            },
            report.scenarios.len() - broken.len(),
            report.scenarios.len(),
        );
        assert!(
            broken.len() <= allowed,
            "{fault:?} broke {} scenarios ({broken:?}) but is accounted for {allowed}; either a \
             scenario is over-reaching or the allowance needs updating with a reason",
            broken.len()
        );
    }
}

#[test]
fn the_widest_blast_radius_is_scenarios_that_could_not_be_arranged_rather_than_extra_verdicts() {
    // Why `WrongEvent`'s allowance is 24 and why that is a finding rather than a flaw, the way
    // `aep_conformance` records `DropAffected`'s 8. `billing.invoice.InvoiceCreated` is where a new
    // invoice's identity is published, and `CaptureInstance` reads it in seventeen further
    // scenarios; rename it and those scenarios cannot be set up at all.
    //
    // §28's distinction is what keeps that readable: they come back as `error` — nobody found out —
    // and not as `failed`, which would claim the implementation contradicted the specification
    // nineteen more times.
    let report = injected(Fault::WrongEvent);
    let broken = not_passed(&report);

    let failed = broken
        .iter()
        .filter(|(_, status)| *status == Status::Failed)
        .count();
    let errored = broken
        .iter()
        .filter(|(_, status)| *status == Status::Error)
        .count();
    assert_eq!(
        (failed, errored),
        (5, 19),
        "the split between a verdict about the implementation and a scenario nobody could arrange \
         is the whole reason §28 has four words rather than two: {broken:?}"
    );

    let arrangement = report
        .diagnostics()
        .find(|diagnostic| diagnostic.code == CheckCode::Instance)
        .expect("a scenario that could not be arranged says so")
        .to_string();
    assert!(
        arrangement.contains("carries the new identity"),
        "an `error` has to name what could not be established, or it reads as noise:\n{arrangement}"
    );
}

#[test]
fn dropping_one_binding_leaves_the_other_two_green() {
    // §26 in as many words — "unrelated core scenarios still pass" — and the claim
    // `examples/billing/` cannot make, because it declares one binding and dropping it fails every
    // binding scenario there is. The oracle fixture declares three, on three events, and its
    // `README.md` records that this is what the extra two are for.
    let report = injected(Fault::DropBinding);

    let mut green = 0;
    for result in &report.scenarios {
        let id = result.scenario.to_string();
        if !id.contains("/binding/") {
            continue;
        }
        if id.starts_with(Oracle::HANDOFF_ON_PLACED) {
            assert_eq!(
                result.status,
                Status::Failed,
                "every aspect of the binding that stopped running is unobservable: {id}"
            );
        } else {
            assert_eq!(
                result.status,
                Status::Passed,
                "a binding that still runs must still be provable: {id}"
            );
            green += 1;
        }
    }
    assert_eq!(
        green, 7,
        "the two surviving bindings oblige seven scenarios between them, and all seven are the \
         evidence that a binding failure is attributable rather than systemic"
    );
}

// ---- the rows nothing catches --------------------------------------------------------------------

#[test]
fn a_fault_nothing_catches_is_recorded_rather_than_quietly_dropped() {
    // The most valuable rows in the matrix, and the ones this slice went looking for. Each of these
    // is a wrong implementation that the generated suite passes in full — which is a statement about
    // what the model can express or what synthesis asks for, not about this file.
    //
    // Asserting that they *still* pass is deliberate. The day a later slice teaches synthesis to
    // compare an event's payload against the input that caused it, this test fails, and the row has
    // to be moved to `Caught::By` with the scenario that now catches it. A gap nobody re-checks is a
    // gap that gets forgotten.
    let mut closed: Vec<String> = Vec::new();

    for fault in Fault::ALL {
        let Caught::Nothing(why) = fault.caught() else {
            continue;
        };
        let report = injected(*fault);
        if report.status != ConformanceStatus::Passed {
            closed.push(format!(
                "{fault:?} ({}) is recorded as uncaught because {why}, but {:?} failed — the gap \
                 has been closed and the row belongs on the other side of the matrix",
                fault.describe(),
                not_passed(&report)
            ));
        }
    }

    assert!(closed.is_empty(), "{}", closed.join("\n  "));
}

#[test]
fn a_wrong_mapping_is_invisible_to_a_target_that_cannot_show_its_invocations() {
    // What the matrix does with a fault whose scenario degrades to `unsupported`, and the finding
    // behind the question. §16 refuses to require command tracing of every implementation, and
    // `handoff-on-placed/binding/mapping` is the *only* scenario that catches a wrong mapping — so
    // against a target that legitimately cannot answer it, the whole defect is invisible.
    //
    // Conformance still fails, which is §28 doing its job. What is lost is the diagnostic: the
    // report says "I cannot show you my invocations", never "you mailed the wrong address".
    let mis_mapped = Untraced(faulty::oracle(Fault::WrongMapping));
    let report = run(System::Oracle, &mis_mapped);

    assert_eq!(
        status_of(&report, "handoff-on-placed/binding/mapping"),
        Some(Status::Unsupported),
        "the scenario that catches this fault is the one §16 lets a target refuse"
    );
    assert_eq!(
        report.status,
        ConformanceStatus::Failed,
        "an unsupported required scenario still fails the run (§28)"
    );

    let broken = not_passed(&report);
    assert_eq!(
        broken.len(),
        3,
        "only the three mapping scenarios are unanswerable; the wrong address is not among the \
         findings at all: {broken:?}"
    );
    assert!(
        report
            .diagnostics()
            .all(|diagnostic| !diagnostic.to_string().contains("alternate_contact")),
        "nothing in the report names the value that was actually mapped, which is the cost of \
         having exactly one scenario able to see it"
    );
}

// ---- the row only a concurrent history catches ---------------------------------------------------

#[test]
fn a_fault_only_a_concurrent_history_catches_passes_every_suite_scenario_and_fails_the_history() {
    // The epic's rule for a check that adds a bug class: a planted fault the new check catches and
    // no earlier check catches. Both halves are asserted. The earlier checks are the synthesized
    // suite, every scenario of which passes against the fault; the new one is a two-client history
    // recorded against it and checked against the interpreter. The same recordings against the
    // unfaulted reference are linearizable, so the violation is the fault's and not the workload's.
    let model = example(System::Billing.directory());
    let mut rows = Vec::new();
    for fault in Fault::ALL {
        let Caught::ByHistory(_) = fault.caught() else {
            continue;
        };
        rows.push(*fault);

        let report = injected(*fault);
        assert_eq!(
            report.status,
            ConformanceStatus::Passed,
            "{fault:?} is recorded as caught only by a concurrent history, but the suite caught \
             it: {:?}",
            not_passed(&report)
        );

        let mut violations = 0;
        for seed in 0..24 {
            let target = faulty::billing(*fault);
            let faulted = recorded(&model, *fault, &target, &target, seed);
            let checked =
                linearize::check(&model, &faulted, linearize::DEFAULT_BUDGET).expect("checked");
            if checked.verdict == Verdict::Violation {
                violations += 1;
                if *fault == Fault::StaleReadUnderReadYourWrites {
                    // The acceptance's second half: the violation names the client and the read.
                    let read = checked
                        .read
                        .as_ref()
                        .expect("a stale read is reported as a read violation");
                    let named = faulted
                        .operations
                        .iter()
                        .find(|operation| operation.operation_id.as_str() == read.operation_id)
                        .expect("the named read is in the history");
                    assert_eq!(named.command.as_str(), OUTSTANDING_INVOICES);
                    assert_eq!(named.client, read.client);
                    assert_eq!(read.anomaly, linearize::Anomaly::StaleRead);
                    assert_eq!(read.consistency, "read_your_writes");
                }
            }
            let reference = Billing::new();
            let control = recorded(&model, *fault, &Atomic(&reference), &reference, seed);
            assert_eq!(
                linearize::check(&model, &control, linearize::DEFAULT_BUDGET)
                    .expect("checked")
                    .verdict,
                Verdict::Linearizable,
                "seed {seed}: the reference is linearizable, or the violation is not the fault's"
            );
        }
        println!(
            "{:<24} {:<14} caught by a concurrent history in {violations} of 24 seeds; the suite \
             caught nothing",
            fault.written(),
            fault.system().directory()
        );
        assert!(
            violations > 0,
            "{fault:?}: no recorded two-client history was a violation"
        );
    }
    assert_eq!(
        rows,
        vec![Fault::LostUpdate, Fault::StaleReadUnderReadYourWrites]
    );
}

#[test]
fn the_single_client_stale_read_and_the_concurrent_one_are_two_rows_caught_two_ways() {
    // `stale-read-your-writes` (`F-VIEW-RACE`) answers a demanding read one read behind, which one
    // client already sees; `stale-read-under-read-your-writes` answers from a copy only another
    // client's write leaves behind. Two defects, two rows, two checks.
    assert_eq!(
        Fault::StaleReadYourWrites.caught(),
        Caught::By("billing.invoice.IssueInvoice/outcome/issued")
    );
    assert_eq!(
        status_of(
            &injected(Fault::StaleReadYourWrites),
            "billing.invoice.IssueInvoice/outcome/issued"
        ),
        Some(Status::Failed)
    );
    assert!(matches!(
        Fault::StaleReadUnderReadYourWrites.caught(),
        Caught::ByHistory(_)
    ));
    assert_ne!(
        Fault::StaleReadYourWrites.written(),
        Fault::StaleReadUnderReadYourWrites.written()
    );
}

/// `billing.invoice.OutstandingInvoices`, the `read_your_writes` view.
const OUTSTANDING_INVOICES: &str = "billing.invoice.OutstandingInvoices";

/// The history a row caught only by a concurrent history is recorded as, under `seed`.
///
/// Each such row names its workload: [`Fault::LostUpdate`] races two payments and reads nothing;
/// [`Fault::StaleReadUnderReadYourWrites`] has each client issue an invoice and then read its own
/// list, so it is recorded by the session recorder, which sends its reads to `views`.
fn recorded<T: record::Interleaved, V: ConformanceTarget>(
    model: &EssIr,
    fault: Fault,
    target: &T,
    views: &V,
    seed: u64,
) -> ess_conformance::history::History {
    match fault {
        Fault::LostUpdate => record::record(model, target, &faulty::lost_update_workload(), seed),
        Fault::StaleReadUnderReadYourWrites => {
            sessions::record(model, target, views, &faulty::stale_read_workload(), seed)
        }
        other => panic!("{other:?} names no concurrent workload"),
    }
    .expect("the workload names what billing declares")
}

// ---- the rows only a declared fault shows --------------------------------------------------------

/// How many seeds each row caught only with declared faults injected is recorded under.
const INJECTED_SEEDS: u64 = 24;

/// How many scenarios the retry fixture's suite obliges.
const RETRY_SCENARIOS: usize = 2;

/// The workload a row caught only with declared faults injected is recorded under.
fn injected_workload(fault: Fault) -> sessions::Workload {
    match fault {
        Fault::DoubleApplyOnRedelivery => faulty::double_apply_workload(),
        Fault::RetryCreatesSecondEntity => faulty::retry_workload(),
        other => panic!("{other:?} names no workload recorded with injection"),
    }
}

/// The histories one seed records for a row caught only with declared faults injected: against
/// the faulty target without injection, against it with injection, and against the reference with
/// injection.
fn three_recordings(
    model: &EssIr,
    fault: Fault,
    seed: u64,
) -> (sessions::Recorded, sessions::Recorded, sessions::Recorded) {
    let workload = injected_workload(fault);
    match fault.system() {
        System::Billing => {
            let faulted = |injection| {
                let target = faulty::billing(fault);
                sessions::record_with(model, &target, &target, &workload, seed, injection)
                    .expect("the workload names what billing declares")
            };
            let reference = |injection| {
                let target = Billing::new();
                sessions::record_with(model, &Atomic(&target), &target, &workload, seed, injection)
                    .expect("the workload names what billing declares")
            };
            (
                faulted(FaultInjection::None),
                faulted(FaultInjection::Declared),
                reference(FaultInjection::Declared),
            )
        }
        System::Retry => {
            let faulted = |injection| {
                let target = faulty::retry(fault);
                sessions::record_with(model, &target, &target, &workload, seed, injection)
                    .expect("the workload names what the retry fixture declares")
            };
            let reference = |injection| {
                let target = Retained::new();
                sessions::record_with(model, &Atomic(&target), &target, &workload, seed, injection)
                    .expect("the workload names what the retry fixture declares")
            };
            (
                faulted(FaultInjection::None),
                faulted(FaultInjection::Declared),
                reference(FaultInjection::Declared),
            )
        }
        System::Oracle => panic!("{fault:?}: no oracle fault is recorded with injection"),
    }
}

/// Adds every count of `more` to `into`.
fn add(into: &mut sessions::Injected, more: &sessions::Injected) {
    for (total, counts) in [
        (&mut into.redeliveries, &more.redeliveries),
        (&mut into.refused, &more.refused),
        (&mut into.retries, &more.retries),
        (&mut into.delayed, &more.delayed),
        (&mut into.unanswered, &more.unanswered),
        (&mut into.reached, &more.reached),
    ] {
        for (key, count) in counts {
            *total.entry(key.clone()).or_default() += count;
        }
    }
}

/// Asserts that the violation `checked` found in `history` is the defect `fault` plants, and not
/// some other one.
fn the_violation_is_the_faults(
    fault: Fault,
    history: &ess_conformance::history::History,
    checked: &linearize::Checked,
) {
    match fault {
        Fault::DoubleApplyOnRedelivery => {
            // The witness is a read listing the invoice nobody created.
            let read = checked
                .read
                .as_ref()
                .expect("a read shows the second invoice");
            assert_eq!(read.anomaly, linearize::Anomaly::FutureRead);
            assert!(read.shown);
            assert!(
                history
                    .operations
                    .iter()
                    .all(|operation| operation.subject_key != read.subject_key),
                "no recorded call created {}",
                read.subject_key
            );
        }
        Fault::RetryCreatesSecondEntity => {
            // One request, two operations, both answering `seeded`.
            assert!(checked.read.is_none());
            let named: Vec<_> = history
                .operations
                .iter()
                .filter(|operation| {
                    checked
                        .linearization
                        .contains(&operation.operation_id.as_str().to_owned())
                })
                .collect();
            assert_eq!(named.len(), 2, "{checked:?}");
            assert!(named.iter().any(|operation| operation.retry_of.is_some()));
            assert!(named.iter().all(|operation| operation
                .outcome
                .as_ref()
                .map(ess_conformance::history::QualifiedName::as_str)
                == Some("seeded")));
        }
        other => panic!("{other:?} is not recorded with injection"),
    }
}

#[test]
fn a_fault_only_a_declared_fault_shows_passes_the_suite_and_uninjected_histories_and_fails_an_injected_one(
) {
    // The story's acceptance, both halves per row: found by the concurrent recorder with the
    // declared faults injected, missed by the same seeds without injection — and by every suite
    // scenario. The reference, recorded with the same injections, is linearizable under every
    // seed, so the violation is the fault's and not the injection's.
    let mut rows = Vec::new();
    for fault in Fault::ALL {
        let Caught::ByInjection(_) = fault.caught() else {
            continue;
        };
        rows.push(*fault);
        let report = injected(*fault);
        assert_eq!(
            report.status,
            ConformanceStatus::Passed,
            "{fault:?} is recorded as caught only with declared faults injected, but the suite \
             caught it: {:?}",
            not_passed(&report)
        );

        let model = model_at(fault.system().path());
        let verdict = |history: &ess_conformance::history::History| {
            linearize::check(&model, history, linearize::DEFAULT_BUDGET).expect("checked")
        };
        let mut caught = 0;
        let mut totals = sessions::Injected::default();
        for seed in 0..INJECTED_SEEDS {
            let (without, with, control) = three_recordings(&model, *fault, seed);
            assert_eq!(
                without.injected,
                sessions::Injected::default(),
                "seed {seed}: nothing is injected without injection"
            );
            assert_eq!(
                verdict(&without.history).verdict,
                Verdict::Linearizable,
                "{fault:?}, seed {seed}: the same seed without injection must miss it"
            );
            assert_eq!(
                verdict(&control.history).verdict,
                Verdict::Linearizable,
                "{fault:?}, seed {seed}: the reference with the same injections is linearizable, \
                 or the violation is not the fault's"
            );
            add(&mut totals, &with.injected);
            let checked = verdict(&with.history);
            if checked.verdict != Verdict::Violation {
                continue;
            }
            caught += 1;
            the_violation_is_the_faults(*fault, &with.history, &checked);
        }
        println!(
            "{:<28} {:<14} caught with declared faults injected in {caught} of {INJECTED_SEEDS} \
             seeds, without in 0; the suite caught nothing; injected: {}",
            fault.written(),
            fault.system().directory(),
            serde_json::to_string(&totals).expect("counts serialize")
        );
        assert!(
            caught > 0,
            "{fault:?}: no history recorded with declared faults injected was a violation"
        );
        assert!(
            totals.total() > 0 && totals.refused.is_empty(),
            "{totals:?}"
        );
    }
    assert_eq!(
        rows,
        vec![
            Fault::DoubleApplyOnRedelivery,
            Fault::RetryCreatesSecondEntity
        ]
    );
}

#[test]
fn an_undeclared_fault_is_never_injected() {
    // `delivery: at_most_once` declares that nothing delivers an event a second time, so a second
    // delivery of `InvoiceCreated` under it is a fault the specification does not declare. The
    // count is read twice: from what the recorder says it injected, and from the target itself —
    // every run of `notify-on-invoice-created` is an invocation the billing reference records.
    let undeclared = model_rewritten("examples/billing", |text| {
        text.replace("delivery: at_least_once", "delivery: at_most_once")
    });
    let declared = example("billing");
    let invocations = |target: &faulty::Faulty<Billing>| {
        target
            .observe_invocations(InvocationObservationRequest {
                binding: "notify-on-invoice-created".parse().unwrap(),
                command: "billing.email.SendEmail".parse().unwrap(),
                correlation: ess_primitives::ids::CorrelationId::new("history-1").unwrap(),
                deadline: Deadline::at(Timestamp::from_epoch_millis(u64::MAX)),
            })
            .expect("the reference shows its invocations")
            .len() as u64
    };
    let mut declared_redeliveries = 0;
    for seed in 0..INJECTED_SEEDS {
        let target = faulty::billing(Fault::DoubleApplyOnRedelivery);
        let recorded = sessions::record_injected(
            &undeclared,
            &target,
            &target,
            &faulty::double_apply_workload(),
            seed,
        )
        .expect("recorded");
        assert_eq!(
            recorded.injected.redeliveries.values().sum::<u64>(),
            0,
            "seed {seed}: a second delivery was injected under `at_most_once`"
        );
        assert_eq!(
            invocations(&target),
            1,
            "seed {seed}: the binding ran once per creation and never again"
        );
        assert_eq!(
            linearize::check(&undeclared, &recorded.history, linearize::DEFAULT_BUDGET)
                .expect("checked")
                .verdict,
            Verdict::Linearizable,
            "seed {seed}: with nothing delivered twice, the second invoice never appears"
        );

        // The same seed, the binding declaring `at_least_once`: the delivery is injected, and the
        // target saw it.
        let target = faulty::billing(Fault::DoubleApplyOnRedelivery);
        let recorded = sessions::record_injected(
            &declared,
            &target,
            &target,
            &faulty::double_apply_workload(),
            seed,
        )
        .expect("recorded");
        let counted = recorded.injected.redeliveries["notify-on-invoice-created"];
        assert_eq!(counted, 1, "seed {seed}: one creation, one second delivery");
        // The creation, its second delivery, and the creation the defect applied again, which
        // runs the binding once more.
        assert_eq!(invocations(&target), 3, "seed {seed}");
        declared_redeliveries += counted;
    }
    println!(
        "undeclared (at_most_once): 0 second deliveries in {INJECTED_SEEDS} seeds; declared \
         (at_least_once): {declared_redeliveries}"
    );
}

#[test]
fn a_declared_external_branch_is_delayed_or_left_unanswered_and_written_indeterminate() {
    // `billing.email.SendEmail/failed` is `external:`, so a call of `SendEmail` may be delayed past
    // the client's wait or never answered. Either is written `Indeterminate`, and the reference
    // stays linearizable: a call that may have taken effect is placed after every other one.
    let model = example("billing");
    let email = || {
        Act::Call(Call::new(
            "billing.email.SendEmail",
            BTreeMap::from([
                (
                    "recipient".to_owned(),
                    Node::Text("payer@example.com".to_owned()),
                ),
                (
                    "template".to_owned(),
                    Node::Text("invoice-created".to_owned()),
                ),
            ]),
            Subject::Creates,
        ))
    };
    let workload = sessions::Workload {
        prefix: Vec::new(),
        clients: vec![vec![email(), email()], vec![email(), email()]],
    };
    let mut totals = sessions::Injected::default();
    for seed in 0..INJECTED_SEEDS {
        let reference = Billing::new();
        let recorded =
            sessions::record_injected(&model, &Atomic(&reference), &reference, &workload, seed)
                .expect("recorded");
        let indeterminate = recorded
            .history
            .operations
            .iter()
            .filter(|operation| operation.completion == Completion::Indeterminate)
            .count() as u64;
        assert_eq!(
            indeterminate,
            recorded.injected.delayed.values().sum::<u64>()
                + recorded.injected.unanswered.values().sum::<u64>(),
            "seed {seed}: every injected delay or loss, and nothing else, is Indeterminate"
        );
        assert_eq!(
            linearize::check(&model, &recorded.history, linearize::DEFAULT_BUDGET)
                .expect("checked")
                .verdict,
            Verdict::Linearizable,
            "seed {seed}"
        );
        add(&mut totals, &recorded.injected);
    }
    println!(
        "external: {}",
        serde_json::to_string(&totals).expect("counts serialize")
    );
    assert!(totals.delayed["billing.email.SendEmail"] > 0, "{totals:?}");
    assert!(
        totals.unanswered["billing.email.SendEmail"] > 0,
        "{totals:?}"
    );
    assert!(
        totals.reached.contains_key("billing.email.SendEmail/sent"),
        "a delayed call's late answer names the branch it reached: {totals:?}"
    );
    assert!(totals.retries.is_empty() && totals.redeliveries.is_empty());
}

#[test]
fn a_retry_of_a_command_declaring_replays_is_written_as_one_and_replayed_by_the_reference() {
    // Every `Seed` a client sends is sent again, unchanged; the second operation names the first,
    // the document is admitted as written, and the reference answers each request's second
    // operation from what it retained.
    let model = model_at(System::Retry.path());
    let digest = ess_conformance::scenario::SuiteProvenance::of(&model).spec_digest;
    let mut totals = sessions::Injected::default();
    for seed in 0..INJECTED_SEEDS {
        let reference = Retained::new();
        let recorded = sessions::record_injected(
            &model,
            &Atomic(&reference),
            &reference,
            &faulty::retry_workload(),
            seed,
        )
        .expect("recorded");
        let retries: Vec<_> = recorded
            .history
            .operations
            .iter()
            .filter(|operation| operation.retry_of.is_some())
            .collect();
        assert_eq!(retries.len(), 2, "seed {seed}: one retry per `Seed`");
        let bytes = serde_json::to_vec(&recorded.history).expect("serializes");
        let read = ess_conformance::history::read(&bytes, &digest).expect("admitted");
        assert_eq!(read, recorded.history, "seed {seed}");
        assert_eq!(
            linearize::check(&model, &recorded.history, linearize::DEFAULT_BUDGET)
                .expect("checked")
                .verdict,
            Verdict::Linearizable,
            "seed {seed}"
        );
        add(&mut totals, &recorded.injected);
    }
    println!(
        "retry: {}",
        serde_json::to_string(&totals).expect("counts serialize")
    );
    assert_eq!(totals.retries["retry.core.Seed"], 2 * INJECTED_SEEDS);
    assert!(
        totals.reached.contains_key("retry.core.Seed/replayed"),
        "{totals:?}"
    );
}

/// A retry fixture history of two `Seed` operations on client 0, the second a retry of the first
/// when `retry_of` is given, answering `first` and `second`.
fn two_seeds(retry_of: Option<&str>, client: u64, first: &str, second: &str) -> Vec<u8> {
    let model = model_at(System::Retry.path());
    let digest = ess_conformance::scenario::SuiteProvenance::of(&model).spec_digest;
    let retry = retry_of.map_or_else(String::new, |id| format!(r#","retry_of":"{id}""#));
    let key = |outcome: &str, n: u8| {
        if outcome == "seeded" {
            format!("00000000-0000-4000-8000-00000000000{n}")
        } else {
            String::new()
        }
    };
    format!(
        r#"{{"format":"ess-history/1","history_id":"00000000-0000-4000-8000-000000000001","spec_digest":"{digest}","seed":1,"clients":2,"operations":[{{"operation_id":"00000000-0000-4000-8000-000000000001","client":0,"command":"retry.core.Seed","subject_key":"{}","invoked_at":1,"returned_at":3,"completion":"Returned","outcome":"{first}"}},{{"operation_id":"00000000-0000-4000-8000-000000000002","client":{client},"command":"retry.core.Seed","subject_key":"{}","invoked_at":2,"returned_at":4,"completion":"Returned","outcome":"{second}"{retry}}}]}}"#,
        key(first, 1),
        key(second, 2),
    )
    .into_bytes()
}

#[test]
fn a_retry_names_an_earlier_call_of_its_own_client_and_command_or_the_history_is_refused() {
    let model = model_at(System::Retry.path());
    let digest = ess_conformance::scenario::SuiteProvenance::of(&model).spec_digest;
    let first = "00000000-0000-4000-8000-000000000001";
    let admitted =
        ess_conformance::history::read(&two_seeds(Some(first), 0, "seeded", "replayed"), &digest)
            .expect("a retry of the client's own earlier call is admitted");
    assert_eq!(
        admitted.operations[1]
            .retry_of
            .as_ref()
            .map(|id| id.as_str().to_owned()),
        Some(first.to_owned())
    );
    // Written back in the reader's own spelling, `retry_of` last.
    assert_eq!(
        serde_json::to_vec(&admitted).unwrap(),
        two_seeds(Some(first), 0, "seeded", "replayed")
    );
    for (retry_of, client, why) in [
        (
            "00000000-0000-4000-8000-000000000009",
            0,
            "an operation the history does not hold",
        ),
        (first, 1, "another client's call"),
        (
            "00000000-0000-4000-8000-000000000002",
            0,
            "itself, which is not earlier",
        ),
    ] {
        let refused = ess_conformance::history::read(
            &two_seeds(Some(retry_of), client, "seeded", "replayed"),
            &digest,
        )
        .expect_err(why);
        assert_eq!(
            refused.code(),
            "history.retry-of-unknown",
            "{why}: {refused}"
        );
    }
}

#[test]
fn one_request_answered_by_its_origin_branch_twice_is_a_violation_and_once_is_not() {
    let model = model_at(System::Retry.path());
    let digest = ess_conformance::scenario::SuiteProvenance::of(&model).spec_digest;
    let first = "00000000-0000-4000-8000-000000000001";
    let checked = |bytes: Vec<u8>| {
        let history = ess_conformance::history::read(&bytes, &digest).expect("admitted");
        linearize::check(&model, &history, linearize::DEFAULT_BUDGET).expect("checked")
    };
    // Seeded, then replayed: the declared answer to a request sent twice.
    assert_eq!(
        checked(two_seeds(Some(first), 0, "seeded", "replayed")).verdict,
        Verdict::Linearizable
    );
    // Seeded twice, as two requests: two records, which nothing forbids.
    assert_eq!(
        checked(two_seeds(None, 0, "seeded", "seeded")).verdict,
        Verdict::Linearizable
    );
    // Seeded twice, as one request sent twice: applied twice.
    let twice = checked(two_seeds(Some(first), 0, "seeded", "seeded"));
    assert_eq!(twice.verdict, Verdict::Violation);
    assert_eq!(
        twice.subject_key.as_deref(),
        Some("00000000-0000-4000-8000-000000000002")
    );
    assert_eq!(
        twice.linearization,
        vec![
            first.to_owned(),
            "00000000-0000-4000-8000-000000000002".to_owned()
        ]
    );
}

#[test]
fn one_seed_with_injection_records_one_history_and_one_count() {
    // Each injection is part of the seed: the same seed injects the same faults at the same ticks.
    let model = example("billing");
    for seed in [0, 7, 23] {
        let first = faulty::billing(Fault::DoubleApplyOnRedelivery);
        let second = faulty::billing(Fault::DoubleApplyOnRedelivery);
        let workload = faulty::double_apply_workload();
        assert_eq!(
            sessions::record_injected(&model, &first, &first, &workload, seed).unwrap(),
            sessions::record_injected(&model, &second, &second, &workload, seed).unwrap()
        );
    }
}

// ---- the matrix has to be repeatable, or it is not a matrix ---------------------------------------

#[test]
fn two_runs_against_one_faulty_target_produce_byte_identical_reports() {
    // §37, applied to the faults rather than to the reference: a faulty target that introduced a new
    // source of variation would make every row above flaky, and a flaky matrix is worth less than no
    // matrix. `StaleReadYourWrites` is the one that carries state of its own — the previous answer
    // to a view — so it is the one worth pinning.
    for fault in [
        Fault::StaleReadYourWrites,
        Fault::WrongEvent,
        Fault::DropBinding,
    ] {
        let first = injected(fault);
        let second = injected(fault);
        assert_eq!(
            first.to_canonical_json(),
            second.to_canonical_json(),
            "{fault:?} did not reproduce, so nothing it reports can be evidence"
        );
        assert_eq!(
            first.implementation,
            ImplementationIdentity::new(
                match fault.system() {
                    System::Billing => format!("billing-reference-{}", fault.written()),
                    System::Oracle => format!("oracle-reference-{}", fault.written()),
                    System::Retry => format!("retry-reference-{}", fault.written()),
                },
                env!("CARGO_PKG_VERSION")
            ),
            "a report names the build that answered, and a deliberately wrong build says so (§30)"
        );
    }
}

#[test]
fn every_fault_that_could_be_a_boundary_perturbation_is_one() {
    // The judgement §25 leaves open, pinned rather than assumed: a wrapper that perturbs what goes
    // in and what comes out is in the same position as a real client, and an implementation-side
    // defect is not. Only the two the interface cannot express are allowed the second mechanism, and
    // `crates/verify/ess-conformance/src/faulty.rs` argues why.
    let implementation: Vec<Fault> = Fault::ALL
        .iter()
        .copied()
        .filter(|fault| fault.injection() == Injection::Implementation)
        .collect();
    assert_eq!(
        implementation,
        vec![Fault::DropBinding, Fault::WrongMapping]
    );

    // And the argument itself is checkable: the events a dropped binding would have caused are
    // indistinguishable at the boundary from the ones the other two bindings cause, because an
    // observation carries no attribution to the binding behind it.
    let placed: BTreeMap<String, Status> = injected(Fault::DropBinding)
        .scenarios
        .iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    assert_eq!(
        placed.get("handoff-on-held/binding/flow"),
        Some(&Status::Passed),
        "`oracle.dispatch.HandedOff` is one event name for three bindings, so a wrapper filtering \
         it out of `observe_events` would have silenced this one too"
    );
}
