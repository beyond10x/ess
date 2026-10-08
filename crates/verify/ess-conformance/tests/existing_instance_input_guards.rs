//! An input-guarded refusal on a creating command that also declares `existing_instance: true` is
//! witnessed, not refused (beyond10x/ess#479).
//!
//! The issue's `catalog` reproducer: `CreateItem` refuses an id its guard calls invalid
//! (`invalid-item-id`), creates an item (`created`), and refuses an id an item already has
//! (`already-created`). On 0.55.0 every guard form the issue lists was refused with ESS-SYNTH-019,
//! "a step requires `invalid-item-id` for `{item_id: item_id-1048595, title: title}`, which its
//! own guard (`item_id == ""`) refutes": the half of the refusal's scenario that sends the refused
//! input for a stored row replaced the input's id with the stored row's, which the guard refutes.
//! Without `already-created` the refusal was witnessed with `item_id: ""`; without a view of `Item`
//! both refusals were refused with ESS-SYNTH-001 instead.

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{ConformanceScenario, ScenarioStep, ScenarioValue};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

/// The issue's `system.yaml`.
const SYSTEM: &str = "format: ess/22
system: catalog
version: v1

domains:
  - catalog.items
";

/// The issue's `domains/items.yaml`, its one guard line replaced by `{GUARD}`.
const DOMAIN: &str = r"domain: catalog.items

summary: Items, each kept under an id of 1 to 170 UTF-8 bytes.

naming:
  wire: items
  display: Items

types:
  - name: catalog.items.ItemId
    kind: newtype
    of: String

entities:
  - name: catalog.items.Item
    identity:
      name: item_id
      type: catalog.items.ItemId
    fields:
      - name: title
        type: String
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
      transitions: []

errors:
  - name: catalog.items.ItemExists
    summary: An item already has this id.
    fields:
      - name: item_id
        type: catalog.items.ItemId

  - name: catalog.items.InvalidItemId
    summary: The item id is empty or longer than 170 UTF-8 bytes.
    fields:
      - name: item_id
        type: catalog.items.ItemId

commands:
  - name: catalog.items.CreateItem
    naming:
      wire: create-item
      display: Create an item
    input:
      - name: item_id
        type: catalog.items.ItemId
      - name: title
        type: String
    outcomes:
      - name: invalid-item-id
        when:
          any:
{GUARD}
        error: catalog.items.InvalidItemId
        payload:
          catalog.items.InvalidItemId:
            item_id: input.item_id
        summary: The id cannot be kept, so nothing was created.
      - name: created
        creates: catalog.items.Item
        instance: item_id
        sets:
          title: input.title
        emits:
          - catalog.items.ItemCreated
        payload:
          catalog.items.ItemCreated:
            item_id: input.item_id
            title: input.title
      - name: already-created
        existing_instance: true
        error: catalog.items.ItemExists
        payload:
          catalog.items.ItemExists:
            item_id: input.item_id
        summary: An item already has this id.

events:
  - name: catalog.items.ItemCreated
    fields:
      - name: item_id
        type: catalog.items.ItemId
      - name: title
        type: String

views:
  - name: catalog.items.Items
    source: catalog.items.Item
    consistency: read_your_writes
    fields:
      - name: item_id
        type: catalog.items.ItemId
      - name: state
        type: catalog.items.Item.State
      - name: title
        type: String
    naming:
      wire: items
      display: Items
";

/// The `already-created` branch, which the issue's control removes.
const ALREADY_CREATED: &str = "      - name: already-created
        existing_instance: true
        error: catalog.items.ItemExists
        payload:
          catalog.items.ItemExists:
            item_id: input.item_id
        summary: An item already has this id.
";

const INVALID: &str = "catalog.items.CreateItem/outcome/invalid-item-id";
const ALREADY: &str = "catalog.items.CreateItem/outcome/already-created";

/// One guard form from the issue's table: the members of its `any:`, and whether an id satisfies
/// it.
struct Form {
    any: &'static [&'static str],
    holds: fn(&str) -> bool,
}

impl Form {
    fn written(&self) -> String {
        self.any.join(" or ")
    }
}

fn empty(id: &str) -> bool {
    id.is_empty()
}

fn over_170_bytes(id: &str) -> bool {
    id.len() > 170
}

