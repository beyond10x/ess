//! `existing_instance: true` on a creating command whose creation sits behind guards on a related
//! catalogue row: the create-or-refuse scenario first creates the row through the guarded branch,
//! so the related row has to be arranged for the input that creation is sent.
//!
//! Two shapes of the same need, each with a key guard and a revision guard:
//!
//! * **through an input** (`when_related: {via: input.item_id, …}`, ess/18): the item is named by
//!   its identity, `unknown-item` is its `exists: false` and `stale-revision` compares its stored
//!   revision with the input's. A revision guard alone is not declarable there (a predicate needs
//!   its `exists: false` sibling), so only the key-only variant is.
//! * **as a row set** (`when_related: {entity, where, exists: false}`, ess/27): the item is found
//!   by a key field, and each guard refuses where no row matches.
//!
//! Before the fix the creation was arranged with no row (row set) or with a row searched apart from
//! the input the creation is sent (through an input), and the existence scenario was refused with
//! ESS-SYNTH-004 "… which no input reaches" or ESS-SYNTH-001 "… do not decide", although the
//! creating branch's own scenario reaches it.
//!
//! The fixtures are inline: a YAML file anywhere in the repository is a row of
//! `synthesis_suite_bytes_table`.
use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::synthesize::Synthesis;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use sha2::{Digest, Sha256};

/// Through an input: both guards.
const VIA: &str = r"format: ess/27
system: demo
version: v1
domain: demo.shop
types:
  - {name: demo.shop.ItemId, kind: newtype, of: Uuid}
  - {name: demo.shop.SelectionId, kind: newtype, of: String}
entities:
  - name: demo.shop.Item
    identity: {name: item_id, type: demo.shop.ItemId}
    fields:
      - {name: revision, type: Integer}
    lifecycle: {initial: Listed, states: [Listed], terminal: [Listed]}
  - name: demo.shop.Selection
    identity: {name: selection_id, type: demo.shop.SelectionId}
    fields:
      - {name: item_id, type: demo.shop.ItemId}
      - {name: revision, type: Integer}
    lifecycle: {initial: Selected, states: [Selected], terminal: [Selected]}
events:
  - name: demo.shop.ItemListed
    fields:
      - {name: item_id, type: demo.shop.ItemId}
  - name: demo.shop.ItemSelected
    fields:
      - {name: selection_id, type: demo.shop.SelectionId}
errors:
  - {name: demo.shop.SelectionExists, summary: The selection id is taken., fields: []}
  - {name: demo.shop.UnknownItem, summary: No such item., fields: []}
  - {name: demo.shop.StaleRevision, summary: The item has another revision., fields: []}
actors:
  - {name: demo.shop.Clerk, may: [demo.shop.List, demo.shop.Select]}
commands:
  - name: demo.shop.List
    input:
      - {name: revision, type: Integer}
    outcomes:
      - name: listed
        creates: demo.shop.Item
        instance: item_id
        emits: [demo.shop.ItemListed]
        payload:
          demo.shop.ItemListed: {item_id: {generated: true}}
        sets:
          revision: input.revision
  - name: demo.shop.Select
    input:
      - {name: selection_id, type: demo.shop.SelectionId}
      - {name: item_id, type: demo.shop.ItemId}
      - {name: revision, type: Integer}
    outcomes:
      - {name: selection-exists, existing_instance: true, error: demo.shop.SelectionExists}
      - name: unknown-item
        when_related: {via: input.item_id, exists: false}
        error: demo.shop.UnknownItem
      - name: stale-revision
        when_related: {via: input.item_id, predicate: revision != input.revision}
        error: demo.shop.StaleRevision
      - name: selected
        creates: demo.shop.Selection
        instance: selection_id
        emits: [demo.shop.ItemSelected]
        payload:
          demo.shop.ItemSelected: {selection_id: input.selection_id}
        sets:
          item_id: input.item_id
          revision: input.revision
