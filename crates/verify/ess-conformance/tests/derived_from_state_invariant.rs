//! A view field derived from the lifecycle state, written the way the model already allows: a
//! stored field, written by `sets:` on every branch that changes its value, with invariants over
//! `state` stating which value each state holds. The interpreter checks the mapping after every
//! branch, so a branch that enters a state without writing the mapped value is a broken invariant.
//! The guide section that teaches the idiom is read here too, and its model is the one compiled.
use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::execute::{execute, Externals, Store, Undetermined};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;
use std::collections::BTreeMap;
use std::path::PathBuf;

const SYSTEM: &str = "\
format: ess/22
system: desk
version: v1
domains: [desk.tickets]
";

/// Everything but the entity and its view, which the guide shows.
const COMMANDS: &str = "\
domain: desk.tickets
events:
  - {name: desk.tickets.Changed, fields: [{name: ticket_id, type: Uuid}]}
errors:
  - {name: desk.tickets.NotOpen, summary: The ticket cannot move from its state.}
commands:
  - name: desk.tickets.OpenTicket
    input: [{name: ticket_id, type: Uuid}]
    outcomes:
      - name: opened
        creates: desk.tickets.Ticket
        instance: ticket_id
        sets: {live: true}
        emits: [desk.tickets.Changed]
        payload: {desk.tickets.Changed: {ticket_id: input.ticket_id}}
  - name: desk.tickets.WaitTicket
    input: [{name: ticket_id, type: Uuid}]
    outcomes:
      - name: waiting
        moves: desk.tickets.Ticket.wait
        instance: ticket_id
        emits: [desk.tickets.Changed]
        payload: {desk.tickets.Changed: {ticket_id: input.ticket_id}}
      - {name: wrong-state, wrong_state: true, error: desk.tickets.NotOpen}
  - name: desk.tickets.CloseTicket
    input: [{name: ticket_id, type: Uuid}]
    outcomes:
      - name: closed
        moves: desk.tickets.Ticket.close
        instance: ticket_id
        sets: {live: false}
        emits: [desk.tickets.Changed]
        payload: {desk.tickets.Changed: {ticket_id: input.ticket_id}}
      - {name: wrong-state, wrong_state: true, error: desk.tickets.NotOpen}
";

/// The entity and view the guide section shows, byte for byte.
const TICKET: &str = "\
domain: desk.tickets
entities:
  - name: desk.tickets.Ticket
    identity: {name: ticket_id, type: Uuid}
    fields: [{name: live, type: Boolean}]          # the stored field
    invariants:                                    # one value per state, checked after every branch
      - {any: [state != Closed, live == false]}
      - {any: [state == Closed, live == true]}
    lifecycle:
      initial: Open
      states: [Open, Waiting, Closed]
      terminal: [Closed]
      transitions:
        - {name: wait, from: [Open], to: Waiting}
        - {name: close, from: [Open, Waiting], to: Closed}
views:
  - name: desk.tickets.Status
    source: desk.tickets.Ticket
    consistency: read_your_writes
    fields: [{name: ticket_id, type: Uuid}, {name: live, type: Boolean}]
";

/// The close branch as an author who forgot the mapping would write it.
fn forgetful() -> String {
    let drifted = COMMANDS.replacen("        sets: {live: false}\n", "", 1);
    assert_ne!(drifted, COMMANDS, "the fixture writes `live` on close");
    drifted
}

fn model(commands: &str, ticket: &str) -> Result<EssIr, String> {
    let files = [
        ("system.yaml", SYSTEM),
        ("commands.yaml", commands),
        ("ticket.yaml", ticket),
    ];
    let mut sources = SourceMap::new();
    let mut parsed = Vec::new();
    for (label, text) in files {
        sources.insert(label.to_owned(), text.to_owned());
        parsed.push((
            Source::new(label),
            RawSpecFile::parse(text).map_err(|error| format!("{label}: {error}"))?,
        ));
    }
    let specification = Specification::assemble(parsed).map_err(|errors| errors.to_string())?;
    compile(&specification, &sources).map_err(|diagnostics| {
        diagnostics
            .as_slice()
            .iter()
            .map(|diagnostic| format!("{} {}", diagnostic.code, diagnostic.message))
            .collect::<Vec<_>>()
            .join("\n")
    })
}

