//! A row-set branch arranges rows past the creator's own guard (beyond10x/ess#682).
//!
//! `StoreBottle` creates a `Bottle` whose `rack_id` it sets from `input.rack_id`, and first refuses
//! with `when_related: {via: input.rack_id, exists: false}` where no such rack exists. `CheckRack`
//! refuses with `empty` while no bottle's `rack_id` is `input.rack_id`, and otherwise answers
//! `holds-bottles`. Synthesis arranged the bottle through `StoreBottle` against a literal rack
//! identity, the creator's guard then pointed the bottle at a rack of its own, and no arranged row
//! was selected: `holds-bottles` was refused as ESS-SYNTH-001. Without the guard it was synthesized.
//!
//! Where no rack can be arranged at all, because no command creates one, the creator's guard can
//! never be passed, and the refusal names that guard rather than asking for the bottle's fields to
//! be written from an input, which `StoreBottle` already does.
//!
//! The model is inline, so it is no repository model the suite-bytes tables walk; the unguarded
//! and the rackless models are the same text with parts of it taken out.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::{ScenarioResult, Status};
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

/// The guard on the creator, as [`GUARDED`] writes it.
const GUARD: &str = "      - name: no-rack
        when_related: {via: input.rack_id, exists: false}
        error: demo.cellar.NoSuchRack
";

const GUARDED: &str = r#"format: ess/29
system: demo
version: v1
domain: demo.cellar
summary: A bottle is stored only in a rack that exists; a check asks whether a rack holds any bottle.
types:
  - {name: demo.cellar.RackId, kind: newtype, of: Uuid}
  - {name: demo.cellar.BottleId, kind: newtype, of: Uuid}
entities:
  - name: demo.cellar.Rack
    identity: {name: rack_id, type: demo.cellar.RackId}
    fields:
      - {name: title, type: String}
    lifecycle: {initial: Standing, states: [Standing], terminal: [Standing]}
  - name: demo.cellar.Bottle
    identity: {name: bottle_id, type: demo.cellar.BottleId}
    fields:
      - {name: rack_id, type: demo.cellar.RackId}
      - {name: label, type: String}
    lifecycle: {initial: Resting, states: [Resting], terminal: [Resting]}
errors:
  - {name: demo.cellar.NoSuchRack, summary: No rack carries this identity., fields: []}
  - {name: demo.cellar.Empty, summary: The rack holds no bottle., fields: []}
events:
  - name: demo.cellar.RackBuilt
    fields: [{name: rack_id, type: demo.cellar.RackId}]
  - name: demo.cellar.BottleStored
    fields: [{name: bottle_id, type: demo.cellar.BottleId}]
actors:
  - name: demo.cellar.Keeper
    may: [demo.cellar.BuildRack, demo.cellar.StoreBottle, demo.cellar.CheckRack]
commands:
  - name: demo.cellar.BuildRack
    input:
      - {name: title, type: String}
    outcomes:
      - name: built
        creates: demo.cellar.Rack
        instance: rack_id
        emits: [demo.cellar.RackBuilt]
        payload: {demo.cellar.RackBuilt: {rack_id: {generated: true}}}
        sets: {title: input.title}
  - name: demo.cellar.StoreBottle
    input:
      - {name: rack_id, type: demo.cellar.RackId}
      - {name: label, type: String}
    outcomes:
      - name: no-rack
        when_related: {via: input.rack_id, exists: false}
        error: demo.cellar.NoSuchRack
      - name: stored
        creates: demo.cellar.Bottle
        instance: bottle_id
        emits: [demo.cellar.BottleStored]
        payload: {demo.cellar.BottleStored: {bottle_id: {generated: true}}}
        sets: {rack_id: input.rack_id, label: input.label}
  - name: demo.cellar.CheckRack
    input:
      - {name: rack_id, type: demo.cellar.RackId}
    outcomes:
      - name: empty
        when_related:
          entity: demo.cellar.Bottle
          where: "rack_id == input.rack_id"
          exists: false
        error: demo.cellar.Empty
      - {name: holds-bottles, accepts: nothing}
views:
  - name: demo.cellar.Bottles
    source: demo.cellar.Bottle
    consistency: read_your_writes
    fields:
      - {name: bottle_id, type: demo.cellar.BottleId}
      - {name: rack_id, type: demo.cellar.RackId}
      - {name: label, type: String}
"#;

/// The accepting branch of the row-set command, whose scenario was lost.
const HOLDS: &str = "demo.cellar.CheckRack/outcome/holds-bottles";

/// The refusing branch of the row-set command, synthesized before and after.
const EMPTY: &str = "demo.cellar.CheckRack/outcome/empty";

