//! beyond10x/ess#320: the generated collection keys its rows by the read's `key` — in a section
//! without a channel, in an overlay, and as the default `live.match` of a section with one.

use std::path::Path;

fn page(document: &str) -> String {
    let document =
        ess_ui::load_str(document).unwrap_or_else(|error| panic!("the document loads: {error}"));
    let files = ess_ui_react::render(&document, Path::new("."))
        .unwrap_or_else(|error| panic!("the project renders: {error}"));
    files
        .files
        .get("src/pages/Objectives.tsx")
        .unwrap_or_else(|| panic!("the page is generated; have {:?}", files.files.keys()))
        .clone()
}

fn document(section: &str) -> String {
    format!(
        "format: ess-ui/1
app: factory
model: factory.system
placement_profile: fat
shells:
  app:
    regions:
      main: {{kind: page_outlet}}
      overlay: {{kind: overlay_outlet}}
navigation:
  home: objectives
  sections: [{{name: all, pages: [objectives]}}]
channels:
  objectives:
    carries: {{events: [factory.ObjectiveChanged]}}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  objectives:
    kind: detail_page
    title: Objectives
    overlays:
      pick: {{kind: drawer, component: collection, reads: {{view: factory.Picks, key: pick_id}}, columns: [title]}}
    sections:
      - name: objectives
        component: collection
        reads: {{view: factory.Objectives, key: objective_id}}
        columns: [title]
{section}"
    )
}

/// Every `<Collection …/>` element of a generated file.
fn collections(page: &str) -> Vec<&str> {
    page.match_indices("<Collection")
        .map(|(start, _)| {
            let end = page[start..]
                .find("/>")
                .map_or(page.len(), |end| start + end);
            &page[start..end]
        })
        .collect()
}

#[test]
fn a_section_without_a_channel_keys_its_rows_by_the_read_key() {
    let page = page(&document(""));
    let elements = collections(&page);
    assert!(
        elements
            .iter()
            .any(|element| element.contains("rowKey=\"objective_id\"")
                || element.contains("rowKey={\"objective_id\"}")),
        "the section's collection is not keyed by `objective_id`:\n{elements:#?}"
    );
}

#[test]
fn an_overlay_collection_keys_its_rows_by_its_read_key() {
    let page = page(&document(""));
    let elements = collections(&page);
    assert!(
        elements
            .iter()
            .any(|element| element.contains("rowKey=\"pick_id\"")
                || element.contains("rowKey={\"pick_id\"}")),
        "the overlay's collection is not keyed by `pick_id`:\n{elements:#?}"
    );
}

#[test]
fn a_live_block_without_match_matches_events_by_the_read_key() {
    let page = page(&document(
        "        live: {channel: objectives, effect: patch_row}\n",
    ));
    let live = page
        .lines()
        .find(|line| line.contains("useLive("))
        .unwrap_or_else(|| panic!("the section applies its channel:\n{page}"));
    assert!(
        live.contains("match: \"objective_id\""),
        "the live block does not match events by `objective_id`: {live}"
    );
}
