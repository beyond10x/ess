//! A set update's rows-change witness past what the rows' creator does with the filtered value.
//!
//! A set update (`updates:` with `instances: {where: <field> == input.<x>}`) is witnessed on three
//! rows its filter selects, each holding a value the update changes. Synthesis refused it as
//! ESS-SYNTH-001 ("no three rows the filter selects that the arranging commands can produce") in
//! three shapes, each a fixture under `tests/fixtures/`:
//!
//! - The creating command copies a field from the related row the filtered field names
//!   (`flag: {related: {via: input.parent_id, field: flag}}`), with or without a
//!   `when_related: {via: input.parent_id, exists: false}` refusal beside it: the rows were created
//!   naming a related row that did not exist, so the copied value was unknown.
//! - The creating command refuses a missing related row through the input it sets the filtered
//!   field from: the rows were created naming a related row that did not exist, and refused.
//! - The creating command writes the field the update writes, from a literal or from its own
//!   input, and the update writes it from its input: both were sent the same value, so no
//!   selected row would change.
//!
//! In the first two shapes the rows are now created naming a related row arranged first, and the
//! update is sent naming it; in the third the update's input is varied against what the rows hold.
//! Where the refusal still fires, its help names the obstacle rather than asking for the filtered
//! field to be written from an input, which these creators already do.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// The creator copies the filtered row's flag from the parent it names.
const COPIES: &str = include_str!("fixtures/set-update-creator-copies-related-value.yaml");

/// [`COPIES`], with the creator refusing a missing parent and the update writing its input.
const COPIES_GUARDED: &str =
    include_str!("fixtures/set-update-creator-copies-guarded-related-value.yaml");

/// The creator refuses a missing shelf through the input the filter compares.
const REFUSES_MISSING: &str =
    include_str!("fixtures/set-update-creator-refuses-missing-related-row.yaml");

/// [`REFUSES_MISSING`] without the creator's refusal.
const WITHOUT_REFUSAL: &str =
    include_str!("fixtures/set-update-creator-without-related-refusal.yaml");

/// The creator writes `flag: true`; the update writes `input.flag`.
const LITERAL_CREATOR: &str =
    include_str!("fixtures/set-update-written-input-against-literal-creator.yaml");

/// The creator and the update both write `flag` from their own input.
const CREATOR_INPUT: &str =
    include_str!("fixtures/set-update-written-input-against-creator-input.yaml");

const SYNCED: &str = "demo.z.SyncFlag/outcome/synced";
const RELABELLED: &str = "demo.depot.RelabelShelf/outcome/relabelled";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("spec.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

struct Run {
    synthesis: Synthesis,
    results: BTreeMap<String, ScenarioResult>,
}

fn run(text: &str) -> Run {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let results = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result))
        .collect();
    Run { synthesis, results }
}

/// Every refusal of the model, as it prints.
fn refusals(run: &Run) -> Vec<String> {
    run.synthesis
        .refusals
        .iter()
        .map(|refusal| {
            format!(
                "{:?} [{}] {}",
                refusal.scenario.as_ref().map(ToString::to_string),
                refusal.cause.code(),
                refusal.cause
            )
        })
        .collect()
}

/// The model synthesizes with no refusal, `wanted` among its scenarios, and every scenario passes
/// the interpreted target.
fn assert_witnessed(text: &str, wanted: &str) {
    let run = run(text);
    let refused = refusals(&run);
    assert!(refused.is_empty(), "no refusal: {refused:#?}");
    assert!(
        run.results.contains_key(wanted),
        "{wanted} is synthesized: {:#?}",
        run.results.keys().collect::<Vec<_>>()
    );
    for (id, result) in &run.results {
        if result.status != Status::Passed {
            let scenario = run
                .synthesis
                .suite
                .scenarios
                .iter()
                .find(|(scenario, _)| scenario.to_string() == *id)
                .map(|(_, scenario)| serde_json::to_string_pretty(scenario).unwrap());
            panic!(
                "{id} passes on the interpreted target:\n{:#?}\n{}",
                result.diagnostics().collect::<Vec<_>>(),
                scenario.unwrap_or_default()
            );
        }
    }
}