fn under_one_byte(id: &str) -> bool {
    id.is_empty()
}

fn over_170_characters(id: &str) -> bool {
    id.chars().count() > 170
}

fn empty_or_over_170_bytes(id: &str) -> bool {
    id.is_empty() || id.len() > 170
}

/// Every guard form the issue's table refuses on 0.55.0.
fn forms() -> Vec<Form> {
    vec![
        Form {
            any: &[r#"item_id == """#],
            holds: empty,
        },
        Form {
            any: &["item_id.utf8_bytes > 170"],
            holds: over_170_bytes,
        },
        Form {
            any: &["item_id.utf8_bytes < 1"],
            holds: under_one_byte,
        },
        Form {
            any: &["item_id.count > 170"],
            holds: over_170_characters,
        },
        Form {
            any: &[r#"item_id == """#, "item_id.utf8_bytes > 170"],
            holds: empty_or_over_170_bytes,
        },
    ]
}

/// The domain file with `any` as the refusal's guard.
fn domain(any: &[&str]) -> String {
    let lines: String = any
        .iter()
        .flat_map(|member| ["            - ", member, "\n"])
        .collect();
    let text = DOMAIN.replace("{GUARD}\n", &lines);
    assert!(!text.contains("{GUARD}"), "the guard is substituted");
    text
}

/// `text` with `from` removed, which must be there.
fn without(text: &str, from: &str) -> String {
    assert!(text.contains(from), "the reproducer holds the part removed");
    text.replace(from, "")
}

/// `text` with everything from `views:` on removed: no view of `Item`.
fn without_views(text: &str) -> String {
    let at = text
        .find("\nviews:\n")
        .expect("the reproducer declares a view");
    text[..=at].to_owned()
}

fn ir(domain: &str) -> EssIr {
    let spec = Specification::assemble([
        (
            Source::new("system.yaml"),
            RawSpecFile::parse(SYSTEM).unwrap_or_else(|error| panic!("{error}")),
        ),
        (
            Source::new("domains/items.yaml"),
            RawSpecFile::parse(domain).unwrap_or_else(|error| panic!("{error}")),
        ),
    ])
    .unwrap_or_else(|errors| panic!("the reproducer validates:\n{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis.refusals.iter().map(ToString::to_string).collect()
}

fn scenario<'s>(synthesis: &'s Synthesis, id: &str) -> Option<&'s ConformanceScenario> {
    synthesis
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
        .map(|(_, scenario)| scenario)
}

/// One `CreateItem` step: the branch the next step requires, and the literal `item_id` and `title`
/// it sends (`None` where the value is not a literal).
#[derive(Debug)]
struct Send {
    outcome: String,
    item_id: Option<String>,
    title: Option<String>,
}

fn text_at(input: &BTreeMap<String, ScenarioValue>, field: &str) -> Option<String> {
    match input.get(field).and_then(|value| value.as_literal()) {
        Some(Node::Text(text)) => Some(text.clone()),
        _ => None,
    }
}

/// Every `CreateItem` step of `scenario` followed by the outcome it requires, in order.
fn sends(scenario: &ConformanceScenario) -> Vec<Send> {
    let mut found = Vec::new();
    let mut steps = scenario.steps.iter().peekable();
    while let Some(step) = steps.next() {
        let ScenarioStep::ExecuteCommand { command, input, .. } = step else {
            continue;
        };
        if command.name().to_string() != "catalog.items.CreateItem" {
            continue;
        }
        let Some(ScenarioStep::ExpectOutcome { outcome }) = steps.peek() else {
            continue;
        };
        found.push(Send {
            outcome: outcome.outcome.to_string(),
            item_id: text_at(input, "item_id"),
            title: text_at(input, "title"),
        });
    }
    found
}

/// The sends of `scenario` requiring `invalid-item-id`, each checked to satisfy `holds` by its id.
fn refused_sends_hold(form: &Form, scenario: &ConformanceScenario) {
    let refused: Vec<Send> = sends(scenario)
        .into_iter()
        .filter(|send| send.outcome == "invalid-item-id")
        .collect();
    assert!(
        !refused.is_empty(),
        "{}: the scenario sends `CreateItem` requiring `invalid-item-id`",
        form.written()
    );
    for send in &refused {
        let id = send
            .item_id
            .as_deref()
            .unwrap_or_else(|| panic!("{}: a literal id: {send:?}", form.written()));
        assert!(
            (form.holds)(id),
            "{}: `{id}` is sent requiring `invalid-item-id`, so it satisfies the guard",
            form.written()
        );
    }
}

#[test]
fn every_guard_form_witnesses_the_refusal_beside_an_existing_instance_branch() {
    for form in forms() {
        let synthesis = synthesize(&ir(&domain(form.any)));
        let refused = refusals(&synthesis);
        assert!(
            refused.is_empty(),
            "{}: nothing is refused, as nothing is without `already-created`:\n{}",
            form.written(),
            refused.join("")
        );
        let invalid = scenario(&synthesis, INVALID)
            .unwrap_or_else(|| panic!("{}: `{INVALID}` is synthesized", form.written()));
        refused_sends_hold(&form, invalid);
        assert!(
            scenario(&synthesis, ALREADY).is_some(),
            "{}: `{ALREADY}` keeps its scenario",
            form.written()
        );
    }
}

/// The issue's control, which 0.55.0 already passed: without `already-created` the refusal is sent
/// `item_id: ""`.
#[test]
fn without_the_existing_instance_branch_the_refusal_is_sent_an_empty_id() {
    let forms = forms();
    let form = &forms[0];
    let synthesis = synthesize(&ir(&without(&domain(form.any), ALREADY_CREATED)));
    let refused = refusals(&synthesis);
    assert!(refused.is_empty(), "{}", refused.join(""));
    let invalid = scenario(&synthesis, INVALID).expect("the refusal is synthesized");
    refused_sends_hold(form, invalid);
}

/// Without a view of `Item` only `already-created` needs one to show the stored row unchanged; the
/// input refusal, whose guard no stored id satisfies, is witnessed by its own send (0.55.0 refused
/// both with ESS-SYNTH-001).
#[test]
fn without_a_view_only_the_existence_refusal_is_refused() {
    for form in forms() {
        let synthesis = synthesize(&ir(&without_views(&domain(form.any))));
        let about: Vec<String> = synthesis
            .refusals
            .iter()
            .map(|refusal| {
                refusal
                    .scenario
                    .as_ref()
                    .map_or_else(String::new, ToString::to_string)
            })
            .collect();
        assert_eq!(
            about,
            vec![ALREADY.to_owned()],
            "{}: only the existence refusal is refused:\n{}",
            form.written(),
            refusals(&synthesis).join("")
        );
        let invalid = scenario(&synthesis, INVALID)
            .unwrap_or_else(|| panic!("{}: `{INVALID}` is synthesized", form.written()));
        refused_sends_hold(&form, invalid);
    }
}

/// A guard that reads another input as well keeps the half that sends the refused input for a
/// stored row: the stored row's id, with `title: ""` (0.55.0 refused it with ESS-SYNTH-019, the
/// half sending `{item_id: item_id-1048595, title: title}`).
#[test]
fn a_guard_reading_another_input_keeps_its_stored_row_half() {
    let synthesis = synthesize(&ir(&domain(&[r#"item_id == """#, r#"title == """#])));
    let refused = refusals(&synthesis);
    assert!(
        refused.is_empty(),
        "nothing is refused:\n{}",
        refused.join("")
    );
    let invalid = scenario(&synthesis, INVALID).expect("the refusal is synthesized");
    let sent = sends(invalid);
    let created: Vec<&str> = sent
        .iter()
        .filter(|send| send.outcome == "created")
        .filter_map(|send| send.item_id.as_deref())
        .collect();
    let stored_half = sent.iter().find(|send| {
        send.outcome == "invalid-item-id"
            && send
                .item_id
                .as_deref()
                .is_some_and(|id| created.contains(&id))
    });
    let stored_half =
        stored_half.unwrap_or_else(|| panic!("a refused send names a stored id: {sent:#?}"));
    assert_eq!(
        stored_half.title.as_deref(),
        Some(""),
        "the stored id satisfies no guard member, so the title does: {sent:#?}"
    );
    for send in sent.iter().filter(|send| send.outcome == "invalid-item-id") {
        let holds = send.item_id.as_deref() == Some("") || send.title.as_deref() == Some("");
        assert!(holds, "every refused send satisfies the guard: {send:?}");
    }
}
