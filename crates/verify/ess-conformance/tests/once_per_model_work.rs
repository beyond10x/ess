//! What synthesis derives from the model alone is derived once per model, or not at all where
//! nothing reads it, rather than once per command, branch or step that asks.
//!
//! Each case builds a model whose commands all ask the same question of the model and holds a
//! synthesis to a count of how often the answer was derived: a work bound rather than a clock.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{
    absent_event_lists_built, precedence_commands_read, precedence_replays, synthesize, Synthesis,
};
use ess_conformance::ScenarioStep;
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

/// Twelve commands emitting one event of forty: the events none of them may publish are listed
/// once, for the one list of emitted events they share. Unfixed, once per command: 12.
#[test]
fn the_events_a_branch_must_not_publish_are_listed_once_per_emitted_list() {
    let mut text = String::from(
        "format: ess/25\nsystem: ticket\nversion: v1\ndomain: ticket.desk\ncommands:\n",
    );
    for command in 0..12 {
        text.push_str(&format!(
            "  - name: ticket.desk.Raise{command}\n    outcomes: [{{name: raised, emits: \
             [ticket.desk.Raised]}}]\n"
        ));
    }
    text.push_str("events:\n  - {name: ticket.desk.Raised, fields: []}\n");
    for event in 1..40 {
        text.push_str(&format!(
            "  - {{name: ticket.desk.Note{event}, fields: []}}\n"
        ));
    }
    let ir = ir(&text);
    let (synthesis, built) = absent_event_lists_built(|| synthesize(&ir));
    assert!(
        refusals(&synthesis).is_empty(),
        "refusals: {:#?}",
        refusals(&synthesis)
    );
    assert_eq!(synthesis.suite.scenarios.len(), 12);
    // One list per distinct emitted list: `[Raised]`, and at most the empty list a refusal path
    // reads.
    assert!(
        built <= 2,
        "the list was built {built} times for one emitted list"
    );
}

/// A command with no guard on any branch has nothing a sent input could contradict, so the
/// precedence check replays none of its inputs; a guarded command's input is still replayed.
/// Unfixed, each of the 8 scenarios' sent inputs was replayed: 8.
#[test]
fn an_unguarded_command_s_input_is_not_replayed_for_precedence() {
    let ir_plain = ir(&filing(8, 4, "", "", ""));
    let (synthesis, replays) = precedence_replays(|| synthesize(&ir_plain));
    assert!(
        refusals(&synthesis).is_empty(),
        "refusals: {:#?}",
        refusals(&synthesis)
    );
    assert_eq!(synthesis.suite.scenarios.len(), 8);
    assert_eq!(
        replays, 0,
        "{replays} inputs of unguarded commands were replayed"
    );

    let guarded = filing(1, 1, "", "", "").replace(
        "    outcomes:\n",
        "    outcomes:\n      - name: blank\n        when: line_0 == \"\"\n        error: \
         ticket.desk.Blank\n",
    ) + "errors:\n  - {name: ticket.desk.Blank, fields: []}\n";
    let ir_guarded = ir(&guarded);
    let (_, replays) = precedence_replays(|| synthesize(&ir_guarded));
    assert!(replays > 0, "the guarded command's input was not replayed");
}

/// Resolving the command a step sends reads that command, not every command declared before it.
/// Unfixed, the 32 steps over 32 commands read 1 + 2 + ... + 32 = 528 commands.
#[test]
fn a_step_s_command_is_resolved_by_its_name() {
    let ir = ir(&filing(32, 1, "", "", ""));
    let (synthesis, read) = precedence_commands_read(|| synthesize(&ir));
    assert!(
        refusals(&synthesis).is_empty(),
        "refusals: {:#?}",
        refusals(&synthesis)
    );
    let sent = u64::try_from(
        synthesis
            .suite
            .scenarios
            .values()
            .flat_map(|scenario| &scenario.steps)
            .filter(|step| matches!(step, ScenarioStep::ExecuteCommand { .. }))
            .count(),
    )
    .expect("the count fits");
    assert!(
        read <= sent,
        "{read} commands were read to resolve the {sent} steps that send one"
    );
}
