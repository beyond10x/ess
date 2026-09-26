//! The mutation audit: specification mutants, each replayed against an unchanged target.
//!
//! `docs/design/mutation-audit-and-model-runner.md`, Part 1, and its deciding checks P1-1 to P1-10.
//! A mutant is killed when the suite synthesized from the mutated specification fails against a
//! target that implements the original; a survivor names a declared rule the suite does not pin.
//!
//! Every survivor below is pinned with a reason from a closed set, the way `faults.rs` pins
//! `Caught::Nothing`: a change to the list is a change to this file, with a reason.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use ess_compiler::source::SourceMap;
use ess_conformance::faulty::{self, Fault};
use ess_conformance::interpret::Interpreted;
use ess_conformance::mutate::{
    self, AuditRefusal, Document, MutantClass, MutantEntry, MutateCode, Mutation, MutationReport,
    Verdict,
};
use ess_conformance::reference::{Billing, Oracle};
use ess_conformance::report::Status;
use ess_conformance::runner::Runner;
use ess_conformance::target::ConformanceTarget;
use ess_conformance::AdmittedSuite;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;

// ---- the specifications under test ---------------------------------------------------------------

/// Every `.yaml` under `dir`, parsed, with the text map diagnostics locate against.
fn documents(base: &Path) -> (Vec<Document>, SourceMap) {
    let mut found: Vec<PathBuf> = Vec::new();
    if base.is_file() {
        found.push(base.to_path_buf());
    } else {
        let mut pending = vec![base.to_path_buf()];
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
    }
    found.sort();
    let mut texts = SourceMap::new();
    let mut parsed = Vec::new();
    for path in found {
        let label = path.display().to_string();
        let text = std::fs::read_to_string(&path).expect("readable");
        let raw = RawSpecFile::parse(&text)
            .unwrap_or_else(|error| panic!("{label} is well formed: {error}"));
        texts.insert(label.clone(), text);
        parsed.push((Source::new(label), raw));
    }
    (parsed, texts)
}

fn example(name: &str) -> (Vec<Document>, SourceMap) {
    documents(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../examples")
            .join(name)
            .canonicalize()
            .unwrap_or_else(|error| panic!("`{name}` exists: {error}")),
    )
}

fn fixture(name: &str) -> (Vec<Document>, SourceMap) {
    documents(
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
}

fn audit<T: ConformanceTarget>(
    (files, texts): &(Vec<Document>, SourceMap),
    classes: &[MutantClass],
    target: impl Fn() -> T,
) -> MutationReport {
    mutate::audit(files, texts, classes, target).unwrap_or_else(|refusal| panic!("{refusal}"))
}

/// Why a mutant survives, from a closed set.
///
/// Read by a person, not by the assertions: the list is pinned by id, and the reason is what a
/// change to it has to argue with. Two of the three are pinned nowhere today, so the lint that
/// would otherwise say so is allowed rather than the set being narrowed to what is in use.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Why {
    /// The witnesses synthesis draws cannot tell the two specifications apart, though some input
    /// could.
    WeakWitness,
    /// No input tells the two specifications apart.
    Equivalent,
    /// Synthesis derives no scenario that observes the rule; filed as a story, not fixed here.
    SynthesisGap { story: &'static str },
}

/// The mutant count per class, derived from the site rules by reading the specification.
fn per_class(report: &MutationReport) -> BTreeMap<&'static str, usize> {
    let mut counts = BTreeMap::new();
    for class in MutantClass::ALL {
        counts.insert(class.as_str(), 0);
    }
    for entry in &report.mutants {
        *counts.get_mut(entry.class.as_str()).expect("a known class") += 1;
    }
    counts
}

fn survivors(report: &MutationReport) -> Vec<&str> {
    report
        .mutants
        .iter()
        .filter(|entry| entry.verdict == Verdict::Survived)
        .map(|entry| entry.id.as_str())
        .collect()
}

fn pinned_survivors(report: &MutationReport, pinned: &[(&str, Why)]) {
    let found = survivors(report);
    let expected: Vec<&str> = pinned.iter().map(|(id, _)| *id).collect();
    assert_eq!(
        found,
        expected,
        "the survivors moved; a change to this list needs a reason\n{}",
        report.render_text()
    );
}

// ---- P1-1 and P1-2: the two reference systems ---------------------------------------------------

