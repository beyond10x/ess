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

/// The creator [`CATALOG`] fixed, now a route for a command that addresses the row it creates and
/// refuses a missing row through two inputs (weft's formations, 2026-11-26): `AddToBorder` updates
/// the component `input.component_id` names and refuses first a missing formation, then a missing
/// component. The component is arranged through `AddComponent`, which refuses a name already taken
/// in its formation. Under the one rule the addressed component's existence is read first, and
/// `AddToBorder` declares no `unknown_instance:`, so a send naming no component reaches no declared
/// outcome: the `no-such-formation` scenario sent both rows missing beside its own send, and the
/// interpreted target failed it.
const FORMATIONS: &str = r#"format: ess/37
system: p
version: v1
domain: p.f
types:
  - {name: p.f.FormationId, kind: newtype, of: Uuid}
  - {name: p.f.ComponentId, kind: newtype, of: Uuid}
entities:
  - name: p.f.Formation
    identity: {name: formation_id, type: p.f.FormationId}
    fields:
      - {name: name, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: p.f.Component
    identity: {name: component_id, type: p.f.ComponentId}
    fields:
      - {name: formation_id, type: p.f.FormationId}
      - {name: name, type: String}
      - {name: on_border, type: Boolean}
    lifecycle: {initial: Declared, states: [Declared], terminal: [Declared]}
errors:
  - name: p.f.NoSuchFormation
  - name: p.f.NoSuchComponent
  - name: p.f.NameTaken
events:
  - name: p.f.Created
    fields: [{name: formation_id, type: p.f.FormationId}]
  - name: p.f.Added
    fields: [{name: component_id, type: p.f.ComponentId}]
  - name: p.f.Bordered
    fields: [{name: component_id, type: p.f.ComponentId}]
commands:
  - name: p.f.CreateFormation
    input:
      - {name: name, type: String}
    outcomes:
      - name: created
        creates: p.f.Formation
        instance: formation_id
        sets: {name: input.name}
        emits: [p.f.Created]
        payload: {p.f.Created: {formation_id: {generated: true}}}
  - name: p.f.AddComponent
    input:
      - {name: formation_id, type: p.f.FormationId}
      - {name: name, type: String}
    outcomes:
      - name: no-such-formation
        when_related: {via: input.formation_id, exists: false}
        error: p.f.NoSuchFormation
      - name: name-taken
        when_related:
          entity: p.f.Component
          where: {all: [formation_id == input.formation_id, name == input.name]}
          exists: true
        error: p.f.NameTaken
      - name: added
        creates: p.f.Component
        instance: component_id
        sets: {formation_id: input.formation_id, name: input.name, on_border: false}
        emits: [p.f.Added]
        payload: {p.f.Added: {component_id: {generated: true}}}
  - name: p.f.AddToBorder
    input:
      - {name: formation_id, type: p.f.FormationId}
      - {name: component_id, type: p.f.ComponentId}
    outcomes:
      - name: no-such-formation
        when_related: {via: input.formation_id, exists: false}
        error: p.f.NoSuchFormation
      - name: no-such-component
        when_related: {via: input.component_id, exists: false}
        error: p.f.NoSuchComponent
      - name: foreign-component
        when_related: {via: input.component_id, predicate: formation_id != input.formation_id}
        error: p.f.NoSuchComponent
      - name: bordered
        updates: p.f.Component
        instance: component_id
        sets: {on_border: true}
        emits: [p.f.Bordered]
        payload: {p.f.Bordered: {component_id: input.component_id}}
  - name: p.f.RemoveFromBorder
    input:
      - {name: formation_id, type: p.f.FormationId}
      - {name: component_id, type: p.f.ComponentId}
    outcomes:
      - name: no-such-formation
        when_related: {via: input.formation_id, exists: false}
        error: p.f.NoSuchFormation
      - name: no-such-component
        when_related: {via: input.component_id, exists: false}
        error: p.f.NoSuchComponent
      - name: foreign-component
        when_related: {via: input.component_id, predicate: formation_id != input.formation_id}
        error: p.f.NoSuchComponent
      - name: namesake
        when_related:
          entity: p.f.Component
          where: {all: [formation_id == input.formation_id, name == "main"]}
          exists: true
        error: p.f.NameTaken
      - name: unbordered
        updates: p.f.Component
        instance: component_id
        sets: {on_border: false}
        emits: [p.f.Bordered]
        payload: {p.f.Bordered: {component_id: input.component_id}}
views:
  - name: p.f.Components
    source: p.f.Component
    consistency: read_your_writes
    fields:
      - {name: component_id, type: p.f.ComponentId}
      - {name: on_border, type: Boolean}
"#;

/// The `exists: false` branches over the component each command addresses: never reached under
/// the one rule, as `AddToBorder`'s already was. `RemoveFromBorder`'s, beside a row set, was sent
/// an arranged component and failed on the interpreted target.
const ADDRESSED_ABSENT: [&str; 2] = [
    "p.f.AddToBorder/outcome/no-such-component",
    "p.f.RemoveFromBorder/outcome/no-such-component",
];

#[test]
fn every_scenario_through_the_fixed_creator_passes_its_own_interpreter() {
    let run = run(FORMATIONS);
    assert_passes(&run, "p.f.AddToBorder/outcome/no-such-formation");
    assert_passes(&run, "p.f.RemoveFromBorder/outcome/no-such-formation");
    let failed: Vec<_> = run
        .results
        .iter()
        .filter(|(_, result)| result.status != Status::Passed)
        .map(|(id, result)| (id, result.diagnostics().collect::<Vec<_>>()))
        .collect();
    assert!(
        failed.is_empty(),
        "every synthesized scenario passes the interpreted target: {failed:#?}"
    );
}

#[test]
fn an_absent_branch_over_the_addressed_row_is_refused_as_never_reached() {
    let run = run(FORMATIONS);
    for wanted in ADDRESSED_ABSENT {
        assert!(
            !run.results.contains_key(wanted),
            "{wanted} is not synthesized"
        );
        let refusals = refused(&run, wanted);
        assert!(
            refusals.len() == 1 && refusals[0].contains("is answered by that row's existence"),
            "{wanted} is refused as never reached: {refusals:#?}"
        );
    }
}

#[test]
fn no_send_names_a_component_no_row_carries() {
    let run = run(FORMATIONS);
    let wanted = "p.f.AddToBorder/outcome/no-such-formation";
    assert_passes(&run, wanted);
    for (command, input, outcome) in sends(&run, wanted) {
        if command == "p.f.AddToBorder" {
            assert_eq!(
                input["component_id"]["kind"], "instance",
                "the addressed component is an arranged row ({outcome}): {input:#}"
            );
        }
    }
}

/// A row-set refusal that holds over no rows still answers a run that arranges no row for it
/// (weft's formations, 2026-11-26): `CommitRevision` refuses a formation with no component
/// (`empty`, `exists: false`) and otherwise creates a `Revision` the formation owns. Passing every
/// row-set refusal over, a run arranging a revision sent `committed` on a formation with no
/// component: the revision's own scenario was refused, and the one-time disclosure scenario that
/// carries such a run, which no row-set walk reads, failed on the interpreted target.
const REVISIONS: &str = r"format: ess/37
system: p
version: v1
domain: p.f
types:
  - {name: p.f.FormationId, kind: newtype, of: Uuid}
  - {name: p.f.ComponentId, kind: newtype, of: Uuid}
  - {name: p.f.RevisionId, kind: newtype, of: Uuid}
  - {name: p.f.KeyId, kind: newtype, of: Uuid}
entities:
  - name: p.f.Key
    identity: {name: key_id, type: p.f.KeyId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Issued, states: [Issued], terminal: [Issued]}
  - name: p.f.Formation
    identity: {name: formation_id, type: p.f.FormationId}
    fields:
      - {name: name, type: String}
    relations:
      - {name: revisions, kind: owns, target: p.f.Revision, cardinality: many, via: formation_id}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: p.f.Component
    identity: {name: component_id, type: p.f.ComponentId}
    fields:
      - {name: formation_id, type: p.f.FormationId}
      - {name: name, type: String}
    lifecycle: {initial: Declared, states: [Declared], terminal: [Declared]}
  - name: p.f.Revision
    identity: {name: revision_id, type: p.f.RevisionId}
    fields:
      - {name: formation_id, type: p.f.FormationId}
    lifecycle: {initial: Committed, states: [Committed], terminal: [Committed]}
errors:
  - name: p.f.NoSuchFormation
  - name: p.f.Empty
events:
  - name: p.f.KeyIssued
    fields: [{name: key_id, type: p.f.KeyId}]
  - name: p.f.Created
    fields: [{name: formation_id, type: p.f.FormationId}]
  - name: p.f.Added
    fields: [{name: component_id, type: p.f.ComponentId}]
  - name: p.f.Committed
    fields: [{name: revision_id, type: p.f.RevisionId}]
commands:
  - name: p.f.IssueKey
    input:
      - {name: label, type: String}
    response:
      - {name: key_id, type: p.f.KeyId}
      - {name: secret, type: String}
    outcomes:
      - name: issued
        creates: p.f.Key
        instance: key_id
        returns: true
        one_time_response: [secret]
        sets: {label: input.label}
        emits: [p.f.KeyIssued]
        payload: {p.f.KeyIssued: {key_id: {response: key_id}}}
  - name: p.f.CreateFormation
    input:
      - {name: name, type: String}
    outcomes:
      - name: created
        creates: p.f.Formation
        instance: formation_id
        sets: {name: input.name}
        emits: [p.f.Created]
        payload: {p.f.Created: {formation_id: {generated: true}}}
  - name: p.f.AddComponent
    input:
      - {name: formation_id, type: p.f.FormationId}
      - {name: name, type: String}
    outcomes:
      - name: no-such-formation
        when_related: {via: input.formation_id, exists: false}
        error: p.f.NoSuchFormation
      - name: added
        creates: p.f.Component
        instance: component_id
        sets: {formation_id: input.formation_id, name: input.name}
        emits: [p.f.Added]
        payload: {p.f.Added: {component_id: {generated: true}}}
  - name: p.f.CommitRevision
    input:
      - {name: formation_id, type: p.f.FormationId}
    outcomes:
      - name: no-such-formation
        when_related: {via: input.formation_id, exists: false}
        error: p.f.NoSuchFormation
      - name: empty
        when_related:
          entity: p.f.Component
          where: formation_id == input.formation_id
          exists: false
        error: p.f.Empty
      - name: committed
        creates: p.f.Revision
        instance: revision_id
        sets: {formation_id: input.formation_id}
        emits: [p.f.Committed]
        payload: {p.f.Committed: {revision_id: {generated: true}}}
views:
  - name: p.f.Revisions
    source: p.f.Revision
    consistency: read_your_writes
    params:
      - {name: formation_id, type: p.f.FormationId}
    filter: formation_id == param.formation_id
    fields:
      - {name: revision_id, type: p.f.RevisionId}
      - {name: formation_id, type: p.f.FormationId}
";

#[test]
fn a_row_set_refusal_that_holds_over_no_rows_still_answers_an_arranging_run() {
    let run = run(REVISIONS);
    assert_refuses_nothing(&run);
    assert_every_scenario_passes(&run);
}

/// Every scenario of `run` passes the interpreted target.
fn assert_every_scenario_passes(run: &Run) {
    let failed: Vec<_> = run
        .results
        .iter()
        .filter(|(_, result)| result.status != Status::Passed)
        .map(|(id, result)| (id, result.diagnostics().collect::<Vec<_>>()))
        .collect();
    assert!(
        failed.is_empty(),
        "every synthesized scenario passes the interpreted target: {failed:#?}"
    );
}

/// A row set over the addressed row itself, beside a one-time response (weft's accounts,
/// 2026-11-26): `Revoke` moves the binding `input.binding_id` names and refuses it first where
/// that binding's tenant is not `input.tenant_id` (a row set selecting the binding by its
/// identity). The one-time disclosure scenarios carry the runs of other scenarios, arranged
/// again, and were built after the row-set walk: one sent `revoked` with a tenant the binding does
/// not hold, which `other-tenant` answers, and failed on the interpreted target. On main too.
const BINDINGS: &str = r"format: ess/37
system: p
version: v1
domain: p.t
types:
  - {name: p.t.TenantId, kind: newtype, of: Uuid}
  - {name: p.t.BindingId, kind: newtype, of: Uuid}
  - {name: p.t.KeyId, kind: newtype, of: Uuid}
entities:
  - name: p.t.Key
    identity: {name: key_id, type: p.t.KeyId}
    fields:
      - {name: label, type: String}
    lifecycle: {initial: Issued, states: [Issued], terminal: [Issued]}
  - name: p.t.Tenant
    identity: {name: tenant_id, type: p.t.TenantId}
    fields:
      - {name: name, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: p.t.Binding
    identity: {name: binding_id, type: p.t.BindingId}
    fields:
      - {name: tenant_id, type: p.t.TenantId}
    lifecycle:
      initial: Active
      states: [Active, Revoked]
      terminal: [Revoked]
      transitions:
        - {name: revoke, from: [Active], to: Revoked}
errors:
  - name: p.t.NotFound
  - name: p.t.CrossTenant
  - name: p.t.StateConflict
events:
  - name: p.t.KeyIssued
    fields: [{name: key_id, type: p.t.KeyId}]
  - name: p.t.Created
    fields: [{name: tenant_id, type: p.t.TenantId}]
  - name: p.t.Bound
    fields: [{name: binding_id, type: p.t.BindingId}]
  - name: p.t.Revoked
    fields: [{name: binding_id, type: p.t.BindingId}]
commands:
  - name: p.t.IssueKey
    input:
      - {name: label, type: String}
    response:
      - {name: key_id, type: p.t.KeyId}
      - {name: secret, type: String}
    outcomes:
      - name: issued
        creates: p.t.Key
        instance: key_id
        returns: true
        one_time_response: [secret]
        sets: {label: input.label}
        emits: [p.t.KeyIssued]
        payload: {p.t.KeyIssued: {key_id: {response: key_id}}}
  - name: p.t.CreateTenant
    input:
      - {name: name, type: String}
    outcomes:
      - name: created
        creates: p.t.Tenant
        instance: tenant_id
        sets: {name: input.name}
        emits: [p.t.Created]
        payload: {p.t.Created: {tenant_id: {generated: true}}}
  - name: p.t.Bind
    input:
      - {name: tenant_id, type: p.t.TenantId}
    outcomes:
      - name: no-such-tenant
        when_related: {via: input.tenant_id, exists: false}
        error: p.t.NotFound
      - name: bound
        creates: p.t.Binding
        instance: binding_id
        sets: {tenant_id: input.tenant_id}
        emits: [p.t.Bound]
        payload: {p.t.Bound: {binding_id: {generated: true}}}
  - name: p.t.Revoke
    input:
      - {name: binding_id, type: p.t.BindingId}
      - {name: tenant_id, type: p.t.TenantId}
    outcomes:
      - {name: no-such-binding, unknown_instance: true, error: p.t.NotFound}
      - name: other-tenant
        when_related:
          entity: p.t.Binding
          where: {all: [binding_id == input.binding_id, tenant_id != input.tenant_id]}
          exists: true
        error: p.t.CrossTenant
      - name: revoked
        moves: p.t.Binding.revoke
        instance: binding_id
        emits: [p.t.Revoked]
        payload: {p.t.Revoked: {binding_id: input.binding_id}}
      - {name: wrong-state, wrong_state: true, error: p.t.StateConflict}
views:
  - name: p.t.Bindings
    source: p.t.Binding
    consistency: read_your_writes
    fields:
      - {name: binding_id, type: p.t.BindingId}
      - {name: state, type: p.t.Binding.State}
";

#[test]
fn a_disclosure_scenario_is_held_to_the_row_sets_its_sends_read() {
    assert_every_scenario_passes(&run(BINDINGS));
}

/// Scopes held by `when:` guards over Optional references, before `via:` guards over the same
/// references and a row set (weft's `CreateSecret`, 2026-11-26): `scope` says which of
/// `formation_id` and `environment_id` are sent, six `when:` refusals hold the scope to them, then
/// each reference's row must exist and nest, then `name-taken` refuses a name in use.
///
/// - `no-such-environment` was also sent with `formation_id` left out and `scope: Environment`,
///   which `environment-scope-without-formation` answers first, so the scenario failed.
/// - `created` had no scenario: a row was sought for every reference, also for the ones the
///   `Account` scope leaves out, where an absent Optional reference reads no row and selects no
///   `when_related` branch (the interpreter's `related_absent` and `held_row`).
const SECRETS: &str = r"format: ess/37
system: p
version: v1
domain: p.s
types:
  - {name: p.s.Scope, kind: enum, variants: [Account, Formation, Environment]}
entities:
  - name: p.s.Account
    identity: {name: account_id, type: Uuid}
    fields:
      - {name: title, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: p.s.Formation
    identity: {name: formation_id, type: Uuid}
    fields:
      - {name: account_id, type: Uuid}
    relations:
      - {name: account, kind: references, target: p.s.Account, cardinality: one, via: account_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: p.s.Environment
    identity: {name: environment_id, type: Uuid}
    fields:
      - {name: formation_id, type: Uuid}
    relations:
      - {name: formation, kind: references, target: p.s.Formation, cardinality: one, via: formation_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: p.s.Secret
    identity: {name: secret_id, type: Uuid}
    fields:
      - {name: account_id, type: Uuid}
      - {name: scope, type: p.s.Scope}
      - {name: formation_id, type: Optional<Uuid>}
      - {name: environment_id, type: Optional<Uuid>}
      - {name: name, type: String}
    relations:
      - {name: account, kind: references, target: p.s.Account, cardinality: one, via: account_id}
      - {name: formation, kind: references, target: p.s.Formation, cardinality: one, via: formation_id}
      - {name: environment, kind: references, target: p.s.Environment, cardinality: one, via: environment_id}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
errors:
  - {name: p.s.ScopeMismatch, summary: The scope and the references disagree., fields: []}
  - {name: p.s.ScopeNotFound, summary: No such scope., fields: []}
  - {name: p.s.NameTaken, summary: The name is taken., fields: []}
events:
  - name: p.s.AccountOpened
    fields: [{name: account_id, type: Uuid}]
  - name: p.s.FormationOpened
    fields: [{name: formation_id, type: Uuid}]
  - name: p.s.EnvironmentOpened
    fields: [{name: environment_id, type: Uuid}]
  - name: p.s.SecretCreated
    fields: [{name: secret_id, type: Uuid}]
actors:
  - name: p.s.Operator
    may: [p.s.OpenAccount, p.s.OpenFormation, p.s.OpenEnvironment, p.s.CreateSecret]
commands:
  - name: p.s.OpenAccount
    input:
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: p.s.Account
        instance: account_id
        sets: {title: input.title}
        emits: [p.s.AccountOpened]
        payload: {p.s.AccountOpened: {account_id: {generated: true}}}
  - name: p.s.OpenFormation
    input:
      - {name: account_id, type: Uuid}
    outcomes:
      - name: no-such-account
        when_related: {via: input.account_id, exists: false}
        error: p.s.ScopeNotFound
      - name: opened
        creates: p.s.Formation
        instance: formation_id
        sets: {account_id: input.account_id}
        emits: [p.s.FormationOpened]
        payload: {p.s.FormationOpened: {formation_id: {generated: true}}}
  - name: p.s.OpenEnvironment
    input:
      - {name: formation_id, type: Uuid}
    outcomes:
      - name: no-such-formation
        when_related: {via: input.formation_id, exists: false}
        error: p.s.ScopeNotFound
      - name: opened
        creates: p.s.Environment
        instance: environment_id
        sets: {formation_id: input.formation_id}
        emits: [p.s.EnvironmentOpened]
        payload: {p.s.EnvironmentOpened: {environment_id: {generated: true}}}
  - name: p.s.CreateSecret
    input:
      - {name: account_id, type: Uuid}
      - {name: scope, type: p.s.Scope}
      - {name: formation_id, type: Optional<Uuid>}
      - {name: environment_id, type: Optional<Uuid>}
      - {name: name, type: String}
    outcomes:
      - name: account-scope-with-formation
        when: {all: [scope == Account, {formation_id: {defined: true}}]}
        error: p.s.ScopeMismatch
      - name: account-scope-with-environment
        when: {all: [scope == Account, {environment_id: {defined: true}}]}
        error: p.s.ScopeMismatch
      - name: formation-scope-without-formation
        when: {all: [scope == Formation, {formation_id: {defined: false}}]}
        error: p.s.ScopeMismatch
      - name: formation-scope-with-environment
        when: {all: [scope == Formation, {environment_id: {defined: true}}]}
        error: p.s.ScopeMismatch
      - name: environment-scope-without-formation
        when: {all: [scope == Environment, {formation_id: {defined: false}}]}
        error: p.s.ScopeMismatch
      - name: environment-scope-without-environment
        when: {all: [scope == Environment, {environment_id: {defined: false}}]}
        error: p.s.ScopeMismatch
      - name: no-such-account
        when_related: {via: input.account_id, exists: false}
        error: p.s.ScopeNotFound
      - name: no-such-formation
        when_related: {via: input.formation_id, exists: false}
        error: p.s.ScopeNotFound
      - name: no-such-environment
        when_related: {via: input.environment_id, exists: false}
        error: p.s.ScopeNotFound
      - name: formation-of-other-account
        when_related: {via: input.formation_id, predicate: account_id != input.account_id}
        error: p.s.ScopeNotFound
      - name: environment-of-other-formation
        when_related: {via: input.environment_id, predicate: formation_id != input.formation_id}
        error: p.s.ScopeNotFound
      - name: name-taken
        when_related:
          entity: p.s.Secret
          where:
            all:
              - account_id == input.account_id
              - scope == input.scope
              - formation_id == input.formation_id
              - environment_id == input.environment_id
              - name == input.name
          exists: true
        error: p.s.NameTaken
      - name: created
        creates: p.s.Secret
        instance: secret_id
        sets:
          account_id: input.account_id
          scope: input.scope
          formation_id: input.formation_id
          environment_id: input.environment_id
          name: input.name
        emits: [p.s.SecretCreated]
        payload: {p.s.SecretCreated: {secret_id: {generated: true}}}
views:
  - name: p.s.Secrets
    source: p.s.Secret
    consistency: read_your_writes
    fields:
      - {name: secret_id, type: Uuid}
      - {name: name, type: String}
";

/// The row set of [`SECRETS`], as it writes it.
const NAME_TAKEN: &str = "      - name: name-taken
        when_related:
          entity: p.s.Secret
          where:
            all:
              - account_id == input.account_id
              - scope == input.scope
              - formation_id == input.formation_id
              - environment_id == input.environment_id
              - name == input.name
          exists: true
        error: p.s.NameTaken
";

const NO_SUCH_ENVIRONMENT: &str = "p.s.CreateSecret/outcome/no-such-environment";
const SECRET_CREATED: &str = "p.s.CreateSecret/outcome/created";

#[test]
fn no_missing_row_send_is_one_an_earlier_scope_guard_answers() {
    let run = run(SECRETS);
    assert_passes(&run, NO_SUCH_ENVIRONMENT);
    for (command, input, outcome) in sends(&run, NO_SUCH_ENVIRONMENT) {
        if command == "p.s.CreateSecret" {
            assert_eq!(outcome, "no-such-environment", "{input:#}");
            assert!(
                input.get("formation_id").is_some(),
                "an Environment-scoped send names its formation: {input:#}"
            );
        }
    }
    assert_every_scenario_passes(&run);
}

/// The send of [`SECRET_CREATED`] in `run` leaves out both Optional references.
fn assert_created_leaves_the_scopes_out(run: &Run) {
    assert_passes(run, SECRET_CREATED);
    let sent = sends(run, SECRET_CREATED);
    let (command, input, outcome) = sent.last().expect("the scenario sends");
    assert_eq!(
        (command.as_str(), outcome.as_str()),
        ("p.s.CreateSecret", "created"),
        "{sent:#?}"
    );
    for field in ["formation_id", "environment_id"] {
        assert!(input.get(field).is_none(), "{field} is left out: {input:#}");
    }
}

#[test]
fn the_creating_branch_leaves_out_the_references_its_scope_leaves_absent() {
    let run = run(SECRETS);
    assert_created_leaves_the_scopes_out(&run);
}

#[test]
fn without_the_row_set_the_creating_branch_leaves_them_out_too() {
    let run = run(&without(SECRETS, NAME_TAKEN));
    assert_created_leaves_the_scopes_out(&run);
    assert_every_scenario_passes(&run);
}