/// The one refusal of `wanted`, with its rendered cause and its help.
fn refusal_of(text: &str, wanted: &str) -> (String, String) {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(text));
    let found: Vec<_> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(wanted)
        })
        .collect();
    let [refusal] = found.as_slice() else {
        panic!("{wanted} is refused once: {found:#?}");
    };
    assert_eq!(refusal.code().to_string(), "ESS-SYNTH-001", "{refusal}");
    (refusal.cause.to_string(), refusal.hint().to_string())
}

/// `text` with `part`, which it holds once, replaced by `with`.
fn replaced(text: &str, part: &str, with: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, with, 1)
}

#[test]
fn a_set_update_arranges_rows_whose_value_the_creator_copies_from_a_related_row() {
    assert_witnessed(COPIES, SYNCED);
}

#[test]
fn a_set_update_arranges_rows_whose_copied_value_the_creator_also_guards() {
    assert_witnessed(COPIES_GUARDED, SYNCED);
}

#[test]
fn a_set_update_arranges_rows_past_a_creators_exists_false_branch() {
    assert_witnessed(REFUSES_MISSING, RELABELLED);
}

#[test]
fn without_the_creators_refusal_the_set_update_is_witnessed_as_before() {
    assert_witnessed(WITHOUT_REFUSAL, RELABELLED);
}

#[test]
fn a_set_update_witness_varies_the_creators_input_against_the_written_value() {
    assert_witnessed(LITERAL_CREATOR, SYNCED);
    assert_witnessed(CREATOR_INPUT, SYNCED);
}

/// No command creates a parent, so no child can be created naming one: the refusal names the
/// related row the creator reads, not the creator's input.
#[test]
fn where_no_related_row_can_be_arranged_the_refusal_names_it() {
    let parentless = replaced(
        COPIES,
        "  - name: demo.z.CreateParent
    naming: {wire: create-parent, display: Create parent}
    input: []
    outcomes:
      - name: created
        creates: demo.z.Parent
        instance: parent_id
        emits: [demo.z.Created]
        payload: {demo.z.Created: {parent_id: {generated: true}}}
        sets: {flag: false}
        summary: A parent is created.
",
        "",
    );
    let parentless = replaced(
        &parentless,
        "[demo.z.CreateParent, demo.z.AddChild",
        "[demo.z.AddChild",
    );
    let parentless = replaced(
        &parentless,
        "  - {name: demo.z.Created, fields: [{name: parent_id, type: demo.z.ParentId}]}\n",
        "",
    );
    let (cause, help) = refusal_of(&parentless, SYNCED);
    assert!(cause.contains("related row"), "{cause}");
    assert!(
        help.contains("related row") && !help.contains("caller attribute"),
        "the help names the related row, not the creator's input: {help}"
    );
}

/// The update and the creator both write `flag: true`: every row the filter selects already holds
/// what the update writes, and the refusal says so rather than asking for an input.
#[test]
fn where_every_selected_row_already_holds_the_written_value_the_refusal_says_so() {
    let unchanged = replaced(
        LITERAL_CREATOR,
        "        sets: {flag: input.flag}\n",
        "        sets: {flag: true}\n",
    );
    let unchanged = replaced(
        &unchanged,
        "      - {name: flag, type: Boolean}\n    outcomes:\n      - name: synced",
        "    outcomes:\n      - name: synced",
    );
    let (cause, help) = refusal_of(&unchanged, SYNCED);
    assert!(cause.contains("changes"), "{cause}");
    assert!(
        help.contains("already") && !help.contains("caller attribute"),
        "the help names the value the rows already hold: {help}"
    );
}