/// Every mutant's verdict, and for a stillborn one the model's own code, for a killed one one
/// scenario that must be among its killers.
fn pinned_verdicts(report: &MutationReport, pinned: &[(&str, Verdict, &str)]) {
    let found: Vec<(&str, Verdict)> = report
        .mutants
        .iter()
        .map(|entry| (entry.id.as_str(), entry.verdict))
        .collect();
    let expected: Vec<(&str, Verdict)> = pinned
        .iter()
        .map(|(id, verdict, _)| (*id, *verdict))
        .collect();
    assert_eq!(found, expected, "{}", report.render_text());
    for ((_, _, evidence), entry) in pinned.iter().zip(&report.mutants) {
        match entry.verdict {
            Verdict::Stillborn => {
                let stillborn = entry
                    .stillborn
                    .as_ref()
                    .expect("a stillborn entry says why");
                assert_eq!(stillborn.code, "ESS-MUTATE-002");
                assert_eq!(stillborn.cause, *evidence, "{}", entry.id);
            }
            Verdict::Killed => assert!(
                entry
                    .killers
                    .as_deref()
                    .unwrap_or_default()
                    .iter()
                    .any(|killer| killer == evidence),
                "{} is not killed by `{evidence}`: {:?}",
                entry.id,
                entry.killers
            ),
            Verdict::Survived | Verdict::Inconclusive => {}
        }
    }
}

/// Billing's survivors, each with the reason no scenario kills it. None: every mutant that ran
/// was killed.
const BILLING_SURVIVORS: &[(&str, Why)] = &[];

/// Billing's twenty mutants.
///
/// **The design page predicted every `transition-to` mutant would be killed by its own transition
/// scenario. Measured, all three are stillborn**, refused `ESS-ENTITY-011`: in `examples/billing/`
/// every state is entered by exactly one transition, so sending any transition elsewhere leaves
/// its old arrival state unreachable, and validation refuses an unreachable state. No choice of
/// alternative state changes that. The same is true of every `emit-drop`: each billing outcome
/// emits exactly one event, so dropping it leaves an outcome that neither emits nor names an error,
/// which `ESS-COMMAND-007` refuses.
///
/// What P1-1 was there to catch — an audit that ran the *original* suite for every mutant — is
/// caught instead by the two `from-drop` mutants: each is killed by a refusal scenario,
/// `…/state/Draft/refuses/…CancelInvoice` and `…/state/Issued/refuses/…CancelInvoice`, that exists
/// only in the mutant's own suite, because the original lets `cancel` run from both states.
const BILLING_VERDICTS: &[(&str, Verdict, &str)] = &[
    (
        "emit-drop/billing.email.SendEmail/sent/billing.email.EmailSent",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "emit-drop/billing.invoice.CancelInvoice/cancelled/billing.invoice.InvoiceCancelled",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "emit-drop/billing.invoice.CreateInvoice/accepted/billing.invoice.InvoiceCreated",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "emit-drop/billing.invoice.IssueInvoice/issued/billing.invoice.InvoiceIssued",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "emit-drop/billing.invoice.PayInvoice/settled/billing.invoice.InvoicePaid",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "error-swap/billing.invoice.CancelInvoice/wrong-state",
        Verdict::Killed,
        "billing.invoice.Invoice/state/Paid/refuses/billing.invoice.CancelInvoice",
    ),
    (
        "error-swap/billing.invoice.CreateInvoice/rejected",
        Verdict::Killed,
        "billing.invoice.CreateInvoice/outcome/rejected",
    ),
    (
        "error-swap/billing.invoice.IssueInvoice/wrong-state",
        Verdict::Killed,
        "billing.invoice.Invoice/state/Issued/refuses/billing.invoice.IssueInvoice",
    ),
    (
        "error-swap/billing.invoice.PayInvoice/rejected",
        Verdict::Killed,
        "billing.invoice.PayInvoice/outcome/rejected",
    ),
    (
        "error-swap/billing.invoice.PayInvoice/wrong-state",
        Verdict::Killed,
        "billing.invoice.Invoice/state/Draft/refuses/billing.invoice.PayInvoice",
    ),
    (
        "from-drop/billing.invoice.Invoice.cancel/Draft",
        Verdict::Killed,
        "billing.invoice.Invoice/state/Draft/refuses/billing.invoice.CancelInvoice",
    ),
    (
        "from-drop/billing.invoice.Invoice.cancel/Issued",
        Verdict::Killed,
        "billing.invoice.Invoice/state/Issued/refuses/billing.invoice.CancelInvoice",
    ),
    (
        "guard-boundary/billing.invoice.CreateInvoice/accepted/0",
        Verdict::Killed,
        "billing.invoice.CreateInvoice/outcome/accepted",
    ),
    (
        "guard-boundary/billing.invoice.PayInvoice/settled/0",
        Verdict::Killed,
        "billing.invoice.PayInvoice/outcome/settled",
    ),
    (
        "guard-negate/billing.invoice.CreateInvoice/accepted",
        Verdict::Killed,
        "billing.invoice.CreateInvoice/outcome/accepted",
    ),
    (
        "guard-negate/billing.invoice.PayInvoice/settled",
        Verdict::Killed,
        "billing.invoice.PayInvoice/outcome/settled",
    ),
    (
        "order-flip/billing.invoice.OutstandingInvoices/issued_at",
        Verdict::Killed,
        "billing.invoice.IssueInvoice/outcome/issued",
    ),
    (
        "transition-to/billing.invoice.Invoice.cancel",
        Verdict::Stillborn,
        "ESS-ENTITY-011",
    ),
    (
        "transition-to/billing.invoice.Invoice.issue",
        Verdict::Stillborn,
        "ESS-ENTITY-011",
    ),
    (
        "transition-to/billing.invoice.Invoice.settle",
        Verdict::Stillborn,
        "ESS-ENTITY-011",
    ),
];

