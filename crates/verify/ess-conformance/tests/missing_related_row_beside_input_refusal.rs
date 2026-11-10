//! The `exists: false` branch beside an input-guarded refusal that takes the plain input
//! (beyond10x/ess#731).
//!
//! Under the one precedence rule an input-guarded refusal answers before a missing related row, so
//! the `exists: false` scenario is sent an input no such refusal takes. Where the plain value of an
//! enum input is taken, the search is directed to the values the refusals do not take; where every
//! value is taken the branch is refused ESS-SYNTH-003 naming the refusal that took the input.
//!
//! `tests/fixtures/marks-beside-input-refusal.yaml`: `RecordMark` refuses `kind == Satisfied`, the
//! first variant of `Kind`.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MARKS: &str = include_str!("fixtures/marks-beside-input-refusal.yaml");

const NO_DECISION: &str = "example.marks.RecordMark/outcome/no-decision";

const REFUSAL: &str = "      - name: not-permitted\n        when: kind == Satisfied\n        error: example.marks.NotPermitted\n";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), text.to_owned());
    let spec = Specification::assemble(vec![(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result.refusals.iter().map(ToString::to_string).collect()
}

#[test]
fn missing_decision_is_sent_the_kind_no_refusal_takes() {
    let result = synthesis(MARKS);
    assert_eq!(refusals(&result), Vec::<String>::new());
    let scenario = result
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == NO_DECISION)
        .map_or_else(|| panic!("no scenario {NO_DECISION}"), |(_, scenario)| scenario);
    let sent: Vec<_> = scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "example.marks.RecordMark" =>
            {
                Some(input)
            }
            _ => None,
        })
        .collect();
    assert!(!sent.is_empty(), "{scenario:#?}");
    let kind = sent[sent.len() - 1]
        .get("kind")
        .unwrap_or_else(|| panic!("{sent:#?}"));
    let rendered = format!("{kind:?}");
    assert!(
        rendered.contains("Unsupported"),
        "{rendered}\n{scenario:#?}"
    );

    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|e| panic!("{e}"));
    let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir(MARKS)))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    let failed: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn missing_decision_names_the_refusal_that_took_every_kind() {
    let second = "      - name: not-supported\n        when: kind == Unsupported\n        error: example.marks.NotPermitted\n";
    let both = MARKS.replace(REFUSAL, &format!("{REFUSAL}{second}"));
    assert_ne!(both, MARKS);
    let result = synthesis(&both);
    let missing: Vec<_> = result
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .is_some_and(|id| id.to_string() == NO_DECISION)
        })
        .collect();
    assert_eq!(missing.len(), 1, "{:#?}", refusals(&result));
    let text = missing[0].to_string();
    assert_eq!(missing[0].code().to_string(), "ESS-SYNTH-003", "{text}");
    assert!(text.contains("every such input taken first by `"), "{text}");
}