views:
  - name: demo.shop.Items
    source: demo.shop.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: demo.shop.ItemId}
      - {name: revision, type: Integer}
  - name: demo.shop.Selections
    source: demo.shop.Selection
    consistency: read_your_writes
    fields:
      - {name: selection_id, type: demo.shop.SelectionId}
      - {name: item_id, type: demo.shop.ItemId}
      - {name: revision, type: Integer}
";

/// As a row set: both guards.
const SET: &str = r"format: ess/27
system: demo
version: v1
domain: demo.shop
types:
  - {name: demo.shop.ItemId, kind: newtype, of: Uuid}
  - {name: demo.shop.SelectionId, kind: newtype, of: String}
entities:
  - name: demo.shop.Item
    identity: {name: item_id, type: demo.shop.ItemId}
    fields:
      - {name: sku, type: String}
      - {name: revision, type: Integer}
    lifecycle: {initial: Listed, states: [Listed], terminal: [Listed]}
  - name: demo.shop.Selection
    identity: {name: selection_id, type: demo.shop.SelectionId}
    fields:
      - {name: sku, type: String}
      - {name: revision, type: Integer}
    lifecycle: {initial: Selected, states: [Selected], terminal: [Selected]}
events:
  - name: demo.shop.ItemListed
    fields:
      - {name: item_id, type: demo.shop.ItemId}
  - name: demo.shop.ItemSelected
    fields:
      - {name: selection_id, type: demo.shop.SelectionId}
errors:
  - {name: demo.shop.SelectionExists, summary: The selection id is taken., fields: []}
  - {name: demo.shop.UnknownItem, summary: No item carries the sku., fields: []}
  - {name: demo.shop.StaleRevision, summary: No item carries the sku at the revision., fields: []}
actors:
  - {name: demo.shop.Clerk, may: [demo.shop.List, demo.shop.Select]}
commands:
  - name: demo.shop.List
    input:
      - {name: sku, type: String}
      - {name: revision, type: Integer}
    outcomes:
      - name: listed
        creates: demo.shop.Item
        instance: item_id
        emits: [demo.shop.ItemListed]
        payload:
          demo.shop.ItemListed: {item_id: {generated: true}}
        sets:
          sku: input.sku
          revision: input.revision
  - name: demo.shop.Select
    input:
      - {name: selection_id, type: demo.shop.SelectionId}
      - {name: sku, type: String}
      - {name: revision, type: Integer}
    outcomes:
      - {name: selection-exists, existing_instance: true, error: demo.shop.SelectionExists}
      - name: unknown-item
        when_related:
          entity: demo.shop.Item
          where: sku == input.sku
          exists: false
        error: demo.shop.UnknownItem
      - name: stale-revision
        when_related:
          entity: demo.shop.Item
          where: {all: [sku == input.sku, revision == input.revision]}
          exists: false
        error: demo.shop.StaleRevision
      - name: selected
        creates: demo.shop.Selection
        instance: selection_id
        emits: [demo.shop.ItemSelected]
        payload:
          demo.shop.ItemSelected: {selection_id: input.selection_id}
        sets:
          sku: input.sku
          revision: input.revision
views:
  - name: demo.shop.Items
    source: demo.shop.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: demo.shop.ItemId}
      - {name: sku, type: String}
      - {name: revision, type: Integer}
  - name: demo.shop.Selections
    source: demo.shop.Selection
    consistency: read_your_writes
    fields:
      - {name: selection_id, type: demo.shop.SelectionId}
      - {name: sku, type: String}
      - {name: revision, type: Integer}
";

const EXISTS: &str =
    "      - {name: selection-exists, existing_instance: true, error: demo.shop.SelectionExists}\n";

const VIA_REVISION: &str = "      - name: stale-revision
        when_related: {via: input.item_id, predicate: revision != input.revision}
        error: demo.shop.StaleRevision
";

const SET_KEY: &str = "      - name: unknown-item
        when_related:
          entity: demo.shop.Item
          where: sku == input.sku
          exists: false
        error: demo.shop.UnknownItem
";