#[test]
fn the_billing_audit_pins_every_mutant_and_kills_every_one_that_ran() {
    let report = audit(&example("billing"), MutantClass::ALL, Billing::new);
    assert_eq!(
        per_class(&report),
        BTreeMap::from([
            ("emit-drop", 5),
            ("error-swap", 5),
            ("from-drop", 2),
            ("guard-boundary", 2),
            ("guard-connective", 0),
            ("guard-negate", 2),
            ("order-flip", 1),
            ("sets-retarget", 0),
            ("transition-to", 3),
        ]),
        "{}",
        report.render_text()
    );
    pinned_verdicts(&report, BILLING_VERDICTS);
    pinned_survivors(&report, BILLING_SURVIVORS);
    assert_eq!(report.implementation, "billing-reference");
    assert_eq!(report.specification, "billing v3");
    assert_eq!(report.counts.mutants, 20);
    assert_eq!(report.counts.killed, 12);
    assert_eq!(report.counts.stillborn, 8);
}

/// The oracle fixture's survivors. None.
const ORACLE_SURVIVORS: &[(&str, Why)] = &[];

/// The oracle fixture's twenty mutants. `from-drop/…cancel/Held` is stillborn for the reason
/// `ESS-ENTITY-011` gives: `cancel` is `Held`'s only way out, so dropping it strands the state.
const ORACLE_VERDICTS: &[(&str, Verdict, &str)] = &[
    (
        "emit-drop/oracle.dispatch.Handoff/accepted/oracle.dispatch.HandedOff",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "emit-drop/oracle.order.AmendOrder/amended/oracle.order.OrderAmended",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "emit-drop/oracle.order.CancelOrder/cancelled/oracle.order.OrderCancelled",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "emit-drop/oracle.order.HoldOrder/held/oracle.order.OrderHeld",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "emit-drop/oracle.order.PlaceOrder/accepted/oracle.order.OrderPlaced",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "emit-drop/oracle.order.ShipOrder/shipped/oracle.order.OrderShipped",
        Verdict::Stillborn,
        "ESS-COMMAND-007",
    ),
    (
        "error-swap/oracle.order.AmendOrder/rejected",
        Verdict::Killed,
        "oracle.order.AmendOrder/outcome/rejected",
    ),
    (
        "error-swap/oracle.order.CancelOrder/wrong-state",
        Verdict::Killed,
        "oracle.order.Order/state/Shipped/refuses/oracle.order.CancelOrder",
    ),
    (
        "error-swap/oracle.order.HoldOrder/wrong-state",
        Verdict::Killed,
        "oracle.order.Order/state/Held/refuses/oracle.order.HoldOrder",
    ),
    (
        "error-swap/oracle.order.PlaceOrder/rejected",
        Verdict::Killed,
        "oracle.order.PlaceOrder/outcome/rejected",
    ),
    (
        "error-swap/oracle.order.ShipOrder/wrong-state",
        Verdict::Killed,
        "oracle.order.Order/state/Held/refuses/oracle.order.ShipOrder",
    ),
    (
        "from-drop/oracle.order.Order.cancel/Held",
        Verdict::Stillborn,
        "ESS-ENTITY-011",
    ),
    (
        "from-drop/oracle.order.Order.cancel/Placed",
        Verdict::Killed,
        "oracle.order.Order/state/Placed/refuses/oracle.order.CancelOrder",
    ),
    (
        "guard-boundary/oracle.order.AmendOrder/amended/0",
        Verdict::Killed,
        "oracle.order.AmendOrder/outcome/rejected",
    ),
    (
        "guard-boundary/oracle.order.PlaceOrder/accepted/0",
        Verdict::Killed,
        "oracle.order.PlaceOrder/outcome/rejected",
    ),
    (
        "guard-negate/oracle.order.AmendOrder/amended",
        Verdict::Killed,
        "oracle.order.AmendOrder/outcome/amended",
    ),
    (
        "guard-negate/oracle.order.PlaceOrder/accepted",
        Verdict::Killed,
        "oracle.order.PlaceOrder/outcome/accepted",
    ),
    (
        "transition-to/oracle.order.Order.cancel",
        Verdict::Stillborn,
        "ESS-ENTITY-011",
    ),
    (
        "transition-to/oracle.order.Order.hold",
        Verdict::Stillborn,
        "ESS-ENTITY-011",
    ),
    (
        "transition-to/oracle.order.Order.ship",
        Verdict::Stillborn,
        "ESS-ENTITY-011",
    ),
];