const TICKET_ID: &str = "6f1c1d6e-6a3b-4c8e-9f1e-2b7d4a1c0e01";

fn invoke(ir: &EssIr, store: &mut Store, command: &str) -> Result<(), Undetermined> {
    let input = BTreeMap::from([("ticket_id".to_owned(), Node::Text(TICKET_ID.into()))]);
    let mut steps = execute(
        ir,
        store,
        &format!("desk.tickets.{command}").parse().unwrap(),
        &input,
        &Externals::Withheld,
    )?;
    assert_eq!(steps.len(), 1, "{command} has one possible step");
    *store = steps.remove(0).next;
    Ok(())
}

#[test]
fn invariant_over_state_catches_a_forgotten_set() {
    let faithful = model(COMMANDS, TICKET).expect("the faithful model compiles");
    let mut store = Store::default();
    invoke(&faithful, &mut store, "OpenTicket").expect("opening holds the mapping");
    invoke(&faithful, &mut store, "WaitTicket").expect("waiting keeps `live` true");
    invoke(&faithful, &mut store, "CloseTicket").expect("closing writes `live: false`");

    let drifted = model(&forgetful(), TICKET).expect("the drifted model also validates");
    let mut store = Store::default();
    invoke(&drifted, &mut store, "OpenTicket").expect("opening holds the mapping");
    let before = store.clone();
    match invoke(&drifted, &mut store, "CloseTicket") {
        Err(Undetermined::BrokenInvariant {
            instance,
            invariant,
        }) => {
            assert!(
                instance.contains("desk.tickets.Ticket"),
                "the broken instance is the ticket: {instance}"
            );
            assert!(
                invariant.contains("state") && invariant.contains("Closed"),
                "the report names the invariant over `state`: {invariant}"
            );
        }
        other => panic!("a close that forgets `live` is a broken invariant, got {other:?}"),
    }
    assert_eq!(store, before, "the broken step changed nothing");
}

/// The `## ` section of `page` titled `heading`, up to the next `## ` heading outside a fence.
fn section<'a>(page: &'a str, heading: &str) -> Option<&'a str> {
    let start = page.find(&format!("\n{heading}\n"))? + 1;
    let mut fenced = false;
    let mut offset = start;
    for line in page[start..].split_inclusive('\n') {
        if line.starts_with("```") {
            fenced = !fenced;
        }
        if offset > start && !fenced && line.starts_with("## ") {
            return Some(&page[start..offset]);
        }
        offset += line.len();
    }
    Some(&page[start..])
}

#[test]
fn lifecycle_derived_field_guide_section_states_the_idiom() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .find(|p| p.join("website/docs").is_dir())
        .expect("the workspace holds website/docs")
        .join("website/docs/guides/specify/values-and-views.md");
    let page = std::fs::read_to_string(&path).expect("values-and-views.md");
    let heading = "## A view field derived from the lifecycle state";
    let section = section(&page, heading).unwrap_or_else(|| panic!("missing heading `{heading}`"));
    for phrase in [
        "stored field",
        "`sets:`",
        "invariants over `state`",
        "projects the field",
        "every state some branch enters",
    ] {
        assert!(section.contains(phrase), "`{heading}` is missing {phrase}");
    }
    let fence = section
        .split("```yaml\n")
        .nth(1)
        .and_then(|rest| rest.split("```").next())
        .unwrap_or_else(|| panic!("`{heading}` shows no ```yaml model"));
    assert_eq!(
        fence, TICKET,
        "the section's model is the one this file compiles"
    );
    model(COMMANDS, fence).expect("the section's model validates");
}