/// The command creating racks, as [`GUARDED`] writes it.
const BUILD_RACK: &str = "  - name: demo.cellar.BuildRack
    input:
      - {name: title, type: String}
    outcomes:
      - name: built
        creates: demo.cellar.Rack
        instance: rack_id
        emits: [demo.cellar.RackBuilt]
        payload: {demo.cellar.RackBuilt: {rack_id: {generated: true}}}
        sets: {title: input.title}
";

/// The event only [`BUILD_RACK`] emits, as [`GUARDED`] writes it.
const RACK_BUILT: &str = "  - name: demo.cellar.RackBuilt
    fields: [{name: rack_id, type: demo.cellar.RackId}]
";

/// `text` without `part`, which it holds once.
fn without(text: &str, part: &str) -> String {
    assert_eq!(text.matches(part).count(), 1, "written once: {part}");
    text.replacen(part, "", 1)
}

/// [`GUARDED`] without the creator's guard, and nothing else changed.
fn unguarded() -> String {
    without(GUARDED, GUARD)
}

/// [`GUARDED`] without the command creating racks, the event it emits and its grant: no rack can
/// be arranged, so the creator's guard is never passed.
fn rackless() -> String {
    let text = without(GUARDED, BUILD_RACK);
    let text = without(&text, RACK_BUILT);
    without(&text, "demo.cellar.BuildRack, ")
}

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

/// Every refusal naming `wanted`, as it prints.
fn refused(run: &Run, wanted: &str) -> Vec<String> {
    run.synthesis
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
        .map(|refusal| format!("[{}] {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn assert_passes(run: &Run, wanted: &str) {
    let refusals = refused(run, wanted);
    assert!(
        refusals.is_empty(),
        "{wanted} is not refused: {refusals:#?}"
    );
    let result = run.results.get(wanted).unwrap_or_else(|| {
        panic!(
            "{wanted} is synthesized: {:#?}",
            run.results.keys().collect::<Vec<_>>()
        )
    });
    if result.status != Status::Passed {
        let scenario = run
            .synthesis
            .suite
            .scenarios
            .iter()
            .find(|(id, _)| id.to_string() == wanted)
            .map(|(_, scenario)| serde_json::to_string_pretty(scenario).unwrap());
        panic!(
            "{wanted} passes on the interpreted target:\n{:#?}\n{}",
            result.diagnostics().collect::<Vec<_>>(),
            scenario.unwrap_or_default()
        );
    }
}

#[test]
fn the_accepting_branch_is_synthesized_past_the_creators_guard_and_passes() {
    let run = run(GUARDED);
    assert_passes(&run, HOLDS);
}

#[test]
fn the_refusing_branch_still_passes_beside_the_creators_guard() {
    let run = run(GUARDED);
    assert_passes(&run, EMPTY);
}

#[test]
fn without_the_guard_the_same_scenarios_are_synthesized_and_pass() {
    let run = run(&unguarded());
    assert_passes(&run, HOLDS);
    assert_passes(&run, EMPTY);
}

#[test]
fn the_guard_is_the_only_difference_between_the_two_models() {
    let guarded = run(GUARDED);
    let unguarded = run(&unguarded());
    let ids = |run: &Run| -> Vec<String> {
        run.synthesis
            .suite
            .scenarios
            .keys()
            .map(ToString::to_string)
            .filter(|id| id.starts_with("demo.cellar.CheckRack/"))
            .collect()
    };
    assert_eq!(
        ids(&guarded),
        ids(&unguarded),
        "the row-set command has the same scenarios whether or not the creator is guarded"
    );
}

#[test]
fn where_no_related_row_can_be_arranged_the_refusal_names_the_creators_guard() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(&rackless()));
    let refusals: Vec<_> = synthesis
        .refusals
        .iter()
        .filter(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(ToString::to_string)
                .as_deref()
                == Some(HOLDS)
        })
        .collect();
    let [refusal] = refusals.as_slice() else {
        panic!("{HOLDS} is refused once: {refusals:#?}");
    };
    let message = refusal.cause.to_string();
    for named in [
        "demo.cellar.StoreBottle",
        "`no-rack`",
        "demo.cellar.Rack",
        "when_related: {via: input.rack_id, exists: false}",
    ] {
        assert!(
            message.contains(named),
            "the refusal names {named}: {message}"
        );
    }
    let hint = refusal.hint();
    assert!(
        hint.contains("related row") && !hint.contains("caller attribute"),
        "the help names the related row, not the creator's input: {hint}"
    );
    assert_eq!(
        refusal.code().to_string(),
        "ESS-SYNTH-001",
        "the refusal keeps its code: {message}"
    );
}