#[test]
fn the_oracle_audit_pins_every_mutant_and_kills_every_one_that_ran() {
    let report = audit(&example("oracle-fixture"), MutantClass::ALL, Oracle::new);
    assert_eq!(
        per_class(&report),
        BTreeMap::from([
            ("emit-drop", 6),
            ("error-swap", 5),
            ("from-drop", 2),
            ("guard-boundary", 2),
            ("guard-connective", 0),
            ("guard-negate", 2),
            ("order-flip", 0),
            ("sets-retarget", 0),
            ("transition-to", 3),
        ]),
        "{}",
        report.render_text()
    );
    pinned_verdicts(&report, ORACLE_VERDICTS);
    pinned_survivors(&report, ORACLE_SURVIVORS);
    assert_eq!(report.implementation, "oracle-reference");
}

// ---- P1-3: a killer is a failed scenario of that mutant's own suite -------------------------------

#[test]
fn every_killer_is_a_scenario_of_the_mutants_own_suite_that_failed() {
    let (files, texts) = example("billing");
    let report = audit(
        &(files.clone(), texts.clone()),
        MutantClass::ALL,
        Billing::new,
    );
    let mut checked = 0;
    for entry in report
        .mutants
        .iter()
        .filter(|entry| entry.verdict == Verdict::Killed)
    {
        let mutant = mutate::mutants(&files, &[entry.class])
            .into_iter()
            .find(|mutant| mutant.id == entry.id)
            .expect("the reported mutant is enumerated again");
        let mutated = mutate::apply(&files, &mutant.mutation).expect("the site exists");
        let ir = mutate::compile(mutated, &texts).expect("a killed mutant compiled");
        let mut suite = ess_conformance::synthesize(&ir).suite;
        suite.select_fresh_format();
        let admitted = AdmittedSuite::from_suite(&suite).unwrap();
        let run = Runner::for_suite(&suite).run_admitted(&admitted, &Billing::new());
        let killers = entry
            .killers
            .as_deref()
            .expect("a killed mutant names its killers");
        assert!(!killers.is_empty(), "{} is killed by nothing", entry.id);
        for killer in killers {
            let status = run
                .scenarios
                .iter()
                .find(|result| result.scenario.to_string() == *killer)
                .map(|result| result.status);
            assert_eq!(status, Some(Status::Failed), "{}: {killer}", entry.id);
        }
        let mut sorted = killers.to_vec();
        sorted.sort();
        assert_eq!(sorted, killers, "killers are sorted");
        checked += 1;
    }
    assert!(checked > 0, "no killed mutant was checked");
}

// ---- P1-4: a baseline that does not pass refuses the audit ---------------------------------------