const SET_REVISION: &str = "      - name: stale-revision
        when_related:
          entity: demo.shop.Item
          where: {all: [sku == input.sku, revision == input.revision]}
          exists: false
        error: demo.shop.StaleRevision
";

const SELECTION_EXISTS: &str = "demo.shop.Select/outcome/selection-exists";

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the text");
    out
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| {
            let scenario = refusal
                .scenario
                .as_ref()
                .map_or_else(String::new, ToString::to_string);
            format!("{scenario} {}: {}", refusal.cause.code(), refusal.cause)
        })
        .collect()
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn run(model: EssIr, synthesis: &Synthesis) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect()
}

/// The model synthesizes with no refusal, its existence scenario among the rest, and every
/// scenario passes against the interpreter.
fn witnessed(label: &str, text: &str) {
    let model = ir(text);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(
        refusals(&synthesis),
        Vec::<String>::new(),
        "{label}: refusals"
    );
    assert!(
        ids(&synthesis).iter().any(|id| id == SELECTION_EXISTS),
        "{label}: no `{SELECTION_EXISTS}` in {:?}",
        ids(&synthesis)
    );
    let statuses = run(model, &synthesis);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(
        failing.is_empty(),
        "{label}: not passed against interpreted: {failing:?}"
    );
    assert_eq!(
        statuses.get(SELECTION_EXISTS),
        Some(&Status::Passed),
        "{label}: the existence scenario ran"
    );
}

#[test]
fn existing_instance_beside_related_guards_is_witnessed() {
    witnessed("through an input", VIA);
    witnessed("as a row set", SET);
}

#[test]
fn existing_instance_beside_one_related_guard_is_witnessed() {
    witnessed(
        "through an input, key only",
        &replaced(VIA, VIA_REVISION, ""),
    );
    witnessed("as a row set, key only", &replaced(SET, SET_REVISION, ""));
    witnessed("as a row set, revision only", &replaced(SET, SET_KEY, ""));
}
/// The item's revision is derived from its creator's input, so no row the declared commands create
/// is steered to a revision the existence scenario's creation was sent for an input chosen first.
/// The creating branch's own scenario reaches it, though, and the existence scenario's first call
/// is that run (beyond10x/ess#733): it is witnessed, and passes.
#[test]
fn a_derived_related_value_is_witnessed_by_the_creating_branchs_own_run() {
    let derived = replaced(
        &replaced(
            VIA,
            "  - name: demo.shop.List\n    input:\n      - {name: revision, type: Integer}\n",
            "  - name: demo.shop.List\n    input:\n      - {name: base, type: Integer}\n",
        ),
        "        sets:\n          revision: input.revision\n",
        "        sets:\n          revision: {expr: \"input.base + 1\"}\n",
    );
    let model = ir(&derived);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(refusals(&synthesis), Vec::<String>::new(), "refusals");
    for id in ["demo.shop.Select/outcome/selected", SELECTION_EXISTS] {
        assert!(
            ids(&synthesis).iter().any(|found| found == id),
            "`{id}` is in {:?}",
            ids(&synthesis)
        );
    }
    let statuses = run(model, &synthesis);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(
        failing.is_empty(),
        "not passed against interpreted: {failing:?}"
    );
}

fn digest(synthesis: &Synthesis) -> String {
    let json = synthesis
        .suite
        .to_canonical_json()
        .unwrap_or_else(|error| panic!("{error:?}"));
    let mut hex = String::new();
    for byte in &Sha256::digest(json.as_bytes()) {
        write!(hex, "{byte:02x}").unwrap();
    }
    format!("{hex}:{} refusals={:?}", json.len(), refusals(synthesis))
}

