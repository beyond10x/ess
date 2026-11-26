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
//!
//! Two fixture models (`tests/fixtures/row-set-refusal-beside-*.yaml`) hold the refusing branch of
//! the same shape: the shelf named by a literal identity the scenario sends, with no subject, with
//! the rows created by the command under test, and as the subject the row-set command addresses
//! (story:a-row-set-refusal-is-witnessed-on-rows-whose-creator-needs-a-related-row-the-send-names).

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

/// A row-set refusal over rows whose creator refuses a missing shelf (`ess/37`): the shelf's
/// identity is a literal the scenario sends, `ClearShelf` addresses no subject, and `PinTicket`
/// refuses on a row set over the tickets it creates itself.
const NAMED: &str = include_str!("fixtures/row-set-refusal-beside-a-named-related-row.yaml");

/// The same shape where the row-set command addresses the shelf the rows name (`CloseShelf`,
/// `instance: shelf_id`), with no `owns` relation from shelf to ticket.
const ADDRESSED: &str =
    include_str!("fixtures/row-set-refusal-beside-the-addressed-related-row.yaml");

const CLEAR_PINNED: &str = "shop.store.ClearShelf/outcome/tickets-pinned";
const OUT_OF_ORDER: &str = "shop.store.PinTicket/outcome/out-of-order";
const CLOSE_PINNED: &str = "shop.store.CloseShelf/outcome/tickets-pinned";

/// `text` with `from`, which it holds once, replaced by `to`.
fn replaced(text: &str, from: &str, to: &str) -> String {
    assert_eq!(text.matches(from).count(), 1, "written once: {from}");
    text.replacen(from, to, 1)
}

/// [`ADDRESSED`] with the shelf's identity generated by the target, not sent as a literal.
fn generated_shelf() -> String {
    let text = replaced(
        ADDRESSED,
        "  - {name: shop.store.ShelfId, kind: newtype, of: String}\n",
        "  - {name: shop.store.ShelfId, kind: newtype, of: Uuid}\n",
    );
    let text = replaced(
        &text,
        "      - {name: shelf_id, type: shop.store.ShelfId}\n      - {name: title, type: String}\n",
        "      - {name: title, type: String}\n",
    );
    replaced(
        &text,
        "{shop.store.ShelfOpened: {shelf_id: input.shelf_id}}",
        "{shop.store.ShelfOpened: {shelf_id: {generated: true}}}",
    )
}

/// Every send of scenario `wanted`, in order: the command, its input as JSON, and the outcome the
/// step after it expects.
fn sends(run: &Run, wanted: &str) -> Vec<(String, serde_json::Value, String)> {
    let (_, scenario) = run
        .synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == wanted)
        .unwrap_or_else(|| panic!("{wanted} is synthesized"));
    let value = serde_json::to_value(scenario).unwrap();
    let steps = value["steps"].as_array().cloned().unwrap_or_default();
    steps
        .iter()
        .enumerate()
        .filter(|(_, step)| step["step"] == "execute_command")
        .map(|(at, step)| {
            let expected = steps
                .get(at + 1)
                .and_then(|next| next["outcome"]["outcome"].as_str())
                .unwrap_or_default();
            (
                step["command"].as_str().unwrap_or_default().to_owned(),
                step["input"].clone(),
                expected.to_owned(),
            )
        })
        .collect()
}

fn assert_refuses_nothing(run: &Run) {
    let all: Vec<String> = run
        .synthesis
        .refusals
        .iter()
        .map(|refusal| format!("[{}] {}", refusal.cause.code(), refusal.cause))
        .collect();
    assert!(
        all.is_empty(),
        "the model synthesizes without a refusal: {all:#?}"
    );
}

/// The tickets [`CLOSE_PINNED`] pins name the shelf its `CloseShelf` send addresses, the one shelf
/// it opens.
fn assert_rows_name_the_addressed_shelf(run: &Run) {
    let sent = sends(run, CLOSE_PINNED);
    let opened = sent
        .iter()
        .filter(|(command, _, _)| command == "shop.store.OpenShelf")
        .count();
    assert_eq!(
        opened, 1,
        "one shelf is opened, the one the send addresses: {sent:#?}"
    );
    let (last, input, outcome) = sent.last().expect("the scenario sends");
    assert_eq!(
        (last.as_str(), outcome.as_str()),
        ("shop.store.CloseShelf", "tickets-pinned"),
        "{sent:#?}"
    );
    let pinned: Vec<_> = sent
        .iter()
        .filter(|(command, _, outcome)| command == "shop.store.PinTicket" && outcome == "pinned")
        .collect();
    assert!(!pinned.is_empty(), "a ticket is pinned first: {sent:#?}");
    for (_, ticket, _) in pinned {
        assert_eq!(
            ticket["shelf_id"], input["shelf_id"],
            "the ticket names the shelf the send addresses: {sent:#?}"
        );
    }
}

#[test]
fn a_row_set_refusal_without_a_subject_is_witnessed_beside_the_shelf_the_send_names() {
    let run = run(NAMED);
    assert_passes(&run, CLEAR_PINNED);
    assert_refuses_nothing(&run);
    let sent = sends(&run, CLEAR_PINNED);
    assert!(
        sent.iter()
            .any(|(command, _, outcome)| command == "shop.store.PinTicket" && outcome == "pinned"),
        "a ticket is pinned before the shelf is cleared: {sent:#?}"
    );
}

