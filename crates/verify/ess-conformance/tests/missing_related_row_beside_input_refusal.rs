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
        .map_or_else(
            || panic!("no scenario {NO_DECISION}"),
            |(_, scenario)| scenario,
        );
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

/// `PostSlip` refuses four inputs over different leaves before its `exists: false` branch
/// (beyond10x/ess#804): no plain candidate and no candidate directed to refute the refusals one at a
/// time escapes all four, while one input refutes every one of them at once.
const SLIPS: &str = r"format: ess/37
system: example
version: v1
domain: example.slips
types:
  - {name: example.slips.SlipId, kind: newtype, of: String}
  - {name: example.slips.Kind, kind: enum, variants: [Plain, Rush, Bulk]}
  - {name: example.slips.BinId, kind: newtype, of: String}
entities:
  - name: example.slips.Slip
    identity: {name: slip_id, type: example.slips.SlipId}
    fields:
      - {name: writer, type: String}
    lifecycle: {initial: Posted, states: [Posted], terminal: [Posted], transitions: []}
  - name: example.slips.Bin
    identity: {name: bin_id, type: example.slips.BinId}
    fields:
      - {name: closed, type: Boolean}
    lifecycle: {initial: Open, states: [Open], terminal: [Open], transitions: []}
actors:
  - name: example.slips.Clerk
    attributes:
      - {name: principal, type: String}
    may: [example.slips.PostSlip, example.slips.AddBin]
errors:
  - name: example.slips.NotYours
  - name: example.slips.BadWindow
  - name: example.slips.BulkUncarried
  - name: example.slips.SelfAddressed
  - name: example.slips.UnknownBin
  - name: example.slips.Duplicate
  - name: example.slips.BinExists
commands:
  - name: example.slips.PostSlip
    input:
      - {name: slip_id, type: example.slips.SlipId}
      - {name: writer, type: String}
      - {name: to, type: String}
      - {name: reply_to, type: String}
      - {name: kind, type: example.slips.Kind}
      - {name: created, type: Timestamp}
      - {name: expires, type: Timestamp}
      - {name: courier, type: Optional<String>}
      - {name: memo, type: Optional<String>}
      - {name: bin_id, type: example.slips.BinId}
    outcomes:
      - name: not-yours
        when: 'writer != caller.principal'
        error: example.slips.NotYours
      - name: bad-window
        when: 'expires <= created'
        error: example.slips.BadWindow
      - name: bulk-uncarried
        when: {all: ['kind == Bulk', 'not defined(courier)']}
        error: example.slips.BulkUncarried
      - name: self-addressed
        when: {all: ['to == reply_to', 'not defined(memo)']}
        error: example.slips.SelfAddressed
      - name: unknown-bin
        when_related: {via: input.bin_id, exists: false}
        error: example.slips.UnknownBin
      - {name: duplicate, existing_instance: true, error: example.slips.Duplicate}
      - name: posted
        creates: example.slips.Slip
        instance: slip_id
        sets:
          writer: input.writer
        emits: [example.slips.SlipPosted]
        payload:
          example.slips.SlipPosted: {slip_id: input.slip_id}
  - name: example.slips.AddBin
    input:
      - {name: bin_id, type: example.slips.BinId}
      - {name: closed, type: Boolean}
    outcomes:
      - {name: bin-exists, existing_instance: true, error: example.slips.BinExists}
      - name: added
        creates: example.slips.Bin
        instance: bin_id
        sets:
          closed: input.closed
        emits: [example.slips.BinAdded]
        payload:
          example.slips.BinAdded: {bin_id: input.bin_id}
events:
  - name: example.slips.SlipPosted
    fields:
      - {name: slip_id, type: example.slips.SlipId}
  - name: example.slips.BinAdded
    fields:
      - {name: bin_id, type: example.slips.BinId}
views:
  - name: example.slips.Slips
    source: example.slips.Slip
    consistency: read_your_writes
    fields:
      - {name: slip_id, type: example.slips.SlipId}
      - {name: writer, type: String}
  - name: example.slips.Bins
    source: example.slips.Bin
    consistency: read_your_writes
    fields:
      - {name: bin_id, type: example.slips.BinId}
      - {name: closed, type: Boolean}
      - {name: state, type: example.slips.Bin.State}
";

const UNKNOWN_BIN: &str = "example.slips.PostSlip/outcome/unknown-bin";

fn scenario_ids(result: &Synthesis) -> Vec<String> {
    result
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn a_missing_row_branch_beneath_four_refusals_is_witnessed() {
    let result = synthesis(SLIPS);
    assert_eq!(refusals(&result), Vec::<String>::new());
    assert!(
        scenario_ids(&result).iter().any(|id| id == UNKNOWN_BIN),
        "{:#?}",
        scenario_ids(&result)
    );
    let admitted = AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|e| panic!("{e}"));
    let failed: Vec<_> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir(SLIPS)))
        .into_report()
        .scenarios
        .into_iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| result.scenario.to_string())
        .collect();
    assert!(failed.is_empty(), "{failed:#?}");
}

#[test]
fn a_missing_row_branch_does_not_depend_on_input_names() {
    let expected = scenario_ids(&synthesis(SLIPS));
    for name in ["from", "zz_writer", "a_writer"] {
        let renamed = SLIPS.replace("writer", name);
        assert_ne!(renamed, SLIPS);
        let result = synthesis(&renamed);
        assert_eq!(refusals(&result), Vec::<String>::new(), "named `{name}`");
        assert_eq!(scenario_ids(&result), expected, "named `{name}`");
    }
}
