//! Adversary pass 1 for beyond10x/ess#354: the loader driven from the unit's own schema note.
//!
//! `schemas/ui/ess-ui.schema.yaml` (`CompositeNode`, written by this unit): "A nested composite
//! with its own `reads` ... takes `live` as a section does ... `live` on a node that reads nothing
//! is refused". The `live` field note: "only on a member with `reads`". A `choice` declares
//! `reads` (the rows its options come from), so a choice with `reads` is a member with `reads`.

const DOCUMENT: &str = r"
format: ess-ui/1
app: console
model: console.system
placement_profile: fat
shells:
  app: {regions: {main: {kind: page_outlet}}}
navigation:
  home: board
  sections: [{name: all, pages: [board]}]
channels:
  people:
    carries: {events: [people.PersonAdded]}
    direction: server_to_client
    delivery: every_event
    resume: refetch
pages:
  board:
    kind: detail_page
    title: Board
    sections:
      - name: objective
        component: record
        reads: {view: objectives.ById, key: objective_id}
        fields: [goal]
        item:
          - name: owner
            component: choice
            reads: {view: people.All}
            live: {channel: people, effect: refetch}
";

/// The choice reads `people.All`; the loader refuses its `live` saying the node "reads
/// nothing", because `Composite::reads` answers `None` for every choice.
#[test]
fn a_choice_with_its_own_reads_takes_live() {
    let without = DOCUMENT.replace("            live: {channel: people, effect: refetch}\n", "");
    assert_ne!(without, DOCUMENT);
    if let Err(error) = ess_ui::load_str(&without) {
        panic!("control: the page loads without the choice's `live`: {error}");
    }
    if let Err(error) = ess_ui::load_str(DOCUMENT) {
        panic!(
            "a choice with its own `reads` is a nested composite with its own `reads`, so it \
             takes `live`; refused instead: {error}"
        );
    }
}
