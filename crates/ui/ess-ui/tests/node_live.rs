//! beyond10x/ess#354: a nested node that reads — a tab's node, a header metric, a record's item —
//! takes a `live` block with the same fields and effects as a section's, and a header title can
//! read a field of the record a named section of the page holds (`header.title_from`).
//!
//! `header.live` keeps its one meaning: the channels whose connection state the header shows.

use ess_ui::{Body, Composite, Document, Effect, Node, TabForm};

const DOCUMENT: &str = r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: objective
  sections: [{name: all, pages: [objective]}]
channels:
  progress:
    carries: {events: [objectives.EvidenceAdded, objectives.GoalChanged]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  objective:
    kind: detail_page
    title: Objective
    header:
      title: Objective
      title_from: {section: objective, field: goal}
      live: [progress]
      metrics:
        - name: evidence_count
          component: metric
          reads: {view: objectives.Evidence}
          aggregate: count
          label: Evidence
          live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}
    sections:
      - name: objective
        component: record
        reads: {view: objectives.ById, key: objective_id}
        live: {channel: progress, on: [objectives.GoalChanged], effect: patch_row}
        fields: [goal]
        tabs:
          - name: evidence
            label: Evidence
            form:
              name: evidence
              component: collection
              reads: {view: objectives.Evidence, key: evidence_id}
              columns: [note]
              live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}
          - name: cost
            label: Cost
            form:
              name: cost
              component: metric
              reads: {view: objectives.Cost}
              aggregate: sum
              field: amount
              label: Cost
              live: {channel: progress, effect: refetch}
";

fn document(text: &str) -> Document {
    ess_ui::load_str(text).unwrap_or_else(|error| panic!("the document loads: {error}"))
}

fn refused(text: &str) -> String {
    match ess_ui::load_str(text) {
        Ok(_) => panic!("the document loads, and it must be refused"),
        Err(error) => error.to_string(),
    }
}

fn tab_node<'a>(document: &'a Document, tab: &str) -> &'a Node {
    let section = document.pages["objective"]
        .sections
        .iter()
        .find(|section| section.name == "objective")
        .expect("the objective section");
    let Body::Composite(Composite::Record(record)) = &section.body else {
        panic!("the objective section is a record");
    };
    let found = record
        .tabs
        .iter()
        .find(|candidate| candidate.name == tab)
        .expect("the tab exists");
    match &found.form {
        Some(TabForm::Node(node)) => node,
        other => panic!("the tab holds a node, not {other:?}"),
    }
}

#[test]
fn a_reading_node_in_a_tab_loads_its_live_block() {
    let document = document(DOCUMENT);
    let evidence = tab_node(&document, "evidence");
    let live = evidence
        .live
        .as_ref()
        .expect("the tab's collection is live");
    assert_eq!(live.channel, "progress");
    assert_eq!(live.effect, Effect::InsertTop);
    assert_eq!(live.on, vec!["objectives.EvidenceAdded".to_owned()]);
    // `match` absent: the node's read's `key`, as for a section (#320).
    assert_eq!(live.match_field.as_deref(), Some("evidence_id"));
    let cost = tab_node(&document, "cost");
    assert_eq!(
        cost.live.as_ref().map(|live| &live.effect),
        Some(&Effect::Refetch)
    );
    assert_eq!(cost.live.as_ref().unwrap().match_field, None);
}

#[test]
fn a_header_metric_loads_its_live_block_and_header_live_keeps_its_meaning() {
    let document = document(DOCUMENT);
    let header = document.pages["objective"]
        .header
        .as_ref()
        .expect("a header");
    let metric = &header.metrics[0];
    let live = metric.live.as_ref().expect("the header metric is live");
    assert_eq!(live.channel, "progress");
    assert_eq!(live.effect, Effect::InsertTop);
    assert_eq!(
        header.live,
        vec!["progress".to_owned()],
        "header.live still lists the channels whose state the header shows"
    );
}

#[test]
fn a_header_title_from_names_a_section_and_a_field() {
    let document = document(DOCUMENT);
    let header = document.pages["objective"]
        .header
        .as_ref()
        .expect("a header");
    let from = header.title_from.as_ref().expect("title_from loads");
    assert_eq!(from.section, "objective");
    assert_eq!(from.field, "goal");
    assert_eq!(
        header.title.as_deref(),
        Some("Objective"),
        "the literal title stays the text shown until the record holds the field"
    );
}

#[test]
fn a_node_without_live_loads_with_none() {
    let text = DOCUMENT.replace(
        "\n              live: {channel: progress, effect: refetch}",
        "",
    );
    let document = document(&text);
    assert!(tab_node(&document, "cost").live.is_none());
}

#[test]
fn live_on_a_node_that_reads_nothing_is_refused() {
    let text = DOCUMENT.replace(
        "              component: metric\n              reads: {view: objectives.Cost}\n              aggregate: sum\n              field: amount\n",
        "              component: metric\n              from: channel.progress.total\n",
    );
    let message = refused(&text);
    assert!(
        message.contains("`live`") && message.contains("reads nothing"),
        "{message}"
    );
}

#[test]
fn live_on_a_primitive_or_widget_is_refused() {
    let text = DOCUMENT.replace(
        "    sections:\n      - name: objective\n",
        "    sections:\n      - name: note\n        component: record\n        reads: {view: objectives.ById}\n        item:\n          - {name: hint, primitive: text, text: hello, live: {channel: progress, effect: refetch}}\n      - name: objective\n",
    );
    let message = refused(&text);
    assert!(
        message.contains("`live`") && message.contains("reads nothing"),
        "{message}"
    );
}

#[test]
fn when_paged_away_on_a_node_is_refused() {
    let text = DOCUMENT.replace(
        "live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top}\n          - name: cost",
        "live: {channel: progress, on: [objectives.EvidenceAdded], effect: insert_top, when_paged_away: ignore}\n          - name: cost",
    );
    let message = refused(&text);
    assert!(message.contains("when_paged_away"), "{message}");
}

#[test]
fn an_unknown_title_from_key_is_refused() {
    let text = DOCUMENT.replace(
        "title_from: {section: objective, field: goal}",
        "title_from: {section: objective, field: goal, view: objectives.ById}",
    );
    let message = refused(&text);
    assert!(message.contains("view"), "{message}");
}