#[test]
fn a_faulty_baseline_is_refused_with_the_failing_scenarios() {
    let (files, texts) = example("billing");
    let fault = Fault::AllowIllegalTransition;
    let refusal = mutate::audit(&files, &texts, MutantClass::ALL, || faulty::billing(fault))
        .expect_err("a faulty baseline is refused");
    let AuditRefusal::BaselineFailed { not_passed, .. } = &refusal else {
        panic!("expected ESS-MUTATE-001, got {refusal}");
    };
    assert_eq!(
        refusal.code().map(|code| code.to_string()).as_deref(),
        Some("ESS-MUTATE-001")
    );

    // Exactly the scenarios the fault matrix sees fail, measured the same way.
    let ir = mutate::compile(files.clone(), &texts).unwrap();
    let mut suite = ess_conformance::synthesize(&ir).suite;
    suite.select_fresh_format();
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let run = Runner::for_suite(&suite).run_admitted(&admitted, &faulty::billing(fault));
    let expected: Vec<String> = run
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| result.scenario.to_string())
        .collect();
    assert!(
        expected
            .iter()
            .any(|id| id
                == "billing.invoice.Invoice/state/Paid/refuses/billing.invoice.CancelInvoice")
    );
    assert_eq!(not_passed, &expected);
}

#[test]
fn the_interpreted_target_is_refused_with_mutate_001() {
    let (files, texts) = example("billing");
    let refusal = mutate::audit(&files, &texts, MutantClass::ALL, Interpreted::new)
        .expect_err("a target that answers nothing has no green baseline");
    assert!(
        matches!(refusal, AuditRefusal::BaselineFailed { .. }),
        "{refusal}"
    );
    assert!(
        refusal.to_string().starts_with("refusal[ESS-MUTATE-001]"),
        "{refusal}"
    );
}

// ---- P1-5: two audits, identical bytes ----------------------------------------------------------

#[test]
fn two_audits_of_one_tree_are_byte_identical() {
    let spec = example("billing");
    let first = audit(&spec, MutantClass::ALL, Billing::new).to_canonical_json();
    let second = audit(&spec, MutantClass::ALL, Billing::new).to_canonical_json();
    assert_eq!(first, second);
    assert!(first.ends_with("}\n"), "one trailing LF");
    let value: serde_json::Value = serde_json::from_str(&first).unwrap();
    assert_eq!(value["format"], "ess-mutation-report/1");
    assert_eq!(value["spec_digest"].as_str().map(str::len), Some(64));
    let ids: Vec<&str> = value["mutants"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["id"].as_str().unwrap())
        .collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted, "mutants run in byte order of id");
}

// ---- P1-6: a mutant the model refuses is stillborn ----------------------------------------------

#[test]
fn a_transition_to_an_undeclared_state_is_stillborn_with_the_compilers_code() {
    let (files, texts) = example("billing");
    let mutation = Mutation::TransitionTo {
        entity: "billing.invoice.Invoice".to_owned(),
        transition: "settle".to_owned(),
        to: "Nowhere".to_owned(),
    };
    let mutated = mutate::apply(&files, &mutation).expect("the site exists");
    let stillborn = mutate::compile(mutated, &texts).expect_err("`Nowhere` is not a state");
    assert_eq!(stillborn.code, "ESS-MUTATE-002");
    assert!(
        stillborn.cause.starts_with("ESS-") && stillborn.cause != "ESS-MUTATE-002",
        "the refusal carries the model's own code: {stillborn:?}"
    );
    assert!(stillborn.message.contains("Nowhere"), "{stillborn:?}");

    let entry = mutate::evaluate(
        &files,
        &texts,
        &mutate::Mutant::new(&files, mutation).expect("describable"),
        Billing::new,
    )
    .expect("admissible");
    assert_eq!(entry.verdict, Verdict::Stillborn);
    assert_eq!(entry.stillborn.as_ref(), Some(&stillborn));
    assert_eq!(entry.killers, None);
    assert_eq!(entry.scenarios, None);
}

// ---- P1-7: the verdict of a mutant's run ---------------------------------------------------------

#[test]
fn the_verdict_classification_reads_scenario_statuses() {
    use Status::{Error, Failed, Passed, Unsupported};
    assert_eq!(Verdict::classify(&[Passed, Passed]), Verdict::Survived);
    assert_eq!(Verdict::classify(&[Passed, Failed]), Verdict::Killed);
    assert_eq!(
        Verdict::classify(&[Error, Failed, Unsupported]),
        Verdict::Killed
    );
    assert_eq!(Verdict::classify(&[Passed, Error]), Verdict::Inconclusive);
    assert_eq!(
        Verdict::classify(&[Passed, Unsupported]),
        Verdict::Inconclusive
    );
    assert_eq!(Verdict::classify(&[]), Verdict::Survived);
}