/// The suites these shapes synthesized before the change, byte for byte: without the existence
/// outcome, and through an input with the key guard alone, which synthesized already. Pinned on the
/// base commit.
#[test]
fn synthesis_bytes_unchanged_without_existing_instance() {
    let cases = [
        ("through an input, no existence", replaced(VIA, EXISTS, "")),
        ("as a row set, no existence", replaced(SET, EXISTS, "")),
        (
            "through an input, key only",
            replaced(VIA, VIA_REVISION, ""),
        ),
    ];
    let pinned = [PINNED_VIA_PLAIN, PINNED_SET_PLAIN, PINNED_VIA_KEY];
    let moved: Vec<String> = cases
        .iter()
        .zip(pinned)
        .filter_map(|((label, text), pinned)| {
            let found = digest(&ess_conformance::synthesize::synthesize(&ir(text)));
            (found != pinned).then(|| format!("{label}: {found}"))
        })
        .collect();
    assert!(moved.is_empty(), "moved from the base pins: {moved:#?}");
}

const PINNED_VIA_PLAIN: &str =
    "de4694fafa68b7879d9a3c515d7dd405b247e2a562391c5ffc690ddffd71ec08:15845 refusals=[]";
const PINNED_SET_PLAIN: &str =
    "521625cfbfc43ca2c48b5247e025f70ad6ce2103e3a3259335de8c76d34d56ec:19352 refusals=[]";
const PINNED_VIA_KEY: &str =
    "a6800efae8efc61f072aea29c87eb5da476f2d7a3f4af1055848c4a4b4f96416:16941 refusals=[]";

/// A decision confirms another one, and creates only after guards on that related row pass:
/// `existing_instance:` beside `when_related:` through an input (beyond10x/ess#733). The guards
/// refuse a missing row, a row of another item and a row not `Stale`; `ExpireDecision` makes the
/// `Stale` row, so the creating branch's own scenario sends `ConfirmDecision` after it.
const CONFIRM: &str = "format: ess/35
system: example
version: v1
domain: example.review
types:
  - {name: example.review.DecisionId, kind: newtype, of: Uuid}
entities:
  - name: example.review.Decision
    identity: {name: decision_id, type: example.review.DecisionId}
    fields:
      - {name: item, type: String}
    lifecycle:
      initial: Current
      states: [Current, Stale]
      terminal: [Stale]
      transitions:
        - {name: expire, from: [Current], to: Stale}
events:
  - name: example.review.DecisionRecorded
    fields:
      - {name: decision_id, type: example.review.DecisionId}
  - name: example.review.DecisionExpired
    fields:
      - {name: decision_id, type: example.review.DecisionId}
  - name: example.review.DecisionConfirmed
    fields:
      - {name: decision_id, type: example.review.DecisionId}
errors:
  - {name: example.review.AlreadyExists, fields: []}
  - {name: example.review.Missing, fields: []}
  - {name: example.review.OtherItem, fields: []}
  - {name: example.review.NotStale, fields: []}
  - {name: example.review.NotCurrent, fields: []}
commands:
  - name: example.review.RecordDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
      - {name: item, type: String}
    outcomes:
      - name: recorded
        creates: example.review.Decision
        instance: decision_id
        sets: {item: input.item}
        emits: [example.review.DecisionRecorded]
        payload:
          example.review.DecisionRecorded: {decision_id: input.decision_id}
  - name: example.review.ExpireDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
    outcomes:
      - {name: wrong-state, when_subject_state: [Stale], error: example.review.NotCurrent}
      - name: expired
        moves: example.review.Decision.expire
        instance: decision_id
        emits: [example.review.DecisionExpired]
        payload:
          example.review.DecisionExpired: {decision_id: input.decision_id}
  - name: example.review.ConfirmDecision
    input:
      - {name: decision_id, type: example.review.DecisionId}
      - {name: confirms, type: example.review.DecisionId}
      - {name: item, type: String}
    outcomes:
      - {name: already-exists, existing_instance: true, error: example.review.AlreadyExists}
      - {name: missing, when_related: {via: input.confirms, exists: false}, error: example.review.Missing}
      - {name: other-item, when_related: {via: input.confirms, predicate: item != input.item}, error: example.review.OtherItem}
      - {name: not-stale, when_related: {via: input.confirms, predicate: state != Stale}, error: example.review.NotStale}
      - name: confirmed
        creates: example.review.Decision
        instance: decision_id
        sets: {item: input.item}
        emits: [example.review.DecisionConfirmed]
        payload:
          example.review.DecisionConfirmed: {decision_id: input.decision_id}