#[test]
fn a_creators_own_row_set_refusal_is_witnessed_beside_the_shelf_the_send_names() {
    let run = run(NAMED);
    assert_passes(&run, OUT_OF_ORDER);
    let sent = sends(&run, OUT_OF_ORDER);
    let (last, _, outcome) = sent.last().expect("the scenario sends");
    assert_eq!(
        (last.as_str(), outcome.as_str()),
        ("shop.store.PinTicket", "out-of-order"),
        "{sent:#?}"
    );
    assert!(
        sent.iter()
            .any(|(command, _, outcome)| command == "shop.store.PinTicket" && outcome == "pinned"),
        "an earlier ticket is pinned first: {sent:#?}"
    );
}

#[test]
fn a_row_set_refusal_is_witnessed_beside_the_shelf_the_send_addresses() {
    let run = run(ADDRESSED);
    assert_passes(&run, CLOSE_PINNED);
    assert_refuses_nothing(&run);
    assert_rows_name_the_addressed_shelf(&run);
}

#[test]
fn with_a_generated_shelf_identity_the_rows_still_name_the_addressed_shelf() {
    let run = run(&generated_shelf());
    assert_passes(&run, CLOSE_PINNED);
    assert_refuses_nothing(&run);
    assert_rows_name_the_addressed_shelf(&run);
}

/// A creator guarded both by a related row its input names and by a row set over the rows it
/// creates itself (weft's catalog, 2026-11-26): `PublishOffering` refuses an engine no row carries
/// (`via: input.engine_id`) and a name an offering already has (`name-taken`), and otherwise
/// creates an `Offering` under a generated identity. Synthesis refused `name-taken`, and every
/// route through `published` was "a branch no input reaches", so no later command could act on an
/// offering.
const CATALOG: &str = r"format: ess/37
system: p
version: v1
domain: p.c
entities:
  - name: p.c.Engine
    identity: {name: engine_id, type: Uuid}
    fields:
      - {name: name, type: String}
    lifecycle: {initial: Registered, states: [Registered], terminal: [Registered]}
  - name: p.c.Offering
    identity: {name: offering_id, type: Uuid}
    fields:
      - {name: name, type: String}
      - {name: engine_id, type: Uuid}
    relations:
      - {name: engine, kind: references, target: p.c.Engine, cardinality: one, via: engine_id}
    lifecycle:
      initial: Published
      states: [Published, Retired]
      terminal: [Retired]
      transitions:
        - {name: retire, from: [Published], to: Retired}
errors:
  - name: p.c.NotFound
  - name: p.c.Taken
  - name: p.c.StateConflict
events:
  - name: p.c.Registered
    fields: [{name: engine_id, type: Uuid}]
  - name: p.c.Published
    fields: [{name: offering_id, type: Uuid}]
  - name: p.c.Retired
    fields: [{name: offering_id, type: Uuid}]
commands:
  - name: p.c.RegisterEngine
    input:
      - {name: name, type: String}
    outcomes:
      - name: registered
        creates: p.c.Engine
        instance: engine_id
        sets: {name: input.name}
        emits: [p.c.Registered]
        payload: {p.c.Registered: {engine_id: {generated: true}}}
  - name: p.c.PublishOffering
    input:
      - {name: name, type: String}
      - {name: engine_id, type: Uuid}
    outcomes:
      - name: no-such-engine
        when_related: {via: input.engine_id, exists: false}
        error: p.c.NotFound
      - name: name-taken
        when_related:
          entity: p.c.Offering
          where: name == input.name
          exists: true
        error: p.c.Taken
      - name: published
        creates: p.c.Offering
        instance: offering_id
        sets: {name: input.name, engine_id: input.engine_id}
        emits: [p.c.Published]
        payload: {p.c.Published: {offering_id: {generated: true}}}
  - name: p.c.RetireOffering
    input: [{name: offering_id, type: Uuid}]
    outcomes:
      - {name: no-such, unknown_instance: true, error: p.c.NotFound}
      - name: retired
        moves: p.c.Offering.retire
        instance: offering_id
        emits: [p.c.Retired]
        payload: {p.c.Retired: {offering_id: input.offering_id}}
      - {name: wrong-state, wrong_state: true, error: p.c.StateConflict}
views:
  - name: p.c.Offerings
    source: p.c.Offering
    consistency: read_your_writes
    fields:
      - {name: offering_id, type: Uuid}
      - {name: state, type: p.c.Offering.State}
";

#[test]
fn a_creator_beside_a_named_row_and_its_own_row_set_is_a_route_for_every_later_command() {
    let run = run(CATALOG);
    for wanted in [
        "p.c.PublishOffering/outcome/no-such-engine",
        "p.c.PublishOffering/outcome/name-taken",
        "p.c.PublishOffering/outcome/published",
        "p.c.RetireOffering/outcome/retired",
        "p.c.Offering/transition/retire/by/p.c.RetireOffering/retired",
        "p.c.Offering/state/Retired/refuses/p.c.RetireOffering",
    ] {
        assert_passes(&run, wanted);
    }
    assert_refuses_nothing(&run);
}

#[test]
fn the_name_taken_refusal_is_sent_the_name_an_offering_was_published_under() {
    let run = run(CATALOG);
    let wanted = "p.c.PublishOffering/outcome/name-taken";
    assert_passes(&run, wanted);
    let sent = sends(&run, wanted);
    let (last, input, outcome) = sent.last().expect("the scenario sends");
    assert_eq!(
        (last.as_str(), outcome.as_str()),
        ("p.c.PublishOffering", "name-taken"),
        "{sent:#?}"
    );
    assert!(
        sent.iter().any(
            |(command, earlier, outcome)| command == "p.c.PublishOffering"
                && outcome == "published"
                && earlier["name"] == input["name"]
        ),
        "an offering of the same name is published first: {sent:#?}"
    );
}