// ---- P1-8: the closed class list -----------------------------------------------------------------

#[test]
fn the_classes_are_exactly_the_nine_altering_classes() {
    let names: Vec<&str> = MutantClass::ALL
        .iter()
        .map(|class| class.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "from-drop",
            "transition-to",
            "guard-boundary",
            "sets-retarget",
            "guard-negate",
            "guard-connective",
            "error-swap",
            "emit-drop",
            "order-flip",
        ]
    );
}

#[test]
fn every_mutate_code_is_derived_from_its_variant() {
    let codes: Vec<String> = MutateCode::ALL
        .iter()
        .map(|code| code.code().to_string())
        .collect();
    assert_eq!(
        codes,
        ["ESS-MUTATE-001", "ESS-MUTATE-002", "ESS-MUTATE-003"]
    );
}

// ---- P1-9: an audit that finds no site ran nothing ------------------------------------------------

#[test]
fn a_class_with_no_site_is_refused_with_mutate_003() {
    let (files, texts) = example("oracle-fixture");
    let refusal = mutate::audit(&files, &texts, &[MutantClass::OrderFlip], Oracle::new)
        .expect_err("the oracle fixture declares no order_by");
    assert!(matches!(refusal, AuditRefusal::NoSite { .. }), "{refusal}");
    assert_eq!(
        refusal.code().map(|code| code.to_string()).as_deref(),
        Some("ESS-MUTATE-003")
    );
}

// ---- P1-10: a sets field nothing reads ----------------------------------------------------------

#[test]
fn a_sets_field_no_view_reads_is_the_one_survivor() {
    let report = audit(
        &fixture("mutation-survivor.yaml"),
        MutantClass::ALL,
        Billing::new,
    );
    // No view projects `contact` and no event carries it, so nothing a target publishes can tell
    // `customer_email` from `billing_email` there.
    pinned_survivors(
        &report,
        &[(
            "sets-retarget/billing.invoice.CreateInvoice/accepted/contact",
            Why::Equivalent,
        )],
    );
    let killed: Vec<&MutantEntry> = report
        .mutants
        .iter()
        .filter(|entry| entry.verdict == Verdict::Killed)
        .collect();
    assert!(!killed.is_empty(), "{}", report.render_text());
    for entry in killed {
        assert!(
            !entry.killers.as_deref().unwrap_or_default().is_empty(),
            "{}",
            entry.id
        );
    }
    assert_eq!(report.counts.survived, 1);
    let entry = report
        .mutants
        .iter()
        .find(|entry| entry.class == MutantClass::SetsRetarget)
        .unwrap();
    assert_eq!(
        entry.change,
        "`contact: input.customer_email` becomes `contact: input.billing_email`"
    );
}

// ---- the mutants that are applied are the ones described -----------------------------------------

#[test]
fn each_class_changes_exactly_what_its_description_says() {
    let (files, _) = example("billing");
    let described: BTreeMap<String, String> = mutate::mutants(&files, MutantClass::ALL)
        .into_iter()
        .map(|mutant| (mutant.id, mutant.change))
        .collect();
    for (id, change) in [
        (
            "transition-to/billing.invoice.Invoice.settle",
            "`to: Paid` becomes `to: Cancelled`",
        ),
        (
            "from-drop/billing.invoice.Invoice.cancel/Draft",
            "`from: [Draft, Issued]` becomes `from: [Issued]`",
        ),
        (
            "guard-boundary/billing.invoice.PayInvoice/settled/0",
            "`amount.amount > 0` becomes `amount.amount >= 0`",
        ),
        (
            "guard-negate/billing.invoice.CreateInvoice/accepted",
            "`when: amount.amount > 0` becomes `when: not (amount.amount > 0)`",
        ),
        (
            "error-swap/billing.invoice.CancelInvoice/wrong-state",
            "`error: billing.invoice.InvoiceStateConflict` becomes `error: billing.invoice.InvalidAmount`",
        ),
        (
            "emit-drop/billing.invoice.IssueInvoice/issued/billing.invoice.InvoiceIssued",
            "`billing.invoice.InvoiceIssued` is no longer emitted",
        ),
        (
            "order-flip/billing.invoice.OutstandingInvoices/issued_at",
            "`issued_at desc` becomes `issued_at asc`",
        ),
    ] {
        assert_eq!(described.get(id).map(String::as_str), Some(change), "{id}: {described:#?}");
    }
}