views:
  - name: example.review.Decisions
    source: example.review.Decision
    consistency: read_your_writes
    fields:
      - {name: decision_id, type: example.review.DecisionId}
      - {name: item, type: String}
      - {name: state, type: example.review.Decision.State}
";

/// A selection is made against a catalogue and rests in `Selected`, where the entity starts in
/// `Draft`: `selected` creates `into: Selected`. Its two guards on the catalogue's revision refuse
/// a row whatever the revision is, so no row of the catalogue lets the creation pass, and the
/// creating branch is reached by no run. `Project`, the one creator of the catalogue, already sets
/// the revision the guards compare from its own input.
const PICK: &str = "format: ess/35
system: demo
version: v1
domain: demo.pick
types:
  - {name: demo.pick.CatalogueId, kind: newtype, of: Uuid}
  - {name: demo.pick.SelectionId, kind: newtype, of: Uuid}
entities:
  - name: demo.pick.Catalogue
    identity: {name: catalogue_id, type: demo.pick.CatalogueId}
    fields:
      - {name: revision, type: Integer}
    lifecycle: {initial: Projected, states: [Projected], terminal: [Projected]}
  - name: demo.pick.Selection
    identity: {name: selection_id, type: demo.pick.SelectionId}
    fields:
      - {name: revision, type: Integer}
    lifecycle:
      initial: Draft
      states: [Draft, Selected]
      terminal: [Selected]
      transitions:
        - {name: settle, from: [Draft], to: Selected}
errors:
  - {name: demo.pick.SelectionExists, fields: []}
  - {name: demo.pick.CatalogueNotFound, fields: []}
  - {name: demo.pick.RevisionMismatch, fields: []}
  - {name: demo.pick.RevisionMatch, fields: []}
  - {name: demo.pick.AlreadySelected, fields: []}
events:
  - name: demo.pick.CatalogueProjected
    fields: [{name: catalogue_id, type: demo.pick.CatalogueId}]
  - name: demo.pick.SelectionMade
    fields: [{name: selection_id, type: demo.pick.SelectionId}]
  - name: demo.pick.SelectionSettled
    fields: [{name: selection_id, type: demo.pick.SelectionId}]
commands:
  - name: demo.pick.Project
    input:
      - {name: catalogue_id, type: demo.pick.CatalogueId}
      - {name: revision, type: Integer}
    outcomes:
      - name: projected
        creates: demo.pick.Catalogue
        instance: catalogue_id
        sets: {revision: input.revision}
        emits: [demo.pick.CatalogueProjected]
        payload:
          demo.pick.CatalogueProjected: {catalogue_id: input.catalogue_id}
  - name: demo.pick.Settle
    input:
      - {name: selection_id, type: demo.pick.SelectionId}
    outcomes:
      - {name: wrong-state, when_subject_state: [Selected], error: demo.pick.AlreadySelected}
      - name: settled
        moves: demo.pick.Selection.settle
        instance: selection_id
        emits: [demo.pick.SelectionSettled]
        payload:
          demo.pick.SelectionSettled: {selection_id: input.selection_id}
  - name: demo.pick.Select
    input:
      - {name: selection_id, type: demo.pick.SelectionId}
      - {name: catalogue_id, type: demo.pick.CatalogueId}
      - {name: revision, type: Integer}
    outcomes:
      - {name: selection-exists, existing_instance: true, error: demo.pick.SelectionExists}
      - {name: catalogue-unknown, when_related: {via: input.catalogue_id, exists: false}, error: demo.pick.CatalogueNotFound}
      - {name: revision-mismatch, when_related: {via: input.catalogue_id, predicate: revision != input.revision}, error: demo.pick.RevisionMismatch}
      - {name: revision-match, when_related: {via: input.catalogue_id, predicate: revision == input.revision}, error: demo.pick.RevisionMatch}
      - name: selected
        creates: demo.pick.Selection
        instance: selection_id
        into: Selected
        sets: {revision: input.revision}
        emits: [demo.pick.SelectionMade]
        payload:
          demo.pick.SelectionMade: {selection_id: input.selection_id}
