//! Accepted actual generic Interpreter parity, not a native Runner using another implementation.
mod support_interpreted;
use ess_conformance::{
    interpret::Interpreted,
    reference::{Billing, Oracle},
    report::{ConformanceReport, Status},
    target::ConformanceTarget,
    Runner,
};
fn compare(example: &str, reference: &impl ConformanceTarget) {
    let ir = support_interpreted::model(example);
    let input = support_interpreted::suite(example);
    assert_eq!(
        ess_conformance::SuiteProvenance::of(&ir).spec_digest,
        input.suite().provenance.spec_digest
    );
    let expected = Runner::for_suite(input.suite())
        .run_admitted(&input, reference)
        .into_report();
    let actual = Runner::for_suite(input.suite())
        .run_admitted(&input, &Interpreted::for_model(ir))
        .into_report();
    assert_same(&actual, &expected);
}
fn assert_same(actual: &ConformanceReport, expected: &ConformanceReport) {
    assert_eq!(actual.scenarios.len(), expected.scenarios.len());
    let differences: Vec<_> = actual
        .scenarios
        .iter()
        .zip(&expected.scenarios)
        .filter_map(|(actual, expected)| {
            assert_eq!(actual.scenario, expected.scenario);
            (actual.status != expected.status).then(|| {
                format!(
                    "{}: {:?} vs {:?}: {:?}",
                    actual.scenario,
                    actual.status,
                    expected.status,
                    actual
                        .checks
                        .iter()
                        .filter(|check| check.status != Status::Passed)
                        .collect::<Vec<_>>()
                )
            })
        })
        .collect();
    assert!(
        differences.is_empty(),
        "{} of {} scenario verdicts differ:\n{}",
        differences.len(),
        actual.scenarios.len(),
        differences.join("\n")
    );
}
#[test]
fn every_committed_billing_scenario_matches_handwritten_target() {
    compare("billing", &Billing::new());
}
#[test]
fn every_committed_oracle_scenario_matches_handwritten_target() {
    compare("oracle-fixture", &Oracle::new());
}

#[test]
fn every_billing_and_oracle_fault_discriminates_the_same_scenarios() {
    use ess_conformance::faulty::{self, Fault, Faulty, System};
    let mut compared = 0;
    for &fault in Fault::ALL {
        let example = match fault.system() {
            System::Billing => "billing",
            System::Oracle => "oracle-fixture",
            System::Retry => continue,
        };
        println!("fault {}", fault.written());
        let input = support_interpreted::suite(example);
        let ir = support_interpreted::model_with(example, |text| {
            if !text.contains("  - id: handoff-on-placed") {
                return text.into();
            }
            let start = text.find("  - id: handoff-on-placed").unwrap();
            let end = text[start + 1..]
                .find("  - id: ")
                .map_or(text.len(), |offset| start + 1 + offset);
            match fault {
                Fault::DropBinding => format!("{}{}", &text[..start], &text[end..]),
                Fault::WrongMapping => format!(
                    "{}{}{}",
                    &text[..start],
                    text[start..end].replace(
                        "recipient: event.contact",
                        "recipient: event.alternate_contact"
                    ),
                    &text[end..]
                ),
                _ => text.into(),
            }
        });
        let actual = Runner::for_suite(input.suite())
            .run_admitted(&input, &Faulty::new(Interpreted::for_model(ir), fault))
            .into_report();
        let expected = match fault.system() {
            System::Billing => Runner::for_suite(input.suite())
                .run_admitted(&input, &faulty::billing(fault))
                .into_report(),
            System::Oracle => Runner::for_suite(input.suite())
                .run_admitted(&input, &faulty::oracle(fault))
                .into_report(),
            System::Retry => unreachable!(),
        };
        assert_same(&actual, &expected);
        compared += 1;
    }
    assert_eq!(
        compared,
        Fault::ALL
            .iter()
            .filter(|fault| fault.system() != System::Retry)
            .count()
    );
    assert!(compared >= 16);
}
