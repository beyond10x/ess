//! What synthesis derives from the model alone is derived once per model, or not at all where
//! nothing reads it, rather than once per command, branch or step that asks.
//!
//! Each case builds a model whose commands all ask the same question of the model and holds a
//! synthesis to a count of how often the answer was derived: a work bound rather than a clock.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

/// `commands` commands, each taking a sequence and `inputs` text inputs beside `extra` and
/// emitting `Filed`, with no guard on any branch.
fn filing(
    commands: usize,
    inputs: usize,
    event_fields: &str,
    extra: &str,
    payload: &str,
) -> String {
    let mut text = format!(
        "format: ess/37\nsystem: ticket\nversion: v1\ndomain: ticket.desk\nevents:\n  - name: \
         ticket.desk.Filed\n    fields:\n      - {{name: sequence, type: Integer}}\n{event_fields}\
         commands:\n"
    );
    for command in 0..commands {
        text.push_str(&format!(
            "  - name: ticket.desk.File{command:03}\n    input:\n      - {{name: sequence, type: \
             Integer}}\n{extra}"
        ));
        for input in 0..inputs {
            text.push_str(&format!("      - {{name: line_{input}, type: String}}\n"));
        }
        text.push_str(&format!(
            "    outcomes:\n      - name: filed\n        emits: [ticket.desk.Filed]\n        \
             payload:\n          ticket.desk.Filed: {{sequence: input.sequence{payload}}}\n"
        ));
    }
    text
}

/// One synthesis, with the admission checks the command line runs beside it, scans the model for
/// `Binary64` once. Unfixed, this run scanned it five times (admission, `used_by`, the uncarried
/// refusals, and the unreached notes twice), each formatting every field's position; a
/// command-line run scanned it seven times.
#[test]
fn a_model_is_scanned_for_binary64_once() {
    for (event_fields, extra, payload) in [
        ("", "", ""),
        (
            "      - {name: weight, type: Binary64}\n",
            "      - {name: weight, type: Binary64}\n",
            ", weight: input.weight",
        ),
    ] {
        let ir = ir(&filing(4, 2, event_fields, extra, payload));
        let (synthesis, scans) = ess_compiler::binary64::scans(|| {
            ess_conformance::admission::model(&ir).expect("the model is admitted");
            let _ = ess_conformance::binary64_values::used_by(&ir);
            synthesize(&ir)
        });
        assert!(
            refusals(&synthesis).is_empty(),
            "refusals: {:#?}",
            refusals(&synthesis)
        );
        assert_eq!(scans, 1, "the model was scanned for Binary64 {scans} times");
    }
}