views:
  - name: demo.pick.Selections
    source: demo.pick.Selection
    consistency: read_your_writes
    fields:
      - {name: selection_id, type: demo.pick.SelectionId}
      - {name: revision, type: Integer}
";

const CONFIRM_EXISTS: &str = "example.review.ConfirmDecision/outcome/already-exists";
const PICK_EXISTS: &str = "demo.pick.Select/outcome/selection-exists";

/// The names of the commands the scenario `scenario` sends, in order.
fn executed(synthesis: &Synthesis, scenario: &str) -> Vec<String> {
    use ess_conformance::scenario::ScenarioStep;
    let (_, found) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == scenario)
        .unwrap_or_else(|| panic!("no `{scenario}` in {:?}", ids(synthesis)));
    found
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, .. } => Some(command.to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn existence_reuses_the_creating_branchs_own_run() {
    let model = ir(CONFIRM);
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    assert_eq!(refusals(&synthesis), Vec::<String>::new(), "refusals");
    let calls = executed(&synthesis, CONFIRM_EXISTS);
    let first = calls
        .iter()
        .position(|name| name == "example.review.ConfirmDecision")
        .expect("the scenario sends ConfirmDecision");
    let record = calls[..first]
        .iter()
        .position(|name| name == "example.review.RecordDecision")
        .unwrap_or_else(|| panic!("a RecordDecision comes first: {calls:?}"));
    let expire = calls[..first]
        .iter()
        .position(|name| name == "example.review.ExpireDecision")
        .unwrap_or_else(|| panic!("an ExpireDecision comes first: {calls:?}"));
    assert!(record < expire, "{calls:?}");
    assert_eq!(
        calls
            .iter()
            .filter(|name| *name == "example.review.ConfirmDecision")
            .count(),
        2,
        "the same identity is sent twice: {calls:?}"
    );
    let statuses = run(model, &synthesis);
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(
        failing.is_empty(),
        "not passed against interpreted: {failing:?}"
    );
    assert_eq!(statuses.get(CONFIRM_EXISTS), Some(&Status::Passed));
}

/// The refusal of the existence scenario of `PICK`.
fn pick_refusal() -> (String, String, &'static str) {
    let synthesis = ess_conformance::synthesize::synthesize(&ir(PICK));
    let Some(refusal) = synthesis.refusals.iter().find(|refusal| {
        refusal
            .scenario
            .as_ref()
            .is_some_and(|id| id.to_string() == PICK_EXISTS)
    }) else {
        panic!("a refusal of `{PICK_EXISTS}`: {:?}", refusals(&synthesis));
    };
    (
        refusal.cause.code().to_string(),
        refusal.cause.to_string(),
        refusal.hint(),
    )
}

#[test]
fn existence_refusal_names_the_created_state() {
    let (code, text, _) = pick_refusal();
    assert_eq!(code, "ESS-SYNTH-004", "{text}");
    assert!(text.contains("resting in `Selected`"), "{text}");
    assert!(!text.contains("resting in `Draft`"), "{text}");
}

#[test]
fn existence_refusal_help_names_no_fix_already_present() {
    let (_, _, help) = pick_refusal();
    assert!(
        !help.contains("sets: <field>: input.<field>"),
        "asks for what `Project` declares: {help}"
    );
    assert!(help.contains("authored scenario"), "{help}");
}

#[test]
fn existence_refusal_names_the_guards_in_the_way() {
    let (_, text, _) = pick_refusal();
    for guard in [
        "demo.pick.Select/revision-mismatch",
        "demo.pick.Select/revision-match",
    ] {
        assert!(
            text.contains(&format!("`{guard}`")),
            "names `{guard}`: {text}"
        );
    }
}
